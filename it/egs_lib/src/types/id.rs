//! Identifier types for EGS
//!
//! Provides strongly-typed identifiers for enterprise objects.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A unique identifier for EGS objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EgsId(Uuid);

impl EgsId {
    /// Creates a new EGS ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates an EGS ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Returns the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Returns the UUID as a string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for EgsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EgsId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EgsId> for Uuid {
    fn from(id: EgsId) -> Self {
        id.0
    }
}

/// Application identifier (trigram)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(String);

impl AppId {
    /// Creates a new application ID
    pub fn new(trigram: impl Into<String>) -> Self {
        Self(trigram.into())
    }

    /// Returns the trigram as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message type identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageType(String);

impl MessageType {
    /// Creates a new message type
    pub fn new(type_name: impl Into<String>) -> Self {
        Self(type_name.into())
    }

    /// Returns the type name as a string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
