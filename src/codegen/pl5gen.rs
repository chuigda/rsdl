//! PL5 代码生成器

use std::error::Error;

use smallvec::SmallVec;

use crate::{
    codegen::{
        CodeGenerator,
        CodeGeneratorFactory,
        Doc
    },
    parser::hir::{
        SumType,
        RSDLType,
        AttrItem,
        TypeConstructor,
        check_boxed,
        check_ident_attr,
        check_inline,
        check_private,
        extract_doc_strings
    },
    min_resolv::ResolveContext
};

pub struct PL5Generator();

impl PL5Generator {
    fn type_to_string(&self, ctx: &ResolveContext, ty: &RSDLType) -> Option<String> {
        match ty {
            RSDLType::Identifier(ident) => {
                if let Some((_, rsdl_type, is_inline)) = ctx.known_types.get(ident) {
                    if *is_inline {
                        let Some(rsdl_type) = rsdl_type else { return None; };
                        self.type_to_string(ctx, rsdl_type)
                    } else {
                        Some(ident.to_string())
                    }
                } else {
                    None
                }
            },
            RSDLType::Native(native) => {
                if let Some(pl5_name) = native.get(self.lang_ident()) {
                    Some(pl5_name.to_string())
                } else {
                    None
                }
            },
            RSDLType::List(inner) => {
                let inner = self.type_to_string(ctx, &inner)?;
                Some("list".to_string())
            },
            RSDLType::Record(inner) => {
                None
            }
        }
    }

    fn gen_doc(
        &self,
        attr_list: &[AttrItem],
        doc_attr_name: &str,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        let doc_string_lines = extract_doc_strings(attr_list, doc_attr_name)?;
        if doc_string_lines.is_empty() {
            return Ok(());
        }

        for line in doc_string_lines {
            output.push_string(format!(";;; {}", line));
        }

        Ok(())
    }

    fn imp_visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type_attr: Option<&[AttrItem]>,
        type_ctor: &TypeConstructor,
        output: &mut Doc,

        doc_attr_name: &str,
        rust_attr_name: &str
    ) -> Result<(), Box<dyn Error>> {
        let ctor_name_dashed = self.underscore2dash(&type_ctor.name);

        self.gen_doc(attr, doc_attr_name, output)?;

        for (attr, _, _, name) in &type_ctor.fields {
            output.push_string(format!("- {}", self.underscore2dash(name)));
            let mut field_doc = Box::new(Doc::new(4));
            self.gen_doc(attr, "doc", &mut field_doc)?;
            if !field_doc.items.is_empty() {
                output.push_doc(field_doc);
            }
        }

        let mut maker_args = String::new();
        for (_, _, _, name) in &type_ctor.fields {
            maker_args.push_str(&self.underscore2dash(name));
            maker_args.push(' ');
        }
        output.push_string(format!("(define (make-{} {})", ctor_name_dashed, maker_args));
        let mut maker_body = Box::new(Doc::new(4));

        for (_, optional, ty, name) in &type_ctor.fields {
            let type_name = self.type_to_string(ctx, ty).ok_or_else(|| {
                format!("无法将类型 {:?} 转换为 PL5 类型", ty)
            })?;

            let name_dashed = self.underscore2dash(name);
            if *optional {
                maker_body.push_string(format!(
                    "(assert (or (null? {name_dashed}) ({type_name}? {name_dashed})) \"字段 {name_dashed} 的类型必须是 {type_name} 或者 null\")",
                ));
            } else {
                maker_body.push_string(format!(
                    "(assert ({type_name}? {name_dashed}) \"字段 {name_dashed} 的类型必须是 {type_name}\")",
                ));
            }
        }

        maker_body.push_string(format!("(struct 'k '{ctor_name_dashed}"));
        let mut struct_fields = Box::new(Doc::new(8));
        for (_, _, _, name) in &type_ctor.fields {
            let name_dashed = self.underscore2dash(name);
            struct_fields.push_string(format!(
                "'{name_dashed} {name_dashed}",
            ));
        }
        maker_body.push_doc(struct_fields);
        output.push_doc(maker_body);
        output.push_str("))");

        output.push_empty_line();
        Ok(())
    }

    fn underscore2dash(&self, s: &str) -> String {
        s.replace('_', "-")
    }
}

impl CodeGenerator for PL5Generator {
    fn generator_name(&self) -> &'static str { "PL5 struct 代码生成器" }

    fn lang_ident(&self) -> &'static str { "pl5" }

    fn reserved_idents(&self) -> &[&'static str] {
        &[
            "and",
            "or",
            "define",
            "lambda",
            "loop",
            "break"
        ]
    }

    fn visit_namespace_begin(
        &mut self,
        _namespace: &str,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        Err("PL5 不支持 namespace 功能".into())
    }

    fn visit_namespace_end(
        &mut self,
        _namespace: &str,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        unreachable!()
    }

    fn visit_type_alias(
        &mut self,
        _ctx: &ResolveContext,
        attr: &[AttrItem],
        _alias_name: &str,
        _target_type: &RSDLType,
        _output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        if check_inline(attr) {
            return Ok(());
        }

        Err("PL5 不支持类型别名".into())
    }

    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        self.imp_visit_simple_type(
            ctx,
            attr,
            None,
            type_ctor,
            output,
            "doc",
            "rust"
        )
    }

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &SumType,
        output: &mut Doc
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}

pub struct PL5GeneratorFactory();

impl CodeGeneratorFactory for PL5GeneratorFactory {
    fn generator_name(&self) -> &'static str {
        PL5Generator().generator_name()
    }

    fn lang_ident(&self) -> &'static str {
        PL5Generator().lang_ident()
    }

    fn create(&self) -> Box<dyn CodeGenerator> {
        Box::new(PL5Generator())
    }
}
