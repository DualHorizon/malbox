use anyhow::anyhow;
use libloading::Library;
use malbox_abi_common::{AnalysisPluginDyn, Plugin};
use stabby::libloading::StabbyLibrary;
use std::path::Path;

pub fn load_plugin(path: &Path) -> anyhow::Result<()> {
    unsafe {
        let lib = Library::new(path).map_err(|e| anyhow!("Failed to load library: {}", e))?;

        let init_plugin = lib
            .get_stabbied::<extern "C" fn() -> stabby::result::Result<Plugin, stabby::string::String>>(b"init_plugin")
            .map_err(|e| anyhow!("Failed to get init_plugin symbol: {}", e))?;

        let plugin = init_plugin().unwrap(); // needs to be handled properly, maybe implement
                                             // map_err on init_plugin
        todo!()
    }

    Ok(())
}
