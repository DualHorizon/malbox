use stabby::string::String;

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
}

#[stabby::stabby(checked)]
pub trait AnalysisPlugin {
    extern "C" fn get_info(&self) -> PluginInfo;
}

pub type Plugin = stabby::dynptr!(stabby::boxed::Box<dyn AnalysisPlugin>);
