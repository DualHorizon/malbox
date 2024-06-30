use std::{fs, path::Path};

use serde::Deserialize;
use virtualbox::VirtualBoxConfig;
use vmware::VmwareConfig;

mod virtualbox;
mod vmware;

#[derive(Debug, Deserialize)]
pub enum MachineryConfig {
    Vmware(VmwareConfig),
    VirtualBox(VirtualBoxConfig),
}

pub fn load_config(hypervisor: &str) -> Result<MachineryConfig, String> {
    let specific_path = format!("../configuration/machinery/{}.toml", hypervisor);
    let default_path = format!(
        "../configuration/machinery/default/{}.default.toml",
        hypervisor
    );

    let config_path = if Path::new(&specific_path).exists() {
        specific_path
    } else if Path::new(&default_path).exists() {
        default_path
    } else {
        return Err(format!(
            "No configuration file found for hypervisor `{}`",
            hypervisor
        ));
    };

    let contents = fs::read_to_string(&config_path)
        .map_err(|e| format!("Could not read file `{}`: {}", config_path, e))?;

    match hypervisor {
        "vmware" => {
            let config: VmwareConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse Vmware config: {}", e))?;
            Ok(MachineryConfig::Vmware(config))
        }
        "virtualbox" => {
            let config: VirtualBoxConfig = toml::from_str(&contents)
                .map_err(|e| format!("Failed to parse VirtualBox config: {}", e))?;
            Ok(MachineryConfig::VirtualBox(config))
        }
        _ => Err(format!("Unsupported hypervisor: {}", hypervisor)),
    }
}
