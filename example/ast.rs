/// A program is a list of statements
#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "$kind")]
pub enum Stmt {
    /// if-then-else statement
    IfStmt(Box<IfStmt>),
    /// an expression is also sometimes considered a statement
    ExprStmt(Box<ExprStmt>),
    /// a block is a list of statements
    BlockStmt(Box<BlockStmt>),
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct IfStmt {
    /// if condition
    pub cond: Expr,
    /// then statement
    pub then: Stmt,
    /// else statement, can be empty
    pub otherwise: Option<Stmt>,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct ExprStmt {
    pub expr: Expr,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

/// An expression is a literal, an identifier, or a binary expression
/// 
/// All I want is a room somewhere,
/// Far away from the cold night air,
/// With one enormous chair,
/// Aow, wouldn't it be loverly?
#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(tag = "$kind")]
pub enum Expr {
    /// numeric literal expression
    NumericLiteral(NumericLiteral),
    /// string literal expression
    StringLiteral(StringLiteral),
    /// boolean literal expression
    BoolLiteral(BoolLiteral),
    /// identifier expression
    Identifier(Identifier),
    /// binary expression
    BinaryExpr(Box<BinaryExpr>),
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct NumericLiteral {
    pub value: i64,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BoolLiteral {
    pub value: bool,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Identifier {
    pub name: String,
}

#[derive(Archive, Clone, Copy, Debug, Deserialize, Serialize)]
pub struct BinaryExpr {
    /// binary operator
    pub op: String,
    /// left operand
    pub left: Expr,
    /// right operand
    pub right: Expr,
}
