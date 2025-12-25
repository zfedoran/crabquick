//! JavaScript compiler (lexer, parser, code generator)

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;
pub mod debug;

use alloc::string::String;
use alloc::vec::Vec;

// Re-exports
pub use lexer::{Lexer, Token, TokenKind, SourceLocation};
pub use ast::{Expr, Stmt, Program, Literal, BinaryOp, UnaryOp};
pub use parser::{Parser, ParseError};
pub use codegen::{CodeGenerator, CodeGenError};

/// Compilation error
#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    Parse(ParseError),
    CodeGen(CodeGenError),
}

impl From<ParseError> for CompileError {
    fn from(err: ParseError) -> Self {
        CompileError::Parse(err)
    }
}

impl From<CodeGenError> for CompileError {
    fn from(err: CodeGenError) -> Self {
        CompileError::CodeGen(err)
    }
}

/// Compiles JavaScript source code into bytecode
///
/// # Arguments
///
/// * `source` - JavaScript source code to compile
///
/// # Returns
///
/// * `Ok(bytecode)` - Compiled bytecode ready for execution
/// * `Err(error)` - Compilation error with location information
///
/// # Example
///
/// ```ignore
/// use crabquick::compiler::compile;
///
/// let bytecode = compile("2 + 3")?;
/// ```
pub fn compile(source: &str) -> Result<Vec<u8>, CompileError> {
    // Parse source into AST
    let parser = Parser::new(source);
    let program = parser.parse()?;

    // Generate bytecode from AST
    let generator = CodeGenerator::new();
    let bytecode = generator.generate(&program)?;

    Ok(bytecode)
}
