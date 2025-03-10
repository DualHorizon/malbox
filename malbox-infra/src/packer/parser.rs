// FIXME:
// If PACKER_LOG is set to 1, not all
// lines will be displayed as machine-readable.
// Therefore causing issues in our parser.
// We should have a stub function to check if lines are in machine-readable format.
// And according to that, adapt our parsing.

use tracing::{debug, error, info, warn};

#[derive(Debug)]
pub enum PackerEventType {
    Error(String),
    UI {
        ui_type: String,
        message: String,
    },
    Artifact {
        builder: String,
        artifact_type: String,
        detail: String,
    },
    ErrorCount(u32),
    BuildStart(String),
    BuildEnd {
        builder: String,
        duration: Option<String>,
    },
    Other {
        event_type: String,
        data: Vec<String>,
    },
}

#[derive(Debug)]
pub struct PackerEvent {
    pub timestamp: String,
    pub target: String,
    pub event: PackerEventType,
}

#[derive(Default)]
pub struct PackerBuildState {
    pub errors: Vec<String>,
    pub artifacts: Vec<String>,
    pub error_count: u32,
    pub build_duration: Option<String>,
}

impl PackerBuildState {
    pub fn add_event(&mut self, event: &PackerEvent) {
        match &event.event {
            PackerEventType::Error(msg) => {
                self.errors.push(msg.clone());
            }
            PackerEventType::UI { ui_type, message } => {
                if ui_type == "error" {
                    self.errors.push(message.clone());
                }
            }
            PackerEventType::ErrorCount(count) => {
                self.error_count = *count;
            }
            PackerEventType::Artifact {
                builder: _,
                artifact_type,
                detail,
            } => {
                self.artifacts
                    .push(format!("{}: {}", artifact_type, detail));
            }
            PackerEventType::BuildEnd {
                builder: _,
                duration,
            } => {
                if let Some(dur) = duration {
                    self.build_duration = Some(dur.clone());
                }
            }
            _ => {}
        }
    }
}

pub fn parse_packer_event(line: &str) -> Option<PackerEvent> {
    let parts: Vec<&str> = line.split(',').collect();
    if parts.len() < 3 {
        return None;
    }

    let timestamp = parts[0].to_string();
    let target = parts[1].to_string();
    let event_type = parts[2];

    let event = match event_type {
        "error" => {
            if parts.len() >= 4 {
                PackerEventType::Error(parts[3].to_string())
            } else {
                PackerEventType::Error("Unknown error".to_string())
            }
        }
        "ui" => {
            if parts.len() >= 5 {
                PackerEventType::UI {
                    ui_type: parts[3].to_string(),
                    message: parts[4].to_string(),
                }
            } else {
                PackerEventType::UI {
                    ui_type: "unknown".to_string(),
                    message: "Incomplete UI message".to_string(),
                }
            }
        }
        "error-count" => {
            if parts.len() >= 4 {
                if let Ok(count) = parts[3].parse::<u32>() {
                    PackerEventType::ErrorCount(count)
                } else {
                    PackerEventType::ErrorCount(0)
                }
            } else {
                PackerEventType::ErrorCount(0)
            }
        }
        "artifact" => {
            if parts.len() >= 5 {
                PackerEventType::Artifact {
                    builder: target.clone(),
                    artifact_type: parts[3].to_string(),
                    detail: parts[4].to_string(),
                }
            } else {
                PackerEventType::Other {
                    event_type: "artifact".to_string(),
                    data: parts.iter().skip(3).map(|s| s.to_string()).collect(),
                }
            }
        }
        "build-start" => {
            if parts.len() >= 4 {
                PackerEventType::BuildStart(parts[3].to_string())
            } else {
                PackerEventType::BuildStart("".to_string())
            }
        }
        "build-end" => {
            let duration = if parts.len() >= 4 {
                Some(parts[3].to_string())
            } else {
                None
            };

            PackerEventType::BuildEnd {
                builder: target.clone(),
                duration,
            }
        }
        _ => PackerEventType::Other {
            event_type: event_type.to_string(),
            data: parts.iter().skip(3).map(|s| s.to_string()).collect(),
        },
    };

    Some(PackerEvent {
        timestamp,
        target,
        event,
    })
}

pub fn log_packer_event(event: &PackerEvent) {
    match &event.event {
        PackerEventType::Error(msg) => {
            error!("[PACKER ERROR] {}", msg);
        }
        PackerEventType::UI { ui_type, message } => match ui_type.as_str() {
            "error" => error!("[PACKER ERROR] {}", message),
            "warning" => warn!("[PACKER WARNING] {}", message),
            "say" => {
                if message.starts_with("==>") {
                    info!("[PACKER] {}", message);
                } else {
                    debug!("[PACKER] {}", message);
                }
            }
            "message" => debug!("[PACKER MESSAGE] {}", message),
            _ => debug!("[PACKER UI] {}: {}", ui_type, message),
        },
        PackerEventType::Artifact {
            builder,
            artifact_type,
            detail,
        } => {
            info!(
                "[PACKER ARTIFACT] Builder '{}' created {} artifact: {}",
                builder, artifact_type, detail
            );
        }
        PackerEventType::ErrorCount(count) => {
            if *count > 0 {
                error!("[PACKER] Found {} errors", count);
            }
        }
        PackerEventType::BuildStart(builder) => {
            info!("[PACKER] Build started for {}", builder);
        }
        PackerEventType::BuildEnd { builder, duration } => {
            if let Some(dur) = duration {
                info!("[PACKER] Build finished for {} after {}", builder, dur);
            } else {
                info!("[PACKER] Build finished for {}", builder);
            }
        }
        PackerEventType::Other { event_type, data } => {
            debug!("[PACKER EVENT] Type: {}, Data: {:?}", event_type, data);
        }
    }
}
