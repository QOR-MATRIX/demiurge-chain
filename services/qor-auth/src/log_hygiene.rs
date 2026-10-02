//! The standing check that nothing secret reaches a log.
//!
//! Token material has reached QOR ID's logs three times, each from a different direction: reset links
//! written to the service log, message bodies, and full request URIs at debug level once links carried
//! tokens in their query. So this test does not look at any one of those. It drives the real router
//! (`crate::router`) through every flow that handles a token, key, password or address, captures
//! every log record at every level (including records from crates that log through `log`), and fails
//! if any of those values appears anywhere in the output.
//!
//! It proves the capture worked before trusting an empty result: the log must hold request spans,
//! Resend acceptances from sends spawned onto other threads, webhook lines, and a record at trace
//! level.
//!
//! The same property is checked statically in CI ("Fail if QOR ID's source logs a token, key, link or
//! address"), and named in docs/GATES.toml as alpha.no-secrets-in-logs.

use axum::{
    Json as AxumJson, Router,
    body::Body,
    extract::State as AxumState,
    http::{Request, StatusCode},
    routing::post,
};
use base64::Engine;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use sqlx::PgPool;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;

use sp_core::Pair as _;
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};

use crate::config::AppConfig;
use crate::services::auth_service::AuthService;
use crate::services::{EmailConfig, EmailService};
use crate::state::AppState;

/// Every log line, as the service would write it.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("lock").extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

type Inbox = Arc<Mutex<Vec<Value>>>;

async fn fake_resend() -> (String, Inbox) {
    let inbox: Inbox = Arc::default();
    let app = Router::new()
        .route(
            "/emails",
            post(
                |AxumState(inbox): AxumState<Inbox>, AxumJson(body): AxumJson<Value>| async move {
                    inbox.lock().expect("lock").push(body);
                    AxumJson(json!({ "id": "4ef9a417-02e9-4d39-ad75-9611e0fcc33c" }))
                },
            ),
        )
        .with_state(inbox.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("address"));
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
    (url, inbox)
}

struct Client {
    app: Router,
}

struct Reply {
    status: StatusCode,
    body: String,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

impl Client {
    async fn send(&self, request: Request<Body>) -> Reply {
        let response = self.app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        Reply {
            status,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }

    async fn json(&self, path: &str, body: Value, bearer: Option<&str>) -> Reply {
        let mut request = Request::post(path).header("content-type", "application/json");
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        self.send(request.body(Body::from(body.to_string())).expect("request"))
            .await
    }

    async fn get(&self, path: &str, bearer: Option<&str>) -> Reply {
        let mut request = Request::get(path);
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        self.send(request.body(Body::empty()).expect("request"))
            .await
    }

    async fn form(&self, path: &str, fields: &[(&str, &str)]) -> Reply {
        let body = fields
            .iter()
            .map(|(k, v)| format!("{k}={}", form_encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        self.send(
            Request::post(path)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(body))
                .expect("request"),
        )
        .await
    }

    async fn webhook(&self, secret: &str, id: &str, timestamp: i64, body: &str) -> Reply {
        let key = base64::engine::general_purpose::STANDARD
            .decode(secret.trim_start_matches("whsec_"))
            .expect("secret");
        let mut mac = Hmac::<Sha256>::new_from_slice(&key).expect("key");
        mac.update(format!("{id}.{timestamp}.{body}").as_bytes());
        let signature = format!(
            "v1,{}",
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        );
        self.send(
            Request::post("/api/v1/webhooks/resend")
                .header("content-type", "application/json")
                .header("svix-id", id)
                .header("svix-timestamp", timestamp.to_string())
                .header("svix-signature", signature)
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
    }
}

fn form_encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Waits for a message to `to` whose HTML matches `needle`, and returns the token after it.
async fn token_sent_to(inbox: &Inbox, to: &str, needle: &str, skip: usize) -> String {
    for _ in 0..100 {
        let found = inbox
            .lock()
            .expect("lock")
            .iter()
            .filter(|m| m["to"] == json!([to]))
            .filter_map(|m| m["html"].as_str())
            .filter(|html| html.contains(needle))
            .nth(skip)
            .and_then(|html| html.split(needle).nth(1))
            .map(|rest| {
                rest.chars()
                    .take_while(char::is_ascii_hexdigit)
                    .collect::<String>()
            });
        if let Some(token) = found.filter(|t| !t.is_empty()) {
            return token;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("no message with {needle} reached the stand-in for Resend");
}

async fn messages_to(inbox: &Inbox, to: &str) -> usize {
    // Let any send spawned after a response run first.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    inbox
        .lock()
        .expect("lock")
        .iter()
        .filter(|m| m["to"] == json!([to]))
        .count()
}

#[sqlx::test]
#[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
async fn nothing_secret_reaches_a_log_at_any_level(db: PgPool) {
    let captured = Captured::default();
    let writer = captured.clone();
    // Process-wide, not for this thread: sends spawned after a response run on other runtime threads,
    // and a thread-local subscriber missed them (the first version of this check did). `try_init` also
    // routes records from crates that log through `log`. No other test installs a subscriber; records
    // from tests running alongside join the capture, which only widens what is checked.
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::FULL)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish()
        .try_init()
        .expect("the log check is the only test that installs a subscriber");
    tracing::trace!("log check: capturing at trace level");

    let (resend_url, inbox) = fake_resend().await;
    let api_key = format!("re_logcheck_{}", uuid::Uuid::new_v4().simple());
    let webhook_secret = format!(
        "whsec_{}",
        base64::engine::general_purpose::STANDARD.encode(uuid::Uuid::new_v4().as_bytes())
    );
    let config = AppConfig::default();
    let email = EmailService::new(EmailConfig {
        resend_api_key: api_key.clone(),
        from: "Log Check <noreply@sender.logcheck.invalid>".into(),
        base_url: "https://links.logcheck.invalid".into(),
        api_url: resend_url,
    })
    .with_webhook_secret(Some(webhook_secret.clone()));
    let redis_url =
        std::env::var("QOR_AUTH_TEST_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:1".into());
    let redis = deadpool_redis::Config::from_url(redis_url)
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("redis pool");
    let jwt_secrets = [
        config.jwt.access_secret.clone(),
        config.jwt.refresh_secret.clone(),
    ];
    let state = Arc::new(AppState::new(config, db.clone(), redis, email));
    let client = Client {
        app: crate::router(state),
    };

    let username = "logcheck";
    let address = "logcheck@mail.logcheck.invalid";
    let moved = "logcheck.moved@mail.logcheck.invalid";
    let password = "a log check password";
    let new_password = "a new log check password";
    let mut secrets: Vec<String> = vec![
        api_key.clone(),
        webhook_secret.clone(),
        password.into(),
        new_password.into(),
        "logcheck.invalid".into(),
        "?token=".into(),
    ];
    secrets.extend(jwt_secrets);

    // Registration, and its verification link opened and confirmed as a page.
    let r = client
        .json(
            "/api/v1/auth/register",
            json!({ "username": username, "email": address, "password": password }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    let verify = token_sent_to(&inbox, address, "verify-email?token=", 0).await;
    secrets.push(verify.clone());
    assert_eq!(
        client
            .get(&format!("/verify-email?token={verify}"), None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        client
            .form("/verify-email", &[("token", &verify)])
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        client
            .get(&format!("/verify-email?token={verify}"), None)
            .await
            .status,
        StatusCode::GONE
    );
    let r = client
        .json(
            "/api/v1/auth/resend-verification",
            json!({ "identifier": address }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);

    // A reset through the page: a refused form, then the real one.
    let r = client
        .json(
            "/api/v1/auth/forgot-password",
            json!({ "identifier": username }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    let reset = token_sent_to(&inbox, address, "reset-password?token=", 0).await;
    secrets.push(reset.clone());
    assert_eq!(
        client
            .get(&format!("/reset-password?token={reset}"), None)
            .await
            .status,
        StatusCode::OK
    );
    let r = client
        .form(
            "/reset-password",
            &[
                ("token", &reset),
                ("new_password", new_password),
                ("confirm_password", "not the same password"),
            ],
        )
        .await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    let r = client
        .form(
            "/reset-password",
            &[
                ("token", &reset),
                ("new_password", new_password),
                ("confirm_password", new_password),
            ],
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let r = client
        .form("/forgot-password", &[("identifier", address)])
        .await;
    assert_eq!(r.status, StatusCode::OK);

    // Sign-in, a refused sign-in, and the profile.
    let r = client
        .json(
            "/api/v1/auth/login",
            json!({ "identifier": username, "password": password }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = client
        .json(
            "/api/v1/auth/login",
            json!({ "identifier": address, "password": new_password }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let access = r.json()["access_token"]
        .as_str()
        .expect("access token")
        .to_string();
    secrets.push(access.clone());
    if let Some(refresh) = r.json()["refresh_token"].as_str() {
        secrets.push(refresh.to_string());
    }
    assert_eq!(
        client.get("/api/v1/profile", Some(&access)).await.status,
        StatusCode::OK
    );

    // A refresh, which writes to the session that it was used, and the list that shows it. Both
    // tokens it mints are secrets too. A refused refresh handles a token as well.
    let refresh = r.json()["refresh_token"]
        .as_str()
        .expect("refresh token")
        .to_string();
    let r = client
        .json(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": refresh }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    for minted in ["access_token", "refresh_token"] {
        secrets.push(r.json()[minted].as_str().expect("a token").to_string());
    }
    let r = client.get("/api/v1/profile/sessions", Some(&access)).await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    assert!(r.json()["sessions"][0]["last_used_at"].is_string());
    let forged = format!("{refresh}x");
    secrets.push(forged.clone());
    let r = client
        .json(
            "/api/v1/auth/refresh",
            json!({ "refresh_token": forged }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{}", r.body);

    // A change of address: a notice to the old one, a link to the new one, confirmed as a page.
    sqlx::query("UPDATE users SET email_verification_sent_at = NOW() - INTERVAL '10 minutes' WHERE username = $1")
        .bind(username)
        .execute(&db)
        .await
        .expect("backdate");
    let r = client
        .json(
            "/api/v1/profile/email",
            json!({ "email": moved, "password": new_password }),
            Some(&access),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let confirm = token_sent_to(&inbox, moved, "verify-email?token=", 0).await;
    secrets.push(confirm.clone());
    assert_eq!(
        client
            .get(&format!("/verify-email?token={confirm}"), None)
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        client
            .form("/verify-email", &[("token", &confirm)])
            .await
            .status,
        StatusCode::OK
    );

    // Webhooks: refused, then a permanent bounce that stops sending.
    let now = chrono::Utc::now().timestamp();
    let bounce = json!({
        "type": "email.bounced",
        "created_at": "2026-09-15T12:00:00.000Z",
        "data": {
            "email_id": "4ef9a417-02e9-4d39-ad75-9611e0fcc33c",
            "from": "Log Check <noreply@sender.logcheck.invalid>",
            "to": [moved],
            "subject": "Reset your QOR ID password",
            "bounce": { "message": "mailbox does not exist", "subType": "General", "type": "Permanent" }
        }
    })
    .to_string();
    let r = client
        .webhook(
            "whsec_plJ3nmyCDGBKInavdOK15jsl",
            "msg_logcheck_forged",
            now,
            &bounce,
        )
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = client
        .webhook(&webhook_secret, "msg_logcheck_stale", now - 600, &bounce)
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    let r = client
        .webhook(&webhook_secret, "msg_logcheck_bounce", now, &bounce)
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);
    let before = messages_to(&inbox, moved).await;
    let r = client
        .json(
            "/api/v1/auth/forgot-password",
            json!({ "identifier": moved }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(
        messages_to(&inbox, moved).await,
        before,
        "nothing is sent to a bounced address"
    );

    // An administrator takes the mark off, after the account holder is refused.
    let r = client
        .json(
            "/api/v1/admin/email-suppressions/unmark",
            json!({ "email": moved, "reason": "The mailbox exists again." }),
            Some(&access),
        )
        .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    let admin_password = "an administrator's log check password";
    secrets.push(admin_password.into());
    let r = client
        .json(
            "/api/v1/auth/register",
            json!({ "username": "logadmin", "password": admin_password }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    for code in r.json()["backup_codes"].as_array().into_iter().flatten() {
        secrets.push(code.as_str().expect("code").to_string());
    }
    sqlx::query("UPDATE users SET role = 'god' WHERE username = 'logadmin'")
        .execute(&db)
        .await
        .expect("promote");
    let r = client
        .json(
            "/api/v1/auth/login",
            json!({ "identifier": "logadmin", "password": admin_password }),
            None,
        )
        .await;
    let admin = r.json()["access_token"]
        .as_str()
        .expect("admin token")
        .to_string();
    secrets.push(admin.clone());
    let r = client
        .json(
            "/api/v1/admin/email-suppressions/unmark",
            json!({ "email": moved, "reason": "The mailbox exists again." }),
            Some(&admin),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);

    // Links that no longer work.
    assert_eq!(
        client
            .get(&format!("/reset-password?token={reset}"), None)
            .await
            .status,
        StatusCode::GONE
    );
    assert_eq!(
        client
            .form(
                "/verify-email",
                &[("token", "0123456789abcdef0123456789abcdef")]
            )
            .await
            .status,
        StatusCode::GONE
    );

    // The key flows (ADR-023, ADR-024, AGENTS.md section 9). A challenge is
    // asked for with the account in the query string, which is exactly how
    // token material reached the log the third time, so the address, the
    // account ID and every signature go in the list above and must not appear
    // anywhere in the output.
    let signer = sp_core::sr25519::Pair::from_seed(&[0x5au8; 32]);
    let account_id = format!("0x{}", hex::encode(signer.public().0));
    let key_address = signer
        .public()
        .to_ss58check_with_version(Ss58AddressFormat::custom(42));
    secrets.push(key_address.clone());
    secrets.push(account_id.clone());

    // Asked for in each form the route accepts, so neither can slip into a log
    // unseen.
    let answer = |query: &str| {
        let query = query.to_string();
        let client = &client;
        let signer = &signer;
        async move {
            let r = client
                .get(&format!("/api/v1/auth/challenge?{query}"), None)
                .await;
            assert_eq!(r.status, StatusCode::OK, "{}", r.body);
            let challenge = r.json()["challenge"]
                .as_str()
                .expect("a challenge")
                .to_string();
            let signature = hex::encode(
                signer
                    .sign(format!("demiurge:qor-id:challenge:v1:{challenge}").as_bytes())
                    .0,
            );
            (challenge, signature)
        }
    };

    let (challenge, signature) = answer(&format!("address={key_address}")).await;
    secrets.push(signature.clone());
    let r = client
        .json(
            "/api/v1/auth/keypair-register",
            json!({
                "address": key_address,
                "username": "logcheckkey",
                "challenge": challenge,
                "signature": signature,
            }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);

    let (challenge, signature) = answer(&format!("account_id={account_id}")).await;
    secrets.push(signature.clone());
    let r = client
        .json(
            "/api/v1/auth/keypair-login",
            json!({
                "account_id": account_id,
                "challenge": challenge,
                "signature": signature,
            }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);

    // A refused signature: the challenge signed bare, without the domain tag.
    let r = client
        .get(
            &format!("/api/v1/auth/challenge?address={key_address}"),
            None,
        )
        .await;
    let challenge = r.json()["challenge"]
        .as_str()
        .expect("a challenge")
        .to_string();
    let bare = hex::encode(signer.sign(challenge.as_bytes()).0);
    secrets.push(bare.clone());
    let r = client
        .json(
            "/api/v1/auth/keypair-login",
            json!({ "address": key_address, "challenge": challenge, "signature": bare }),
            None,
        )
        .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED, "{}", r.body);

    // Linking a key to the password account, which is the only way a
    // password-only account gets a chain identity at all (ADR-017).
    let linked = sp_core::sr25519::Pair::from_seed(&[0x7bu8; 32]);
    let linked_address = linked
        .public()
        .to_ss58check_with_version(Ss58AddressFormat::custom(42));
    secrets.push(linked_address.clone());
    let r = client
        .get(
            &format!("/api/v1/auth/challenge?address={linked_address}"),
            None,
        )
        .await;
    let challenge = r.json()["challenge"]
        .as_str()
        .expect("a challenge")
        .to_string();
    let signature = hex::encode(
        linked
            .sign(format!("demiurge:qor-id:challenge:v1:{challenge}").as_bytes())
            .0,
    );
    secrets.push(signature.clone());
    let r = client
        .json(
            "/api/v1/auth/link-keypair",
            json!({
                "address": linked_address,
                "challenge": challenge,
                "signature": signature,
            }),
            Some(&access),
        )
        .await;
    assert_eq!(r.status, StatusCode::OK, "{}", r.body);

    // A hash of the password is not the password, but it is still not for a log.
    let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE username = $1")
        .bind(username)
        .fetch_one(&db)
        .await
        .expect("hash");
    secrets.push(hash);
    assert!(
        AuthService::verify_password(new_password, &secrets.last().cloned().unwrap_or_default())
            .unwrap_or(false)
    );

    let log = String::from_utf8_lossy(&captured.0.lock().expect("lock")).into_owned();

    // The capture worked: an empty log would pass everything below.
    for expected in [
        "log check: capturing at trace level",
        "/verify-email",
        "/api/v1/webhooks/resend",
        "Email accepted by Resend",
        "Resend webhook refused",
        "Resend webhook email.bounced",
        "Email not sent, because the address is marked undeliverable",
        "/api/v1/auth/challenge",
        "/api/v1/auth/keypair-register",
        "/api/v1/auth/refresh",
        "/api/v1/profile/sessions",
    ] {
        assert!(
            log.contains(expected),
            "the log should contain {expected:?}"
        );
    }

    // A failure message is itself output, and on CI a log: every line it quotes has every known secret
    // masked, longest first, not only the one it is quoted for.
    let mut masks: Vec<&String> = secrets.iter().filter(|secret| secret.len() >= 6).collect();
    masks.sort_by_key(|secret| std::cmp::Reverse(secret.len()));
    let masked = |line: &str| {
        masks.iter().fold(line.to_string(), |line, secret| {
            line.replace(secret.as_str(), "<SECRET>")
        })
    };
    let leaks: Vec<String> = masks
        .iter()
        .filter(|secret| log.contains(secret.as_str()))
        .map(|secret| {
            let lines: Vec<String> = log
                .lines()
                .filter(|line| line.contains(secret.as_str()))
                .take(3)
                .map(masked)
                .collect();
            format!(
                "a value of {} characters, in:\n  {}",
                secret.len(),
                lines.join("\n  ")
            )
        })
        .collect();
    assert!(
        leaks.is_empty(),
        "secret values reached the log:\n{}",
        leaks.join("\n")
    );
}
