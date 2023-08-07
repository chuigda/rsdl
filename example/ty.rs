#[derive(Serialize, Deserialize, Debug, Clone)]
#[nonexhaustive]
pub enum Type {
    StringType,
    FuncType(FuncType),
    IntType(IntType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FuncType {
    pub params: Vec<Type>,
    pub ret: Vec<Type>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IntType {
    pub bits: i64,
}
