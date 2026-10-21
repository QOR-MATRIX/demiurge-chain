//! A stand-in product for the product host's tests (`src/host/tests.rs`): started by `host::launch` exactly as QQ
//! Studio is, it reads its token from stdin, connects to the channel it was named, says hello and asks one thing.
//!
//! `--mode good` (the token as given), `wrong-token`, `wrong-product`, or `late` (waits two seconds before connecting).
//! Exit code: 0 its request was answered with what it sent; 2 refused at the handshake; 3 anything else unexpected;
//! 4 could not connect at all.
//!
//! `--mode publish --project <folder>` asks as QQ Studio does: commit the project, then publish it as its Cartridge.
//! Exit code: 10 minted, 11 revised, 12 unchanged, 13 declined, 14 refused for another reason.
//!
//! `--mode motion` says, by its exit code, what the launcher passed it as `QOR_REDUCED_MOTION` (ADR-108 decision 5):
//! 20 `1`, 21 `0`, 22 nothing, 23 anything else.

use qor_launcher_lib::host::frame::{read_frame, write_frame};
use serde_json::{json, Value};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    std::process::exit(run().await);
}

async fn run() -> i32 {
    let args: Vec<String> = std::env::args().collect();
    let value = |flag: &str| {
        args.iter()
            .position(|a| a == flag)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_default()
    };
    let mode = value("--mode");
    let channel = value("--launcher-channel");

    let mut token = String::new();
    if std::io::stdin().read_line(&mut token).is_err() {
        return 3;
    }
    let token = token.trim().to_string();

    if mode == "motion" {
        return match std::env::var("QOR_REDUCED_MOTION").as_deref() {
            Ok("1") => 20,
            Ok("0") => 21,
            Err(std::env::VarError::NotPresent) => 22,
            _ => 23,
        };
    }

    if mode == "late" {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    #[cfg(windows)]
    let stream = tokio::net::windows::named_pipe::ClientOptions::new().open(&channel);
    #[cfg(unix)]
    let stream = tokio::net::UnixStream::connect(&channel).await;
    let Ok(mut stream) = stream else { return 4 };

    let hello = json!({
        "type": "hello",
        "protocol": 1,
        "min_protocol": 1,
        "product": if mode == "wrong-product" { "someone-else" } else { "fake-product" },
        "version": "0.0.1",
        "token": if mode == "wrong-token" { "00".repeat(32) } else { token },
    });
    if write_frame(&mut stream, hello.to_string().as_bytes())
        .await
        .is_err()
    {
        return 3;
    }
    let Ok(Some(answer)) = read_frame(&mut stream).await else {
        return 3;
    };
    let answer: Value = serde_json::from_slice(&answer).unwrap_or_default();
    if answer["type"] != "welcome" {
        return 2;
    }

    if mode == "publish" {
        return publish(&mut stream, &value("--project")).await;
    }

    let request =
        json!({ "type": "request", "id": 7, "method": "test.echo", "params": { "said": "hello" } });
    if write_frame(&mut stream, request.to_string().as_bytes())
        .await
        .is_err()
    {
        return 3;
    }
    let Ok(Some(answer)) = read_frame(&mut stream).await else {
        return 3;
    };
    let answer: Value = serde_json::from_slice(&answer).unwrap_or_default();
    if answer["type"] == "result" && answer["id"] == 7 && answer["result"]["said"] == "hello" {
        0
    } else {
        3
    }
}

/// Ask one request and read its answer.
async fn ask<S>(stream: &mut S, id: u64, method: &str, params: Value) -> Option<Value>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let request = json!({ "type": "request", "id": id, "method": method, "params": params });
    // Said on stderr, which the launcher keeps the end of, so a failing test shows where it stopped.
    if let Err(e) = write_frame(stream, request.to_string().as_bytes()).await {
        eprintln!("{method}: could not send: {e}");
        return None;
    }
    match read_frame(stream).await {
        Ok(Some(answer)) => {
            let answer: Value = serde_json::from_slice(&answer).ok()?;
            if answer["type"] == "error" {
                eprintln!("{method}: {answer}");
            }
            Some(answer)
        }
        Ok(None) => {
            eprintln!("{method}: the launcher closed the channel");
            None
        }
        Err(e) => {
            eprintln!("{method}: could not read the answer: {e}");
            None
        }
    }
}

async fn publish<S>(stream: &mut S, project: &str) -> i32
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let Some(committed) = ask(
        stream,
        1,
        "qontrol.commit",
        json!({ "project": project, "message": "Publish" }),
    )
    .await
    else {
        return 3;
    };
    if committed["type"] != "result" {
        return 14;
    }
    let Some(answer) = ask(
        stream,
        2,
        "cartridge.publish",
        json!({ "project": project }),
    )
    .await
    else {
        return 3;
    };
    match (
        answer["type"].as_str(),
        answer["result"]["action"].as_str(),
        answer["code"].as_str(),
    ) {
        (Some("result"), Some("minted"), _) => 10,
        (Some("result"), Some("revised"), _) => 11,
        (Some("result"), Some("unchanged"), _) => 12,
        (Some("error"), _, Some("declined")) => 13,
        _ => 14,
    }
}
