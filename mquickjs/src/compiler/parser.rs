//! JavaScript parser

use super::lexer::Lexer;

/// JavaScript parser
pub struct Parser<'a> {
    // TODO: Implement fields:
    // - lexer: Lexer<'a>
    // - current_token: Token
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> Parser<'a> {
    /// Creates a new parser
    pub fn new(_source: &'a str) -> Self {
        // TODO: Initialize lexer
        Parser {
            _marker: core::marker::PhantomData,
        }
    }

    /// Parses the source code
    pub fn parse(&mut self) {
        // TODO: Parse script
        // TODO: Generate bytecode
    }
}
