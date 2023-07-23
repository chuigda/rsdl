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
