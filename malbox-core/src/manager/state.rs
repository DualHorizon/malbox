use crate::{
    plugin::PluginRequirements,
    types::{ExecutionMode, PluginEvent, PluginType},
};
use std::collections::HashMap;

pub struct PluginState {
    plugins: HashMap<String, PluginRequirements>,
    running: HashMap<String, PluginType>,
}

impl PluginState {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            running: HashMap::new(),
        }
    }

    pub fn add_plugin(&mut self, id: String, requirements: PluginRequirements) {
        self.plugins.insert(id, requirements);
    }

    pub fn get_plugin(&self, id: &str) -> Option<&PluginRequirements> {
        self.plugins.get(id)
    }

    pub fn mark_as_running(&mut self, id: &str) {
        if let Some(plugin) = self.plugins.get(id) {
            self.running
                .insert(id.to_string(), plugin.plugin_type.clone());
        }
    }

    pub fn get_next_plugins(&self) -> Vec<String> {
        let mut runnable = Vec::new();

        if let Some((id, _)) = self
            .plugins
            .iter()
            .find(|(_, req)| matches!(req.execution_mode, ExecutionMode::Exclusive))
        {
            return if self.can_start_plugin(id) {
                vec![id.clone()]
            } else {
                vec![]
            };
        }

        for (id, req) in &self.plugins {
            if !self.running.contains_key(id) && self.can_start_plugin(id) {
                match &req.execution_mode {
                    ExecutionMode::Sequential => runnable.insert(0, id.clone()),
                    ExecutionMode::Parallel(_) | ExecutionMode::Unrestricted => {
                        runnable.push(id.clone())
                    }
                    ExecutionMode::Exclusive => {}
                }
            }
        }

        runnable
    }

    fn can_start_plugin(&self, id: &str) -> bool {
        let plugin = match self.plugins.get(id) {
            Some(p) => p,
            None => return false,
        };

        if !plugin
            .required_plugins
            .iter()
            .all(|req_id| self.running.contains_key(req_id))
        {
            return false;
        }

        if plugin
            .incompatible_plugins
            .iter()
            .all(|incomp_id| self.running.contains_key(incomp_id))
        {
            return false;
        }

        match &plugin.execution_mode {
            ExecutionMode::Exclusive => self.running.is_empty(),
            ExecutionMode::Sequential => true,
            ExecutionMode::Parallel(group) => {
                self.running.iter()
                    .all(|(_, p_type)| {
                        self.plugins.iter()
                            .find(|(_, req)| req.plugin_type == *p_type)
                            .map_or(false, |(_, req)| {
                                matches!(&req.execution_mode, ExecutionMode::Parallel(other_group) if other_group == group)
                            })
                    })
            },
            ExecutionMode::Unrestricted => true,
        }
    }

    pub fn handle_event(&mut self, event: PluginEvent) {
        match event {
            PluginEvent::Started(id) => self.mark_as_running(&id),
            PluginEvent::Completed(id) | PluginEvent::Failed(id, _) => {
                self.running.remove(&id);
            }
            PluginEvent::ResourceReady(_) => {}
        }
    }
}
