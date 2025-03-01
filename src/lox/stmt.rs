use std::fmt::Display;

use super::{expr::Expr, scanner::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expression(Expr),
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Print(Expr),
    Return {
        #[allow(unused)]
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Expr,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(stmts) => {
                for stmt in stmts {
                    writeln!(f, "{stmt:?}")?;
                }
                Ok(())
            }
            _ => write!(f, "{self:?}"),
        }
    }
}
