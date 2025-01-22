use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyModule};
use crate::plugin::{Plugin, PluginRequirements};
use anyhow::{anyhow, Result};
use std::collections::HashSet;

pub struct PythonPlugin {
    py_plugin: Py<PyModule>,
}

impl PythonPlugin {
    pub fn new(module_name: &str) -> Result<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let py_plugin = PyModule::import(py, module_name)?;
        Ok(Self {
            py_plugin: py_plugin.into(),
        })
    }
}

impl Plugin for PythonPlugin {
    fn requirements(&self) -> PluginRequirements {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let plugin_type: String = self
            .py_plugin
            .as_ref(py)
            .getattr("plugin_type")
            .unwrap()
            .extract()
            .unwrap();

        let execution_mode: String = self
            .py_plugin
            .as_ref(py)
            .getattr("execution_mode")
            .unwrap()
            .extract()
            .unwrap();

        let required_plugins: HashSet<String> = self
            .py_plugin
            .as_ref(py)
            .getattr("required_plugins")
            .unwrap()
            .extract()
            .unwrap();

        let incompatible_plugins: HashSet<String> = self
            .py_plugin
            .as_ref(py)
            .getattr("incompatible_plugins")
            .unwrap()
            .extract()
            .unwrap();

        PluginRequirements {
            plugin_type: plugin_type.parse().unwrap(),
            execution_mode: execution_mode.parse().unwrap(),
            required_plugins,
            incompatible_plugins,
        }
    }

    fn process(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let result: Vec<u8> = self
            .py_plugin
            .as_ref(py)
            .call_method1("process", (PyBytes::new(py, data),))?
            .extract()?;

        Ok(result)
    }
}

/*#[test]
fn test_python_plugin_runtime_with_real_plugin() {
    let py_plugin = PythonPlugin::new("example_plugin").unwrap();
    let mut runtime = PluginRuntime::new(py_plugin, "python-plugin-id".to_string()).unwrap();

    runtime.communication = PluginCommunication::mock(vec![b"test".to_vec()]);

    let result = runtime.run();
    assert!(result.is_ok());
}
#[test]
fn test_node_plugin_runtime_with_real_plugin() {
    // Load the Node.js plugin
    let node_plugin = NodePlugin::new("./example_plugin.js").unwrap();
    let mut runtime = PluginRuntime::new(node_plugin, "node-plugin-id".to_string()).unwrap();

    // Provide test input
    runtime.communication = PluginCommunication::mock(vec![b"hello from Rust".to_vec()]);

    let result = runtime.run();
    assert!(result.is_ok());
}*/