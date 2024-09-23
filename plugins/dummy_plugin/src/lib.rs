use malbox_abi_common::{AnalysisResult, PluginType};
use malbox_plugin_macros::{analysis, plugin};
use stabby::result::Result as StabbyResult;
use stabby::string::String as StabbyString;
use stabby::vec::Vec as StabbyVec;

#[derive(Default)]
#[plugin("DummyPlugin", Analysis, "DependencyPlugin1", "DependencyPlugin2")]
pub struct DummyPlugin {
    analysis_count: u32,
}

impl DummyPlugin {
    #[analysis]
    fn analyze(&self) -> StabbyResult<AnalysisResult, StabbyString> {
        println!("DummyPlugin is performing analysis...");

        let result = AnalysisResult { score: 0.75 };

        println!("Analysis complete. Score: {}", result.score);

        StabbyResult::Ok(result)
    }
}
