use crate::runtime::{Completion, JsError};
use num_bigint::BigInt;
use num_traits::ToPrimitive;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(f64),
    BigInt(BigInt),
    String(String),
    RegExp(String, String),
    Punct(char),
    Eof,
}

pub struct Lexer<'src> {
    chars: Vec<char>,
    index: usize,
    _source: &'src str,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            _source: source,
        }
    }

    pub fn tokenize(mut self) -> Completion<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut regexp_allowed = true;
        loop {
            self.skip_ws_and_comments();
            let Some(ch) = self.peek() else {
                tokens.push(Token::Eof);
                return Ok(tokens);
            };
            let token = match ch {
                ch if is_identifier_start(ch) => Token::Identifier(self.identifier()),
                '0'..='9' => self.number()?,
                '.' if matches!(self.peek_next(), Some('0'..='9')) => self.number()?,
                '"' | '\'' => Token::String(self.string(ch)?),
                '/' if regexp_allowed
                    && self.peek_next() != Some('/')
                    && self.peek_next() != Some('*') =>
                {
                    self.regexp_literal()?
                }
                '{' | '}' | '(' | ')' | '[' | ']' | '.' | ',' | ':' | ';' | '?' | '=' | '!'
                | '+' | '-' | '*' | '/' | '%' | '<' | '>' | '&' | '|' => {
                    self.bump();
                    Token::Punct(ch)
                }
                _ => {
                    return Err(JsError::syntax(format!(
                        "unexpected character `{ch}` in JBS-0 script"
                    )))
                }
            };
            regexp_allowed = regexp_allowed_after(&token);
            tokens.push(token);
        }
    }

    fn skip_ws_and_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(ch) if ch.is_whitespace()) {
                self.bump();
            }
            if self.peek() == Some('/') && self.peek_next() == Some('/') {
                while !matches!(self.peek(), None | Some('\n')) {
                    self.bump();
                }
            } else if self.peek() == Some('/') && self.peek_next() == Some('*') {
                self.bump();
                self.bump();
                while let Some(ch) = self.bump() {
                    if ch == '*' && self.peek() == Some('/') {
                        self.bump();
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn identifier(&mut self) -> String {
        let mut value = String::new();
        while matches!(self.peek(), Some(ch) if is_identifier_continue(ch)) {
            value.push(self.bump().unwrap());
        }
        value
    }

    fn number(&mut self) -> Completion<Token> {
        if self.peek() == Some('0') {
            if let Some(radix) = match self.peek_next() {
                Some('x' | 'X') => Some(16),
                Some('o' | 'O') => Some(8),
                Some('b' | 'B') => Some(2),
                _ => None,
            } {
                self.bump();
                self.bump();
                return self.radix_number(radix);
            }
        }

        let mut value = String::new();
        let mut saw_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                value.push(self.bump().unwrap());
            } else if ch == '.' && !saw_dot {
                saw_dot = true;
                value.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            value.push(self.bump().unwrap());
            if matches!(self.peek(), Some('+' | '-')) {
                value.push(self.bump().unwrap());
            }
            let exponent_start = value.len();
            while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
                value.push(self.bump().unwrap());
            }
            if exponent_start == value.len() {
                return Err(JsError::syntax(format!("invalid number literal `{value}`")));
            }
        }
        if !saw_dot
            && !matches!(value.as_bytes().last(), Some(b'e' | b'E'))
            && self.peek() == Some('n')
        {
            self.bump();
            return value
                .parse::<BigInt>()
                .map(Token::BigInt)
                .map_err(|_| JsError::syntax(format!("invalid BigInt literal `{value}n`")));
        }
        value
            .parse::<f64>()
            .map(Token::Number)
            .map_err(|_| JsError::syntax(format!("invalid number literal `{value}`")))
    }

    fn radix_number(&mut self, radix: u32) -> Completion<Token> {
        let mut digits = String::new();
        while matches!(self.peek(), Some(ch) if digit_value(ch).is_some_and(|digit| digit < radix))
        {
            digits.push(self.bump().unwrap());
        }
        if digits.is_empty() {
            return Err(JsError::syntax("invalid numeric literal"));
        }
        let integer = BigInt::parse_bytes(digits.as_bytes(), radix)
            .ok_or_else(|| JsError::syntax(format!("invalid numeric literal `{digits}`")))?;
        if self.peek() == Some('n') {
            self.bump();
            Ok(Token::BigInt(integer))
        } else {
            let number = integer
                .to_f64()
                .ok_or_else(|| JsError::syntax(format!("invalid numeric literal `{digits}`")))?;
            Ok(Token::Number(number))
        }
    }

    fn string(&mut self, quote: char) -> Completion<String> {
        self.bump();
        let mut value = String::new();
        while let Some(ch) = self.bump() {
            if ch == quote {
                return Ok(value);
            }
            if ch == '\\' {
                let Some(escaped) = self.bump() else {
                    return Err(JsError::syntax("unterminated string escape"));
                };
                value.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'u' => self.unicode_escape()?,
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                });
            } else {
                value.push(ch);
            }
        }
        Err(JsError::syntax("unterminated string literal"))
    }

    fn unicode_escape(&mut self) -> Completion<char> {
        let mut value = 0u32;
        for _ in 0..4 {
            let Some(digit) = self.bump().and_then(|ch| ch.to_digit(16)) else {
                return Err(JsError::syntax("invalid unicode escape"));
            };
            value = (value << 4) | digit;
        }
        char::from_u32(value).ok_or_else(|| JsError::syntax("invalid unicode escape"))
    }

    fn regexp_literal(&mut self) -> Completion<Token> {
        self.bump();
        let mut pattern = String::new();
        let mut in_class = false;
        let mut escaped = false;
        while let Some(ch) = self.bump() {
            if escaped {
                pattern.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                pattern.push(ch);
                escaped = true;
                continue;
            }
            if ch == '[' {
                in_class = true;
                pattern.push(ch);
                continue;
            }
            if ch == ']' {
                in_class = false;
                pattern.push(ch);
                continue;
            }
            if ch == '/' && !in_class {
                let flags = self.regexp_flags();
                return Ok(Token::RegExp(pattern, flags));
            }
            if ch == '\n' || ch == '\r' {
                return Err(JsError::syntax("unterminated regular expression literal"));
            }
            pattern.push(ch);
        }
        Err(JsError::syntax("unterminated regular expression literal"))
    }

    fn regexp_flags(&mut self) -> String {
        let mut flags = String::new();
        while matches!(self.peek(), Some(ch) if ch.is_ascii_alphabetic()) {
            flags.push(self.bump().unwrap());
        }
        flags
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        Some(ch)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch == '$' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_alphanumeric()
}

fn digit_value(ch: char) -> Option<u32> {
    match ch {
        '0'..='9' => Some(ch as u32 - '0' as u32),
        'a'..='f' => Some(ch as u32 - 'a' as u32 + 10),
        'A'..='F' => Some(ch as u32 - 'A' as u32 + 10),
        _ => None,
    }
}

fn regexp_allowed_after(token: &Token) -> bool {
    match token {
        Token::Identifier(name) => matches!(
            name.as_str(),
            "return"
                | "throw"
                | "case"
                | "delete"
                | "typeof"
                | "void"
                | "new"
                | "in"
                | "instanceof"
                | "else"
                | "do"
        ),
        Token::Punct(ch) => matches!(
            ch,
            '(' | '{'
                | '['
                | ','
                | ';'
                | ':'
                | '?'
                | '='
                | '!'
                | '+'
                | '-'
                | '*'
                | '/'
                | '%'
                | '<'
                | '>'
                | '&'
                | '|'
        ),
        Token::Number(_) | Token::BigInt(_) | Token::String(_) | Token::RegExp(_, _) => false,
        Token::Eof => true,
    }
}
