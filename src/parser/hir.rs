use std::collections::HashMap;
use std::error::Error;
use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub enum AttrItem {
    Identifier(String),
    String(String),
    Assignment(String, Box<AttrItem>),
    CallAlike(String, Vec<AttrItem>)
}

#[derive(Debug, Clone)]
pub enum RSDLType {
    Identifier(String),
    Native(HashMap<String, String>),
    List(Box<RSDLType>),
    Record(Box<RSDLType>)
}

#[derive(Debug, Clone)]
pub struct TypeConstructor {
    pub name: String,
    pub fields: Vec<(SmallVec<[AttrItem; 2]>, bool, RSDLType, String)>
}

#[derive(Debug, Clone)]
pub struct SumType {
    pub name: String,
    pub scalar_variants: Vec<(SmallVec<[AttrItem; 2]>, String)>,
    pub ctors: Vec<(SmallVec<[AttrItem; 2]>, TypeConstructor)>
}

#[derive(Debug, Clone)]
pub enum TypeDefInner {
    AliasType(String, RSDLType),
    SimpleType(TypeConstructor),
    SumType(SumType)
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub file: String,
    pub attr: SmallVec<[AttrItem; 2]>,
    pub inner: TypeDefInner
}

pub fn check_ident_attr(attr_list: &[AttrItem], checked_ident: &str) -> bool {
    for attr in attr_list {
        if let AttrItem::Identifier(ident) = attr {
            if ident == checked_ident {
                return true;
            }
        }
    }
    false
}

pub fn check_private(attr_list: &[AttrItem]) -> bool {
    check_ident_attr(attr_list, "private")
}

pub fn check_boxed(attr_list: &[AttrItem]) -> bool {
    check_ident_attr(attr_list, "boxed")
}

pub fn extract_doc_strings(
    attr_list: &[AttrItem],
    doc_attr_name: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut ret = Vec::new();

    fn add_doc_string(output: &mut Vec<String>, doc_string: &str) {
        if doc_string.contains("\n") {
            for line in doc_string.split("\n") {
                output.push(line.trim().to_string());
            }
        } else {
            output.push(doc_string.trim().to_string());
        }
    }

    for attr in attr_list {
        match attr {
            AttrItem::CallAlike(fn_alike, param_alike) => {
                if fn_alike == doc_attr_name {
                    if param_alike.len() != 1 {
                        return Err(format!(
                            "{} 属性的参数数量必须为 1，但是此处有 {} 个参数",
                            doc_attr_name,
                            param_alike.len()
                        ).into());
                    }

                    if let AttrItem::String(doc) = &param_alike[0] {
                        add_doc_string(&mut ret, doc);
                    } else {
                        return Err(format!(
                            "{} 属性的参数必须是字符串字面量",
                            doc_attr_name
                        ).into());
                    }
                }
            },
            AttrItem::Assignment(assignee, value) => {
                if assignee == doc_attr_name {
                    if let AttrItem::String(doc) = value.as_ref() {
                        add_doc_string(&mut ret, &doc);
                    } else {
                        return Err(format!(
                            "{} 属性的值必须是字符串字面量",
                            doc_attr_name
                        ).into());
                    }
                }
            },
            _ => {}
        }
    }

    Ok(ret)
}
