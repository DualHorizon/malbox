use serde::Deserialize;

// NOTE: Most of the String params here should be replaced
// with proper, specific values, this is TBD

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
pub enum PluginType {
    Storage,
    Analysis,
    Machinery,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Storage => write!(f, "Storage"),
            PluginType::Analysis => write!(f, "Analysis"),
            PluginType::Machinery => write!(f, "Machinery"),
        }
    }
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
    ResourceReady { id: String, plugin_type: PluginType },
    Started(String),
    Shutdown(String),
    Failed(String, String),
}
