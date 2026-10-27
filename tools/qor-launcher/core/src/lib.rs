//! The QOR Launcher's core (ADR-104): what holds the keys and talks to the chain, shared by the Windows launcher, which
//! uses it through Tauri, the Android launcher, which links it through [`bridge`], and QOR OS, which builds it at a
//! pinned commit (ADR-120).
//!
//! It was moved here from the Tauri crate one module at a time, with its tests (DIRECTION L13.2): Sparks and amounts
//! ([`cgt`]), the error every boundary returns ([`error`]), the wallet ([`vault`]) with the confirmation it asks through
//! ([`confirm`]), the profiles each QOR ID on a device has ([`profiles`], ADR-099), and, each behind a feature of its own
//! so the Android shell links only what it uses, QOR ID's client (`identity`), Social's socket (`social`), the chain
//! client and what is built on it (`chain`: `pay`, `agents`, `listings`, `partners`), the object model (`content`), the Library's index
//! (`library`), the Mesh (`mesh`) and Qontrol (`qontrol`). The Windows launcher keeps its Tauri commands, its window and
//! the products it docks.

pub mod bridge;
pub mod cgt;
pub mod commands;
pub mod confirm;
pub mod error;
pub mod profiles;
pub mod vault;

#[cfg(feature = "identity")]
pub mod identity;
#[cfg(feature = "social")]
pub mod social;

#[cfg(feature = "chain")]
pub mod agents;
#[cfg(feature = "chain")]
pub mod chain;
#[cfg(feature = "content")]
pub mod content;
#[cfg(feature = "chain")]
pub mod listings;
#[cfg(feature = "chain")]
pub mod partners;
#[cfg(feature = "chain")]
pub mod pay;

#[cfg(feature = "library")]
pub mod library;
#[cfg(feature = "mesh")]
pub mod mesh;
#[cfg(feature = "qontrol")]
pub mod qontrol;

#[cfg(any(test, feature = "testing"))]
pub mod testing;

// What the person is asked to approve, and the trait that asks them, at the crate's root, where the modules moved from
// the Windows launcher name them.
pub use confirm::{Answer, Confirm, Prompt};

/// This core's version, as the shell shows it.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
