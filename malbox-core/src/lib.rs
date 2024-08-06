use abi_stable::{library::lib_header_from_path, std_types::ROption};
use anyhow::anyhow;
use malbox_module_common::{ModuleMod_Ref, Value};
use std::path::Path;

mod modules;
use modules::Module;

pub async fn load_module(path: &Path, config: Option<Value>) -> anyhow::Result<Module> {
    let module_lib = lib_header_from_path(path)?.init_root_module::<ModuleMod_Ref>()?;

    let new_module = module_lib
        .new_module()
        .ok_or_else(|| anyhow!("method new_module not found"))?;

    let r_config = match config {
        Some(v) => ROption::RSome(v),
        None => ROption::RNone,
    };

    let raw_module = new_module(r_config);

    Ok(Module::new(raw_module).unwrap())
}
