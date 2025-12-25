//! Runtime initialization
//!
//! This module sets up the JavaScript runtime environment by creating the
//! global object and installing all built-in constructors, prototypes, and
//! global functions.

use crate::context::Context;
use crate::value::{JSValue, JSAtom};
use crate::object::PropertyFlags;
use crate::builtins;
use alloc::string::ToString;

/// Initialize the JavaScript runtime environment
///
/// This function sets up the global object with all built-in constructors,
/// prototypes, and global functions.
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
///
/// # Returns
///
/// The global object with all built-ins installed
///
/// # Note
///
/// This is a simplified initialization that installs placeholders for
/// built-in objects. Full ECMAScript compliance would require much more
/// extensive initialization.
pub fn init_runtime(ctx: &mut Context) -> Result<JSValue, JSValue> {
    // Get the global object (should already be created in Context::new())
    let global = ctx.global_object();
    if global.is_null() {
        return Err(make_error(ctx, "Global object not initialized"));
    }

    // Install global constants
    install_global_constants(ctx, global)?;

    // Install built-in constructors and prototypes
    install_object_constructor(ctx, global)?;
    install_array_constructor(ctx, global)?;
    install_string_constructor(ctx, global)?;
    install_number_constructor(ctx, global)?;
    install_boolean_constructor(ctx, global)?;
    install_function_constructor(ctx, global)?;

    // Install Math object
    install_math_object(ctx, global)?;

    // Install Error constructors
    install_error_constructors(ctx, global)?;

    // Install console object
    install_console_object(ctx, global)?;

    // Install global functions
    install_global_functions(ctx, global)?;

    Ok(global)
}

/// Install global constants (undefined, NaN, Infinity)
fn install_global_constants(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // undefined
    set_property(ctx, global, "undefined", JSValue::undefined())?;

    // NaN
    let nan = ctx.new_number(f64::NAN)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "NaN", nan)?;

    // Infinity
    let infinity = ctx.new_number(f64::INFINITY)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "Infinity", infinity)?;

    Ok(())
}

/// Install Object constructor and Object.prototype
fn install_object_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Object.prototype
    let object_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create Object constructor (placeholder - would be a native function)
    let object_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Object.prototype
    set_property(ctx, object_ctor, "prototype", object_proto)?;

    // Set Object on global
    set_property(ctx, global, "Object", object_ctor)?;

    // TODO: Install Object static methods (keys, values, entries, assign, create, etc.)

    Ok(())
}

/// Install Array constructor and Array.prototype
fn install_array_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Array.prototype
    let array_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create Array constructor (placeholder)
    let array_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Array.prototype
    set_property(ctx, array_ctor, "prototype", array_proto)?;

    // Set Array on global
    set_property(ctx, global, "Array", array_ctor)?;

    // TODO: Install Array.isArray
    // TODO: Install Array.prototype methods (push, pop, shift, unshift, slice, etc.)

    Ok(())
}

/// Install String constructor and String.prototype
fn install_string_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create String.prototype
    let string_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create String constructor (placeholder)
    let string_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set String.prototype
    set_property(ctx, string_ctor, "prototype", string_proto)?;

    // Set String on global
    set_property(ctx, global, "String", string_ctor)?;

    // TODO: Install String.prototype methods (length, charAt, slice, indexOf, etc.)

    Ok(())
}

/// Install Number constructor and Number.prototype
fn install_number_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Number.prototype
    let number_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create Number constructor (placeholder)
    let number_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Number.prototype
    set_property(ctx, number_ctor, "prototype", number_proto)?;

    // Set Number on global
    set_property(ctx, global, "Number", number_ctor)?;

    // TODO: Install Number constants (MAX_VALUE, MIN_VALUE, etc.)
    // TODO: Install Number.isNaN, Number.isFinite

    Ok(())
}

/// Install Boolean constructor and Boolean.prototype
fn install_boolean_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Boolean.prototype
    let boolean_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create Boolean constructor (placeholder)
    let boolean_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Boolean.prototype
    set_property(ctx, boolean_ctor, "prototype", boolean_proto)?;

    // Set Boolean on global
    set_property(ctx, global, "Boolean", boolean_ctor)?;

    Ok(())
}

/// Install Function constructor and Function.prototype
fn install_function_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Function.prototype
    let function_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Create Function constructor (placeholder)
    let function_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Function.prototype
    set_property(ctx, function_ctor, "prototype", function_proto)?;

    // Set Function on global
    set_property(ctx, global, "Function", function_ctor)?;

    // TODO: Install Function.prototype methods (call, apply, bind)

    Ok(())
}

/// Install Math object
fn install_math_object(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Math object
    let math = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Install Math constants
    let pi = ctx.new_number(core::f64::consts::PI)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "PI", pi)?;

    let e = ctx.new_number(core::f64::consts::E)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "E", e)?;

    let ln2 = ctx.new_number(core::f64::consts::LN_2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "LN2", ln2)?;

    let ln10 = ctx.new_number(core::f64::consts::LN_10)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "LN10", ln10)?;

    let sqrt2 = ctx.new_number(core::f64::consts::SQRT_2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "SQRT2", sqrt2)?;

    // TODO: Install Math methods (abs, floor, ceil, round, min, max, pow, sqrt, etc.)
    // These would be native function objects

    // Set Math on global
    set_property(ctx, global, "Math", math)?;

    Ok(())
}

/// Install Error constructors
fn install_error_constructors(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create Error constructor (placeholder)
    let error_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "Error", error_ctor)?;

    // Create TypeError constructor (placeholder)
    let type_error_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "TypeError", type_error_ctor)?;

    // Create ReferenceError constructor (placeholder)
    let ref_error_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "ReferenceError", ref_error_ctor)?;

    // Create RangeError constructor (placeholder)
    let range_error_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "RangeError", range_error_ctor)?;

    // Create SyntaxError constructor (placeholder)
    let syntax_error_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, global, "SyntaxError", syntax_error_ctor)?;

    Ok(())
}

/// Install console object
fn install_console_object(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // Create console object
    let console = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // TODO: Install console methods (log, error, warn, info)
    // These would be native function objects

    // Set console on global
    set_property(ctx, global, "console", console)?;

    Ok(())
}

/// Install global functions (parseInt, parseFloat, isNaN, isFinite)
fn install_global_functions(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    // TODO: Create native function objects for:
    // - parseInt
    // - parseFloat
    // - isNaN
    // - isFinite
    // - eval
    // - encodeURI / decodeURI
    // - encodeURIComponent / decodeURIComponent

    // For now, we just return Ok - these will be implemented later
    // when we have proper native function support

    Ok(())
}

// ========== Helper Functions ==========

/// Set a property on an object (convenience wrapper)
fn set_property(ctx: &mut Context, obj: JSValue, key: &str, value: JSValue) -> Result<(), JSValue> {
    // Create an atom for the key
    // For now we use a simple hash of the string
    let atom = string_to_atom(key);

    ctx.add_property(obj, atom, value, PropertyFlags::default())
        .map_err(|_| make_error(ctx, "Out of memory setting property"))
}

/// Convert a string to an atom (simplified - just hash the string)
fn string_to_atom(s: &str) -> JSAtom {
    // Simple hash function
    let mut hash: u32 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
    }
    JSAtom::from_id(hash)
}

/// Create an error value
fn make_error(ctx: &mut Context, message: &str) -> JSValue {
    ctx.new_string(message).unwrap_or(JSValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_runtime() {
        let mut ctx = Context::new(16384);
        let result = init_runtime(&mut ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_global_constants() {
        let mut ctx = Context::new(16384);
        let global = ctx.new_object().unwrap();
        let result = install_global_constants(&mut ctx, global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_object() {
        let mut ctx = Context::new(16384);
        let global = ctx.new_object().unwrap();
        let result = install_object_constructor(&mut ctx, global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_install_math() {
        let mut ctx = Context::new(16384);
        let global = ctx.new_object().unwrap();
        let result = install_math_object(&mut ctx, global);
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_to_atom() {
        let atom1 = string_to_atom("test");
        let atom2 = string_to_atom("test");
        let atom3 = string_to_atom("other");

        assert_eq!(atom1.id(), atom2.id());
        assert_ne!(atom1.id(), atom3.id());
    }
}
