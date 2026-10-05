//! Authentication service.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

use crate::error::{AppError, AppResult};
use crate::models::User;

/// Authentication service
pub struct AuthService {
    db: PgPool,
}

impl AuthService {
    /// Create new auth service
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| {
                AppError::InternalError(anyhow::anyhow!("Password hashing failed: {}", e))
            })?;

        Ok(hash.to_string())
    }

    /// An Argon2 hash of a random password, made once per process. Sign-in verifies
    /// against it when no account matches, so an unknown name costs the same work as
    /// a wrong password on a real account.
    pub fn stand_in_password_hash() -> &'static str {
        static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        HASH.get_or_init(|| {
            let random = hex::encode(rand::random::<[u8; 32]>());
            Self::hash_password(&random).expect("hashing a random password")
        })
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Invalid hash format: {}", e)))?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE LOWER(email) = LOWER($1)")
            .bind(email)
            .fetch_optional(&self.db)
            .await?;

        Ok(user)
    }

    /// Increment login attempts and lock if needed
    pub async fn increment_login_attempts(
        &self,
        user_id: uuid::Uuid,
        max_attempts: u32,
        lockout_secs: i64,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET 
                login_attempts = login_attempts + 1,
                locked_until = CASE 
                    WHEN login_attempts + 1 >= $2 
                    THEN NOW() + INTERVAL '1 second' * $3
                    ELSE locked_until
                END
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(max_attempts as i32)
        .bind(lockout_secs)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Reset login attempts after successful login
    pub async fn reset_login_attempts(&self, user_id: uuid::Uuid) -> AppResult<()> {
        sqlx::query("UPDATE users SET login_attempts = 0, locked_until = NULL WHERE id = $1")
            .bind(user_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// The one account with this username, whatever its letter case (ADR-075: names are unique).
    pub async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE LOWER(username) = LOWER($1)")
                .bind(username)
                .fetch_optional(&self.db)
                .await?;

        Ok(user)
    }

    /// Check if identifier is an email
    pub fn is_email(identifier: &str) -> bool {
        identifier.contains('@') && identifier.contains('.')
    }

    /// Generate a secure backup code (32 characters)
    pub fn generate_backup_code() -> String {
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Exclude confusing chars
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }

    /// How many backup codes an account without an email address receives.
    pub const BACKUP_CODE_COUNT: usize = 10;

    /// Generate the backup codes for a new account. Each resets the password once.
    pub fn generate_backup_codes() -> Vec<String> {
        (0..Self::BACKUP_CODE_COUNT)
            .map(|_| Self::generate_backup_code())
            .collect()
    }

    /// The stored form of a backup code: SHA-256, as hex, of the code in upper case
    /// with spaces and hyphens removed, so a code typed in groups or lower case still
    /// matches. A code carries 160 bits of randomness, so a fast hash is enough; the
    /// code itself is never stored. Migration 014 hashes existing codes the same way.
    pub fn hash_backup_code(code: &str) -> String {
        use sha2::{Digest, Sha256};
        let normalised: String = code
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '-')
            .collect::<String>()
            .to_uppercase();
        hex::encode(Sha256::digest(normalised.as_bytes()))
    }

    /// Generate email verification token
    pub fn generate_verification_token() -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(uuid::Uuid::new_v4().as_bytes());
        hasher.update(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_be_bytes(),
        );
        hex::encode(hasher.finalize())[..32].to_string()
    }

    // `hash_to_address` was removed on 19 September 2026 (ADR-017). It derived a
    // chain address by hashing a name and a timestamp, so no key corresponded to
    // it: anything sent there could never be moved, while every interface showed
    // it as a real account. A chain account now arrives only by proving a key,
    // and nothing can derive one again.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "SecureP@ssw0rd!123";
        let hash = AuthService::hash_password(password).unwrap();

        assert!(AuthService::verify_password(password, &hash).unwrap());
        assert!(!AuthService::verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn a_backup_code_matches_however_it_is_typed_and_is_never_stored_as_itself() {
        let code = AuthService::generate_backup_code();
        let stored = AuthService::hash_backup_code(&code);
        let grouped = code
            .as_bytes()
            .chunks(4)
            .map(|chunk| std::str::from_utf8(chunk).unwrap().to_lowercase())
            .collect::<Vec<_>>()
            .join("- ");
        assert_eq!(AuthService::hash_backup_code(&grouped), stored);
        assert_ne!(stored, code);
        assert_eq!(stored.len(), 64);
        assert_ne!(
            AuthService::hash_backup_code(&AuthService::generate_backup_code()),
            stored
        );
    }

    #[test]
    fn an_account_receives_distinct_backup_codes() {
        let codes = AuthService::generate_backup_codes();
        assert_eq!(codes.len(), AuthService::BACKUP_CODE_COUNT);
        let distinct: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(distinct.len(), codes.len());
    }
}
