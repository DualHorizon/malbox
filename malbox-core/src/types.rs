use std::collections::HashSet;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PluginType {
    Storage,
    Network,
    Scheduler,
    Analysis,
    Monitor,
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
