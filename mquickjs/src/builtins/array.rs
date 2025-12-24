//! Array built-in constructor and methods
//!
//! Implements Array(), Array.isArray(), and Array.prototype methods:
//! push, pop, shift, unshift, indexOf, includes, join, slice, concat,
//! reverse, forEach, map, filter, reduce

use crate::context::Context;
use crate::value::JSValue;
use crate::memory::HeapIndex;
use alloc::vec::Vec;
use alloc::string::String;

/// Array() constructor
///
/// Creates a new array with optional initial elements
pub fn array_constructor(ctx: &mut Context, elements: &[JSValue]) -> Result<JSValue, JSValue> {
    let arr_idx = ctx.alloc_value_array(elements.len())
        .map_err(|_| JSValue::exception())?;

    if let Some(arr) = ctx.get_value_array_mut(arr_idx) {
        for elem in elements {
            arr.push(*elem);
        }
    }

    Ok(JSValue::from_ptr(arr_idx))
}

/// Array.isArray() - Determines whether a value is an array
pub fn is_array(ctx: &Context, value: JSValue) -> bool {
    if let Some(idx) = value.to_ptr() {
        // Check if it's a value array
        ctx.get_value_array(idx).is_some()
    } else {
        false
    }
}

/// Array.prototype.push() - Adds elements to the end of an array
///
/// Returns the new length
pub fn array_push(ctx: &mut Context, arr: JSValue, elements: &[JSValue]) -> Result<i32, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    for elem in elements {
        if !arr_ref.push(*elem) {
            // Array is full
            return Err(JSValue::exception());
        }
    }

    Ok(arr_ref.header().count() as i32)
}

/// Array.prototype.pop() - Removes and returns the last element
pub fn array_pop(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    Ok(arr_ref.pop().unwrap_or(JSValue::undefined()))
}

/// Array.prototype.shift() - Removes and returns the first element
pub fn array_shift(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    Ok(arr_ref.shift().unwrap_or(JSValue::undefined()))
}

/// Array.prototype.unshift() - Adds elements to the beginning of an array
///
/// Returns the new length
pub fn array_unshift(ctx: &mut Context, arr: JSValue, elements: &[JSValue]) -> Result<i32, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    // Insert elements in reverse order to maintain their order
    for elem in elements.iter().rev() {
        if !arr_ref.unshift(*elem) {
            return Err(JSValue::exception());
        }
    }

    Ok(arr_ref.header().count() as i32)
}

/// Array.prototype.indexOf() - Returns the first index of an element
///
/// Returns -1 if not found
pub fn array_index_of(ctx: &Context, arr: JSValue, search_element: JSValue, from_index: Option<i32>) -> Result<i32, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array(idx).ok_or(JSValue::exception())?;

    let count = arr_ref.header().count() as i32;
    let start = from_index.unwrap_or(0).max(0);

    let slice = arr_ref.as_slice();
    for i in start..count {
        if i < slice.len() as i32 && slice[i as usize] == search_element {
            return Ok(i);
        }
    }

    Ok(-1)
}

/// Array.prototype.includes() - Determines whether an array contains a value
pub fn array_includes(ctx: &Context, arr: JSValue, search_element: JSValue, from_index: Option<i32>) -> Result<bool, JSValue> {
    let index = array_index_of(ctx, arr, search_element, from_index)?;
    Ok(index >= 0)
}

/// Array.prototype.join() - Joins all elements into a string
pub fn array_join(ctx: &Context, arr: JSValue, separator: Option<&str>) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array(idx).ok_or(JSValue::exception())?;

    let sep = separator.unwrap_or(",");
    let mut result = String::new();

    let slice = arr_ref.as_slice();
    for (i, elem) in slice.iter().enumerate() {
        if i > 0 {
            result.push_str(sep);
        }

        // Convert element to string
        if let Some(s) = ctx.get_string(*elem) {
            result.push_str(s);
        } else if let Some(n) = ctx.get_number(*elem) {
            result.push_str(&alloc::format!("{}", n));
        } else if elem.is_null() {
            // null becomes empty string
        } else if elem.is_undefined() {
            // undefined becomes empty string
        } else if let Some(b) = elem.to_bool() {
            result.push_str(if b { "true" } else { "false" });
        }
    }

    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

/// Array.prototype.slice() - Returns a shallow copy of a portion of an array
pub fn array_slice(ctx: &mut Context, arr: JSValue, start: Option<i32>, end: Option<i32>) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array(idx).ok_or(JSValue::exception())?;

    let len = arr_ref.header().count() as i32;
    let start_idx = start.unwrap_or(0).max(0).min(len);
    let end_idx = end.unwrap_or(len).max(0).min(len);

    if start_idx >= end_idx {
        return array_constructor(ctx, &[]);
    }

    let slice = arr_ref.as_slice();
    let elements: Vec<JSValue> = slice[start_idx as usize..end_idx as usize].to_vec();

    array_constructor(ctx, &elements)
}

/// Array.prototype.concat() - Merges two or more arrays
pub fn array_concat(ctx: &mut Context, arr: JSValue, others: &[JSValue]) -> Result<JSValue, JSValue> {
    let mut elements = Vec::new();

    // Add elements from original array
    if let Some(idx) = arr.to_ptr() {
        if let Some(arr_ref) = ctx.get_value_array(idx) {
            elements.extend_from_slice(arr_ref.as_slice());
        }
    }

    // Add elements from other arrays
    for other in others {
        if let Some(idx) = other.to_ptr() {
            if let Some(arr_ref) = ctx.get_value_array(idx) {
                elements.extend_from_slice(arr_ref.as_slice());
            } else {
                // Not an array, add as single element
                elements.push(*other);
            }
        } else {
            // Primitive value, add as single element
            elements.push(*other);
        }
    }

    array_constructor(ctx, &elements)
}

/// Array.prototype.reverse() - Reverses an array in place
pub fn array_reverse(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    let count = arr_ref.header().count() as usize;
    unsafe {
        let slice = arr_ref.as_full_mut_slice();
        let slice = &mut slice[..count];
        slice.reverse();
    }

    Ok(arr)
}

/// Array.prototype.forEach() - Executes a function for each element
///
/// Simplified: Just returns the array (proper implementation needs VM integration)
pub fn array_for_each(_ctx: &mut Context, arr: JSValue, _callback: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement callback execution via VM
    Ok(arr)
}

/// Array.prototype.map() - Creates a new array with results of calling a function
///
/// Simplified: Returns a copy (proper implementation needs VM integration)
pub fn array_map(ctx: &mut Context, arr: JSValue, _callback: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement callback execution via VM
    // For now, just return a copy
    array_slice(ctx, arr, None, None)
}

/// Array.prototype.filter() - Creates a new array with filtered elements
///
/// Simplified: Returns a copy (proper implementation needs VM integration)
pub fn array_filter(ctx: &mut Context, arr: JSValue, _callback: JSValue) -> Result<JSValue, JSValue> {
    // TODO: Implement callback execution via VM
    // For now, just return a copy
    array_slice(ctx, arr, None, None)
}

/// Array.prototype.reduce() - Reduces array to a single value
///
/// Simplified: Returns undefined (proper implementation needs VM integration)
pub fn array_reduce(_ctx: &mut Context, _arr: JSValue, _callback: JSValue, _initial: Option<JSValue>) -> Result<JSValue, JSValue> {
    // TODO: Implement callback execution via VM
    Ok(JSValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_constructor() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
        ]).unwrap();

        assert!(is_array(&ctx, arr));

        let idx = arr.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        assert_eq!(arr_ref.header().count(), 3);
    }

    #[test]
    fn test_array_push_pop() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[]).unwrap();

        // Push elements
        array_push(&mut ctx, arr, &[JSValue::from_int(1)]).unwrap();
        array_push(&mut ctx, arr, &[JSValue::from_int(2)]).unwrap();

        let idx = arr.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        assert_eq!(arr_ref.header().count(), 2);

        // Pop element
        let val = array_pop(&mut ctx, arr).unwrap();
        assert_eq!(val.to_int(), Some(2));
    }

    #[test]
    fn test_array_shift_unshift() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
        ]).unwrap();

        // Shift first element
        let val = array_shift(&mut ctx, arr).unwrap();
        assert_eq!(val.to_int(), Some(1));

        // Unshift new element
        array_unshift(&mut ctx, arr, &[JSValue::from_int(0)]).unwrap();

        let idx = arr.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        let slice = arr_ref.as_slice();
        assert_eq!(slice[0].to_int(), Some(0));
    }

    #[test]
    fn test_array_index_of() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(10),
            JSValue::from_int(20),
            JSValue::from_int(30),
        ]).unwrap();

        let idx = array_index_of(&ctx, arr, JSValue::from_int(20), None).unwrap();
        assert_eq!(idx, 1);

        let idx = array_index_of(&ctx, arr, JSValue::from_int(99), None).unwrap();
        assert_eq!(idx, -1);
    }

    #[test]
    fn test_array_includes() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
        ]).unwrap();

        assert!(array_includes(&ctx, arr, JSValue::from_int(1), None).unwrap());
        assert!(!array_includes(&ctx, arr, JSValue::from_int(3), None).unwrap());
    }

    #[test]
    fn test_array_join() {
        let mut ctx = Context::new(4096);

        let s1 = ctx.new_string("a").unwrap();
        let s2 = ctx.new_string("b").unwrap();
        let arr = array_constructor(&mut ctx, &[s1, s2]).unwrap();

        let result = array_join(&ctx, arr, Some(",")).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "a,b");
    }

    #[test]
    fn test_array_slice() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
            JSValue::from_int(4),
        ]).unwrap();

        let sliced = array_slice(&mut ctx, arr, Some(1), Some(3)).unwrap();
        let idx = sliced.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        assert_eq!(arr_ref.header().count(), 2);
    }

    #[test]
    fn test_array_concat() {
        let mut ctx = Context::new(4096);

        let arr1 = array_constructor(&mut ctx, &[JSValue::from_int(1)]).unwrap();
        let arr2 = array_constructor(&mut ctx, &[JSValue::from_int(2)]).unwrap();

        let result = array_concat(&mut ctx, arr1, &[arr2]).unwrap();
        let idx = result.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        assert_eq!(arr_ref.header().count(), 2);
    }

    #[test]
    fn test_array_reverse() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
        ]).unwrap();

        array_reverse(&mut ctx, arr).unwrap();

        let idx = arr.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        let slice = arr_ref.as_slice();
        assert_eq!(slice[0].to_int(), Some(3));
        assert_eq!(slice[2].to_int(), Some(1));
    }
}
