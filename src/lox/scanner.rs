use std::{collections::HashMap, hash::Hash, str::Chars, sync::OnceLock};

use itertools::{peek_nth, PeekNth};

use super::error::ParseError;

static KEYWORDS: OnceLock<HashMap<&str, TokenType>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Ident,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    EOF,
}

#[derive(Debug)]
pub enum Literal<'a> {
    String(&'a str),
    Number(f64),
    Bool(bool),
}

#[derive(Debug)]
pub struct Token<'a> {
    t_type: TokenType,
    lexeme: &'a str,
    literal: Option<Literal<'a>>,
    col: usize,
    line: usize,
}

pub struct Parser<'a> {
    had_error: bool,
    source: &'a str,
    chars: PeekNth<Chars<'a>>,
    tokens: Vec<Token<'a>>,
    start: usize,
    current: usize,
    line_start: usize,
    col: usize,
    line: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            had_error: false,
            source,
            chars: peek_nth(source.chars()),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line_start: 0,
            col: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<&Vec<Token>, ()> {
        while self.chars.peek().is_some() {
            self.start = self.current;
            self.scan_token();
        }

        if !self.had_error {
            Ok(&self.tokens)
        } else {
            Err(())
        }
    }

    fn lexeme(&self) -> &str {
        &self.source[self.line_start..self.current]
    }

    fn check_keyword(&self, key: &str) -> Option<&TokenType> {
        KEYWORDS
            .get_or_init(|| {
                let mut keywords = HashMap::new();
                keywords.insert("and", TokenType::And);
                keywords.insert("class", TokenType::Class);
                keywords.insert("else", TokenType::Else);
                keywords.insert("false", TokenType::False);
                keywords.insert("for", TokenType::For);
                keywords.insert("fun", TokenType::Fun);
                keywords.insert("if", TokenType::If);
                keywords.insert("nil", TokenType::Nil);
                keywords.insert("or", TokenType::Or);
                keywords.insert("print", TokenType::Print);
                keywords.insert("return", TokenType::Return);
                keywords.insert("super", TokenType::Super);
                keywords.insert("this", TokenType::This);
                keywords.insert("true", TokenType::True);
                keywords.insert("var", TokenType::Var);
                keywords.insert("while", TokenType::While);
                keywords
            })
            .get(key)
    }

    fn record_error(&mut self, msg: String) {
        self.had_error = true;
        eprintln!(
            "{}",
            ParseError::new(msg, self.lexeme(), self.col, self.line,)
        )
    }

    fn add_token(&mut self, t_type: TokenType, literal: Option<Literal<'a>>) {
        self.tokens.push(Token {
            t_type,
            lexeme: &self.source[self.start..self.current],
            literal,
            col: self.col,
            line: self.line,
        });
    }

    fn matches(&mut self, pred: impl FnOnce(&char) -> bool) -> Option<char> {
        self.chars
            .next_if(pred)
            .map(|c| {
                self.current += c.len_utf8();
                self.col += 1;
                Some(c)
            })
            .unwrap_or(None)
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().map(|c| {
            self.current += c.len_utf8();
            self.col += 1;
            c
        })
    }

    fn string(&mut self) {
        while let Some(c) = self.matches(|&c| c != '"') {
            if c == '\n' {
                self.line += 1;
                self.line_start = self.current;
                self.col = 0;
            }
        }
        if self.chars.peek().is_some() {
            self.advance();
            self.add_token(
                TokenType::String,
                Some(Literal::String(
                    &self.source[(self.start + 1)..(self.current - 1)],
                )),
            )
        } else {
            self.record_error(format!("Unterminated string: {}.", self.lexeme()));
        }
    }

    fn identifier(&mut self) {
        while self.matches(|&c| c.is_alphanumeric() || c == '_').is_some() {}
        if let Some(t_type) = self.check_keyword(self.lexeme()) {
            self.add_token(t_type.clone(), None)
        } else {
            self.add_token(TokenType::Ident, None);
        }
    }

    fn number(&mut self) {
        while self.matches(|&c| c.is_numeric()).is_some() {}
        if self.chars.peek() == Some(&'.') {
            if let Some(&c) = self.chars.peek_nth(1) {
                if c.is_numeric() {
                    self.advance();
                    while self.matches(|&c| c.is_numeric()).is_some() {}
                }
            }
        }
        if let Ok(number) = self.lexeme().parse::<f64>() {
            self.add_token(TokenType::Number, Some(Literal::Number(number)))
        } else {
            self.record_error(format!("Invalid number: {}.", self.lexeme()));
        }
    }

    fn scan_token(&mut self) -> bool {
        if let Some(c) = self.advance() {
            match c {
                '(' => self.add_token(TokenType::LeftParen, None),
                ')' => self.add_token(TokenType::RightParen, None),
                '{' => self.add_token(TokenType::LeftBrace, None),
                '}' => self.add_token(TokenType::RightBrace, None),
                ',' => self.add_token(TokenType::Comma, None),
                '.' => self.add_token(TokenType::Dot, None),
                '-' => self.add_token(TokenType::Minus, None),
                '+' => self.add_token(TokenType::Plus, None),
                ';' => self.add_token(TokenType::Semicolon, None),
                '/' => {
                    if self.matches(|&c| c == '/').is_some() {
                        while self.matches(|&c| c != '\n').is_some() {}
                    } else {
                        self.add_token(TokenType::Slash, None);
                    }
                }
                '*' => self.add_token(TokenType::Star, None),
                '!' => {
                    if self.matches(|&c| c == '=').is_some() {
                        self.add_token(TokenType::BangEqual, None);
                    } else {
                        self.add_token(TokenType::Bang, None);
                    }
                }
                '=' => {
                    if self.matches(|&c| c == '=').is_some() {
                        self.add_token(TokenType::EqualEqual, None);
                    } else {
                        self.add_token(TokenType::Equal, None)
                    }
                }
                '<' => {
                    if self.matches(|&c| c == '=').is_some() {
                        self.add_token(TokenType::LessEqual, None);
                    } else {
                        self.add_token(TokenType::Less, None);
                    }
                }
                '>' => {
                    if self.matches(|&c| c == '=').is_some() {
                        self.add_token(TokenType::GreaterEqual, None);
                    } else {
                        self.add_token(TokenType::Greater, None);
                    }
                }
                c if c.is_alphabetic() => self.identifier(),
                c if c.is_numeric() => self.number(),
                '"' => self.string(),
                '\n' => {
                    self.line += 1;
                    self.line_start = self.current;
                    self.col = 0;
                }
                '\t' => self.col += 3,
                ' ' | '\r' => {}
                _ => {
                    self.record_error(format!("Unexpected \"{}\" character.", c));
                }
            }
            true
        } else {
            false
        }
    }
}
