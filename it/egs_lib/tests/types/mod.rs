//! Tests for EGS types
//!
//! Unit tests for the types module.

use egs_lib::types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egs_id_new() {
        let id1 = EgsId::new();
        let id2 = EgsId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_egs_id_from_uuid() {
        let uuid = uuid::Uuid::new_v4();
        let id = EgsId::from_uuid(uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }

    #[test]
    fn test_egs_id_to_string() {
        let uuid = uuid::Uuid::new_v4();
        let id = EgsId::from_uuid(uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn test_app_id_new() {
        let app_id = AppId::new("TEST");
        assert_eq!(app_id.as_str(), "TEST");
    }

    #[test]
    fn test_app_id_display() {
        let app_id = AppId::new("TEST");
        assert_eq!(format!("{}", app_id), "TEST");
    }

    #[test]
    fn test_message_type_new() {
        let msg_type = MessageType::new("test.message");
        assert_eq!(msg_type.as_str(), "test.message");
    }

    #[test]
    fn test_message_type_display() {
        let msg_type = MessageType::new("test.message");
        assert_eq!(format!("{}", msg_type), "test.message");
    }

    #[test]
    fn test_message_new() {
        let msg_type = MessageType::new("test.message");
        let sender = AppId::new("SENDER");
        let payload = b"test payload".to_vec();

        let msg = Message::new(msg_type, sender, payload);

        assert_eq!(msg.message_type, msg_type);
        assert_eq!(msg.sender, sender);
        assert_eq!(msg.payload, payload);
        assert!(msg.receiver.is_none());
        assert!(msg.metadata.is_none());
    }

    #[test]
    fn test_message_with_receiver() {
        let msg_type = MessageType::new("test.message");
        let sender = AppId::new("SENDER");
        let receiver = AppId::new("RECEIVER");
        let payload = b"test payload".to_vec();

        let msg = Message::with_receiver(msg_type, sender, receiver, payload);

        assert_eq!(msg.message_type, msg_type);
        assert_eq!(msg.sender, sender);
        assert_eq!(msg.receiver, Some(receiver));
        assert_eq!(msg.payload, payload);
    }

    #[test]
    fn test_message_set_receiver() {
        let msg_type = MessageType::new("test.message");
        let sender = AppId::new("SENDER");
        let receiver = AppId::new("RECEIVER");
        let payload = b"test payload".to_vec();

        let msg = Message::new(msg_type, sender.clone(), payload)
            .set_receiver(receiver.clone());

        assert_eq!(msg.sender, sender);
        assert_eq!(msg.receiver, Some(receiver));
    }

    #[test]
    fn test_message_metadata_new() {
        let metadata = MessageMetadata::new();
        assert_eq!(metadata.priority, MessagePriority::Normal);
        assert!(!metadata.encrypted);
        assert!(metadata.correlation_id.is_none());
        assert!(metadata.reply_to.is_none());
    }

    #[test]
    fn test_message_metadata_with_priority() {
        let metadata = MessageMetadata::new().with_priority(MessagePriority::High);
        assert_eq!(metadata.priority, MessagePriority::High);
    }

    #[test]
    fn test_message_metadata_encrypted() {
        let metadata = MessageMetadata::new().encrypted(true);
        assert!(metadata.encrypted);
    }

    #[test]
    fn test_egs_timestamp_now() {
        let ts = EgsTimestamp::now();
        let now = chrono::Utc::now();
        let diff = now - *ts.as_datetime();
        assert!(diff.num_seconds() < 1);
    }

    #[test]
    fn test_egs_timestamp_to_rfc3339() {
        let dt = chrono::DateTime::<chrono::Utc>::from(
            chrono::DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
        );
        let ts = EgsTimestamp::from_datetime(dt);
        assert_eq!(ts.to_rfc3339(), "2023-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_egs_duration_from_secs() {
        let duration = EgsDuration::from_secs(60);
        assert_eq!(duration.as_secs(), 60);
    }

    #[test]
    fn test_egs_duration_from_millis() {
        let duration = EgsDuration::from_millis(1000);
        assert_eq!(duration.as_millis(), 1000);
    }

    #[test]
    fn test_message_priority_default() {
        assert_eq!(MessagePriority::default(), MessagePriority::Normal);
    }
}
