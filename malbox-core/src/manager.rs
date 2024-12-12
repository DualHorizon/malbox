use super::communication::PluginCommunication;
use super::plugin::PluginRequirements;
use super::registry::PluginRegistry;
use iceoryx2::prelude::*;
use state::PluginState;
use std::sync::Arc;
use std::time::Duration;

pub mod state;

pub struct PluginManager {
    node: Node<ipc::Service>,
    state: PluginState,
    communication: PluginCommunication,
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    pub fn new(registry: Arc<PluginRegistry>) -> Result<Self, anyhow::Error> {
        let node = NodeBuilder::new().create()?;

        Ok(Self {
            communication: PluginCommunication::new(&node)?,
            state: PluginState::new(),
            node,
            registry,
        })
    }

    pub fn register_plugin(
        &mut self,
        id: String,
        requirements: PluginRequirements,
    ) -> Result<(), anyhow::Error> {
        self.registry.verify_plugin(&id)?;
        self.state.add_plugin(id, requirements);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), anyhow::Error> {
        while let NodeEvent::Tick = self.node.wait(Duration::from_millis(100)) {
            self.process_cycle()?;
        }
        Ok(())
    }

    fn process_cycle(&mut self) -> Result<(), anyhow::Error> {
        self.communication.process_messages(&mut self.state)?;

        for plugin_id in self.state.get_next_plugins() {
            self.start_plugin(&plugin_id)?;
        }

        Ok(())
    }

    fn start_plugin(&mut self, id: &str) -> Result<(), anyhow::Error> {
        if let Some(plugin) = self.state.get_plugin(id) {
            self.communication.send_start_signal(plugin)?;
            self.state.mark_as_running(id);
        }
        Ok(())
    }
}
