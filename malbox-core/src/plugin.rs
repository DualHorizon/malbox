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

    fn is_dynamic(&self) -> bool {
        false
    }
}
pub struct PluginRuntime<P: Plugin> {
    plugin: P,
    communication: PluginCommunication,
}

impl<P: Plugin> PluginRuntime<P> {
    pub fn new(plugin: P, id: String) -> Result<Self> {
        // Create the communication node for the plugin runtime
        let node = NodeBuilder::new().create()
            .map_err(|e| anyhow::anyhow!("Failed to create Node: {}", e))?;
        
        // Retrieve plugin requirements
        let requirements = plugin.requirements();
        
        // Set up communication with the plugin
        let communication = PluginCommunication::new(&node, requirements.plugin_type.clone(), id)
            .map_err(|e| anyhow::anyhow!("Failed to initialize PluginCommunication: {}", e))?;

        log::info!("PluginRuntime created for plugin ID: {}", id);

        Ok(Self {
            plugin,
            communication,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        log::info!(
            "Starting PluginRuntime for plugin ID: {}",
            self.communication.plugin_id
        );

        // Step 1: Initialize the plugin
        self.plugin
            .init()
            .map_err(|e| anyhow::anyhow!("Plugin initialization failed: {}", e))?;
        
        log::info!(
            "Plugin initialized and resources are ready for plugin ID: {}",
            self.communication.plugin_id
        );

        // Announce resources ready
        self.communication.notify_event(PluginEvent::ResourceReady {
            id: self.communication.plugin_id.clone(),
            plugin_type: self.communication.plugin_type.clone(),
        })?;

        // Step 2: Main processing loop
        while let Some(sample) = self.communication.receive_data()? {
            log::info!(
                "Processing started for plugin ID: {}",
                self.communication.plugin_id
            );

            // Notify that processing has started
            self.communication.notify_event(PluginEvent::Started(
                self.communication.plugin_id.clone(),
            ))?;

            // Process the data
            match self.plugin.process(sample.payload().data.as_slice()) {
                Ok(result) => {
                    log::info!(
                        "Processing successful for plugin ID: {}, sending result...",
                        self.communication.plugin_id
                    );

                    self.communication
                        .send_result(result)
                        .map_err(|e| anyhow::anyhow!("Failed to send result: {}", e))?;
                }
                Err(e) => {
                    log::error!(
                        "Processing failed for plugin ID: {}: {}",
                        self.communication.plugin_id,
                        e
                    );

                    // Notify failure
                    self.communication.notify_event(PluginEvent::Failed(
                        self.communication.plugin_id.clone(),
                        e.to_string(),
                    ))?;

                    return Err(anyhow::anyhow!(
                        "Plugin processing error for plugin ID {}: {}",
                        self.communication.plugin_id,
                        e
                    ));
                }
            }
        }

        // Step 3: Graceful shutdown
        log::info!(
            "Shutting down PluginRuntime for plugin ID: {}",
            self.communication.plugin_id
        );

        self.plugin
            .shutdown()
            .map_err(|e| anyhow::anyhow!("Plugin shutdown failed: {}", e))?;

        self.communication.notify_event(PluginEvent::Shutdown(
            self.communication.plugin_id.clone(),
        ))?;

        log::info!(
            "PluginRuntime shutdown complete for plugin ID: {}",
            self.communication.plugin_id
        );

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
