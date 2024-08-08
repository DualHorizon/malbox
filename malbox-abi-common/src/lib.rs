use abi_stable::{
    declare_root_module_statics,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, RBoxError, ROption, RStr, RString, RVec},
    StableAbi,
};

pub mod modules;
pub mod plugins;
pub mod util;

use modules::RawModule_TO;
use plugins::RawPlugin_TO;

#[repr(C)]
#[derive(StableAbi, Clone)]
pub enum Value {
    String(RString),
    Integer(i32),
}

pub type RResult<T> = abi_stable::std_types::RResult<T, RBoxError>;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct ModuleMod {
    pub new: extern "C" fn(config: ROption<Value>) -> RawModule_TO<'static, RBox<()>>,
}

impl RootModule for ModuleMod_Ref {
    const BASE_NAME: &'static str = "module";
    const NAME: &'static str = "module";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    declare_root_module_statics! {ModuleMod_Ref}
}

// note: it may be better to not implement plugins as RootModule(s) maybe we could propagate the plugin libs through the modules RootModule
#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct PluginMod {
    pub new: extern "C" fn(config: ROption<Value>) -> RawPlugin_TO<'static, RBox<()>>,
}

impl RootModule for PluginMod_Ref {
    const BASE_NAME: &'static str = "plugin";
    const NAME: &'static str = "plugin";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
    declare_root_module_statics! {PluginMod_Ref}
}
