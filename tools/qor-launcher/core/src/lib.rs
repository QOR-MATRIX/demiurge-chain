//! The QOR Launcher's core (ADR-104): what holds the keys and talks to the chain, shared by the Windows launcher, which
//! uses it through Tauri, and the Android launcher, which links it through [`bridge`].
//!
//! It is moved here from the Tauri crate one module at a time, with its tests (DIRECTION L13.2). So far: Sparks and
//! amounts ([`cgt`]) and the error every boundary returns ([`error`]).

pub mod bridge;
pub mod cgt;
pub mod commands;
pub mod error;

/// This core's version, as the shell shows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
