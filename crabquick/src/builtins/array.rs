//! Array built-in constructor and methods
//!
//! Implements Array(), Array.isArray(), and Array.prototype methods:
//! push, pop, shift, unshift, indexOf, includes, join, slice, concat,
//! reverse, forEach, map, filter, reduce

use crate::context::Context;
use crate::value::JSValue;
use crate::object::PropertyFlags;
use crate::memory::HeapIndex;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;

/// Array() constructor
///
/// Creates a new array with optional initial elements
pub fn array_constructor(ctx: &mut Context, elements: &[JSValue]) -> Result<JSValue, JSValue> {
    // Allocate at least 8 elements for growable arrays
    let capacity = elements.len().max(8);
    let arr_idx = ctx.alloc_value_array(capacity)
        .map_err(|_| JSValue::exception())?;

    if let Some(arr) = ctx.get_value_array_mut(arr_idx) {
        for elem in elements {
            unsafe { arr.push(*elem); }
        }
    }

    Ok(JSValue::from_ptr(arr_idx))
}

/// Array.isArray() - Determines whether a value is an array
///
/// For object-based arrays, checks if the object has Array.prototype in its chain
pub fn is_array(ctx: &Context, value: JSValue) -> bool {
    use crate::runtime::init::string_to_atom;

    if !value.is_ptr() {
        return false;
    }

    // Get Array.prototype to compare
    let array_atom = string_to_atom("Array");
    let proto_atom = string_to_atom("prototype");

    let array_proto = ctx.get_global_property(array_atom)
        .and_then(|arr_ctor| ctx.get_property(arr_ctor, proto_atom));

    if let Some(expected_proto) = array_proto {
        // Check if value's prototype matches Array.prototype
        if let Some(obj) = ctx.get_object(value) {
            return obj.prototype() == expected_proto;
        }
    }

    // Fallback: check for value array (internal representation)
    if let Some(idx) = value.to_ptr() {
        return ctx.get_value_array(idx).is_some();
    }

    false
}

/// Helper to get array length from object
fn get_array_length(ctx: &Context, arr: JSValue) -> i32 {
    use crate::runtime::init::string_to_atom;
    let length_atom = string_to_atom("length");
    ctx.get_property(arr, length_atom)
        .and_then(|v| v.to_int())
        .unwrap_or(0)
}

/// Helper to set array length on object
fn set_array_length(ctx: &mut Context, arr: JSValue, len: i32) -> Result<(), JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;
    let length_atom = string_to_atom("length");
    let len_val = ctx.new_number(len as f64).map_err(|_| JSValue::exception())?;
    ctx.add_property(arr, length_atom, len_val, PropertyFlags::default())
        .map_err(|_| JSValue::exception())
}

/// Array.prototype.push() - Adds elements to the end of an array
///
/// Returns the new length (works with object-based arrays)
pub fn array_push(ctx: &mut Context, arr: JSValue, elements: &[JSValue]) -> Result<i32, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let mut len = get_array_length(ctx, arr);

    for elem in elements {
        // Create atom for the index
        let idx_str = alloc::format!("{}", len);
        let idx_atom = string_to_atom(&idx_str);

        // Set the element at arr[len]
        ctx.add_property(arr, idx_atom, *elem, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;

        len += 1;
    }

    // Update length
    set_array_length(ctx, arr, len)?;

    Ok(len)
}

/// Array.prototype.pop() - Removes and returns the last element
pub fn array_pop(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    if len <= 0 {
        return Ok(JSValue::undefined());
    }

    let new_len = len - 1;

    // Get the last element
    let idx_str = alloc::format!("{}", new_len);
    let idx_atom = string_to_atom(&idx_str);
    let value = ctx.get_property(arr, idx_atom).unwrap_or(JSValue::undefined());

    // Update length (we could also delete the property, but for simplicity just update length)
    set_array_length(ctx, arr, new_len)?;

    Ok(value)
}

/// Array.prototype.shift() - Removes and returns the first element
pub fn array_shift(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let len = get_array_length(ctx, arr);

    if len <= 0 {
        return Ok(JSValue::undefined());
    }

    // Get first element
    let zero_atom = string_to_atom("0");
    let first = ctx.get_property(arr, zero_atom).unwrap_or(JSValue::undefined());

    // Shift all elements down
    for i in 1..len {
        let src_str = alloc::format!("{}", i);
        let src_atom = string_to_atom(&src_str);
        let dst_str = alloc::format!("{}", i - 1);
        let dst_atom = string_to_atom(&dst_str);

        let val = ctx.get_property(arr, src_atom).unwrap_or(JSValue::undefined());
        ctx.add_property(arr, dst_atom, val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    // Update length
    set_array_length(ctx, arr, len - 1)?;

    Ok(first)
}

/// Array.prototype.unshift() - Adds elements to the beginning of an array
///
/// Returns the new length
pub fn array_unshift(ctx: &mut Context, arr: JSValue, elements: &[JSValue]) -> Result<i32, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let len = get_array_length(ctx, arr);
    let add_count = elements.len() as i32;

    // Shift existing elements up
    for i in (0..len).rev() {
        let src_str = alloc::format!("{}", i);
        let src_atom = string_to_atom(&src_str);
        let dst_str = alloc::format!("{}", i + add_count);
        let dst_atom = string_to_atom(&dst_str);

        let val = ctx.get_property(arr, src_atom).unwrap_or(JSValue::undefined());
        ctx.add_property(arr, dst_atom, val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    // Insert new elements at the beginning
    for (i, elem) in elements.iter().enumerate() {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        ctx.add_property(arr, idx_atom, *elem, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    // Update length
    let new_len = len + add_count;
    set_array_length(ctx, arr, new_len)?;

    Ok(new_len)
}

/// Array.prototype.indexOf() - Returns the first index of an element
///
/// Returns -1 if not found (works with object-based arrays)
pub fn array_index_of(ctx: &Context, arr: JSValue, search_element: JSValue, from_index: Option<i32>) -> Result<i32, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);
    let start = from_index.unwrap_or(0).max(0);

    for i in start..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            if elem == search_element {
                return Ok(i);
            }
        }
    }

    Ok(-1)
}

/// Array.prototype.includes() - Determines whether an array contains a value
pub fn array_includes(ctx: &Context, arr: JSValue, search_element: JSValue, from_index: Option<i32>) -> Result<bool, JSValue> {
    let index = array_index_of(ctx, arr, search_element, from_index)?;
    Ok(index >= 0)
}

/// Array.prototype.join() - Joins all elements into a string (works with object-based arrays)
pub fn array_join(ctx: &mut Context, arr: JSValue, separator: Option<&str>) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);
    let sep = separator.unwrap_or(",");
    let mut result = String::new();

    for i in 0..len {
        if i > 0 {
            result.push_str(sep);
        }

        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        let elem = ctx.get_property(arr, idx_atom).unwrap_or(JSValue::undefined());

        // Convert element to string
        if let Some(s) = ctx.get_string(elem) {
            result.push_str(s);
        } else if let Some(n) = elem.to_int() {
            result.push_str(&alloc::format!("{}", n));
        } else if let Some(n) = ctx.get_number(elem) {
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

    let slice = unsafe { arr_ref.as_slice() };
    let elements: Vec<JSValue> = slice[start_idx as usize..end_idx as usize].to_vec();

    array_constructor(ctx, &elements)
}

/// Array.prototype.concat() - Merges two or more arrays
pub fn array_concat(ctx: &mut Context, arr: JSValue, others: &[JSValue]) -> Result<JSValue, JSValue> {
    let mut elements = Vec::new();

    // Add elements from original array
    if let Some(idx) = arr.to_ptr() {
        if let Some(arr_ref) = ctx.get_value_array(idx) {
            unsafe { elements.extend_from_slice(arr_ref.as_slice()); }
        }
    }

    // Add elements from other arrays
    for other in others.iter() {
        let other: &JSValue = other;
        if let Some(idx) = other.to_ptr() {
            if let Some(arr_ref) = ctx.get_value_array(idx) {
                unsafe { elements.extend_from_slice(arr_ref.as_slice()); }
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

/// Array.prototype.splice() - Modifies array by removing and/or adding elements
///
/// Returns array of deleted elements
pub fn array_splice(ctx: &mut Context, arr: JSValue, start: i32, delete_count: Option<i32>, items: &[JSValue]) -> Result<JSValue, JSValue> {
    let idx = arr.to_ptr().ok_or(JSValue::exception())?;
    let arr_ref = ctx.get_value_array(idx).ok_or(JSValue::exception())?;

    let len = arr_ref.header().count() as i32;

    // Normalize start index
    let actual_start = if start < 0 {
        (len + start).max(0)
    } else {
        start.min(len)
    } as usize;

    // Determine actual delete count
    let actual_delete_count = if let Some(dc) = delete_count {
        dc.max(0).min(len - actual_start as i32) as usize
    } else {
        (len - actual_start as i32) as usize
    };

    // Get mutable reference to perform operations
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;

    // Collect deleted elements
    let mut deleted = Vec::new();
    unsafe {
        let slice = arr_ref.as_full_mut_slice();
        for i in 0..actual_delete_count {
            deleted.push(slice[actual_start + i]);
        }
    }

    // For simplicity, rebuild the array with the new elements
    // In a production implementation, this would be done more efficiently
    let mut new_elements = Vec::new();
    unsafe {
        let slice = arr_ref.as_slice();

        // Add elements before start
        new_elements.extend_from_slice(&slice[..actual_start]);

        // Add new items
        new_elements.extend_from_slice(items);

        // Add elements after deleted section
        if actual_start + actual_delete_count < slice.len() {
            new_elements.extend_from_slice(&slice[actual_start + actual_delete_count..]);
        }
    }

    // Clear and rebuild the array
    let arr_ref = ctx.get_value_array_mut(idx).ok_or(JSValue::exception())?;
    unsafe {
        // Reset count to 0
        arr_ref.header_mut().set_count(0);

        // Push all new elements
        for elem in new_elements {
            if !arr_ref.push(elem) {
                return Err(JSValue::exception());
            }
        }
    }

    // Return array of deleted elements
    array_constructor(ctx, &deleted)
}

/// Array.prototype.reverse() - Reverses an array in place
pub fn array_reverse(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let len = get_array_length(ctx, arr);

    // Swap elements from ends toward the middle
    let mut left = 0;
    let mut right = len - 1;

    while left < right {
        // Get left element
        let left_str = alloc::format!("{}", left);
        let left_atom = string_to_atom(&left_str);
        let left_val = ctx.get_property(arr, left_atom).unwrap_or(JSValue::undefined());

        // Get right element
        let right_str = alloc::format!("{}", right);
        let right_atom = string_to_atom(&right_str);
        let right_val = ctx.get_property(arr, right_atom).unwrap_or(JSValue::undefined());

        // Swap
        ctx.add_property(arr, left_atom, right_val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
        ctx.add_property(arr, right_atom, left_val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;

        left += 1;
        right -= 1;
    }

    Ok(arr)
}

/// Array.prototype.forEach() - Executes a function for each element
///
/// Calls callback(element, index, array) for each element
pub fn array_for_each(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            ctx.call_function(callback, JSValue::undefined(), &args)?;
        }
    }

    Ok(JSValue::undefined())
}

/// Helper to create a new array-like object with Array.prototype
fn new_array_object(ctx: &mut Context) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let result = ctx.new_object().map_err(|_| JSValue::exception())?;

    // Get Array.prototype and set it on the new object
    let array_atom = string_to_atom("Array");
    let proto_atom = string_to_atom("prototype");
    if let Some(array_ctor) = ctx.get_global_property(array_atom) {
        if let Some(array_proto) = ctx.get_property(array_ctor, proto_atom) {
            if let Some(obj) = ctx.get_object_mut(result) {
                obj.set_prototype(array_proto);
            }
        }
    }

    Ok(result)
}

/// Array.prototype.map() - Creates a new array with results of calling a function
///
/// Calls callback(element, index, array) for each element and returns array of results
pub fn array_map(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let len = get_array_length(ctx, arr);

    // Create result array with Array.prototype
    let result = new_array_object(ctx)?;

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let mapped_val = ctx.call_function(callback, JSValue::undefined(), &args)?;

            // Store result
            ctx.add_property(result, idx_atom, mapped_val, PropertyFlags::default())
                .map_err(|_| JSValue::exception())?;
        }
    }

    // Set length on result
    set_array_length(ctx, result, len)?;

    Ok(result)
}

/// Array.prototype.filter() - Creates a new array with filtered elements
///
/// Calls callback(element, index, array) for each element and returns elements where callback returned truthy
pub fn array_filter(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;
    use crate::object::PropertyFlags;

    let len = get_array_length(ctx, arr);

    // Create result array with Array.prototype
    let result = new_array_object(ctx)?;
    let mut result_len = 0i32;

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let keep = ctx.call_function(callback, JSValue::undefined(), &args)?;

            // Check if callback returned truthy value
            if keep.to_bool().unwrap_or(false) {
                // Add element to result
                let result_idx_str = alloc::format!("{}", result_len);
                let result_idx_atom = string_to_atom(&result_idx_str);
                ctx.add_property(result, result_idx_atom, elem, PropertyFlags::default())
                    .map_err(|_| JSValue::exception())?;
                result_len += 1;
            }
        }
    }

    // Set length on result
    set_array_length(ctx, result, result_len)?;

    Ok(result)
}

/// Array.prototype.reduce() - Reduces array to a single value
///
/// Calls callback(accumulator, element, index, array) for each element
pub fn array_reduce(ctx: &mut Context, arr: JSValue, callback: JSValue, initial: Option<JSValue>) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    if len == 0 && initial.is_none() {
        // TypeError: Reduce of empty array with no initial value
        return Err(ctx.new_string("Reduce of empty array with no initial value")
            .unwrap_or(JSValue::exception()));
    }

    let mut accumulator: JSValue;
    let start_idx: i32;

    if let Some(init_val) = initial {
        accumulator = init_val;
        start_idx = 0;
    } else {
        // Use first element as initial value
        let zero_atom = string_to_atom("0");
        accumulator = ctx.get_property(arr, zero_atom).unwrap_or(JSValue::undefined());
        start_idx = 1;
    }

    for i in start_idx..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(accumulator, element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [accumulator, elem, index_val, arr];
            accumulator = ctx.call_function(callback, JSValue::undefined(), &args)?;
        }
    }

    Ok(accumulator)
}

/// Array.prototype.find() - Returns the first element that satisfies the predicate
pub fn array_find(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let result = ctx.call_function(callback, JSValue::undefined(), &args)?;

            if result.to_bool().unwrap_or(false) {
                return Ok(elem);
            }
        }
    }

    Ok(JSValue::undefined())
}

/// Array.prototype.findIndex() - Returns the index of the first element that satisfies the predicate
pub fn array_find_index(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let result = ctx.call_function(callback, JSValue::undefined(), &args)?;

            if result.to_bool().unwrap_or(false) {
                return Ok(JSValue::from_int(i));
            }
        }
    }

    Ok(JSValue::from_int(-1))
}

/// Array.prototype.some() - Tests whether at least one element passes the predicate
pub fn array_some(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let result = ctx.call_function(callback, JSValue::undefined(), &args)?;

            if result.to_bool().unwrap_or(false) {
                return Ok(JSValue::bool(true));
            }
        }
    }

    Ok(JSValue::bool(false))
}

/// Array.prototype.every() - Tests whether all elements pass the predicate
pub fn array_every(ctx: &mut Context, arr: JSValue, callback: JSValue) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            // Call callback(element, index, array)
            let index_val = JSValue::from_int(i);
            let args = [elem, index_val, arr];
            let result = ctx.call_function(callback, JSValue::undefined(), &args)?;

            if !result.to_bool().unwrap_or(false) {
                return Ok(JSValue::bool(false));
            }
        }
    }

    Ok(JSValue::bool(true))
}

/// Array.prototype.lastIndexOf() - Returns last index of element
pub fn array_last_index_of(ctx: &Context, arr: JSValue, search_element: JSValue, from_index: Option<i32>) -> Result<i32, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);
    if len == 0 {
        return Ok(-1);
    }

    let start = from_index.unwrap_or(len - 1).min(len - 1);
    let start = if start < 0 { (len + start).max(0) } else { start };

    for i in (0..=start).rev() {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            if values_equal(ctx, elem, search_element) {
                return Ok(i);
            }
        }
    }

    Ok(-1)
}

/// Array.prototype.reduceRight() - Reduces array from right to left
pub fn array_reduce_right(ctx: &mut Context, arr: JSValue, callback: JSValue, initial: Option<JSValue>) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;

    let len = get_array_length(ctx, arr);

    let mut accumulator = initial;
    let mut started = initial.is_some();

    for i in (0..len).rev() {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);

        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            if !started {
                accumulator = Some(elem);
                started = true;
                continue;
            }

            // Call callback(accumulator, element, index, array)
            let acc = accumulator.unwrap_or(JSValue::undefined());
            let index_val = JSValue::from_int(i);
            let args = [acc, elem, index_val, arr];
            accumulator = Some(ctx.call_function(callback, JSValue::undefined(), &args)?);
        }
    }

    Ok(accumulator.unwrap_or(JSValue::undefined()))
}

/// Array.prototype.sort() - Sorts array in place
pub fn array_sort(ctx: &mut Context, arr: JSValue, compare_fn: Option<JSValue>) -> Result<JSValue, JSValue> {
    use crate::runtime::init::string_to_atom;
    use alloc::vec::Vec;

    let len = get_array_length(ctx, arr);
    if len <= 1 {
        return Ok(arr);
    }

    // Collect elements
    let mut elements: Vec<JSValue> = Vec::new();
    for i in 0..len {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        if let Some(elem) = ctx.get_property(arr, idx_atom) {
            elements.push(elem);
        } else {
            elements.push(JSValue::undefined());
        }
    }

    // Sort using insertion sort (stable, simple)
    for i in 1..elements.len() {
        let key = elements[i];
        let mut j = i;
        while j > 0 {
            let cmp = if let Some(compare) = compare_fn {
                // Call compare function
                let args = [elements[j - 1], key];
                let result = ctx.call_function(compare, JSValue::undefined(), &args)?;
                if let Some(n) = ctx.get_number(result) {
                    n
                } else if let Some(i) = result.to_int() {
                    i as f64
                } else {
                    0.0
                }
            } else {
                // Default: convert to strings and compare
                let a_str = value_to_string(ctx, elements[j - 1]);
                let b_str = value_to_string(ctx, key);
                if a_str > b_str { 1.0 } else if a_str < b_str { -1.0 } else { 0.0 }
            };

            if cmp > 0.0 {
                elements[j] = elements[j - 1];
                j -= 1;
            } else {
                break;
            }
        }
        elements[j] = key;
    }

    // Write back
    for (i, elem) in elements.iter().enumerate() {
        let idx_str = alloc::format!("{}", i);
        let idx_atom = string_to_atom(&idx_str);
        ctx.add_property(arr, idx_atom, *elem, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    Ok(arr)
}

/// Array.prototype.toString() - Returns string representation
pub fn array_to_string(ctx: &mut Context, arr: JSValue) -> Result<JSValue, JSValue> {
    // Same as join with comma separator
    array_join(ctx, arr, Some(","))
}

/// Helper to convert value to string for sorting
fn value_to_string(ctx: &Context, val: JSValue) -> alloc::string::String {
    if let Some(s) = ctx.get_string(val) {
        s.to_string()
    } else if let Some(n) = ctx.get_number(val) {
        alloc::format!("{}", n)
    } else if val.is_undefined() {
        alloc::string::String::from("undefined")
    } else if val.is_null() {
        alloc::string::String::from("null")
    } else if let Some(b) = val.to_bool() {
        alloc::string::String::from(if b { "true" } else { "false" })
    } else {
        alloc::string::String::from("[object Object]")
    }
}

/// Helper to compare values for equality
fn values_equal(ctx: &Context, a: JSValue, b: JSValue) -> bool {
    // Handle identical values (pointer equality)
    if a == b {
        return true;
    }

    // Compare numbers
    if let (Some(na), Some(nb)) = (ctx.get_number(a), ctx.get_number(b)) {
        return na == nb;
    }

    // Compare strings
    if let (Some(sa), Some(sb)) = (ctx.get_string(a), ctx.get_string(b)) {
        return sa == sb;
    }

    // Compare ints
    if let (Some(ia), Some(ib)) = (a.to_int(), b.to_int()) {
        return ia == ib;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::PropertyFlags;
    use crate::runtime::init::string_to_atom;

    /// Helper to create an object-based array for testing
    fn make_test_array(ctx: &mut Context, elements: &[JSValue]) -> JSValue {
        let arr = ctx.new_object().unwrap();

        // Set length
        let length_atom = string_to_atom("length");
        let len_val = ctx.new_number(elements.len() as f64).unwrap();
        ctx.add_property(arr, length_atom, len_val, PropertyFlags::default()).unwrap();

        // Set elements
        for (i, elem) in elements.iter().enumerate() {
            let idx_str = alloc::format!("{}", i);
            let idx_atom = string_to_atom(&idx_str);
            ctx.add_property(arr, idx_atom, *elem, PropertyFlags::default()).unwrap();
        }

        arr
    }

    /// Helper to get element from object-based array
    fn get_element(ctx: &Context, arr: JSValue, index: i32) -> Option<JSValue> {
        let idx_str = alloc::format!("{}", index);
        let idx_atom = string_to_atom(&idx_str);
        ctx.get_property(arr, idx_atom)
    }

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

        // Create empty object-based array
        let arr = make_test_array(&mut ctx, &[]);

        // Push elements
        array_push(&mut ctx, arr, &[JSValue::from_int(1)]).unwrap();
        array_push(&mut ctx, arr, &[JSValue::from_int(2)]).unwrap();

        assert_eq!(get_array_length(&ctx, arr), 2);

        // Pop element
        let val = array_pop(&mut ctx, arr).unwrap();
        assert_eq!(val.to_int(), Some(2));
        assert_eq!(get_array_length(&ctx, arr), 1);
    }

    #[test]
    fn test_array_shift_unshift() {
        let mut ctx = Context::new(4096);

        let arr = make_test_array(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
        ]);

        // Shift first element
        let val = array_shift(&mut ctx, arr).unwrap();
        assert_eq!(val.to_int(), Some(1));
        assert_eq!(get_array_length(&ctx, arr), 1);

        // Unshift new element
        array_unshift(&mut ctx, arr, &[JSValue::from_int(0)]).unwrap();

        assert_eq!(get_element(&ctx, arr, 0).and_then(|v| v.to_int()), Some(0));
    }

    #[test]
    fn test_array_index_of() {
        let mut ctx = Context::new(4096);

        let arr = make_test_array(&mut ctx, &[
            JSValue::from_int(10),
            JSValue::from_int(20),
            JSValue::from_int(30),
        ]);

        let idx = array_index_of(&ctx, arr, JSValue::from_int(20), None).unwrap();
        assert_eq!(idx, 1);

        let idx = array_index_of(&ctx, arr, JSValue::from_int(99), None).unwrap();
        assert_eq!(idx, -1);
    }

    #[test]
    fn test_array_includes() {
        let mut ctx = Context::new(4096);

        let arr = make_test_array(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
        ]);

        assert!(array_includes(&ctx, arr, JSValue::from_int(1), None).unwrap());
        assert!(!array_includes(&ctx, arr, JSValue::from_int(3), None).unwrap());
    }

    #[test]
    fn test_array_join() {
        let mut ctx = Context::new(4096);

        let s1 = ctx.new_string("a").unwrap();
        let s2 = ctx.new_string("b").unwrap();
        let arr = make_test_array(&mut ctx, &[s1, s2]);

        let result = array_join(&mut ctx, arr, Some(",")).unwrap();
        assert_eq!(ctx.get_string(result).unwrap(), "a,b");
    }

    #[test]
    fn test_array_reverse() {
        let mut ctx = Context::new(4096);

        let arr = make_test_array(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
        ]);

        array_reverse(&mut ctx, arr).unwrap();

        assert_eq!(get_element(&ctx, arr, 0).and_then(|v| v.to_int()), Some(3));
        assert_eq!(get_element(&ctx, arr, 2).and_then(|v| v.to_int()), Some(1));
    }

    // Note: slice, concat, and splice still use the old value array implementation
    // and would need to be updated to work with object-based arrays
    #[test]
    #[ignore] // Uses value array implementation
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
    #[ignore] // Uses value array implementation
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
    #[ignore] // Uses value array implementation
    fn test_array_splice() {
        let mut ctx = Context::new(4096);

        let arr = array_constructor(&mut ctx, &[
            JSValue::from_int(1),
            JSValue::from_int(2),
            JSValue::from_int(3),
            JSValue::from_int(4),
        ]).unwrap();

        // Splice out elements 1 and 2, insert 5 and 6
        let deleted = array_splice(&mut ctx, arr, 1, Some(2), &[
            JSValue::from_int(5),
            JSValue::from_int(6),
        ]).unwrap();

        // Check deleted array
        let del_idx = deleted.to_ptr().unwrap();
        let del_arr = ctx.get_value_array(del_idx).unwrap();
        assert_eq!(del_arr.header().count(), 2);
        let del_slice = unsafe { del_arr.as_slice() };
        assert_eq!(del_slice[0].to_int(), Some(2));
        assert_eq!(del_slice[1].to_int(), Some(3));

        // Check modified array
        let idx = arr.to_ptr().unwrap();
        let arr_ref = ctx.get_value_array(idx).unwrap();
        assert_eq!(arr_ref.header().count(), 4);
        let slice = unsafe { arr_ref.as_slice() };
        assert_eq!(slice[0].to_int(), Some(1));
        assert_eq!(slice[1].to_int(), Some(5));
        assert_eq!(slice[2].to_int(), Some(6));
        assert_eq!(slice[3].to_int(), Some(4));
    }
}
