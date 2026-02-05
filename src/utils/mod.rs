//! Shared utilities

pub mod fs;
pub mod math;

/// Selects between a singular or plural value based on a count.
///
/// * **1** is singular.
/// * **0** and **2+** are plural.
///
/// # Arguments
///
/// * `count` - The number of items.
/// * `singular` - The value to return if the count is exactly 1.
/// * `plural_val` - The value to return if the count is 0 or > 1.
#[must_use]
#[inline]
pub fn plural<T>(count: impl Into<usize>, singular: T, plural: T) -> T {
    if count.into() == 1 { singular } else { plural }
}
