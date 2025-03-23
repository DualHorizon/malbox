use crate::error::{Error, Result};
use crate::parser::hcl_custom;
use hcl::{Block, Body, Structure};
use std::collections::HashMap;

pub fn parse_config(content: &str) -> Result<Body> {
    hcl_custom::parse(content)
}

pub fn parse_variables_hcl(content: &str) -> Result<HashMap<String, String>> {
    let body = hcl::parse(content)?;
    let mut variables = HashMap::new();

    for entry in body.iter() {
        if let Structure::Attribute(attr) = entry {
            variables.insert(attr.key().to_string(), attr.expr().to_string());
        }
    }

    Ok(variables)
}

pub fn parse_variables(content: &str) -> Result<HashMap<String, String>> {
    match parse_variables_hcl(content) {
        Ok(variables) if !variables.is_empty() => return Ok(variables),
        _ => {
            tracing::debug!("Terraform parsing fallback");
            let mut variables = HashMap::new();

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                    continue;
                }

                if let Some(pos) = line.find('=') {
                    let key = line[..pos].trim().to_string();
                    let mut value = line[pos + 1..].trim().to_string();

                    if value.starts_with('"') && value.ends_with('"') {
                        value = value[1..value.len() - 1].to_string();
                    }

                    variables.insert(key, value);
                }
            }

            Ok(variables)
        }
    }
}

pub fn extract_resources(body: &Body) -> Vec<(String, String, HashMap<String, String>)> {
    let mut resources = Vec::new();

    for structure in body.iter() {
        if let Structure::Block(block) = structure {
            if block.identifier() == "resource" {
                if let (Some(resource_type), Some(resource_name)) =
                    (block.labels().get(0), block.labels().get(1))
                {
                    let mut attributes = HashMap::new();

                    for attr in block.body().attributes() {
                        attributes.insert(attr.key().to_string(), attr.expr().to_string());
                    }

                    resources.push((
                        resource_type.as_str().to_string(),
                        resource_name.as_str().to_string(),
                        attributes,
                    ));
                }
            }
        }
    }

    resources
}

pub fn extract_providers(body: &Body) -> Vec<(String, HashMap<String, String>)> {
    let mut providers = Vec::new();

    for structure in body.iter() {
        if let Structure::Block(block) = structure {
            if block.identifier() == "provider" {
                if let Some(provider_name) = block.labels().first() {
                    let mut attributes = HashMap::new();

                    for attr in block.body().attributes() {
                        attributes.insert(attr.key().to_string(), attr.expr().to_string());
                    }

                    providers.push((provider_name.as_str().to_string(), attributes));
                }
            }
        }
    }

    providers
}

pub fn extract_modules(body: &Body) -> Vec<(String, HashMap<String, String>)> {
    let mut modules = Vec::new();

    for structure in body.iter() {
        if let Structure::Block(block) = structure {
            if block.identifier() == "module" {
                if let Some(module_name) = block.labels().first() {
                    let mut attributes = HashMap::new();

                    for attr in block.body().attributes() {
                        attributes.insert(attr.key().to_string(), attr.expr().to_string());
                    }

                    modules.push((module_name.as_str().to_string(), attributes));
                }
            }
        }
    }

    modules
}

pub fn find_vm_resources(body: &Body) -> Vec<(String, HashMap<String, String>)> {
    let vm_resource_types = [
        "aws_instance",
        "azurerm_virtual_machine",
        "google_compute_instance",
        "vsphere_virtual_machine",
        "libvirt_domain",
        "digitalocean_droplet",
    ];

    let resources = extract_resources(body);

    resources
        .into_iter()
        .filter(|(resource_type, _, _)| vm_resource_types.contains(&resource_type.as_str()))
        .map(|(_, name, attributes)| (name, attributes))
        .collect()
}

pub fn parse_state_output(output: &str) -> HashMap<String, String> {
    let mut results = HashMap::new();

    for line in output.lines() {
        let line = line.trim();

        if line.contains(" = ") {
            let parts: Vec<&str> = line.splitn(2, " = ").collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim().trim_matches('"');
                results.insert(key.to_string(), value.to_string());
            }
        }
    }

    results
}

pub fn parse_vm_instance(state_output: &str) -> Option<(String, String)> {
    let mut vm_id = None;
    let mut vm_ip = None;

    for line in state_output.lines() {
        let line = line.trim();

        if line.contains("id = ") {
            let parts: Vec<&str> = line.splitn(2, " = ").collect();
            if parts.len() == 2 {
                vm_id = Some(parts[1].trim().trim_matches('"').to_string());
            }
        } else if line.contains("private_ip = ") || line.contains("ip_address = ") {
            let parts: Vec<&str> = line.splitn(2, " = ").collect();
            if parts.len() == 2 {
                vm_ip = Some(parts[1].trim().trim_matches('"').to_string());
            }
        }
    }

    if let (Some(id), Some(ip)) = (vm_id, vm_ip) {
        Some((id, ip))
    } else {
        None
    }
}
