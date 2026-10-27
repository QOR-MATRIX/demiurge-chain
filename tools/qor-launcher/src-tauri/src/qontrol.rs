//! Qontrol (DIRECTION P1): reading, committing and branching a project is the core's (`qor_launcher_core::qontrol`,
//! moved there by DIRECTION L13.2 with its tests), re-exported here at its old path; what the webview calls is
//! [`commands`].

pub use qor_launcher_core::qontrol::*;

pub mod commands;
