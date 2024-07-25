use abi_stable::StableAbi;

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

/// For conversions from `catch_unwind` mostly
impl<T> From<std::thread::Result<T>> for MayPanic<T> {
    fn from(res: std::thread::Result<T>) -> Self {
        match res {
            Ok(val) => MayPanic::NoPanic(val),
            Err(_) => MayPanic::Panic,
        }
    }
}
