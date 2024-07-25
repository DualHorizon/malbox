use abi_stable::std_types::{RBox, RResult::ROk, RStr, RString, RVec};
use abi_stable::type_level::downcasting::TD_Opaque;
use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, rstr, sabi_extern_fn};
use malbox_module_common::modules::{ModuleContext, RawModule, RawModule_TO};
use malbox_module_common::plugins::RawPlugin_TO;
use malbox_module_common::util::MayPanic::{self, NoPanic};
use malbox_module_common::Value;
use malbox_module_common::{
    modules::ModuleType,
    plugins::{PluginInfo, PluginType},
    ModuleMod, ModuleMod_Ref, RResult,
};

use std::{
    panic::{self, AssertUnwindSafe},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
struct VmiModule {
    field: String,
}

impl RawModule for VmiModule {
    fn execute_plugin(&self, name: RStr<'_>, data: RVec<u8>) -> MayPanic<RResult<RVec<u8>>> {
        let vec: RVec<u8> = RVec::new();

        panic::catch_unwind(|| ROk(vec)).into()
    }
    fn get_loaded_plugins(&self) -> MayPanic<RResult<RVec<PluginInfo>>> where {
        let mut vec = RVec::new();

        vec.push(PluginInfo {
            name: RString::from("dummy_plugin"),
            version: RString::from("dummy_plugin"),
            description: RString::from("dummy_plugin"),
            plugin_type: PluginType::StaticAnalysis,
        });

        NoPanic(ROk(vec))
    }
    fn get_name<'_self>(&'_self self) -> RStr<'_self> where {
        rstr!("dummy_module")
    }
    fn get_type(&self) -> ModuleType where {
        ModuleType::AnalysisModule
    }
    fn get_version<'_self>(&'_self self) -> RStr<'_self> where {
        rstr!("1.0.0")
    }
    fn load_plugin(&mut self, plugin: RawPlugin_TO<'static, RBox<()>>) -> MayPanic<RResult<()>> where
    {
        todo!();
    }
    fn on_start(&mut self, ctx: &ModuleContext) -> MayPanic<RResult<()>> where {
        todo!();
    }
    fn on_stop(&mut self, ctx: &ModuleContext) -> MayPanic<()> where {
        todo!();
    }
    fn unload_plugin(&mut self, name: RStr<'_>) -> MayPanic<RResult<()>> where {
        todo!();
    }
}
