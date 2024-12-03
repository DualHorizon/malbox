use crate::communication::PluginCommunication;
use std::collections::HashSet;

use super::types::{ExecutionMode, PluginType};

#[derive(Debug, Clone)]
pub struct PluginRequirements {
    pub plugin_type: PluginType,
    pub execution_mode: ExecutionMode,
    pub required_plugins: HashSet<String>,
    pub incompatible_plugins: HashSet<String>,
}

pub trait Plugin: Send {
    fn requirements(&self) -> PluginRequirements;

    fn init(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    fn process(&mut self, data: &[u8]) -> Result<Vec<u8>, anyhow::Error>;

    fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

pub struct PluginRuntime<P: Plugin> {
    plugin: P,
    communication: PluginCommunication,
}

impl<P: Plugin> PluginRuntime<P> {
    pub fn new(plugin: P) -> Result<Self, Box<dyn std::error::Error>> {
        let node = NodeBuilder::new().create()?;
        let communication = PluginCommunication::new(&node)?;

        Ok(Self {
            plugin,
            communication,
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize
        self.plugin.init()?;
        self.communication.notify_event(PluginEvent::Started(
            self.plugin.requirements().plugin_type.to_string(),
        ))?;

        // Main loop
        while let Some(data) = self.communication.receive_data()? {
            match self.plugin.process(&data) {
                Ok(result) => self.communication.send_data(&result)?,
                Err(e) => {
                    self.communication.notify_event(PluginEvent::Failed(
                        self.plugin.requirements().plugin_type.to_string(),
                        e.to_string(),
                    ))?;
                    return Err(e);
                }
            }
        }

        // Cleanup
        self.plugin.shutdown()?;
        self.communication.notify_event(PluginEvent::Completed(
            self.plugin.requirements().plugin_type.to_string(),
        ))?;

        Ok(())
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty) => {
        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let plugin = <$plugin>::default();
            let mut runtime = $crate::plugin::PluginRuntime::new(plugin)?;
            runtime.run()
        }
    };
}
