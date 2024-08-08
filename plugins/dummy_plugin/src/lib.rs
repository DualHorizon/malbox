use abi_stable::rvec;
use abi_stable::std_types::{RBox, RResult::ROk, RStr, RString, RVec};
use abi_stable::std_types::{RHashMap, ROption};
use abi_stable::traits::IntoReprC;
use abi_stable::type_level::downcasting::TD_Opaque;
use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, rstr, sabi_extern_fn};
use malbox_abi_common::plugins::{PluginConfig, RawPlugin, RawPlugin_TO};
use malbox_abi_common::util::AnalysisResult;
use malbox_abi_common::util::MayPanic::{self, NoPanic};
use malbox_abi_common::{
    modules::ModuleType,
    plugins::{PluginInfo, PluginType},
    ModuleMod, ModuleMod_Ref, RResult,
};
use malbox_abi_common::{PluginMod, PluginMod_Ref, Value};
use serde::{Deserialize, Serialize};

use std::{
    panic::{self, AssertUnwindSafe},
    time::{Duration, Instant},
};

#[derive(Clone)]
struct VmiProcessPlugin;

impl RawPlugin for VmiProcessPlugin {
    fn get_info(&self) -> PluginInfo where {
        todo!()
    }
    fn initialize(
        &mut self,
        ctx: &malbox_abi_common::plugins::PluginContext,
    ) -> MayPanic<RResult<()>> where {
        todo!()
    }
    fn execute(&self, data: RVec<u8>) -> MayPanic<RResult<AnalysisResult>> where {
        println!("EXECUTING PLUGIN....");

        MayPanic::NoPanic(ROk(AnalysisResult {
            data: "test".into(),
        }))
    }
    fn cleanup(&mut self) -> MayPanic<()> where {
        todo!()
    }
}

#[export_root_module]
fn instantiate_root_module() -> PluginMod_Ref {
    PluginMod { new }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new(config: ROption<Value>) -> RawPlugin_TO<'static, RBox<()>> {
    RawPlugin_TO::from_value(VmiProcessPlugin, TD_Opaque)
}
