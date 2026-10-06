//! `qor://pay`: a website asks the launcher to pay (ADR-076, ADR-077).
//!
//! A link is `qor://pay?r=<hex of the request's JSON>&s=<hex of an Ed25519 signature over those bytes>`. The
//! signature is the app's: the launcher knows each app's public key (`KNOWN_APPS`), so a link cannot claim to be from
//! an app it is not. What the link asks is checked here, before any dialog, and nothing in it is trusted until it is:
//!
//! - signed by a known app, over exactly the bytes it carries;
//! - Demiurge Devnet, by genesis hash (ADR-077 decision 3; the chain's own genesis is checked again before signing);
//! - a whole number of Sparks above zero and at most the owner's cap of 100,000 CGT (ADR-076 decision 6);
//! - a valid account to pay, a short label, an id, and not expired, nor valid for longer than fifteen minutes;
//! - not already paid (`PaidRequests`, ADR-076 decision 7).
//!
//! Only then is the person asked, in the host dialog, with values built here from the checked request (ADR-077 decision
//! 1). This module does no network work, so every refusal is tested without a chain.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sp_core::crypto::Pair as _;
use sp_core::ed25519;

use crate::error::{QorError, QorResult};
use crate::vault::canonical_address;

/// Demiurge Devnet's genesis (ADR-068). A request for any other chain is refused.
pub const DEVNET_GENESIS: &str =
    "0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a";

/// The most one request may ask: 100,000 CGT in Sparks (18 decimals), the owner's cap for the devnet's test CGT
/// (ADR-076 decision 6).
pub const CAP_SPARKS: u128 = 100_000 * 1_000_000_000_000_000_000;

/// The longest a request may be valid for, from now.
const LONGEST_VALIDITY_SECS: i64 = 15 * 60;

/// The longest label shown in the dialog.
const LABEL_LIMIT: usize = 80;

/// An app allowed to ask for a payment, and the key its requests are signed with.
pub struct KnownApp {
    pub id: &'static str,
    pub name: &'static str,
    /// Ed25519 public key, hex. Its private half is held by the app's server only.
    pub key: &'static str,
}

/// The apps this launcher accepts payment requests from.
pub const KNOWN_APPS: &[KnownApp] = &[KnownApp {
    id: "arqade",
    name: "ARQADE",
    key: "b64304e720b90adcf09ed35be7837300fed8db980626097abe9350ecd0e1fd7f",
}];

/// A request as the app signed it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Signed {
    v: u8,
    app: String,
    id: String,
    to: String,
    /// Sparks, as a decimal string: a JSON number cannot hold every u128.
    amount: String,
    label: String,
    genesis: String,
    /// Expiry, Unix seconds.
    exp: i64,
}

/// A request that passed every check, ready to be shown and paid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayRequest {
    pub app_id: &'static str,
    pub app_name: &'static str,
    pub id: String,
    /// The account to pay, canonical SS58.
    pub to: String,
    pub amount_sparks: u128,
    pub label: String,
    pub expires: i64,
}

impl PayRequest {
    /// The remark carried with the payment, by which the app finds it in finalised blocks (ADR-076 decision 5).
    pub fn remark(&self) -> String {
        format!("qor-pay:{}:{}", self.app_id, self.id)
    }
}

fn refuse(why: impl Into<String>) -> QorError {
    QorError::PaymentRefused(why.into())
}

/// Read and check a `qor://pay` link. `now` is Unix seconds.
pub fn parse(link: &str, now: i64) -> QorResult<PayRequest> {
    parse_from(link, now, KNOWN_APPS)
}

fn parse_from(link: &str, now: i64, apps: &[KnownApp]) -> QorResult<PayRequest> {
    let rest = link
        .strip_prefix("qor://pay?")
        .or_else(|| link.strip_prefix("qor://pay/?"))
        .ok_or_else(|| refuse("this is not a payment link"))?;
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
    let signature = hex::decode(
        query
            .get("s")
            .ok_or_else(|| refuse("the link is not signed"))?,
    )
    .map_err(|_| refuse("the link is malformed"))?;
    if query.len() != 2 || bytes.len() > 2048 {
        return Err(refuse("the link is malformed"));
    }

    let signed: Signed =
        serde_json::from_slice(&bytes).map_err(|_| refuse("the request is malformed"))?;
    let app = apps
        .iter()
        .find(|app| app.id == signed.app)
        .ok_or_else(|| refuse("an unknown website asked you to pay. Nothing was paid"))?;
    let key: [u8; 32] = hex::decode(app.key)
        .ok()
        .and_then(|k| k.try_into().ok())
        .ok_or_else(|| refuse("this launcher's key for the app is damaged"))?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| refuse("the request's signature is malformed"))?;
    if !ed25519::Pair::verify(
        &ed25519::Signature::from_raw(signature),
        &bytes,
        &ed25519::Public::from_raw(key),
    ) {
        return Err(refuse(format!(
            "this request does not come from {}. Nothing was paid",
            app.name
        )));
    }

    if signed.v != 1 {
        return Err(refuse(
            "this request is of a version this launcher does not know",
        ));
    }
    if !signed.genesis.eq_ignore_ascii_case(DEVNET_GENESIS) {
        return Err(refuse(
            "payments from websites are taken on Demiurge Devnet only",
        ));
    }
    if signed.exp <= now {
        return Err(refuse(
            "this request has expired. Ask the website for a new one",
        ));
    }
    if signed.exp > now + LONGEST_VALIDITY_SECS {
        return Err(refuse("this request is valid for too long"));
    }
    let id_ok = (8..=64).contains(&signed.id.len())
        && signed
            .id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-');
    if !id_ok {
        return Err(refuse("the request's id is malformed"));
    }
    if signed.amount.is_empty()
        || signed.amount.len() > 39
        || !signed.amount.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(refuse("the amount is not a whole number of Sparks"));
    }
    let amount_sparks: u128 = signed
        .amount
        .parse()
        .map_err(|_| refuse("the amount is not a whole number of Sparks"))?;
    if amount_sparks == 0 {
        return Err(refuse("the amount is zero"));
    }
    if amount_sparks > CAP_SPARKS {
        return Err(refuse(format!(
            "the amount is above the most one request may ask, {} {}",
            crate::cgt::format_cgt_grouped(CAP_SPARKS),
            crate::cgt::SYMBOL
        )));
    }
    let label = signed.label.trim();
    if label.is_empty()
        || label.chars().count() > LABEL_LIMIT
        || label.chars().any(|c| c.is_control())
    {
        return Err(refuse("the request's description is malformed"));
    }
    let to =
        canonical_address(&signed.to).map_err(|_| refuse("the account to pay is not valid"))?;

    Ok(PayRequest {
        app_id: app.id,
        app_name: app.name,
        id: signed.id,
        to,
        amount_sparks,
        label: label.to_string(),
        expires: signed.exp,
    })
}

/// The ids already paid, per app, so a link pays at most once (ADR-076 decision 7). Kept in the app data directory;
/// an entry is kept for a day, far longer than any request is valid.
pub struct PaidRequests {
    path: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
struct PaidFile {
    /// `app:id` → when it was paid, Unix seconds.
    paid: BTreeMap<String, i64>,
}

impl PaidRequests {
    pub fn in_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            path: dir.as_ref().join("paid-requests.json"),
        }
    }

    fn read(&self) -> PaidFile {
        std::fs::read(&self.path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    pub fn is_paid(&self, request: &PayRequest) -> bool {
        self.read().paid.contains_key(&key(request))
    }

    /// Record a payment. A failure to write is reported; the chain still holds the payment and the app finds it
    /// there, so the worst outcome is that a second click is refused by the app rather than here.
    pub fn record(&self, request: &PayRequest, now: i64) -> QorResult<()> {
        let mut file = self.read();
        file.paid.retain(|_, at| *at > now - 24 * 60 * 60);
        file.paid.insert(key(request), now);
        let bytes = serde_json::to_vec_pretty(&file)
            .map_err(|e| QorError::Io(format!("the paid list could not be written: {e}")))?;
        std::fs::write(&self.path, bytes)
            .map_err(|e| QorError::Io(format!("the paid list could not be written: {e}")))
    }
}

fn key(request: &PayRequest) -> String {
    format!("{}:{}", request.app_id, request.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_800_000_000;
    // Alice's well-known dev account, a valid SS58 address at the generic prefix.
    const ALICE: &str = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";

    fn request(fields: serde_json::Value) -> Vec<u8> {
        let mut base = serde_json::json!({
            "v": 1, "app": "arqade", "id": "0123456789abcdef", "to": ALICE,
            "amount": "5000000000000000000", "label": "Tip for Flux Four",
            "genesis": DEVNET_GENESIS, "exp": NOW + 300,
        });
        for (k, v) in fields.as_object().expect("object") {
            base[k] = v.clone();
        }
        serde_json::to_vec(&base).expect("json")
    }

    fn link(bytes: &[u8], pair: &ed25519::Pair) -> String {
        format!(
            "qor://pay?r={}&s={}",
            hex::encode(bytes),
            hex::encode(pair.sign(bytes).0)
        )
    }

    /// A key standing in for ARQADE's, and the launcher's list rewritten to trust it, for tests only.
    fn signer() -> ed25519::Pair {
        ed25519::Pair::from_seed(&[7u8; 32])
    }

    /// The launcher's list with ARQADE's key replaced by the test key: everything else is the real check.
    fn parse_with(link: &str) -> QorResult<PayRequest> {
        let key: &'static str = Box::leak(hex::encode(signer().public().0).into_boxed_str());
        parse_from(
            link,
            NOW,
            &[KnownApp {
                id: "arqade",
                name: "ARQADE",
                key,
            }],
        )
    }

    #[test]
    fn a_signed_request_within_every_limit_is_accepted() {
        let paid = parse_with(&link(&request(serde_json::json!({})), &signer())).expect("accepted");
        assert_eq!(paid.app_name, "ARQADE");
        assert_eq!(paid.amount_sparks, 5 * 10u128.pow(18));
        assert_eq!(paid.to, canonical_address(ALICE).unwrap());
        assert_eq!(paid.remark(), "qor-pay:arqade:0123456789abcdef");
    }

    /// A link made by ARQADE's own code (`products/arqade/lib/pay.ts`, `payLink`) with this test key, pinned here so
    /// the two cannot drift apart: `products/arqade/tests/pay.test.mjs` makes the same link and checks it is this one.
    const ARQADE_LINK: &str = "qor://pay?r=7b2276223a312c22617070223a22617271616465222c226964223a223031323334353637383961626364656630313233343536373839616263646566222c22746f223a223547727776614546357a58623236467a397263517044575335374374455248704e6568584350634e6f48474b75745159222c22616d6f756e74223a2235303030303030303030303030303030303030222c226c6162656c223a2254697020666f7220466c757820466f7572222c2267656e65736973223a22307839333465326361613336666261353438656535663531313935633264303239303937666262613034303065366538303563613866336530373934376132353461222c22657870223a313830303030303330307d&s=29b04050ade965c33165c5c5a6258e33bb91f3ed45d51b20607fcec8d0349e06b115818e778c7acfdfbbc1073748fb6bba746fed022ab4f2d86077c1f809880d";

    #[test]
    fn a_link_made_by_arqades_code_is_accepted() {
        let paid = parse_with(ARQADE_LINK).expect("ARQADE's link");
        assert_eq!(paid.id, "0123456789abcdef0123456789abcdef");
        assert_eq!(paid.amount_sparks, 5 * 10u128.pow(18));
        assert_eq!(paid.label, "Tip for Flux Four");
    }

    #[test]
    fn a_forged_or_altered_request_is_refused() {
        let bytes = request(serde_json::json!({}));
        let forger = ed25519::Pair::from_seed(&[9u8; 32]);
        assert!(parse_with(&link(&bytes, &forger)).is_err(), "another key");
        let signed = link(&bytes, &signer());
        let altered = request(serde_json::json!({ "amount": "6000000000000000000" }));
        let swapped = signed.replace(&hex::encode(&bytes), &hex::encode(&altered));
        assert!(
            parse_with(&swapped).is_err(),
            "the bytes changed after signing"
        );
        let unknown = request(serde_json::json!({ "app": "elsewhere" }));
        let err = parse_with(&link(&unknown, &signer()))
            .unwrap_err()
            .to_string();
        assert!(err.contains("unknown website"), "{err}");
    }

    #[test]
    fn every_limit_is_enforced() {
        for (fields, why) in [
            (serde_json::json!({ "genesis": "0x00" }), "another chain"),
            (serde_json::json!({ "exp": NOW }), "expired"),
            (
                serde_json::json!({ "exp": NOW + 16 * 60 }),
                "valid too long",
            ),
            (serde_json::json!({ "amount": "0" }), "zero"),
            (serde_json::json!({ "amount": "1.5" }), "not whole Sparks"),
            (
                serde_json::json!({ "amount": (CAP_SPARKS + 1).to_string() }),
                "above the cap",
            ),
            (
                serde_json::json!({ "to": "not an address" }),
                "a bad account",
            ),
            (serde_json::json!({ "label": "" }), "no label"),
            (
                serde_json::json!({ "label": "x".repeat(81) }),
                "a long label",
            ),
            (
                serde_json::json!({ "label": "line\nbreak" }),
                "a control character",
            ),
            (serde_json::json!({ "id": "short" }), "a short id"),
            (serde_json::json!({ "v": 2 }), "another version"),
        ] {
            assert!(
                parse_with(&link(&request(fields), &signer())).is_err(),
                "{why} must be refused"
            );
        }
        let at_cap = request(serde_json::json!({ "amount": CAP_SPARKS.to_string() }));
        assert!(
            parse_with(&link(&at_cap, &signer())).is_ok(),
            "the cap itself is allowed"
        );
    }

    #[test]
    fn a_malformed_link_is_refused() {
        for bad in [
            "qor://other?r=00&s=00",
            "qor://pay?r=zz&s=00",
            "qor://pay?r=00",
            "qor://pay?r=00&s=00&x=1",
            "qor://pay?r=00&r=00&s=00",
        ] {
            assert!(parse_with(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_shipped_key_is_arqades_and_well_formed() {
        let app = KNOWN_APPS
            .iter()
            .find(|a| a.id == "arqade")
            .expect("arqade");
        assert_eq!(hex::decode(app.key).expect("hex").len(), 32);
    }

    #[test]
    fn a_request_is_paid_once() {
        let dir = tempfile::tempdir().expect("dir");
        let paid = PaidRequests::in_dir(dir.path());
        let request = parse_with(&link(&request(serde_json::json!({})), &signer())).unwrap();
        assert!(!paid.is_paid(&request));
        paid.record(&request, NOW).expect("recorded");
        assert!(paid.is_paid(&request));
        assert!(
            PaidRequests::in_dir(dir.path()).is_paid(&request),
            "survives a restart"
        );
    }
}
