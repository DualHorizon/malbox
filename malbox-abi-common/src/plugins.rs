use crate::util::{AnalysisResult, MayPanic};
use crate::RResult;
use crate::Value;
use abi_stable::{
    sabi_trait,
    std_types::{RBox, RHashMap, ROption, RStr, RString, RVec},
    StableAbi,
};

// most of those structs should use RStr instead of RString.. will fix later
#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct PluginConfig {
    pub name: RString,
    pub enabled: bool,
    pub settings: RHashMap<RString, Value>,
}

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
#[derive(StableAbi, Clone)]
pub struct PluginState {
    pub name: RString,
    pub status: PluginStatus,
    pub result: ROption<AnalysisResult>,
}

#[repr(C)]
#[derive(StableAbi, Clone, PartialEq)]
pub enum PluginStatus {
    NotStarted,
    Running,
    Completed,
    Failed,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct PluginContext {
    pub field: u8,
}

#[sabi_trait]
pub trait RawPlugin: Send {
    fn get_info(&self) -> PluginInfo;
    fn initialize(&mut self, ctx: &PluginContext) -> MayPanic<RResult<()>>;
    fn execute(&self, data: RVec<u8>) -> MayPanic<RResult<AnalysisResult>>;
    fn cleanup(&mut self) -> MayPanic<()>;
}
