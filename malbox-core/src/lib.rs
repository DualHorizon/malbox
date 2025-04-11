// TODO: Docs.
// FIXME: Will have cyclic dependencies issues with malbox-plugin-api.
// Either we define plugin types in this crate and export them to malbox-plugin-api,
// or we unify both crates. TBD.

pub mod communication;
pub mod errors;
pub mod plugins;

pub use communication::ipc::plugin::PluginIpc;
pub use plugins::manager::PluginManager;
