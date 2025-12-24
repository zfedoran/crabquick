//! JavaScript lexer/tokenizer

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// End of file
    Eof,
    /// Identifier
    Identifier(alloc::string::String),
    /// Number literal
    Number(f64),
    /// String literal
    String(alloc::string::String),
    /// Keywords and operators
    // TODO: Add all token types
}

/// JavaScript lexer
pub struct Lexer<'a> {
    // TODO: Implement fields:
    // - source: &'a str
    // - pos: usize
    // - line: u32
    // - column: u32
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer
    pub fn new(_source: &'a str) -> Self {
        Lexer {
            _marker: core::marker::PhantomData,
        }
    }

    /// Returns the next token
    pub fn next_token(&mut self) -> Token {
        // TODO: Tokenize next input
        Token::Eof
    }
}
