use itertools::PeekNth;
use std::str::CharIndices;

use crate::error::{InterpretError, Result};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
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
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
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
    #[default]
    Eof,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Token<'a> {
    pub t_type: TokenType,
    pub lexeme: &'a str,
    pub line: usize,
}

impl<'a> Token<'a> {
    fn new(t_type: TokenType, lexeme: &'a str, line: usize) -> Self {
        Self {
            t_type,
            lexeme,
            line,
        }
    }
}

pub struct Scanner<'a> {
    src: &'a str,
    iter: PeekNth<CharIndices<'a>>,
    line: usize,
    is_done: bool,
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            src: s,
            iter: itertools::peek_nth(s.char_indices()),
            line: 1,
            is_done: false,
        }
    }
}

impl<'a> Scanner<'a> {
    fn match_c(&mut self, start: usize, c: char, options: [TokenType; 2]) -> Token<'a> {
        self.iter
            .next_if(|(_, d)| *d == c)
            .map(|(j, d)| Token::new(options[0], &self.src[start..j + d.len_utf8()], self.line))
            .unwrap_or_else(|| {
                Token::new(
                    options[1],
                    &self.src[start..start + c.len_utf8()],
                    self.line,
                )
            })
    }

    fn skip_whitespace(&mut self) {
        while let Some((_, c)) = self.iter.peek() {
            match c {
                '\n' => {
                    self.line += 1;
                    self.iter.next();
                }
                '/' => {
                    let Some((_, c)) = self.iter.peek_nth(1) else {
                        return;
                    };
                    if *c == '/' {
                        while let Some((_, c)) = self.iter.peek() {
                            if *c != '\n' {
                                self.iter.next();
                            }
                        }
                    } else {
                        return;
                    }
                }
                c => {
                    if c.is_whitespace() {
                        self.iter.next();
                    } else {
                        return;
                    }
                }
            }
        }
    }

    fn string(&mut self, start: usize) -> Result<Token<'a>> {
        while let Some((_, c)) = self.iter.peek() {
            if *c != '"' {
                if *c == '\n' {
                    self.line += 1;
                }
                self.iter.next();
            } else {
                break;
            }
        }
        let (i, c) = self
            .iter
            .next()
            .ok_or_else(|| InterpretError::CompileError("Unterminated String.".to_string()))?;
        Ok(Token::new(
            TokenType::String,
            &self.src[start..i + c.len_utf8()],
            self.line,
        ))
    }

    fn number(&mut self, start: usize) -> Token<'a> {
        // Consume all of the ascii digits
        while self.iter.next_if(|(_, c)| c.is_ascii_digit()).is_some() {}

        // If there are no more chars, then slice the rest of the string
        let Some((i, c)) = self.iter.peek() else {
            return Token::new(TokenType::Number, &self.src[start..], self.line);
        };

        // If there is no trailing '.' then slice up until the non-'.' char
        if *c != '.' {
            return Token::new(TokenType::Number, &self.src[start..*i], self.line);
        }

        // If there is a trailing '.' but there is no char after that, then slice up until the '.' char
        // Note: The reinstantiation of `i` here is to unborrow `self.iter`
        let i = *i;
        let Some((_, c)) = self.iter.peek_nth(1) else {
            return Token::new(TokenType::Number, &self.src[start..i], self.line);
        };

        // If the char after the '.' is an ascii digit, then consume trailing digits
        if c.is_ascii_digit() {
            // Consume the '.' char
            let _ = self.iter.next();
            // Consume the ascii digits
            while self.iter.next_if(|(_, c)| c.is_ascii_digit()).is_some() {}
            match self.iter.peek() {
                Some((i, _)) => Token::new(TokenType::Number, &self.src[start..*i], self.line),
                None => Token::new(TokenType::Number, &self.src[start..], self.line),
            }
        } else {
            // Otherwise slice up until the '.' char
            Token::new(TokenType::Number, &self.src[start..i], self.line)
        }
    }

    fn identifier(&mut self, start: usize) -> Token<'a> {
        while self
            .iter
            .next_if(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
            .is_some()
        {}
        match self.iter.peek() {
            Some((i, _)) => {
                let s = &self.src[start..*i];
                Token::new(self.identifier_type(s), s, self.line)
            }
            None => {
                let s = &self.src[start..];
                Token::new(self.identifier_type(s), s, self.line)
            }
        }
    }

    fn identifier_type(&self, lexeme: &'a str) -> TokenType {
        let mut chars = lexeme.char_indices();
        let (_, next) = chars.next().unwrap();
        match next {
            'a' => self.check_keyword(&lexeme[chars.offset()..], "nd", TokenType::And),
            'c' => self.check_keyword(&lexeme[chars.offset()..], "lass", TokenType::Class),
            'e' => self.check_keyword(&lexeme[chars.offset()..], "lse", TokenType::Else),
            'f' => match chars.next() {
                Some((_, c)) => match c {
                    'a' => self.check_keyword(&lexeme[chars.offset()..], "lse", TokenType::False),
                    'o' => self.check_keyword(&lexeme[chars.offset()..], "r", TokenType::For),
                    'u' => self.check_keyword(&lexeme[chars.offset()..], "n", TokenType::Fun),
                    _ => TokenType::Identifier,
                },
                None => TokenType::Identifier,
            },
            'i' => self.check_keyword(&lexeme[chars.offset()..], "f", TokenType::If),
            'n' => self.check_keyword(&lexeme[chars.offset()..], "il", TokenType::Nil),
            'o' => self.check_keyword(&lexeme[chars.offset()..], "r", TokenType::Or),
            'p' => self.check_keyword(&lexeme[chars.offset()..], "rint", TokenType::Print),
            'r' => self.check_keyword(&lexeme[chars.offset()..], "eturn", TokenType::Return),
            's' => self.check_keyword(&lexeme[chars.offset()..], "uper", TokenType::Super),
            't' => match chars.next() {
                Some((_, c)) => match c {
                    'h' => self.check_keyword(&lexeme[chars.offset()..], "is", TokenType::This),
                    'r' => self.check_keyword(&lexeme[chars.offset()..], "ue", TokenType::True),
                    _ => TokenType::Identifier,
                },
                None => TokenType::Identifier,
            },
            'v' => self.check_keyword(&lexeme[chars.offset()..], "ar", TokenType::Var),
            'w' => self.check_keyword(&lexeme[chars.offset()..], "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, lexeme: &'a str, s: &'a str, tt: TokenType) -> TokenType {
        if lexeme == s {
            tt
        } else {
            TokenType::Identifier
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done {
            return None;
        }

        self.skip_whitespace();
        let Some((start, c)) = self.iter.next() else {
            self.is_done = true;
            return Some(Ok(Token::new(
                TokenType::Eof,
                &self.src[self.src.len()..],
                self.line,
            )));
        };

        match c {
            c if c.is_ascii_alphabetic() => Some(Ok(self.identifier(start))),
            c if c.is_ascii_digit() => Some(Ok(self.number(start))),
            '(' => Some(Ok(Token::new(
                TokenType::LeftParen,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            ')' => Some(Ok(Token::new(
                TokenType::RightParen,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '{' => Some(Ok(Token::new(
                TokenType::LeftBrace,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '}' => Some(Ok(Token::new(
                TokenType::RightBrace,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            ';' => Some(Ok(Token::new(
                TokenType::Semicolon,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            ',' => Some(Ok(Token::new(
                TokenType::Comma,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '.' => Some(Ok(Token::new(
                TokenType::Dot,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '-' => Some(Ok(Token::new(
                TokenType::Minus,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '+' => Some(Ok(Token::new(
                TokenType::Plus,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '/' => Some(Ok(Token::new(
                TokenType::Slash,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '*' => Some(Ok(Token::new(
                TokenType::Star,
                &self.src[start..start + c.len_utf8()],
                self.line,
            ))),
            '!' => Some(Ok(self.match_c(
                start,
                '=',
                [TokenType::BangEqual, TokenType::Bang],
            ))),
            '=' => Some(Ok(self.match_c(
                start,
                '=',
                [TokenType::EqualEqual, TokenType::Equal],
            ))),
            '<' => Some(Ok(self.match_c(
                start,
                '=',
                [TokenType::LessEqual, TokenType::Less],
            ))),
            '>' => Some(Ok(self.match_c(
                start,
                '=',
                [TokenType::GreaterEqual, TokenType::Greater],
            ))),
            '"' => Some(self.string(start)),
            _ => {
                self.is_done = true;
                Some(Err(InterpretError::CompileError(format!(
                    "Unexpected character: {}.",
                    c
                ))))
            }
        }
    }
}

#[cfg(test)]
pub mod test {
    use itertools::Itertools;

    use super::{Scanner, Token, TokenType};

    #[test]
    fn test_scanner() {
        let s = r#"( ) { } , . - + ; / * ! != = == > >= < <= my_var "string" 123.456 and class
        else false fun for if nil or print return super this true var while"#;
        let tokens = [
            Token::new(TokenType::LeftParen, "(", 1),
            Token::new(TokenType::RightParen, ")", 1),
            Token::new(TokenType::LeftBrace, "{", 1),
            Token::new(TokenType::RightBrace, "}", 1),
            Token::new(TokenType::Comma, ",", 1),
            Token::new(TokenType::Dot, ".", 1),
            Token::new(TokenType::Minus, "-", 1),
            Token::new(TokenType::Plus, "+", 1),
            Token::new(TokenType::Semicolon, ";", 1),
            Token::new(TokenType::Slash, "/", 1),
            Token::new(TokenType::Star, "*", 1),
            Token::new(TokenType::Bang, "!", 1),
            Token::new(TokenType::BangEqual, "!=", 1),
            Token::new(TokenType::Equal, "=", 1),
            Token::new(TokenType::EqualEqual, "==", 1),
            Token::new(TokenType::Greater, ">", 1),
            Token::new(TokenType::GreaterEqual, ">=", 1),
            Token::new(TokenType::Less, "<", 1),
            Token::new(TokenType::LessEqual, "<=", 1),
            Token::new(TokenType::Identifier, "my_var", 1),
            Token::new(TokenType::String, r#""string""#, 1),
            Token::new(TokenType::Number, "123.456", 1),
            Token::new(TokenType::And, "and", 1),
            Token::new(TokenType::Class, "class", 1),
            Token::new(TokenType::Else, "else", 2),
            Token::new(TokenType::False, "false", 2),
            Token::new(TokenType::Fun, "fun", 2),
            Token::new(TokenType::For, "for", 2),
            Token::new(TokenType::If, "if", 2),
            Token::new(TokenType::Nil, "nil", 2),
            Token::new(TokenType::Or, "or", 2),
            Token::new(TokenType::Print, "print", 2),
            Token::new(TokenType::Return, "return", 2),
            Token::new(TokenType::Super, "super", 2),
            Token::new(TokenType::This, "this", 2),
            Token::new(TokenType::True, "true", 2),
            Token::new(TokenType::Var, "var", 2),
            Token::new(TokenType::While, "while", 2),
            Token::new(TokenType::Eof, "", 2),
        ];
        let scanner = Scanner::new(s);
        for pair in scanner.zip_longest(tokens.iter()) {
            match pair {
                itertools::EitherOrBoth::Both(l, r) => assert_eq!(&l.unwrap(), r),
                _ => panic!("Iterators are not the same size"),
            };
        }
    }
}
