//! JavaScript execution context
//!
//! The Context is the main entry point for interacting with the JavaScript engine.
//! It manages memory, the runtime environment, and provides the API for evaluating
//! JavaScript code.

use crate::memory::{Arena, GarbageCollector, HeapIndex, MemTag};
use crate::value::{JSValue, AtomTable};

/// JavaScript execution context
///
/// Manages memory allocation, garbage collection, and the runtime environment.
/// All JavaScript operations must be performed through a Context.
///
/// # Example
///
/// ```rust,ignore
/// use crabquick::Context;
///
/// let mut ctx = Context::new(8192);
/// let result = ctx.eval("1 + 1", "script.js", 0)?;
/// ```
pub struct Context {
    /// Memory arena for heap allocations
    arena: Arena,
    /// Garbage collector state
    gc: GarbageCollector,
    /// Atom table for interned strings
    atom_table: AtomTable,
    /// Global object (null until initialized)
    global_object: JSValue,
    /// Current exception value (if any)
    exception_value: JSValue,
    // TODO: Add more fields:
    // - class_array: Vec<JSClass>
    // - interrupt_handler: Option<InterruptHandler>
}

impl Context {
    /// Creates a new JavaScript context with the specified memory size
    ///
    /// # Arguments
    ///
    /// * `memory_size` - Size of the heap in bytes
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ctx = Context::new(8192); // 8 KB heap
    /// ```
    pub fn new(memory_size: usize) -> Self {
        let mut ctx = Context {
            arena: Arena::new(memory_size),
            gc: GarbageCollector::new(),
            atom_table: AtomTable::new(),
            global_object: JSValue::null(),
            exception_value: JSValue::undefined(),
        };

        // Initialize global object (store as null if it fails)
        // This is called here to ensure the global object is always available
        ctx.global_object = ctx.new_object().unwrap_or(JSValue::null());

        ctx
    }

    /// Evaluates JavaScript source code
    ///
    /// # Arguments
    ///
    /// * `source` - JavaScript source code
    /// * `filename` - Filename for error reporting
    /// * `eval_flags` - Evaluation flags
    ///
    /// # Returns
    ///
    /// The result of evaluating the script, or an exception value
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = ctx.eval("2 + 2", "calc.js", 0)?;
    /// ```
    pub fn eval(&mut self, _source: &str, _filename: &str, _eval_flags: i32) -> JSValue {
        // TODO: Compile and execute source code
        JSValue::undefined()
    }

    /// Triggers garbage collection
    pub fn gc(&mut self) {
        self.gc.collect(&mut self.arena);
    }

    /// Returns the current memory usage in bytes
    #[inline]
    pub fn memory_usage(&self) -> usize {
        self.arena.heap_usage()
    }

    /// Returns the total arena size in bytes
    #[inline]
    pub fn arena_size(&self) -> usize {
        self.arena.size()
    }

    /// Returns the amount of free memory in bytes
    #[inline]
    pub fn free_memory(&self) -> usize {
        self.arena.free_space()
    }

    /// Adds a GC root to protect a value from garbage collection
    pub fn add_root(&mut self, value: JSValue) {
        self.gc.add_root(value);
    }

    /// Removes a GC root
    pub fn remove_root(&mut self, value: JSValue) {
        self.gc.remove_root(value);
    }

    /// Allocates memory from the arena
    ///
    /// This is a low-level method for internal use.
    ///
    /// # Safety
    ///
    /// The caller must initialize the allocated memory properly.
    pub(crate) unsafe fn alloc_raw(
        &mut self,
        size: usize,
        mtag: crate::memory::MemTag,
    ) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        self.arena.alloc(size, mtag)
    }

    /// Gets a reference to the arena (for internal use)
    #[inline]
    pub(crate) fn arena(&self) -> &Arena {
        &self.arena
    }

    /// Gets a mutable reference to the arena (for internal use)
    #[inline]
    pub(crate) fn arena_mut(&mut self) -> &mut Arena {
        &mut self.arena
    }

    // ========== String Operations ==========

    /// Creates a new JavaScript string from a Rust &str
    ///
    /// The string is allocated on the heap and stored in UTF-8 format.
    pub fn new_string(&mut self, s: &str) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSString, JSStringHeader};

        let bytes = s.as_bytes();
        let len = bytes.len();

        // Check flags
        let is_ascii = JSString::check_ascii(bytes);
        let is_numeric = JSString::check_numeric(bytes);

        // Calculate total size: MemBlockHeader + JSStringHeader + UTF-8 data
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSString::alloc_size(len);

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::String)? };

        // Initialize the string header
        unsafe {
            let string: &mut JSString = self.arena.get_mut(index);
            *string.header_mut() = JSStringHeader::new(len, is_ascii, is_numeric);

            // Copy UTF-8 data
            let data_ptr = (string as *mut JSString as *mut u8)
                .add(core::mem::size_of::<JSStringHeader>());
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), data_ptr, len);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets a &str reference to a JavaScript string
    ///
    /// Returns None if the value is not a string.
    pub fn get_string(&self, val: JSValue) -> Option<&str> {
        let index = val.to_ptr()?;

        unsafe {
            // Check memory tag
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::String {
                return None;
            }

            let string: &crate::value::JSString = self.arena.get(index);
            Some(string.as_str())
        }
    }

    /// Creates a new JavaScript number from an f64
    ///
    /// If the value can be represented as an inline integer, returns an inline value.
    /// Otherwise, allocates a boxed Float64 on the heap.
    pub fn new_number(&mut self, value: f64) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::value::JSFloat64;

        // Try to inline as integer
        if JSFloat64::can_inline(value) {
            return Ok(JSValue::from_int(value as i32));
        }

        // Allocate boxed float64
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSFloat64::alloc_size();

        let index = unsafe { self.alloc_raw(total_size, MemTag::Float64)? };

        unsafe {
            let float64: &mut JSFloat64 = self.arena.get_mut(index);
            *float64 = JSFloat64::new(value);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets the numeric value of a JSValue
    ///
    /// Returns None if the value is not a number.
    pub fn get_number(&self, val: JSValue) -> Option<f64> {
        // Check if it's an inline integer
        if let Some(i) = val.to_int() {
            return Some(i as f64);
        }

        // Check if it's a boxed float64
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::Float64 {
                return None;
            }

            let float64: &crate::value::JSFloat64 = self.arena.get(index);
            Some(float64.value())
        }
    }

    // ========== Array Operations ==========

    /// Allocates a JSValueArray with the specified capacity
    ///
    /// The array is initially empty but has space for `capacity` elements.
    pub fn alloc_value_array(&mut self, capacity: usize) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSValueArray, JSValueArrayHeader};

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSValueArray::alloc_size(capacity);

        let index = unsafe { self.alloc_raw(total_size, MemTag::ValueArray)? };

        unsafe {
            let array: &mut JSValueArray = self.arena.get_mut(index);
            *array.header_mut() = JSValueArrayHeader::new(capacity);

            // Initialize all elements to undefined
            let slice = array.as_full_mut_slice();
            for elem in slice.iter_mut() {
                *elem = JSValue::undefined();
            }
        }

        Ok(index)
    }

    /// Allocates a JSByteArray with the specified capacity
    ///
    /// The array is initially empty but has space for `capacity` bytes.
    pub fn alloc_byte_array(&mut self, capacity: usize) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::value::{JSByteArray, JSByteArrayHeader};

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSByteArray::alloc_size(capacity);

        let index = unsafe { self.alloc_raw(total_size, MemTag::ByteArray)? };

        unsafe {
            let array: &mut JSByteArray = self.arena.get_mut(index);
            *array.header_mut() = JSByteArrayHeader::new(capacity);

            // Initialize all bytes to zero
            let slice = array.as_full_mut_slice();
            for byte in slice.iter_mut() {
                *byte = 0;
            }
        }

        Ok(index)
    }

    /// Gets a reference to a value array
    pub fn get_value_array(&self, index: HeapIndex) -> Option<&crate::value::JSValueArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ValueArray {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a value array
    pub fn get_value_array_mut(&mut self, index: HeapIndex) -> Option<&mut crate::value::JSValueArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ValueArray {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Gets a reference to a byte array
    pub fn get_byte_array(&self, index: HeapIndex) -> Option<&crate::value::JSByteArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ByteArray {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a byte array
    pub fn get_byte_array_mut(&mut self, index: HeapIndex) -> Option<&mut crate::value::JSByteArray> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ByteArray {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    // ========== Object Operations ==========

    /// Creates a new plain JavaScript object
    ///
    /// Returns a JSValue wrapping a pointer to the object on the heap.
    pub fn new_object(&mut self) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        self.new_object_with_proto(JSValue::null())
    }

    /// Creates a new JavaScript object with a specific prototype
    pub fn new_object_with_proto(
        &mut self,
        proto: JSValue,
    ) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::object::JSObject;

        // Calculate size: MemBlockHeader + JSObject
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + core::mem::size_of::<JSObject>();

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::Object)? };

        // Initialize the object
        unsafe {
            let obj: &mut JSObject = self.arena.get_mut(index);
            *obj = JSObject::new_plain(proto);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets a reference to an object
    ///
    /// Returns None if the value is not an object.
    pub fn get_object(&self, val: JSValue) -> Option<&crate::object::JSObject> {
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::Object {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to an object
    pub fn get_object_mut(&mut self, val: JSValue) -> Option<&mut crate::object::JSObject> {
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::Object {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Allocates a new property table with the specified capacity
    ///
    /// Returns the HeapIndex of the allocated property table.
    pub fn alloc_property_table(
        &mut self,
        capacity: u32,
    ) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::object::PropertyTableHeader;

        let alloc_size = PropertyTableHeader::allocation_size(capacity);
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>() + alloc_size;

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::PropertyTable)? };

        // Initialize the property table header
        unsafe {
            let table: &mut crate::object::PropertyTable = self.arena.get_mut(index);
            let header = table.header_mut();
            *header = PropertyTableHeader::new(capacity);

            // Calculate and set hash mask
            let hash_mask = PropertyTableHeader::calculate_hash_mask(capacity);
            header.set_hash_mask(hash_mask);
            let hash_table_size = header.hash_table_size() as usize;

            // Initialize hash table if needed
            if hash_mask != 0 {
                let hash_table_ptr = table.hash_table_ptr_mut();
                for i in 0..hash_table_size {
                    *hash_table_ptr.add(i) = u32::MAX; // Empty slot marker
                }
            }
        }

        Ok(index)
    }

    /// Gets a reference to a property table
    pub fn get_property_table(&self, index: HeapIndex) -> Option<&crate::object::PropertyTable> {
        if index.is_null() {
            return None;
        }

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::PropertyTable {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a property table
    pub fn get_property_table_mut(&mut self, index: HeapIndex) -> Option<&mut crate::object::PropertyTable> {
        if index.is_null() {
            return None;
        }

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::PropertyTable {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Looks up a property in an object's own properties (no prototype chain)
    ///
    /// Returns the property if found, None otherwise.
    pub fn find_own_property(
        &self,
        obj_val: JSValue,
        key: crate::value::JSAtom,
    ) -> Option<&crate::object::Property> {
        use crate::object::Property;

        let obj = self.get_object(obj_val)?;
        if !obj.has_properties() {
            return None;
        }

        let props_table = self.get_property_table(obj.props_index())?;

        unsafe {
            let header = props_table.header();
            let count = header.count();

            if count == 0 {
                return None;
            }

            // For small tables, use linear search
            if !header.has_hash_table() {
                let properties = props_table.properties();
                for prop in properties {
                    if prop.key() == key {
                        return Some(prop);
                    }
                }
                return None;
            }

            // For larger tables, use hash table
            let hash = key.id(); // Use atom ID as hash
            let hash_mask = header.hash_mask();
            let slot = (hash & hash_mask) as usize;

            let hash_table_ptr = props_table.hash_table_ptr();
            let mut prop_idx = *hash_table_ptr.add(slot);

            // Walk the hash chain
            let properties_ptr = props_table.properties_ptr();
            while prop_idx != u32::MAX {
                let prop = &*properties_ptr.add(prop_idx as usize);
                if prop.key() == key {
                    return Some(prop);
                }
                prop_idx = prop.hash_next();
            }

            None
        }
    }

    /// Looks up a property in an object (including prototype chain)
    ///
    /// Returns the property value if found.
    pub fn get_property(
        &self,
        obj_val: JSValue,
        key: crate::value::JSAtom,
    ) -> Option<JSValue> {
        let mut current = obj_val;
        let max_depth = 100; // Prevent infinite loops in broken prototype chains

        for _ in 0..max_depth {
            // Look in own properties
            if let Some(prop) = self.find_own_property(current, key) {
                return Some(prop.value());
            }

            // Walk up prototype chain
            let obj = self.get_object(current)?;
            let proto = obj.prototype();

            if proto.is_null() {
                // Reached end of prototype chain
                return None;
            }

            current = proto;
        }

        // Prototype chain too deep
        None
    }

    /// Adds a property to an object
    ///
    /// This adds to own properties only (doesn't affect prototype chain).
    /// If the object doesn't have a property table yet, one will be created.
    pub fn add_property(
        &mut self,
        obj_val: JSValue,
        key: crate::value::JSAtom,
        value: JSValue,
        flags: crate::object::PropertyFlags,
    ) -> Result<(), crate::memory::allocator::OutOfMemory> {
        use crate::object::Property;

        // Get or create property table
        let obj_index = obj_val.to_ptr().ok_or(crate::memory::allocator::OutOfMemory)?;

        let props_index = {
            let obj: &crate::object::JSObject = unsafe { self.arena.get(obj_index) };
            if !obj.has_properties() {
                // Create initial property table with enough capacity for global object + user vars
                let props_idx = self.alloc_property_table(64)?; // 64 slots for global + user properties
                let obj_mut: &mut crate::object::JSObject = unsafe { self.arena.get_mut(obj_index) };
                obj_mut.set_props_index(props_idx);
                props_idx
            } else {
                obj.props_index()
            }
        };

        // Add the property
        let props_table = self.get_property_table_mut(props_index)
            .ok_or(crate::memory::allocator::OutOfMemory)?;

        unsafe {
            let header = props_table.header_mut();
            let count = header.count();
            let capacity = header.capacity();

            // Check if we need to resize (not implemented yet - just fail if full)
            if count >= capacity {
                return Err(crate::memory::allocator::OutOfMemory);
            }

            let new_prop = Property::new_data(key, value, flags);
            let prop_idx = count;

            // Read hash table info before borrowing mutably
            let has_hash_table = header.has_hash_table();
            let hash_mask = if has_hash_table { header.hash_mask() } else { 0 };

            // Add to properties array
            let properties_ptr = props_table.properties_ptr_mut();
            *properties_ptr.add(prop_idx as usize) = new_prop;

            // Update hash table if present
            if has_hash_table {
                let hash = key.id();
                let slot = (hash & hash_mask) as usize;

                let hash_table_ptr = props_table.hash_table_ptr_mut();
                let first_in_slot = *hash_table_ptr.add(slot);

                // Link this property into the hash chain
                let prop_mut = &mut *properties_ptr.add(prop_idx as usize);
                prop_mut.set_hash_next(first_in_slot);

                // Update slot to point to new property
                *hash_table_ptr.add(slot) = prop_idx;
            }

            // Update count - need to get header again
            let header = props_table.header_mut();
            header.set_count(count + 1);
        }

        Ok(())
    }

    /// Gets the global object
    ///
    /// Returns the global object for this context.
    #[inline]
    pub fn global_object(&self) -> JSValue {
        self.global_object
    }

    /// Gets a property from the global object
    ///
    /// Returns the property value if found, None otherwise.
    pub fn get_global_property(&self, key: crate::value::JSAtom) -> Option<JSValue> {
        if self.global_object.is_null() {
            return None;
        }
        self.get_property(self.global_object, key)
    }

    /// Sets a property on the global object
    ///
    /// Creates the property if it doesn't exist, or updates it if it does.
    /// Note: This is a simplified implementation that always adds properties.
    /// Multiple properties with the same key may exist, but get_property will return the latest one.
    pub fn set_global_property(
        &mut self,
        key: crate::value::JSAtom,
        value: JSValue,
    ) -> Result<(), crate::memory::allocator::OutOfMemory> {
        if self.global_object.is_null() {
            return Err(crate::memory::allocator::OutOfMemory);
        }

        // Simply add the property
        // In a full implementation, we would check if it exists and update in place
        // For now, get_property will return the most recent property with this key
        self.add_property(self.global_object, key, value, crate::object::PropertyFlags::default())
    }

    // ========== VM Execution ==========

    /// Executes bytecode and returns the result
    ///
    /// This is the main entry point for running JavaScript bytecode.
    ///
    /// # Arguments
    ///
    /// * `bytecode_index` - HeapIndex pointing to a JSByteArray containing bytecode
    ///
    /// # Returns
    ///
    /// * `Ok(JSValue)` - The result of execution
    /// * `Err(JSValue)` - An exception value
    pub fn execute_bytecode(&mut self, bytecode_index: HeapIndex) -> Result<JSValue, JSValue> {
        use crate::vm::VM;

        let mut vm = VM::new();
        vm.execute(self, bytecode_index)
    }

    /// Calls a JavaScript function
    ///
    /// # Arguments
    ///
    /// * `func` - The function to call
    /// * `this_val` - The 'this' value for the call
    /// * `args` - The arguments to pass
    ///
    /// # Returns
    ///
    /// * `Ok(JSValue)` - The return value
    /// * `Err(JSValue)` - An exception value
    pub fn call_function(
        &mut self,
        func: JSValue,
        this_val: JSValue,
        args: &[JSValue],
    ) -> Result<JSValue, JSValue> {
        // Check if it's a native function
        let func_index = match func.to_ptr() {
            Some(idx) => idx,
            None => return Err(self.new_string("Not a function").unwrap_or(JSValue::undefined())),
        };

        unsafe {
            let header = self.arena.get_header(func_index);
            if header.mtag() == MemTag::CFunctionData {
                // It's a native function - call it directly
                let cfunc: &crate::object::function::JSCFunction = self.arena.get(func_index);
                let func_ptr = cfunc.func_ptr();
                return func_ptr(self, this_val, args);
            }
        }

        // TODO: Implement bytecode function calling
        // This requires:
        // 1. Extracting the function bytecode from func
        // 2. Setting up a call frame with args
        // 3. Executing the bytecode
        Ok(JSValue::undefined())
    }

    /// Creates a new native function
    ///
    /// # Arguments
    ///
    /// * `func_ptr` - The native function pointer
    /// * `length` - The argument count (for Function.length)
    ///
    /// # Returns
    ///
    /// A JSValue wrapping the native function
    pub fn new_native_function(
        &mut self,
        func_ptr: crate::object::function::NativeFn,
        length: u16,
    ) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::object::function::JSCFunction;

        // Calculate size: MemBlockHeader + JSCFunction
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + core::mem::size_of::<JSCFunction>();

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::CFunctionData)? };

        // Initialize the C function
        unsafe {
            let cfunc: &mut JSCFunction = self.arena.get_mut(index);
            *cfunc = JSCFunction::new(func_ptr, length);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets a reference to a native function
    pub fn get_native_function(&self, val: JSValue) -> Option<&crate::object::function::JSCFunction> {
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::CFunctionData {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Creates a new bytecode function object
    ///
    /// # Arguments
    ///
    /// * `bytecode_index` - HeapIndex pointing to the function's bytecode
    /// * `param_count` - Number of parameters
    /// * `local_count` - Number of local variables (including parameters)
    ///
    /// # Returns
    ///
    /// A JSValue wrapping the bytecode function
    pub fn new_bytecode_function(
        &mut self,
        bytecode_index: crate::memory::HeapIndex,
        param_count: u8,
        local_count: u8,
    ) -> Result<JSValue, crate::memory::allocator::OutOfMemory> {
        use crate::object::function::JSBytecodeFunction;

        // Calculate size: MemBlockHeader + JSBytecodeFunction
        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + core::mem::size_of::<JSBytecodeFunction>();

        // Allocate memory
        let index = unsafe { self.alloc_raw(total_size, MemTag::FunctionBytecode)? };

        // Initialize the bytecode function
        unsafe {
            let func: &mut JSBytecodeFunction = self.arena.get_mut(index);
            *func = JSBytecodeFunction::new(bytecode_index, param_count, local_count);
        }

        Ok(JSValue::from_ptr(index))
    }

    /// Gets a reference to a bytecode function
    pub fn get_bytecode_function(&self, val: JSValue) -> Option<&crate::object::function::JSBytecodeFunction> {
        let index = val.to_ptr()?;

        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::FunctionBytecode {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    // ========== Closure Operations ==========

    /// Allocates a JSVarRef on the heap
    ///
    /// A VarRef holds a captured variable value that can be shared between
    /// the enclosing function and any closures that capture it.
    ///
    /// # Arguments
    ///
    /// * `value` - The initial value for the variable reference
    ///
    /// # Returns
    ///
    /// The HeapIndex of the allocated VarRef
    pub fn alloc_var_ref(&mut self, value: JSValue) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::object::function::JSVarRef;

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + core::mem::size_of::<JSVarRef>();

        let index = unsafe { self.alloc_raw(total_size, MemTag::VarRef)? };

        unsafe {
            let var_ref: &mut JSVarRef = self.arena.get_mut(index);
            *var_ref = JSVarRef::new(value);
        }

        Ok(index)
    }

    /// Gets a reference to a VarRef
    pub fn get_var_ref(&self, index: HeapIndex) -> Option<&crate::object::function::JSVarRef> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::VarRef {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a VarRef
    pub fn get_var_ref_mut(&mut self, index: HeapIndex) -> Option<&mut crate::object::function::JSVarRef> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::VarRef {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Allocates a JSClosure on the heap
    ///
    /// A closure combines a function index with captured variable references.
    ///
    /// # Arguments
    ///
    /// * `bytecode_index` - HeapIndex pointing to the function's bytecode
    /// * `param_count` - Number of parameters
    /// * `local_count` - Number of local variables
    /// * `var_refs` - Array of HeapIndex values pointing to JSVarRef objects
    ///
    /// # Returns
    ///
    /// The HeapIndex of the allocated closure
    pub fn alloc_closure(
        &mut self,
        bytecode_index: HeapIndex,
        param_count: u8,
        local_count: u8,
        var_refs: &[HeapIndex],
    ) -> Result<HeapIndex, crate::memory::allocator::OutOfMemory> {
        use crate::object::function::JSClosure;

        let total_size = core::mem::size_of::<crate::memory::MemBlockHeader>()
            + JSClosure::alloc_size(var_refs.len());

        let index = unsafe { self.alloc_raw(total_size, MemTag::ClosureData)? };

        unsafe {
            let closure: &mut JSClosure = self.arena.get_mut(index);
            closure.bytecode_index = bytecode_index;
            closure.param_count = param_count;
            closure.local_count = local_count;
            closure.var_ref_count = var_refs.len() as u8;
            closure.reserved = 0;

            for (i, &vr_idx) in var_refs.iter().enumerate() {
                closure.set_var_ref(i, vr_idx);
            }
        }

        Ok(index)
    }

    /// Gets a reference to a closure
    pub fn get_closure(&self, index: HeapIndex) -> Option<&crate::object::function::JSClosure> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ClosureData {
                return None;
            }
            Some(self.arena.get(index))
        }
    }

    /// Gets a mutable reference to a closure
    pub fn get_closure_mut(&mut self, index: HeapIndex) -> Option<&mut crate::object::function::JSClosure> {
        unsafe {
            let header = self.arena.get_header(index);
            if header.mtag() != MemTag::ClosureData {
                return None;
            }
            Some(self.arena.get_mut(index))
        }
    }

    /// Checks if a value is a closure
    pub fn is_closure(&self, val: JSValue) -> bool {
        if let Some(index) = val.to_ptr() {
            unsafe {
                let header = self.arena.get_header(index);
                header.mtag() == MemTag::ClosureData
            }
        } else {
            false
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Arena and GC will be dropped automatically
        // TODO: Call finalizers on remaining objects if needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_new() {
        let ctx = Context::new(1024);
        // Memory usage is no longer 0 because we allocate a global object in new()
        assert!(ctx.memory_usage() > 0, "Should have allocated global object");
        assert_eq!(ctx.arena_size(), 1024);
        assert!(ctx.free_memory() < 1024, "Should have used some memory for global object");
        assert_eq!(ctx.memory_usage() + ctx.free_memory(), 1024);
    }

    #[test]
    fn test_context_gc() {
        let mut ctx = Context::new(2048);

        // Allocate some memory
        let idx1 = unsafe {
            ctx.alloc_raw(64, crate::memory::MemTag::Object).unwrap()
        };

        let val1 = JSValue::from_ptr(idx1);
        ctx.add_root(val1);

        // Allocate more
        let _idx2 = unsafe {
            ctx.alloc_raw(128, crate::memory::MemTag::String).unwrap()
        };

        let usage_before_gc = ctx.memory_usage();
        assert!(usage_before_gc > 0);

        // Run GC
        ctx.gc();

        // Memory usage should still be > 0 because we have a root
        let usage_after_gc = ctx.memory_usage();
        assert!(usage_after_gc > 0);

        // Clean up
        ctx.remove_root(val1);
    }

    #[test]
    fn test_context_roots() {
        let mut ctx = Context::new(2048);

        let idx = unsafe {
            ctx.alloc_raw(64, crate::memory::MemTag::Object).unwrap()
        };
        let val = JSValue::from_ptr(idx);

        // Add root
        ctx.add_root(val);

        // GC should preserve it
        ctx.gc();

        // Remove root
        ctx.remove_root(val);
    }

    #[test]
    fn test_context_memory_tracking() {
        let mut ctx = Context::new(1024);

        let initial_usage = ctx.memory_usage();
        // Initial usage is no longer 0 due to global object
        assert!(initial_usage > 0, "Should have some initial usage for global object");

        // Allocate something
        let _idx = unsafe {
            ctx.alloc_raw(32, crate::memory::MemTag::String).unwrap()
        };

        let usage_after_alloc = ctx.memory_usage();
        assert!(usage_after_alloc > 0);
        assert!(usage_after_alloc < 1024);

        let free_space = ctx.free_memory();
        assert_eq!(usage_after_alloc + free_space, 1024);
    }

    #[test]
    fn test_string_creation() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_string("hello").unwrap();
        assert!(val.is_ptr());

        let s = ctx.get_string(val).unwrap();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_string_utf8() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_string("你好世界").unwrap();
        let s = ctx.get_string(val).unwrap();
        assert_eq!(s, "你好世界");
    }

    #[test]
    fn test_number_inline() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_number(42.0).unwrap();
        assert!(val.is_int());
        assert_eq!(ctx.get_number(val), Some(42.0));
    }

    #[test]
    fn test_number_boxed() {
        let mut ctx = Context::new(2048);

        let val = ctx.new_number(3.14).unwrap();
        assert!(val.is_ptr());
        assert_eq!(ctx.get_number(val), Some(3.14));
    }

    #[test]
    fn test_value_array() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_value_array(10).unwrap();
        let array = ctx.get_value_array(idx).unwrap();

        assert_eq!(array.header().capacity(), 10);
        assert_eq!(array.header().count(), 0);
    }

    #[test]
    fn test_byte_array() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_byte_array(100).unwrap();
        let array = ctx.get_byte_array(idx).unwrap();

        assert_eq!(array.header().capacity(), 100);
        assert_eq!(array.header().count(), 0);
    }

    #[test]
    fn test_array_push_pop() {
        let mut ctx = Context::new(2048);

        let idx = ctx.alloc_value_array(5).unwrap();

        unsafe {
            let array = ctx.get_value_array_mut(idx).unwrap();

            // Push values
            assert!(array.push(JSValue::from_int(1)));
            assert!(array.push(JSValue::from_int(2)));
            assert!(array.push(JSValue::from_int(3)));

            assert_eq!(array.header().count(), 3);

            // Pop values
            assert_eq!(array.pop(), Some(JSValue::from_int(3)));
            assert_eq!(array.pop(), Some(JSValue::from_int(2)));
            assert_eq!(array.pop(), Some(JSValue::from_int(1)));
            assert_eq!(array.pop(), None);
        }
    }

    #[test]
    fn test_object_creation() {
        let mut ctx = Context::new(2048);

        let obj_val = ctx.new_object().unwrap();
        assert!(obj_val.is_ptr());

        let obj = ctx.get_object(obj_val).unwrap();
        assert!(obj.is_plain_object());
        assert!(!obj.has_properties());
        assert!(obj.is_extensible());
    }

    #[test]
    fn test_object_with_prototype() {
        let mut ctx = Context::new(2048);

        let proto = ctx.new_object().unwrap();
        let obj_val = ctx.new_object_with_proto(proto).unwrap();

        let obj = ctx.get_object(obj_val).unwrap();
        assert_eq!(obj.prototype(), proto);
    }

    #[test]
    fn test_property_table_allocation() {
        let mut ctx = Context::new(4096);

        // Allocate small property table (no hash table)
        let idx1 = ctx.alloc_property_table(4).unwrap();
        let table1 = ctx.get_property_table(idx1).unwrap();
        unsafe {
            let header = table1.header();
            assert_eq!(header.capacity(), 4);
            assert_eq!(header.count(), 0);
            assert!(!header.has_hash_table());
        }

        // Allocate large property table (with hash table)
        let idx2 = ctx.alloc_property_table(16).unwrap();
        let table2 = ctx.get_property_table(idx2).unwrap();
        unsafe {
            let header = table2.header();
            assert_eq!(header.capacity(), 16);
            assert_eq!(header.count(), 0);
            assert!(header.has_hash_table());
            assert_eq!(header.hash_mask(), 15); // 16 - 1
        }
    }

    #[test]
    fn test_add_property() {
        use crate::object::PropertyFlags;
        use crate::value::JSAtom;

        let mut ctx = Context::new(4096);

        let obj_val = ctx.new_object().unwrap();
        let key = JSAtom::from_id(1);
        let value = JSValue::from_int(42);

        // Add a property
        ctx.add_property(obj_val, key, value, PropertyFlags::default())
            .unwrap();

        // Object should now have a property table
        let obj = ctx.get_object(obj_val).unwrap();
        assert!(obj.has_properties());

        // Find the property
        let prop = ctx.find_own_property(obj_val, key).unwrap();
        assert_eq!(prop.value(), value);
        assert!(prop.flags().is_writable());
        assert!(prop.flags().is_enumerable());
        assert!(prop.flags().is_configurable());
    }

    #[test]
    fn test_property_lookup_chain() {
        use crate::object::PropertyFlags;
        use crate::value::JSAtom;

        let mut ctx = Context::new(8192);

        // Create prototype with a property
        let proto = ctx.new_object().unwrap();
        let key = JSAtom::from_id(1);
        let proto_value = JSValue::from_int(100);
        ctx.add_property(proto, key, proto_value, PropertyFlags::default())
            .unwrap();

        // Create object with prototype
        let obj = ctx.new_object_with_proto(proto).unwrap();

        // Should find property in prototype
        let found_value = ctx.get_property(obj, key);
        assert_eq!(found_value, Some(proto_value));
    }

    #[test]
    fn test_property_shadowing() {
        use crate::object::PropertyFlags;
        use crate::value::JSAtom;

        let mut ctx = Context::new(8192);

        // Create prototype with a property
        let proto = ctx.new_object().unwrap();
        let key = JSAtom::from_id(1);
        ctx.add_property(proto, key, JSValue::from_int(100), PropertyFlags::default())
            .unwrap();

        // Create object with same property
        let obj = ctx.new_object_with_proto(proto).unwrap();
        let obj_value = JSValue::from_int(200);
        ctx.add_property(obj, key, obj_value, PropertyFlags::default())
            .unwrap();

        // Should find own property (shadows prototype)
        let found_value = ctx.get_property(obj, key);
        assert_eq!(found_value, Some(obj_value));
    }

    #[test]
    fn test_multiple_properties() {
        use crate::object::PropertyFlags;
        use crate::value::JSAtom;

        let mut ctx = Context::new(8192);

        let obj = ctx.new_object().unwrap();

        // Add multiple properties
        for i in 0..10 {
            let key = JSAtom::from_id(i);
            let value = JSValue::from_int(i as i32 * 10);
            ctx.add_property(obj, key, value, PropertyFlags::default())
                .unwrap();
        }

        // Look up all properties
        for i in 0..10 {
            let key = JSAtom::from_id(i);
            let value = ctx.get_property(obj, key);
            assert_eq!(value, Some(JSValue::from_int(i as i32 * 10)));
        }
    }

    #[test]
    fn test_gc_compaction_frees_memory() {
        let mut ctx = Context::new(4096);

        // Allocate objects without rooting them - they should be collected
        for _ in 0..10 {
            let _ = ctx.new_object().unwrap();
            let _ = ctx.new_string("temporary string").unwrap();
        }

        let usage_before = ctx.memory_usage();
        assert!(usage_before > 0, "Should have allocated some memory");

        // Run GC - all objects should be collected since they're not rooted
        ctx.gc();

        let usage_after = ctx.memory_usage();

        // Memory should be freed (should be 0 or very close to 0)
        assert!(
            usage_after < usage_before,
            "GC should free memory: before={}, after={}",
            usage_before,
            usage_after
        );
        assert_eq!(
            usage_after, 0,
            "All unreachable objects should be collected, usage={}",
            usage_after
        );
    }

    #[test]
    fn test_gc_preserves_rooted_objects() {
        let mut ctx = Context::new(4096);

        // Allocate and root some objects
        let obj1 = ctx.new_object().unwrap();
        let obj2 = ctx.new_object().unwrap();
        let str1 = ctx.new_string("rooted string").unwrap();

        ctx.add_root(obj1);
        ctx.add_root(obj2);
        ctx.add_root(str1);

        // Allocate some garbage objects
        for _ in 0..5 {
            let _ = ctx.new_object().unwrap();
            let _ = ctx.new_string("garbage").unwrap();
        }

        let usage_before = ctx.memory_usage();

        // Run GC
        ctx.gc();

        let usage_after = ctx.memory_usage();

        // Some memory should be freed (the garbage objects)
        assert!(
            usage_after < usage_before,
            "GC should free garbage: before={}, after={}",
            usage_before,
            usage_after
        );

        // But rooted objects should still be accessible
        assert!(ctx.get_object(obj1).is_some());
        assert!(ctx.get_object(obj2).is_some());
        assert_eq!(ctx.get_string(str1), Some("rooted string"));

        // Clean up roots
        ctx.remove_root(obj1);
        ctx.remove_root(obj2);
        ctx.remove_root(str1);

        // Now everything should be collectable
        ctx.gc();
        assert_eq!(ctx.memory_usage(), 0);
    }

    #[test]
    fn test_gc_compaction_moves_objects() {
        let mut ctx = Context::new(8192);

        // Create some objects with gaps
        let obj1 = ctx.new_object().unwrap();
        ctx.add_root(obj1);

        let _garbage1 = ctx.new_object().unwrap(); // Will be collected

        let obj2 = ctx.new_object().unwrap();
        ctx.add_root(obj2);

        let _garbage2 = ctx.new_object().unwrap(); // Will be collected

        let obj3 = ctx.new_object().unwrap();
        ctx.add_root(obj3);

        let usage_before = ctx.memory_usage();

        // Run GC - should compact memory
        ctx.gc();

        let usage_after = ctx.memory_usage();

        // Memory should be compacted
        assert!(
            usage_after < usage_before,
            "GC should compact: before={}, after={}",
            usage_before,
            usage_after
        );

        // All rooted objects should still be accessible
        assert!(ctx.get_object(obj1).is_some());
        assert!(ctx.get_object(obj2).is_some());
        assert!(ctx.get_object(obj3).is_some());

        // Clean up
        ctx.remove_root(obj1);
        ctx.remove_root(obj2);
        ctx.remove_root(obj3);
    }

    #[test]
    fn test_native_function_as_property() {
        use crate::value::JSAtom;
        use crate::object::PropertyFlags;

        let mut ctx = Context::new(32768); // 32KB heap

        // Create a native function and add it as a property
        let test_fn = ctx.new_native_function(crate::builtins::native_functions::math_abs, 1).unwrap();
        assert!(ctx.get_native_function(test_fn).is_some());

        // Create a test object
        let test_obj = ctx.new_object().unwrap();

        // Add the function as a property
        let test_atom = JSAtom::from_id(12345);
        ctx.add_property(test_obj, test_atom, test_fn, PropertyFlags::default()).unwrap();

        // Retrieve it and verify it's still a native function
        let retrieved = ctx.get_property(test_obj, test_atom).unwrap();
        assert!(ctx.get_native_function(retrieved).is_some());
    }

    #[test]
    fn test_gc_with_object_references() {
        use crate::object::PropertyFlags;
        use crate::value::JSAtom;

        let mut ctx = Context::new(8192);

        // Create an object graph: obj1 -> obj2 -> obj3
        let obj3 = ctx.new_object().unwrap();
        let obj2 = ctx.new_object().unwrap();
        let obj1 = ctx.new_object().unwrap();

        // Link them together
        let key = JSAtom::from_id(1);
        ctx.add_property(obj1, key, obj2, PropertyFlags::default())
            .unwrap();
        ctx.add_property(obj2, key, obj3, PropertyFlags::default())
            .unwrap();

        // Only root obj1 - obj2 and obj3 should be kept alive through the reference
        ctx.add_root(obj1);

        // Allocate some garbage
        for _ in 0..5 {
            let _ = ctx.new_object().unwrap();
        }

        // Run GC
        ctx.gc();

        // All objects in the chain should still be accessible
        assert!(ctx.get_object(obj1).is_some());
        assert!(ctx.get_object(obj2).is_some());
        assert!(ctx.get_object(obj3).is_some());

        // Verify the links are still intact
        assert_eq!(ctx.get_property(obj1, key), Some(obj2));
        assert_eq!(ctx.get_property(obj2, key), Some(obj3));

        ctx.remove_root(obj1);
    }
}
