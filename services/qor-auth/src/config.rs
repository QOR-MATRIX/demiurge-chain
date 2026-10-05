//! Application configuration module.
//!
//! Loads configuration from environment variables and config files.

use config::{Config, Environment, File};
use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub security: SecurityConfig,
    pub chain: ChainConfig,
    /// The apps that may sign people in through QOR ID's own page (ADR-043).
    #[serde(default)]
    pub oauth: OAuthConfig,
}

/// Sign-in for other apps by redirect: OAuth 2.1 authorization code with PKCE (ADR-043).
///
/// Clients are registered here, by configuration, because every one is first-party: ARQADE today.
/// Third-party sign-in is ADR-043's excluded case. The list comes from the `QOR_OAUTH_CLIENTS`
/// variable, a JSON array, so a deployment registers an app without a code change.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthConfig {
    #[serde(default)]
    pub clients: Vec<OAuthClient>,
    /// How long a signed-in code may wait to be exchanged. Seconds.
    pub code_ttl_secs: u64,
    /// How long the sign-in page stays usable once opened. Seconds.
    pub request_ttl_secs: u64,
    /// How long an app's session lives, refresh tokens included. ADR-043 asks for short; the number
    /// is an engineering choice recorded in ADR-073. Seconds.
    pub session_lifetime_secs: i64,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            clients: vec![],
            code_ttl_secs: 60,
            request_ttl_secs: 600,
            session_lifetime_secs: 8 * 60 * 60,
        }
    }
}

/// One registered app.
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthClient {
    /// What the app sends as `client_id`.
    pub id: String,
    /// What the sign-in page calls it.
    pub name: String,
    /// The only addresses a code is ever sent to. Compared exactly.
    pub redirect_uris: Vec<String>,
    /// For an app with a server (a confidential client): the SHA-256 of its secret, in hex. The secret
    /// itself is never stored here. Absent for an app that cannot keep a secret; PKCE is required
    /// either way.
    #[serde(default)]
    pub secret_sha256: Option<String>,
}

impl OAuthConfig {
    pub fn client(&self, id: &str) -> Option<&OAuthClient> {
        self.clients.iter().find(|c| c.id == id)
    }

    /// Refuse a registry that would send codes somewhere it should not.
    pub fn validate(&self) -> anyhow::Result<()> {
        let mut seen = std::collections::HashSet::new();
        for c in &self.clients {
            if c.id.is_empty()
                || c.id.len() > 64
                || !c
                    .id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
            {
                anyhow::bail!("an OAuth client id must be 1 to 64 letters, digits, '-' or '_'");
            }
            if !seen.insert(c.id.as_str()) {
                anyhow::bail!("OAuth client {} is registered twice", c.id);
            }
            if c.redirect_uris.is_empty() {
                anyhow::bail!("OAuth client {} has no redirect URI", c.id);
            }
            for uri in &c.redirect_uris {
                let url = reqwest::Url::parse(uri).map_err(|_| {
                    anyhow::anyhow!("OAuth client {} has a redirect URI that is not a URL", c.id)
                })?;
                let loopback = matches!(
                    url.host_str(),
                    Some("127.0.0.1") | Some("localhost") | Some("[::1]")
                );
                if !(url.scheme() == "https" || (url.scheme() == "http" && loopback)) {
                    anyhow::bail!(
                        "OAuth client {} has a redirect URI that is neither https nor loopback",
                        c.id
                    );
                }
                if url.fragment().is_some() {
                    anyhow::bail!("OAuth client {} has a redirect URI with a fragment", c.id);
                }
            }
            if let Some(hash) = &c.secret_sha256
                && (hash.len() != 64 || !hash.chars().all(|ch| ch.is_ascii_hexdigit()))
            {
                anyhow::bail!(
                    "OAuth client {}: secret_sha256 must be 64 hexadecimal characters",
                    c.id
                );
            }
        }
        if self.code_ttl_secs == 0 || self.code_ttl_secs > 600 {
            anyhow::bail!("oauth.code_ttl_secs must be between 1 and 600");
        }
        if self.session_lifetime_secs <= 0 {
            anyhow::bail!("oauth.session_lifetime_secs must be above zero");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
    /// Browser origins allowed to make cross-origin requests.
    ///
    /// **Never `*`.** Until 21 September 2026 the service answered
    /// `Access-Control-Allow-Origin: *` on every route, including
    /// `/api/v1/admin/*`, which let any page in any tab script this API from a
    /// visitor's browser and read the response. Bearer tokens made that less
    /// dangerous than cookies would have been, but nothing constrained which
    /// frontends could talk to QOR ID, and there was no list to check a
    /// redirect against.
    ///
    /// It is configuration, not a constant, because the origins are a
    /// deployment fact: localhost while developing, and the real frontend
    /// hostnames once they exist. An empty list means no cross-origin browser
    /// request is allowed at all, which is the right answer for a deployment
    /// that serves no browser frontend.
    #[serde(default)]
    pub allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// Secret for signing access tokens
    pub access_secret: String,
    /// Secret for signing refresh tokens
    pub refresh_secret: String,
    /// Access token expiry in seconds (default: 900 = 15 minutes)
    pub access_expiry_secs: i64,
    /// Refresh token expiry in seconds (default: 2592000 = 30 days)
    pub refresh_expiry_secs: i64,
    /// Issuer claim
    pub issuer: String,
}

/// What QOR ID needs to know about the chain its accounts live on.
///
/// It holds no key and calls no node (R-3): this is only how an account is
/// read and shown.
#[derive(Debug, Clone, Deserialize)]
pub struct ChainConfig {
    /// The SS58 prefix addresses are written and accepted at (ADR-024).
    ///
    /// Configured rather than compiled in, because the mainnet prefix is not
    /// decided and is a Public Release criterion. **It is not chain
    /// identification**: prefix 42 is shared by many networks, and what tells
    /// them apart is the genesis hash (ADR-024, clarified 17 September 2026).
    /// Refusing another prefix catches a Polkadot or Kusama address pasted by
    /// mistake, and nothing more.
    pub ss58_prefix: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SecurityConfig {
    /// Maximum login attempts before lockout
    pub max_login_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration_secs: i64,
    /// Password minimum length
    pub password_min_length: usize,
}

/// The origins a developer's browser uses. Loopback only, both spellings,
/// and both the Vite dev server and the Tauri devtools port.
fn default_dev_origins() -> Vec<String> {
    vec![
        "http://localhost:1420".to_string(),
        "http://127.0.0.1:1420".to_string(),
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
    ]
}

impl AppConfig {
    /// Load configuration from environment and files
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            // Start with default values
            .set_default("server.host", "0.0.0.0")?
            .set_default("server.port", 3000)?
            .set_default("server.environment", env.as_str())?
            .set_default("database.max_connections", 10)?
            .set_default("jwt.access_expiry_secs", 900)?
            .set_default("jwt.refresh_expiry_secs", 2592000)?
            .set_default("jwt.issuer", "qor-auth")?
            .set_default("security.max_login_attempts", 5)?
            .set_default("security.lockout_duration_secs", 900)?
            .set_default("security.password_min_length", 12)?
            // Development only. A deployment sets its real frontend origins,
            // and a deployment that serves no browser frontend sets none.
            .set_default("server.allowed_origins", default_dev_origins())?
            // Empty, so that deserialisation SUCCEEDS and the flat overrides
            // below get their turn. Before 21 September 2026 these four had no
            // default at all, so `try_deserialize` failed with "missing field
            // `access_secret`" before a single flat name was read -- which meant
            // the flat names this file's own comment promises to accept did not
            // work, and a deployer following it got a service that would not
            // boot. `validate()` refuses an empty value, so nothing starts on a
            // blank secret; the failure just happens in the place that can name
            // both spellings of the variable.
            .set_default("database.url", "")?
            .set_default("redis.url", "")?
            .set_default("jwt.access_secret", "")?
            .set_default("jwt.refresh_secret", "")?
            // 42, the generic Substrate prefix, is what development and test
            // networks use (ADR-024). The runtime declares the same number, so
            // the service and the chain show one address for one account.
            .set_default("chain.ss58_prefix", 42)?
            // Load config file based on environment
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            // Override with environment variables (QOR_AUTH_*)
            .add_source(
                Environment::with_prefix("QOR_AUTH")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let mut cfg: Self = config.try_deserialize()?;

        // Accept the flat variable names documented in .env.example. These take
        // precedence over the nested QOR_AUTH__JWT__* form when both are set.
        if let Ok(v) = std::env::var("JWT_ACCESS_SECRET") {
            cfg.jwt.access_secret = v;
        }
        if let Ok(v) = std::env::var("JWT_REFRESH_SECRET") {
            cfg.jwt.refresh_secret = v;
        }
        if let Ok(v) = std::env::var("DATABASE_URL") {
            cfg.database.url = v;
        }
        if let Ok(v) = std::env::var("REDIS_URL") {
            cfg.redis.url = v;
        }
        // The registered apps, as one JSON array (see `OAuthConfig`).
        if let Ok(v) = std::env::var("QOR_OAUTH_CLIENTS") {
            cfg.oauth.clients = serde_json::from_str(&v)
                .map_err(|_| anyhow::anyhow!("QOR_OAUTH_CLIENTS is not a JSON array of clients"))?;
        }
        // Railway, Heroku and most container platforms assign the port and pass
        // it as PORT. Honoured last so an explicit QOR_AUTH__SERVER__PORT can
        // still win on a host that sets PORT for its own reasons.
        if let (Ok(v), Err(_)) = (
            std::env::var("PORT"),
            std::env::var("QOR_AUTH__SERVER__PORT"),
        ) && let Ok(port) = v.parse::<u16>()
        {
            cfg.server.port = port;
        }

        cfg.validate()?;
        Ok(cfg)
    }

    /// Reject configurations that would run with placeholder or weak secrets.
    ///
    /// This fails closed at startup rather than issuing forgeable tokens.
    fn validate(&self) -> anyhow::Result<()> {
        const PLACEHOLDERS: [&str; 6] = [
            "CHANGE_ME_ACCESS_SECRET",
            "CHANGE_ME_REFRESH_SECRET",
            "your-super-secret-access-key-minimum-32-characters-change-in-production",
            "your-super-secret-refresh-key-minimum-32-characters-change-in-production",
            "changeme",
            "secret",
        ];

        for (label, nested, value) in [
            (
                "DATABASE_URL",
                "QOR_AUTH__DATABASE__URL",
                &self.database.url,
            ),
            ("REDIS_URL", "QOR_AUTH__REDIS__URL", &self.redis.url),
        ] {
            if value.trim().is_empty() {
                anyhow::bail!("{label} is not set (or set {nested})");
            }
        }

        for (label, secret) in [
            ("JWT_ACCESS_SECRET", &self.jwt.access_secret),
            ("JWT_REFRESH_SECRET", &self.jwt.refresh_secret),
        ] {
            if secret.len() < 32 {
                anyhow::bail!(
                    "{} must be at least 32 characters (got {})",
                    label,
                    secret.len()
                );
            }
            if PLACEHOLDERS.iter().any(|p| p.eq_ignore_ascii_case(secret)) {
                anyhow::bail!("{} is still set to a placeholder value", label);
            }
        }

        if self.jwt.access_secret == self.jwt.refresh_secret {
            anyhow::bail!("JWT_ACCESS_SECRET and JWT_REFRESH_SECRET must differ");
        }

        self.oauth.validate()?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 3000,
                environment: "development".into(),
                allowed_origins: default_dev_origins(),
            },
            database: DatabaseConfig {
                url: "postgres://localhost/qor_auth".into(),
                max_connections: 10,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".into(),
            },
            jwt: JwtConfig {
                // Deliberately empty: `validate()` rejects these, so a service
                // can never start on default secrets.
                access_secret: String::new(),
                refresh_secret: String::new(),
                access_expiry_secs: 900,
                refresh_expiry_secs: 2592000,
                issuer: "qor-auth".into(),
            },
            security: SecurityConfig {
                max_login_attempts: 5,
                lockout_duration_secs: 900,
                password_min_length: 12,
            },
            chain: ChainConfig { ss58_prefix: 42 },
            oauth: OAuthConfig::default(),
        }
    }
}
