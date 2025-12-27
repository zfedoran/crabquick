//! Code generation (AST to bytecode)
//!
//! Converts Abstract Syntax Tree nodes into bytecode instructions.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use crate::bytecode::{BytecodeWriter, Instruction, Opcode, ConstantPool};
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

/// Label for forward jumps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LabelId(usize);

/// Variable binding information
#[derive(Debug, Clone)]
struct VarBinding {
    name: String,
    index: u8,
    kind: VarKind,
    /// True if this variable is captured by a closure
    is_captured: bool,
}

/// Where a variable is located
#[derive(Debug, Clone, Copy, PartialEq)]
enum VarLocation {
    /// Local variable at given index
    Local(u8),
    /// Captured variable at given var_ref index
    Captured(u8),
    /// Global variable (use atom table)
    Global,
}

/// Captured variable info for closure creation
#[derive(Debug, Clone)]
struct CapturedVar {
    /// Name of the variable
    name: String,
    /// Index in parent's local or captured vars
    parent_index: u8,
    /// True if captured from parent's captured vars, false if from parent's locals
    from_capture: bool,
}

/// Scope for variable resolution
#[derive(Debug, Clone)]
struct Scope {
    bindings: Vec<VarBinding>,
    parent: Option<Box<Scope>>,
    /// Base index for this scope (cumulative from parent scopes)
    base_index: u8,
}

impl Scope {
    fn new() -> Self {
        Scope {
            bindings: Vec::new(),
            parent: None,
            base_index: 0,
        }
    }

    fn with_parent(parent: Scope) -> Self {
        // Child scope starts after parent's variables
        let base_index = parent.next_index();
        Scope {
            bindings: Vec::new(),
            parent: Some(Box::new(parent)),
            base_index,
        }
    }

    /// Get the next available index (base + current bindings count)
    fn next_index(&self) -> u8 {
        self.base_index + self.bindings.len() as u8
    }

    fn add_binding(&mut self, name: String, kind: VarKind) -> u8 {
        let index = self.next_index();
        self.bindings.push(VarBinding { name, index, kind, is_captured: false });
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

    /// Mark a binding as captured by name
    fn mark_captured(&mut self, name: &str) -> bool {
        for binding in &mut self.bindings {
            if binding.name == name {
                binding.is_captured = true;
                return true;
            }
        }
        if let Some(ref mut parent) = self.parent {
            parent.mark_captured(name)
        } else {
            false
        }
    }
}

/// Loop context for break/continue
#[derive(Debug, Clone)]
struct LoopContext {
    break_label: LabelId,
    continue_label: LabelId,
}

/// Function bytecode entry
#[derive(Debug, Clone)]
struct FunctionBytecode {
    bytecode: Vec<u8>,
    param_count: u8,
    local_count: u8,
    /// Variables captured from outer scope (for closures)
    captured_vars: Vec<CapturedVar>,
}

/// Code generator
pub struct CodeGenerator {
    writer: BytecodeWriter,
    constants: ConstantPool,
    /// Track which constants are f64 (true) vs JSValue (false)
    const_is_f64: Vec<bool>,
    labels: Vec<Option<usize>>, // Label ID -> bytecode offset
    scope: Scope,
    loop_stack: Vec<LoopContext>,
    /// Atom table for identifier names (maps string to sequential index)
    atom_table: BTreeMap<String, u16>,
    /// Atom strings in order (index -> string)
    atom_strings: Vec<String>,
    /// Function bytecode table
    function_bytecodes: Vec<FunctionBytecode>,
    /// Variables captured from parent (when compiling a closure)
    captured_vars: Vec<CapturedVar>,
    /// Parent scope for closure variable lookup (reference to outer CodeGenerator's scope)
    /// This is a flat list of accessible outer variables with their original indices
    outer_vars: Vec<(String, u8)>,
    /// Is this a closure (has access to outer scope)?
    is_closure: bool,
}

impl CodeGenerator {
    /// Creates a new code generator
    pub fn new() -> Self {
        CodeGenerator {
            writer: BytecodeWriter::new(),
            constants: ConstantPool::new(),
            const_is_f64: Vec::new(),
            labels: Vec::new(),
            scope: Scope::new(),
            loop_stack: Vec::new(),
            atom_table: BTreeMap::new(),
            atom_strings: Vec::new(),
            function_bytecodes: Vec::new(),
            captured_vars: Vec::new(),
            outer_vars: Vec::new(),
            is_closure: false,
        }
    }

    /// Creates a new code generator for a closure with access to outer variables
    fn new_for_closure(outer_vars: Vec<(String, u8)>) -> Self {
        CodeGenerator {
            writer: BytecodeWriter::new(),
            constants: ConstantPool::new(),
            const_is_f64: Vec::new(),
            labels: Vec::new(),
            scope: Scope::new(),
            loop_stack: Vec::new(),
            atom_table: BTreeMap::new(),
            atom_strings: Vec::new(),
            function_bytecodes: Vec::new(),
            captured_vars: Vec::new(),
            outer_vars,
            is_closure: true,
        }
    }

    /// Gets or creates an atom for an identifier name
    /// Returns a sequential index (0, 1, 2, ...) for each unique identifier
    fn get_or_create_atom(&mut self, name: &str) -> u16 {
        if let Some(&atom_idx) = self.atom_table.get(name) {
            return atom_idx;
        }

        let atom_idx = self.atom_strings.len() as u16;
        self.atom_strings.push(name.to_string());
        self.atom_table.insert(name.to_string(), atom_idx);
        atom_idx
    }

    /// Resolves a variable to its location (local, captured, or global)
    /// If it's an outer variable that hasn't been captured yet, adds it to captured_vars
    fn resolve_variable(&mut self, name: &str) -> VarLocation {
        // First check local scope
        if let Some((index, _kind)) = self.scope.find_binding(name) {
            return VarLocation::Local(index);
        }

        // Check if this is a closure and we can access outer variables
        if self.is_closure {
            // Check if already captured
            for (i, cv) in self.captured_vars.iter().enumerate() {
                if cv.name == name {
                    return VarLocation::Captured(i as u8);
                }
            }

            // Check if available in outer scope
            for (outer_name, outer_index) in &self.outer_vars {
                if outer_name == name {
                    // Add to captured vars
                    let capture_index = self.captured_vars.len() as u8;
                    self.captured_vars.push(CapturedVar {
                        name: name.to_string(),
                        parent_index: *outer_index,
                        from_capture: false, // TODO: track if parent is also a closure
                    });
                    return VarLocation::Captured(capture_index);
                }
            }
        }

        // Not found - it's a global
        VarLocation::Global
    }

    /// Generates bytecode for a program
    pub fn generate(mut self, program: &Program) -> CodeGenResult<Vec<u8>> {
        let len = program.body.len();

        // Generate code for all statements
        for (i, stmt) in program.body.iter().enumerate() {
            let is_last = i == len - 1;
            self.gen_stmt_with_context(stmt, is_last)?;
        }

        // Implicit return undefined at end if program is empty
        if len == 0 {
            self.emit_simple(Opcode::ReturnUndef);
        }

        // Serialize the constant pool, atom table, function table, and bytecode
        // Format: [constant_count: u16][(type: u8, value: usize)...]
        //         [atom_count: u16][(len: u16, string_bytes)...]
        //         [function_count: u16][(param_count: u8, local_count: u8, bytecode_len: u32, bytecode_bytes)...]
        //         [bytecode...]
        // Type: 0 = f64 bits, 1 = JSValue
        let mut result = Vec::new();

        // Write constant count
        let const_count = self.constants.len() as u16;
        result.extend_from_slice(&const_count.to_le_bytes());

        // Write constants with type tags
        for i in 0..self.constants.len() {
            if let Some(value) = self.constants.get(i as u16) {
                let raw = value.as_raw();
                let is_f64 = self.const_is_f64.get(i).copied().unwrap_or(false);

                result.push(if is_f64 { 0u8 } else { 1u8 });
                result.extend_from_slice(&raw.to_le_bytes());
            }
        }

        // Write atom count
        let atom_count = self.atom_strings.len() as u16;
        result.extend_from_slice(&atom_count.to_le_bytes());

        // Write atom strings
        for atom_str in &self.atom_strings {
            let bytes = atom_str.as_bytes();
            let len = bytes.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(bytes);
        }

        // Write function count and function bytecodes
        let func_count = self.function_bytecodes.len() as u16;
        result.extend_from_slice(&func_count.to_le_bytes());

        for func in &self.function_bytecodes {
            result.push(func.param_count);
            result.push(func.local_count);
            let bytecode_len = func.bytecode.len() as u32;
            result.extend_from_slice(&bytecode_len.to_le_bytes());
            result.extend_from_slice(&func.bytecode);
        }

        // Append the main bytecode
        result.extend_from_slice(self.writer.as_slice());

        Ok(result)
    }

    /// Compiles a function body into bytecode
    ///
    /// Creates a new code generator with a fresh scope containing parameters,
    /// compiles the function body, and returns the complete bytecode plus local count
    /// and captured variables.
    fn compile_function_body(&mut self, params: &[String], body: &[Stmt]) -> CodeGenResult<(Vec<u8>, u8, Vec<CapturedVar>)> {
        // Collect current scope bindings to make accessible to the function
        let mut outer_vars = Vec::new();
        self.collect_scope_vars(&self.scope.clone(), &mut outer_vars);

        // Create a new code generator for the function with access to outer vars
        let mut func_gen = CodeGenerator::new_for_closure(outer_vars);

        // Create a new scope and add parameters as local variables
        for param in params {
            func_gen.scope.add_binding(param.clone(), VarKind::Var);
        }

        // Compile all statements in the function body
        let last_idx = body.len().saturating_sub(1);
        for (i, stmt) in body.iter().enumerate() {
            let is_last = i == last_idx;

            // Check if this is a return statement
            if matches!(stmt, Stmt::Return { .. }) {
                func_gen.gen_stmt(stmt)?;
                // Return statement already emits Return opcode
            } else if is_last && matches!(stmt, Stmt::Expression { .. }) {
                // Last expression - don't return its value, just drop it
                func_gen.gen_stmt(stmt)?;
            } else {
                func_gen.gen_stmt(stmt)?;
            }
        }

        // If the function doesn't end with an explicit return, emit ReturnUndef
        if body.is_empty() || !matches!(body.last(), Some(Stmt::Return { .. })) {
            func_gen.emit_simple(Opcode::ReturnUndef);
        }

        // Get the local count (includes params and local vars)
        let local_count = func_gen.scope.bindings.len() as u8;

        // Get captured vars before consuming func_gen
        let captured_vars = func_gen.captured_vars.clone();

        // Generate the complete bytecode (includes constant pool and atom table)
        Ok((func_gen.generate_raw()?, local_count, captured_vars))
    }

    /// Collects all variable bindings from a scope hierarchy
    fn collect_scope_vars(&self, scope: &Scope, vars: &mut Vec<(String, u8)>) {
        for binding in &scope.bindings {
            vars.push((binding.name.clone(), binding.index));
        }
        if let Some(ref parent) = scope.parent {
            self.collect_scope_vars(parent, vars);
        }
    }

    /// Generates raw bytecode without wrapping in a Program
    fn generate_raw(self) -> CodeGenResult<Vec<u8>> {
        let mut result = Vec::new();

        // Write constant count
        let const_count = self.constants.len() as u16;
        result.extend_from_slice(&const_count.to_le_bytes());

        // Write constants with type tags
        for i in 0..self.constants.len() {
            if let Some(value) = self.constants.get(i as u16) {
                let raw = value.as_raw();
                let is_f64 = self.const_is_f64.get(i).copied().unwrap_or(false);

                result.push(if is_f64 { 0u8 } else { 1u8 });
                result.extend_from_slice(&raw.to_le_bytes());
            }
        }

        // Write atom count
        let atom_count = self.atom_strings.len() as u16;
        result.extend_from_slice(&atom_count.to_le_bytes());

        // Write atom strings
        for atom_str in &self.atom_strings {
            let bytes = atom_str.as_bytes();
            let len = bytes.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(bytes);
        }

        // Write function count (0 for simple functions without nested functions)
        let func_count = self.function_bytecodes.len() as u16;
        result.extend_from_slice(&func_count.to_le_bytes());

        // Write function bytecodes (if any nested functions)
        for func in &self.function_bytecodes {
            result.push(func.param_count);
            result.push(func.local_count);
            let bytecode_len = func.bytecode.len() as u32;
            result.extend_from_slice(&bytecode_len.to_le_bytes());
            result.extend_from_slice(&func.bytecode);
        }

        // Append the bytecode
        result.extend_from_slice(self.writer.as_slice());

        Ok(result)
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


    /// Emits a simple instruction (no operands)
    fn emit_simple(&mut self, opcode: Opcode) {
        self.writer.emit(&Instruction::new(opcode));
    }

    /// Emits an instruction
    fn emit(&mut self, instruction: Instruction) {
        self.writer.emit(&instruction);
    }

    /// Generates bytecode for a statement with context about position
    fn gen_stmt_with_context(&mut self, stmt: &Stmt, is_last: bool) -> CodeGenResult<()> {
        match stmt {
            Stmt::Expression { expr, .. } => {
                self.gen_expr(expr)?;
                if is_last {
                    // Last expression in program - return its value
                    self.emit_simple(Opcode::Return);
                } else {
                    // Not last - drop the result
                    self.emit_simple(Opcode::Drop);
                }
                Ok(())
            }
            _ => {
                // For non-expression statements, use normal gen_stmt
                // and emit ReturnUndef after if it's the last statement
                self.gen_stmt(stmt)?;
                if is_last {
                    self.emit_simple(Opcode::ReturnUndef);
                }
                Ok(())
            }
        }
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
                let new_scope = Scope::with_parent(self.scope.clone());
                let old_scope = core::mem::replace(&mut self.scope, new_scope);

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
                // Compile function body to bytecode
                let (func_bytecode, local_count, captured_vars) = self.compile_function_body(params, body)?;
                let param_count = params.len() as u8;
                let has_captures = !captured_vars.is_empty();

                // Add to function table
                let func_index = self.function_bytecodes.len() as u16;
                self.function_bytecodes.push(FunctionBytecode {
                    bytecode: func_bytecode,
                    param_count,
                    local_count,
                    captured_vars: captured_vars.clone(),
                });

                if has_captures {
                    // Emit FClosure which creates a closure with captured variables
                    // Format: FClosure func_idx, captured_count, [local_idx...]
                    self.emit(Instruction::with_u8(Opcode::FClosure, func_index as u8));
                    self.writer.emit_u8(captured_vars.len() as u8);
                    for cv in &captured_vars {
                        self.writer.emit_u8(cv.parent_index);
                    }
                } else {
                    // No captures - emit regular PushFunc
                    if func_index <= 255 {
                        self.emit(Instruction::with_u8(Opcode::PushFunc8, func_index as u8));
                    } else {
                        self.emit(Instruction::with_u16(Opcode::PushFunc, func_index));
                    }
                }

                // Check if we're at global scope (no parent)
                if self.scope.parent.is_none() {
                    // Global scope - use PutGlobal
                    let atom_id = self.get_or_create_atom(name);
                    if atom_id <= 255 {
                        self.emit(Instruction::with_atom8(Opcode::PutGlobal8, atom_id as u8));
                    } else {
                        self.emit(Instruction::with_atom16(Opcode::PutGlobal16, atom_id));
                    }
                } else {
                    // Local scope - add to scope and use PutLoc
                    let index = self.scope.add_binding(name.clone(), VarKind::Var);
                    self.emit(Instruction::with_u8(Opcode::PutLoc, index));
                }

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
                let new_scope = Scope::with_parent(self.scope.clone());
                let old_scope = core::mem::replace(&mut self.scope, new_scope);

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

            Expr::Identifier(name, _loc) => {
                match self.resolve_variable(name) {
                    VarLocation::Local(index) => {
                        self.emit(Instruction::with_u8(Opcode::GetLoc, index));
                    }
                    VarLocation::Captured(index) => {
                        self.emit(Instruction::with_u8(Opcode::GetVarRef, index));
                    }
                    VarLocation::Global => {
                        // Global variable - emit GetGlobal
                        let atom_id = self.get_or_create_atom(name);
                        if atom_id <= 255 {
                            self.emit(Instruction::with_atom8(Opcode::GetGlobal8, atom_id as u8));
                        } else {
                            self.emit(Instruction::with_atom16(Opcode::GetGlobal16, atom_id as u16));
                        }
                    }
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
                // Increment/decrement operators need to:
                // 1. Get the current value
                // 2. Compute the new value (old +/- 1)
                // 3. Store the new value back
                // 4. Leave either old (postfix) or new (prefix) on stack

                let add_opcode = match op {
                    UpdateOp::Inc => Opcode::Add,
                    UpdateOp::Dec => Opcode::Sub,
                };

                match arg.as_ref() {
                    Expr::Identifier(name, _) => {
                        match self.resolve_variable(name) {
                            VarLocation::Local(index) => {
                                // Local variable
                                if *prefix {
                                    // ++i or --i: get, add/sub 1, store (leave new value)
                                    self.emit(Instruction::with_u8(Opcode::GetLoc, index));
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    self.emit(Instruction::with_u8(Opcode::SetLoc, index));
                                } else {
                                    // i++ or i--: get, dup, add/sub 1, store (leave old value)
                                    self.emit(Instruction::with_u8(Opcode::GetLoc, index));
                                    self.emit_simple(Opcode::Dup);
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    self.emit(Instruction::with_u8(Opcode::PutLoc, index));
                                }
                            }
                            VarLocation::Captured(index) => {
                                // Captured variable
                                if *prefix {
                                    // ++i or --i: get, add/sub 1, store (leave new value)
                                    self.emit(Instruction::with_u8(Opcode::GetVarRef, index));
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    self.emit(Instruction::with_u8(Opcode::SetVarRef, index));
                                } else {
                                    // i++ or i--: get, dup, add/sub 1, store (leave old value)
                                    self.emit(Instruction::with_u8(Opcode::GetVarRef, index));
                                    self.emit_simple(Opcode::Dup);
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    self.emit(Instruction::with_u8(Opcode::PutVarRef, index));
                                }
                            }
                            VarLocation::Global => {
                                // Global variable
                                let atom_id = self.get_or_create_atom(name);
                                let get_op = if atom_id <= 255 { Opcode::GetGlobal8 } else { Opcode::GetGlobal16 };

                                if *prefix {
                                    // ++x or --x (global): get, add/sub 1, store (leave new value)
                                    if atom_id <= 255 {
                                        self.emit(Instruction::with_u8(get_op, atom_id as u8));
                                    } else {
                                        self.emit(Instruction::with_u16(get_op, atom_id as u16));
                                    }
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    // SetGlobal leaves value on stack
                                    if atom_id <= 255 {
                                        self.emit(Instruction::with_u8(Opcode::SetGlobal8, atom_id as u8));
                                    } else {
                                        self.emit(Instruction::with_u16(Opcode::SetGlobal16, atom_id as u16));
                                    }
                                } else {
                                    // x++ or x-- (global): get, dup, add/sub 1, store (leave old value)
                                    if atom_id <= 255 {
                                        self.emit(Instruction::with_u8(get_op, atom_id as u8));
                                    } else {
                                        self.emit(Instruction::with_u16(get_op, atom_id as u16));
                                    }
                                    self.emit_simple(Opcode::Dup);
                                    self.emit_simple(Opcode::Push1);
                                    self.emit_simple(add_opcode);
                                    // PutGlobal pops the value
                                    if atom_id <= 255 {
                                        self.emit(Instruction::with_u8(Opcode::PutGlobal8, atom_id as u8));
                                    } else {
                                        self.emit(Instruction::with_u16(Opcode::PutGlobal16, atom_id as u16));
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Property access and other lvalues not yet supported
                        // Fall back to the simple (broken) behavior for now
                        self.gen_expr(arg)?;
                        let opcode = match (op, prefix) {
                            (UpdateOp::Inc, true) => Opcode::Inc,
                            (UpdateOp::Dec, true) => Opcode::Dec,
                            (UpdateOp::Inc, false) => Opcode::PostInc,
                            (UpdateOp::Dec, false) => Opcode::PostDec,
                        };
                        self.emit_simple(opcode);
                    }
                }
                Ok(())
            }

            Expr::Assignment { op, left, right, .. } => {
                // Compile right side
                self.gen_expr(right)?;

                // Handle assignment target
                match left.as_ref() {
                    Expr::Identifier(name, _) => {
                        match self.resolve_variable(name) {
                            VarLocation::Local(index) => {
                                // Local variable
                                let opcode = if matches!(op, AssignOp::Assign) {
                                    Opcode::SetLoc
                                } else {
                                    // Compound assignment - need to load, operate, store
                                    Opcode::PutLoc
                                };
                                self.emit(Instruction::with_u8(opcode, index));
                            }
                            VarLocation::Captured(index) => {
                                // Captured variable
                                let opcode = if matches!(op, AssignOp::Assign) {
                                    Opcode::SetVarRef
                                } else {
                                    // Compound assignment
                                    Opcode::PutVarRef
                                };
                                self.emit(Instruction::with_u8(opcode, index));
                            }
                            VarLocation::Global => {
                                // Global variable
                                let atom_id = self.get_or_create_atom(name);
                                let opcode = if matches!(op, AssignOp::Assign) {
                                    // Use SetGlobal to return the value
                                    if atom_id <= 255 {
                                        Opcode::SetGlobal8
                                    } else {
                                        Opcode::SetGlobal16
                                    }
                                } else {
                                    // Compound assignment - use PutGlobal (doesn't return value)
                                    if atom_id <= 255 {
                                        Opcode::PutGlobal8
                                    } else {
                                        Opcode::PutGlobal16
                                    }
                                };

                                if atom_id <= 255 {
                                    self.emit(Instruction::with_atom8(opcode, atom_id as u8));
                                } else {
                                    self.emit(Instruction::with_atom16(opcode, atom_id as u16));
                                }
                            }
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
                // Check if it's a method call (callee is a member expression)
                let is_method_call = matches!(**callee, Expr::Member { .. });

                if is_method_call {
                    // For method calls: Math.abs(-5)
                    // We need to emit: obj, func, args... then CallMethod
                    if let Expr::Member { object, property, computed, .. } = &**callee {
                        // Emit object (for 'this' binding)
                        self.gen_expr(object)?;

                        // Duplicate object on stack for property access
                        self.emit_simple(Opcode::Dup);

                        // Get the method
                        if *computed {
                            self.gen_expr(property)?;
                            self.emit_simple(Opcode::GetArrayEl);
                        } else {
                            // Static property access
                            if let Expr::Identifier(name, _) = &**property {
                                let atom_idx = self.get_or_create_atom(name);
                                if atom_idx < 256 {
                                    self.emit(Instruction::with_atom8(Opcode::GetField8, atom_idx as u8));
                                } else {
                                    self.emit(Instruction::with_u16(Opcode::GetField, atom_idx));
                                }
                            } else {
                                self.emit_simple(Opcode::Undefined);
                            }
                        }

                        // Compile arguments
                        for arg in args {
                            self.gen_expr(arg)?;
                        }

                        // Emit method call
                        let argc = args.len() as u8;
                        self.emit(Instruction::with_u8(Opcode::CallMethod, argc));
                    }
                } else {
                    // Regular function call
                    // Compile callee
                    self.gen_expr(callee)?;

                    // Compile arguments
                    for arg in args {
                        self.gen_expr(arg)?;
                    }

                    // Emit call
                    let argc = args.len() as u8;
                    self.emit(Instruction::with_u8(Opcode::Call, argc));
                }

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
                    // Static property access
                    if let Expr::Identifier(name, _) = &**property {
                        let atom_idx = self.get_or_create_atom(name);
                        if atom_idx < 256 {
                            self.emit(Instruction::with_atom8(Opcode::GetField8, atom_idx as u8));
                        } else {
                            self.emit(Instruction::with_u16(Opcode::GetField, atom_idx));
                        }
                    } else {
                        self.emit_simple(Opcode::Undefined);
                    }
                }

                Ok(())
            }

            Expr::Array { elements, .. } => {
                // Create empty array object
                self.emit(Instruction::with_u8(Opcode::Array, 0));

                // For each element, we need to:
                // 1. Dup the array object on the stack
                // 2. Push the index
                // 3. Push the element value
                // 4. Call PutArrayEl to store it
                for (i, elem_opt) in elements.iter().enumerate() {
                    // Duplicate array ref
                    self.emit_simple(Opcode::Dup);

                    // Push index
                    if i <= 7 {
                        let opcode = match i {
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
                    } else if i <= 127 {
                        self.emit(Instruction::with_i8(Opcode::PushI8, i as i8));
                    } else {
                        self.emit(Instruction::with_i16(Opcode::PushI16, i as i16));
                    }

                    // Push element value (or undefined for holes)
                    if let Some(elem) = elem_opt {
                        self.gen_expr(elem)?;
                    } else {
                        self.emit_simple(Opcode::Undefined);
                    }

                    // Store: [arr, index, value] -> [arr]
                    self.emit_simple(Opcode::PutArrayEl);
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

            Expr::Function { name: _name, params, body, .. } => {
                // Compile function expression - similar to FunctionDecl but push result to stack
                let (func_bytecode, local_count, captured_vars) = self.compile_function_body(params, body)?;
                let param_count = params.len() as u8;
                let has_captures = !captured_vars.is_empty();

                // Add to function table
                let func_index = self.function_bytecodes.len() as u16;
                self.function_bytecodes.push(FunctionBytecode {
                    bytecode: func_bytecode,
                    param_count,
                    local_count,
                    captured_vars: captured_vars.clone(),
                });

                if has_captures {
                    // Emit FClosure which creates a closure with captured variables
                    self.emit(Instruction::with_u8(Opcode::FClosure, func_index as u8));
                    self.writer.emit_u8(captured_vars.len() as u8);
                    for cv in &captured_vars {
                        self.writer.emit_u8(cv.parent_index);
                    }
                } else {
                    // No captures - emit regular PushFunc
                    if func_index <= 255 {
                        self.emit(Instruction::with_u8(Opcode::PushFunc8, func_index as u8));
                    } else {
                        self.emit(Instruction::with_u16(Opcode::PushFunc, func_index));
                    }
                }
                // Function value is now on the stack
                Ok(())
            }

            Expr::New { .. } | Expr::Arrow { .. } => {
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
                if libm::floor(*n) == *n && *n >= -1.0 && *n <= 7.0 {
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
                } else if libm::floor(*n) == *n && *n >= i8::MIN as f64 && *n <= i8::MAX as f64 {
                    self.emit(Instruction::with_i8(Opcode::PushI8, *n as i8));
                } else if libm::floor(*n) == *n && *n >= i16::MIN as f64 && *n <= i16::MAX as f64 {
                    self.emit(Instruction::with_i16(Opcode::PushI16, *n as i16));
                } else if libm::floor(*n) == *n && *n >= i32::MIN as f64 && *n <= i32::MAX as f64 {
                    self.emit(Instruction::with_i32(Opcode::PushI32, *n as i32));
                } else {
                    // Add to constant pool - store raw f64 bits as JSValue
                    // We encode f64 as its bit representation in usize
                    // The VM will need to convert this back to an f64
                    #[cfg(target_pointer_width = "64")]
                    {
                        let bits = n.to_bits();
                        let value = unsafe { core::mem::transmute::<usize, JSValue>(bits as usize) };
                        let index = self.constants.add(value)
                            .ok_or_else(|| CodeGenError::new("Too many constants".to_string()))?;
                        // Mark this constant as f64
                        self.const_is_f64.push(true);
                        if index <= 255 {
                            self.emit(Instruction::with_const8(Opcode::PushConst8, index as u8));
                        } else {
                            self.emit(Instruction::with_const16(Opcode::PushConst16, index));
                        }
                    }
                    #[cfg(not(target_pointer_width = "64"))]
                    {
                        let value = JSValue::from_int(*n as i32); // Fallback for 32-bit
                        let index = self.constants.add(value)
                            .ok_or_else(|| CodeGenError::new("Too many constants".to_string()))?;
                        self.const_is_f64.push(false);
                        if index <= 255 {
                            self.emit(Instruction::with_const8(Opcode::PushConst8, index as u8));
                        } else {
                            self.emit(Instruction::with_const16(Opcode::PushConst16, index));
                        }
                    }
                }
            }

            Literal::String(s) => {
                if s.is_empty() {
                    self.emit_simple(Opcode::PushEmptyString);
                } else {
                    // Add string to atom table and emit PushAtomString instruction
                    let atom_idx = self.get_or_create_atom(s);
                    if atom_idx <= 255 {
                        self.emit(Instruction::with_atom8(Opcode::PushAtomString8, atom_idx as u8));
                    } else {
                        self.emit(Instruction::with_atom16(Opcode::PushAtomString16, atom_idx));
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

    #[test]
    fn test_expression_statement_returns_value() {
        // Test that the last expression in a program returns its value
        let parser = Parser::new("2 + 2");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        // The bytecode should end with Return, not ReturnUndef
        // Bytecode should be: Push2, Push2, Add, Return
        assert!(!bytecode.is_empty());

        // Check that Return opcode is present (opcode value 163)
        assert!(bytecode.contains(&163), "Bytecode should contain Return opcode");
        // Should NOT contain ReturnUndef
        assert!(!bytecode.contains(&164), "Bytecode should NOT contain ReturnUndef for expression");
    }

    #[test]
    fn test_float_constant_pool() {
        // Test that floats go into the constant pool
        let parser = Parser::new("3.14");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        // Check first 2 bytes are constant count
        assert!(bytecode.len() >= 2);
        let const_count = u16::from_le_bytes([bytecode[0], bytecode[1]]);
        assert_eq!(const_count, 1, "Should have 1 constant");

        // The bytecode should contain PushConst8 or PushConst16
        // PushConst8 = 17
        assert!(bytecode.contains(&17), "Should contain PushConst8 opcode");
    }

    #[test]
    fn test_multiple_expressions_last_one_returned() {
        // Test that only the last expression is returned
        let parser = Parser::new("1 + 1; 2 + 2");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        assert!(!bytecode.is_empty());

        // Should contain Drop (for first expression) and Return (for last)
        assert!(bytecode.contains(&163), "Should contain Return opcode");
        assert!(bytecode.contains(&0), "Should contain Drop opcode");
    }

    #[test]
    fn test_var_decl_returns_undefined() {
        // Test that variable declarations still return undefined
        let parser = Parser::new("var x = 5;");
        let program = parser.parse().unwrap();

        let gen = CodeGenerator::new();
        let bytecode = gen.generate(&program).unwrap();

        assert!(!bytecode.is_empty());

        // Should end with ReturnUndef (opcode value 164)
        assert!(bytecode.contains(&164), "Should contain ReturnUndef opcode");
    }
}
