use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use stabby::boxed::Box;
use stabby::string::String as StabbyString;

pub struct PluginRegistry {
    plugins: RwLock<HashMap<StabbyString, Arc<dyn PluginApi>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
        }
    }

    pub fn register(&self, name: StabbyString, plugin: Arc<dyn PluginApi>) {
        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(name, plugin);
    }

    pub fn get(&self, name: &StabbyString) -> Option<Arc<dyn PluginApi>> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(&name.cloned())
    }
}

pub trait PluginApi: Send + Sync {
    fn get_version(&self) -> StabbyString;
}
