//! A test agent asks the QOR Launcher to let it act for the QOR ID signed in there (ADR-112, L5.1).
//!
//!   cargo run --example agent_link -- [--qor-id <QOR ID's /api/v1 address>] [--name <3 to 14 letters, digits, spaces>]
//!                                      [--capabilities read,trade] [--autonomy supervised] [--limit <whole CGT>]
//!                                      [--print]
//!
//! `--print` prints the link instead of opening it, for a PC whose launcher is not the one to ask.
//!
//! It does what any agent does: makes an Sr25519 key of its own, asks QOR ID for a challenge for that account, signs the
//! challenge, and opens the `qor://agent` link (`agents::request_link`). The launcher then checks the request and asks in
//! its own dialog; nothing is authorised unless the person presses Allow agent, and only the profile's own key can sign
//! for it. Defaults: the live QOR ID, a name of `Test` and four digits, allowed only to read, supervised.
//!
//! The agent's private key is never written down or shown: it lives in this process and is gone when it ends. That is
//! enough to see the dialog and authorise the agent end to end; an agent that will act afterwards keeps its own key.

use qor_launcher_lib::agents;
use sp_core::Pair as _;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(why) = run().await {
        eprintln!("agent_link: {why}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let qor_id = value("--qor-id").unwrap_or_else(|| "https://id.qorsync.dev/api/v1".into());
    let name = value("--name").unwrap_or_else(|| format!("Test {:04}", rand_digits()));
    let capabilities = value("--capabilities").unwrap_or_else(|| "read".into());
    let capabilities: Vec<&str> = capabilities.split(',').map(str::trim).collect();
    let autonomy = value("--autonomy").unwrap_or_else(|| "supervised".into());
    let limit = value("--limit")
        .map(|l| {
            l.parse::<i64>()
                .map_err(|_| format!("--limit {l} is not a whole number of CGT"))
        })
        .transpose()?;

    let (agent, _) = sp_core::sr25519::Pair::generate();
    let account_id = format!("0x{}", hex::encode(agent.public().0));
    let answer = reqwest::Client::new()
        .get(format!(
            "{}/auth/challenge?account_id={account_id}",
            qor_id.trim_end_matches('/')
        ))
        .send()
        .await
        .map_err(|e| format!("QOR ID at {qor_id} could not be reached: {e}"))?;
    if !answer.status().is_success() {
        return Err(format!(
            "QOR ID refused a challenge: {}",
            answer.text().await.unwrap_or_default()
        ));
    }
    let issued: serde_json::Value = answer
        .json()
        .await
        .map_err(|e| format!("QOR ID's challenge could not be read: {e}"))?;
    let challenge = issued["challenge"]
        .as_str()
        .ok_or("QOR ID sent no challenge")?;

    let link = agents::request_link(&agent, challenge, &name, &capabilities, &autonomy, limit);
    println!(
        "Agent \"{name}\" (agent_{}) asks to act for the QOR ID signed in to the launcher, allowed: {}.",
        name.to_lowercase().replace(' ', "_"),
        capabilities.join(", ")
    );
    if args.iter().any(|a| a == "--print") {
        println!("{link}");
        println!("Open it within five minutes, while QOR ID keeps the challenge.");
        return Ok(());
    }
    open(&link)?;
    println!(
        "The launcher shows its own dialog now. Allow or decline it there; the agent's key is gone when this ends."
    );
    Ok(())
}

/// Four digits for a default name, so each run asks for a QOR ID nobody holds yet.
fn rand_digits() -> u32 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or_default();
    nanos % 10_000
}

/// Hand the link to whatever the system opens `qor://` with: the installed launcher.
fn open(link: &str) -> Result<(), String> {
    #[cfg(windows)]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", "", link])
        .status();
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(link).status();
    #[cfg(all(unix, not(target_os = "macos")))]
    let status = std::process::Command::new("xdg-open").arg(link).status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => Err(format!(
            "the link could not be opened; open it yourself: {link}"
        )),
    }
}
