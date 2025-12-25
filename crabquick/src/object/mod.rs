//! Object system implementation
//!
//! JavaScript objects, property management, prototypes, arrays, and functions.

pub mod class;
pub mod object;
pub mod property;
pub mod array;
pub mod function;
pub mod string;

// Re-exports
pub use class::JSClassID;
pub use object::{JSObject, JSArrayData};
pub use property::{Property, PropertyFlags, PropertyTable, PropertyTableHeader};
pub use array::JSArray;
pub use function::{JSFunction, JSClosure};
pub use string::JSString;
