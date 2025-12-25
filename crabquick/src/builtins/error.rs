//! Error built-in constructors and methods
//!
//! Implements Error, TypeError, ReferenceError, SyntaxError, RangeError,
//! URIError, EvalError and Error.prototype methods

use crate::context::Context;
use crate::value::{JSValue, JSAtom};
use crate::object::PropertyFlags;

/// Error types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorType {
    Error,
    TypeError,
    ReferenceError,
    SyntaxError,
    RangeError,
    URIError,
    EvalError,
}

impl ErrorType {
    pub fn name(&self) -> &'static str {
        match self {
            ErrorType::Error => "Error",
            ErrorType::TypeError => "TypeError",
            ErrorType::ReferenceError => "ReferenceError",
            ErrorType::SyntaxError => "SyntaxError",
            ErrorType::RangeError => "RangeError",
            ErrorType::URIError => "URIError",
            ErrorType::EvalError => "EvalError",
        }
    }
}

/// Creates an error object
pub fn create_error(ctx: &mut Context, error_type: ErrorType, message: Option<&str>) -> Result<JSValue, JSValue> {
    let err = ctx.new_object().map_err(|_| JSValue::exception())?;

    // Set name property
    let name_atom = JSAtom::from_id(1); // Simplified: should use proper atom for "name"
    let name_val = ctx.new_string(error_type.name()).map_err(|_| JSValue::exception())?;
    ctx.add_property(err, name_atom, name_val, PropertyFlags::default())
        .map_err(|_| JSValue::exception())?;

    // Set message property
    if let Some(msg) = message {
        let msg_atom = JSAtom::from_id(2); // Simplified: should use proper atom for "message"
        let msg_val = ctx.new_string(msg).map_err(|_| JSValue::exception())?;
        ctx.add_property(err, msg_atom, msg_val, PropertyFlags::default())
            .map_err(|_| JSValue::exception())?;
    }

    // TODO: Add stack trace

    Ok(err)
}

/// Error() constructor
pub fn error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::Error, message)
}

/// TypeError() constructor
pub fn type_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::TypeError, message)
}

/// ReferenceError() constructor
pub fn reference_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::ReferenceError, message)
}

/// SyntaxError() constructor
pub fn syntax_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::SyntaxError, message)
}

/// RangeError() constructor
pub fn range_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::RangeError, message)
}

/// URIError() constructor
pub fn uri_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::URIError, message)
}

/// EvalError() constructor
pub fn eval_error_constructor(ctx: &mut Context, message: Option<&str>) -> Result<JSValue, JSValue> {
    create_error(ctx, ErrorType::EvalError, message)
}

/// Error.prototype.toString() - Returns string representation
pub fn to_string(ctx: &mut Context, error: JSValue) -> Result<JSValue, JSValue> {
    // Simplified: just return "[ErrorType: message]"
    let name_atom = JSAtom::from_id(1);
    let msg_atom = JSAtom::from_id(2);

    let name = ctx.get_property(error, name_atom)
        .and_then(|v| ctx.get_string(v))
        .unwrap_or("Error");

    let message = ctx.get_property(error, msg_atom)
        .and_then(|v| ctx.get_string(v))
        .unwrap_or("");

    let result = if message.is_empty() {
        alloc::format!("{}", name)
    } else {
        alloc::format!("{}: {}", name, message)
    };

    ctx.new_string(&result).map_err(|_| JSValue::exception())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_constructor() {
        let mut ctx = Context::new(4096);

        let err = error_constructor(&mut ctx, Some("test error")).unwrap();
        assert!(err.is_object());
    }

    #[test]
    fn test_type_error_constructor() {
        let mut ctx = Context::new(4096);

        let err = type_error_constructor(&mut ctx, Some("not a function")).unwrap();
        assert!(err.is_object());
    }

    #[test]
    fn test_error_types() {
        assert_eq!(ErrorType::Error.name(), "Error");
        assert_eq!(ErrorType::TypeError.name(), "TypeError");
        assert_eq!(ErrorType::ReferenceError.name(), "ReferenceError");
        assert_eq!(ErrorType::SyntaxError.name(), "SyntaxError");
    }
}
