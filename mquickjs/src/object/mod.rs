//! Object system implementation
//!
//! JavaScript objects, property management, prototypes, arrays, and functions.

pub mod object;
pub mod property;
pub mod array;
pub mod function;
pub mod string;

// Re-exports
pub use object::JSObject;
pub use property::{Property, PropertyTable};
pub use array::JSArray;
pub use function::{JSFunction, JSClosure};
pub use string::JSString;
