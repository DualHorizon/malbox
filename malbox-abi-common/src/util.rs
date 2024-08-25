use abi_stable::{
    declare_root_module_statics,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{
        RArc, RBox, RBoxError, RHashMap, ROption,
        RResult::{RErr, ROk},
        RStr, RString, RVec,
    },
    StableAbi,
};

use crate::{RResult, Value};
use serde_json::Value as JsonValue;
use std::fmt;

#[repr(C)]
#[derive(Debug, StableAbi)]
pub enum MayPanic<T> {
    Panic,
    NoPanic(T),
}

impl<T> MayPanic<T> {
    /// NOTE: Until https://doc.rust-lang.org/std/ops/trait.Try.html is
    /// stabilized.
    pub fn unwrap(self) -> T {
        match self {
            MayPanic::Panic => panic!("unwrap: unhandled panic"),
            MayPanic::NoPanic(t) => t,
        }
    }
}

impl<T: Default> Default for MayPanic<T> {
    fn default() -> Self {
        MayPanic::NoPanic(T::default())
    }
}

/// For conversions from `catch_unwind` mostly (we can use c-unwind ABI instead, just temporary)
impl<T> From<std::thread::Result<T>> for MayPanic<T> {
    fn from(res: std::thread::Result<T>) -> Self {
        match res {
            Ok(val) => MayPanic::NoPanic(val),
            Err(_) => MayPanic::Panic,
        }
    }
}

#[derive(Debug)]
struct AnalysisError(String);

impl fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AnalysisError {}

#[repr(C)]
#[derive(StableAbi, Clone)]
pub struct AnalysisResult {
    pub data: RString,
}

impl AnalysisResult {
    pub fn new<T: serde::Serialize>(value: &T) -> MayPanic<RResult<Self>> {
        match serde_json::to_string(value) {
            Ok(json) => MayPanic::NoPanic(ROk(Self { data: json.into() })),
            Err(e) => MayPanic::NoPanic(RErr(RBoxError::new(AnalysisError(e.to_string())))),
        }
    }

    pub fn parse<T: serde::de::DeserializeOwned>(&self) -> MayPanic<RResult<T>> {
        match serde_json::from_str(&self.data) {
            Ok(value) => MayPanic::NoPanic(ROk(value)),
            Err(e) => MayPanic::NoPanic(RErr(RBoxError::new(AnalysisError(e.to_string())))),
        }
    }

    pub fn as_json(&self) -> MayPanic<RResult<JsonValue>> {
        match serde_json::from_str(&self.data) {
            Ok(value) => MayPanic::NoPanic(ROk(value)),
            Err(e) => MayPanic::NoPanic(RErr(RBoxError::new(AnalysisError(e.to_string())))),
        }
    }
}
