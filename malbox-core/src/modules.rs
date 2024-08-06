use abi_stable::std_types::{
    RBox, RCowStr, RHashMap, ROption,
    RResult::{RErr, ROk},
    RString, RVec, Tuple2,
};
use anyhow::anyhow;
use malbox_module_common::{
    modules::{ModuleConfig, ModuleContext, ModuleState, RawModule_TO},
    plugins::{PluginState, PluginStatus, RawPlugin_TO},
    util::{AnalysisResult, MayPanic},
    Result,
};
use std::collections::HashMap;

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

        for Tuple2(plugin_name, plugin_config) in config.plugins.iter() {
            if plugin_config.enabled {
                match Self::load_plugin(plugin_name) {
                    Ok(plugin) => {
                        plugins.insert(plugin_name.to_string().into(), plugin);
                        plugin_states.insert(
                            plugin_name.clone(),
                            PluginState {
                                name: plugin_name.clone(),
                                status: PluginStatus::NotStarted,
                                result: ROption::RNone,
                            },
                        );
                    }
                    Err(e) => {
                        return Err(anyhow!("Failed to load plugin {}: {:?}", plugin_name, e).into())
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
        // Implement plugin loading logic here
        // This is just a placeholder implementation
        Err(anyhow!("Plugin loading not implemented for {}", name).into())
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

    pub async fn execute_plugins(&mut self, data: RVec<u8>) -> Result<Vec<AnalysisResult>> {
        let mut results = Vec::new();

        for plugin_name in &self.config.plugin_order {
            let plugin = self.plugins.get(plugin_name).unwrap();
            let plugin_state = self.state.plugin_states.get_mut(plugin_name).unwrap();

            plugin_state.status = PluginStatus::Running;

            match plugin.execute(data.clone()).unwrap() {
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
