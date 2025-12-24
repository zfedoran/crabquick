//! Code generation (AST to bytecode)

/// Code generator
pub struct CodeGenerator {
    // TODO: Implement fields:
    // - bytecode_writer: BytecodeWriter
    // - constant_pool: ConstantPool
    // - labels: Vec<LabelId>
    _placeholder: u8,
}

impl CodeGenerator {
    /// Creates a new code generator
    pub fn new() -> Self {
        CodeGenerator {
            _placeholder: 0,
        }
    }

    /// Generates bytecode for an expression
    pub fn gen_expr(&mut self) {
        // TODO: Generate bytecode
    }

    /// Generates bytecode for a statement
    pub fn gen_stmt(&mut self) {
        // TODO: Generate bytecode
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}
