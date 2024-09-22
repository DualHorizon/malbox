use malbox_abi_common::{AnalysisPlugin, Plugin, PluginInfo, PluginType};
use stabby::{result::Result, string::String};

struct MyPlugin;

impl AnalysisPlugin for MyPlugin {
    extern "C" fn get_info(&self) -> PluginInfo {
        PluginInfo {
            name: "hi".into(),
            _type: PluginType::Analysis,
            version: "1.1.1".into(),
        }
    }
}

#[stabby::export]
pub extern "C" fn init_plugin() -> Result<Plugin, String> {
    Result::Ok(stabby::boxed::Box::new(MyPlugin).into())
}
