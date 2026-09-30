//! Validation utilities for EGS
//!
//! Provides validation functions and traits for validating data structures.

use crate::error::EgsError;
use validator::Validate;

/// Validates a struct that implements Validate trait
pub fn validate<T: Validate>(value: &T) -> Result<(), EgsError> {
    value
        .validate()
        .map_err(|e| EgsError::validation_error(format!("{:?}", e)))?;
    Ok(())
}

/// Validates a struct and returns validation errors
pub fn validate_with_errors<T: Validate>(value: &T) -> Result<(), Vec<String>> {
    value
        .validate()
        .map_err(|e| {
            e.field_errors()
                .into_iter()
                .flat_map(|(field, errors)| {
                    errors.iter().map(move |e| format!("Field '{}': {}", field, e))
                })
                .collect()
        })?;
    Ok(())
}

/// Trait for custom validation
pub trait EgsValidate {
    /// Validates the object
    fn egs_validate(&self) -> Result<(), EgsError>;
}

impl<T: Validate> EgsValidate for T {
    fn egs_validate(&self) -> Result<(), EgsError> {
        validate(self)
    }
}

/// Validates that a string is not empty
pub fn validate_not_empty(value: &str, field_name: &str) -> Result<(), EgsError> {
    if value.is_empty() {
        return Err(EgsError::validation_error(format!(
            "Field '{}' cannot be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validates that a value is within a range
pub fn validate_range<T: PartialOrd + std::fmt::Display + Copy>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<(), EgsError> {
    if value < min || value > max {
        return Err(EgsError::validation_error(format!(
            "Field '{}' must be between {} and {}, got {}",
            field_name, min, max, value
        )));
    }
    Ok(())
}

/// Validates that a string matches a regex pattern
pub fn validate_pattern(
    value: &str,
    pattern: &str,
    field_name: &str,
) -> Result<(), EgsError> {
    use regex::Regex;
    let re = Regex::new(pattern).map_err(|e| {
        EgsError::validation_error(format!("Invalid regex pattern '{}': {}", pattern, e))
    })?;
    if !re.is_match(value) {
        return Err(EgsError::validation_error(format!(
            "Field '{}' does not match pattern '{}'",
            field_name, pattern
        )));
    }
    Ok(())
}
