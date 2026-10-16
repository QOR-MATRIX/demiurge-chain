//! The product host's checks (ADR-046, ADR-088): frames, the handshake and its refusals, the dialog that names the
//! product, nothing secret in the log, the golden frames, and a real spawn of a stand-in product
//! (`examples/host_fake_product.rs`) through the platform's channel, including a connection from a process the
//! launcher did not start.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tokio::io::{AsyncWriteExt, DuplexStream};

use super::frame::{read_frame, write_frame, FRAME_LIMIT};
use super::protocol::{FromProduct, ToProduct};
use super::session::{serve, Ticket, TOKEN_LIFETIME};
use super::*;
use crate::testing::Scripted;

/// Echoes `test.echo`'s parameters back; offers nothing else.
struct Echo;

impl Handler for Echo {
    fn capabilities(&self) -> &'static [&'static str] {
        &["test.echo"]
    }

    async fn handle(&self, _method: &str, params: Value) -> Result<Value, Refusal> {
        Ok(params)
    }
}

fn record() -> ProductRecord {
    ProductRecord {
        id: "fake-product",
        name: "Fake Product",
        path: PathBuf::from("C:/Programs/fake-product.exe"),
    }
}

fn ticket() -> (Ticket, String) {
    let token = SpawnToken::new();
    let hex = token.hex().to_owned();
    (
        Ticket {
            product: record(),
            token,
            issued: Instant::now(),
        },
        hex,
    )
}

fn hello(token: &str) -> Value {
    json!({ "type": "hello", "protocol": 1, "min_protocol": 1, "product": "fake-product", "version": "1", "token": token })
}

async fn say(stream: &mut DuplexStream, frame: &Value) {
    write_frame(stream, frame.to_string().as_bytes())
        .await
        .expect("send");
}

async fn hear(stream: &mut DuplexStream) -> Option<Value> {
    read_frame(stream)
        .await
        .expect("read")
        .map(|frame| serde_json::from_slice(&frame).expect("JSON"))
}

/// Run a session against `product`, which plays the product's side; returns how the session ended.
async fn session<F, Fut>(ticket: Ticket, product: F) -> Result<(), HostError>
where
    F: FnOnce(DuplexStream) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let (launcher, theirs) = tokio::io::duplex(64 * 1024);
    let (ended, ()) = tokio::join!(serve(launcher, ticket, &Echo), product(theirs));
    ended
}

// ── Frames ────────────────────────────────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_frame_goes_and_comes_back_whole() {
    let (mut a, mut b) = tokio::io::duplex(1024);
    write_frame(&mut a, b"{\"type\":\"hello\"}").await.unwrap();
    assert_eq!(
        read_frame(&mut b).await.unwrap().unwrap(),
        b"{\"type\":\"hello\"}"
    );
    drop(a);
    assert!(
        read_frame(&mut b).await.unwrap().is_none(),
        "closed between frames is the end, not an error"
    );
}

/// A length over the limit closes the connection before anything is allocated for it.
#[tokio::test]
async fn a_frame_over_the_limit_is_refused_both_ways() {
    let (mut a, mut b) = tokio::io::duplex(1024);
    a.write_all(&u32::MAX.to_be_bytes()).await.unwrap();
    let refused = read_frame(&mut b).await.unwrap_err();
    assert_eq!(refused.kind(), std::io::ErrorKind::InvalidData);

    let (mut a, _b) = tokio::io::duplex(1024);
    assert!(write_frame(&mut a, &vec![b' '; FRAME_LIMIT + 1])
        .await
        .is_err());
}

// ── The handshake ─────────────────────────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn the_product_it_started_is_welcomed_and_its_requests_answered() {
    let (issued, token) = ticket();
    let ended = session(issued, |mut product| async move {
        say(&mut product, &hello(&token)).await;
        let welcome = hear(&mut product).await.expect("welcome");
        assert_eq!(welcome["type"], "welcome");
        assert_eq!(welcome["protocol"], 1);
        assert_eq!(welcome["capabilities"], json!(["test.echo"]));
        assert_eq!(welcome["session"].as_str().unwrap().len(), 32);

        say(
            &mut product,
            &json!({ "type": "request", "id": 1, "method": "test.echo", "params": { "a": 1 } }),
        )
        .await;
        assert_eq!(
            hear(&mut product).await.unwrap(),
            json!({ "type": "result", "id": 1, "result": { "a": 1 } })
        );

        // A method not offered is refused, and the session goes on.
        say(
            &mut product,
            &json!({ "type": "request", "id": 2, "method": "vault.sign", "params": {} }),
        )
        .await;
        let refused = hear(&mut product).await.unwrap();
        assert_eq!(refused["code"], "unsupported_method");
        assert_eq!(refused["id"], 2);
        say(
            &mut product,
            &json!({ "type": "request", "id": 3, "method": "test.echo" }),
        )
        .await;
        assert_eq!(hear(&mut product).await.unwrap()["type"], "result");
    })
    .await;
    assert!(ended.is_ok(), "{ended:?}");
}

#[tokio::test]
async fn a_wrong_token_is_refused_and_the_channel_closed() {
    let (issued, _) = ticket();
    let ended = session(issued, |mut product| async move {
        say(&mut product, &hello(&"00".repeat(32))).await;
        let refused = hear(&mut product).await.expect("a refusal");
        assert_eq!(refused["type"], "error");
        assert_eq!(refused["code"], "not_authorised");
        assert!(refused["id"].is_null());
        assert!(hear(&mut product).await.is_none(), "the channel closes");
    })
    .await;
    assert!(matches!(ended, Err(HostError::BadToken)), "{ended:?}");
}

#[tokio::test]
async fn a_token_used_after_its_lifetime_is_refused() {
    let (mut issued, token) = ticket();
    issued.issued = Instant::now() - TOKEN_LIFETIME - Duration::from_secs(1);
    let ended = session(issued, |mut product| async move {
        say(&mut product, &hello(&token)).await;
        let _ = hear(&mut product).await;
    })
    .await;
    assert!(
        matches!(ended, Err(HostError::Expired | HostError::BadToken)),
        "{ended:?}"
    );
}

#[tokio::test]
async fn another_products_hello_is_refused_even_with_the_token() {
    let (issued, token) = ticket();
    let ended = session(issued, |mut product| async move {
        let mut frame = hello(&token);
        frame["product"] = json!("someone-else");
        say(&mut product, &frame).await;
        assert_eq!(hear(&mut product).await.unwrap()["code"], "not_authorised");
    })
    .await;
    assert!(
        matches!(ended, Err(HostError::BadHandshake(_))),
        "{ended:?}"
    );
}

#[tokio::test]
async fn a_request_before_hello_and_a_second_hello_are_refused() {
    let (issued, _) = ticket();
    let ended = session(issued, |mut product| async move {
        say(
            &mut product,
            &json!({ "type": "request", "id": 1, "method": "test.echo" }),
        )
        .await;
        assert_eq!(hear(&mut product).await.unwrap()["code"], "bad_request");
    })
    .await;
    assert!(
        matches!(ended, Err(HostError::BadHandshake(_))),
        "{ended:?}"
    );

    let (issued, token) = ticket();
    let ended = session(issued, |mut product| async move {
        say(&mut product, &hello(&token)).await;
        let _welcome = hear(&mut product).await;
        say(&mut product, &hello(&token)).await;
        assert_eq!(hear(&mut product).await.unwrap()["code"], "bad_request");
    })
    .await;
    assert!(
        matches!(ended, Err(HostError::BadHandshake(_))),
        "{ended:?}"
    );
}

#[tokio::test]
async fn a_product_that_needs_a_later_protocol_is_told_which_this_speaks() {
    let (issued, token) = ticket();
    let ended = session(issued, |mut product| async move {
        let mut frame = hello(&token);
        frame["protocol"] = json!(3);
        frame["min_protocol"] = json!(2);
        say(&mut product, &frame).await;
        let refused = hear(&mut product).await.unwrap();
        assert_eq!(refused["code"], "protocol_mismatch");
        assert!(refused["message"].as_str().unwrap().contains("1 to 1"));
    })
    .await;
    assert!(
        matches!(ended, Err(HostError::BadHandshake(_))),
        "{ended:?}"
    );

    // A product newer than the launcher, which still speaks version 1, is welcomed at version 1.
    let (issued, token) = ticket();
    session(issued, |mut product| async move {
        let mut frame = hello(&token);
        frame["protocol"] = json!(4);
        say(&mut product, &frame).await;
        assert_eq!(hear(&mut product).await.unwrap()["protocol"], 1);
    })
    .await
    .expect("welcomed");
}

// ── The dialog names who is asking ────────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn a_products_dialog_says_which_product_asks_from_the_launchers_record() {
    let asked = Scripted::approving();
    let product = record();
    let dialog = ProductDialog {
        inner: &asked,
        product: &product,
    };
    let prompt = Prompt {
        title: "Approve this mint".into(),
        body: "Mint \"a-game\" as a DRC-369 asset".into(),
        approve: "Mint".into(),
    };
    assert!(dialog.confirm(&prompt).await);

    let shown = asked.asked.lock()[0].clone();
    assert_eq!(shown.title, "Fake Product asks: Approve this mint");
    assert!(
        shown
            .body
            .starts_with("Asked by Fake Product, which the launcher started from"),
        "{}",
        shown.body
    );
    assert!(shown.body.contains(&product.path.display().to_string()));
    assert!(shown.body.ends_with("Mint \"a-game\" as a DRC-369 asset"));
    assert_eq!(shown.approve, "Mint");

    let declining = Scripted::declining();
    let dialog = ProductDialog {
        inner: &declining,
        product: &product,
    };
    assert!(!dialog.confirm(&prompt).await, "a decline stays a decline");
}

// ── Nothing secret reaches a log (ADR-046 decision 10) ────────────────────────────────────────────────────────────

use crate::testing::captured_log;

/// A whole session, a good one and a refused one, with every level of the launcher's log on: neither the token, nor
/// the session id, nor the wrong token offered appears in it. The log is checked to have something in it, so a
/// silent logger cannot pass.
#[tokio::test]
async fn nothing_secret_reaches_the_log_at_any_level() {
    let captured = captured_log();
    let from = captured.0.lock().unwrap().len();

    let (good, token) = ticket();
    let session_id = Arc::new(Mutex::new(String::new()));
    let seen = session_id.clone();
    session(good, |mut product| async move {
        say(&mut product, &hello(&token)).await;
        let welcome = hear(&mut product).await.unwrap();
        *seen.lock().unwrap() = welcome["session"].as_str().unwrap().to_owned();
        say(
            &mut product,
            &json!({ "type": "request", "id": 1, "method": "vault.sign" }),
        )
        .await;
        let _ = hear(&mut product).await;
    })
    .await
    .expect("a session");

    let (bad, real) = ticket();
    let offered = "ab".repeat(32);
    let ended = session(bad, |mut product| async move {
        say(&mut product, &hello(&offered)).await;
        let _ = hear(&mut product).await;
    })
    .await;
    if let Err(error) = ended {
        tracing::warn!(%error, "a product's channel ended badly");
    }

    let log = String::from_utf8_lossy(&captured.0.lock().unwrap()[from..]).into_owned();
    assert!(
        log.contains("a product the launcher started connected"),
        "the log is empty:\n{log}"
    );
    let session_id = session_id.lock().unwrap().clone();
    for secret in [&session_id, &real, &"ab".repeat(32)] {
        assert!(!secret.is_empty());
        assert!(
            !log.contains(secret.as_str()),
            "a secret reached the log:\n{log}"
        );
    }
    assert_eq!(format!("{:?}", SpawnToken::new()), "SpawnToken(…)");
}

// ── The golden frames (tools/qor-launcher/host-protocol/golden) ───────────────────────────────────────────────────

fn golden() -> Vec<(String, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../host-protocol/golden");
    let mut frames: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .expect("the golden frames")
        .map(|entry| {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).unwrap();
            (
                name,
                serde_json::from_str(&text).unwrap_or_else(|e| panic!("{path:?}: {e}")),
            )
        })
        .collect();
    frames.sort_by(|a, b| a.0.cmp(&b.0));
    assert!(frames.len() >= 8, "{frames:?}");
    frames
}

/// Every golden frame a product sends is read as what it says it is; every golden frame the launcher sends is what
/// the launcher writes, byte for byte as JSON.
#[test]
fn the_golden_frames_are_what_each_side_reads_and_writes() {
    for (name, frame) in golden() {
        if name.starts_with("from-product.") {
            let read: FromProduct =
                serde_json::from_value(frame.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
            match (frame["type"].as_str(), read) {
                (Some("hello"), FromProduct::Hello { product, .. }) => {
                    assert_eq!(product, "qq-studio")
                }
                (Some("request"), FromProduct::Request { method, params, .. }) => {
                    assert!(
                        super::studio::CAPABILITIES.contains(&method.as_str()),
                        "{name}"
                    );
                    assert!(params["project"].is_string(), "{name}");
                }
                other => panic!("{name}: read as {other:?}"),
            }
        }
    }

    let written = |message: ToProduct| serde_json::to_value(message).unwrap();
    let expect = |name: &str| {
        golden()
            .into_iter()
            .find(|(n, _)| n == name)
            .unwrap_or_else(|| panic!("no {name}"))
            .1
    };
    assert_eq!(
        written(ToProduct::Welcome {
            protocol: 1,
            capabilities: super::studio::CAPABILITIES.to_vec(),
            session: "0".repeat(32),
        }),
        expect("to-product.welcome.json")
    );
    assert_eq!(
        written(ToProduct::Error {
            id: Some(2),
            code: "declined".into(),
            message: crate::error::QorError::Declined.to_string(),
        }),
        expect("to-product.declined.json")
    );
    assert_eq!(
        written(ToProduct::Error {
            id: None,
            code: "not_authorised".into(),
            message: "this connection was not the one the launcher started".into(),
        }),
        expect("to-product.not-authorised.json")
    );
    assert_eq!(
        written(ToProduct::Result {
            id: 1,
            result: json!({ "commit": null }),
        }),
        expect("to-product.nothing-to-commit.json")
    );
}

// ── A real spawn, through the platform's channel ──────────────────────────────────────────────────────────────────

/// The stand-in product, built from its source once per test run, so what runs is what is in the tree. `cargo test`
/// only checks an example compiles and never makes its executable, so these tests build it themselves; a build that
/// fails is a failure, not a skip.
fn fake_product() -> PathBuf {
    static BUILT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    BUILT
        .get_or_init(|| {
            let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
            let built = std::process::Command::new(option_env!("CARGO").unwrap_or("cargo"))
                .args([
                    "build",
                    "--locked",
                    "--example",
                    "host_fake_product",
                    "--manifest-path",
                ])
                .arg(&manifest)
                .status()
                .expect("cargo, to build the stand-in product");
            assert!(built.success(), "the stand-in product did not build");
            let exe = std::env::current_exe().expect("this test");
            let debug = exe
                .parent()
                .and_then(|deps| deps.parent())
                .expect("the target folder");
            let path = debug
                .join("examples")
                .join(format!("host_fake_product{}", std::env::consts::EXE_SUFFIX));
            assert!(path.is_file(), "{path:?} is missing after building it");
            path
        })
        .clone()
}

async fn run_fake(mode: &str) -> (Exited, String) {
    run_fake_with(vec!["--mode".into(), mode.into()], Arc::new(Echo)).await
}

async fn run_fake_with<H: Handler>(
    args: Vec<std::ffi::OsString>,
    handler: Arc<H>,
) -> (Exited, String) {
    let (sender, exited) = tokio::sync::oneshot::channel();
    let path = fake_product();
    let launch = Launch {
        cwd: path.parent().unwrap().to_path_buf(),
        args,
        product: ProductRecord {
            id: "fake-product",
            name: "Fake Product",
            path,
        },
    };
    let name = super::launch(launch, handler, move |exited| {
        let _ = sender.send(exited);
    })
    .await
    .expect("launched");
    let exited = tokio::time::timeout(Duration::from_secs(60), exited)
        .await
        .expect("the product ended")
        .expect("reported");
    (exited, name)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_spawned_product_proves_itself_and_is_answered() {
    let (exited, _) = run_fake("good").await;
    assert_eq!(exited.code, Some(0), "{exited:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_spawned_product_with_the_wrong_token_or_name_is_refused() {
    for mode in ["wrong-token", "wrong-product"] {
        let (exited, _) = run_fake(mode).await;
        assert_eq!(exited.code, Some(2), "{mode}: {exited:?}");
    }
}

/// Another process, here this test, connecting to the channel before the product does: the operating system says it
/// is not the process the launcher started, so it is never welcomed, and the channel takes nobody afterwards, so the
/// product, arriving late, cannot connect either.
#[tokio::test(flavor = "multi_thread")]
async fn a_process_the_launcher_did_not_start_is_not_welcomed() {
    let (sender, exited) = tokio::sync::oneshot::channel();
    let path = fake_product();
    let launch = Launch {
        cwd: path.parent().unwrap().to_path_buf(),
        args: vec!["--mode".into(), "late".into()],
        product: ProductRecord {
            id: "fake-product",
            name: "Fake Product",
            path,
        },
    };
    let name = super::launch(launch, Arc::new(Echo), move |exited| {
        let _ = sender.send(exited);
    })
    .await
    .expect("launched");

    #[cfg(windows)]
    let mut intruder = tokio::net::windows::named_pipe::ClientOptions::new()
        .open(&name)
        .expect("the pipe is there to connect to");
    #[cfg(unix)]
    let mut intruder = tokio::net::UnixStream::connect(&name)
        .await
        .expect("the socket is there");
    // Whatever it sends, it hears no welcome: the channel is closed on it.
    let _ = write_frame(
        &mut intruder,
        hello(&"00".repeat(32)).to_string().as_bytes(),
    )
    .await;
    let heard = tokio::time::timeout(Duration::from_secs(10), read_frame(&mut intruder)).await;
    assert!(
        !matches!(heard, Ok(Ok(Some(_)))),
        "a process the launcher did not start was answered: {heard:?}"
    );

    let exited = tokio::time::timeout(Duration::from_secs(60), exited)
        .await
        .expect("the product ended")
        .expect("reported");
    assert_eq!(
        exited.code,
        Some(4),
        "the late product found no channel: {exited:?}"
    );
}

// ── Publishing for real, against a development node ──────────────────────────────────────────────────────────────

/// P3.4 end to end on the launcher's side, against a real node (`chain::live`'s instructions): a product the launcher
/// started commits a project and publishes it, through the channel, the token and the operating system's checks, with
/// the dialog's answer scripted. The first publish mints the Cartridge, behind a dialog that names the product; a later
/// commit revises the same Cartridge; publishing again with nothing new asks nothing; a declined publish changes
/// nothing on the chain. The account that signs is ADR-090's: the first publish signs as the account the rail shows (here
/// the vault's second), and a revision as the account holding the Cartridge, whatever the rail shows by then. QQ
/// Studio's own side of this is `products/qq/tests/tst_publish.cpp`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs a running Demiurge development node; see the chain module's live tests"]
async fn a_project_is_minted_then_revised_then_left_alone_through_the_channel() {
    use crate::chain::live::{endpoint, fund_from_alice};
    use crate::chain::ChainClient;
    use crate::vault::normalise_address;

    let endpoint = endpoint();
    let chain = Arc::new(ChainClient::new(&endpoint).unwrap());
    let status = chain.status().await;
    assert!(
        status.reachable,
        "no node at {endpoint}: {:?}",
        status.detail
    );

    let data = tempfile::tempdir().unwrap();
    let vault = crate::vault::testing::vault_in(data.path());
    let phrase = crate::vault::derive::generate_mnemonic().unwrap();
    let first_account = vault.create(&phrase).unwrap()[0].address.clone();
    // The rail shows the second account, and only it holds anything: a publish signed by the first would be refused as
    // unfunded (ADR-090).
    let creator = vault.add_account("").unwrap().address;
    vault.set_active(&creator).unwrap();
    // A first mint at the placeholder deposits (U-14) holds about 1,002.5 CGT; room to spare.
    fund_from_alice(
        &endpoint,
        normalise_address(&creator).unwrap(),
        3_000 * crate::cgt::SPARKS_PER_CGT,
    )
    .await;
    let vault = Arc::new(vault);

    let project = tempfile::tempdir().unwrap();
    crate::qontrol::open_or_init(project.path()).unwrap();
    let config = project.path().join(".git").join("config");
    let existing = std::fs::read_to_string(&config).unwrap_or_default();
    std::fs::write(
        &config,
        format!("{existing}\n[user]\n\tname = Host Test\n\temail = test@invalid\n"),
    )
    .unwrap();
    std::fs::create_dir_all(project.path().join("scenes")).unwrap();
    let level = project.path().join("scenes").join("level.qml");
    std::fs::write(&level, "import QQ\nScene { name: \"level\" }\n").unwrap();

    let handler = |confirm: Arc<Scripted>| {
        Arc::new(super::studio::StudioHandler {
            product: record(),
            vault: vault.clone(),
            chain: chain.clone(),
            data_dir: data.path().to_path_buf(),
            confirm,
        })
    };
    let args = || -> Vec<std::ffi::OsString> {
        vec![
            "--mode".into(),
            "publish".into(),
            "--project".into(),
            project.path().into(),
        ]
    };
    let head = || {
        crate::qontrol::read(project.path()).unwrap().history[0]
            .id
            .clone()
    };
    let cartridges = || async {
        chain
            .assets_of(&creator)
            .await
            .unwrap()
            .into_iter()
            .filter(|a| a.commit.is_some())
            .collect::<Vec<_>>()
    };

    // 1. The first publish commits and mints, behind a dialog that names who asks.
    let approving = Arc::new(Scripted::approving());
    let (exited, _) = run_fake_with(args(), handler(approving.clone())).await;
    assert_eq!(exited.code, Some(10), "minted: {exited:?}");
    let first = cartridges().await;
    assert_eq!(first.len(), 1, "{first:?}");
    assert_eq!(first[0].commit.as_ref().unwrap().id, head());
    assert_eq!(approving.times_asked(), 1);
    let asked = approving.asked.lock()[0].clone();
    assert_eq!(asked.title, "Fake Product asks: Approve this mint");
    assert!(
        asked.body.contains(&head()),
        "the dialog names the commit:\n{}",
        asked.body
    );
    assert!(
        asked.body.contains(&format!("Owner: {creator}")),
        "the rail's account signs, and the dialog says so:\n{}",
        asked.body
    );

    // 2. A later commit revises the same Cartridge: no second asset. The rail now shows the first account, which holds
    // nothing; the revision is still signed by the Cartridge's holder.
    vault.set_active(&first_account).unwrap();
    std::fs::write(&level, "import QQ\nScene { name: \"level, again\" }\n").unwrap();
    let (exited, _) = run_fake_with(args(), handler(approving.clone())).await;
    assert_eq!(exited.code, Some(11), "revised: {exited:?}");
    let second = cartridges().await;
    assert_eq!(second.len(), 1, "{second:?}");
    assert_eq!(
        (second[0].collection, second[0].item),
        (first[0].collection, first[0].item)
    );
    assert_eq!(second[0].commit.as_ref().unwrap().id, head());
    assert_eq!(
        approving.asked.lock()[1].title,
        "Fake Product asks: Approve this revision"
    );
    assert!(
        approving.asked.lock()[1]
            .body
            .contains(&format!("Owner: {creator}")),
        "the holder revises"
    );
    assert!(
        chain.assets_of(&first_account).await.unwrap().is_empty(),
        "nothing went to the rail's account"
    );

    // 3. Nothing new: nothing asked, nothing sent.
    let (exited, _) = run_fake_with(args(), handler(approving.clone())).await;
    assert_eq!(exited.code, Some(12), "unchanged: {exited:?}");
    assert_eq!(approving.times_asked(), 2);

    // 4. Declined: the commit is made (it is local), and the chain is as it was.
    std::fs::write(&level, "import QQ\nScene { name: \"level, declined\" }\n").unwrap();
    let declining = Arc::new(Scripted::declining());
    let (exited, _) = run_fake_with(args(), handler(declining.clone())).await;
    assert_eq!(exited.code, Some(13), "declined: {exited:?}");
    assert_eq!(declining.times_asked(), 1);
    let after = cartridges().await;
    assert_eq!(
        after[0].commit, second[0].commit,
        "a declined publish changed the chain"
    );
}
