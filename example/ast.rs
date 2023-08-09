#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "$kind")]
pub enum Stmt {
    IfStmt(Box<IfStmt>),
    ExprStmt(Box<ExprStmt>),
    BlockStmt(Box<BlockStmt>),
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct IfStmt {
    pub cond: Expr,
    pub then: Stmt,
    pub otherwise: Stmt,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}
