use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub enum MachineArch {
    #[serde(alias = "x86")]
    X86,
    #[serde(alias = "x64")]
    X64,
}

#[derive(Debug, Deserialize, Clone)]
pub enum MachinePlatform {
    #[serde(alias = "windows")]
    Windows,
    #[serde(alias = "linux")]
    Linux,
    #[serde(alias = "macos", alias = "Macos")]
    MacOs,
}
