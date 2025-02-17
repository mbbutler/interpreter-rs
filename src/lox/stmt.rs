use super::{expr::Expr, scanner::Token};

#[derive(Debug)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression { expr: Expr },
    Print { expr: Expr },
    Var { name: Token, initializer: Expr },
}
