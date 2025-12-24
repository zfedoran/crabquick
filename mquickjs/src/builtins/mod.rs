//! JavaScript standard library built-in functions

pub mod object;
pub mod array;
pub mod string;
pub mod number;
pub mod boolean;
pub mod function;
pub mod math;
pub mod console;
pub mod error;

// Legacy modules (stubs for future implementation)
pub mod json;
pub mod regexp;
pub mod typed_array;

// Re-exports for convenience
pub use object::{object_constructor, object_keys, object_values, object_entries, object_assign, object_create};
pub use array::{array_constructor, is_array as array_is_array, array_push, array_pop};
pub use string::{string_constructor, string_length};
pub use number::{number_constructor, is_nan as number_is_nan, is_finite as number_is_finite};
pub use boolean::{boolean_constructor, to_boolean};
pub use console::{console_log, console_error, console_warn, console_info};
pub use error::{error_constructor, type_error_constructor, ErrorType};
