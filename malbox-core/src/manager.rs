pub mod state;

use crate::communication::MasterCommunication;
use crate::plugin::PluginRequirements;
use crate::registry::PluginRegistry;
use iceoryx2::prelude::*;
use state::PluginState;
use std::sync::Arc;
use std::time::Duration;

pub struct PluginManager {
    node: Node<ipc::Service>,
    state: PluginState,
    communication: MasterCommunication,
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    pub fn new(registry: Arc<PluginRegistry>) -> anyhow::Result<Self> {
        let node = NodeBuilder::new().create()?;
        Ok(Self {
            communication: MasterCommunication::new(&node)?,
            state: PluginState::new(),
            node,
            registry,
        })
    }

    pub fn register_plugin(
        &mut self,
        id: String,
        requirements: PluginRequirements,
    ) -> anyhow::Result<()> {
        self.registry.verify_plugin(&id)?;
        self.state.add_plugin(id, requirements);
        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        while let NodeEvent::Tick = self.node.wait(Duration::from_millis(100)) {
            self.process_cycle()?;
        }
        Ok(())
    }

    fn process_cycle(&mut self) -> anyhow::Result<()> {
        // Handle plugin events
        while let Some(event) = self.communication.receive_events()? {
            self.state.handle_event(event);
        }

        // Process results
        while let Some(result) = self.communication.receive_result()? {
            println!("Received result from plugin: {:?}", result.payload());
        }

        // Start plugins that are ready
        for plugin_id in self.state.get_next_plugins() {
            self.start_plugin(&plugin_id)?;
        }

        Ok(())
    }

    fn start_plugin(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(plugin) = self.state.get_plugin(id) {
            let data = vec![]; // NOTE: TBD - Data to be processed
            self.communication
                .send_data(plugin.plugin_type.clone(), data)?;
            self.state.mark_as_running(id);
        }
        Ok(())
    }
}
