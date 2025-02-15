use super::expr::Expr;

pub enum Stmt {
    Expression { expr: Expr },
    Print { expr: Expr },
}
