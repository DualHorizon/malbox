pub mod state;

use super::communication::PluginCommunication;
use super::plugin::PluginRequirements;
use iceoryx2::prelude::*;
use state::PluginState;
use std::time::Duration;

pub struct PluginManager {
    node: Node<ipc::Service>,
    state: PluginState,
    communication: PluginCommunication,
}

impl PluginManager {
    pub fn new() -> Result<Self, anyhow::Error> {
        let node = NodeBuilder::new().create()?;

        Ok(Self {
            communication: PluginCommunication::new(&node)?,
            state: PluginState::new(),
            node,
        })
    }

    // TODO: replace `id` with an actual ID, not a string.
    // We should consider adding checks here. Or in the latters.
    pub fn register_plugin(&mut self, id: String, requirements: PluginRequirements) {
        self.state.add_plugin(id, requirements);
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
