//! HTTP request handlers.

pub mod account;
pub mod admin;
pub mod auth;
pub mod health;
pub mod oauth;
pub mod pages;
pub mod profile;
pub mod progress;
pub mod webhooks;
pub mod zk;
// The music handlers do not compile and never have. They are written against a
// schema and an error type that do not exist:
//
//   * six queries select `users.qor_id`, a column no migration creates (the
//     schema stores `username` + `discriminator` separately);
//   * they call `AppError::internal/unauthorized/not_found/forbidden`, none of
//     which are variants of `AppError`;
//   * they import `crate::middleware::auth::Claims`, which is not there;
//   * several handlers treat nullable columns as non-null.
//
// That is 48 compile errors, and because Rust builds the whole binary, this one
// unused module made the ENTIRE auth service unbuildable: registration, login,
// keypair auth, profile and agents included.
//
// Disabled so the service builds. Re-enable it once the queries are reconciled
// with the schema. See docs/architecture/PLATFORM_REALIGNMENT.md.
// pub mod music;
pub mod agents;
