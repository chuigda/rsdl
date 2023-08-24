use std::collections::HashMap;
use pest::iterators::{Pair, Pairs};
use smallvec::SmallVec;

use crate::parser::hir::{
    AttrItem,
    RSDLType,
    SumType,
    TypeConstructor,
    TypeDef,
    TypeDefInner
};
use crate::parser::pest_parser::Rule;


pub fn treeconv(file_name: &str, mut tree: Pairs<Rule>, global_attr: &mut Vec<AttrItem>, defs: &mut Vec<TypeDef>) {
    let rsdl_program = tree
        .next()
        .unwrap();

    assert_eq!(rsdl_program.as_rule(), Rule::rsdl_program);

    for rsdl_item in rsdl_program.into_inner() {
        match rsdl_item.as_rule() {
            Rule::type_def => defs.push(tydeconv(file_name, rsdl_item)),
            Rule::global_attr => global_attr.push(attrconv(rsdl_item)),
            Rule::EOI => {},
            _ => unreachable!()
        }
    }
}

fn tydeconv(file_name: &str, ty: Pair<Rule>) -> TypeDef {
    let mut attr: SmallVec<[AttrItem; 2]> = SmallVec::new();

    for tyde_item in ty.into_inner() {
        match tyde_item.as_rule() {
            Rule::attr => { attr.push(attrconv(tyde_item)); },
            Rule::type_alias => { return convtypealias(file_name, attr, tyde_item); },
            Rule::sum_type => { return convsumtype(file_name, attr, tyde_item); }
            Rule::type_ctor => {
                let ctor = ctorconv(tyde_item);
                return TypeDef {
                    file: file_name.to_string(),
                    attr,
                    inner: TypeDefInner::SimpleType(ctor)
                }
            },
            _ => {
                dbg!(tyde_item);
                unreachable!()
            }
        }
    }

    unreachable!()
}

fn attrconv(attr: Pair<Rule>) -> AttrItem {
    let attr_item = attr.into_inner().next().unwrap();
    assert_eq!(attr_item.as_rule(), Rule::attr_item);

    let attr_item_inner = attr_item.into_inner().next().unwrap();
    imp_attrconv(attr_item_inner)
}

fn imp_attrconv(inner: Pair<Rule>) -> AttrItem {
    match inner.as_rule() {
        Rule::identifier => AttrItem::Identifier(inner.as_str().to_string()),
        Rule::string => AttrItem::String(strchkconv(inner)),
        Rule::assign_attr => {
            let mut iter = inner.into_inner();
            let identifier = identchkconv(iter.next().unwrap());
            let assigned = imp_attrconv(iter.next().unwrap());

            AttrItem::Assignment(identifier, Box::new(assigned))
        },
        Rule::call_attr => {
            let mut iter = inner.into_inner();
            let identifier = identchkconv(iter.next().unwrap());
            let attr_item_list = iter.next().unwrap();
            assert_eq!(attr_item_list.as_rule(), Rule::attr_item_list);
            let call_args = attr_item_list
                .into_inner()
                .map(|attr_item| {
                    let attr_item_inner = attr_item
                        .into_inner()
                        .next()
                        .unwrap();
                    imp_attrconv(attr_item_inner)
                })
                .collect::<Vec<_>>();

            AttrItem::CallAlike(identifier, call_args)
        },
        _ => unreachable!()
    }
}

fn convtypealias(
    file_name: &str,
    attr: SmallVec<[AttrItem; 2]>,
    alias: Pair<Rule>
) -> TypeDef {
    let mut iter = alias.into_inner();
    let identifier = identchkconv(iter.next().unwrap());
    let rsdl_type = convrsdltype(iter.next().unwrap());

    TypeDef {
        file: file_name.to_string(),
        attr,
        inner: TypeDefInner::AliasType(identifier, rsdl_type)
    }
}

fn convsumtype(
    file_name: &str,
    attr: SmallVec<[AttrItem; 2]>,
    sumtype: Pair<Rule>
) -> TypeDef {
    let mut iter = sumtype.into_inner();
    let name = identchkconv(iter.next().unwrap());

    let mut scalar_variants = Vec::new();
    let mut ctors = Vec::new();
    let variant_list = iter.next().unwrap();
    assert_eq!(variant_list.as_rule(), Rule::variant_list);

    for variant in variant_list.into_inner() {
        variantconv(variant, &mut scalar_variants, &mut ctors);
    }

    TypeDef {
        file: file_name.to_string(),
        attr,
        inner: TypeDefInner::SumType(SumType {
            name,
            scalar_variants,
            ctors
        })
    }
}

fn variantconv(
    variant: Pair<Rule>,
    scalar_variants: &mut Vec<(SmallVec<[AttrItem; 2]>, String)>,
    ctors: &mut Vec<(SmallVec<[AttrItem; 2]>, TypeConstructor)>
) {
    let iter = variant.into_inner();
    let mut attr = SmallVec::new();

    for variant_item in iter {
        match variant_item.as_rule() {
            Rule::attr => { attr.push(attrconv(variant_item)); },
            Rule::identifier => {
                scalar_variants.push((
                    attr,
                    variant_item.as_str().to_string()
                ));
                return;
            },
            Rule::type_ctor => {
                ctors.push((
                    attr,
                    ctorconv(variant_item)
                ));
                return;
            },
            _ => unreachable!()
        }
    }
}

fn ctorconv(ctor: Pair<Rule>) -> TypeConstructor {
    let mut iter = ctor.into_inner();
    let name = identchkconv(iter.next().unwrap());
    let field_list = iter.next().unwrap();
    assert_eq!(field_list.as_rule(), Rule::field_list);

    let fields = field_list.into_inner().map(fldconv).collect::<Vec<_>>();

    TypeConstructor { name, fields }
}

fn fldconv(fld: Pair<Rule>) -> (SmallVec<[AttrItem; 2]>, bool, RSDLType, String) {
    let mut attr = SmallVec::new();
    let mut iter = fld.into_inner();

    let mut item = iter.next().unwrap();
    while item.as_rule() == Rule::attr {
        attr.push(attrconv(item));
        item = iter.next().unwrap();
    }

    let ident = identchkconv(item);

    let optional_mark = iter.next().unwrap();
    assert_eq!(optional_mark.as_rule(), Rule::optional_mark);
    let is_optional = optional_mark.as_span().as_str().trim() == "?";

    let rsdl_type = convrsdltype(iter.next().unwrap());

    (attr, is_optional, rsdl_type, ident)
}

fn convrsdltype(rsdl_type: Pair<Rule>) -> RSDLType {
    let mut cloned_iter = rsdl_type.clone().into_inner();
    let inner = cloned_iter.next().unwrap();
    
    match inner.as_rule() {
        Rule::identifier => RSDLType::Identifier(rsdl_type.as_str().to_string()),
        Rule::list_type => convrsdltype_list(inner),
        Rule::record_type => convrsdltype_record(inner),
        Rule::native_type => convrsdltype_native(inner),
        _ => unreachable!()
    }
}

fn convrsdltype_list(list_type: Pair<Rule>) -> RSDLType {
    let eltype = convrsdltype(list_type.into_inner().next().unwrap());
    RSDLType::List(Box::new(eltype))
}

fn convrsdltype_record(record_type: Pair<Rule>) -> RSDLType {
    let eltype = convrsdltype(record_type.into_inner().next().unwrap());
    RSDLType::Record(Box::new(eltype))
}

fn convrsdltype_native(native_type: Pair<Rule>) -> RSDLType {
    let mapping_seq = native_type.into_inner().next().unwrap();
    assert_eq!(mapping_seq.as_rule(), Rule::mapping_seq);

    let mut lang_tyname_map = HashMap::new();
    for mapping in mapping_seq.into_inner() {
        assert_eq!(mapping.as_rule(), Rule::mapping);

        let mut iter = mapping.into_inner();
        let lang = identchkconv(iter.next().unwrap());
        let tyname = strchkconv(iter.next().unwrap());

        lang_tyname_map.insert(lang, tyname);
    }

    RSDLType::Native(lang_tyname_map)
}

fn identchkconv(identifier: Pair<Rule>) -> String {
    assert_eq!(identifier.as_rule(), Rule::identifier);
    identifier.as_str().to_string()
}

fn strchkconv(string: Pair<Rule>) -> String {
    assert_eq!(string.as_rule(), Rule::string);

    let raw_string = string.into_inner().next().unwrap();
    assert_eq!(raw_string.as_rule(), Rule::raw_string);

    raw_string.as_str()
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
}
