//! Email for QOR Auth, sent through the Resend HTTP API.
//!
//! Four messages: email address verification, password reset links, confirmation of a newly
//! added or changed address, and a notice to the old address that a change was requested.
//! Configuration comes from the environment, and nothing is committed:
//!
//! - `RESEND_API_KEY`: a Resend API key, best restricted to sending.
//! - `EMAIL_FROM`: the sender, on a domain verified in Resend, such as
//!   `Demiurge-Cloud <noreply@example.com>`.
//! - `BASE_URL`: where the links in those messages point.
//!
//! Without `RESEND_API_KEY` and `EMAIL_FROM` the service is not configured, and
//! every send is refused with 503, so callers fail closed. A message is never
//! logged, because it carries a verification or reset link.
//!
//! Every message is written once and rendered twice from the same words: as HTML in the design
//! system's Architect palette (`docs/design/DESIGN_SYSTEM.md`), and as plain text for clients
//! that show no HTML.

use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{info, warn};

use crate::error::{AppError, AppResult};

/// Resend's API, unless `RESEND_API_URL` names another.
const RESEND_API_URL: &str = "https://api.resend.com";

/// Email service configuration.
///
/// Deliberately not `Debug`, so the API key cannot be printed by accident.
#[derive(Clone)]
pub struct EmailConfig {
    /// Resend API key.
    pub resend_api_key: String,
    /// Sender, on a domain verified in Resend.
    pub from: String,
    /// Base of the links in verification and reset messages.
    pub base_url: String,
    /// Resend API base. Only HTTPS, or plain HTTP to this machine for tests, is used.
    pub api_url: String,
}

// There is deliberately no `Default`. It used to read the environment, so a test that wrote
// `EmailConfig::default()` meaning "nothing in particular" got whatever the machine it ran on had
// set: on the owner's computer, a real Resend key, and two test registrations were sent as real
// mail and bounced. Reading the environment now has to be asked for by name, and only `main` asks.
impl EmailConfig {
    /// The configuration the running service uses, read from the environment. Only `main` calls
    /// this; a test that did would depend on the machine it runs on, and a guard test fails on it.
    pub fn from_env() -> Self {
        Self {
            resend_api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            from: std::env::var("EMAIL_FROM").unwrap_or_default(),
            // No default: links must point at QOR ID's own origin, which only the deployment knows.
            base_url: std::env::var("BASE_URL").unwrap_or_default(),
            api_url: std::env::var("RESEND_API_URL").unwrap_or_else(|_| RESEND_API_URL.to_string()),
        }
    }

    /// For a test that sends no mail: no key and no sender, whatever the environment holds, and an
    /// API base on this machine that nothing listens on.
    #[cfg(test)]
    pub fn unconfigured() -> Self {
        Self {
            resend_api_key: String::new(),
            from: String::new(),
            base_url: "https://example.invalid".into(),
            api_url: "http://127.0.0.1:9".into(),
        }
    }
}

/// Whether the API base is on this machine, where a stand-in for Resend listens.
fn on_this_machine(url: &str) -> bool {
    reqwest::Url::parse(url).is_ok_and(|parsed| {
        parsed.scheme() == "http" && matches!(parsed.host_str(), Some("127.0.0.1" | "localhost"))
    })
}

/// Whether a service that `sends` through `api_url` puts mail on the network: it sends at all, and
/// not to a stand-in on this machine. Anything that is not plainly this machine counts as leaving,
/// so a URL this cannot read is never reported as safe.
fn mail_leaves_this_machine(sends: bool, api_url: &str) -> bool {
    sends && !on_this_machine(api_url)
}

/// Whether messages may be sent to this API base: HTTPS anywhere, or plain HTTP only to this
/// machine. The URL is parsed, so `http://localhost:@elsewhere` is not mistaken for localhost.
fn api_url_allowed(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    match parsed.scheme() {
        "https" => parsed.host_str().is_some(),
        "http" => matches!(parsed.host_str(), Some("127.0.0.1") | Some("localhost")),
        _ => false,
    }
}

/// Whether links may point at this base. QOR ID serves the pages its links open, on its own subdomain
/// (ADR-015), so the base is an origin of its own: HTTPS, or plain HTTP only to this machine, with no
/// path, query, fragment or credentials. Anything else would send a one-use token somewhere QOR ID
/// does not answer.
fn base_url_allowed(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    api_url_allowed(url)
        && parsed.path() == "/"
        && parsed.query().is_none()
        && parsed.fragment().is_none()
}

// The Architect theme (`tools/qor-launcher/src/styles/themes.ts`). A message is dark by design and
// says so with `color-scheme: dark`, so clients that honour it leave these colours alone, and the
// contrast holds in clients that invert them anyway. The accent is spent once: on the action.
const BASE: &str = "#0B0C10";
const SURFACE: &str = "#12151C";
const WELL: &str = "#0E1117";
const EDGE: &str = "#333A47";
const ACCENT: &str = "#FF6A00";
const ON_ACCENT: &str = "#06070A";
const INK: &str = "#FFFFFF";
const INK_BODY: &str = "#D7D8DA";
const INK_MUTED: &str = "#93A0AE";
const SANS: &str = "'Segoe UI Variable Text','Segoe UI',-apple-system,BlinkMacSystemFont,Roboto,Helvetica,Arial,sans-serif";
const MONO: &str = "'Cascadia Mono','SF Mono',Consolas,Menlo,monospace";

/// Closes every message. No address at the sending domain receives mail.
const FOOTER: &str = "You received this because this address was given for a QOR ID on Demiurge. \
                      Replies to this message are not received.";

/// What a message says, before it is rendered.
struct Message {
    /// Shown by most clients beside the subject in the inbox list, and nowhere in the message.
    preview: String,
    heading: &'static str,
    /// Paragraphs before the link.
    lead: Vec<String>,
    /// The one thing to do, if there is one.
    action: Option<Action>,
    /// Paragraphs after the link.
    after: Vec<String>,
}

struct Action {
    label: &'static str,
    url: String,
}

/// How an address is kept on the list of undeliverable addresses: a SHA-256 of it, trimmed and
/// lower-cased, so the list holds no address in the clear.
pub fn address_hash(address: &str) -> String {
    hex::encode(Sha256::digest(address.trim().to_lowercase().as_bytes()))
}

pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// HTML for email clients: tables and inline styles only, since many clients drop `<style>`, and no
/// images, so nothing is blocked or tracked.
fn render_html(message: &Message) -> String {
    let paragraphs = |texts: &[String]| -> String {
        texts
            .iter()
            .map(|text| {
                let text = escape(text);
                format!(
                    r#"<p style="margin:0 0 16px 0;font-family:{SANS};font-size:15px;line-height:24px;color:{INK_BODY};">{text}</p>"#
                )
            })
            .collect()
    };

    // The address is shown in full under the button, as the text of its own link. A client that
    // rewrites links for scanning still shows the real address, so it can be copied if the
    // rewritten one fails.
    let action = match &message.action {
        Some(action) => {
            let label = escape(action.label);
            let url = escape(&action.url);
            format!(
                r#"<table role="presentation" cellpadding="0" cellspacing="0" border="0" style="margin:8px 0 24px 0;">
<tr><td bgcolor="{ACCENT}" style="background-color:{ACCENT};border-radius:2px;">
<a href="{url}" target="_blank" style="display:inline-block;padding:12px 24px;font-family:{SANS};font-size:13px;line-height:16px;font-weight:600;letter-spacing:0.1em;text-transform:uppercase;color:{ON_ACCENT};text-decoration:none;border-radius:2px;">{label}</a>
</td></tr>
</table>
<p style="margin:0 0 8px 0;font-family:{SANS};font-size:12px;line-height:18px;color:{INK_MUTED};">If the button does not work, copy this address into your browser:</p>
<p style="margin:0 0 24px 0;padding:12px;background-color:{WELL};border:1px solid {EDGE};border-radius:2px;font-family:{MONO};font-size:13px;line-height:20px;word-break:break-all;"><a href="{url}" target="_blank" style="color:{INK};text-decoration:underline;">{url}</a></p>
"#
            )
        }
        None => String::new(),
    };

    let preview = escape(&message.preview);
    let heading = escape(message.heading);
    let lead = paragraphs(&message.lead);
    let after = paragraphs(&message.after);
    let footer = escape(FOOTER);
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark">
<meta name="supported-color-schemes" content="dark">
<title>{heading}</title>
</head>
<body style="margin:0;padding:0;background-color:{BASE};">
<div style="display:none;max-height:0;overflow:hidden;opacity:0;mso-hide:all;">{preview}</div>
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" bgcolor="{BASE}" style="background-color:{BASE};">
<tr><td align="center" style="padding:32px 16px;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0" style="max-width:560px;">
<tr><td style="padding:0 0 16px 0;font-family:{SANS};font-size:11px;line-height:16px;font-weight:600;letter-spacing:0.18em;text-transform:uppercase;color:{INK_MUTED};">Demiurge &middot; QOR ID</td></tr>
<tr><td bgcolor="{SURFACE}" style="background-color:{SURFACE};border:1px solid {EDGE};border-radius:2px;padding:32px;">
<h1 style="margin:0 0 16px 0;font-family:{SANS};font-size:24px;line-height:32px;font-weight:600;color:{INK};">{heading}</h1>
{lead}{action}{after}</td></tr>
<tr><td style="padding:16px 0 0 0;font-family:{SANS};font-size:12px;line-height:18px;color:{INK_MUTED};">{footer}</td></tr>
</table>
</td></tr>
</table>
</body>
</html>
"#
    )
}

/// Plain text for clients that show no HTML. A link stands alone on its line, with a blank line
/// either side, so no client takes punctuation as part of it.
fn render_text(message: &Message) -> String {
    let mut out = format!("{}\n\n", message.heading);
    for paragraph in &message.lead {
        out.push_str(paragraph);
        out.push_str("\n\n");
    }
    if let Some(action) = &message.action {
        out.push_str(&action.url);
        out.push_str("\n\n");
    }
    for paragraph in &message.after {
        out.push_str(paragraph);
        out.push_str("\n\n");
    }
    out.push_str("-- \n");
    out.push_str(FOOTER);
    out.push('\n');
    out
}

/// Email service for sending transactional emails
pub struct EmailService {
    config: EmailConfig,
    http: Option<reqwest::Client>,
    /// Where undeliverable addresses are listed (`with_suppression_list`).
    suppressions: Option<PgPool>,
    /// The signing secret of Resend's webhook endpoint (`with_webhook_secret`).
    webhook_secret: Option<String>,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: EmailConfig) -> Self {
        let http = if config.resend_api_key.is_empty() || config.from.is_empty() {
            warn!("Email is not configured (RESEND_API_KEY and EMAIL_FROM): no email will be sent");
            None
        } else if !api_url_allowed(&config.api_url) {
            warn!("RESEND_API_URL must be HTTPS, or HTTP to this machine: no email will be sent");
            None
        } else if !base_url_allowed(&config.base_url) {
            warn!(
                "BASE_URL must be QOR ID's own origin, HTTPS or HTTP to this machine, with no path: no email will be sent"
            );
            None
        } else {
            match reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
            {
                Ok(client) => Some(client),
                Err(e) => {
                    warn!("The HTTP client for Resend could not be built: {}", e);
                    None
                }
            }
        };

        // In a test build, mail goes to a stand-in on this machine or nowhere. Whatever a test is
        // configured with, by mistake or from the environment, it cannot reach Resend.
        #[cfg(test)]
        let http = http.filter(|_| on_this_machine(&config.api_url));

        Self {
            config,
            http,
            suppressions: None,
            webhook_secret: None,
        }
    }

    /// Check the list of undeliverable addresses before every send. `AppState::new` attaches it,
    /// so every service that handlers use has it.
    pub fn with_suppression_list(mut self, db: PgPool) -> Self {
        self.suppressions = Some(db);
        self
    }

    /// The signing secret of Resend's webhook endpoint (`whsec_…`), from `RESEND_WEBHOOK_SECRET`.
    /// Without it, every webhook delivery is refused.
    pub fn with_webhook_secret(mut self, secret: Option<String>) -> Self {
        self.webhook_secret = secret.filter(|secret| !secret.trim().is_empty());
        self
    }

    pub fn webhook_secret(&self) -> Option<&str> {
        self.webhook_secret.as_deref()
    }

    /// Whether the address is marked undeliverable: it bounced permanently, Resend suppressed it,
    /// or its owner marked a message from us as spam.
    pub async fn is_suppressed(&self, address: &str) -> AppResult<bool> {
        let Some(db) = &self.suppressions else {
            return Ok(false);
        };
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM email_suppressions WHERE address_hash = $1)",
        )
        .bind(address_hash(address))
        .fetch_one(db)
        .await?)
    }

    fn link(&self, path: &str, token: &str) -> String {
        format!(
            "{}/{}?token={}",
            self.config.base_url.trim_end_matches('/'),
            path,
            token
        )
    }

    /// Send a message through Resend, as HTML with a plain-text part. The message is never logged.
    async fn send_email(&self, to: &str, subject: &str, message: &Message) -> AppResult<()> {
        let Some(http) = &self.http else {
            warn!(
                "Email not sent, because email is not configured: {}",
                subject
            );
            return Err(AppError::ServiceUnavailable(
                "Email is not configured on this service".into(),
            ));
        };

        // An address that bounced permanently, or whose owner complained, is not sent to again.
        if self.is_suppressed(to).await? {
            warn!(
                "Email not sent, because the address is marked undeliverable: {}",
                subject
            );
            return Err(AppError::Undeliverable(
                "That address does not accept email from this service. Use a different address."
                    .into(),
            ));
        }

        let response = http
            .post(format!(
                "{}/emails",
                self.config.api_url.trim_end_matches('/')
            ))
            .bearer_auth(&self.config.resend_api_key)
            .json(&json!({
                "from": self.config.from,
                "to": [to],
                "subject": subject,
                "html": render_html(message),
                "text": render_text(message),
            }))
            .send()
            .await
            .map_err(|e| {
                AppError::ServiceUnavailable(format!("Resend could not be reached: {}", e))
            })?;

        let status = response.status();
        let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
        if !status.is_success() {
            // Resend's error names the problem, such as an unverified domain or a bad key.
            let reason = body
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("no reason given");
            return Err(AppError::ServiceUnavailable(format!(
                "Resend refused the email ({}): {}",
                status.as_u16(),
                reason
            )));
        }

        let id = body.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
        info!("Email accepted by Resend: {} ({})", id, subject);
        Ok(())
    }

    /// Ask an address given at registration, or given again for a new link, to confirm itself.
    pub async fn send_verification_email(
        &self,
        to: &str,
        username: &str,
        token: &str,
    ) -> AppResult<()> {
        let message = Message {
            preview: format!("Confirm this address for {username}. The link lasts 24 hours."),
            heading: "Confirm your email address",
            lead: vec![
                format!(
                    "This address was given for the QOR ID {username}. Once it is confirmed, password reset links can come here."
                ),
                "To confirm it, open the link below.".into(),
            ],
            action: Some(Action {
                label: "Confirm email address",
                url: self.link("verify-email", token),
            }),
            after: vec![
                "The link lasts 24 hours and works once. If you asked for a new link, only the newest one works.".into(),
                "If you did not create this account, ignore this message. The address will not be confirmed.".into(),
            ],
        };
        self.send_email(to, "Confirm your email address for QOR ID", &message)
            .await
    }

    /// Send a password reset link to an account's verified address.
    pub async fn send_password_reset_email(
        &self,
        to: &str,
        username: &str,
        token: &str,
    ) -> AppResult<()> {
        let message = Message {
            preview: format!(
                "A password reset was requested for {username}. The link lasts 1 hour."
            ),
            heading: "Reset your password",
            lead: vec![
                format!("Someone asked to reset the password for the QOR ID {username}."),
                "To choose a new password, open the link below.".into(),
            ],
            action: Some(Action {
                label: "Reset password",
                url: self.link("reset-password", token),
            }),
            after: vec![
                "The link lasts 1 hour and works once. Resetting the password signs out every session on the account.".into(),
                "If you did not ask for this, ignore this message. Your password stays as it is.".into(),
            ],
        };
        self.send_email(to, "Reset your QOR ID password", &message)
            .await
    }

    /// Ask a newly added or changed address to confirm itself before it goes on the account.
    pub async fn send_email_change_verification(
        &self,
        to: &str,
        username: &str,
        token: &str,
    ) -> AppResult<()> {
        let message = Message {
            preview: format!("Confirm this address for {username}. The link lasts 24 hours."),
            heading: "Confirm your new email address",
            lead: vec![
                format!(
                    "The QOR ID {username} asked to use this address. Once it is confirmed, it becomes the account's email address, and password reset links come here."
                ),
                "To confirm it, open the link below.".into(),
            ],
            action: Some(Action {
                label: "Confirm email address",
                url: self.link("verify-email", token),
            }),
            after: vec![
                "The link lasts 24 hours and works once. Until it is used, the account does not change.".into(),
                "If you did not ask for this, ignore this message. The address will not be added.".into(),
            ],
        };
        self.send_email(to, "Confirm your new email address for QOR ID", &message)
            .await
    }

    /// Tell an account's current address that a change to another address was requested.
    /// It carries no link: it only warns.
    pub async fn send_email_change_notice(&self, to: &str, username: &str) -> AppResult<()> {
        let message = Message {
            preview: "The change completes only if the new address is confirmed.".into(),
            heading: "Your email address is being changed",
            lead: vec![
                format!(
                    "Someone signed in to the QOR ID {username} with its password and asked to change its email address."
                ),
                "The change completes only when the new address is confirmed. Until then this address stays on the account, and password reset links still come here.".into(),
            ],
            action: None,
            after: vec![
                "If this was you, there is nothing to do.".into(),
                "If it was not you, reset your password now. A reset signs out every session and cancels the change.".into(),
            ],
        };
        self.send_email(
            to,
            "A change of email address was requested for your QOR ID",
            &message,
        )
        .await
    }

    /// Check if email service is configured
    pub fn is_configured(&self) -> bool {
        self.http.is_some()
    }

    /// Whether a message sent by this service leaves this machine, for a real inbox. False when
    /// nothing is sent, and when it goes to a stand-in for Resend on this machine.
    ///
    /// `/health` reports it, so the end-to-end scripts, which register addresses nobody holds, can
    /// refuse to run against a service that would send them as real mail. It is one boolean, and
    /// says nothing of the key, the sender or where the links point.
    pub fn leaves_this_machine(&self) -> bool {
        mail_leaves_this_machine(self.http.is_some(), &self.config.api_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Json as AxumJson, Router,
        extract::State as AxumState,
        http::{HeaderMap, StatusCode},
        routing::post,
    };
    use std::sync::{Arc, Mutex};

    fn configured(api_url: String) -> EmailConfig {
        EmailConfig {
            resend_api_key: "re_test_key".into(),
            from: "Demiurge-Cloud <noreply@example.invalid>".into(),
            base_url: "https://example.invalid".into(),
            api_url,
        }
    }

    type Received = Arc<Mutex<Vec<(Option<String>, serde_json::Value)>>>;

    /// A stand-in for Resend's `POST /emails`, on a free local port. It records the
    /// authorisation header and body of every request and answers with `status`.
    async fn fake_resend(status: StatusCode) -> (String, Received) {
        let received: Received = Arc::default();
        let app = Router::new()
            .route(
                "/emails",
                post(
                    move |AxumState(received): AxumState<Received>,
                          headers: HeaderMap,
                          AxumJson(body): AxumJson<serde_json::Value>| async move {
                        let auth = headers
                            .get("authorization")
                            .and_then(|v| v.to_str().ok())
                            .map(str::to_string);
                        received.lock().expect("lock").push((auth, body));
                        let reply = if status.is_success() {
                            json!({ "id": "email-123" })
                        } else {
                            json!({
                                "statusCode": status.as_u16(),
                                "name": "validation_error",
                                "message": "The example.invalid domain is not verified"
                            })
                        };
                        (status, AxumJson(reply))
                    },
                ),
            )
            .with_state(received.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", listener.local_addr().expect("address"));
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
        (url, received)
    }

    /// Sends all four messages through a stand-in, and returns what it received, in order.
    async fn all_four() -> Vec<serde_json::Value> {
        let (url, received) = fake_resend(StatusCode::OK).await;
        let service = EmailService::new(configured(url));
        let to = "someone@example.invalid";
        service
            .send_verification_email(to, "someone", "verify-me")
            .await
            .expect("verification");
        service
            .send_password_reset_email(to, "someone", "reset-me")
            .await
            .expect("reset");
        service
            .send_email_change_verification(to, "someone", "confirm-me")
            .await
            .expect("confirmation");
        service
            .send_email_change_notice(to, "someone")
            .await
            .expect("notice");
        let received = received.lock().expect("lock");
        received.iter().map(|(_, body)| body.clone()).collect()
    }

    #[tokio::test]
    async fn an_unconfigured_service_refuses_rather_than_pretending_to_send() {
        let service = EmailService::new(EmailConfig::unconfigured());
        assert!(!service.is_configured());
        assert!(matches!(
            service
                .send_password_reset_email("a@example.invalid", "a", "secret-token")
                .await,
            Err(AppError::ServiceUnavailable(_))
        ));
        assert!(matches!(
            service
                .send_verification_email("a@example.invalid", "a", "secret-token")
                .await,
            Err(AppError::ServiceUnavailable(_))
        ));
    }

    #[tokio::test]
    async fn a_reset_link_goes_through_resend_with_the_key_sender_and_recipient() {
        let (url, received) = fake_resend(StatusCode::OK).await;
        let service = EmailService::new(configured(url));
        assert!(service.is_configured());

        service
            .send_password_reset_email("someone@example.invalid", "someone", "the-token")
            .await
            .expect("accepted");

        let received = received.lock().expect("lock");
        assert_eq!(received.len(), 1);
        let (auth, body) = &received[0];
        assert_eq!(auth.as_deref(), Some("Bearer re_test_key"));
        assert_eq!(
            body["from"],
            json!("Demiurge-Cloud <noreply@example.invalid>")
        );
        assert_eq!(body["to"], json!(["someone@example.invalid"]));
        assert!(
            body["html"]
                .as_str()
                .expect("html")
                .contains("https://example.invalid/reset-password?token=the-token")
        );
    }

    #[tokio::test]
    async fn a_verification_link_goes_through_resend() {
        let (url, received) = fake_resend(StatusCode::OK).await;
        let service = EmailService::new(configured(url));

        service
            .send_verification_email("someone@example.invalid", "someone", "verify-me")
            .await
            .expect("accepted");

        let received = received.lock().expect("lock");
        assert_eq!(received.len(), 1);
        assert!(
            received[0].1["html"]
                .as_str()
                .expect("html")
                .contains("https://example.invalid/verify-email?token=verify-me")
        );
    }

    #[tokio::test]
    async fn every_message_has_a_plain_text_part_and_none_of_the_old_styling() {
        let messages = all_four().await;
        assert_eq!(messages.len(), 4);
        for body in &messages {
            let subject = body["subject"].as_str().expect("subject");
            let html = body["html"].as_str().expect("html");
            let text = body["text"].as_str().expect("a plain-text part");

            assert!(
                html.contains(r#"<meta name="color-scheme" content="dark">"#),
                "{subject}: dark by declaration, so clients do not repaint it"
            );
            let lower = html.to_lowercase();
            for banned in [
                "gradient",
                "shadow",
                "<style",
                "<img",
                "#00ffff",
                "#ff00ff",
                "#00ff88",
                "metaverse",
                "pleroma",
            ] {
                assert!(!lower.contains(banned), "{subject}: contains {banned}");
            }

            assert!(text.contains("someone"), "{subject}: names the account");
            assert!(
                !text.contains('<'),
                "{subject}: the text part has no markup"
            );
            assert!(text.contains("Replies to this message are not received."));
        }
    }

    #[tokio::test]
    async fn a_link_stands_alone_in_text_and_is_shown_as_well_as_linked_in_html() {
        let messages = all_four().await;
        let links = [
            "https://example.invalid/verify-email?token=verify-me",
            "https://example.invalid/reset-password?token=reset-me",
            "https://example.invalid/verify-email?token=confirm-me",
        ];
        for (body, link) in messages.iter().zip(links) {
            let html = body["html"].as_str().expect("html");
            let text = body["text"].as_str().expect("text");
            // The button and the visible address both link; the address is also the link's text,
            // so a client that rewrites the href still shows where it goes.
            assert_eq!(html.matches(&format!(r#"href="{link}""#)).count(), 2);
            assert!(html.contains(&format!(">{link}</a>")));
            assert!(text.contains(&format!("\n\n{link}\n\n")));
        }
    }

    #[tokio::test]
    async fn the_change_notice_carries_no_link() {
        let messages = all_four().await;
        let notice = &messages[3];
        assert!(!notice["html"].as_str().expect("html").contains("href"));
        assert!(!notice["text"].as_str().expect("text").contains("http"));
    }

    #[test]
    fn words_are_escaped_in_html_and_left_as_written_in_text() {
        let message = Message {
            preview: "a <b> & c".into(),
            heading: "Heading",
            lead: vec!["<script>alert(1)</script>".into()],
            action: Some(Action {
                label: "Go",
                url: "https://example.invalid/?a=1&b=\"2\"".into(),
            }),
            after: vec![],
        };
        let html = render_html(&message);
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
        assert!(html.contains("a &lt;b&gt; &amp; c"));
        assert!(html.contains(r#"href="https://example.invalid/?a=1&amp;b=&quot;2&quot;""#));
        assert!(render_text(&message).contains("<script>alert(1)</script>"));
    }

    #[tokio::test]
    async fn a_refusal_from_resend_is_an_error_that_never_carries_the_link() {
        let (url, _) = fake_resend(StatusCode::UNPROCESSABLE_ENTITY).await;
        let service = EmailService::new(configured(url));

        let error = service
            .send_password_reset_email("someone@example.invalid", "someone", "the-token")
            .await
            .expect_err("refused");
        let text = format!("{error:?}");
        assert!(matches!(error, AppError::ServiceUnavailable(_)));
        assert!(text.contains("422") && text.contains("not verified"));
        assert!(!text.contains("the-token"));
    }

    #[test]
    fn links_point_only_at_an_origin_of_qor_ids_own() {
        let with_base = |base: &str| EmailConfig {
            base_url: base.into(),
            ..configured("http://127.0.0.1:4000".into())
        };
        for allowed in [
            "https://auth.example.invalid",
            "https://auth.example.invalid/",
            "http://127.0.0.1:3100",
            "http://localhost:3100/",
        ] {
            assert!(
                EmailService::new(with_base(allowed)).is_configured(),
                "{allowed}"
            );
        }
        for refused in [
            "",
            "auth.example.invalid",
            "http://auth.example.invalid",
            "https://example.invalid/auth",
            "https://example.invalid/?next=1",
            "https://example.invalid/#top",
            "https://user:pass@example.invalid",
        ] {
            assert!(
                !EmailService::new(with_base(refused)).is_configured(),
                "{refused} must be refused"
            );
        }
    }

    /// Two test registrations were once sent as real mail, and bounced: their helper built the
    /// service from the environment, and the machine had a real key set. However a test is
    /// configured, it must not be able to reach Resend.
    #[tokio::test]
    async fn a_test_build_sends_only_to_a_stand_in_on_this_machine() {
        for elsewhere in [
            RESEND_API_URL,
            "https://api.resend.com/",
            "https://mail.example",
        ] {
            // Everything a real deployment has: a key, a sender and an origin for the links.
            let service = EmailService::new(configured(elsewhere.into()));
            assert!(
                !service.is_configured(),
                "a test build must refuse to send through {elsewhere}"
            );
            assert!(matches!(
                service
                    .send_verification_email("someone@example.invalid", "someone", "verify-me")
                    .await,
                Err(AppError::ServiceUnavailable(_))
            ));
        }

        // The service `main` would build from this machine's environment, whatever that holds, is
        // either unconfigured or pointed at this machine. Nothing is sent here.
        let from_this_machine = EmailConfig::from_env();
        assert!(
            !EmailService::new(from_this_machine.clone()).is_configured()
                || on_this_machine(&from_this_machine.api_url)
        );

        assert!(!EmailService::new(EmailConfig::unconfigured()).is_configured());
        assert!(EmailService::new(configured("http://127.0.0.1:9".into())).is_configured());
    }

    fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("the source directory") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                out.push(path);
            }
        }
    }

    /// The environment is read for email in one place, and asked for in one place. A test helper
    /// that reads it gets whatever the machine has set, which is how real mail was sent.
    #[test]
    fn only_main_builds_the_email_service_from_the_environment() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = Vec::new();
        rust_files(&src, &mut files);

        // Assembled, so this test does not find itself.
        let type_name = "EmailConfig";
        let from_env = format!("{type_name}::from_env(");
        let default_call = format!("{type_name}::default(");
        let default_impl = format!("Default for {type_name}");
        let variables: Vec<String> = ["RESEND_API_KEY", "EMAIL_FROM", "RESEND_API_URL", "BASE_URL"]
            .iter()
            .map(|name| format!("var(\"{name}\")"))
            .collect();

        let mut callers = Vec::new();
        let mut problems = Vec::new();
        for path in &files {
            let name = path
                .strip_prefix(&src)
                .expect("under src")
                .to_string_lossy()
                .replace('\\', "/");
            let source = std::fs::read_to_string(path).expect("source");
            for (number, line) in source.lines().enumerate() {
                if line.trim_start().starts_with("//") {
                    continue;
                }
                let at = format!("{name}:{}", number + 1);
                if line.contains(&from_env) {
                    callers.push(at.clone());
                }
                if line.contains(&default_call) || line.contains(&default_impl) {
                    problems.push(format!(
                        "{at}: {type_name} has a default again; a default that reads the environment sent real mail from a test"
                    ));
                }
                if name != "services/email_service.rs"
                    && variables.iter().any(|variable| line.contains(variable))
                {
                    problems.push(format!(
                        "{at}: reads an email setting from the environment outside the email service"
                    ));
                }
            }
        }

        assert!(problems.is_empty(), "{}", problems.join("\n"));
        // This file's own call is the guard above, which sends nothing.
        let outside: Vec<&String> = callers
            .iter()
            .filter(|at| !at.starts_with("services/email_service.rs:"))
            .collect();
        assert_eq!(
            outside.len(),
            1,
            "only main builds the email service from the environment; found {callers:?}"
        );
        assert!(
            outside[0].starts_with("main.rs:"),
            "only main builds the email service from the environment; found {callers:?}"
        );
    }

    /// The end-to-end scripts run only against a service that reports false here, so a wrong
    /// "false" is real mail to addresses nobody holds. A test build cannot hold a service that sends
    /// elsewhere (the guard above), so the rule is checked on its own.
    #[test]
    fn mail_leaves_this_machine_unless_nothing_is_sent_or_it_goes_to_a_stand_in() {
        for elsewhere in [
            RESEND_API_URL,
            "https://api.resend.com/",
            "https://mail.example",
            "https://127.0.0.1:4000",
            "http://127.0.0.1.elsewhere.example",
            "http://localhost:@elsewhere.example",
            "not a url",
        ] {
            assert!(
                mail_leaves_this_machine(true, elsewhere),
                "{elsewhere} is not this machine"
            );
            assert!(
                !mail_leaves_this_machine(false, elsewhere),
                "a service that sends nothing sends nothing through {elsewhere}"
            );
        }
        for stand_in in ["http://127.0.0.1:59925", "http://localhost:59925/"] {
            assert!(!mail_leaves_this_machine(true, stand_in), "{stand_in}");
        }

        // As the service reports it: unconfigured, and configured for a stand-in.
        assert!(!EmailService::new(EmailConfig::unconfigured()).leaves_this_machine());
        let stand_in = EmailService::new(configured("http://127.0.0.1:9".into()));
        assert!(stand_in.is_configured() && !stand_in.leaves_this_machine());
    }

    #[test]
    fn messages_go_only_to_https_or_to_this_machine() {
        assert!(api_url_allowed("https://api.resend.com"));
        assert!(api_url_allowed("http://127.0.0.1:4000"));
        assert!(api_url_allowed("http://localhost:4000"));
        for refused in [
            "http://api.resend.com",
            "http://localhost:@elsewhere.example",
            "http://127.0.0.1.elsewhere.example",
            "ftp://api.resend.com",
            "not a url",
        ] {
            assert!(!api_url_allowed(refused), "{refused} must be refused");
        }
        assert!(!EmailService::new(configured("http://api.resend.com".into())).is_configured());
    }
}
