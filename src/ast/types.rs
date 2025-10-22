//! # Type Definitions
//!
//! This module defines the set of basic data types available in the Nebulang language
//! and implements logic for type compatibility checks.

/// Represents the fundamental data types in Nebulang.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A whole number type, typically a 64-bit signed integer.
    Integer,
    /// A floating-point number type.
    Float,
    /// A sequence of characters.
    String,
    /// A boolean type, representing `true` or `false`.
    Boolean,
    /// A type that is currently unknown (e.g., during initial parsing or type inference).
    Unknown,
}

impl Type {
    /// Checks if this type is compatible with another type for operations or assignments.
    ///
    /// Compatibility allows for certain implicit conversions (e.g., Integer to Float).
    ///
    /// # Arguments
    ///
    /// * `other` - The other type to check compatibility against.
    ///
    /// # Returns
    ///
    /// `true` if the types are compatible, `false` otherwise.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        match (self, other) {
            // Unknown is compatible with anything and vice versa, allowing flexibility in type checking.
            (Self::Unknown, _) | (_, Self::Unknown) => true,
            // Integer and Float are compatible with each other.
            (Self::Integer, Self::Float) | (Self::Float, Self::Integer) => true,
            // All other types must be strictly equal.
            (a, b) => a == b,
        }
    }
}
