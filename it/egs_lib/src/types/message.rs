//! Message types for ESB communication
//!
//! Defines the message structures used for communication between EGS components.

use crate::types::id::{AppId, EgsId, MessageType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A message for ESB communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: EgsId,
    /// Type of the message
    pub message_type: MessageType,
    /// Application that sent the message
    pub sender: AppId,
    /// Application that should receive the message
    pub receiver: Option<AppId>,
    /// Timestamp when the message was created
    pub timestamp: DateTime<Utc>,
    /// The actual payload of the message
    pub payload: Vec<u8>,
    /// Optional metadata
    pub metadata: Option<MessageMetadata>,
}

impl Message {
    /// Creates a new message
    pub fn new(
        message_type: MessageType,
        sender: AppId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: EgsId::new(),
            message_type,
            sender,
            receiver: None,
            timestamp: Utc::now(),
            payload,
            metadata: None,
        }
    }

    /// Creates a new message with receiver
    pub fn with_receiver(
        message_type: MessageType,
        sender: AppId,
        receiver: AppId,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            id: EgsId::new(),
            message_type,
            sender,
            receiver: Some(receiver),
            timestamp: Utc::now(),
            payload,
            metadata: None,
        }
    }

    /// Sets the receiver
    pub fn set_receiver(mut self, receiver: AppId) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Sets the metadata
    pub fn set_metadata(mut self, metadata: MessageMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Metadata associated with a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// Priority of the message
    pub priority: MessagePriority,
    /// Whether the message should be encrypted
    pub encrypted: bool,
    /// Optional correlation ID for tracking
    pub correlation_id: Option<EgsId>,
    /// Optional reply to message ID
    pub reply_to: Option<EgsId>,
}

impl MessageMetadata {
    /// Creates new metadata
    pub fn new() -> Self {
        Self {
            priority: MessagePriority::Normal,
            encrypted: false,
            correlation_id: None,
            reply_to: None,
        }
    }

    /// Sets the priority
    pub fn with_priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    /// Sets whether the message should be encrypted
    pub fn encrypted(mut self, encrypted: bool) -> Self {
        self.encrypted = encrypted;
        self
    }

    /// Sets the correlation ID
    pub fn with_correlation_id(mut self, id: EgsId) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Sets the reply to message ID
    pub fn with_reply_to(mut self, id: EgsId) -> Self {
        self.reply_to = Some(id);
        self
    }
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority messages
    Low,
    /// Normal priority messages (default)
    Normal,
    /// High priority messages
    High,
    /// Critical priority messages
    Critical,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}
