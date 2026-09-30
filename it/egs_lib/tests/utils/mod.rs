//! Tests for EGS utilities
//!
//! Unit tests for the utils module.

use egs_lib::error::EgsError;
use egs_lib::utils::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty_valid() {
        let result = validate_not_empty("test", "field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_not_empty_empty() {
        let result = validate_not_empty("", "field");
        assert!(matches!(result, Err(EgsError::ValidationError(_))));
    }

    #[test]
    fn test_validate_range_valid() {
        let result = validate_range(5, 1, 10, "field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_range_too_low() {
        let result = validate_range(0, 1, 10, "field");
        assert!(matches!(result, Err(EgsError::ValidationError(_))));
    }

    #[test]
    fn test_validate_range_too_high() {
        let result = validate_range(11, 1, 10, "field");
        assert!(matches!(result, Err(EgsError::ValidationError(_))));
    }

    #[test]
    fn test_validate_pattern_valid() {
        let result = validate_pattern("test123", r"^[a-z]+\d+$", "field");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pattern_invalid() {
        let result = validate_pattern("Test123", r"^[a-z]+\d+$", "field");
        assert!(matches!(result, Err(EgsError::ValidationError(_))));
    }

    #[test]
    fn test_egs_error_types() {
        let err1 = EgsError::invalid_configuration("test");
        assert!(matches!(err1, EgsError::InvalidConfiguration(_)));

        let err2 = EgsError::invalid_message("test");
        assert!(matches!(err2, EgsError::InvalidMessage(_)));

        let err3 = EgsError::communication_error("test");
        assert!(matches!(err3, EgsError::CommunicationError(_)));

        let err4 = EgsError::authentication_error("test");
        assert!(matches!(err4, EgsError::AuthenticationError(_)));

        let err5 = EgsError::authorization_error("test");
        assert!(matches!(err5, EgsError::AuthorizationError(_)));

        let err6 = EgsError::not_found("test");
        assert!(matches!(err6, EgsError::NotFound(_)));

        let err7 = EgsError::already_exists("test");
        assert!(matches!(err7, EgsError::AlreadyExists(_)));

        let err8 = EgsError::internal_error("test");
        assert!(matches!(err8, EgsError::InternalError(_)));

        let err9 = EgsError::serialization_error("test");
        assert!(matches!(err9, EgsError::SerializationError(_)));

        let err10 = EgsError::deserialization_error("test");
        assert!(matches!(err10, EgsError::DeserializationError(_)));

        let err11 = EgsError::validation_error("test");
        assert!(matches!(err11, EgsError::ValidationError(_)));

        let err12 = EgsError::timeout_error("test");
        assert!(matches!(err12, EgsError::TimeoutError(_)));
    }

    #[test]
    fn test_egs_error_display() {
        let err = EgsError::invalid_configuration("Invalid config");
        assert_eq!(format!("{}", err), "Invalid configuration: Invalid config");
    }
}
