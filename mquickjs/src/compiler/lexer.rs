//! JavaScript lexer/tokenizer
//!
//! Converts JavaScript source code into a stream of tokens.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// Byte offset in source
    pub offset: usize,
}

impl SourceLocation {
    /// Creates a new source location
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        SourceLocation { line, column, offset }
    }
}

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    /// Number literal (includes integers and floats)
    Number(f64),
    /// String literal
    String(String),
    /// true
    True,
    /// false
    False,
    /// null
    Null,
    /// undefined
    Undefined,

    // Identifiers and Keywords
    /// Identifier
    Identifier(String),

    // Keywords
    /// var
    Var,
    /// let
    Let,
    /// const
    Const,
    /// function
    Function,
    /// return
    Return,
    /// if
    If,
    /// else
    Else,
    /// while
    While,
    /// for
    For,
    /// do
    Do,
    /// break
    Break,
    /// continue
    Continue,
    /// switch
    Switch,
    /// case
    Case,
    /// default
    Default,
    /// try
    Try,
    /// catch
    Catch,
    /// finally
    Finally,
    /// throw
    Throw,
    /// new
    New,
    /// this
    This,
    /// typeof
    TypeOf,
    /// void
    Void,
    /// delete
    Delete,
    /// in
    In,
    /// instanceof
    InstanceOf,

    // Operators
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// **
    StarStar,
    /// =
    Assign,
    /// ==
    Eq,
    /// ===
    StrictEq,
    /// !=
    NotEq,
    /// !==
    StrictNotEq,
    /// <
    Lt,
    /// <=
    LtEq,
    /// >
    Gt,
    /// >=
    GtEq,
    /// !
    Bang,
    /// &&
    LogicalAnd,
    /// ||
    LogicalOr,
    /// &
    Ampersand,
    /// |
    Pipe,
    /// ^
    Caret,
    /// ~
    Tilde,
    /// <<
    LtLt,
    /// >>
    GtGt,
    /// >>>
    GtGtGt,
    /// ++
    PlusPlus,
    /// --
    MinusMinus,
    /// +=
    PlusAssign,
    /// -=
    MinusAssign,
    /// *=
    StarAssign,
    /// /=
    SlashAssign,
    /// %=
    PercentAssign,
    /// &=
    AmpersandAssign,
    /// |=
    PipeAssign,
    /// ^=
    CaretAssign,
    /// <<=
    LtLtAssign,
    /// >>=
    GtGtAssign,
    /// >>>=
    GtGtGtAssign,
    /// ?
    Question,
    /// ??
    NullishCoalescing,

    // Punctuation
    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,
    /// [
    LBracket,
    /// ]
    RBracket,
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// .
    Dot,
    /// :
    Colon,
    /// =>
    Arrow,

    // Special
    /// End of file
    Eof,

    /// Error token
    Error(String),
}

/// A token with location information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Token kind
    pub kind: TokenKind,
    /// Source location
    pub location: SourceLocation,
}

impl Token {
    /// Creates a new token
    pub fn new(kind: TokenKind, location: SourceLocation) -> Self {
        Token { kind, location }
    }
}

/// JavaScript lexer
pub struct Lexer<'a> {
    /// Source code
    source: &'a str,
    /// Source as bytes for faster access
    bytes: &'a [u8],
    /// Current position in source
    pos: usize,
    /// Current line number (1-based)
    line: u32,
    /// Current column number (1-based)
    column: u32,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer
    pub fn new(source: &'a str) -> Self {
        Lexer {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Returns the current source location
    fn location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column, self.pos)
    }

    /// Peeks at the current character without consuming
    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    /// Peeks at the next character (lookahead 1)
    fn peek_next(&self) -> Option<char> {
        let mut chars = self.source[self.pos..].chars();
        chars.next();
        chars.next()
    }

    /// Consumes and returns the current character
    fn consume(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    /// Skips whitespace
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.consume();
            } else {
                break;
            }
        }
    }

    /// Skips a single-line comment
    fn skip_line_comment(&mut self) {
        // Skip '//'
        self.consume();
        self.consume();

        // Skip until newline
        while let Some(ch) = self.peek() {
            self.consume();
            if ch == '\n' {
                break;
            }
        }
    }

    /// Skips a multi-line comment
    fn skip_block_comment(&mut self) -> Result<(), String> {
        // Skip '/*'
        self.consume();
        self.consume();

        // Skip until '*/'
        loop {
            match self.peek() {
                None => return Err("Unterminated block comment".to_string()),
                Some('*') => {
                    self.consume();
                    if self.peek() == Some('/') {
                        self.consume();
                        break;
                    }
                }
                Some(_) => {
                    self.consume();
                }
            }
        }

        Ok(())
    }

    /// Checks if a character is a valid identifier start
    fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
    }

    /// Checks if a character is a valid identifier continue
    fn is_identifier_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'
    }

    /// Reads an identifier or keyword
    fn read_identifier(&mut self) -> TokenKind {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if Self::is_identifier_continue(ch) {
                self.consume();
            } else {
                break;
            }
        }

        let text = &self.source[start..self.pos];

        // Check for keywords
        match text {
            "var" => TokenKind::Var,
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "function" => TokenKind::Function,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "do" => TokenKind::Do,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "default" => TokenKind::Default,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            "finally" => TokenKind::Finally,
            "throw" => TokenKind::Throw,
            "new" => TokenKind::New,
            "this" => TokenKind::This,
            "typeof" => TokenKind::TypeOf,
            "void" => TokenKind::Void,
            "delete" => TokenKind::Delete,
            "in" => TokenKind::In,
            "instanceof" => TokenKind::InstanceOf,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "undefined" => TokenKind::Undefined,
            _ => TokenKind::Identifier(text.to_string()),
        }
    }

    /// Reads a number literal
    fn read_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        // Check for hex/octal/binary prefix
        if self.peek() == Some('0') {
            self.consume();
            match self.peek() {
                Some('x') | Some('X') => {
                    self.consume();
                    return self.read_hex_number();
                }
                Some('o') | Some('O') => {
                    self.consume();
                    return self.read_octal_number();
                }
                Some('b') | Some('B') => {
                    self.consume();
                    return self.read_binary_number();
                }
                Some(ch) if ch.is_ascii_digit() => {
                    // Legacy octal (just parse as decimal for simplicity)
                }
                _ => {}
            }
        }

        // Read integer part
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.consume();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.peek() == Some('.') && self.peek_next().map_or(false, |c| c.is_ascii_digit()) {
            self.consume(); // consume '.'

            // Read fractional part
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    self.consume();
                } else {
                    break;
                }
            }
        }

        // Check for exponent
        if let Some(ch) = self.peek() {
            if ch == 'e' || ch == 'E' {
                self.consume();

                // Optional +/- sign
                if let Some(sign) = self.peek() {
                    if sign == '+' || sign == '-' {
                        self.consume();
                    }
                }

                // Read exponent digits
                let exp_start = self.pos;
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() {
                        self.consume();
                    } else {
                        break;
                    }
                }

                if exp_start == self.pos {
                    return Err("Invalid number: expected exponent digits".to_string());
                }
            }
        }

        let text = &self.source[start..self.pos];
        let value = text.parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", text))?;

        Ok(TokenKind::Number(value))
    }

    /// Reads a hexadecimal number
    fn read_hex_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_hexdigit() {
                self.consume();
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err("Invalid hex number: no digits".to_string());
        }

        let text = &self.source[start..self.pos];
        let value = u64::from_str_radix(text, 16)
            .map_err(|_| format!("Invalid hex number: {}", text))?;

        Ok(TokenKind::Number(value as f64))
    }

    /// Reads an octal number
    fn read_octal_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if ch >= '0' && ch <= '7' {
                self.consume();
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err("Invalid octal number: no digits".to_string());
        }

        let text = &self.source[start..self.pos];
        let value = u64::from_str_radix(text, 8)
            .map_err(|_| format!("Invalid octal number: {}", text))?;

        Ok(TokenKind::Number(value as f64))
    }

    /// Reads a binary number
    fn read_binary_number(&mut self) -> Result<TokenKind, String> {
        let start = self.pos;

        while let Some(ch) = self.peek() {
            if ch == '0' || ch == '1' {
                self.consume();
            } else {
                break;
            }
        }

        if start == self.pos {
            return Err("Invalid binary number: no digits".to_string());
        }

        let text = &self.source[start..self.pos];
        let value = u64::from_str_radix(text, 2)
            .map_err(|_| format!("Invalid binary number: {}", text))?;

        Ok(TokenKind::Number(value as f64))
    }

    /// Reads a string literal
    fn read_string(&mut self, quote: char) -> Result<TokenKind, String> {
        // Skip opening quote
        self.consume();

        let mut result = String::new();

        loop {
            match self.peek() {
                None => return Err("Unterminated string literal".to_string()),
                Some('\n') => return Err("Unterminated string literal (newline)".to_string()),
                Some(ch) if ch == quote => {
                    self.consume();
                    break;
                }
                Some('\\') => {
                    self.consume();
                    match self.peek() {
                        None => return Err("Unterminated string escape".to_string()),
                        Some('n') => {
                            self.consume();
                            result.push('\n');
                        }
                        Some('r') => {
                            self.consume();
                            result.push('\r');
                        }
                        Some('t') => {
                            self.consume();
                            result.push('\t');
                        }
                        Some('\\') => {
                            self.consume();
                            result.push('\\');
                        }
                        Some('\'') => {
                            self.consume();
                            result.push('\'');
                        }
                        Some('"') => {
                            self.consume();
                            result.push('"');
                        }
                        Some('0') => {
                            self.consume();
                            result.push('\0');
                        }
                        Some('x') => {
                            self.consume();
                            let hex = self.read_hex_escape(2)?;
                            if let Some(ch) = char::from_u32(hex) {
                                result.push(ch);
                            } else {
                                result.push('\0');
                            }
                        }
                        Some('u') => {
                            self.consume();
                            let hex = self.read_hex_escape(4)?;
                            if let Some(ch) = char::from_u32(hex) {
                                result.push(ch);
                            } else {
                                return Err(format!("Invalid unicode escape: \\u{:04x}", hex));
                            }
                        }
                        Some(ch) => {
                            // Invalid escape, just include the character
                            self.consume();
                            result.push(ch);
                        }
                    }
                }
                Some(ch) => {
                    self.consume();
                    result.push(ch);
                }
            }
        }

        Ok(TokenKind::String(result))
    }

    /// Reads a hex escape sequence
    fn read_hex_escape(&mut self, len: usize) -> Result<u32, String> {
        let mut value = 0u32;

        for _ in 0..len {
            match self.peek() {
                Some(ch) if ch.is_ascii_hexdigit() => {
                    self.consume();
                    value = value * 16 + ch.to_digit(16).unwrap();
                }
                _ => return Err("Invalid hex escape sequence".to_string()),
            }
        }

        Ok(value)
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> Token {
        // Skip whitespace and comments
        loop {
            self.skip_whitespace();

            // Check for comments
            if self.peek() == Some('/') {
                match self.peek_next() {
                    Some('/') => {
                        self.skip_line_comment();
                        continue;
                    }
                    Some('*') => {
                        let loc = self.location();
                        if let Err(err) = self.skip_block_comment() {
                            return Token::new(TokenKind::Error(err), loc);
                        }
                        continue;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }

        let loc = self.location();

        // Check for EOF
        let ch = match self.peek() {
            None => return Token::new(TokenKind::Eof, loc),
            Some(ch) => ch,
        };

        // Identifier or keyword
        if Self::is_identifier_start(ch) {
            let kind = self.read_identifier();
            return Token::new(kind, loc);
        }

        // Number
        if ch.is_ascii_digit() {
            let kind = match self.read_number() {
                Ok(k) => k,
                Err(err) => TokenKind::Error(err),
            };
            return Token::new(kind, loc);
        }

        // String
        if ch == '"' || ch == '\'' {
            let kind = match self.read_string(ch) {
                Ok(k) => k,
                Err(err) => TokenKind::Error(err),
            };
            return Token::new(kind, loc);
        }

        // Operators and punctuation
        self.consume();

        let kind = match ch {
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ';' => TokenKind::Semicolon,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            '~' => TokenKind::Tilde,
            '?' => {
                if self.peek() == Some('?') {
                    self.consume();
                    TokenKind::NullishCoalescing
                } else {
                    TokenKind::Question
                }
            }
            '.' => TokenKind::Dot,
            '+' => {
                match self.peek() {
                    Some('+') => {
                        self.consume();
                        TokenKind::PlusPlus
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::PlusAssign
                    }
                    _ => TokenKind::Plus,
                }
            }
            '-' => {
                match self.peek() {
                    Some('-') => {
                        self.consume();
                        TokenKind::MinusMinus
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::MinusAssign
                    }
                    _ => TokenKind::Minus,
                }
            }
            '*' => {
                match self.peek() {
                    Some('*') => {
                        self.consume();
                        TokenKind::StarStar
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::StarAssign
                    }
                    _ => TokenKind::Star,
                }
            }
            '/' => {
                if self.peek() == Some('=') {
                    self.consume();
                    TokenKind::SlashAssign
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                if self.peek() == Some('=') {
                    self.consume();
                    TokenKind::PercentAssign
                } else {
                    TokenKind::Percent
                }
            }
            '=' => {
                match self.peek() {
                    Some('=') => {
                        self.consume();
                        if self.peek() == Some('=') {
                            self.consume();
                            TokenKind::StrictEq
                        } else {
                            TokenKind::Eq
                        }
                    }
                    Some('>') => {
                        self.consume();
                        TokenKind::Arrow
                    }
                    _ => TokenKind::Assign,
                }
            }
            '!' => {
                match self.peek() {
                    Some('=') => {
                        self.consume();
                        if self.peek() == Some('=') {
                            self.consume();
                            TokenKind::StrictNotEq
                        } else {
                            TokenKind::NotEq
                        }
                    }
                    _ => TokenKind::Bang,
                }
            }
            '<' => {
                match self.peek() {
                    Some('<') => {
                        self.consume();
                        if self.peek() == Some('=') {
                            self.consume();
                            TokenKind::LtLtAssign
                        } else {
                            TokenKind::LtLt
                        }
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::LtEq
                    }
                    _ => TokenKind::Lt,
                }
            }
            '>' => {
                match self.peek() {
                    Some('>') => {
                        self.consume();
                        match self.peek() {
                            Some('>') => {
                                self.consume();
                                if self.peek() == Some('=') {
                                    self.consume();
                                    TokenKind::GtGtGtAssign
                                } else {
                                    TokenKind::GtGtGt
                                }
                            }
                            Some('=') => {
                                self.consume();
                                TokenKind::GtGtAssign
                            }
                            _ => TokenKind::GtGt,
                        }
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::GtEq
                    }
                    _ => TokenKind::Gt,
                }
            }
            '&' => {
                match self.peek() {
                    Some('&') => {
                        self.consume();
                        TokenKind::LogicalAnd
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::AmpersandAssign
                    }
                    _ => TokenKind::Ampersand,
                }
            }
            '|' => {
                match self.peek() {
                    Some('|') => {
                        self.consume();
                        TokenKind::LogicalOr
                    }
                    Some('=') => {
                        self.consume();
                        TokenKind::PipeAssign
                    }
                    _ => TokenKind::Pipe,
                }
            }
            '^' => {
                if self.peek() == Some('=') {
                    self.consume();
                    TokenKind::CaretAssign
                } else {
                    TokenKind::Caret
                }
            }
            _ => TokenKind::Error(format!("Unexpected character: '{}'", ch)),
        };

        Token::new(kind, loc)
    }

    /// Peeks at the next token without consuming it
    pub fn peek_token(&mut self) -> Token {
        let saved_pos = self.pos;
        let saved_line = self.line;
        let saved_column = self.column;

        let token = self.next_token();

        self.pos = saved_pos;
        self.line = saved_line;
        self.column = saved_column;

        token
    }

    /// Gets the current position for parser checkpointing
    pub fn pc(&self) -> usize {
        self.pos
    }

    /// Sets the position for parser restore (simplified implementation)
    pub fn set_pc(&mut self, pos: usize) {
        self.pos = pos;
        // Recalculate line/column (simplified - in production would track more state)
        self.line = 1;
        self.column = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("var let const function return");

        assert!(matches!(lexer.next_token().kind, TokenKind::Var));
        assert!(matches!(lexer.next_token().kind, TokenKind::Let));
        assert!(matches!(lexer.next_token().kind, TokenKind::Const));
        assert!(matches!(lexer.next_token().kind, TokenKind::Function));
        assert!(matches!(lexer.next_token().kind, TokenKind::Return));
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz $test _internal");

        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "foo"));
        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "bar_baz"));
        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "$test"));
        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "_internal"));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0.5 1e10 2.5e-3");

        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 42.0));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 3.14));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 0.5));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 1e10));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 2.5e-3));
    }

    #[test]
    fn test_hex_numbers() {
        let mut lexer = Lexer::new("0xFF 0x10 0xABCD");

        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 255.0));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 16.0));
        assert!(matches!(lexer.next_token().kind, TokenKind::Number(n) if n == 43981.0));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new(r#""hello" 'world' "foo\"bar""#);

        assert!(matches!(lexer.next_token().kind, TokenKind::String(ref s) if s == "hello"));
        assert!(matches!(lexer.next_token().kind, TokenKind::String(ref s) if s == "world"));
        assert!(matches!(lexer.next_token().kind, TokenKind::String(ref s) if s == "foo\"bar"));
    }

    #[test]
    fn test_string_escapes() {
        let mut lexer = Lexer::new(r#""line1\nline2" "tab\there""#);

        assert!(matches!(lexer.next_token().kind, TokenKind::String(ref s) if s == "line1\nline2"));
        assert!(matches!(lexer.next_token().kind, TokenKind::String(ref s) if s == "tab\there"));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / % == === != !== < > <= >= && || !");

        assert!(matches!(lexer.next_token().kind, TokenKind::Plus));
        assert!(matches!(lexer.next_token().kind, TokenKind::Minus));
        assert!(matches!(lexer.next_token().kind, TokenKind::Star));
        assert!(matches!(lexer.next_token().kind, TokenKind::Slash));
        assert!(matches!(lexer.next_token().kind, TokenKind::Percent));
        assert!(matches!(lexer.next_token().kind, TokenKind::Eq));
        assert!(matches!(lexer.next_token().kind, TokenKind::StrictEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::NotEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::StrictNotEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::Lt));
        assert!(matches!(lexer.next_token().kind, TokenKind::Gt));
        assert!(matches!(lexer.next_token().kind, TokenKind::LtEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::GtEq));
        assert!(matches!(lexer.next_token().kind, TokenKind::LogicalAnd));
        assert!(matches!(lexer.next_token().kind, TokenKind::LogicalOr));
        assert!(matches!(lexer.next_token().kind, TokenKind::Bang));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("foo // comment\nbar /* block */ baz");

        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "foo"));
        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "bar"));
        assert!(matches!(lexer.next_token().kind, TokenKind::Identifier(ref s) if s == "baz"));
    }

    #[test]
    fn test_location_tracking() {
        let mut lexer = Lexer::new("foo\nbar");

        let tok1 = lexer.next_token();
        assert_eq!(tok1.location.line, 1);
        assert_eq!(tok1.location.column, 1);

        let tok2 = lexer.next_token();
        assert_eq!(tok2.location.line, 2);
        assert_eq!(tok2.location.column, 1);
    }
}
