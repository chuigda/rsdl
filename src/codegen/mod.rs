use crate::min_resolv::ResolveContext;
use crate::parser::hir::TypeDef;

pub trait CodeGenerator {
    type Error;

    fn lang_name() -> &'static str;
    fn reserved_idents() -> &'static [&'static str];
    fn supported_attr() -> &'static [&'static str];

    fn generate(
        &self,
        tyde: &TypeDef,
        ctx: &ResolveContext,
        output: &mut Vec<String>
    ) -> Result<(), Self::Error>;
}
