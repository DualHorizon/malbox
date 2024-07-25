use crate::plugins::{PluginInfo, RawPlugin_TO};
use crate::util::MayPanic;
use crate::RResult;

use abi_stable::{
    declare_root_module_statics,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, RBoxError, ROption, RStr, RString, RVec},
    StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
pub enum ModuleType {
    MachineryModule,
    AnalysisModule,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct ModuleContext {
    pub state: RString,
}

#[sabi_trait]
pub trait RawModule: Send {
    fn load_plugin(&mut self, plugin: RawPlugin_TO<'static, RBox<()>>) -> MayPanic<RResult<()>>;
    fn unload_plugin(&mut self, name: RStr<'_>) -> MayPanic<RResult<()>>;
    fn get_loaded_plugins(&self) -> MayPanic<RResult<RVec<PluginInfo>>>;
    fn execute_plugin(&self, name: RStr<'_>, data: RVec<u8>) -> MayPanic<RResult<RVec<u8>>>;
    fn on_start(&mut self, ctx: &ModuleContext) -> MayPanic<RResult<()>>;
    fn on_stop(&mut self, ctx: &ModuleContext) -> MayPanic<()>;
    fn get_type(&self) -> ModuleType;
    fn get_name(&self) -> RStr<'_>;
    fn get_version(&self) -> RStr<'_>;
}
