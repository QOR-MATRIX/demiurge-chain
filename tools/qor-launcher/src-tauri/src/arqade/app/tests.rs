//! ARQADE's side of the launcher: the queue `arqade.next` waits on, the sign-in it may ask for and every check on it,
//! the website it may open, and nothing of a sign-in reaching the log.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use super::*;
use crate::host::frame::{read_frame, write_frame};
use crate::host::session::{serve, Ticket};
use crate::host::{ProductRecord, SpawnToken};

const PAGE: &str =
    "https://id.qorsync.dev/oauth/authorize?client_id=arqade&state=st4te-0f-arqade&code_challenge=ch4llenge";
const CALLBACK: &str = "https://qor-arqade-tau.vercel.app/api/auth/callback?code=c0de-fr0m-qor-id&state=st4te-0f-arqade";

/// A launcher whose QOR ID answers what the test says, and which counts what it was asked.
struct Fake {
    host: Option<String>,
    answer: Mutex<Option<QorResult<String>>>,
    asked: Mutex<Vec<String>>,
    web: Result<(), String>,
    opened: AtomicUsize,
}

impl Fake {
    fn answering(answer: QorResult<String>) -> Arc<Self> {
        Arc::new(Self {
            host: Some("id.qorsync.dev".into()),
            answer: Mutex::new(Some(answer)),
            asked: Mutex::new(vec![]),
            web: Ok(()),
            opened: AtomicUsize::new(0),
        })
    }
}

impl Launcher for Arc<Fake> {
    fn qor_id_host(&self) -> Option<String> {
        self.host.clone()
    }

    async fn handoff(&self, authorize_url: String) -> QorResult<String> {
        self.asked.lock().push(authorize_url);
        self.answer
            .lock()
            .take()
            .unwrap_or(Err(QorError::Internal("asked twice".into())))
    }

    fn open_web(&self) -> Result<(), String> {
        self.opened.fetch_add(1, Ordering::SeqCst);
        self.web.clone()
    }
}

fn handler(fake: &Arc<Fake>, wait: Duration) -> ArqadeHandler<Arc<Fake>> {
    ArqadeHandler {
        queue: Arc::new(Queue::new(wait)),
        launcher: Arc::clone(fake),
    }
}

async fn sign_in(fake: &Arc<Fake>, page: &str) -> Result<Value, Refusal> {
    handler(fake, WAIT)
        .handle("arqade.signin", json!({ "authorize_url": page }))
        .await
}

fn rect(x: i32) -> Rect {
    Rect {
        x,
        y: 0,
        w: 800,
        h: 600,
    }
}

#[test]
fn it_offers_its_three_methods_and_quiets_only_the_wait() {
    let h = handler(&Fake::answering(Ok(CALLBACK.into())), WAIT);
    assert_eq!(
        h.capabilities(),
        ["arqade.next", "arqade.signin", "arqade.web"]
    );
    assert!(h.quiet("arqade.next"));
    assert!(!h.quiet("arqade.signin"));
    assert!(!h.quiet("arqade.web"));
    // Short, so a sign-in is not held long behind the wait.
    assert!(WAIT <= Duration::from_secs(5));
}

#[tokio::test]
async fn arqade_next_answers_a_command_at_once_and_nothing_after_the_wait() {
    let h = handler(
        &Fake::answering(Ok(CALLBACK.into())),
        Duration::from_millis(200),
    );
    h.queue.push(Command::Hide);
    let started = Instant::now();
    assert_eq!(
        h.handle("arqade.next", Value::Null).await.unwrap(),
        json!({ "command": { "type": "hide" } })
    );
    assert!(started.elapsed() < Duration::from_millis(150));

    let started = Instant::now();
    assert_eq!(
        h.handle("arqade.next", Value::Null).await.unwrap(),
        json!({ "command": null })
    );
    assert!(started.elapsed() >= Duration::from_millis(190));
}

#[tokio::test]
async fn a_newer_command_of_a_kind_replaces_the_older_and_the_rest_keep_their_order() {
    let queue: Queue<Command> = Queue::new(Duration::from_millis(50));
    queue.push(Command::Place {
        rect: rect(1),
        parent: None,
    });
    queue.push(Command::Player {
        player: Player {
            name: Some("old".into()),
            ..Player::default()
        },
    });
    queue.push(Command::Hide);
    queue.push(Command::Prefs {
        prefs: Prefs {
            reduced_motion: false,
        },
    });
    queue.push(Command::Place {
        rect: rect(2),
        parent: Some(7),
    });
    queue.push(Command::Player {
        player: Player {
            name: Some("new".into()),
            ..Player::default()
        },
    });
    queue.push(Command::Prefs {
        prefs: Prefs {
            reduced_motion: true,
        },
    });
    queue.push(Command::Quit);

    let mut got = vec![];
    while let Some(command) = queue.next().await {
        got.push(command);
    }
    assert_eq!(
        got,
        vec![
            Command::Hide,
            Command::Place {
                rect: rect(2),
                parent: Some(7)
            },
            Command::Player {
                player: Player {
                    name: Some("new".into()),
                    ..Player::default()
                }
            },
            Command::Prefs {
                prefs: Prefs {
                    reduced_motion: true
                }
            },
            Command::Quit,
        ]
    );
}

#[test]
fn commands_are_written_as_the_app_reads_them() {
    let written = |c: Command| serde_json::to_value(c).unwrap();
    assert_eq!(
        written(Command::Place {
            rect: rect(10),
            parent: Some(4242)
        }),
        json!({ "type": "place", "rect": { "x": 10, "y": 0, "w": 800, "h": 600 }, "parent": 4242 })
    );
    assert_eq!(written(Command::Hide), json!({ "type": "hide" }));
    assert_eq!(written(Command::Quit), json!({ "type": "quit" }));
    assert_eq!(
        written(Command::Player {
            player: Player {
                signed_in: true,
                name: Some("yaldabaoth".into()),
                level: Some(7),
                sparks: Some("37,000,000 Sparks".into()),
                cgt: Some("37,000".into()),
            }
        }),
        json!({ "type": "player", "player": { "signed_in": true, "name": "yaldabaoth", "level": 7, "sparks": "37,000,000 Sparks", "cgt": "37,000" } })
    );
    assert_eq!(
        written(Command::Prefs {
            prefs: Prefs {
                reduced_motion: true
            }
        }),
        json!({ "type": "prefs", "prefs": { "reduced_motion": true } })
    );
    assert_eq!(
        written(Command::Theme {
            colors: BTreeMap::from([("accent".to_string(), "#ff6a00".to_string())])
        }),
        json!({ "type": "theme", "colors": { "accent": "#ff6a00" } })
    );
}

#[test]
fn a_player_is_tidied_before_it_is_sent() {
    let player = Player {
        signed_in: true,
        name: Some(format!("a\u{7}b{}", "x".repeat(100))),
        level: Some(3),
        sparks: Some(String::new()),
        cgt: None,
    }
    .tidy();
    let name = player.name.unwrap();
    assert!(name.starts_with("ab"));
    assert_eq!(name.chars().count(), 64);
    assert_eq!(
        player.sparks, None,
        "nothing to show is not sent as an empty text"
    );
}

#[tokio::test]
async fn a_sign_in_is_handed_to_qor_id_and_its_answer_returned() {
    let fake = Fake::answering(Ok(CALLBACK.into()));
    let answer = sign_in(&fake, PAGE).await.expect("signed in");
    assert_eq!(answer, json!({ "redirect": CALLBACK }));
    assert_eq!(*fake.asked.lock(), vec![PAGE.to_string()]);
}

#[tokio::test]
async fn only_qor_id_s_own_sign_in_page_is_handed_over() {
    for page in [
        "https://evil.example/oauth/authorize?client_id=arqade",
        "https://id.qorsync.dev/account",
        "https://id.qorsync.dev/oauth/authorize/extra",
        "http://id.qorsync.dev/oauth/authorize?client_id=arqade",
        "https://id.qorsync.dev.example.com/oauth/authorize",
        "qor://pay?r=00",
    ] {
        let fake = Fake::answering(Ok(CALLBACK.into()));
        let refused = sign_in(&fake, page).await.expect_err(page);
        assert_eq!(refused.code, code::NOT_AUTHORISED, "{page}");
        assert!(fake.asked.lock().is_empty(), "{page} reached QOR ID");
    }
    let fake = Fake::answering(Ok(CALLBACK.into()));
    let refused = sign_in(&fake, "not an address").await.expect_err("bad");
    assert_eq!(refused.code, code::BAD_REQUEST);
    let refused = handler(&fake, WAIT)
        .handle("arqade.signin", json!({ "page": PAGE }))
        .await
        .expect_err("bad params");
    assert_eq!(refused.code, code::BAD_REQUEST);
    assert!(fake.asked.lock().is_empty());
}

#[tokio::test]
async fn an_answer_that_does_not_land_on_arqade_is_not_passed_on() {
    for redirect in [
        "https://evil.example/api/auth/callback?code=c",
        "http://qor-arqade-tau.vercel.app/api/auth/callback?code=c",
        "https://qor-arqade-tau.vercel.app.example.com/api/auth/callback?code=c",
        "qor://pay?r=00",
        "not an address",
    ] {
        let fake = Fake::answering(Ok(redirect.into()));
        let refused = sign_in(&fake, PAGE).await.expect_err(redirect);
        assert_eq!(refused.code, code::NOT_AUTHORISED, "{redirect}");
        assert!(!refused.message.contains("code="), "{redirect}");
    }
}

#[tokio::test]
async fn without_a_session_or_a_qor_id_it_says_so() {
    let fake = Fake::answering(Err(QorError::NotAuthenticated));
    let refused = sign_in(&fake, PAGE).await.expect_err("no session");
    assert_eq!(refused.code, "not_authenticated");

    let fake = Fake::answering(Err(QorError::Auth("QOR ID refused".into())));
    let refused = sign_in(&fake, PAGE).await.expect_err("refused");
    assert_eq!(refused.code, "handoff");
    assert!(!refused.message.contains("st4te"));

    let fake = Arc::new(Fake {
        host: None,
        answer: Mutex::new(Some(Ok(CALLBACK.into()))),
        asked: Mutex::new(vec![]),
        web: Ok(()),
        opened: AtomicUsize::new(0),
    });
    let refused = sign_in(&fake, PAGE).await.expect_err("no QOR ID");
    assert_eq!(refused.code, code::NOT_AUTHORISED);
    assert!(fake.asked.lock().is_empty());
}

#[tokio::test]
async fn the_website_opens_in_the_launcher_s_window_or_says_why_not() {
    let fake = Fake::answering(Ok(CALLBACK.into()));
    assert_eq!(
        handler(&fake, WAIT)
            .handle("arqade.web", Value::Null)
            .await
            .unwrap(),
        json!({})
    );
    assert_eq!(fake.opened.load(Ordering::SeqCst), 1);

    let fake = Arc::new(Fake {
        host: Some("id.qorsync.dev".into()),
        answer: Mutex::new(None),
        asked: Mutex::new(vec![]),
        web: Err("ARQADE's window could not be opened".into()),
        opened: AtomicUsize::new(0),
    });
    let refused = handler(&fake, WAIT)
        .handle("arqade.web", Value::Null)
        .await
        .expect_err("no window");
    assert_eq!(refused.code, "web");
}

#[test]
fn arqade_is_found_where_it_is_put() {
    let root = std::env::temp_dir().join(format!("arqade-find-{}", uuid::Uuid::new_v4().simple()));
    let installed = root.join("install");
    let local = root.join("local");
    let program = if cfg!(windows) {
        "arqade.exe"
    } else {
        "arqade"
    };
    std::fs::create_dir_all(local.join("arqade")).unwrap();
    std::fs::write(local.join("arqade").join(program), b"").unwrap();

    assert_eq!(
        find(None, Some(installed.clone()), Some(local.clone())),
        Some(local.join("arqade").join(program))
    );
    std::fs::create_dir_all(installed.join("arqade")).unwrap();
    std::fs::write(installed.join("arqade").join(program), b"").unwrap();
    assert_eq!(
        find(None, Some(installed.clone()), Some(local.clone())),
        Some(installed.join("arqade").join(program)),
        "the installed copy comes first"
    );
    let named = root.join("elsewhere.exe");
    assert_eq!(
        find(Some(named.clone().into()), Some(installed.clone()), None),
        None,
        "a name that is not there is not passed over"
    );
    std::fs::write(&named, b"").unwrap();
    assert_eq!(
        find(Some(named.clone().into()), Some(installed), None),
        Some(named)
    );
    let _ = std::fs::remove_dir_all(root);
}

/// A sign-in through a whole session, the good one and a refused one, with every level of the launcher's log on: the
/// sign-in page's state and challenge, and QOR ID's code, appear nowhere in it (AGENTS.md §9 for the launcher's log).
#[tokio::test]
async fn nothing_of_a_sign_in_reaches_the_log_at_any_level() {
    let captured = crate::testing::captured_log();
    let from = captured.0.lock().unwrap().len();

    let fake = Fake::answering(Ok(CALLBACK.into()));
    let h = handler(&fake, Duration::from_millis(50));
    let token = SpawnToken::new();
    let hex = token.hex().to_owned();
    let ticket = Ticket {
        product: ProductRecord {
            id: "arqade",
            name: "ARQADE",
            path: PathBuf::from("C:/ARQADE/arqade.exe"),
        },
        token,
        issued: Instant::now(),
    };
    let (launcher, mut product) = tokio::io::duplex(64 * 1024);
    let say = |frame: Value| frame.to_string().into_bytes();
    let (ended, ()) = tokio::join!(serve(launcher, ticket, &h), async move {
        let hear = |bytes: Vec<u8>| serde_json::from_slice::<Value>(&bytes).unwrap();
        write_frame(&mut product, &say(json!({ "type": "hello", "protocol": 1, "min_protocol": 1, "product": "arqade", "version": "0.1.0", "token": hex }))).await.unwrap();
        let welcome = hear(read_frame(&mut product).await.unwrap().unwrap());
        assert_eq!(welcome["capabilities"], json!(CAPABILITIES));
        write_frame(&mut product, &say(json!({ "type": "request", "id": 1, "method": "arqade.signin", "params": { "authorize_url": PAGE } }))).await.unwrap();
        let answer = hear(read_frame(&mut product).await.unwrap().unwrap());
        assert_eq!(answer["result"]["redirect"], CALLBACK);
        // Refused: another QOR ID's page.
        write_frame(&mut product, &say(json!({ "type": "request", "id": 2, "method": "arqade.signin", "params": { "authorize_url": PAGE.replace("id.qorsync.dev", "evil.example") } }))).await.unwrap();
        let refused = hear(read_frame(&mut product).await.unwrap().unwrap());
        assert_eq!(refused["code"], code::NOT_AUTHORISED);
        write_frame(
            &mut product,
            &say(json!({ "type": "request", "id": 3, "method": "arqade.next" })),
        )
        .await
        .unwrap();
        let _ = read_frame(&mut product).await.unwrap();
    });
    ended.expect("a session");

    let log = String::from_utf8_lossy(&captured.0.lock().unwrap()[from..]).into_owned();
    assert!(log.contains("arqade.signin"), "the log is empty:\n{log}");
    for secret in [
        "st4te-0f-arqade",
        "ch4llenge",
        "c0de-fr0m-qor-id",
        "authorize?",
    ] {
        assert!(!log.contains(secret), "{secret} reached the log:\n{log}");
    }
}

/// ARQADE's server, as far as the real app's first step goes: its sign-in sends the app to QOR ID's page.
async fn fake_arqade_server() -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = format!("http://{}/", listener.local_addr().unwrap());
    tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut head = Vec::new();
                let mut chunk = [0u8; 1024];
                while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                    match socket.read(&mut chunk).await {
                        Ok(0) | Err(_) => return,
                        Ok(n) => head.extend_from_slice(&chunk[..n]),
                    }
                }
                let answer = if head.starts_with(b"GET /api/auth/login ") {
                    format!("HTTP/1.1 302 Found\r\nLocation: {PAGE}\r\nSet-Cookie: arq_login=L; Path=/api/auth; HttpOnly\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                } else {
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                        .to_string()
                };
                let _ = socket.write_all(answer.as_bytes()).await;
            });
        }
    });
    address
}

/// The real ARQADE, started through the product host as the ARQADE screen starts it: it is welcomed, takes every
/// command it is sent, asks the launcher to sign it in with exactly the page its server sent it to, and closes when
/// told to. Needs ARQADE built and deployed (`pwsh products/arqade/qt/build.ps1 -Deploy`, which needs the owner's Qt),
/// so it runs only when asked: `cargo test --lib arqade::app::tests::arqade_started -- --ignored`.
#[test]
#[ignore = "needs ARQADE deployed by products/arqade/qt/build.ps1 -Deploy"]
fn arqade_started_by_the_launcher_docks_signs_in_through_it_and_quits() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(docked_signed_in_and_quit());
    runtime.shutdown_timeout(Duration::from_secs(1));
}

async fn docked_signed_in_and_quit() {
    let program = find(
        std::env::var_os("ARQADE_APP"),
        None,
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    )
    .expect("ARQADE is deployed");
    // The app reads its server from ARQADE_SERVER, which the child inherits: a stand-in here, never ARQADE's own.
    std::env::set_var("ARQADE_SERVER", fake_arqade_server().await);

    // QOR ID answers with an address that is not ARQADE's, so the launcher's own check refuses to pass it on.
    let fake = Fake::answering(Ok("https://evil.example/api/auth/callback?code=c".into()));
    let arqade = App::default();
    *arqade.theme.lock() = BTreeMap::from([("accent".to_string(), "#ff6a00".to_string())]);
    *arqade.prefs.lock() = Some(Prefs {
        reduced_motion: true,
    });
    *arqade.rect.lock() = Some(Rect {
        x: 40,
        y: 40,
        w: 960,
        h: 600,
    });
    *arqade.player.lock() = Some(Player {
        signed_in: true,
        name: Some("e2e".into()),
        level: Some(1),
        sparks: Some("1,000 Sparks".into()),
        cgt: Some("1".into()),
    });
    // Left from a run that had already ended when a sign-out told it to quit: not this run's to obey.
    arqade.queue.push(Command::Quit);

    let exited = Arc::new(Mutex::new(None));
    let noted = Arc::clone(&exited);
    arqade
        .open(program.clone(), Arc::clone(&fake), None, move |e| {
            *noted.lock() = Some(e)
        })
        .await
        .expect("ARQADE starts");
    assert!(arqade.running());
    // A second visit while it runs starts nothing new.
    let again = Arc::new(Mutex::new(None));
    let noted_again = Arc::clone(&again);
    arqade
        .open(program, Arc::clone(&fake), None, move |e| {
            *noted_again.lock() = Some(e)
        })
        .await
        .expect("still running");

    let started = Instant::now();
    while fake.asked.lock().is_empty() {
        if let Some(e) = exited.lock().clone() {
            panic!("ARQADE ended: {e:?}");
        }
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "ARQADE never asked to be signed in"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(*fake.asked.lock(), vec![PAGE.to_string()]);

    arqade.quit();
    let started = Instant::now();
    loop {
        if let Some(e) = exited.lock().clone() {
            assert_eq!(e.code, Some(0), "{e:?}");
            break;
        }
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "ARQADE did not close when told"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(!arqade.running(), "an ended run is marked so");
    assert!(
        again.lock().is_none(),
        "the second visit started no second ARQADE"
    );
    assert_eq!(
        *arqade.player.lock(),
        None,
        "who was playing is not kept after a sign-out"
    );
}

#[tokio::test]
async fn a_cleared_queue_holds_nothing_from_before() {
    let queue: Queue<Command> = Queue::new(Duration::from_millis(50));
    queue.push(Command::Quit);
    queue.push(Command::Hide);
    queue.clear();
    queue.push(Command::Place {
        rect: rect(3),
        parent: None,
    });
    assert_eq!(
        queue.next().await,
        Some(Command::Place {
            rect: rect(3),
            parent: None
        })
    );
    assert_eq!(queue.next().await, None);
}
