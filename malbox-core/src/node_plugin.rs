use crate::plugin::{Plugin, PluginRequirements};
use anyhow::Result;
use std::collections::HashSet;
use napi::bindgen_prelude::*;
use napi::Env;
use std::process::Command;

pub struct NodePlugin {
    plugin_path: String,
}

impl NodePlugin {
    pub fn new(plugin_path: &str) -> Result<Self> {
        Ok(Self {
            plugin_path: plugin_path.to_string(),
        })
    }
}

impl Plugin for NodePlugin {
    fn requirements(&self) -> PluginRequirements {
        // Dummy requirements for example
        PluginRequirements {
            plugin_type: "ExamplePlugin".parse().unwrap(),
            execution_mode: "Synchronous".parse().unwrap(),
            required_plugins: HashSet::new(),
            incompatible_plugins: HashSet::new(),
        }
    }

    fn process(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let output = Command::new("node")
            .arg(&self.plugin_path)
            .arg(std::str::from_utf8(data)?)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Node.js plugin failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(output.stdout)
    }
}
