//! Sealed trait pattern implementation
//!
//! This module contains the sealed trait used to prevent external implementations
//! of core traits while still allowing them to be used publicly.

/// Sealed trait to prevent external trait implementations.
///
/// This trait cannot be implemented by external crates, which allows us to:
/// - Add methods to public traits without breaking changes
/// - Maintain API compatibility guarantees
/// - Control the implementation surface
///
/// The sealed trait pattern is a common Rust idiom for "closed" traits.
pub trait Sealed {}

// Implement Sealed for types that are allowed to implement our public traits
impl<T> Sealed for T where T: crate::api::v1::PluginImpl {}
