//! VM integration tests
//!
//! These tests demonstrate the VM executing various bytecode programs.

extern crate alloc;
use alloc::vec::Vec;

use crabquick::{
    Context,
    bytecode::{BytecodeWriter, Instruction, Opcode},
};

/// Helper function to execute bytecode
fn execute_bytecode(ctx: &mut Context, instructions: &[Instruction]) -> Result<crabquick::JSValue, crabquick::JSValue> {
    // Build bytecode
    let mut writer = BytecodeWriter::new();
    for inst in instructions {
        writer.emit(inst);
    }
    let code = writer.finish();

    // Add constant pool and atom table headers
    // Format: [const_count: u16][constants...][atom_count: u16][atoms...][bytecode...]
    let mut bytecode = Vec::new();
    bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 constants
    bytecode.extend_from_slice(&0u16.to_le_bytes()); // 0 atoms
    bytecode.extend_from_slice(&code);

    // Allocate bytecode array
    let bc_index = ctx.alloc_byte_array(bytecode.len()).unwrap();
    unsafe {
        let bc_array = ctx.get_byte_array_mut(bc_index).unwrap();
        let slice = bc_array.as_full_mut_slice();
        slice[..bytecode.len()].copy_from_slice(&bytecode);
        bc_array.header_mut().set_count(bytecode.len());
    }

    // Execute
    ctx.execute_bytecode(bc_index)
}

#[test]
fn test_simple_addition() {
    let mut ctx = Context::new(4096);

    // Code: 2 + 3
    let instructions = vec![
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::Add),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();

    // Result should be 5.0 (boxed number)
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 5.0);
}

#[test]
fn test_arithmetic_expression() {
    let mut ctx = Context::new(4096);

    // Code: (10 + 5) * 2
    let instructions = vec![
        Instruction::new(Opcode::Push0),
        Instruction::with_i8(Opcode::PushI8, 10),
        Instruction::with_i8(Opcode::PushI8, 5),
        Instruction::new(Opcode::Add),
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Mul),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();

    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 30.0);
}

#[test]
fn test_comparison() {
    let mut ctx = Context::new(4096);

    // Code: 5 > 3
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::Gt),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_conditional_jump() {
    let mut ctx = Context::new(4096);

    // Code: if (false) { return 1; } else { return 2; }
    let instructions = vec![
        Instruction::new(Opcode::PushFalse),
        Instruction::with_label(Opcode::IfFalse, 2),  // Jump to push 2
        Instruction::new(Opcode::Push1),
        Instruction::new(Opcode::Return),
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_int(), Some(2));
}

#[test]
fn test_stack_operations() {
    let mut ctx = Context::new(4096);

    // Code: test dup and swap
    // push 5, dup, swap -> stack should be [5, 5]
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Dup),
        Instruction::new(Opcode::Add),  // 5 + 5 = 10
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 10.0);
}

#[test]
fn test_bitwise_operations() {
    let mut ctx = Context::new(4096);

    // Code: 5 & 3
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::And),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_int(), Some(5 & 3));
}

#[test]
fn test_logical_not() {
    let mut ctx = Context::new(4096);

    // Code: !false
    let instructions = vec![
        Instruction::new(Opcode::PushFalse),
        Instruction::new(Opcode::LNot),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_typeof() {
    let mut ctx = Context::new(4096);

    // Code: typeof 42
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 42),
        Instruction::new(Opcode::TypeOf),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let type_str = ctx.get_string(result).unwrap();
    assert_eq!(type_str, "number");
}

#[test]
fn test_undefined_null() {
    let mut ctx = Context::new(4096);

    // Code: return undefined
    let instructions = vec![
        Instruction::new(Opcode::Undefined),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert!(result.is_undefined());

    // Code: return null
    let instructions = vec![
        Instruction::new(Opcode::Null),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert!(result.is_null());
}

#[test]
fn test_increment_decrement() {
    let mut ctx = Context::new(4096);

    // Code: ++5 (should be 6)
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Inc),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 6.0);

    // Code: --5 (should be 4)
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Dec),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 4.0);
}

#[test]
fn test_object_creation() {
    let mut ctx = Context::new(4096);

    // Code: return {}
    let instructions = vec![
        Instruction::with_u8(Opcode::Object, 0),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert!(result.is_object());
}

#[test]
fn test_exception_throw_catch() {
    let mut ctx = Context::new(4096);

    // Code: throw 42
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 42),
        Instruction::new(Opcode::Throw),
    ];

    let result = execute_bytecode(&mut ctx, &instructions);
    assert!(result.is_err());

    let exception = result.unwrap_err();
    assert_eq!(exception.to_int(), Some(42));
}

#[test]
fn test_string_operations() {
    let mut ctx = Context::new(4096);

    // Code: return ""
    let instructions = vec![
        Instruction::new(Opcode::PushEmptyString),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let s = ctx.get_string(result).unwrap();
    assert_eq!(s, "");
}

#[test]
fn test_nan_infinity() {
    let mut ctx = Context::new(4096);

    // Code: return NaN
    let instructions = vec![
        Instruction::new(Opcode::PushNaN),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert!(num.is_nan());

    // Code: return Infinity
    let instructions = vec![
        Instruction::new(Opcode::PushInfinity),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, f64::INFINITY);

    // Code: return -Infinity
    let instructions = vec![
        Instruction::new(Opcode::PushNegInfinity),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, f64::NEG_INFINITY);
}

#[test]
fn test_modulo() {
    let mut ctx = Context::new(4096);

    // Code: 10 % 3
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 10),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::Mod),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 1.0);
}

#[test]
fn test_power() {
    let mut ctx = Context::new(4096);

    // Code: 2 ** 3 (should be 8)
    let instructions = vec![
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::Pow),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    let num = ctx.get_number(result).unwrap();
    assert_eq!(num, 8.0);
}

#[test]
fn test_shift_operations() {
    let mut ctx = Context::new(4096);

    // Code: 8 << 2 (should be 32)
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 8),
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Shl),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_int(), Some(32));

    // Code: 32 >> 2 (should be 8)
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 32),
        Instruction::new(Opcode::Push2),
        Instruction::new(Opcode::Sar),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_int(), Some(8));
}

#[test]
fn test_strict_equality() {
    let mut ctx = Context::new(4096);

    // Code: 5 === 5
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::StrictEq),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Code: 5 !== 3
    let instructions = vec![
        Instruction::new(Opcode::Push5),
        Instruction::new(Opcode::Push3),
        Instruction::new(Opcode::StrictNeq),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_void_operator() {
    let mut ctx = Context::new(4096);

    // Code: void 42 (should return undefined)
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 42),
        Instruction::new(Opcode::Void),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert!(result.is_undefined());
}

#[test]
fn test_nop() {
    let mut ctx = Context::new(4096);

    // Code: 42; nop; return
    let instructions = vec![
        Instruction::with_i8(Opcode::PushI8, 42),
        Instruction::new(Opcode::Nop),
        Instruction::new(Opcode::Return),
    ];

    let result = execute_bytecode(&mut ctx, &instructions).unwrap();
    assert_eq!(result.to_int(), Some(42));
}
