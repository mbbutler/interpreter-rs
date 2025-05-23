use std::fmt::Display;

use super::{scanner::Token, value::Value};

#[derive(Debug, Clone)]
pub enum Expr {
    Assign {
        id: usize,
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: Token,
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Set {
        object: Box<Expr>,
        name: Token,
        value: Box<Expr>,
    },
    Super {
        id: usize,
        keyword: Token,
        method: Token,
    },
    This {
        id: usize,
        keyword: Token,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable {
        id: usize,
        name: Token,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign { id: _, name, value } => write!(f, "{} = {value}", name.lexeme),
            Self::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
            Self::Call {
                callee,
                paren: _,
                arguments,
            } => write!(
                f,
                "{callee}({})",
                arguments
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Get { object, name } => write!(f, "{object}.{}", &name.lexeme),
            Self::Grouping(expr) => write!(f, "(group {expr})"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Logical {
                left,
                operator,
                right,
            } => write!(f, "{left} {} {right}", operator.lexeme),
            Self::Set {
                object,
                name,
                value,
            } => write!(f, "{object}.{} = {value}", &name.lexeme),
            Self::Super {
                id: _,
                keyword: _,
                method,
            } => write!(f, "super.{}", method.lexeme),
            Self::This { id: _, keyword } => write!(f, "{}", &keyword.lexeme),
            Self::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Self::Variable { id: _, name } => write!(f, "{}", name.lexeme),
        }
    }
}

#[cfg(test)]
mod expr_tests {
    use crate::lox::{
        scanner::{Token, TokenType},
        value::Value,
    };

    use super::Expr;

    #[test]
    fn prettyish_print() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token {
                    t_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    // col: 0,
                    line: 0,
                },
                right: Box::new(Expr::Literal(Value::Number(123.0))),
            }),
            operator: Token {
                t_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
                // col: 0,
                line: 0,
            },
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Value::Number(
                45.67,
            ))))),
        };

        assert_eq!(expr.to_string(), "(* (- 123) (group 45.67))".to_string())
    }
}
