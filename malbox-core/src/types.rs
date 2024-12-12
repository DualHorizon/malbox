use serde::Deserialize;
use std::collections::HashSet;

// NOTE: Most of the String params here should be replaced
// with proper, specific values, this is TBD

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub enum PluginType {
    Storage,
    Analysis,
    Machinery,
}

#[derive(Debug, Clone)]
pub enum ExecutionMode {
    Exclusive,
    Sequential,
    Parallel(String),
    Unrestricted,
}

#[derive(Debug, Clone)]
pub enum PluginEvent {
    Ready {
        id: String,
        plugin_type: PluginType,
        required_plugins: HashSet<String>,
    },
    Completed(String),
    Failed(String, String),
}
