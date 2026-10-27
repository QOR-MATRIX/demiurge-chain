//! The Library (ADR-094, DIRECTION L9): its index is the core's (`qor_launcher_core::library`, moved there by DIRECTION
//! L13.2 with its tests), re-exported here at its old path; what the webview calls is [`commands`].

pub use qor_launcher_core::library::*;

pub mod commands;
