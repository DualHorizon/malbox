use crate::error::{Result, TerraformError};
use malbox_hcl_utils::{
    extractor::{extract_string, HclExtractor},
    parser::{parse, parse_document, HclParser},
    types::{HclBlock, HclDocument, HclValue},
};
use std::collections::HashMap;
use tracing::debug;

/// Parse Terraform configuration from HCL content.
pub fn parse_config(content: &str) -> Result<HclDocument> {
    parse_document(content).map_err(|e| TerraformError::HclParsing {
        message: "Failed to parse Terraform configuration".to_string(),
        source: e,
    })
}

/// Parse Terraform variables from tfvars or HCL content.
pub fn parse_variables(content: &str) -> Result<HashMap<String, String>> {
    // Try parsing as HCL first
    match parse_hcl_variables(content) {
        Ok(vars) if !vars.is_empty() => Ok(vars),
        _ => {
            debug!("Falling back to simple tfvars parsing");
            parse_tfvars_simple(content)
        }
    }
}

/// Parse variables from HCL format.
fn parse_hcl_variables(content: &str) -> Result<HashMap<String, String>> {
    let doc = parse_document(content).map_err(|e| TerraformError::HclParsing {
        message: "Failed to parse HCL variables".to_string(),
        source: e,
    })?;

    let mut variables = HashMap::new();

    // Extract root-level attributes
    for (key, value) in &doc.attributes {
        if let Some(string_value) = value_to_string(value) {
            variables.insert(key.clone(), string_value);
        }
    }

    // Extract variable blocks
    for block in &doc.blocks {
        if block.identifier == "variable" && !block.labels.is_empty() {
            let var_name = &block.labels[0];
            if let Some(default_value) = block.attributes.get("default") {
                if let Some(string_value) = value_to_string(default_value) {
                    variables.insert(var_name.clone(), string_value);
                }
            }
        }
    }

    Ok(variables)
}

/// Parse simple tfvars format (key = value).
fn parse_tfvars_simple(content: &str) -> Result<HashMap<String, String>> {
    let mut variables = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }

        // Parse key = value format
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            variables.insert(key, value);
        }
    }

    Ok(variables)
}

/// Convert HclValue to string representation.
fn value_to_string(value: &HclValue) -> Option<String> {
    match value {
        HclValue::String(s) => Some(s.clone()),
        HclValue::Number(n) => Some(n.to_string()),
        HclValue::Bool(b) => Some(b.to_string()),
        HclValue::Array(arr) => {
            let strings: Vec<String> = arr.iter().filter_map(value_to_string).collect();
            Some(format!("[{}]", strings.join(", ")))
        }
        _ => None,
    }
}

/// Resource information extracted from Terraform configuration.
#[derive(Debug, Clone)]
pub struct TerraformResource {
    pub resource_type: String,
    pub resource_name: String,
    pub attributes: HashMap<String, String>,
}

/// Extract all resources from a Terraform configuration.
pub fn extract_resources(content: &str) -> Result<Vec<TerraformResource>> {
    let doc = parse_config(content)?;
    let mut resources = Vec::new();

    for block in &doc.blocks {
        if block.identifier == "resource" && block.labels.len() >= 2 {
            let resource = TerraformResource {
                resource_type: block.labels[0].clone(),
                resource_name: block.labels[1].clone(),
                attributes: extract_attributes(&block.attributes),
            };
            resources.push(resource);
        }
    }

    Ok(resources)
}

/// Provider information extracted from Terraform configuration.
#[derive(Debug, Clone)]
pub struct TerraformProvider {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

/// Extract all providers from a Terraform configuration.
pub fn extract_providers(content: &str) -> Result<Vec<TerraformProvider>> {
    let doc = parse_config(content)?;
    let mut providers = Vec::new();

    for block in &doc.blocks {
        if block.identifier == "provider" && !block.labels.is_empty() {
            let provider = TerraformProvider {
                name: block.labels[0].clone(),
                attributes: extract_attributes(&block.attributes),
            };
            providers.push(provider);
        }
    }

    Ok(providers)
}

/// Module information extracted from Terraform configuration.
#[derive(Debug, Clone)]
pub struct TerraformModule {
    pub name: String,
    pub attributes: HashMap<String, String>,
}

/// Extract all modules from a Terraform configuration.
pub fn extract_modules(content: &str) -> Result<Vec<TerraformModule>> {
    let doc = parse_config(content)?;
    let mut modules = Vec::new();

    for block in &doc.blocks {
        if block.identifier == "module" && !block.labels.is_empty() {
            let module = TerraformModule {
                name: block.labels[0].clone(),
                attributes: extract_attributes(&block.attributes),
            };
            modules.push(module);
        }
    }

    Ok(modules)
}

/// Convert HCL attributes to string map.
fn extract_attributes(attributes: &HashMap<String, HclValue>) -> HashMap<String, String> {
    attributes
        .iter()
        .filter_map(|(key, value)| value_to_string(value).map(|v| (key.clone(), v)))
        .collect()
}

/// Virtual machine resource types supported by Terraform.
const VM_RESOURCE_TYPES: &[&str] = &[
    "aws_instance",
    "azurerm_virtual_machine",
    "azurerm_linux_virtual_machine",
    "azurerm_windows_virtual_machine",
    "google_compute_instance",
    "vsphere_virtual_machine",
    "libvirt_domain",
    "digitalocean_droplet",
    "proxmox_vm_qemu",
    "hyperv_machine_instance",
];

/// Find all VM resources in a Terraform configuration.
pub fn find_vm_resources(content: &str) -> Result<Vec<TerraformResource>> {
    let resources = extract_resources(content)?;

    Ok(resources
        .into_iter()
        .filter(|r| VM_RESOURCE_TYPES.contains(&r.resource_type.as_str()))
        .collect())
}

/// State output information.
#[derive(Debug, Clone)]
pub struct StateOutput {
    pub outputs: HashMap<String, String>,
}

/// Parse Terraform state output.
pub fn parse_state_output(output: &str) -> StateOutput {
    let mut outputs = HashMap::new();

    for line in output.lines() {
        let line = line.trim();

        // Look for output format: key = "value"
        if let Some((key, value)) = parse_state_line(line) {
            outputs.insert(key, value);
        }
    }

    StateOutput { outputs }
}

/// Parse a single line from state output.
fn parse_state_line(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, " = ").collect();

    if parts.len() == 2 {
        let key = parts[0].trim();
        let value = parts[1].trim().trim_matches('"').trim_matches('\'');

        if !key.is_empty() && !value.is_empty() {
            return Some((key.to_string(), value.to_string()));
        }
    }

    None
}

/// VM instance information extracted from state.
#[derive(Debug, Clone)]
pub struct VmInstance {
    pub id: String,
    pub ip_address: String,
    pub attributes: HashMap<String, String>,
}

/// Parse VM instance information from Terraform state output.
pub fn parse_vm_instance(state_output: &str) -> Option<VmInstance> {
    let state = parse_state_output(state_output);

    // Extract ID
    let id = state.outputs.get("id").cloned()?;

    // Extract IP address (try multiple common attribute names)
    let ip_address = state
        .outputs
        .get("private_ip")
        .or_else(|| state.outputs.get("ip_address"))
        .or_else(|| state.outputs.get("public_ip"))
        .or_else(|| state.outputs.get("network_interface.0.ip_address"))
        .cloned()?;

    Some(VmInstance {
        id,
        ip_address,
        attributes: state.outputs,
    })
}

/// Extract specific VM attributes from state output.
pub fn extract_vm_attributes(state_output: &str) -> HashMap<String, String> {
    let state = parse_state_output(state_output);

    let mut vm_attrs = HashMap::new();

    // Common VM attributes to extract
    let vm_keys = [
        "id",
        "name",
        "private_ip",
        "public_ip",
        "ip_address",
        "status",
        "state",
        "cpu",
        "memory",
        "disk_size",
        "network_interface.0.ip_address",
        "network_interface.0.mac_address",
        "guest_id",
        "hostname",
        "domain",
    ];

    for key in &vm_keys {
        if let Some(value) = state.outputs.get(*key) {
            vm_attrs.insert(key.to_string(), value.clone());
        }
    }

    vm_attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tfvars_simple() {
        let content = r#"
            # Comment
            vm_name = "test-vm"
            cpu_count = 4
            memory = "8192"
        "#;

        let vars = parse_tfvars_simple(content).unwrap();
        assert_eq!(vars.get("vm_name"), Some(&"test-vm".to_string()));
        assert_eq!(vars.get("cpu_count"), Some(&"4".to_string()));
        assert_eq!(vars.get("memory"), Some(&"8192".to_string()));
    }

    #[test]
    fn test_parse_state_output() {
        let output = r#"
            id = "vm-12345"
            private_ip = "10.0.0.10"
            status = "running"
        "#;

        let state = parse_state_output(output);
        assert_eq!(state.outputs.get("id"), Some(&"vm-12345".to_string()));
        assert_eq!(
            state.outputs.get("private_ip"),
            Some(&"10.0.0.10".to_string())
        );
        assert_eq!(state.outputs.get("status"), Some(&"running".to_string()));
    }

    #[test]
    fn test_parse_vm_instance() {
        let output = r#"
            id = "vm-12345"
            private_ip = "10.0.0.10"
            cpu = "4"
            memory = "8192"
        "#;

        let vm = parse_vm_instance(output).unwrap();
        assert_eq!(vm.id, "vm-12345");
        assert_eq!(vm.ip_address, "10.0.0.10");
        assert_eq!(vm.attributes.len(), 4);
    }
}
