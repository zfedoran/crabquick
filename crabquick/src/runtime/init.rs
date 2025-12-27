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
    use crate::builtins::native_functions;

    // Create Object.prototype (with null prototype - this is the root)
    let object_proto = ctx.new_object_with_proto(JSValue::null())
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Store Object.prototype in context so all future objects inherit from it
    ctx.set_object_prototype(object_proto);

    // Install Object.prototype methods
    let has_own_prop_fn = ctx.new_native_function(native_functions::object_has_own_property_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_proto, "hasOwnProperty", has_own_prop_fn)?;

    let to_string_fn = ctx.new_native_function(native_functions::object_to_string_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_proto, "toString", to_string_fn)?;

    // Create Object constructor (now inherits from Object.prototype)
    let object_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Object.prototype
    set_property(ctx, object_ctor, "prototype", object_proto)?;

    // Install Object static methods
    let keys_fn = ctx.new_native_function(native_functions::object_keys_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "keys", keys_fn)?;

    let values_fn = ctx.new_native_function(native_functions::object_values_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "values", values_fn)?;

    let entries_fn = ctx.new_native_function(native_functions::object_entries_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "entries", entries_fn)?;

    let assign_fn = ctx.new_native_function(native_functions::object_assign_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "assign", assign_fn)?;

    let create_fn = ctx.new_native_function(native_functions::object_create_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "create", create_fn)?;

    let get_proto_fn = ctx.new_native_function(native_functions::object_get_prototype_of_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "getPrototypeOf", get_proto_fn)?;

    let set_proto_fn = ctx.new_native_function(native_functions::object_set_prototype_of_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "setPrototypeOf", set_proto_fn)?;

    let define_prop_fn = ctx.new_native_function(native_functions::object_define_property_native, 3)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, object_ctor, "defineProperty", define_prop_fn)?;

    // Set Object on global
    set_property(ctx, global, "Object", object_ctor)?;

    Ok(())
}

/// Install Array constructor and Array.prototype
fn install_array_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    use crate::builtins::native_functions;

    // Create Array.prototype
    let array_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Install Array.prototype methods
    let push_fn = ctx.new_native_function(native_functions::array_push_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "push", push_fn)?;

    let pop_fn = ctx.new_native_function(native_functions::array_pop_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "pop", pop_fn)?;

    let shift_fn = ctx.new_native_function(native_functions::array_shift_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "shift", shift_fn)?;

    let unshift_fn = ctx.new_native_function(native_functions::array_unshift_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "unshift", unshift_fn)?;

    let slice_fn = ctx.new_native_function(native_functions::array_slice_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "slice", slice_fn)?;

    let splice_fn = ctx.new_native_function(native_functions::array_splice_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "splice", splice_fn)?;

    let concat_fn = ctx.new_native_function(native_functions::array_concat_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "concat", concat_fn)?;

    let index_of_fn = ctx.new_native_function(native_functions::array_index_of_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "indexOf", index_of_fn)?;

    let includes_fn = ctx.new_native_function(native_functions::array_includes_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "includes", includes_fn)?;

    let join_fn = ctx.new_native_function(native_functions::array_join_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "join", join_fn)?;

    let reverse_fn = ctx.new_native_function(native_functions::array_reverse_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "reverse", reverse_fn)?;

    // Callback methods
    let for_each_fn = ctx.new_native_function(native_functions::array_for_each_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "forEach", for_each_fn)?;

    let map_fn = ctx.new_native_function(native_functions::array_map_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "map", map_fn)?;

    let filter_fn = ctx.new_native_function(native_functions::array_filter_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "filter", filter_fn)?;

    let reduce_fn = ctx.new_native_function(native_functions::array_reduce_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "reduce", reduce_fn)?;

    let find_fn = ctx.new_native_function(native_functions::array_find_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "find", find_fn)?;

    let find_index_fn = ctx.new_native_function(native_functions::array_find_index_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "findIndex", find_index_fn)?;

    let some_fn = ctx.new_native_function(native_functions::array_some_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "some", some_fn)?;

    let every_fn = ctx.new_native_function(native_functions::array_every_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "every", every_fn)?;

    let last_index_of_fn = ctx.new_native_function(native_functions::array_last_index_of_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "lastIndexOf", last_index_of_fn)?;

    let reduce_right_fn = ctx.new_native_function(native_functions::array_reduce_right_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "reduceRight", reduce_right_fn)?;

    let sort_fn = ctx.new_native_function(native_functions::array_sort_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "sort", sort_fn)?;

    let to_string_fn = ctx.new_native_function(native_functions::array_to_string_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_proto, "toString", to_string_fn)?;

    // Create Array constructor (placeholder)
    let array_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Array.prototype
    set_property(ctx, array_ctor, "prototype", array_proto)?;

    // Install Array.isArray as static method on constructor
    let is_array_fn = ctx.new_native_function(native_functions::array_is_array_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, array_ctor, "isArray", is_array_fn)?;

    // Set Array on global
    set_property(ctx, global, "Array", array_ctor)?;

    Ok(())
}

/// Install String constructor and String.prototype
fn install_string_constructor(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    use crate::builtins::native_functions;

    // Create String.prototype
    let string_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Install String.prototype methods
    let char_at_fn = ctx.new_native_function(native_functions::string_char_at_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "charAt", char_at_fn)?;

    let char_code_at_fn = ctx.new_native_function(native_functions::string_char_code_at_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "charCodeAt", char_code_at_fn)?;

    let slice_fn = ctx.new_native_function(native_functions::string_slice_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "slice", slice_fn)?;

    let substring_fn = ctx.new_native_function(native_functions::string_substring_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "substring", substring_fn)?;

    let index_of_fn = ctx.new_native_function(native_functions::string_index_of_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "indexOf", index_of_fn)?;

    let last_index_of_fn = ctx.new_native_function(native_functions::string_last_index_of_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "lastIndexOf", last_index_of_fn)?;

    let to_lower_case_fn = ctx.new_native_function(native_functions::string_to_lower_case_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "toLowerCase", to_lower_case_fn)?;

    let to_upper_case_fn = ctx.new_native_function(native_functions::string_to_upper_case_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "toUpperCase", to_upper_case_fn)?;

    let split_fn = ctx.new_native_function(native_functions::string_split_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "split", split_fn)?;

    let trim_fn = ctx.new_native_function(native_functions::string_trim_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "trim", trim_fn)?;

    let trim_start_fn = ctx.new_native_function(native_functions::string_trim_start_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "trimStart", trim_start_fn)?;

    let trim_end_fn = ctx.new_native_function(native_functions::string_trim_end_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "trimEnd", trim_end_fn)?;

    let replace_fn = ctx.new_native_function(native_functions::string_replace_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "replace", replace_fn)?;

    let replace_all_fn = ctx.new_native_function(native_functions::string_replace_all_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "replaceAll", replace_all_fn)?;

    let includes_fn = ctx.new_native_function(native_functions::string_includes_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "includes", includes_fn)?;

    let starts_with_fn = ctx.new_native_function(native_functions::string_starts_with_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "startsWith", starts_with_fn)?;

    let ends_with_fn = ctx.new_native_function(native_functions::string_ends_with_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "endsWith", ends_with_fn)?;

    let concat_fn = ctx.new_native_function(native_functions::string_concat_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "concat", concat_fn)?;

    let code_point_at_fn = ctx.new_native_function(native_functions::string_code_point_at_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_proto, "codePointAt", code_point_at_fn)?;

    // Create String constructor
    let string_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Add static methods to String constructor
    let from_char_code_fn = ctx.new_native_function(native_functions::string_from_char_code_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_ctor, "fromCharCode", from_char_code_fn)?;

    let from_code_point_fn = ctx.new_native_function(native_functions::string_from_code_point_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, string_ctor, "fromCodePoint", from_code_point_fn)?;

    // Set String.prototype
    set_property(ctx, string_ctor, "prototype", string_proto)?;

    // Set String on global
    set_property(ctx, global, "String", string_ctor)?;

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
    use crate::builtins::native_functions;

    // Create Function.prototype
    let function_proto = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Store Function.prototype in context so all functions inherit from it
    ctx.set_function_prototype(function_proto);

    // Install Function.prototype methods
    let call_fn = ctx.new_native_function(native_functions::function_call_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, function_proto, "call", call_fn)?;

    let apply_fn = ctx.new_native_function(native_functions::function_apply_native, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, function_proto, "apply", apply_fn)?;

    let bind_fn = ctx.new_native_function(native_functions::function_bind_native, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, function_proto, "bind", bind_fn)?;

    // Create Function constructor (placeholder)
    let function_ctor = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Set Function.prototype
    set_property(ctx, function_ctor, "prototype", function_proto)?;

    // Set Function on global
    set_property(ctx, global, "Function", function_ctor)?;

    Ok(())
}

/// Install Math object
fn install_math_object(ctx: &mut Context, global: JSValue) -> Result<(), JSValue> {
    use crate::builtins::native_functions;

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

    // Install Math methods as native functions
    let abs_fn = ctx.new_native_function(native_functions::math_abs, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "abs", abs_fn)?;

    let floor_fn = ctx.new_native_function(native_functions::math_floor, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "floor", floor_fn)?;

    let ceil_fn = ctx.new_native_function(native_functions::math_ceil, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "ceil", ceil_fn)?;

    let round_fn = ctx.new_native_function(native_functions::math_round, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "round", round_fn)?;

    let min_fn = ctx.new_native_function(native_functions::math_min, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "min", min_fn)?;

    let max_fn = ctx.new_native_function(native_functions::math_max, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "max", max_fn)?;

    let pow_fn = ctx.new_native_function(native_functions::math_pow, 2)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "pow", pow_fn)?;

    let sqrt_fn = ctx.new_native_function(native_functions::math_sqrt, 1)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, math, "sqrt", sqrt_fn)?;

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
    use crate::builtins::native_functions;

    // Create console object
    let console = ctx.new_object()
        .map_err(|_| make_error(ctx, "Out of memory"))?;

    // Install console methods as native functions
    let log_fn = ctx.new_native_function(native_functions::console_log_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, console, "log", log_fn)?;

    let error_fn = ctx.new_native_function(native_functions::console_error_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, console, "error", error_fn)?;

    let warn_fn = ctx.new_native_function(native_functions::console_warn_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, console, "warn", warn_fn)?;

    let info_fn = ctx.new_native_function(native_functions::console_info_native, 0)
        .map_err(|_| make_error(ctx, "Out of memory"))?;
    set_property(ctx, console, "info", info_fn)?;

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
pub fn string_to_atom(s: &str) -> JSAtom {
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
        let mut ctx = Context::new(32768); // 32KB for property tables
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
