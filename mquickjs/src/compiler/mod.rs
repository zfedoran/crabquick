//! JavaScript compiler (lexer, parser, code generator)

pub mod lexer;
pub mod parser;
pub mod codegen;
pub mod debug;

// Re-exports
pub use lexer::{Lexer, Token};
pub use parser::Parser;
pub use codegen::CodeGenerator;
