use serde::{Deserialize, Serialize};
use std::fmt;

mod macros;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Windows,
    Linux,
}

impl_display_fromstr!(Platform,
    Windows => "windows",
    Linux => "linux"
);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Vmware,
    VirtualBox,
    Kvm,
}

impl_display_fromstr!(Provider,
    Vmware => "vmware",
    VirtualBox => "virtualbox",
    Kvm => "kvm"
);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Staging,
    Production,
    Custom(String),
}

impl_display_fromstr!(Environment,
    Development => "development",
    Staging => "staging",
    Production => "production"
);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl_display_fromstr!(LogLevel,
    Error => "error",
    Warn => "warn",
    Info => "info",
    Debug => "debug",
    Trace => "trace"
);
