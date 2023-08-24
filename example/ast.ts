/** A program is a list of statements */
export type Stmt = 
    | IfStmt
    | ExprStmt
    | BlockStmt

export interface StmtBase<K extends string> {
    $kind: K;
}

/** if-then-else statement */
export interface IfStmt extends StmtBase<"IfStmt"> {
    /** if condition */
    cond: Expr,
    /** then statement */
    then: Stmt,
    /** else statement, can be empty */
    otherwise?: Stmt,
}

/** an expression is also sometimes considered a statement */
export interface ExprStmt extends StmtBase<"ExprStmt"> {
    expr: Expr,
}

/** a block is a list of statements */
export interface BlockStmt extends StmtBase<"BlockStmt"> {
    stmts: Stmt[],
}

/**
 * An expression is a literal, an identifier, or a binary expression
 * 
 * All I want is a room somewhere,
 * Far away from the cold night air,
 * With one enormous chair,
 * Aow, wouldn't it be loverly?
 */
export type Expr = 
    | NumericLiteral
    | StringLiteral
    | BoolLiteral
    | Identifier
    | BinaryExpr

export interface ExprBase<K extends string> {
    $kind: K;
}

/** numeric literal expression */
export interface NumericLiteral extends ExprBase<"NumericLiteral"> {
    value: number,
}

/** string literal expression */
export interface StringLiteral extends ExprBase<"StringLiteral"> {
    value: string,
}

/** boolean literal expression */
export interface BoolLiteral extends ExprBase<"BoolLiteral"> {
    value: boolean,
}

/** identifier expression */
export interface Identifier extends ExprBase<"Identifier"> {
    name: string,
}

/** binary expression */
export interface BinaryExpr extends ExprBase<"BinaryExpr"> {
    /** binary operator */
    op: string,
    /** left operand */
    left: Expr,
    /** right operand */
    right: Expr,
}
