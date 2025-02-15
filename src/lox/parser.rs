use super::{
    error::ParseError,
    expr::Expr,
    scanner::{Literal, Token, TokenType},
    stmt::Stmt,
};

type ParserResult<T> = std::result::Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub(crate) fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> ParserResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.match_t_types(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print { expr })
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression { expr })
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparison()?;
        while self.match_t_types(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<Expr> {
        let mut expr = self.term()?;
        while self.match_t_types(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().to_owned();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn term(&mut self) -> ParserResult<Expr> {
        let mut expr = self.factor()?;
        while self.match_t_types(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().to_owned();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;
        while self.match_t_types(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operator,
                right: Box::new(right),
            };
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.match_t_types(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator: operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        let next = self.peek();
        match next.t_type {
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenType::Nil => {
                self.advance();
                Ok(Expr::Literal(Literal::Nil))
            }
            TokenType::Number | TokenType::String => {
                self.advance();
                Ok(Expr::Literal(
                    self.previous().literal.as_ref().unwrap().to_owned(),
                ))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => Err(ParseError::new(
                next.to_owned(),
                "Expect expression.".to_string(),
            )),
        }
    }

    fn match_t_types(&mut self, t_types: &[TokenType]) -> bool {
        for t_type in t_types {
            if self.check(t_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, t_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().t_type == t_type
        }
    }

    fn consume(&mut self, t_type: &TokenType, msg: &str) -> ParserResult<Token> {
        if self.check(t_type) {
            Ok(self.advance().to_owned())
        } else {
            Err(ParseError::new(self.peek().to_owned(), msg.to_string()))
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().t_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().t_type == TokenType::Semicolon {
                return;
            }
            match self.peek().t_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var
                | TokenType::While => return,
                _ => {
                    self.advance();
                }
            }
        }
    }
}
