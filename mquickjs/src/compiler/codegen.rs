//! Code generation (AST to bytecode)
//!
//! Converts Abstract Syntax Tree nodes into bytecode instructions.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use crate::bytecode::{BytecodeWriter, Instruction, Opcode};
use crate::value::JSValue;
use super::ast::*;
use super::lexer::SourceLocation;

/// Code generation error
#[derive(Debug, Clone, PartialEq)]
pub struct CodeGenError {
    pub message: String,
    pub location: Option<SourceLocation>,
}

impl CodeGenError {
    fn new(message: String) -> Self {
        CodeGenError { message, location: None }
    }

    fn with_location(message: String, location: SourceLocation) -> Self {
        CodeGenError { message, location: Some(location) }
    }
}

/// Code generation result
pub type CodeGenResult<T> = Result<T, CodeGenError>;

/// Represents a constant value in the constant pool
#[derive(Debug, Clone, PartialEq)]
enum Constant {
    Number(f64),
    String(String),
    // Could add function bytecode, etc.
}

/// Label for forward jumps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LabelId(usize);

/// Variable binding information
#[derive(Debug, Clone)]
struct VarBinding {
    name: String,
    index: u8,
    kind: VarKind,
}

/// Scope for variable resolution
#[derive(Debug, Clone)]
struct Scope {
    bindings: Vec<VarBinding>,
    parent: Option<Box<Scope>>,
}

impl Scope {
    fn new() -> Self {
        Scope {
            bindings: Vec::new(),
            parent: None,
        }
    }

    fn with_parent(parent: Scope) -> Self {
        Scope {
            bindings: Vec::new(),
            parent: Some(Box::new(parent)),
        }
    }

    fn add_binding(&mut self, name: String, kind: VarKind) -> u8 {
        let index = self.bindings.len() as u8;
        self.bindings.push(VarBinding { name, index, kind });
        index
    }

    fn find_binding(&self, name: &str) -> Option<(u8, &VarKind)> {
        for binding in &self.bindings {
            if binding.name == name {
                return Some((binding.index, &binding.kind));
            }
        }

        if let Some(ref parent) = self.parent {
            parent.find_binding(name)
        } else {
            None
        }
    }
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
struct LoopContext {
    break_label: LabelId,
    continue_label: LabelId,
}

/// Code generator
pub struct CodeGenerator {
    writer: BytecodeWriter,
    constants: Vec<Constant>,
    labels: Vec<Option<usize>>, // Label ID -> bytecode offset
    scope: Scope,
    loop_stack: Vec<LoopContext>,
}

impl CodeGenerator {
    /// Creates a new code generator
    pub fn new() -> Self {
        CodeGenerator {
            writer: BytecodeWriter::new(),
            constants: Vec::new(),
            labels: Vec::new(),
            scope: Scope::new(),
            loop_stack: Vec::new(),
        }
    }

    /// Generates bytecode for a program
    pub fn generate(mut self, program: &Program) -> CodeGenResult<Vec<u8>> {
        // Generate code for all statements
        for stmt in &program.body {
            self.gen_stmt(stmt)?;
        }

        // Implicit return undefined at end
        self.emit_simple(Opcode::ReturnUndef);

        Ok(self.writer.finish())
    }

    /// Creates a new label
    fn create_label(&mut self) -> LabelId {
        let id = LabelId(self.labels.len());
        self.labels.push(None);
        id
    }

    /// Marks a label at the current position
    fn mark_label(&mut self, label: LabelId) {
        self.labels[label.0] = Some(self.writer.pc());
    }

    /// Emits a jump to a label (will be patched later)
    fn emit_jump(&mut self, opcode: Opcode, label: LabelId) {
        let patch_offset = self.writer.pc() + 1; // After opcode byte
        self.emit(Instruction::with_label(opcode, 0)); // Placeholder

        // Store for patching
        if self.labels[label.0].is_none() {
            // Forward jump - will patch in resolve_labels
        }
    }

    /// Resolves all forward jumps
    fn resolve_labels(&mut self) -> CodeGenResult<()> {
        // This is a simplified approach - in production we'd track which jumps need patching
        // For now, we'll emit jumps with immediate values when possible
        Ok(())
    }

    /// Adds a constant to the pool and returns its index
    fn add_constant(&mut self, constant: Constant) -> CodeGenResult<u16> {
        // Check if constant already exists
        for (i, c) in self.constants.iter().enumerate() {
            if c == &constant {
                return Ok(i as u16);
            }
        }

        let index = self.constants.len();
        if index > 0xFFFF {
            return Err(CodeGenError::new("Too many constants".to_string()));
        }

        self.constants.push(constant);
        Ok(index as u16)
    }

    /// Emits a simple instruction (no operands)
    fn emit_simple(&mut self, opcode: Opcode) {
        self.writer.emit(&Instruction::new(opcode));
    }

    /// Emits an instruction
    fn emit(&mut self, instruction: Instruction) {
        self.writer.emit(&instruction);
    }

    /// Generates bytecode for a statement
    fn gen_stmt(&mut self, stmt: &Stmt) -> CodeGenResult<()> {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.gen_expr(expr)?;
                self.emit_simple(Opcode::Drop); // Drop result
                Ok(())
            }

            Stmt::Block { stmts, .. } => {
                // Create new scope
                let old_scope = core::mem::replace(&mut self.scope, Scope::with_parent(self.scope.clone()));

                for stmt in stmts {
                    self.gen_stmt(stmt)?;
                }

                // Restore scope
                self.scope = old_scope;
                Ok(())
            }

            Stmt::VarDecl { kind, declarations, .. } => {
                for decl in declarations {
                    let index = self.scope.add_binding(decl.name.clone(), *kind);

                    if let Some(ref init) = decl.init {
                        self.gen_expr(init)?;
                        self.emit(Instruction::with_u8(Opcode::PutLoc, index));
                    } else {
                        // Initialize to undefined
                        self.emit_simple(Opcode::Undefined);
                        self.emit(Instruction::with_u8(Opcode::PutLoc, index));
                    }
                }
                Ok(())
            }

            Stmt::FunctionDecl { name, params, body, .. } => {
                // Add function to scope
                let index = self.scope.add_binding(name.clone(), VarKind::Var);

                // For now, create a stub - full function compilation would require
                // compiling the function body separately and storing as a constant
                self.emit_simple(Opcode::Undefined);
                self.emit(Instruction::with_u8(Opcode::PutLoc, index));

                Ok(())
            }

            Stmt::If { test, consequent, alternate, .. } => {
                // Compile test
                self.gen_expr(test)?;

                // Create labels
                let else_label = self.create_label();
                let end_label = self.create_label();

                // Jump to else if test is false
                let if_false_offset = self.writer.pc() + 1;
                self.emit(Instruction::with_label(Opcode::IfFalse, 0)); // Will patch

                // Compile consequent
                self.gen_stmt(consequent)?;

                if alternate.is_some() {
                    // Jump to end after consequent
                    let goto_offset = self.writer.pc() + 1;
                    self.emit(Instruction::with_label(Opcode::Goto, 0)); // Will patch

                    // Patch else jump
                    let else_pos = self.writer.pc();
                    self.writer.patch_i32(if_false_offset, (else_pos as i32) - (if_false_offset as i32) - 4);

                    // Compile alternate
                    self.gen_stmt(alternate.as_ref().unwrap())?;

                    // Patch end jump
                    let end_pos = self.writer.pc();
                    self.writer.patch_i32(goto_offset, (end_pos as i32) - (goto_offset as i32) - 4);
                } else {
                    // No else - just patch the if_false jump to end
                    let end_pos = self.writer.pc();
                    self.writer.patch_i32(if_false_offset, (end_pos as i32) - (if_false_offset as i32) - 4);
                }

                Ok(())
            }

            Stmt::While { test, body, .. } => {
                let loop_start = self.writer.pc();
                let break_label = self.create_label();
                let continue_label = self.create_label();

                self.loop_stack.push(LoopContext { break_label, continue_label });

                // Compile test
                self.gen_expr(test)?;

                // Jump to end if false
                let if_false_offset = self.writer.pc() + 1;
                self.emit(Instruction::with_label(Opcode::IfFalse, 0)); // Will patch

                // Compile body
                self.gen_stmt(body)?;

                // Jump back to start
                let goto_offset = self.writer.pc() + 1;
                let jump_dist = (loop_start as i32) - (goto_offset as i32) - 4;
                self.emit(Instruction::with_label(Opcode::Goto, jump_dist));

                // Patch break jump
                let end_pos = self.writer.pc();
                self.writer.patch_i32(if_false_offset, (end_pos as i32) - (if_false_offset as i32) - 4);

                self.loop_stack.pop();
                Ok(())
            }

            Stmt::For { init, test, update, body, .. } => {
                // Create new scope for loop variable
                let old_scope = core::mem::replace(&mut self.scope, Scope::with_parent(self.scope.clone()));

                // Compile init
                if let Some(ref init) = init {
                    match init {
                        ForInit::VarDecl { kind, declarations } => {
                            for decl in declarations {
                                let index = self.scope.add_binding(decl.name.clone(), *kind);
                                if let Some(ref init_expr) = decl.init {
                                    self.gen_expr(init_expr)?;
                                    self.emit(Instruction::with_u8(Opcode::PutLoc, index));
                                }
                            }
                        }
                        ForInit::Expr(expr) => {
                            self.gen_expr(expr)?;
                            self.emit_simple(Opcode::Drop);
                        }
                    }
                }

                let loop_start = self.writer.pc();
                let break_label = self.create_label();
                let continue_label = self.create_label();

                self.loop_stack.push(LoopContext { break_label, continue_label });

                // Compile test (if present)
                let if_false_offset = if let Some(ref test) = test {
                    self.gen_expr(test)?;
                    let offset = self.writer.pc() + 1;
                    self.emit(Instruction::with_label(Opcode::IfFalse, 0)); // Will patch
                    Some(offset)
                } else {
                    None
                };

                // Compile body
                self.gen_stmt(body)?;

                // Compile update
                if let Some(ref update) = update {
                    self.gen_expr(update)?;
                    self.emit_simple(Opcode::Drop);
                }

                // Jump back to start
                let goto_offset = self.writer.pc() + 1;
                let jump_dist = (loop_start as i32) - (goto_offset as i32) - 4;
                self.emit(Instruction::with_label(Opcode::Goto, jump_dist));

                // Patch test jump (if present)
                if let Some(offset) = if_false_offset {
                    let end_pos = self.writer.pc();
                    self.writer.patch_i32(offset, (end_pos as i32) - (offset as i32) - 4);
                }

                self.loop_stack.pop();
                self.scope = old_scope;
                Ok(())
            }

            Stmt::Return { argument, .. } => {
                if let Some(ref arg) = argument {
                    self.gen_expr(arg)?;
                    self.emit_simple(Opcode::Return);
                } else {
                    self.emit_simple(Opcode::ReturnUndef);
                }
                Ok(())
            }

            Stmt::Break { .. } => {
                if let Some(ctx) = self.loop_stack.last() {
                    // Jump to break label - for simplicity, we'll emit a placeholder
                    self.emit_simple(Opcode::ReturnUndef); // Stub
                }
                Ok(())
            }

            Stmt::Continue { .. } => {
                if let Some(ctx) = self.loop_stack.last() {
                    // Jump to continue label - for simplicity, we'll emit a placeholder
                    self.emit_simple(Opcode::Nop); // Stub
                }
                Ok(())
            }

            Stmt::Throw { argument, .. } => {
                self.gen_expr(argument)?;
                self.emit_simple(Opcode::Throw);
                Ok(())
            }

            Stmt::Try { .. } => {
                // Try/catch requires complex control flow - stub for now
                self.emit_simple(Opcode::Nop);
                Ok(())
            }

            Stmt::DoWhile { .. } | Stmt::ForIn { .. } | Stmt::Switch { .. } | Stmt::Empty { .. } => {
                // These are stubs for now
                Ok(())
            }
        }
    }

    /// Generates bytecode for an expression
    fn gen_expr(&mut self, expr: &Expr) -> CodeGenResult<()> {
        match expr {
            Expr::Literal(lit, _) => {
                self.gen_literal(lit)?;
                Ok(())
            }

            Expr::Identifier(name, loc) => {
                if let Some((index, _)) = self.scope.find_binding(name) {
                    self.emit(Instruction::with_u8(Opcode::GetLoc, index));
                } else {
                    // Global variable - for now, push undefined
                    self.emit_simple(Opcode::Undefined);
                }
                Ok(())
            }

            Expr::This(_) => {
                self.emit_simple(Opcode::PushThis);
                Ok(())
            }

            Expr::Binary { op, left, right, .. } => {
                // Compile left operand
                self.gen_expr(left)?;

                // Compile right operand
                self.gen_expr(right)?;

                // Emit operator
                let opcode = match op {
                    BinaryOp::Add => Opcode::Add,
                    BinaryOp::Sub => Opcode::Sub,
                    BinaryOp::Mul => Opcode::Mul,
                    BinaryOp::Div => Opcode::Div,
                    BinaryOp::Mod => Opcode::Mod,
                    BinaryOp::Pow => Opcode::Pow,
                    BinaryOp::Eq => Opcode::Eq,
                    BinaryOp::NotEq => Opcode::Neq,
                    BinaryOp::StrictEq => Opcode::StrictEq,
                    BinaryOp::StrictNotEq => Opcode::StrictNeq,
                    BinaryOp::Lt => Opcode::Lt,
                    BinaryOp::LtEq => Opcode::Lte,
                    BinaryOp::Gt => Opcode::Gt,
                    BinaryOp::GtEq => Opcode::Gte,
                    BinaryOp::BitAnd => Opcode::And,
                    BinaryOp::BitOr => Opcode::Or,
                    BinaryOp::BitXor => Opcode::Xor,
                    BinaryOp::LeftShift => Opcode::Shl,
                    BinaryOp::RightShift => Opcode::Sar,
                    BinaryOp::UnsignedRightShift => Opcode::Shr,
                    BinaryOp::In => Opcode::In,
                    BinaryOp::InstanceOf => Opcode::Instanceof,
                    BinaryOp::LogicalAnd | BinaryOp::LogicalOr | BinaryOp::NullishCoalescing => {
                        // These require short-circuit evaluation - handled separately
                        return Ok(());
                    }
                };

                self.emit_simple(opcode);
                Ok(())
            }

            Expr::Unary { op, arg, .. } => {
                self.gen_expr(arg)?;

                let opcode = match op {
                    UnaryOp::Plus => Opcode::Plus,
                    UnaryOp::Minus => Opcode::Neg,
                    UnaryOp::LogicalNot => Opcode::LNot,
                    UnaryOp::BitwiseNot => Opcode::Not,
                    UnaryOp::TypeOf => Opcode::TypeOf,
                    UnaryOp::Void => Opcode::Void,
                    UnaryOp::Delete => Opcode::Delete,
                };

                self.emit_simple(opcode);
                Ok(())
            }

            Expr::Update { op, arg, prefix, .. } => {
                // For simplicity, we'll just increment/decrement
                self.gen_expr(arg)?;

                let opcode = match (op, prefix) {
                    (UpdateOp::Inc, true) => Opcode::Inc,
                    (UpdateOp::Dec, true) => Opcode::Dec,
                    (UpdateOp::Inc, false) => Opcode::PostInc,
                    (UpdateOp::Dec, false) => Opcode::PostDec,
                };

                self.emit_simple(opcode);
                Ok(())
            }

            Expr::Assignment { op, left, right, .. } => {
                // Compile right side
                self.gen_expr(right)?;

                // Handle assignment target
                match left.as_ref() {
                    Expr::Identifier(name, _) => {
                        if let Some((index, _)) = self.scope.find_binding(name) {
                            let opcode = if matches!(op, AssignOp::Assign) {
                                Opcode::SetLoc
                            } else {
                                // Compound assignment - need to load, operate, store
                                Opcode::PutLoc
                            };
                            self.emit(Instruction::with_u8(opcode, index));
                        }
                    }
                    _ => {
                        // Member expressions, etc. - stub for now
                        self.emit_simple(Opcode::Drop);
                    }
                }

                Ok(())
            }

            Expr::Conditional { test, consequent, alternate, .. } => {
                // Compile test
                self.gen_expr(test)?;

                // Create labels
                let else_label = self.create_label();
                let end_label = self.create_label();

                // Jump to else if false
                let if_false_offset = self.writer.pc() + 1;
                self.emit(Instruction::with_label(Opcode::IfFalse, 0)); // Will patch

                // Compile consequent
                self.gen_expr(consequent)?;

                // Jump to end
                let goto_offset = self.writer.pc() + 1;
                self.emit(Instruction::with_label(Opcode::Goto, 0)); // Will patch

                // Patch else jump
                let else_pos = self.writer.pc();
                self.writer.patch_i32(if_false_offset, (else_pos as i32) - (if_false_offset as i32) - 4);

                // Compile alternate
                self.gen_expr(alternate)?;

                // Patch end jump
                let end_pos = self.writer.pc();
                self.writer.patch_i32(goto_offset, (end_pos as i32) - (goto_offset as i32) - 4);

                Ok(())
            }

            Expr::Call { callee, args, .. } => {
                // Compile callee
                self.gen_expr(callee)?;

                // Compile arguments
                for arg in args {
                    self.gen_expr(arg)?;
                }

                // Emit call
                let argc = args.len() as u8;
                self.emit(Instruction::with_u8(Opcode::Call, argc));

                Ok(())
            }

            Expr::Member { object, property, computed, .. } => {
                // Compile object
                self.gen_expr(object)?;

                if *computed {
                    // Compile property expression
                    self.gen_expr(property)?;
                    self.emit_simple(Opcode::GetArrayEl);
                } else {
                    // Static property access - would need atom table
                    self.emit_simple(Opcode::Undefined);
                }

                Ok(())
            }

            Expr::Array { elements, .. } => {
                // Create array
                let count = elements.len() as u8;
                self.emit(Instruction::with_u8(Opcode::Array, count));

                // For now, just push undefined for each element
                for _ in elements {
                    self.emit_simple(Opcode::Undefined);
                }

                Ok(())
            }

            Expr::Object { properties, .. } => {
                // Create object
                let count = properties.len() as u8;
                self.emit(Instruction::with_u8(Opcode::Object, count));

                // Stub - would need to emit property definitions
                Ok(())
            }

            Expr::Sequence { exprs, .. } => {
                for (i, expr) in exprs.iter().enumerate() {
                    self.gen_expr(expr)?;
                    // Drop all but last
                    if i < exprs.len() - 1 {
                        self.emit_simple(Opcode::Drop);
                    }
                }
                Ok(())
            }

            Expr::New { .. } | Expr::Function { .. } | Expr::Arrow { .. } => {
                // These are stubs for now
                self.emit_simple(Opcode::Undefined);
                Ok(())
            }
        }
    }

    /// Generates bytecode for a literal
    fn gen_literal(&mut self, lit: &Literal) -> CodeGenResult<()> {
        match lit {
            Literal::Number(n) => {
                // Try to emit as inline integer
                if n.floor() == *n && *n >= -1.0 && *n <= 7.0 {
                    let opcode = match *n as i32 {
                        -1 => Opcode::PushMinus1,
                        0 => Opcode::Push0,
                        1 => Opcode::Push1,
                        2 => Opcode::Push2,
                        3 => Opcode::Push3,
                        4 => Opcode::Push4,
                        5 => Opcode::Push5,
                        6 => Opcode::Push6,
                        7 => Opcode::Push7,
                        _ => unreachable!(),
                    };
                    self.emit_simple(opcode);
                } else if n.floor() == *n && *n >= i8::MIN as f64 && *n <= i8::MAX as f64 {
                    self.emit(Instruction::with_i8(Opcode::PushI8, *n as i8));
                } else if n.floor() == *n && *n >= i16::MIN as f64 && *n <= i16::MAX as f64 {
                    self.emit(Instruction::with_i16(Opcode::PushI16, *n as i16));
                } else if n.floor() == *n && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    self.emit(Instruction::with_i32(Opcode::PushI32, *n as i32));
                } else {
                    // Add to constant pool
                    let index = self.add_constant(Constant::Number(*n))?;
                    if index <= 255 {
                        self.emit(Instruction::with_const8(Opcode::PushConst8, index as u8));
                    } else {
                        self.emit(Instruction::with_const16(Opcode::PushConst16, index));
                    }
                }
            }

            Literal::String(s) => {
                if s.is_empty() {
                    self.emit_simple(Opcode::PushEmptyString);
                } else {
                    let index = self.add_constant(Constant::String(s.clone()))?;
                    if index <= 255 {
                        self.emit(Instruction::with_const8(Opcode::PushConst8, index as u8));
                    } else {
                        self.emit(Instruction::with_const16(Opcode::PushConst16, index));
                    }
                }
            }

            Literal::Boolean(b) => {
                self.emit_simple(if *b { Opcode::PushTrue } else { Opcode::PushFalse });
            }

            Literal::Null => {
                self.emit_simple(Opcode::Null);
            }

            Literal::Undefined => {
                self.emit_simple(Opcode::Undefined);
            }
        }

        Ok(())
    }
}

impl Default for CodeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Parser;

    #[test]
    fn test_gen_number() {
        let mut gen = CodeGenerator::new();
        gen.gen_literal(&Literal::Number(42.0)).unwrap();

        let bytecode = gen.writer.finish();
        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_gen_binary_expr() {
        let parser = Parser::new("2 + 3");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_gen_var_decl() {
        let parser = Parser::new("var x = 10;");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        assert!(!bytecode.is_empty());
    }

    #[test]
    fn test_gen_function() {
        let parser = Parser::new("function add(a, b) { return a + b; }");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        assert!(!bytecode.is_empty());
    }
}
