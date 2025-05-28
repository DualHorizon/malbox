//! Message types and payload definitions for IPC communication.

use iceoryx2_bb_container::{byte_string::FixedSizeByteString, vec::FixedSizeVec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{CommunicationError, Result};

/// Message type discriminant for zero-copy IPC.
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

/// Event types for plugin notifications.
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

/// Command types for plugin control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(C)]
pub enum CommandType {
    #[default]
    Stop = 0,
    Pause = 1,
    Resume = 2,
    Status = 3,
}

/// Zero-copy message payload for IPC.
#[derive(Debug)]
#[repr(C)]
pub struct MessagePayload {
    pub message_type: MessageType,
    pub message_id: FixedSizeByteString<64>,
    pub sender_id: FixedSizeByteString<64>,
    pub recipient_id: FixedSizeByteString<64>,
    pub has_task_id: bool,
    pub task_id: FixedSizeByteString<64>,
    pub content: MessageContent,
}

impl MessagePayload {
    pub fn new(message_type: MessageType, sender_id: &str, recipient_id: &str) -> Result<Self> {
        Ok(Self {
            message_type,
            message_id: FixedSizeByteString::from_bytes(Uuid::new_v4().to_string().as_bytes())
                .map_err(|e| {
                    CommunicationError::SerializationError(format!("Message ID: {}", e))
                })?,
            sender_id: FixedSizeByteString::from_bytes(sender_id.as_bytes())
                .map_err(|e| CommunicationError::SerializationError(format!("Sender ID: {}", e)))?,
            recipient_id: FixedSizeByteString::from_bytes(recipient_id.as_bytes()).map_err(
                |e| CommunicationError::SerializationError(format!("Recipient ID: {}", e)),
            )?,
            has_task_id: false,
            task_id: FixedSizeByteString::from_bytes("".as_bytes())
                .map_err(|e| CommunicationError::SerializationError(format!("Task ID: {}", e)))?,
            content: MessageContent::default(),
        })
    }

    pub fn with_task_id(mut self, task_id: &str) -> Result<Self> {
        self.has_task_id = true;
        self.task_id = FixedSizeByteString::from_bytes(task_id.as_bytes())
            .map_err(|e| CommunicationError::SerializationError(format!("Task ID: {}", e)))?;
        Ok(self)
    }

    pub fn with_task(mut self, task: &TaskMessage) -> Result<Self> {
        self.content.task_data_size = task.data_size;
        self.content.task_priority = task.priority;
        self.content.task_timeout_ms = task.timeout_ms;

        for (i, &byte) in task
            .data
            .iter()
            .enumerate()
            .take(self.content.task_data.capacity())
        {
            self.content.task_data.push(byte);
        }

        Ok(self)
    }

    pub fn with_result(mut self, result: &ResultMessage) -> Result<Self> {
        self.content.result_plugin_id = result.plugin_id.clone();
        self.content.result_success = result.success;
        self.content.result_has_error = result.has_error;
        self.content.result_error_message = result.error_message.clone();
        self.content.result_data_size = result.data_size;

        for (i, &byte) in result
            .data
            .iter()
            .enumerate()
            .take(self.content.result_data.capacity())
        {
            self.content.result_data.push(byte);
        }

        Ok(self)
    }

    pub fn with_event(mut self, event: &EventMessage) -> Result<Self> {
        self.content.event_plugin_id = event.plugin_id.clone();
        self.content.event_type = event.event_type;
        self.content.event_error_message = event.error_message.clone();
        self.content.event_progress_percent = event.progress_percent;
        self.content.event_progress_message = event.progress_message.clone();
        self.content.event_success = event.success;

        Ok(self)
    }

    pub fn with_command(mut self, command: &CommandMessage) -> Result<Self> {
        self.content.command_type = command.command_type;
        self.content.command_custom = command.custom_command.clone();
        self.content.command_param_count = command.param_count;

        for i in 0..command.param_count.min(16) as usize {
            self.content.command_param_keys[i] = command.param_keys[i].clone();
            self.content.command_param_values[i] = command.param_values[i].clone();
        }

        Ok(self)
    }

    pub fn to_task(&self) -> Result<TaskMessage> {
        if self.message_type != MessageType::Task {
            return Err(CommunicationError::InvalidMessageType {
                expected: MessageType::Task,
                actual: self.message_type,
            });
        }

        let mut task = TaskMessage::default();
        if self.has_task_id {
        for i in 0..self.content.command_param_count.min(16) as usize {
            command.param_keys[i] = self.content.command_param_keys[i].clone();
            command.param_values[i] = self.content.command_param_values[i].clone();
        }

        Ok(command)
    }
}

/// Union of all possible message contents for zero-copy IPC.
#[derive(Debug, Default)]
#[repr(C)]
pub struct MessageContent {
    // Task message fields
    pub task_data_size: u32,
    pub task_data: FixedSizeVec<u8, 256>,
    pub task_priority: u8,
    pub task_timeout_ms: u64,
    // Result message fields
    pub result_plugin_id: FixedSizeByteString<64>,
    pub result_success: bool,
    pub result_has_error: bool,
    pub result_error_message: FixedSizeByteString<256>,
    pub result_data_size: u32,
    pub result_data: FixedSizeVec<u8, 256>,
    // Event message fields
    pub event_plugin_id: FixedSizeByteString<64>,
    pub event_type: EventType,
    pub event_error_message: FixedSizeByteString<256>,
    pub event_progress_percent: u8,
    pub event_progress_message: FixedSizeByteString<256>,
    pub event_success: bool,
    // Command message fields
    pub command_type: CommandType,
    pub command_custom: FixedSizeByteString<64>,
    pub command_param_count: u32,
    pub command_param_keys: [FixedSizeByteString<64>; 16],
    pub command_param_values: [FixedSizeByteString<256>; 16],
}

/// Individual message types for type-safe handling.
#[derive(Debug, Default)]
#[repr(C)]
pub struct TaskMessage {
    pub task_id: FixedSizeByteString<64>,
    pub data_size: u32,
    pub data: FixedSizeVec<u8, 256>,
    pub priority: u8,
    pub timeout_ms: u64,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct ResultMessage {
    pub task_id: FixedSizeByteString<64>,
    pub plugin_id: FixedSizeByteString<64>,
    pub success: bool,
    pub has_error: bool,
    pub error_message: FixedSizeByteString<256>,
    pub data_size: u32,
    pub data: FixedSizeVec<u8, 256>,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct EventMessage {
    pub has_task_id: bool,
    pub task_id: FixedSizeByteString<64>,
    pub plugin_id: FixedSizeByteString<64>,
    pub event_type: EventType,
    pub error_message: FixedSizeByteString<256>,
    pub progress_percent: u8,
    pub progress_message: FixedSizeByteString<256>,
    pub success: bool,
}

#[derive(Debug, Default)]
#[repr(C)]
pub struct CommandMessage {
    pub command_type: CommandType,
    pub custom_command: FixedSizeByteString<64>,
    pub param_count: u32,
    pub param_keys: [FixedSizeByteString<64>; 16],
    pub param_values: [FixedSizeByteString<256>; 16],
}

/// High-level channel message wrapper.
#[derive(Debug)]
pub enum ChannelMessage {
    Task(TaskMessage),
    Result(ResultMessage),
    Event(EventMessage),
    Command(CommandMessage),
    Registration(String),
    Heartbeat,
}
