use std::fmt::Display;

use super::{scanner::Token, value::Value};

#[derive(Debug)]
pub enum Expr {
    Assign {
        name: Token,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping(Box<Expr>),
    Literal(Value),
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign { name, value } => write!(f, "{} = {value}", name.lexeme),
            Self::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {left} {right})", operator.lexeme),
            Self::Grouping(expr) => write!(f, "(group {expr})"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Logical {
                left,
                operator,
                right,
            } => write!(f, "{left} {} {right}", operator.lexeme),
            Self::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme),
            Self::Variable(token) => write!(f, "{}", token.lexeme),
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
