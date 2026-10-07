//! Infrastructure layer.
//!
//! Adapters for external concerns: HTTP, queues, FFI.
//! These modules depend on the domain, never the other way around.

pub mod ffi;
pub mod http;
pub mod queue;