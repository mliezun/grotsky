#![allow(dead_code)]

use crate::state::*;
use crate::token::*;
use fnv::FnvHashMap;
use std::collections::HashMap;
use std::time::Instant;

pub struct Lexer<'a> {
    start: usize,
    current: usize,
    line: i32,

    state: &'a mut InterpreterState,
}

fn keywords() -> HashMap<&'static str, Token> {
    HashMap::from([
        ("and", Token::And),
        ("class", Token::Class),
        ("else", Token::Else),
        ("false", Token::False),
        ("fn", Token::Fn),
        ("for", Token::For),
        ("if", Token::If),
        ("elif", Token::Elif),
        ("nil", Token::Nil),
        ("or", Token::Or),
        ("return", Token::Return),
        ("break", Token::Break),
        ("continue", Token::Continue),
        ("super", Token::Super),
        ("this", Token::This),
        ("true", Token::True),
        ("let", Token::Let),
        ("while", Token::While),
        ("not", Token::Not),
        ("in", Token::In),
        ("try", Token::Try),
        ("catch", Token::Catch),
    ])
}

impl Lexer<'_> {
    pub fn new(state: &'_ mut InterpreterState) -> Lexer<'_> {
        Lexer {
            start: 0,
            current: 0,
            line: 1,
            state: state,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.state.source.len()
    }

    fn next(&self) -> u8 {
        self.state.source.as_bytes()[self.current]
    }

    fn advance(&mut self) -> u8 {
        let c = self.next();
        self.current += 1;
        return c;
    }

    fn matches(&self, c: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        return char::from(self.next()) == c;
    }

    fn is_digit(&self, c: u8) -> bool {
        let zero: u8 = 48;
        let nine: u8 = 57;
        c >= zero && c <= nine
    }

    fn is_alpha(&self, c: u8) -> bool {
        let a: u8 = 97;
        let z: u8 = 122;
        let big_a: u8 = 65;
        let big_z: u8 = 90;
        let underscore: u8 = 95;
        (c >= a && c <= z) || (c >= big_a && c <= big_z) || c == underscore
    }

    fn emit(&mut self, token: Token, l: Option<Literal>) {
        let lexeme =
            String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
                .to_string();
        self.state.tokens.push(TokenData {
            token: token,
            lexeme: string_to_static_str(lexeme),
            literal: l,
            line: self.line,
        })
    }

    pub fn scan(&mut self) {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }
        let token_count = self.state.tokens.len();
        if token_count > 0
            && self.state.tokens.get(token_count - 1).unwrap().token != Token::Newline
        {
            self.state.tokens.push(TokenData {
                token: Token::Newline,
                lexeme: string_to_static_str("".to_string()),
                literal: None,
                line: self.line,
            })
        }
        self.state.tokens.push(TokenData {
            token: Token::EOF,
            lexeme: string_to_static_str("".to_string()),
            literal: None,
            line: self.line,
        })
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match char::from(c) {
            '[' => self.emit(Token::LeftBrace, None),
            ']' => self.emit(Token::RightBrace, None),
            '{' => self.emit(Token::LeftCurlyBrace, None),
            '}' => self.emit(Token::RightCurlyBrace, None),
            '(' => self.emit(Token::LeftParen, None),
            ')' => self.emit(Token::RightParen, None),
            ',' => self.emit(Token::Comma, None),
            '.' => self.emit(Token::Dot, None),
            '-' => self.emit(Token::Minus, None),
            '+' => self.emit(Token::Plus, None),
            '/' => self.emit(Token::Slash, None),
            '%' => self.emit(Token::Mod, None),
            '*' => self.emit(Token::Star, None),
            '^' => self.emit(Token::Power, None),
            ':' => self.emit(Token::Colon, None),
            ';' => self.emit(Token::Semicolon, None),
            '#' => {
                while !self.matches('\n') && !self.is_at_end() {
                    self.advance();
                }
            }
            '!' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::BangEqual, None);
                } else {
                    self.state.set_error(InterpreterError {
                        message: "'!' cannot be used here".to_string(),
                        line: self.line,
                        pos: self.start,
                    });
                }
            }
            '=' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::EqualEqual, None);
                } else {
                    self.emit(Token::Equal, None);
                }
            }
            '<' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::LessEqual, None);
                } else {
                    self.emit(Token::Less, None);
                }
            }
            '>' => {
                if self.matches('=') {
                    self.advance();
                    self.emit(Token::GreaterEqual, None);
                } else {
                    self.emit(Token::Greater, None);
                }
            }

            // Ignore whitespace
            ' ' => (),
            '\r' => (),
            '\t' => (),

            '\n' => {
                self.line += 1;
                self.emit(Token::Newline, None);
            }

            '"' => self.string(),

            _ => {
                if self.is_digit(c) {
                    self.number();
                } else if self.is_alpha(c) {
                    self.identifier();
                } else {
                    self.state.set_error(InterpreterError {
                        message: "Illegal character".to_string(),
                        line: self.line,
                        pos: self.start,
                    })
                }
            }
        }
    }

    fn string(&mut self) {
        let mut lit = "".to_string();
        while !self.is_at_end() && !self.matches('"') {
            if self.matches('\n') {
                self.line += 1;
            }
            if self.matches('\\') {
                self.advance();
                let unescaped = self.unescape_sequence();
                if unescaped == "".to_string() {
                    self.state.set_error(InterpreterError {
                        message: "Closing \" was expected".to_string(),
                        line: self.line,
                        pos: self.start,
                    });
                    return;
                }
                lit.push_str(&unescaped);
                continue;
            }
            lit.push(self.state.source.as_bytes()[self.current].into());
            self.advance();
        }

        if self.is_at_end() {
            self.state.set_error(InterpreterError {
                message: "Closing \" was expected".to_string(),
                line: self.line,
                pos: self.start,
            });
            return;
        }

        // Consume ending "
        self.advance();

        self.emit(Token::String, Some(Literal::String(lit)));
    }

    fn unescape_sequence(&mut self) -> String {
        if self.is_at_end() {
            return "".to_string();
        }
        let c = self.advance();
        match char::from(c) {
            // 'a' => "\a".to_string(),
            // 'b' => "\b".to_string(),
            // 'f' => "\f".to_string(),
            // 'v' => "\v".to_string(),
            'n' => "\n".to_string(),
            'r' => "\r".to_string(),
            't' => "\t".to_string(),
            '\\' => "\\".to_string(),
            '"' => "\"".to_string(),
            _ => "".to_string(),
        }
    }

    fn number(&mut self) {
        while !self.is_at_end() && self.is_digit(self.next()) {
            self.advance();
        }

        if self.matches('.') {
            self.advance();
            while self.is_digit(self.next()) {
                self.advance();
            }
        }

        let lit = String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
            .parse::<f64>()
            .unwrap();

        self.emit(Token::Number, Some(Literal::Number(lit)));
    }

    fn identifier(&mut self) {
        while !self.is_at_end() && self.is_alpha(self.next()) {
            self.advance();
        }

        let identifier =
            String::from_utf8_lossy(&self.state.source.as_bytes()[self.start..self.current])
                .to_string();

        let kwds: HashMap<&str, Token> = keywords();
        let token = match kwds.get(identifier.as_str()) {
            Some(tok) => tok,
            None => &Token::Identifier,
        };

        self.emit(*token, None);
    }
}
