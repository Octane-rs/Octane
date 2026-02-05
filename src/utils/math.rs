/// Rounds a numeric value to the given number of decimal places.
///
/// # Arguments
///
/// - `value`: The value to round. Can be any type convertible into `f64`.
/// - `precision`: The number of digits to keep after the decimal point.
///
/// # Returns
///
/// The rounded value as `f64`.
#[inline]
pub fn round<T>(value: T, precision: impl Into<i32>) -> f64
where
    T: Into<f64>,
{
    let value: f64 = value.into();
    let factor = 10.0_f64.powi(precision.into());
    (value * factor).round() / factor
}

/// Rounds a value to the given number of decimal places after dividing it by a power-of-ten magnitude.
///
/// Useful for formatting values scaled by units (e.g., milliseconds to seconds).
///
/// # Arguments
///
/// - `value`: The value to round. Can be any type convertible into `f64`.
/// - `magnitude`: The power of ten to divide the value by.
/// - `precision`: Number of digits after the decimal point to round to.
///
/// # Returns
///
/// The scaled and rounded value as `f64`.
#[inline]
pub fn round_magnitude<T>(value: T, magnitude: impl Into<i32>, precision: impl Into<i32>) -> f64
where
    T: Into<f64>,
{
    round(value.into() / 10f64.powi(magnitude.into()), precision)
}

#[cfg(test)]
mod test {
    use crate::utils::math::{round, round_magnitude};

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_round() {
        use std::f64::consts::PI;

        assert_eq!(round(PI, 2), 3.14);
        assert_eq!(round(PI, 0), 3.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_round_magnitude() {
        // 12345 ms → 12.35 s
        assert_eq!(round_magnitude(12345, 3, 2), 12.35);
    }
}
