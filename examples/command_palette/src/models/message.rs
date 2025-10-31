use chrono::{DateTime, Local};

/// Represents a message in the application
#[derive(Clone, PartialEq)]
pub struct Message {
    pub text: String,
    pub timestamp: DateTime<Local>,
    pub message_type: MessageType,
}

/// Types of messages with different visual styling
#[derive(Clone, PartialEq)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}
