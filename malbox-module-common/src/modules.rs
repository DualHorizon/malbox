use crate::plugins::{PluginConfig, PluginState, RawPlugin_TO};
use crate::util::MayPanic;
use crate::RResult;
use abi_stable::{
    sabi_trait,
    std_types::{RBox, RCowStr, RHashMap, RStr, RString, RVec},
    StableAbi,
};

#[repr(C)]
#[derive(StableAbi, Clone)]
pub enum ModuleType {
    MachineryModule,
    AnalysisModule,
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct ModuleConfig {
    pub name: RString,
    pub version: RString,
    pub module_type: ModuleType,
    pub plugins: RHashMap<RString, PluginConfig>,
    pub plugin_order: RVec<RString>,
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct ModuleState {
    pub config: ModuleConfig,
    pub plugin_states: RHashMap<RString, PluginState>,
    pub current_plugin_index: usize,
}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct ModuleContext {
    pub execution_id: u32,
    pub input_data: RVec<u8>,
}
#[sabi_trait]
pub trait RawModule: Send {
    fn get_config(&self) -> ModuleConfig;
    fn on_start(&mut self, ctx: &ModuleContext) -> MayPanic<RResult<()>>;
    fn on_stop(&mut self, ctx: &ModuleContext) -> MayPanic<()>;
    fn get_state(&self) -> MayPanic<RResult<ModuleState>>;
}
