//! Session management service.

use chrono::{Duration, Utc};
use deadpool_redis::Pool as RedisPool;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::config::JwtConfig;
use crate::error::{AppError, AppResult};
use crate::models::{Claims, Scope, Session, TokenPair};

/// What a new session is created for.
pub struct NewSession<'a> {
    pub user_id: Uuid,
    pub qor_id: &'a str,
    /// User role: 'user', 'moderator', 'admin', 'god'
    pub role: Option<&'a str>,
    pub device_id: &'a str,
    pub ip_address: &'a str,
    pub user_agent: Option<&'a str>,
    pub scopes: Vec<Scope>,
}

/// Session management service
pub struct SessionService {
    redis: RedisPool,
    jwt_config: JwtConfig,
}

impl SessionService {
    /// Create new session service
    pub fn new(redis: RedisPool, jwt_config: JwtConfig) -> Self {
        Self { redis, jwt_config }
    }

    /// Create a new session and generate tokens
    pub async fn create_session(&self, new: NewSession<'_>) -> AppResult<(Session, TokenPair)> {
        let NewSession {
            user_id,
            qor_id,
            role,
            device_id,
            ip_address,
            user_agent,
            scopes,
        } = new;
        let session_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::seconds(self.jwt_config.refresh_expiry_secs);

        let session = Session {
            session_id,
            user_id,
            qor_id: qor_id.to_string(),
            device_id: device_id.to_string(),
            ip_address: ip_address.to_string(),
            user_agent: user_agent.map(|s| s.to_string()),
            scopes: scopes.clone(),
            created_at: now,
            last_activity: now,
            expires_at,
        };

        // Store session in Redis
        self.store_session(&session).await?;

        // Generate tokens with role
        let tokens = self.generate_tokens(&session, role)?;

        Ok((session, tokens))
    }

    /// Generate JWT access and refresh tokens
    pub fn generate_tokens(&self, session: &Session, role: Option<&str>) -> AppResult<TokenPair> {
        let now = Utc::now();

        // Access token claims
        let access_claims = Claims {
            sub: session.user_id.to_string(),
            qor_id: session.qor_id.clone(),
            sid: session.session_id.to_string(),
            role: role.map(|r| r.to_string()),
            scopes: session.scopes.clone(),
            iss: self.jwt_config.issuer.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.jwt_config.access_expiry_secs)).timestamp(),
        };

        // Refresh token claims (longer expiry, fewer claims)
        let refresh_claims = Claims {
            sub: session.user_id.to_string(),
            qor_id: session.qor_id.clone(),
            sid: session.session_id.to_string(),
            role: role.map(|r| r.to_string()),
            scopes: vec![], // Refresh tokens don't carry scopes
            iss: self.jwt_config.issuer.clone(),
            iat: now.timestamp(),
            exp: (now + Duration::seconds(self.jwt_config.refresh_expiry_secs)).timestamp(),
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &EncodingKey::from_secret(self.jwt_config.access_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Token generation failed: {}", e)))?;

        let refresh_token = encode(
            &Header::default(),
            &refresh_claims,
            &EncodingKey::from_secret(self.jwt_config.refresh_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Token generation failed: {}", e)))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: self.jwt_config.access_expiry_secs,
            token_type: "Bearer".into(),
        })
    }

    /// Validate an access token
    pub fn validate_access_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.access_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        })?;

        Ok(token_data.claims)
    }

    /// Authenticate an access token: a valid signature and expiry, and a session that still exists.
    ///
    /// The signature alone would keep a token working until it expires. Requiring the session means
    /// logout, or an administrator revoking a user's sessions, ends the token at once.
    pub async fn authenticate_access_token(&self, token: &str) -> AppResult<Claims> {
        let claims = self.validate_access_token(token)?;
        let session_id = Uuid::parse_str(&claims.sid).map_err(|_| AppError::InvalidToken)?;
        let session = self.get_session(session_id).await?;
        check_session(&claims, session.as_ref())?;
        Ok(claims)
    }

    /// Validate a refresh token
    pub fn validate_refresh_token(&self, token: &str) -> AppResult<Claims> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_config.refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        })?;

        Ok(token_data.claims)
    }

    /// Store session in Redis
    async fn store_session(&self, session: &Session) -> AppResult<()> {
        let mut conn = self.redis.get().await?;

        let session_key = format!("session:{}", session.session_id);
        let session_json = serde_json::to_string(session)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Serialization failed: {}", e)))?;

        // Store session with expiry
        let ttl = (session.expires_at - Utc::now()).num_seconds().max(0) as u64;

        deadpool_redis::redis::cmd("SETEX")
            .arg(&session_key)
            .arg(ttl)
            .arg(&session_json)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        // Add to user's session set
        let user_sessions_key = format!("user_sessions:{}", session.user_id);
        deadpool_redis::redis::cmd("SADD")
            .arg(&user_sessions_key)
            .arg(session.session_id.to_string())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: Uuid) -> AppResult<Option<Session>> {
        let mut conn = self.redis.get().await?;

        let session_key = format!("session:{}", session_id);
        let result: Option<String> = deadpool_redis::redis::cmd("GET")
            .arg(&session_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        match result {
            Some(json) => {
                let session: Session = serde_json::from_str(&json).map_err(|e| {
                    AppError::InternalError(anyhow::anyhow!("Deserialization failed: {}", e))
                })?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// Record that a session was used just now, when its refresh token mints new tokens.
    ///
    /// Returns false when the session no longer exists, and then writes nothing: `XX` sets the key
    /// only if it is still there, so a session revoked a moment ago is not brought back. `KEEPTTL`
    /// leaves its expiry as it was, so using a session never lengthens its life. Both need Redis 6
    /// or later.
    ///
    /// This is one write for each refresh, which a client makes about once per access token's
    /// lifetime. It is deliberately not done for every authenticated request.
    pub async fn record_use(&self, session: &Session) -> AppResult<bool> {
        let mut used = session.clone();
        used.last_activity = Utc::now();
        let session_json = serde_json::to_string(&used)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Serialization failed: {}", e)))?;

        let mut conn = self.redis.get().await?;
        let written: Option<String> = deadpool_redis::redis::cmd("SET")
            .arg(format!("session:{}", session.session_id))
            .arg(&session_json)
            .arg("XX")
            .arg("KEEPTTL")
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;
        Ok(written.is_some())
    }

    /// List a user's live sessions, oldest first.
    ///
    /// The user's session set can still name sessions that expired or were deleted. Those ids are
    /// dropped from the set rather than listed.
    pub async fn list_sessions(&self, user_id: Uuid) -> AppResult<Vec<Session>> {
        let user_sessions_key = format!("user_sessions:{}", user_id);
        let ids: Vec<String> = {
            let mut conn = self.redis.get().await?;
            deadpool_redis::redis::cmd("SMEMBERS")
                .arg(&user_sessions_key)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?
        };

        let mut live = Vec::new();
        let mut stale = Vec::new();
        for id in ids {
            let session = match Uuid::parse_str(&id) {
                Ok(session_id) => self.get_session(session_id).await?,
                Err(_) => None,
            };
            match session {
                Some(session) if owned_by(Some(&session), user_id) => live.push(session),
                _ => stale.push(id),
            }
        }

        if !stale.is_empty() {
            let mut conn = self.redis.get().await?;
            deadpool_redis::redis::cmd("SREM")
                .arg(&user_sessions_key)
                .arg(&stale)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;
        }

        live.sort_by_key(|session| session.created_at);
        Ok(live)
    }

    /// Count the sessions held in Redis. Sessions expire on their own, so every one counted is live.
    ///
    /// Uses SCAN, which can return a key more than once while Redis rehashes; keys are counted once.
    pub async fn count_sessions(&self) -> AppResult<u64> {
        let mut conn = self.redis.get().await?;
        let mut seen = std::collections::HashSet::new();
        let mut cursor: u64 = 0;
        loop {
            let (next, keys): (u64, Vec<String>) = deadpool_redis::redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg("session:*")
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut conn)
                .await
                .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;
            seen.extend(keys);
            cursor = next;
            if cursor == 0 {
                break;
            }
        }
        Ok(seen.len() as u64)
    }

    /// Revoke one of a user's sessions. Its access and refresh tokens stop working at once.
    ///
    /// Returns false when the user holds no such live session, so another user's session and one
    /// that never existed cannot be told apart.
    pub async fn revoke_session(&self, user_id: Uuid, session_id: Uuid) -> AppResult<bool> {
        let session = self.get_session(session_id).await?;
        if !owned_by(session.as_ref(), user_id) {
            return Ok(false);
        }
        self.delete_session(session_id, user_id).await?;
        Ok(true)
    }

    /// Delete session
    pub async fn delete_session(&self, session_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let mut conn = self.redis.get().await?;

        let session_key = format!("session:{}", session_id);
        let user_sessions_key = format!("user_sessions:{}", user_id);

        // Delete session
        deadpool_redis::redis::cmd("DEL")
            .arg(&session_key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        // Remove from user's session set
        deadpool_redis::redis::cmd("SREM")
            .arg(&user_sessions_key)
            .arg(session_id.to_string())
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        Ok(())
    }

    /// Delete all sessions for a user
    pub async fn delete_all_sessions(&self, user_id: Uuid) -> AppResult<()> {
        let mut conn = self.redis.get().await?;

        let user_sessions_key = format!("user_sessions:{}", user_id);

        // Get all session IDs
        let session_ids: Vec<String> = deadpool_redis::redis::cmd("SMEMBERS")
            .arg(&user_sessions_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        // Delete each session
        for sid in session_ids {
            let session_key = format!("session:{}", sid);
            deadpool_redis::redis::cmd("DEL")
                .arg(&session_key)
                .query_async::<()>(&mut conn)
                .await
                .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;
        }

        // Delete the set
        deadpool_redis::redis::cmd("DEL")
            .arg(&user_sessions_key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))?;

        Ok(())
    }
}

/// A session counts for a user only while it exists, belongs to that user and has not expired.
fn owned_by(session: Option<&Session>, user_id: Uuid) -> bool {
    matches!(session, Some(session) if session.user_id == user_id && !session.is_expired())
}

/// A token's claims are honoured only by a live session that belongs to the same user.
fn check_session(claims: &Claims, session: Option<&Session>) -> AppResult<()> {
    match Uuid::parse_str(&claims.sub) {
        Ok(user_id) if owned_by(session, user_id) => Ok(()),
        _ => Err(AppError::InvalidToken),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session_for(user_id: Uuid, expires_in: Duration) -> Session {
        let now = Utc::now();
        Session {
            session_id: Uuid::new_v4(),
            user_id,
            qor_id: "tester".into(),
            device_id: "device".into(),
            ip_address: "127.0.0.1".into(),
            user_agent: None,
            scopes: vec![],
            created_at: now,
            last_activity: now,
            expires_at: now + expires_in,
        }
    }

    fn claims_for(user_id: Uuid, session: &Session) -> Claims {
        Claims {
            sub: user_id.to_string(),
            qor_id: "tester".into(),
            sid: session.session_id.to_string(),
            role: None,
            scopes: vec![],
            iss: "test".into(),
            iat: 0,
            exp: 0,
        }
    }

    #[test]
    fn a_live_session_of_the_same_user_is_honoured() {
        let user = Uuid::new_v4();
        let session = session_for(user, Duration::hours(1));
        assert!(check_session(&claims_for(user, &session), Some(&session)).is_ok());
    }

    #[test]
    fn a_deleted_session_is_refused() {
        let user = Uuid::new_v4();
        let session = session_for(user, Duration::hours(1));
        assert!(matches!(
            check_session(&claims_for(user, &session), None),
            Err(AppError::InvalidToken)
        ));
    }

    #[test]
    fn another_users_session_is_refused() {
        let session = session_for(Uuid::new_v4(), Duration::hours(1));
        let claims = claims_for(Uuid::new_v4(), &session);
        assert!(matches!(
            check_session(&claims, Some(&session)),
            Err(AppError::InvalidToken)
        ));
    }

    #[test]
    fn an_expired_session_is_refused() {
        let user = Uuid::new_v4();
        let session = session_for(user, Duration::seconds(-1));
        assert!(matches!(
            check_session(&claims_for(user, &session), Some(&session)),
            Err(AppError::InvalidToken)
        ));
    }

    #[test]
    fn a_session_is_owned_only_by_its_user_while_live() {
        let user = Uuid::new_v4();
        let live = session_for(user, Duration::hours(1));
        let expired = session_for(user, Duration::seconds(-1));
        assert!(owned_by(Some(&live), user));
        assert!(!owned_by(Some(&live), Uuid::new_v4()));
        assert!(!owned_by(Some(&expired), user));
        assert!(!owned_by(None, user));
    }

    fn test_jwt() -> JwtConfig {
        JwtConfig {
            access_secret: "test-access-secret-at-least-32-characters".into(),
            refresh_secret: "test-refresh-secret-at-least-32-characters".into(),
            access_expiry_secs: 900,
            refresh_expiry_secs: 3600,
            issuer: "test".into(),
        }
    }

    fn new_session(user_id: Uuid) -> NewSession<'static> {
        NewSession {
            user_id,
            qor_id: "tester#0001",
            role: None,
            device_id: "test-device",
            ip_address: "127.0.0.1",
            user_agent: None,
            scopes: vec![],
        }
    }

    /// Runs against a real Redis:
    /// `QOR_AUTH_TEST_REDIS_URL=redis://127.0.0.1:6379 cargo test -- --ignored`
    #[tokio::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn sessions_list_and_revoke_against_redis() {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL").expect("QOR_AUTH_TEST_REDIS_URL");
        let pool = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        let service = SessionService::new(pool.clone(), test_jwt());
        let user = Uuid::new_v4();
        let other = Uuid::new_v4();

        let (a, _) = service.create_session(new_session(user)).await.unwrap();
        let (b, tokens_b) = service.create_session(new_session(user)).await.unwrap();
        let (c, _) = service.create_session(new_session(other)).await.unwrap();

        let listed: Vec<Uuid> = service
            .list_sessions(user)
            .await
            .unwrap()
            .iter()
            .map(|s| s.session_id)
            .collect();
        assert_eq!(listed.len(), 2);
        assert!(listed.contains(&a.session_id) && listed.contains(&b.session_id));
        assert!(!listed.contains(&c.session_id));

        // Another user's session cannot be revoked, and is left untouched.
        assert!(!service.revoke_session(other, a.session_id).await.unwrap());
        assert!(service.get_session(a.session_id).await.unwrap().is_some());

        // Revoking takes effect at once, and only once.
        assert!(service.revoke_session(user, b.session_id).await.unwrap());
        assert!(matches!(
            service
                .authenticate_access_token(&tokens_b.access_token)
                .await,
            Err(AppError::InvalidToken)
        ));
        assert!(!service.revoke_session(user, b.session_id).await.unwrap());
        assert!(!service.revoke_session(user, Uuid::new_v4()).await.unwrap());

        // A session that disappears without going through revoke is dropped from the list and set.
        let mut conn = pool.get().await.unwrap();
        deadpool_redis::redis::cmd("DEL")
            .arg(format!("session:{}", a.session_id))
            .query_async::<()>(&mut conn)
            .await
            .unwrap();
        assert!(service.list_sessions(user).await.unwrap().is_empty());
        let members: Vec<String> = deadpool_redis::redis::cmd("SMEMBERS")
            .arg(format!("user_sessions:{}", user))
            .query_async(&mut conn)
            .await
            .unwrap();
        assert!(members.is_empty());

        service.delete_all_sessions(user).await.unwrap();
        service.delete_all_sessions(other).await.unwrap();
    }

    async fn seconds_left(pool: &RedisPool, session_id: Uuid) -> i64 {
        let mut conn = pool.get().await.unwrap();
        deadpool_redis::redis::cmd("TTL")
            .arg(format!("session:{session_id}"))
            .query_async(&mut conn)
            .await
            .unwrap()
    }

    fn redis_for_tests() -> RedisPool {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL").expect("QOR_AUTH_TEST_REDIS_URL");
        deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool")
    }

    /// A session as it would be an hour after sign-in: created and last used then, with eleven
    /// minutes left to live.
    async fn an_hour_old_session(service: &SessionService, user: Uuid) -> Session {
        let then = Utc::now() - Duration::hours(1);
        let session = Session {
            created_at: then,
            last_activity: then,
            expires_at: Utc::now() + Duration::seconds(660),
            ..session_for(user, Duration::zero())
        };
        service.store_session(&session).await.unwrap();
        session
    }

    #[tokio::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn using_a_session_records_when_and_changes_nothing_else() {
        let pool = redis_for_tests();
        let service = SessionService::new(pool.clone(), test_jwt());
        let user = Uuid::new_v4();
        let session = an_hour_old_session(&service, user).await;

        let before = Utc::now();
        assert!(service.record_use(&session).await.unwrap());
        let after = Utc::now();

        let stored = service
            .get_session(session.session_id)
            .await
            .unwrap()
            .expect("still there");
        assert!(
            stored.last_activity >= before && stored.last_activity <= after,
            "last used is now, not {}",
            stored.last_activity
        );
        // Everything else is as it was. Redis keeps the times as text, so compare what it holds.
        let expected = Session {
            last_activity: stored.last_activity,
            ..session.clone()
        };
        assert_eq!(
            serde_json::to_value(&stored).unwrap(),
            serde_json::to_value(&expected).unwrap()
        );
        // It is still listed, once.
        let listed = service.list_sessions(user).await.unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].last_activity, stored.last_activity);

        service.delete_all_sessions(user).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn using_a_session_never_lengthens_its_life() {
        let pool = redis_for_tests();
        let service = SessionService::new(pool.clone(), test_jwt());
        let user = Uuid::new_v4();
        let session = an_hour_old_session(&service, user).await;

        let left = seconds_left(&pool, session.session_id).await;
        assert!((1..=660).contains(&left), "{left} seconds left before");
        assert!(service.record_use(&session).await.unwrap());
        let left_after = seconds_left(&pool, session.session_id).await;
        // -1 is Redis for "never expires", which is what a plain SET would leave.
        assert!(
            (1..=left).contains(&left_after),
            "{left_after} seconds left after use, {left} before"
        );

        service.delete_all_sessions(user).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn using_a_revoked_session_does_not_bring_it_back() {
        let pool = redis_for_tests();
        let service = SessionService::new(pool.clone(), test_jwt());
        let user = Uuid::new_v4();
        // Read by a refresh, then revoked before the refresh records its use.
        let session = an_hour_old_session(&service, user).await;
        assert!(
            service
                .revoke_session(user, session.session_id)
                .await
                .unwrap()
        );

        assert!(!service.record_use(&session).await.unwrap());
        assert!(
            service
                .get_session(session.session_id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(service.list_sessions(user).await.unwrap().is_empty());
    }
}
