use std::error::Error;

use crate::{
    codegen::CodeGenerator,
    parser::hir::{
        RSDLType,
        AttrItem,
        TypeConstructor
    },
    min_resolv::ResolveContext
};


pub struct TSClassGenerator();

impl CodeGenerator for TSClassGenerator {
    fn generator_name(&self) -> &'static str {
        todo!()
    }

    fn lang_ident(&self) -> &'static str {
        todo!()
    }

    fn reserved_idents(&self) -> &[&'static str] {
        todo!()
    }

    fn visit_namespace_begin(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_namespace_end(
        &mut self,
        namespace: &str,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_type_alias(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        alias_name: &str,
        target_type: &RSDLType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_simple_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        type_ctor: &TypeConstructor,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_ctor(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        ctor: &TypeConstructor,
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type_scalar_variant(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        variant_name: &str,
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn visit_sum_type(
        &mut self,
        ctx: &ResolveContext,
        attr: &[AttrItem],
        sum_type: &crate::parser::hir::SumType,
        output: &mut Vec<String>
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
