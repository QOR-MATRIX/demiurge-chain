//! HTTP request handlers.

pub mod account;
pub mod admin;
pub mod auth;
pub mod avatar;
pub mod health;
pub mod oauth;
pub mod pages;
pub mod profile;
pub mod progress;
pub mod webhooks;
pub mod zk;
// The music handlers are dead code: not compiled, not routed, and outside the
// current scope (docs/DIRECTION.md). They do not compile and never have. They are
// written against a schema and an error type that do not exist:
//
//   * six queries select `users.qor_id`, a column no migration creates (a
//     username is unique on its own since ADR-075);
//   * they call `AppError::internal/unauthorized/not_found/forbidden`, none of
//     which are variants of `AppError`;
//   * they import `crate::middleware::auth::Claims`, which is not there;
//   * several handlers treat nullable columns as non-null.
//
// That is 48 compile errors, and because Rust builds the whole binary, this one
// unused module made the ENTIRE auth service unbuildable: registration, login,
// keypair auth, profile and agents included.
//
// Disabled so the service builds. Whether to delete the file or rewrite it is a
// scope call left to the owner (docs/REALIGNMENT-2026-09-21.md). Migration 004's
// music tables still exist, since migrations are never removed.
// pub mod music;
pub mod agents;
