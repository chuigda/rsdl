use std::collections::HashMap;
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
    Int,
    Float,
    Str,
    Bool,
    Identifier(String),
    Native(HashMap<String, String>),
    List(Box<RSDLType>),
    Record(Box<RSDLType>)
}

#[derive(Debug, Clone)]
pub struct TypeConstructor {
    pub name: String,
    pub fields: Vec<(SmallVec<[AttrItem; 2]>, RSDLType, String)>
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
    SumType(SumType),
    SimpleType(TypeConstructor)
}

#[derive(Debug, Clone)]
pub struct TypeDef {
    pub attr: SmallVec<[AttrItem; 2]>,
    pub inner: TypeDefInner
}
