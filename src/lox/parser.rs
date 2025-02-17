use super::{
    error::ParseError,
    expr::Expr,
    scanner::{Token, TokenType},
    stmt::Stmt,
    value::Value,
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParseError>> {
        let mut statements = Vec::new();
        let mut errors = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(declaration) => statements.push(declaration),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(statements)
        } else {
            eprintln!("Parsed Stmts:");
            for stmt in statements {
                eprintln!("  {stmt}");
            }
            Err(errors)
        }
    }

    fn declaration(&mut self) -> ParserResult<Stmt> {
        if self.match_t_types(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name.")?;
        let initializer = if self.match_t_types(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Value::Nil)
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        match self.peek().t_type {
            TokenType::For => {
                self.advance();
                self.for_statement()
            }
            TokenType::If => {
                self.advance();
                self.if_statement()
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block()?))
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.")?;

        let initializer = if self.match_t_types(&[TokenType::Semicolon]) {
            None
        } else if self.match_t_types(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::Semicolon, "Expect ';' after loop condition.")?;

        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(&TokenType::RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        let condition = condition.unwrap_or_else(|| Expr::Literal(Value::Bool(true)));

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after condition.")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_t_types(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> ParserResult<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.assgnment()
    }

    fn assgnment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;
        if self.match_t_types(&[TokenType::Equal]) {
            let equals = self.previous().to_owned();
            let value = self.assgnment()?;
            if let Expr::Variable(name) = expr {
                Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                })
            } else {
                Err(ParseError::new(
                    equals,
                    "Invalid assignment target.".to_string(),
                ))
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> ParserResult<Expr> {
        let mut expr = self.and()?;
        while self.match_t_types(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = Box::new(self.and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expr> {
        let mut expr = self.equality()?;
        while self.match_t_types(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparison()?;
        while self.match_t_types(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().to_owned();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
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
                operator,
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
                operator,
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
                operator,
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
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        let next = self.advance();
        match next.t_type {
            TokenType::False => Ok(Expr::Literal(Value::Bool(false))),
            TokenType::True => Ok(Expr::Literal(Value::Bool(true))),
            TokenType::Nil => Ok(Expr::Literal(Value::Nil)),
            TokenType::Number | TokenType::String => Ok(Expr::Literal(
                self.previous().literal.as_ref().unwrap().to_owned(),
            )),
            TokenType::Identifier => Ok(Expr::Variable(self.previous().to_owned())),
            TokenType::LeftParen => {
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
            self.peek().t_type == *t_type
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
        self.peek().t_type == TokenType::Eof
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
