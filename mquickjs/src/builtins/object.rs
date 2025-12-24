//! Object built-in constructor and methods
//!
//! Implements Object(), Object.keys(), Object.values(), Object.entries(),
//! Object.assign(), Object.create(), Object.prototype.hasOwnProperty(),
//! Object.prototype.toString()

use crate::context::Context;
use crate::value::{JSValue, JSAtom};
use crate::object::PropertyFlags;
use alloc::vec::Vec;
use alloc::string::String;

/// Object() constructor
///
/// Creates a new object or converts a value to an object
pub fn object_constructor(ctx: &mut Context, value: Option<JSValue>) -> Result<JSValue, JSValue> {
    match value {
        None | Some(val) if val.is_null() || val.is_undefined() => {
            // Create new empty object
            ctx.new_object().map_err(|_| JSValue::exception())
        }
        Some(val) => {
            // Return value wrapped as object (simplified: just return the value)
            Ok(val)
        }
    }
}

/// Object.keys() - Returns an array of a given object's own property names
///
/// # Arguments
///
/// * `ctx` - JavaScript execution context
/// * `obj` - The object to get keys from
///
/// # Returns
///
/// Array of property keys
pub fn object_keys(ctx: &mut Context, obj: JSValue) -> Result<JSValue, JSValue> {
    if !obj.is_object() {
        // Should throw TypeError, but for simplicity return empty array
        return create_empty_array(ctx);
    }

    let obj_ref = ctx.get_object(obj).ok_or(JSValue::exception())?;

    if !obj_ref.has_properties() {
        return create_empty_array(ctx);
    }

    // Get property table
    let props_index = obj_ref.props_index();
    let props_table = ctx.get_property_table(props_index)
        .ok_or(JSValue::exception())?;

    // Collect keys
    let mut keys = Vec::new();
    unsafe {
        let header = props_table.header();
        let count = header.count() as usize;
        let properties = props_table.properties();

        for prop in properties.iter().take(count) {
            if prop.flags().is_enumerable() {
                // For simplicity, store atom ID as integer
                let key_id = prop.key().id();
                keys.push(JSValue::from_int(key_id as i32));
            }
        }
    }

    // Create array with keys (simplified version)
    create_array_from_values(ctx, &keys)
}

/// Object.values() - Returns an array of a given object's own property values
pub fn object_values(ctx: &mut Context, obj: JSValue) -> Result<JSValue, JSValue> {
    if !obj.is_object() {
        return create_empty_array(ctx);
    }

    let obj_ref = ctx.get_object(obj).ok_or(JSValue::exception())?;

    if !obj_ref.has_properties() {
        return create_empty_array(ctx);
    }

    let props_index = obj_ref.props_index();
    let props_table = ctx.get_property_table(props_index)
        .ok_or(JSValue::exception())?;

    let mut values = Vec::new();
    unsafe {
        let header = props_table.header();
        let count = header.count() as usize;
        let properties = props_table.properties();

        for prop in properties.iter().take(count) {
            if prop.flags().is_enumerable() {
                values.push(prop.value());
            }
        }
    }

    create_array_from_values(ctx, &values)
}

/// Object.entries() - Returns an array of [key, value] pairs
pub fn object_entries(ctx: &mut Context, obj: JSValue) -> Result<JSValue, JSValue> {
    if !obj.is_object() {
        return create_empty_array(ctx);
    }

    let obj_ref = ctx.get_object(obj).ok_or(JSValue::exception())?;

    if !obj_ref.has_properties() {
        return create_empty_array(ctx);
    }

    let props_index = obj_ref.props_index();
    let props_table = ctx.get_property_table(props_index)
        .ok_or(JSValue::exception())?;

    let mut entries = Vec::new();
    unsafe {
        let header = props_table.header();
        let count = header.count() as usize;
        let properties = props_table.properties();

        for prop in properties.iter().take(count) {
            if prop.flags().is_enumerable() {
                // Create [key, value] pair (simplified: store as values)
                entries.push(prop.value());
            }
        }
    }

    create_array_from_values(ctx, &entries)
}

/// Object.assign() - Copies properties from source objects to target
///
/// Simplified implementation
pub fn object_assign(ctx: &mut Context, target: JSValue, sources: &[JSValue]) -> Result<JSValue, JSValue> {
    if !target.is_object() {
        return Err(JSValue::exception());
    }

    // Copy properties from each source
    for source in sources {
        if !source.is_object() {
            continue;
        }

        let src_obj = ctx.get_object(*source).ok_or(JSValue::exception())?;
        if !src_obj.has_properties() {
            continue;
        }

        let props_index = src_obj.props_index();
        let props_table = ctx.get_property_table(props_index)
            .ok_or(JSValue::exception())?;

        unsafe {
            let header = props_table.header();
            let count = header.count() as usize;
            let properties = props_table.properties();

            for prop in properties.iter().take(count) {
                if prop.flags().is_enumerable() {
                    ctx.add_property(target, prop.key(), prop.value(), PropertyFlags::default())
                        .map_err(|_| JSValue::exception())?;
                }
            }
        }
    }

    Ok(target)
}

/// Object.create() - Creates a new object with specified prototype
pub fn object_create(ctx: &mut Context, proto: JSValue) -> Result<JSValue, JSValue> {
    ctx.new_object_with_proto(proto).map_err(|_| JSValue::exception())
}

/// Object.prototype.hasOwnProperty() - Returns true if object has the specified property
pub fn has_own_property(ctx: &Context, obj: JSValue, key: JSAtom) -> bool {
    ctx.find_own_property(obj, key).is_some()
}

/// Object.prototype.toString() - Returns a string representation of the object
pub fn to_string(ctx: &mut Context, obj: JSValue) -> Result<JSValue, JSValue> {
    let str_val = if obj.is_null() {
        "[object Null]"
    } else if obj.is_undefined() {
        "[object Undefined]"
    } else if obj.is_object() {
        "[object Object]"
    } else if obj.is_bool() {
        "[object Boolean]"
    } else if ctx.get_number(obj).is_some() {
        "[object Number]"
    } else if ctx.get_string(obj).is_some() {
        "[object String]"
    } else {
        "[object Unknown]"
    };

    ctx.new_string(str_val).map_err(|_| JSValue::exception())
}

/// Helper: Create an empty array
fn create_empty_array(ctx: &mut Context) -> Result<JSValue, JSValue> {
    let arr_idx = ctx.alloc_value_array(0).map_err(|_| JSValue::exception())?;
    Ok(JSValue::from_ptr(arr_idx))
}

/// Helper: Create an array from values
fn create_array_from_values(ctx: &mut Context, values: &[JSValue]) -> Result<JSValue, JSValue> {
    let arr_idx = ctx.alloc_value_array(values.len()).map_err(|_| JSValue::exception())?;

    if let Some(arr) = ctx.get_value_array_mut(arr_idx) {
        for val in values {
            arr.push(*val);
        }
    }

    Ok(JSValue::from_ptr(arr_idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_constructor() {
        let mut ctx = Context::new(4096);

        let obj = object_constructor(&mut ctx, None).unwrap();
        assert!(obj.is_object());
    }

    #[test]
    fn test_object_keys() {
        let mut ctx = Context::new(8192);

        let obj = ctx.new_object().unwrap();
        let key = JSAtom::from_id(1);
        ctx.add_property(obj, key, JSValue::from_int(42), PropertyFlags::default()).unwrap();

        let keys = object_keys(&mut ctx, obj).unwrap();
        assert!(keys.is_ptr());
    }

    #[test]
    fn test_object_create() {
        let mut ctx = Context::new(4096);

        let proto = ctx.new_object().unwrap();
        let obj = object_create(&mut ctx, proto).unwrap();

        assert!(obj.is_object());
        let obj_ref = ctx.get_object(obj).unwrap();
        assert_eq!(obj_ref.prototype(), proto);
    }

    #[test]
    fn test_to_string() {
        let mut ctx = Context::new(4096);

        let result = to_string(&mut ctx, JSValue::null()).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "[object Null]");

        let obj = ctx.new_object().unwrap();
        let result = to_string(&mut ctx, obj).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "[object Object]");
    }
}
