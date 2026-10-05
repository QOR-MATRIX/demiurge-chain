//! User model for Qor ID system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User entity stored in PostgreSQL
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>, // Optional for username-only accounts
    pub username: String,
    pub discriminator: i16,
    pub password_hash: String,
    pub email_verified: bool,
    pub avatar_url: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    /// The 32 bytes of the account's chain identity, once it has proven a key
    /// (ADR-017, ADR-023). Stored as bytes rather than as a string, because an
    /// SS58 address depends on a prefix that is not decided for mainnet and
    /// Postgres can neither produce nor check one (ADR-024).
    pub chain_account_id: Option<Vec<u8>>,
    pub email_verification_token: Option<String>,
    pub email_verification_expires_at: Option<DateTime<Utc>>,
    pub login_attempts: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub auth_method: Option<String>, // password, keypair, or both
    // Agent-specific fields
    pub account_type: Option<String>, // human or agent
    pub controller_id: Option<Uuid>,  // Human owner of this agent
    pub agent_did: Option<String>,    // did:demiurge:agent:...
    pub agent_capabilities: Option<serde_json::Value>, // JSON array of capabilities
    pub agent_autonomy: Option<String>, // supervised, bounded, autonomous, sovereign
    pub agent_spending_limit: Option<i64>, // CGT spending limit
    pub agent_model: Option<String>,  // AI model identifier
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User role for RBAC
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Moderator,
    Admin,
    System,
    God, // God-level access - full system control
}

impl UserRole {
    /// The role as access tokens carry it, and as the admin routes check it: the same lowercase
    /// form as the database enum and JSON.
    ///
    /// Sign-in used to write `format!("{:?}", role)`, which gives `God`, while `require_god`
    /// compared with `god`, so no caller could reach an admin route.
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
            UserRole::System => "system",
            UserRole::God => "god",
        }
    }
}

#[cfg(test)]
mod role_tests {
    use super::UserRole;

    #[test]
    fn the_token_form_of_every_role_matches_its_serialised_form() {
        for role in [
            UserRole::User,
            UserRole::Moderator,
            UserRole::Admin,
            UserRole::System,
            UserRole::God,
        ] {
            assert_eq!(
                serde_json::to_value(role).expect("serialise"),
                serde_json::json!(role.as_str())
            );
        }
    }
}

/// User account status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Banned,
}

/// Registration request DTO
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: Option<String>, // Optional - if provided, will send confirmation email
    pub password: String,
    pub username: String,
}

/// Login request DTO - accepts email OR username
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    #[serde(alias = "email", alias = "username")]
    pub identifier: String, // Can be email or username
    pub password: String,
    pub device_id: Option<String>,
}

/// Password reset request DTO
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub identifier: String, // Username or email
}

/// Request a new email verification message
#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    /// Username or email address
    pub identifier: String,
}

/// Issue a new set of backup codes, confirmed with the account's password
#[derive(Debug, Deserialize)]
pub struct RegenerateBackupCodesRequest {
    pub password: String,
}

/// Add an email address, or change the current one, confirmed with the account's password
#[derive(Debug, Deserialize)]
pub struct ChangeEmailRequest {
    pub email: String,
    pub password: String,
}

/// Password reset with backup code DTO
#[derive(Debug, Deserialize)]
pub struct ResetPasswordWithBackupRequest {
    pub username: String,
    pub backup_code: String,
    pub new_password: String,
}

/// Password reset with token DTO (for email-based reset)
#[derive(Debug, Deserialize)]
pub struct ResetPasswordWithTokenRequest {
    pub token: String,
    pub new_password: String,
}

/// Request for a signature challenge (keypair auth step 1)
#[derive(Debug, Deserialize)]
pub struct ChallengeRequest {
    /// The account as a person sees it: SS58 at the chain's prefix (ADR-024).
    #[serde(default)]
    pub address: Option<String>,
    /// The same account as raw hex. The advanced form, and named separately so
    /// that a field meant for an address cannot quietly take one.
    #[serde(default)]
    pub account_id: Option<String>,
}

/// Response containing the challenge to sign
#[derive(Debug, Serialize)]
pub struct ChallengeResponse {
    pub challenge: String,
    pub expires_at: DateTime<Utc>,
}

/// Login with keypair signature (keypair auth step 2)
#[derive(Debug, Deserialize)]
pub struct KeypairLoginRequest {
    /// The account, as SS58 (ADR-024).
    #[serde(default)]
    pub address: Option<String>,
    /// The account as raw hex, for an advanced caller.
    #[serde(default)]
    pub account_id: Option<String>,
    pub challenge: String,
    /// The account key's Sr25519 signature over the domain-tagged challenge,
    /// as hex (ADR-023).
    pub signature: String,
    pub device_id: Option<String>,
}

/// Register with keypair (creates an account for a proven key)
#[derive(Debug, Deserialize)]
pub struct KeypairRegisterRequest {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    pub username: Option<String>, // Optional username, will be auto-generated if not provided
    pub challenge: String,
    pub signature: String,
    pub device_id: Option<String>,
}

/// Link keypair to existing account
#[derive(Debug, Deserialize)]
pub struct LinkKeypairRequest {
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    pub challenge: String,
    pub signature: String,
}

// =============================================================================
// Agent Types
// =============================================================================

/// Register a new AI agent
#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub capabilities: Vec<String>,
    pub autonomy: String, // supervised, bounded, autonomous, sovereign
    pub spending_limit: Option<i64>,
    pub model: Option<String>,
    /// The agent's own account, as SS58. The agent generates the keypair;
    /// QOR ID never sees the private key (ADR-014).
    #[serde(default)]
    pub address: Option<String>,
    /// The same account as raw hex, for an advanced caller.
    #[serde(default)]
    pub account_id: Option<String>,
    /// A challenge issued for that account by `GET /api/v1/auth/challenge`.
    pub challenge: String,
    /// The agent key's Sr25519 signature over `challenge`, as hex.
    pub signature: String,
}

/// Agent registration response
#[derive(Debug, Serialize)]
pub struct AgentRegistrationResponse {
    pub agent_id: Uuid,
    pub qor_id: String,
    pub did: String,
    /// The agent's account, SS58 for people and hex for advanced views
    /// (ADR-024).
    pub address: String,
    pub account_id: String,
    pub capabilities: Vec<String>,
    pub autonomy: String,
}

/// Update agent capabilities
#[derive(Debug, Deserialize)]
pub struct UpdateAgentCapabilitiesRequest {
    pub capabilities: Vec<String>,
}

/// Agent info response
#[derive(Debug, Serialize)]
pub struct AgentInfo {
    pub id: Uuid,
    pub qor_id: String,
    pub did: String,
    pub address: Option<String>,
    pub account_id: Option<String>,
    pub capabilities: Vec<String>,
    pub autonomy: String,
    pub spending_limit: Option<i64>,
    pub model: Option<String>,
    pub status: String,
    pub controller_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl User {
    /// The QOR ID: the username alone, unique on its own (ADR-075). There is no `#0001`.
    pub fn qor_id(&self) -> String {
        self.username.to_lowercase()
    }

    /// Check if account is locked
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            Utc::now() < locked_until
        } else {
            false
        }
    }
}
