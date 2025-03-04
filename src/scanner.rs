use crate::domain::token::Token;
use crate::domain::token_type::TokenType;
use crate::domain::Literal;

/*
    The Scanner is responsible for converting the source code into a sequence of tokens.
    The Scanner will read the source code character by character and convert it into tokens.

    Reference - https://craftinginterpreters.com/scanning.html#recognizing-lexemes
*/

#[derive(Debug, Clone)]
pub struct Scanner {
    pub source: Vec<char>,
    pub tokens: Vec<Token>,
    pub start: usize,
    pub current: usize,
    pub line: u32,
    pub column: u32,
    pub errors: Vec<ScannerError>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source: source.chars().collect(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            column: 0,
            errors: Vec::new(),
        }
    }

    pub fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            // We are at the beginning of the next lexeme.
            self.start = self.current;
            Self::scan_token(self);
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            None,
            self.line,
            self.column,
        ));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let current_char = Self::advance(self);
        match current_char {
            '(' => Self::add_token(self, TokenType::LeftParen, None),
            ')' => Self::add_token(self, TokenType::RightParen, None),
            '{' => Self::add_token(self, TokenType::LeftBrace, None),
            '}' => Self::add_token(self, TokenType::RightBrace, None),
            ',' => Self::add_token(self, TokenType::Comma, None),
            '.' => Self::add_token(self, TokenType::Dot, None),
            '-' => Self::add_token(self, TokenType::Minus, None),
            '+' => Self::add_token(self, TokenType::Plus, None),
            ';' => Self::add_token(self, TokenType::Semicolon, None),
            '*' => Self::add_token(self, TokenType::Star, None),
            '!' => {
                if Self::advance_peek(self, '=') {
                    Self::add_token(self, TokenType::BangEqual, None);
                } else {
                    Self::add_token(self, TokenType::Bang, None);
                }
            }
            '=' => {
                if Self::advance_peek(self, '=') {
                    Self::add_token(self, TokenType::EqualEqual, None);
                } else {
                    Self::add_token(self, TokenType::Equal, None);
                }
            }
            '<' => {
                if Self::advance_peek(self, '=') {
                    Self::add_token(self, TokenType::LessEqual, None);
                } else {
                    Self::add_token(self, TokenType::Less, None);
                }
            }
            '>' => {
                if Self::advance_peek(self, '=') {
                    Self::add_token(self, TokenType::GreaterEqual, None);
                } else {
                    Self::add_token(self, TokenType::Greater, None);
                }
            }
            '/' => {
                if Self::advance_peek(self, '/') {
                    // A comment goes until the end of the line.
                    while self.peek() != '\n' && !self.is_at_end() {
                        Self::advance(self);
                    }
                } else {
                    Self::add_token(self, TokenType::Slash, None);
                }
            }
            ' ' | '\r' | '\t' => {
                // Ignore whitespace.
            }
            '\n' => {
                self.line += 1;
                self.column = 0;
            }
            '"' => {
                Self::construct_string(self);
            }
            '0'..='9' => {
                Self::construct_number(self);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                Self::construct_identifier(self);
            }
            _ => {
                self.errors.push(ScannerError {
                    message: format!("Unexpected character: {}", current_char),
                    line: self.line,
                    column: self.column,
                });
            }
        }
    }

    fn advance(&mut self) -> char {
        let current_char = self.source[self.current];
        self.current += 1;
        self.column += 1;

        current_char
    }

    fn advance_peek(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        let next_char = self.source[self.current];
        if next_char != expected {
            return false;
        }

        self.current += 1;
        self.column += 1;

        true
    }

    fn peek(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }

        self.source[self.current]
    }

    fn construct_string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            Self::advance(self);
        }

        // Unterminated string.
        if self.is_at_end() {
            self.errors.push(ScannerError {
                message: "Unterminated string.".to_string(),
                line: self.line,
                column: self.column,
            });
            return;
        }

        // The closing ".
        // We need to advance one more time to consume the closing ".

        Self::advance(self);
        let value: String = self.source[self.start + 1..self.current - 1]
            .iter()
            .collect();

        Self::add_token(self, TokenType::String, Some(Literal::String(value)));
    }

    fn construct_number(&mut self) {
        while self.peek().is_numeric() {
            Self::advance(self);
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_numeric() {
            // Consume the "."
            Self::advance(self);

            while self.peek().is_numeric() {
                Self::advance(self);
            }
        }

        let value: f64 = self.source[self.start..self.current]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap();

        Self::add_token(self, TokenType::Number, Some(Literal::Number(value)));
    }

    fn peek_next(&mut self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }

        self.source[self.current + 1]
    }

    fn construct_identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' {
            Self::advance(self);
        }

        let text: String = self.source[self.start..self.current].iter().collect();
        let token_type = match text.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier,
        };

        let mut literal: Option<Literal> = None;
        match token_type {
            TokenType::True => literal = Some(Literal::Boolean(true)),
            TokenType::False => literal = Some(Literal::Boolean(false)),
            TokenType::Nil => literal = Some(Literal::Nil),
            _ => {}
        }

        Self::add_token(self, token_type, literal);
    }

    fn add_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(
            token_type,
            text,
            literal,
            self.line,
            self.column,
        ));
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ScannerError {
    pub message: String,
    pub line: u32,
    pub column: u32,
}

impl ScannerError {}

impl std::fmt::Display for ScannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_codecrafter_testcase() {
        let source = "(()".to_string();
        let mut scanner = Scanner::new(source);

        scanner.scan_tokens();
        assert_eq!(scanner.tokens.len(), 4);
        assert_eq!(scanner.tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[1].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[2].token_type, TokenType::RightParen);
        assert_eq!(scanner.tokens[3].token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_tokens() {
        let source = "(){},.-+;*".to_string();
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        assert_eq!(scanner.tokens.len(), 11);
        assert_eq!(scanner.tokens[0].token_type, TokenType::LeftParen);
        assert_eq!(scanner.tokens[1].token_type, TokenType::RightParen);
        assert_eq!(scanner.tokens[2].token_type, TokenType::LeftBrace);
        assert_eq!(scanner.tokens[3].token_type, TokenType::RightBrace);
        assert_eq!(scanner.tokens[4].token_type, TokenType::Comma);
        assert_eq!(scanner.tokens[5].token_type, TokenType::Dot);
        assert_eq!(scanner.tokens[6].token_type, TokenType::Minus);
        assert_eq!(scanner.tokens[7].token_type, TokenType::Plus);
        assert_eq!(scanner.tokens[8].token_type, TokenType::Semicolon);
        assert_eq!(scanner.tokens[9].token_type, TokenType::Star);
        assert_eq!(scanner.tokens[10].token_type, TokenType::Eof);
    }

    #[test]
    fn test_scan_tokens_for_string() {
        let source = format!("\"Coolstorm\"");
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, TokenType::String);
    }

    #[test]
    fn test_scan_tokens_for_number() {
        let source = format!("123.45");
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        assert_eq!(scanner.tokens.len(), 2);
        assert_eq!(scanner.tokens[0].token_type, TokenType::Number);
    }
}
