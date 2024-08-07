use abi_stable::{
    library::lib_header_from_path,
    rvec,
    std_types::{
        RBox, RCowStr, RHashMap, ROption,
        RResult::{RErr, ROk},
        RString, RVec, Tuple2,
    },
};
use anyhow::anyhow;
use malbox_abi_common::{
    modules::{ModuleConfig, ModuleContext, ModuleState, RawModule_TO},
    plugins::{PluginState, PluginStatus, RawPlugin_TO},
    util::{AnalysisResult, MayPanic},
    PluginMod_Ref, Result,
};
use std::path::Path;
use std::{collections::HashMap, os::linux::raw};

pub struct Module {
    raw_module: RawModule_TO<'static, RBox<()>>,
    config: ModuleConfig,
    state: ModuleState,
    plugins: HashMap<RString, RawPlugin_TO<'static, RBox<()>>>,
}

impl Module {
    pub fn new(raw_module: RawModule_TO<'static, RBox<()>>) -> Result<Self> {
        let config = raw_module.get_config();
        let mut plugins = HashMap::new();
        let mut plugin_states = RHashMap::new();

        for plugin_config in config.plugins.iter() {
            if plugin_config.enabled {
                match Self::load_plugin(&plugin_config.name) {
                    Ok(plugin) => {
                        plugins.insert(plugin_config.name.to_string().into(), plugin);
                        plugin_states.insert(
                            plugin_config.name.clone(),
                            PluginState {
                                name: plugin_config.name.clone(),
                                status: PluginStatus::NotStarted,
                                result: ROption::RNone,
                            },
                        );
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "Failed to load plugin {}: {:?}",
                            plugin_config.name,
                            e
                        )
                        .into())
                    }
                }
            }
        }

        Ok(Self {
            raw_module,
            config: config.clone(),
            state: ModuleState {
                config,
                plugin_states,
                current_plugin_index: 0,
            },
            plugins,
        })
    }

    fn load_plugin(name: &RString) -> Result<RawPlugin_TO<'static, RBox<()>>> {
        tracing::info!("Loading plugin: {}", name);

        let plugin_path_str = format!("./plugins/{name}/target/debug/lib{name}.so");

        let plugin_path = Path::new(&plugin_path_str);

        let plugin_lib = lib_header_from_path(plugin_path)?.init_root_module::<PluginMod_Ref>()?;

        let new_plugin = plugin_lib
            .new()
            .ok_or_else(|| anyhow!("method new not found"))?;

        Ok(new_plugin(ROption::RNone))
    }

    pub async fn start(&mut self, ctx: &ModuleContext) -> Result<()> {
        match self.raw_module.on_start(ctx).unwrap() {
            ROk(_) => Ok(()),
            RErr(e) => Err(anyhow!("Failed to start module: {:?}", e).into()),
        }
    }

    pub async fn stop(&mut self, ctx: &ModuleContext) -> Result<()> {
        self.raw_module.on_stop(ctx).unwrap();
        Ok(())
    }

    pub async fn execute_plugins(&mut self, data: Vec<u8>) -> Result<Vec<AnalysisResult>> {
        let mut results = Vec::new();

        for plugin_name in &self.config.plugin_order {
            let plugin = self.plugins.get(plugin_name).unwrap();
            let plugin_state = self.state.plugin_states.get_mut(plugin_name).unwrap();

            plugin_state.status = PluginStatus::Running;

            match plugin.execute(data.clone().into()).unwrap() {
                ROk(result) => {
                    plugin_state.status = PluginStatus::Completed;
                    plugin_state.result = ROption::RSome(result.clone());
                    results.push(result);
                }
                RErr(err) => {
                    plugin_state.status = PluginStatus::Failed;
                    return Err(anyhow!("Plugin execution failed: {:?}", err).into());
                }
            }

            self.state.current_plugin_index += 1;
        }

        Ok(results)
    }

    pub async fn get_state(&self) -> Result<ModuleState> {
        match self.raw_module.get_state() {
            MayPanic::NoPanic(ROk(state)) => Ok(state),
            MayPanic::NoPanic(RErr(err)) => Err(anyhow!("Failed to get state: {:?}", err).into()),
            MayPanic::Panic => Err(anyhow!("Getting state panicked").into()),
        }
    }
}
