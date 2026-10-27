//! The client half of the `qontrol-git` sidecar.
//!
//! The sidecar does the two things gitoxide cannot — stage the working tree and
//! write a tree object from the index — in a process that holds no keys. See
//! `tools/qor-launcher/qontrol-git/src/main.rs` for why that separation exists.
//!
//! One request per line in, one response per line out. The process is started
//! for a call and stopped when the call ends: a staging run is milliseconds, and
//! a long-lived child would be one more thing to supervise for no gain.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Deserialize;

use super::QontrolError;

/// Where the helper binary is.
///
/// `QONTROL_GIT_BIN` wins, for a packager who puts it somewhere else. Then a
/// sibling of the launcher executable, which is where the installer puts it.
/// Then the development build — which is what makes `cargo test` work from a
/// clean checkout with nothing set, and is why the tests need no setup at all.
fn binary() -> Result<PathBuf, QontrolError> {
    if let Ok(explicit) = std::env::var("QONTROL_GIT_BIN") {
        let path = PathBuf::from(explicit);
        if path.exists() {
            return Ok(path);
        }
        return Err(QontrolError::Sidecar(format!(
            "QONTROL_GIT_BIN points at {}, which does not exist",
            path.display()
        )));
    }

    let name = if cfg!(windows) {
        "qontrol-git.exe"
    } else {
        "qontrol-git"
    };

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside = dir.join(name);
            if beside.exists() {
                return Ok(beside);
            }
        }
    }

    // The development layout: tools/qor-launcher/qontrol-git/target/<profile>/.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for profile in ["debug", "release"] {
        let candidate = manifest
            .join("..")
            .join("qontrol-git")
            .join("target")
            .join(profile)
            .join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(QontrolError::Sidecar(
        "the qontrol-git helper was not found. Build it with \
         `cargo build --manifest-path ../qontrol-git/Cargo.toml`, or set QONTROL_GIT_BIN"
            .into(),
    ))
}

#[derive(Debug, Deserialize)]
struct Envelope {
    ok: bool,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    staged: Option<usize>,
    #[serde(default)]
    tree: Option<String>,
    #[serde(default)]
    paths: Option<Vec<String>>,
}

fn call(request: &str) -> Result<Envelope, QontrolError> {
    let binary = binary()?;

    let mut child = Command::new(&binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| QontrolError::Sidecar(format!("could not start {}: {e}", binary.display())))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| QontrolError::Sidecar("the helper has no stdin".into()))?;
        writeln!(stdin, "{request}")
            .map_err(|e| QontrolError::Sidecar(format!("could not write to the helper: {e}")))?;
    }
    // Dropping stdin tells the helper to finish and exit.
    drop(child.stdin.take());

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| QontrolError::Sidecar("the helper has no stdout".into()))?;
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .map_err(|e| QontrolError::Sidecar(format!("could not read the helper's answer: {e}")))?;

    let _ = child.wait();

    if line.trim().is_empty() {
        return Err(QontrolError::Sidecar(
            "the helper exited without answering".into(),
        ));
    }

    let envelope: Envelope = serde_json::from_str(line.trim()).map_err(|e| {
        QontrolError::Sidecar(format!("the helper's answer could not be read: {e}"))
    })?;

    if !envelope.ok {
        return Err(QontrolError::Sidecar(
            envelope
                .error
                .unwrap_or_else(|| "the helper refused without saying why".into()),
        ));
    }

    Ok(envelope)
}

fn escape(path: &str) -> String {
    serde_json::to_string(path).unwrap_or_else(|_| "\"\"".into())
}

/// Stage every change, honouring the repository's own ignore rules and
/// attributes. Returns how many entries the index holds afterwards.
pub fn stage_all(repo: &str) -> Result<usize, QontrolError> {
    let response = call(&format!(r#"{{"op":"stage_all","repo":{}}}"#, escape(repo)))?;
    response
        .staged
        .ok_or_else(|| QontrolError::Sidecar("the helper did not say how much it staged".into()))
}

fn list(paths: &[String]) -> String {
    serde_json::to_string(paths).unwrap_or_else(|_| "[]".into())
}

/// Stage only the named paths, matched literally. Returns how many entries the
/// index holds afterwards.
pub fn stage(repo: &str, paths: &[String]) -> Result<usize, QontrolError> {
    let response = call(&format!(
        r#"{{"op":"stage","repo":{},"paths":{}}}"#,
        escape(repo),
        list(paths)
    ))?;
    response
        .staged
        .ok_or_else(|| QontrolError::Sidecar("the helper did not say how much it staged".into()))
}

/// The files a commit of `paths` (or of everything, for `None`) would stage,
/// by the helper's own ignore rules, with nothing staged.
pub fn plan(repo: &str, paths: Option<&[String]>) -> Result<Vec<String>, QontrolError> {
    let paths = paths.map_or_else(|| "null".into(), list);
    let response = call(&format!(
        r#"{{"op":"plan","repo":{},"paths":{paths}}}"#,
        escape(repo)
    ))?;
    response
        .paths
        .ok_or_else(|| QontrolError::Sidecar("the helper did not list what it would stage".into()))
}

/// Check out a local branch. The helper refuses, touching nothing, when an
/// uncommitted change would be overwritten.
pub fn switch(repo: &str, branch: &str) -> Result<(), QontrolError> {
    call(&format!(
        r#"{{"op":"switch","repo":{},"branch":{}}}"#,
        escape(repo),
        escape(branch)
    ))
    .map(|_| ())
}

/// Put the named tracked files back as the index holds them.
pub fn discard(repo: &str, paths: &[String]) -> Result<(), QontrolError> {
    call(&format!(
        r#"{{"op":"discard","repo":{},"paths":{}}}"#,
        escape(repo),
        list(paths)
    ))
    .map(|_| ())
}

/// Write a tree object from the index and return its id.
pub fn write_tree(repo: &str) -> Result<String, QontrolError> {
    let response = call(&format!(r#"{{"op":"write_tree","repo":{}}}"#, escape(repo)))?;
    response
        .tree
        .ok_or_else(|| QontrolError::Sidecar("the helper did not return a tree".into()))
}

/// Is the helper there and answering? Used by the surface to say so plainly
/// rather than failing at the moment someone tries to commit.
pub fn available() -> bool {
    call(r#"{"op":"ping"}"#).is_ok()
}
