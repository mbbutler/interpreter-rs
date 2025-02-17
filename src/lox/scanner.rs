use std::{collections::HashMap, fmt::Display, str::Chars, sync::LazyLock};

use itertools::{peek_nth, PeekNth};

use super::error::ScanError;

static KEYWORDS: LazyLock<HashMap<&str, TokenType>> = LazyLock::new(|| {
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
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Identifier,
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

    Eof,
}

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Number(n) => write!(f, "{n}"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Nil => write!(f, "nill"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub t_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    // pub col: usize,
    pub line: usize,
}

pub struct Scanner<'a> {
    had_error: bool,
    source: &'a str,
    chars: PeekNth<Chars<'a>>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line_start: usize,
    col: usize,
    line: usize,
}

impl<'a> Scanner<'a> {
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

        self.add_token(TokenType::Eof, None);

        if !self.had_error {
            Ok(&self.tokens)
        } else {
            Err(())
        }
    }

    fn lexeme(&self) -> &str {
        &self.source[self.start..self.current]
    }

    fn check_keyword(&self, key: &str) -> Option<&'static TokenType> {
        KEYWORDS.get(key)
    }

    fn record_error(&mut self, msg: String) {
        self.had_error = true;
        eprintln!(
            "{}",
            ScanError::new(msg, self.lexeme(), self.col, self.line,)
        )
    }

    fn add_token(&mut self, t_type: TokenType, literal: Option<Literal>) {
        self.tokens.push(Token {
            t_type,
            lexeme: self.source[self.start..self.current].to_string(),
            literal,
            // col: self.col,
            line: self.line,
        });
    }

    fn matches(&mut self, pred: impl FnOnce(&char) -> bool) -> Option<char> {
        self.chars.next_if(pred).inspect(|c| {
            self.current += c.len_utf8();
            self.col += 1;
        })
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next().inspect(|c| {
            self.current += c.len_utf8();
            self.col += 1;
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
                    self.source[(self.start + 1)..(self.current - 1)].to_string(),
                )),
            )
        } else {
            self.record_error(format!("Unterminated string: {}.", self.lexeme()));
        }
    }

    fn identifier(&mut self) {
        while self.matches(|&c| c.is_alphanumeric() || c == '_').is_some() {}
        if let Some(t_type) = self.check_keyword(self.lexeme()) {
            self.add_token(*t_type, None)
        } else {
            self.add_token(TokenType::Identifier, None);
        }
    }

    fn number(&mut self) {
        while self.matches(char::is_ascii_digit).is_some() {}
        if self.chars.peek() == Some(&'.')
            && self
                .chars
                .peek_nth(1)
                .map(char::is_ascii_digit)
                .unwrap_or(false)
        {
            self.advance();
            while self.matches(char::is_ascii_digit).is_some() {}
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
                c if c.is_ascii_digit() => self.number(),
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
