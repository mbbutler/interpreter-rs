use std::fmt::Display;

use super::scanner::{Literal, Token};

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
    Literal(Literal),
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Variable(Token),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign { name, value } => write!(f, "{} = {}", name.lexeme, value),
            Self::Binary {
                left,
                operator,
                right,
            } => write!(f, "({} {} {})", operator.lexeme, left, right),
            Self::Grouping(expr) => write!(f, "(group {expr})"),
            Self::Literal(literal) => write!(f, "{literal}"),
            Self::Unary { operator, right } => write!(f, "({} {})", operator.lexeme, right),
            Self::Variable(token) => write!(f, "{}", token.lexeme),
        }
    }
}

#[cfg(test)]
mod expr_tests {
    use crate::lox::scanner::{Literal, Token, TokenType};

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
                right: Box::new(Expr::Literal(Literal::Number(123.0))),
            }),
            operator: Token {
                t_type: TokenType::Star,
                lexeme: "*".to_string(),
                literal: None,
                // col: 0,
                line: 0,
            },
            right: Box::new(Expr::Grouping(Box::new(Expr::Literal(Literal::Number(
                45.67,
            ))))),
        };

        assert_eq!(expr.to_string(), "(* (- 123) (group 45.67))".to_string())
    }
}
