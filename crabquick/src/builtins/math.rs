//! Math object built-in functions and constants
//!
//! Implements Math.abs, Math.floor, Math.ceil, Math.round, Math.trunc,
//! Math.min, Math.max, Math.pow, Math.sqrt, Math.sin, Math.cos, Math.tan,
//! Math.log, Math.exp, Math.random, and Math constants

// Math constants
pub const PI: f64 = core::f64::consts::PI;
pub const E: f64 = core::f64::consts::E;
pub const LN2: f64 = core::f64::consts::LN_2;
pub const LN10: f64 = core::f64::consts::LN_10;
pub const LOG2E: f64 = core::f64::consts::LOG2_E;
pub const LOG10E: f64 = core::f64::consts::LOG10_E;
pub const SQRT2: f64 = core::f64::consts::SQRT_2;
pub const SQRT1_2: f64 = core::f64::consts::FRAC_1_SQRT_2;

/// Math.abs() - Returns absolute value
#[inline]
pub fn abs(x: f64) -> f64 {
    libm::fabs(x)
}

/// Math.floor() - Returns largest integer less than or equal to x
#[inline]
pub fn floor(x: f64) -> f64 {
    libm::floor(x)
}

/// Math.ceil() - Returns smallest integer greater than or equal to x
#[inline]
pub fn ceil(x: f64) -> f64 {
    libm::ceil(x)
}

/// Math.round() - Returns value rounded to nearest integer
#[inline]
pub fn round(x: f64) -> f64 {
    libm::round(x)
}

/// Math.trunc() - Returns integer part of x
#[inline]
pub fn trunc(x: f64) -> f64 {
    libm::trunc(x)
}

/// Math.min() - Returns smallest of given numbers
pub fn min(args: &[f64]) -> f64 {
    args.iter().fold(f64::INFINITY, |a, &b| if a < b { a } else { b })
}

/// Math.max() - Returns largest of given numbers
pub fn max(args: &[f64]) -> f64 {
    args.iter().fold(f64::NEG_INFINITY, |a, &b| if a > b { a } else { b })
}

/// Math.pow() - Returns base raised to exponent power
#[inline]
pub fn pow(base: f64, exponent: f64) -> f64 {
    libm::pow(base, exponent)
}

/// Math.sqrt() - Returns square root
#[inline]
pub fn sqrt(x: f64) -> f64 {
    libm::sqrt(x)
}

/// Math.sin() - Returns sine
#[inline]
pub fn sin(x: f64) -> f64 {
    libm::sin(x)
}

/// Math.cos() - Returns cosine
#[inline]
pub fn cos(x: f64) -> f64 {
    libm::cos(x)
}

/// Math.tan() - Returns tangent
#[inline]
pub fn tan(x: f64) -> f64 {
    libm::tan(x)
}

/// Math.asin() - Returns arcsine
#[inline]
pub fn asin(x: f64) -> f64 {
    libm::asin(x)
}

/// Math.acos() - Returns arccosine
#[inline]
pub fn acos(x: f64) -> f64 {
    libm::acos(x)
}

/// Math.atan() - Returns arctangent
#[inline]
pub fn atan(x: f64) -> f64 {
    libm::atan(x)
}

/// Math.atan2() - Returns arctangent of quotient
#[inline]
pub fn atan2(y: f64, x: f64) -> f64 {
    libm::atan2(y, x)
}

/// Math.log() - Returns natural logarithm
#[inline]
pub fn log(x: f64) -> f64 {
    libm::log(x)
}

/// Math.log10() - Returns base-10 logarithm
#[inline]
pub fn log10(x: f64) -> f64 {
    libm::log10(x)
}

/// Math.log2() - Returns base-2 logarithm
#[inline]
pub fn log2(x: f64) -> f64 {
    libm::log2(x)
}

/// Math.exp() - Returns e raised to the power of x
#[inline]
pub fn exp(x: f64) -> f64 {
    libm::exp(x)
}

/// Math.random() - Returns pseudo-random number between 0 and 1
///
/// Simplified PRNG using linear congruential generator
pub fn random(state: &mut u64) -> f64 {
    // Linear congruential generator: x = (a * x + c) mod m
    const A: u64 = 1103515245;
    const C: u64 = 12345;
    const M: u64 = 1u64 << 31;

    *state = (A.wrapping_mul(*state).wrapping_add(C)) % M;
    (*state as f64) / (M as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        assert_eq!(abs(-5.0), 5.0);
        assert_eq!(abs(5.0), 5.0);
    }

    #[test]
    fn test_floor() {
        assert_eq!(floor(3.7), 3.0);
        assert_eq!(floor(-3.7), -4.0);
    }

    #[test]
    fn test_ceil() {
        assert_eq!(ceil(3.2), 4.0);
        assert_eq!(ceil(-3.2), -3.0);
    }

    #[test]
    fn test_round() {
        assert_eq!(round(3.5), 4.0);
        assert_eq!(round(3.4), 3.0);
    }

    #[test]
    fn test_trunc() {
        assert_eq!(trunc(3.7), 3.0);
        assert_eq!(trunc(-3.7), -3.0);
    }

    #[test]
    fn test_min_max() {
        assert_eq!(min(&[1.0, 2.0, 3.0]), 1.0);
        assert_eq!(max(&[1.0, 2.0, 3.0]), 3.0);
    }

    #[test]
    fn test_pow_sqrt() {
        assert_eq!(pow(2.0, 3.0), 8.0);
        assert_eq!(sqrt(9.0), 3.0);
    }

    #[test]
    fn test_trig() {
        assert!((sin(0.0) - 0.0).abs() < 0.0001);
        assert!((cos(0.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_random() {
        let mut state = 12345;
        let r1 = random(&mut state);
        let r2 = random(&mut state);

        assert!(r1 >= 0.0 && r1 < 1.0);
        assert!(r2 >= 0.0 && r2 < 1.0);
        assert_ne!(r1, r2); // Should be different
    }
}
