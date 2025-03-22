use std::sync::atomic::{AtomicUsize, Ordering};

use super::{
    error::ParseError,
    expr::Expr,
    scanner::{Token, TokenType},
    stmt::{Function, Stmt},
    value::Value,
};

static NEXT_EXPR_ID: AtomicUsize = AtomicUsize::new(0);

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
        if self.match_t_types(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.match_t_types(&[TokenType::Fun]) {
            self.function("function")
        } else if self.match_t_types(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        }
    }

    fn class_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect class name.".to_string())?;
        let superclass = if self.match_t_types(&[TokenType::Less]) {
            self.consume(
                &TokenType::Identifier,
                "Expect superclass name.".to_string(),
            )?;
            Some(Expr::Variable {
                id: NEXT_EXPR_ID.fetch_add(1, Ordering::Relaxed),
                name: self.previous().to_owned(),
            })
        } else {
            None
        };
        self.consume(
            &TokenType::LeftBrace,
            "Expect '{' before class body".to_string(),
        )?;
        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let Stmt::Function(f) = self.function("method")? else {
                #[allow(unreachable_code)]
                !unreachable!("function() returned a Stmt variant other than Stmt::Function")
            };
            methods.push(f);
        }
        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after class body.".to_string(),
        )?;
        Ok(Stmt::Class {
            name,
            methods,
            superclass,
        })
    }

    fn function(&mut self, kind: &str) -> ParserResult<Stmt> {
        let name = self.consume(&TokenType::Identifier, format!("Expect {kind} name."))?;
        self.consume(
            &TokenType::LeftParen,
            format!("Expect '(' after {kind} name."),
        )?;
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError::new(
                        name,
                        "Can't have more than 255 parameters.".to_string(),
                    ));
                }
                params.push(
                    self.consume(&TokenType::Identifier, "Expect parameter name.".to_string())?,
                );
                if !self.match_t_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after parameters.".to_string(),
        )?;
        self.consume(
            &TokenType::LeftBrace,
            format!("Expect '{{' before {kind} body."),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(Function { name, params, body }))
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self.consume(&TokenType::Identifier, "Expect variable name.".to_string())?;
        let initializer = if self.match_t_types(&[TokenType::Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after variable declaration.".to_string(),
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
            TokenType::LeftBrace => {
                self.advance();
                Ok(Stmt::Block(self.block()?))
            }
            TokenType::Print => {
                self.advance();
                self.print_statement()
            }
            TokenType::Return => {
                self.advance();
                self.return_statement()
            }
            TokenType::While => {
                self.advance();
                self.while_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn for_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'for'.".to_string())?;

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
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after loop condition.".to_string(),
        )?;

        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after for clauses.".to_string(),
        )?;

        let mut body = self.statement()?;
        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        let condition = condition.unwrap_or(Expr::Literal(Value::Bool(true)));

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn return_statement(&mut self) -> ParserResult<Stmt> {
        let keyword = self.previous().to_owned();
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            &TokenType::Semicolon,
            "Expect ';' after return value.".to_string(),
        )?;
        Ok(Stmt::Return { keyword, value })
    }

    fn while_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(
            &TokenType::LeftParen,
            "Expect '(' after 'while'.".to_string(),
        )?;
        let condition = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after condition.".to_string(),
        )?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { condition, body })
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&TokenType::LeftParen, "Expect '(' after 'if'.".to_string())?;
        let condition = self.expression()?;
        self.consume(
            &TokenType::RightParen,
            "Expect ')' after if condition.".to_string(),
        )?;
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
        self.consume(
            &TokenType::RightBrace,
            "Expect '}' after block.".to_string(),
        )?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Print(expr))
    }

    fn expression_statement(&mut self) -> ParserResult<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expect ';' after value.".to_string())?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;
        if self.match_t_types(&[TokenType::Equal]) {
            let equals = self.previous().to_owned();
            let value = self.assignment()?;
            match expr {
                Expr::Variable { id, name } => Ok(Expr::Assign {
                    id,
                    name,
                    value: Box::new(value),
                }),
                Expr::Get { object, name } => Ok(Expr::Set {
                    object,
                    name,
                    value: Box::new(value),
                }),
                _ => Err(ParseError::new(
                    equals,
                    "Invalid assignment target.".to_string(),
                )),
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
            self.call()
        }
    }

    fn call(&mut self) -> ParserResult<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.match_t_types(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_t_types(&[TokenType::Dot]) {
                let name = self.consume(
                    &TokenType::Identifier,
                    "Expect property name after '.'.".to_string(),
                )?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> ParserResult<Expr> {
        let mut arguments = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(ParseError::new(
                        self.peek().to_owned(),
                        "Cannot have more than 255 arguments.".to_string(),
                    ));
                }
                arguments.push(self.expression()?);
                if !self.match_t_types(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(
            &TokenType::RightParen,
            "Expect ')' after arguments.".to_string(),
        )?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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
            TokenType::Identifier => Ok(Expr::Variable {
                id: NEXT_EXPR_ID.fetch_add(1, Ordering::Relaxed),
                name: self.previous().to_owned(),
            }),
            TokenType::LeftParen => {
                let expr = self.expression()?;
                self.consume(
                    &TokenType::RightParen,
                    "Expect ')' after expression.".to_string(),
                )?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            TokenType::Super => {
                let keyword = self.previous().to_owned();
                self.consume(&TokenType::Dot, "Expect '.' after 'super'.".to_string())?;
                let method = self.consume(
                    &TokenType::Identifier,
                    "Expect superclass method name.".to_string(),
                )?;
                Ok(Expr::Super {
                    id: NEXT_EXPR_ID.fetch_add(1, Ordering::Relaxed),
                    keyword,
                    method,
                })
            }
            TokenType::This => Ok(Expr::This {
                id: NEXT_EXPR_ID.fetch_add(1, Ordering::Relaxed),
                keyword: self.previous().to_owned(),
            }),
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

    fn consume(&mut self, t_type: &TokenType, msg: String) -> ParserResult<Token> {
        if self.check(t_type) {
            Ok(self.advance().to_owned())
        } else {
            Err(ParseError::new(self.peek().to_owned(), msg))
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
