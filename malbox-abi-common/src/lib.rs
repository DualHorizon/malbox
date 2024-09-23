use stabby::{string::String, vec::Vec};

#[stabby::stabby]
#[repr(u8)]
pub enum PluginType {
    Analysis,
    Scheduling,
    Storage,
    Machinery,
}

#[stabby::stabby]
#[repr(u8)]
pub enum PluginStatus {
    NotStarted,
    Running,
    Stopped,
}

#[stabby::stabby]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub _type: PluginType,
    pub dependencies: Vec<PluginDependency>,
}

#[stabby::stabby]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
}

#[stabby::stabby]
pub struct AnalysisResult {
    pub score: f32, // TBD
}

#[stabby::stabby(checked)]
pub trait AnalysisPlugin {
    extern "C" fn get_info(&self) -> PluginInfo;
    extern "C" fn analyze(&self) -> stabby::result::Result<AnalysisResult, String>; // TBD
}

pub type Plugin = stabby::dynptr!(stabby::boxed::Box<dyn AnalysisPlugin>);
