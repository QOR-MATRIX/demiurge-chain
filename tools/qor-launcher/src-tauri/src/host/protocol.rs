//! The QOR host protocol, version 1 (ADR-046 decisions 4 and 7, ADR-088 decision 4): what a product and the launcher
//! say to each other. The schema is `tools/qor-launcher/host-protocol/schema.json`, and every frame in its `golden/`
//! folder is read by the tests, so the two cannot drift apart.
//!
//! Each frame is one JSON object with a `type`. Unknown fields are ignored, so a later version can add one without
//! breaking an earlier side; a new method is announced as a capability; an unknown method is answered
//! `unsupported_method`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The protocol version this launcher speaks.
pub const PROTOCOL: u32 = 1;
/// The oldest version it still speaks.
pub const MIN_PROTOCOL: u32 = 1;

/// What a product sends.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FromProduct {
    /// The first frame, and only the first: who it is, what it speaks, and the token it was given on its stdin.
    Hello {
        protocol: u32,
        min_protocol: u32,
        product: String,
        version: String,
        token: String,
    },
    /// A typed intent (ADR-046 decision 7): a method the launcher announced, with named arguments. Never bytes to
    /// sign.
    Request {
        id: u64,
        method: String,
        #[serde(default)]
        params: Value,
    },
}

/// What the launcher sends.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToProduct {
    /// The answer to a good `hello`: the version chosen, the methods it will honour, and the session's id.
    Welcome {
        protocol: u32,
        capabilities: Vec<&'static str>,
        session: String,
    },
    /// A request done.
    Result { id: u64, result: Value },
    /// A request refused, or (with no id) a handshake refused, after which the channel closes.
    Error {
        id: Option<u64>,
        code: String,
        message: String,
    },
}

/// The codes an `error` frame carries, besides those of the launcher's own errors (`declined`, `vault_locked`,
/// `qontrol`, `rpc`, …, as the interface already knows them).
pub mod code {
    pub const UNSUPPORTED_METHOD: &str = "unsupported_method";
    pub const BAD_REQUEST: &str = "bad_request";
    pub const PROTOCOL_MISMATCH: &str = "protocol_mismatch";
    pub const NOT_AUTHORISED: &str = "not_authorised";
}
