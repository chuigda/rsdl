/// A program is a list of statements
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "$kind")]
pub enum Stmt {
    /// if-then-else statement
    IfStmt(Box<IfStmt>),
    /// an expression is also sometimes considered a statement
    ExprStmt(Box<ExprStmt>),
    /// a block is a list of statements
    BlockStmt(Box<BlockStmt>),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct IfStmt {
    /// if condition
    pub cond: Expr,
    /// then statement
    pub then: Stmt,
    /// else statement, can be empty
    pub otherwise: Option<Stmt>,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

/// An expression is a literal, an identifier, or a binary expression
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct NumericLiteral {
    pub value: i64,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StringLiteral {
    pub value: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BoolLiteral {
    pub value: bool,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BinaryExpr {
    /// binary operator
    pub op: String,
    /// left operand
    pub left: Expr,
    /// right operand
    pub right: Expr,
}
