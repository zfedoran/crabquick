//! Runtime support (type conversions, operators, global functions)

pub mod conversion;
pub mod operators;
pub mod compare;
pub mod globals;
pub mod init;

// Re-exports
pub use conversion::{to_number, to_int32, to_string, to_boolean};
pub use operators::{add, subtract, multiply, divide};
pub use compare::{strict_equal, abstract_equal, less_than};
pub use globals::{parse_int, parse_float, is_nan, is_finite};
pub use init::init_runtime;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;

    #[test]
    fn test_init_runtime() {
        let mut ctx = Context::new(32768); // 32KB for property tables
        let result = init_runtime(&mut ctx);
        assert!(result.is_ok());
    }
}
