//! Session model for token management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Session stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub qor_id: String,
    pub device_id: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub scopes: Vec<Scope>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    /// The registered app that created the session through `/oauth/authorize` (ADR-043 decision 3),
    /// or `None` for the launcher and the JSON API. Absent from sessions stored before it existed.
    #[serde(default)]
    pub client_id: Option<String>,
    /// For an app's session, the id of the one refresh token that may be used next. Each refresh
    /// replaces it; presenting an older one ends the session (ADR-043 decision 2).
    #[serde(default)]
    pub refresh_jti: Option<String>,
}

/// Permission scopes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    ProfileRead,
    ProfileWrite,
    WalletRead,
    WalletTransact,
    GameAccess,
    GovernanceVote,
    AdminRead,
    AdminWrite,
}

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Qor ID
    pub qor_id: String,
    /// Session ID
    pub sid: String,
    /// User role
    pub role: Option<String>, // 'user', 'moderator', 'admin', 'god'
    /// Scopes
    pub scopes: Vec<Scope>,
    /// Issuer
    pub iss: String,
    /// Issued at
    pub iat: i64,
    /// Expiration
    pub exp: i64,
    /// A refresh token's id, for an app's session. Absent from other tokens, and from tokens
    /// issued before it existed, which decode as before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    /// The registered app the session belongs to, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
}

/// Token pair response
#[derive(Debug, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// Refresh token request
#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

impl Session {
    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get default scopes for a new session
    pub fn default_scopes() -> Vec<Scope> {
        vec![Scope::ProfileRead, Scope::ProfileWrite, Scope::GameAccess]
    }
}
