use abi_stable::rvec;
use abi_stable::std_types::RHashMap;
use abi_stable::std_types::{RBox, RResult::ROk, RStr, RString, RVec};
use abi_stable::traits::IntoReprC;
use abi_stable::type_level::downcasting::TD_Opaque;
use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, rstr, sabi_extern_fn};
use malbox_module_common::modules::{
    ModuleConfig, ModuleContext, ModuleState, RawModule, RawModule_TO,
};
use malbox_module_common::plugins::{PluginConfig, RawPlugin_TO};
use malbox_module_common::util::AnalysisResult;
use malbox_module_common::util::MayPanic::{self, NoPanic};
use malbox_module_common::Value;
use malbox_module_common::{
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
struct VmiModule {}

impl RawModule for VmiModule {
    fn get_config(&self) -> ModuleConfig where {
        let mut plugins = RHashMap::new();
        plugins.insert(
            RString::from("plugin1"),
            PluginConfig {
                name: RString::from("plugin1"),
                enabled: true,
                settings: RHashMap::new(), // todo
            },
        );
        plugins.insert(
            RString::from("plugin2"),
            PluginConfig {
                name: RString::from("plugin2"),
                enabled: true,
                settings: RHashMap::new(),
            },
        );

        ModuleConfig {
            name: RString::from("MyModule"),
            version: RString::from("1.0.0"),
            module_type: ModuleType::AnalysisModule,
            plugins,
            plugin_order: vec![RString::from("plugin1"), RString::from("plugin2")].into(),
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
