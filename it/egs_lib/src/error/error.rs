//! Custom error types for EGS
//!
//! Defines all error types used throughout the EGS ecosystem.

use thiserror::Error;

/// Main error type for EGS
#[derive(Debug, Error)]
pub enum EgsError {
    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    /// Invalid message
    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    /// Communication error
    #[error("Communication error: {0}")]
    CommunicationError(String),

    /// Authentication error
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Authorization error
    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    /// Not found error
    #[error("Not found: {0}")]
    NotFound(String),

    /// Already exists error
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    /// Validation error
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Timeout error
    #[error("Timeout error: {0}")]
    TimeoutError(String),
}

impl EgsError {
    /// Creates a new EGS error from a string message
    pub fn new(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Creates an invalid configuration error
    pub fn invalid_configuration(message: impl Into<String>) -> Self {
        Self::InvalidConfiguration(message.into())
    }

    /// Creates an invalid message error
    pub fn invalid_message(message: impl Into<String>) -> Self {
        Self::InvalidMessage(message.into())
    }

    /// Creates a communication error
    pub fn communication_error(message: impl Into<String>) -> Self {
        Self::CommunicationError(message.into())
    }

    /// Creates an authentication error
    pub fn authentication_error(message: impl Into<String>) -> Self {
        Self::AuthenticationError(message.into())
    }

    /// Creates an authorization error
    pub fn authorization_error(message: impl Into<String>) -> Self {
        Self::AuthorizationError(message.into())
    }

    /// Creates a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    /// Creates an already exists error
    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::AlreadyExists(message.into())
    }

    /// Creates an internal error
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// Creates a serialization error
    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Creates a deserialization error
    pub fn deserialization_error(message: impl Into<String>) -> Self {
        Self::DeserializationError(message.into())
    }

    /// Creates a validation error
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    /// Creates a timeout error
    pub fn timeout_error(message: impl Into<String>) -> Self {
        Self::TimeoutError(message.into())
    }
}
