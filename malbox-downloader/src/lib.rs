// NOTE: Don't know about the name of this crate.
// Maybe malbox-fetcher? Open to suggestions.

mod downloader;
mod error;
pub mod registry;

pub use downloader::Downloader;
pub use error::Error;
// pub use registry::{DownloadRegistry, DownloadSource, SourceType};

pub use registry::{
    Architecture, Platform, ProcessingStatus, SourceEdition, SourceFamily, SourceMetadata,
    SourceRegistry, SourceRelease, SourceType, SourceVariant, SystemRequirements,
};
