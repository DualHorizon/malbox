use crate::communication::PluginCommunication;
use crate::types::{ExecutionMode, PluginEvent, PluginType};
use anyhow::Result;
use iceoryx2::node::NodeBuilder;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct PluginRequirements {
    pub plugin_type: PluginType,
    pub execution_mode: ExecutionMode,
    pub required_plugins: HashSet<String>,
    pub incompatible_plugins: HashSet<String>,
}

pub trait Plugin: Send {
    fn requirements(&self) -> PluginRequirements;

    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn process(&mut self, data: &[u8]) -> Result<Vec<u8>>;

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct PluginRuntime<P: Plugin> {
    plugin: P,
    communication: PluginCommunication,
}

impl<P: Plugin> PluginRuntime<P> {
    pub fn new(plugin: P, id: String) -> Result<Self> {
        let node = NodeBuilder::new().create()?;
        let requirements = plugin.requirements();
        let communication = PluginCommunication::new(&node, requirements.plugin_type.clone(), id)?;

        Ok(Self {
            plugin,
            communication,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        // Initialize and prepare resources
        self.plugin.init()?;

        // Announce resources ready
        self.communication
            .notify_event(PluginEvent::ResourceReady {
                id: self.communication.plugin_id.clone(),
                plugin_type: self.communication.plugin_type.clone(),
            })?;

        // Main processing loop
        while let Some(sample) = self.communication.receive_data()? {
            // Announce processing started
            self.communication
                .notify_event(PluginEvent::Started(self.communication.plugin_id.clone()))?;

            match self.plugin.process(sample.payload().data.as_slice()) {
                Ok(result) => {
                    self.communication.send_result(result)?;
                }
                Err(e) => {
                    self.communication.notify_event(PluginEvent::Failed(
                        self.communication.plugin_id.clone(),
                        e.to_string(),
                    ))?;
                    return Err(e);
                }
            }
        }

        // Clean shutdown
        self.plugin.shutdown()?;
        self.communication
            .notify_event(PluginEvent::Shutdown(self.communication.plugin_id.clone()))?;

        Ok(())
    }
}

#[macro_export]
macro_rules! declare_plugin {
    ($plugin:ty) => {
        fn main() -> anyhow::Result<()> {
            let plugin = <$plugin>::default();
            let mut runtime = $crate::plugin::PluginRuntime::new(
                plugin,
                "plugin-id".to_string(), // Should be unique per plugin instance
            )?;
            runtime.run()
        }
    };
}
