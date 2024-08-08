use abi_stable::rvec;
use abi_stable::std_types::{RBox, RResult::ROk, RStr, RString, RVec};
use abi_stable::std_types::{RHashMap, ROption};
use abi_stable::traits::IntoReprC;
use abi_stable::type_level::downcasting::TD_Opaque;
use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, rstr, sabi_extern_fn};
use malbox_abi_common::modules::{
    ModuleConfig, ModuleContext, ModuleState, RawModule, RawModule_TO,
};
use malbox_abi_common::plugins::{PluginConfig, RawPlugin_TO};
use malbox_abi_common::util::AnalysisResult;
use malbox_abi_common::util::MayPanic::{self, NoPanic};
use malbox_abi_common::Value;
use malbox_abi_common::{
    modules::ModuleType,
    plugins::{PluginInfo, PluginType},
    ModuleMod, ModuleMod_Ref, RResult,
};
use serde::{Deserialize, Serialize};

use std::{
    panic::{self, AssertUnwindSafe},
    time::{Duration, Instant},
};

#[derive(Clone)]
struct VmiModule;

impl RawModule for VmiModule {
    fn get_config(&self) -> ModuleConfig where {
        let mut plugins = RVec::new();
        plugins.push(PluginConfig {
            name: RString::from("dummy_plugin"),
            enabled: true,
            settings: RHashMap::new(), // todo
        });
        ModuleConfig {
            name: RString::from("MyModule"),
            version: RString::from("1.0.0"),
            module_type: ModuleType::AnalysisModule,
            plugins,
            plugin_order: vec![RString::from("dummy_plugin")].into(),
        }
    }
    fn get_state(&self) -> MayPanic<RResult<ModuleState>> where {
        todo!()
    }

    fn on_start(&mut self, ctx: &ModuleContext) -> MayPanic<RResult<()>> where {
        todo!()
    }
    fn on_stop(&mut self, ctx: &ModuleContext) -> MayPanic<()> where {
        todo!()
    }
}

#[export_root_module]
fn instantiate_root_module() -> ModuleMod_Ref {
    ModuleMod { new }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new(config: ROption<Value>) -> RawModule_TO<'static, RBox<()>> {
    RawModule_TO::from_value(VmiModule, TD_Opaque)
}
