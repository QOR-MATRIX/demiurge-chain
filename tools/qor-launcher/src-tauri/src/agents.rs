//! `qor://agent`: an agent asks the person to let it act for them (ADR-112, L5.1).
//!
//! A link is `qor://agent?r=<hex of the request's JSON>`. The request is the agent's own: its account, a challenge QOR
//! ID issued for that account, its key's signature over the challenge, and what it asks to be allowed. Nothing in it is
//! trusted until it is checked here, before any dialog:
//!
//! - the agent's signature verifies over the challenge under QOR ID's domain tag, so whoever made the link holds the
//!   agent's key;
//! - the challenge has QOR ID's shape and is no more than ten minutes old (QOR ID keeps one for five; the rest allows for
//!   this PC's clock), so an old link is refused rather than asked;
//! - every capability is one QOR ID knows, named once; the autonomy is one of four; a bounded agent names a limit, and
//!   no limit is negative;
//! - the name makes a QOR ID like any other (`agent_` and 3 to 14 letters, digits, spaces or underscores), and the
//!   model, if any, is one short line.
//!
//! Only then is the person asked, in the host dialog, with text built here from the checked request. On approval the
//! profile's vault signs [`AgentRequest::statement`], which QOR ID verifies against the key the QOR ID proved, and the
//! host registers the agent with the profile's own session. This module does no network work, so every refusal is
//! tested without QOR ID.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::{QorError, QorResult};
use crate::vault::derive;
use crate::Prompt;

/// The first line of what the controller's key signs (ADR-112); QOR ID's `agents.rs` builds the same bytes.
const AUTHORISATION_DOMAIN: &str = "demiurge:qor-id:agent-authorisation:v1";

/// QOR ID's tag for a challenge signature (security track item 7), which the agent's own signature is made under.
const CHALLENGE_DOMAIN: &str = "demiurge:qor-id:challenge:v1:";

/// The capabilities QOR ID knows (`VALID_CAPABILITIES` in its `agents.rs`; a test holds the two lists together).
const CAPABILITIES: &[&str] = &[
    "read", "analyze", "trade", "transfer", "bounty", "nft", "vote", "stake",
];

const AUTONOMIES: &[&str] = &["supervised", "bounded", "autonomous", "sovereign"];

/// How old a challenge may be when the link is opened, and how far ahead of this PC's clock.
const OLDEST_SECS: i64 = 10 * 60;
const AHEAD_SECS: i64 = 5 * 60;

/// The longest model name shown in the dialog.
const MODEL_LIMIT: usize = 64;

/// The request as the agent wrote it.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asked {
    account_id: String,
    challenge: String,
    signature: String,
    name: String,
    capabilities: Vec<String>,
    autonomy: String,
    #[serde(default)]
    spending_limit: Option<i64>,
    #[serde(default)]
    model: Option<String>,
}

/// A request that passed every check, ready to be shown, signed and sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRequest {
    pub account: [u8; 32],
    /// The agent's account as a person sees it.
    pub address: String,
    pub challenge: String,
    /// The agent key's signature over the challenge, lowercase hex.
    pub signature: String,
    pub name: String,
    /// The QOR ID QOR ID will give it: `agent_` and the name.
    pub qor_id: String,
    /// As asked; the statement names them sorted.
    pub capabilities: Vec<String>,
    pub autonomy: String,
    /// Whole CGT, recorded by QOR ID and not yet enforced anywhere (Q-9).
    pub spending_limit: Option<i64>,
    pub model: Option<String>,
}

fn refuse(why: impl Into<String>) -> QorError {
    QorError::AgentRefused(why.into())
}

/// Read and check a `qor://agent` link. `now` is Unix seconds.
pub fn parse(link: &str, now: i64) -> QorResult<AgentRequest> {
    let rest = link
        .strip_prefix("qor://agent?")
        .or_else(|| link.strip_prefix("qor://agent/?"))
        .ok_or_else(|| refuse("this is not an agent's request"))?;
    let mut query: BTreeMap<&str, &str> = BTreeMap::new();
    for part in rest.split('&') {
        let (key, value) = part
            .split_once('=')
            .ok_or_else(|| refuse("the link is malformed"))?;
        if query.insert(key, value).is_some() {
            return Err(refuse("the link is malformed"));
        }
    }
    let bytes = hex::decode(
        query
            .get("r")
            .ok_or_else(|| refuse("the link carries no request"))?,
    )
    .map_err(|_| refuse("the link is malformed"))?;
    if query.len() != 1 || bytes.len() > 2048 {
        return Err(refuse("the link is malformed"));
    }
    let asked: Asked =
        serde_json::from_slice(&bytes).map_err(|_| refuse("the request is malformed"))?;

    let account: [u8; 32] = hex::decode(asked.account_id.trim_start_matches("0x"))
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| refuse("the agent's account is not one"))?;
    let signature: [u8; 64] = hex::decode(asked.signature.trim_start_matches("0x"))
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| refuse("the agent's signature is malformed"))?;

    let issued = issued_at(&asked.challenge)
        .ok_or_else(|| refuse("the request's challenge is not one QOR ID issues"))?;
    if issued < now - OLDEST_SECS {
        return Err(refuse(
            "this request is too old. Ask the agent for a new one",
        ));
    }
    if issued > now + AHEAD_SECS {
        return Err(refuse(
            "this request's challenge is from the future. Check this PC's clock",
        ));
    }
    let message = format!("{CHALLENGE_DOMAIN}{}", asked.challenge);
    if !derive::verify(&account, message.as_bytes(), &signature) {
        return Err(refuse(
            "the request was not signed by the agent's own key. Nothing was asked",
        ));
    }

    let qor_id = qor_id_for(&asked.name)?;
    let mut seen = Vec::new();
    for capability in &asked.capabilities {
        if !CAPABILITIES.contains(&capability.as_str()) {
            return Err(refuse(format!(
                "\"{capability}\" is not something an agent can be allowed"
            )));
        }
        if seen.contains(capability) {
            return Err(refuse("a capability is named twice"));
        }
        seen.push(capability.clone());
    }
    if seen.is_empty() {
        return Err(refuse("the agent asks to be allowed nothing"));
    }
    if !AUTONOMIES.contains(&asked.autonomy.as_str()) {
        return Err(refuse("the agent's autonomy is not one QOR ID knows"));
    }
    match asked.spending_limit {
        None if asked.autonomy == "bounded" => {
            return Err(refuse("a bounded agent must name its spending limit"));
        }
        Some(limit) if limit < 0 => {
            return Err(refuse("a spending limit cannot be negative"));
        }
        _ => {}
    }
    let model = asked.model.filter(|model| !model.is_empty());
    if model.as_deref().is_some_and(|model| {
        model.chars().count() > MODEL_LIMIT || model.chars().any(char::is_control)
    }) {
        return Err(refuse("the agent's model is not one short line"));
    }

    Ok(AgentRequest {
        account,
        address: derive::address_of(&account),
        challenge: asked.challenge,
        signature: hex::encode(signature),
        name: asked.name,
        qor_id,
        capabilities: asked.capabilities,
        autonomy: asked.autonomy,
        spending_limit: asked.spending_limit,
        model,
    })
}

/// When QOR ID issued `challenge`, if it has the only shape QOR ID issues: `demiurge:<unix seconds>:<64 lowercase hex>`.
fn issued_at(challenge: &str) -> Option<i64> {
    let mut parts = challenge.splitn(3, ':');
    if parts.next() != Some("demiurge") {
        return None;
    }
    let seconds = parts.next()?;
    let nonce = parts.next()?;
    let well_formed = !seconds.is_empty()
        && seconds.bytes().all(|b| b.is_ascii_digit())
        && nonce.len() == 64
        && nonce
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
    if !well_formed {
        return None;
    }
    seconds.parse().ok()
}

/// The QOR ID an agent called `name` is given, as QOR ID makes it: `agent_` and the name, lowercased, spaces as
/// underscores, a QOR ID like any other (3 to 20 letters, digits and underscores).
fn qor_id_for(name: &str) -> QorResult<String> {
    let qor_id = format!("agent_{}", name.to_lowercase().replace(' ', "_"));
    let valid = name.len() >= 3
        && qor_id.len() <= 20
        && qor_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        return Err(refuse(
            "an agent's name is 3 to 14 letters, digits, spaces or underscores",
        ));
    }
    Ok(qor_id)
}

impl AgentRequest {
    /// What the controller's key signs (ADR-112): every field QOR ID records, and the agent's single-use challenge.
    /// QOR ID's `authorisation_statement` builds the same bytes; a test in each holds them to one vector.
    pub fn statement(&self, controller: &[u8; 32]) -> String {
        let mut capabilities: Vec<&str> = self.capabilities.iter().map(String::as_str).collect();
        capabilities.sort_unstable();
        format!(
            "{AUTHORISATION_DOMAIN}\ncontroller:{}\nagent:{}\nchallenge:{}\nname:{}\ncapabilities:{}\nautonomy:{}\n\
             spending_limit:{}\nmodel:{}",
            hex::encode(controller),
            hex::encode(self.account),
            self.challenge,
            self.name,
            capabilities.join(","),
            self.autonomy,
            self.spending_limit
                .map(|limit| limit.to_string())
                .unwrap_or_default(),
            self.model.as_deref().unwrap_or_default(),
        )
    }

    /// What the person is asked, built from the checked request.
    pub fn prompt(&self, controller_qor_id: &str) -> Prompt {
        let limit = match self.spending_limit {
            Some(limit) => format!(", spending at most {limit} CGT"),
            None => String::new(),
        };
        let model = match &self.model {
            Some(model) => format!("\nModel: {model}"),
            None => String::new(),
        };
        Prompt {
            title: "Let an agent act for you?".into(),
            body: format!(
                "An agent asks to act for your QOR ID {controller_qor_id}, as {}.\n\nIts account: {}\nIt may: {}\n\
                 Autonomy: {}{limit}{model}\n\nQOR ID records what it may do and its limit; nothing enforces them yet. \
                 Allow only an agent you started yourself.",
                self.qor_id,
                self.address,
                self.capabilities.join(", "),
                self.autonomy,
            ),
            approve: "Allow agent".into(),
        }
    }

    /// The registration QOR ID takes, with the controller's signature over [`AgentRequest::statement`].
    pub fn registration(&self, controller_signature: &str) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "capabilities": self.capabilities,
            "autonomy": self.autonomy,
            "spending_limit": self.spending_limit,
            "model": self.model,
            "account_id": format!("0x{}", hex::encode(self.account)),
            "challenge": self.challenge,
            "signature": self.signature,
            "controller_signature": controller_signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::Pair as _;

    const NOW: i64 = 1_761_177_600;

    fn key(seed: u8) -> sp_core::sr25519::Pair {
        sp_core::sr25519::Pair::from_seed(&[seed; 32])
    }

    fn challenge(issued: i64) -> String {
        format!("demiurge:{issued}:{}", "ab".repeat(32))
    }

    fn asked(agent: &sp_core::sr25519::Pair, challenge: &str) -> serde_json::Value {
        serde_json::json!({
            "account_id": format!("0x{}", hex::encode(agent.public().0)),
            "challenge": challenge,
            "signature": hex::encode(agent.sign(format!("{CHALLENGE_DOMAIN}{challenge}").as_bytes()).0),
            "name": "Lab Assistant",
            "capabilities": ["trade", "read"],
            "autonomy": "bounded",
            "spending_limit": 25,
            "model": "claude-opus-5-5",
        })
    }

    fn link(request: &serde_json::Value) -> String {
        format!("qor://agent?r={}", hex::encode(request.to_string()))
    }

    fn refused(request: &serde_json::Value, words: &str) {
        match parse(&link(request), NOW) {
            Err(QorError::AgentRefused(why)) => {
                assert!(why.contains(words), "{why:?} does not say {words:?}")
            }
            other => panic!("expected a refusal saying {words:?}, got {other:?}"),
        }
    }

    /// QOR ID builds these exact bytes for the same request (`services/qor-auth/src/handlers/agents.rs`, the test of
    /// the same name): if either side changes, both tests must.
    #[test]
    fn the_statement_is_the_one_the_launcher_signs() {
        // QOR ID's vector uses a short challenge, which `parse` would refuse, so the request is built by hand.
        let request = AgentRequest {
            account: key(2).public().0,
            address: derive::address_of(&key(2).public().0),
            challenge: "demiurge:1761177600:0123456789abcdef".into(),
            signature: String::new(),
            name: "Lab Assistant".into(),
            qor_id: "agent_lab_assistant".into(),
            capabilities: vec!["trade".into(), "read".into()],
            autonomy: "bounded".into(),
            spending_limit: Some(25),
            model: Some("claude-opus-5-5".into()),
        };
        assert_eq!(
            request.statement(&key(1).public().0),
            format!(
                "demiurge:qor-id:agent-authorisation:v1\n\
                 controller:{}\n\
                 agent:{}\n\
                 challenge:demiurge:1761177600:0123456789abcdef\n\
                 name:Lab Assistant\n\
                 capabilities:read,trade\n\
                 autonomy:bounded\n\
                 spending_limit:25\n\
                 model:claude-opus-5-5",
                hex::encode(key(1).public().0),
                hex::encode(key(2).public().0),
            )
        );
        let plain = AgentRequest {
            spending_limit: None,
            model: None,
            ..request
        };
        assert!(plain
            .statement(&key(1).public().0)
            .ends_with("\nspending_limit:\nmodel:"));
    }

    /// The capabilities, the domain tags and the name rule are QOR ID's; read from its source, so they cannot drift.
    #[test]
    fn what_is_checked_here_is_what_qor_id_checks() {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../services/qor-auth/src/handlers/agents.rs"),
        )
        .expect("QOR ID's agents.rs");
        let start = source
            .find("const VALID_CAPABILITIES")
            .expect("its capability list");
        let list = &source[start..start + source[start..].find("];").unwrap()];
        let theirs: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
        assert_eq!(theirs, CAPABILITIES);
        assert!(source.contains(&format!(
            "const AUTHORISATION_DOMAIN: &str = \"{AUTHORISATION_DOMAIN}\";"
        )));
        assert!(source.contains("\"supervised\" | \"bounded\" | \"autonomous\" | \"sovereign\""));
        assert!(source.contains(
            "let qor_id = format!(\"agent_{}\", name.to_lowercase().replace(' ', \"_\"));"
        ));
    }

    #[test]
    fn a_request_the_agent_signed_is_read_whole() {
        let agent = key(2);
        let challenge = challenge(NOW - 30);
        let request = parse(&link(&asked(&agent, &challenge)), NOW).expect("a good request");
        assert_eq!(request.account, agent.public().0);
        assert_eq!(request.address, derive::address_of(&agent.public().0));
        assert_eq!(request.qor_id, "agent_lab_assistant");
        assert_eq!(request.capabilities, ["trade", "read"]);
        assert_eq!(request.spending_limit, Some(25));
        assert_eq!(request.model.as_deref(), Some("claude-opus-5-5"));
        let body = request.registration("feed");
        assert_eq!(body["controller_signature"], "feed");
        assert_eq!(
            body["account_id"],
            format!("0x{}", hex::encode(agent.public().0))
        );
        assert_eq!(body["challenge"], challenge);
    }

    #[test]
    fn a_request_not_signed_by_the_agents_own_key_is_refused() {
        let challenge = challenge(NOW);
        let mut forged = asked(&key(2), &challenge);
        forged["signature"] = asked(&key(3), &challenge)["signature"].clone();
        refused(&forged, "not signed by the agent's own key");
        // Signed bare, without QOR ID's tag.
        let mut bare = asked(&key(2), &challenge);
        bare["signature"] = hex::encode(key(2).sign(challenge.as_bytes()).0).into();
        refused(&bare, "not signed by the agent's own key");
    }

    #[test]
    fn an_old_or_odd_challenge_is_refused_before_anyone_is_asked() {
        refused(
            &asked(&key(2), &challenge(NOW - OLDEST_SECS - 1)),
            "too old",
        );
        refused(
            &asked(&key(2), &challenge(NOW + AHEAD_SECS + 1)),
            "from the future",
        );
        refused(
            &asked(&key(2), "demiurge:1761177600:short"),
            "not one QOR ID issues",
        );
        refused(
            &asked(&key(2), &format!("other:{NOW}:{}", "ab".repeat(32))),
            "not one QOR ID issues",
        );
    }

    #[test]
    fn what_an_agent_asks_for_is_held_to_what_qor_id_takes() {
        let base = asked(&key(2), &challenge(NOW));
        let with = |field: &str, value: serde_json::Value| {
            let mut request = base.clone();
            request[field] = value;
            request
        };
        refused(
            &with("capabilities", serde_json::json!(["read", "mint"])),
            "is not something",
        );
        refused(
            &with("capabilities", serde_json::json!(["read", "read"])),
            "named twice",
        );
        refused(
            &with("capabilities", serde_json::json!([])),
            "allowed nothing",
        );
        refused(&with("autonomy", "unbounded".into()), "autonomy");
        refused(
            &with("spending_limit", serde_json::Value::Null),
            "must name its spending limit",
        );
        refused(&with("spending_limit", (-1).into()), "cannot be negative");
        refused(&with("name", "Research helper".into()), "3 to 14");
        refused(&with("name", "help!".into()), "3 to 14");
        refused(
            &with("model", "m\nautonomy:sovereign".into()),
            "one short line",
        );
        refused(
            &with("model", "m".repeat(MODEL_LIMIT + 1).into()),
            "one short line",
        );
        let mut extra = base.clone();
        extra["controller"] = "0x00".into();
        refused(&extra, "malformed");
    }

    #[test]
    fn a_link_is_one_request_and_nothing_else() {
        let request = asked(&key(2), &challenge(NOW));
        assert!(parse(&format!("{}&r=00", link(&request)), NOW).is_err());
        assert!(parse(&format!("{}&s=00", link(&request)), NOW).is_err());
        assert!(parse("qor://pay?r=00", NOW).is_err());
        assert!(parse("qor://agent?r=zz", NOW).is_err());
        assert!(parse(
            &format!("qor://agent/?r={}", hex::encode(request.to_string())),
            NOW
        )
        .is_ok());
    }

    /// The dialog names the QOR ID it acts for, the agent's account and everything it may do, and says nothing
    /// enforces it yet.
    #[test]
    fn the_dialog_says_what_is_being_allowed() {
        let request = parse(&link(&asked(&key(2), &challenge(NOW))), NOW).unwrap();
        let prompt = request.prompt("andrew");
        assert!(prompt
            .body
            .contains("your QOR ID andrew, as agent_lab_assistant"));
        assert!(prompt.body.contains(&request.address));
        assert!(prompt.body.contains("It may: trade, read"));
        assert!(prompt
            .body
            .contains("Autonomy: bounded, spending at most 25 CGT"));
        assert!(prompt.body.contains("Model: claude-opus-5-5"));
        assert!(prompt.body.contains("nothing enforces them yet"));
        assert!(!prompt.body.contains("  "));
    }
}
