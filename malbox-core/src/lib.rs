use anyhow::{anyhow, Context, Result};
use libloading::Library;
use malbox_abi_common::{AnalysisPluginDyn, AnalysisResult, Plugin, PluginInfo};
use petgraph::algo::toposort;
use petgraph::Graph;
use stabby::libloading::StabbyLibrary;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub struct PluginManager {
    plugins: HashMap<String, (Library, Plugin)>,
    dependency_graph: Graph<String, ()>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager {
            plugins: HashMap::new(),
            dependency_graph: Graph::new(),
        }
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<()> {
        let lib = unsafe { Library::new(path) }
            .with_context(|| format!("Failed to load library from path: {:?}", path))?;

        let init_plugin = unsafe {
            lib.get_stabbied::<extern "C" fn() -> stabby::result::Result<Plugin, stabby::string::String>>(b"init_plugin")
        }.map_err(|e| anyhow!("Failed to get init_plugin symbol: {}", e))?;

        let plugin = init_plugin().unwrap();

        let info = plugin.get_info();
        let plugin_name = info.name.to_string();

        // Add node to dependency graph
        let node_index = self.dependency_graph.add_node(plugin_name.clone());

        // Add edges for dependencies
        for dep in info.dependencies.iter() {
            let dep_name = dep.name.to_string();
            if let Some(dep_index) = self
                .dependency_graph
                .node_indices()
                .find(|&i| self.dependency_graph[i] == dep_name)
            {
                self.dependency_graph.add_edge(node_index, dep_index, ());
            } else {
                return Err(anyhow!(
                    "Dependency {} not loaded for plugin {}",
                    dep_name,
                    plugin_name
                ));
            }
        }

        self.plugins.insert(plugin_name, (lib, plugin));

        tracing::info!("{:#?}", self.plugins.keys());
        Ok(())
    }

    pub fn load_plugins_from_directory(&mut self, dir: &Path) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .map_or(false, |ext| ext == "so" || ext == "dll")
            {
                self.load_plugin(&path)?;
            }
        }
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name).map(|(_, plugin)| plugin)
    }

    pub fn execute_plugin_analysis(&self, plugin_name: &str) -> Result<AnalysisResult> {
        let plugin = self
            .get_plugin(plugin_name)
            .ok_or_else(|| anyhow!("Plugin {} not found", plugin_name))?;

        Ok(plugin.analyze().unwrap()) // handle error correctly, note that there's no map_err
                                      // implem
    }

    pub fn get_load_order(&self) -> Result<Vec<String>> {
        let order = toposort(&self.dependency_graph, None)
            .map_err(|_| anyhow!("Circular dependency detected"))?
            .into_iter()
            .map(|idx| self.dependency_graph[idx].clone())
            .collect::<Vec<String>>();
        Ok(order)
    }
}
