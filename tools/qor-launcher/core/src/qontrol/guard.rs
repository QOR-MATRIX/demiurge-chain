//! The guard before staging.
//!
//! Qontrol stages on commit: the creator types a sentence and presses Commit,
//! and is not reading a list of files first. That is the right interface, and
//! it is also exactly how a 40 GB render or a private key enters history — and
//! history is the one place a file cannot be taken back out of once anyone has
//! a copy. So before anything is staged, the files that would be are read, and
//! two kinds are held back until the person says yes to each by name:
//!
//! - **large**: over [`LARGE`] bytes on disk;
//! - **credential**: a file whose name is a known place secrets live, or whose
//!   text holds something shaped like a private key or an access token.
//!
//! The list of files is the helper's own (`sidecar::plan`), so what is checked is
//! chosen by the same ignore rules that choose what is committed. A second
//! ignore implementation here would be a second set of answers.
//!
//! A warning names the file and the reason. It never carries the text that
//! matched: this list is drawn on screen and may be written to a log, and a
//! guard that copied the secret it found would be the leak it exists to stop.
//!
//! This is a warning, not a scanner that promises to find every secret. A
//! credential in an unusual shape passes it; one in a known shape does not
//! pass unseen.

use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{sidecar, QontrolError};

/// 50 MiB. Where GitHub starts warning about a file (it refuses one at
/// 100 MiB), and a size at which a file in history is paid for by everyone who
/// ever clones the project. A warning threshold, not a limit: Phase 4 (P1.5) is
/// where large files get a store of their own.
pub const LARGE: u64 = 50 * 1024 * 1024;

/// How much of a file is read for credential shapes. Keys and tokens sit in
/// configuration files, which are small; a stem is not read to the end.
const SCAN: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Concern {
    Large,
    Credential,
}

/// One file held back, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Warning {
    pub path: String,
    pub concern: Concern,
    /// Plain words for the person: "looks like a private key", "72 MiB".
    pub reason: String,
}

/// The person's yes to one warning, sent back with the commit.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Accepted {
    pub path: String,
    pub concern: Concern,
}

impl Warning {
    fn accepted_by(&self, accepted: &[Accepted]) -> bool {
        accepted
            .iter()
            .any(|a| a.path == self.path && a.concern == self.concern)
    }
}

/// Read what a commit of `files` (or of everything) would stage, and warn.
pub fn check(repo_path: &Path, files: Option<&[String]>) -> Result<Vec<Warning>, QontrolError> {
    let repo = repo_path.to_string_lossy().to_string();
    let planned = sidecar::plan(&repo, files)?;

    let mut warnings = Vec::new();
    for rela in planned {
        let full = repo_path.join(&rela);
        // Not followed: git stores a link's target path, not what it points at.
        let Ok(meta) = std::fs::symlink_metadata(&full) else {
            continue;
        };
        if !meta.is_file() {
            continue;
        }

        if meta.len() > LARGE {
            warnings.push(Warning {
                path: rela.clone(),
                concern: Concern::Large,
                reason: format!("{} MiB", meta.len() / (1024 * 1024)),
            });
        }

        let name = rela.rsplit('/').next().unwrap_or(&rela);
        let credential = credential_name(name).or_else(|| credential_content(&full));
        if let Some(reason) = credential {
            warnings.push(Warning {
                path: rela,
                concern: Concern::Credential,
                reason: reason.into(),
            });
        }
    }
    Ok(warnings)
}

/// Refuse a commit while any warning is unanswered. An answer names the file
/// and the concern, so a yes to "72 MiB" is not a yes to "looks like a key".
pub fn require_accepted(warnings: &[Warning], accepted: &[Accepted]) -> Result<(), QontrolError> {
    let open: Vec<&Warning> = warnings
        .iter()
        .filter(|w| !w.accepted_by(accepted))
        .collect();
    if open.is_empty() {
        return Ok(());
    }
    let names: Vec<String> = open
        .iter()
        .map(|w| format!("{} ({})", w.path, w.reason))
        .collect();
    Err(QontrolError::Unconfirmed(names.join(", ")))
}

/// A file named for where secrets are kept.
fn credential_name(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    match lower.as_str() {
        ".env" => return Some("an environment file, where secrets are usually kept"),
        "id_rsa" | "id_dsa" | "id_ecdsa" | "id_ed25519" => return Some("an SSH private key"),
        ".netrc" | "_netrc" | ".git-credentials" | ".pgpass" => {
            return Some("a file that stores passwords")
        }
        "credentials" | "credentials.json" => return Some("a credentials file"),
        _ => {}
    }
    if let Some(rest) = lower.strip_prefix(".env.") {
        // `.env.example` and its kin are the templates people are told to
        // commit instead of the real thing.
        if !matches!(rest, "example" | "sample" | "template" | "dist") {
            return Some("an environment file, where secrets are usually kept");
        }
    }
    // `.key` and `.pem` are not here: a Keynote deck is a `.key`. A private key
    // in either is caught by its content, whatever it is called.
    let extension = lower.rsplit_once('.').map(|(_, ext)| ext);
    match extension {
        Some("p12" | "pfx") => Some("a certificate bundle, which usually holds a private key"),
        Some("jks" | "keystore") => Some("a key store"),
        Some("ppk") => Some("a PuTTY private key"),
        _ => None,
    }
}

/// Text shaped like a private key or an access token, in the first [`SCAN`]
/// bytes. A file with a NUL byte in what was read is binary and is not scanned.
fn credential_content(path: &Path) -> Option<&'static str> {
    let file = std::fs::File::open(path).ok()?;
    let mut bytes = Vec::new();
    file.take(SCAN).read_to_end(&mut bytes).ok()?;
    if bytes.contains(&0) {
        return None;
    }
    credential_shape(&bytes)
}

/// A token's shape: a prefix, the bytes that may follow it, how many at least,
/// and what to tell the person.
type Shape = (&'static [u8], fn(u8) -> bool, usize, &'static str);

pub(crate) fn credential_shape(text: &[u8]) -> Option<&'static str> {
    if private_key_block(text) {
        return Some("looks like a private key");
    }
    const ALNUM: fn(u8) -> bool = |b| b.is_ascii_alphanumeric();
    const UPPER_DIGIT: fn(u8) -> bool = |b| b.is_ascii_uppercase() || b.is_ascii_digit();
    const TOKEN: fn(u8) -> bool = |b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-';
    let shapes: [Shape; 13] = [
        (b"AKIA", UPPER_DIGIT, 16, "looks like an AWS access key"),
        (b"ghp_", ALNUM, 36, "looks like a GitHub token"),
        (b"gho_", ALNUM, 36, "looks like a GitHub token"),
        (b"ghu_", ALNUM, 36, "looks like a GitHub token"),
        (b"ghs_", ALNUM, 36, "looks like a GitHub token"),
        (b"github_pat_", TOKEN, 22, "looks like a GitHub token"),
        (b"xoxb-", TOKEN, 10, "looks like a Slack token"),
        (b"xoxp-", TOKEN, 10, "looks like a Slack token"),
        (b"sk_live_", ALNUM, 16, "looks like a Stripe secret key"),
        (b"rk_live_", ALNUM, 16, "looks like a Stripe secret key"),
        (b"AIza", TOKEN, 35, "looks like a Google API key"),
        (b"sk-ant-", TOKEN, 32, "looks like an Anthropic API key"),
        (b"sk-proj-", TOKEN, 32, "looks like an OpenAI API key"),
    ];
    shapes
        .iter()
        .find(|(prefix, allowed, least, _)| token(text, prefix, *allowed, *least))
        .map(|(_, _, _, reason)| *reason)
}

/// `-----BEGIN ... PRIVATE KEY-----` on one line: RSA, EC, OpenSSH, PKCS#8,
/// encrypted or not.
fn private_key_block(text: &[u8]) -> bool {
    const BEGIN: &[u8] = b"-----BEGIN ";
    const END: &[u8] = b"PRIVATE KEY-----";
    let mut rest = text;
    while let Some(at) = find(rest, BEGIN) {
        let line = &rest[at..];
        let line = &line[..line.iter().position(|&b| b == b'\n').unwrap_or(line.len())];
        if find(line, END).is_some() {
            return true;
        }
        rest = &rest[at + BEGIN.len()..];
    }
    false
}

/// `prefix` at a word boundary, followed by at least `least` allowed bytes.
fn token(text: &[u8], prefix: &[u8], allowed: fn(u8) -> bool, least: usize) -> bool {
    let mut start = 0;
    while let Some(at) = find(&text[start..], prefix) {
        let at = start + at;
        let boundary = at == 0 || !text[at - 1].is_ascii_alphanumeric();
        let run = text[at + prefix.len()..]
            .iter()
            .take_while(|&&b| allowed(b))
            .count();
        if boundary && run >= least {
            return true;
        }
        start = at + 1;
    }
    false
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
