//! Who a person is, asked of QOR ID (ADR-091 decision 2).
//!
//! A client presents its QOR ID access token in its socket's first frame. Social asks QOR ID's introspection endpoint
//! whether it is live, as a registered service with a secret of its own, and keeps the answer for at most 30 seconds
//! or until the token expires, whichever is sooner, keyed by the token's SHA-256: the token itself is never stored,
//! never logged and never sent anywhere but QOR ID.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;
use uuid::Uuid;

/// A person, as QOR ID says they are now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub sub: Uuid,
    pub qor_id: String,
    /// False while the account is on a name QOR ID made for it; it chooses its own before it posts.
    pub username_chosen: bool,
    pub avatar_url: Option<String>,
    pub level: i32,
    /// When the token expires, in seconds since the epoch.
    pub expires_at: i64,
}

/// QOR ID could not answer; the person is not to be treated as signed out.
#[derive(Debug, thiserror::Error)]
#[error("QOR ID could not be asked: {0}")]
pub struct Unreachable(pub String);

pub type Answer<'a> =
    Pin<Box<dyn Future<Output = Result<Option<Person>, Unreachable>> + Send + 'a>>;

/// Asks whether a token is a live session of a person's own. `None`: it is not.
pub trait Introspect: Send + Sync {
    fn check<'a>(&'a self, token: &'a str) -> Answer<'a>;
}

/// Answers kept, by the SHA-256 of the token they are for, with when each lapses.
type Answers = HashMap<[u8; 32], (Instant, Option<Person>)>;

/// QOR ID over HTTP.
pub struct QorId {
    http: reqwest::Client,
    endpoint: reqwest::Url,
    client_id: String,
    client_secret: String,
    cache: Mutex<Answers>,
}

impl std::fmt::Debug for QorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QorId")
            .field("endpoint", &self.endpoint.as_str())
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

/// How long an answer is kept, at most.
const KEEP: Duration = Duration::from_secs(30);

#[derive(Deserialize)]
struct Introspection {
    active: bool,
    sub: Option<Uuid>,
    qor_id: Option<String>,
    username_chosen: Option<bool>,
    avatar_url: Option<String>,
    exp: Option<i64>,
    progress: Option<serde_json::Value>,
}

impl QorId {
    /// `base` is QOR ID's origin, such as `https://id.qorsync.dev`.
    pub fn new(base: &str, client_id: String, client_secret: String) -> anyhow::Result<Self> {
        let endpoint = reqwest::Url::parse(base)?.join("/oauth/introspect")?;
        anyhow::ensure!(
            endpoint.scheme() == "https"
                || matches!(endpoint.host_str(), Some("127.0.0.1") | Some("localhost")),
            "QOR ID is reached over https, or on this machine"
        );
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Self {
            http,
            endpoint,
            client_id,
            client_secret,
            cache: Mutex::new(HashMap::new()),
        })
    }

    async fn ask(&self, token: &str) -> Result<Option<Person>, Unreachable> {
        let key: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let now = Instant::now();
        {
            let mut cache = self.cache.lock().await;
            cache.retain(|_, (until, _)| *until > now);
            if let Some((_, answer)) = cache.get(&key) {
                return Ok(answer.clone());
            }
        }

        let response = self
            .http
            .post(self.endpoint.clone())
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("token", token),
            ])
            .send()
            .await
            .map_err(|e| Unreachable(e.without_url().to_string()))?;
        if !response.status().is_success() {
            return Err(Unreachable(format!(
                "it answered {}",
                response.status().as_u16()
            )));
        }
        let answer: Introspection = response
            .json()
            .await
            .map_err(|_| Unreachable("its answer was not understood".into()))?;
        let person = match answer {
            Introspection {
                active: true,
                sub: Some(sub),
                qor_id: Some(qor_id),
                exp: Some(expires_at),
                username_chosen,
                avatar_url,
                progress,
            } => Some(Person {
                sub,
                qor_id,
                username_chosen: username_chosen.unwrap_or(true),
                avatar_url,
                level: progress
                    .as_ref()
                    .and_then(|p| p.get("level"))
                    .and_then(serde_json::Value::as_i64)
                    .and_then(|l| i32::try_from(l).ok())
                    .unwrap_or(0),
                expires_at,
            }),
            _ => None,
        };

        let keep = match &person {
            Some(p) => {
                let left = (p.expires_at - chrono::Utc::now().timestamp()).max(0) as u64;
                KEEP.min(Duration::from_secs(left))
            }
            None => KEEP,
        };
        self.cache
            .lock()
            .await
            .insert(key, (now + keep, person.clone()));
        Ok(person)
    }
}

impl Introspect for QorId {
    fn check<'a>(&'a self, token: &'a str) -> Answer<'a> {
        Box::pin(self.ask(token))
    }
}

/// People by token, for tests.
#[cfg(test)]
#[derive(Default)]
pub struct Fake {
    pub people: std::sync::Mutex<HashMap<String, Person>>,
}

#[cfg(test)]
impl Introspect for Fake {
    fn check<'a>(&'a self, token: &'a str) -> Answer<'a> {
        let person = self.people.lock().expect("lock").get(token).cloned();
        Box::pin(async move { Ok(person) })
    }
}

pub type Identity = Arc<dyn Introspect>;
