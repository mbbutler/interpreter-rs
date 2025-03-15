use std::fmt::Display;

use super::{expr::Expr, scanner::Token};

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Class {
        name: Token,
        methods: Vec<Function>,
        superclass: Option<Expr>,
    },
    Expression(Expr),
    Function(Function),
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
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>,
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
