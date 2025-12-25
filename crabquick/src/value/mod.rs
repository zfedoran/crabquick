//! JavaScript value types
//!
//! This module contains the concrete types that represent JavaScript values:
//! - JSValue (tagged union for all JavaScript values)
//! - Strings (JSString)
//! - Arrays (JSValueArray, JSByteArray)
//! - Boxed Float64 values
//! - Atoms (interned strings for property names)

mod core;
pub mod string;
pub mod array;
pub mod boxed;
pub mod atom;

pub use core::JSValue;
pub use string::{JSString, JSStringHeader};
pub use array::{JSValueArray, JSValueArrayHeader, JSByteArray, JSByteArrayHeader};
pub use boxed::JSFloat64;
pub use atom::{JSAtom, AtomTable};
