//! Common types and interfaces for communication.
//!
//! This module defines the shared types and interfaces used by
//! all communication implementations.

// TODO:
// - Implement PlacementDefault in the future

use iceoryx2_bb_container::{byte_string::FixedSizeByteString, vec::FixedSizeVec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::errors::Result;

/// High-level message type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum MessageType {
    Task = 0,
    Result = 1,
    Event = 2,
    Command = 3,
    Registration = 4,
    Heartbeat = 5,
}

/// Plugin event types as numeric values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum EventType {
    #[default]
    ResourceReady = 0,
    Started = 1,
    Failed = 2,
    Shutdown = 3,
    Progress = 4,
    Complete = 5,
}

/// Command types as numeric values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum CommandType {
    #[default]
    Stop = 0,
    Pause = 1,
    Resume = 2,
    Status = 3,
}

/// Container for message payloads.
#[derive(Debug)]
#[repr(C)]
pub struct MessagePayload {
    /// Message type.
    pub message_type: MessageType,

    /// Unique message ID.
    pub message_id: FixedSizeByteString<64>,

    /// Sender ID.
    pub sender_id: FixedSizeByteString<64>,

    /// Recipient ID (empty for broadcast).
    pub recipient_id: FixedSizeByteString<64>,

    /// Task ID (if applicable).
    pub has_task_id: bool,
    pub task_id: FixedSizeByteString<64>,

    /// The actual message content - varies based on message_type
    pub content: MessageContent,
}

impl MessagePayload {
    /// Create a new message payload.
    pub fn new(message_type: MessageType, sender_id: &str, recipient_id: &str) -> Result<Self> {
        let payload = MessagePayload {
            message_type: message_type,
            message_id: FixedSizeByteString::from_bytes(Uuid::new_v4().to_string().as_bytes())
                .unwrap(),
            sender_id: FixedSizeByteString::from_bytes(sender_id.as_bytes()).unwrap(),
            recipient_id: FixedSizeByteString::from_bytes(recipient_id.as_bytes()).unwrap(),
            has_task_id: false,
            task_id: FixedSizeByteString::from_bytes("".as_bytes()).unwrap(),
            content: MessageContent::default(),
        };

        Ok(payload)
    }

    /// Set the task ID for this message.
    pub fn with_task_id(mut self, task_id: &FixedSizeByteString<64>) -> Result<Self> {
        self.has_task_id = true;
        self.task_id = FixedSizeByteString::from_bytes(task_id.as_bytes()).unwrap();
        Ok(self)
    }

    /// Set task content.
    pub fn with_task(mut self, task: &TaskMessage) -> Result<Self> {
        self.content.task_data_size = task.data_size;
        self.content.task_priority = task.priority;
        self.content.task_timeout_ms = task.timeout_ms;

        for i in 0..task.data_size as usize {
            if i < task.data.len() {
                self.content.task_data.push(task.data[i]);
            }
        }

        Ok(self)
    }

    /// Set result content.
    pub fn with_result(mut self, result: &ResultMessage) -> Result<Self> {
        self.content.result_plugin_id = result.plugin_id.clone();
        self.content.result_success = result.success;
        self.content.result_has_error = result.has_error;
        self.content.event_error_message = result.error_message.clone();
        self.content.result_data_size = result.data_size;

        for i in 0..result.data_size as usize {
            if i < result.data.len() {
                self.content.result_data.push(result.data[i]);
            }
        }

        Ok(self)
    }

    /// Set event content.
    pub fn with_event(mut self, event: &EventMessage) -> Result<Self> {
        self.content.event_plugin_id = event.plugin_id.clone();
        self.content.event_type = event.event_type;
        self.content.event_error_message = event.error_message.clone();
        self.content.event_progress_percent = event.progress_percent;
        self.content.event_progress_message = event.progress_message.clone();
        self.content.event_success = event.success;

        Ok(self)
    }

    /// Set command content.
    pub fn with_command(mut self, command: &CommandMessage) -> Result<Self> {
        self.content.command_type = command.command_type;
        self.content.command_custom = command.custom_command.clone();
        self.content.command_param_count = command.param_count;

        // Copy parameters
        for i in 0..command.param_count as usize {
            if i < 16 {
                self.content.command_param_keys[i] = command.param_keys[i].clone();
                self.content.command_param_values[i] = command.param_values[i].clone();
            }
        }

        Ok(self)
    }

    /// Extract task message.
    pub fn to_task(&self) -> Result<TaskMessage> {
        let mut task = TaskMessage::default();

        // If has_task_id is true, copy task ID
        if self.has_task_id {
            task.task_id = self.task_id.clone();
        }

        task.data_size = self.content.task_data_size;
        task.priority = self.content.task_priority;
        task.timeout_ms = self.content.task_timeout_ms;

        // Copy data
        for i in 0..self.content.task_data_size as usize {
            if i < self.content.task_data.len() {
                task.data.push(self.content.task_data[i]);
            }
        }

        Ok(task)
    }

    /// Extract result message.
    pub fn to_result(&self) -> Result<ResultMessage> {
        let mut result = ResultMessage::default();

        // If has_task_id is true, copy task ID
        if self.has_task_id {
            result.task_id = self.task_id.clone();
        }

        result.plugin_id = self.content.result_plugin_id.clone();
        result.success = self.content.result_success;
        result.has_error = self.content.result_has_error;
        result.error_message = self.content.result_error_message.clone();
        result.data_size = self.content.result_data_size;

        for i in 0..self.content.result_data_size as usize {
            if i < self.content.result_data.len() {
                result.data.push(self.content.result_data[i]);
            }
        }

        Ok(result)
    }

    /// Extract event message.
    pub fn to_event(&self) -> Result<EventMessage> {
        let mut event = EventMessage::default();

        event.has_task_id = self.has_task_id;
        if self.has_task_id {
            event.task_id = self.task_id.clone();
        }

        event.plugin_id = self.content.event_plugin_id.clone();
        event.event_type = self.content.event_type;
        event.error_message = self.content.event_error_message.clone();
        event.progress_percent = self.content.event_progress_percent;
        event.progress_message = self.content.event_progress_message.clone();
        event.success = self.content.event_success;

        Ok(event)
    }

    /// Extract command message.
    pub fn to_command(&self) -> Result<CommandMessage> {
        let mut command = CommandMessage::default();

        command.command_type = self.content.command_type;
        command.custom_command = self.content.command_custom.clone();
        command.param_count = self.content.command_param_count;

        // Copy parameters
        for i in 0..self.content.command_param_count as usize {
            if i < 16 {
                command.param_keys[i] = self.content.command_param_keys[i].clone();
                command.param_values[i] = self.content.command_param_values[i].clone();
            }
        }

        Ok(command)
    }
}

/// Union of all possible message contents
#[derive(Debug, Default)]
#[repr(C)]
pub struct MessageContent {
    // Fields for TaskMessage
    pub task_data_size: u32,
    pub task_data: FixedSizeVec<u8, 256>, // 1MB
    pub task_priority: u8,
    pub task_timeout_ms: u64,

    // Fields for ResultMessage
    pub result_plugin_id: FixedSizeByteString<64>,
    pub result_success: bool,
    pub result_has_error: bool,
    pub result_error_message: FixedSizeByteString<256>,
    pub result_data_size: u32,
    pub result_data: FixedSizeVec<u8, 256>, // 1MB

    // Fields for EventMessage
    pub event_plugin_id: FixedSizeByteString<64>,
    pub event_type: EventType,
    pub event_error_message: FixedSizeByteString<256>, // For Failed event
    pub event_progress_percent: u8,                    // For Progress event
    pub event_progress_message: FixedSizeByteString<256>, // For Progress event
    pub event_success: bool,                           // For Complete event

    // Fields for CommandMessage
    pub command_type: CommandType,
    pub command_custom: FixedSizeByteString<64>, // For Custom command
    pub command_param_count: u32,
    pub command_param_keys: [FixedSizeByteString<64>; 16],
    pub command_param_values: [FixedSizeByteString<256>; 16],
}

/// Analysis task message.
#[derive(Debug, Default)]
#[repr(C)]
pub struct TaskMessage {
    /// Unique task ID.
    pub task_id: FixedSizeByteString<64>,

    /// Data to analyze.
    pub data_size: u32,
    pub data: FixedSizeVec<u8, 256>, // 1MB

    /// Task priority (higher = more important).
    pub priority: u8,

    /// Task timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Analysis result message.
#[derive(Debug, Default)]
#[repr(C)]
pub struct ResultMessage {
    /// Task ID this result is for.
    pub task_id: FixedSizeByteString<64>,

    /// Plugin that produced this result.
    pub plugin_id: FixedSizeByteString<64>,

    /// Was the analysis successful?
    pub success: bool,

    /// Error message if not successful.
    pub has_error: bool,
    pub error_message: FixedSizeByteString<256>,

    /// Result data.
    pub data_size: u32,
    pub data: FixedSizeVec<u8, 256>, // 1MB
}

/// Event message.
#[derive(Debug, Default)]
#[repr(C)]
pub struct EventMessage {
    /// Task ID this event is for (if applicable).
    pub has_task_id: bool,
    pub task_id: FixedSizeByteString<64>,

    /// Plugin that generated this event.
    pub plugin_id: FixedSizeByteString<64>,

    /// Event type.
    pub event_type: EventType,

    /// Error message (for Failed events)
    pub error_message: FixedSizeByteString<256>,

    /// Progress percentage (for Progress events)
    pub progress_percent: u8,

    /// Progress message (for Progress events)
    pub progress_message: FixedSizeByteString<256>,

    /// Success flag (for Complete events)
    pub success: bool,
}

/// Command message.
#[derive(Debug, Default)]
#[repr(C)]
pub struct CommandMessage {
    /// Command type.
    pub command_type: CommandType,

    /// Custom command (if type is Custom)
    pub custom_command: FixedSizeByteString<64>,

    /// Parameter count
    pub param_count: u32,

    /// Parameter keys (limited to 16)
    pub param_keys: [FixedSizeByteString<64>; 16],

    /// Parameter values (limited to 16)
    pub param_values: [FixedSizeByteString<256>; 16],
}

/// Channel message wrapper - this stays as a Rust enum since it's not sent across processes
#[derive(Debug)]
pub enum ChannelMessage {
    /// Task to be processed.
    Task(TaskMessage),

    /// Analysis result.
    Result(ResultMessage),

    /// Event notification.
    Event(EventMessage),

    /// Command for a plugin.
    Command(CommandMessage),

    /// Plugin registration.
    Registration(String), // Plugin ID

    /// Heartbeat/keepalive message.
    Heartbeat,
}

/// Communication channel trait.
///
/// This trait defines methods for sending and receiving messages
/// between plugins and the core system. Since iceoryx2 is not Send+Sync,
/// we can't use async_trait directly. Instead, we need to work with channels.
pub trait CommunicationChannel {
    /// Send a message through the channel.
    fn send_message(&self, message: ChannelMessage, plugin_id: Option<&str>) -> Result<()>;

    /// Try to receive a message from the channel.
    /// Returns None if no message is available.
    fn receive_message(&self) -> Result<Option<ChannelMessage>>;

    /// Close the channel.
    fn close(&self) -> Result<()>;

    /// Check if the channel is open.
    fn is_initialized(&self) -> bool;

    /// Get the channel ID.
    fn id(&self) -> &str;
}
