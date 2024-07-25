use abi_stable::{
    declare_root_module_statics,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, ROption, RResult::ROk, RStr, RString, RVec},
    StableAbi,
};

use crate::util::MayPanic;
use crate::RResult;

#[repr(C)]
#[derive(StableAbi)]
pub enum PluginType {
    StaticAnalysis,
    DynamicAnalysis,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginInfo {
    pub name: RString,
    pub version: RString,
    pub description: RString,
    pub plugin_type: PluginType,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginContext {
    pub field: u8,
}

#[sabi_trait]
pub trait RawPlugin: Send {
    fn get_info(&self) -> PluginInfo;
    fn initialize(&mut self, ctf: &PluginContext) -> MayPanic<RResult<()>>;
    fn execute(&self, data: RVec<u8>) -> MayPanic<RResult<RVec<u8>>>;
    fn cleanup(&mut self) -> MayPanic<()>;
}
