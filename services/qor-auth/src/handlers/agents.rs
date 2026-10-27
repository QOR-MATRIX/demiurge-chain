//! Agent management handlers.
//!
//! An agent is an account a human controls. The agent generates its own
//! Sr25519 keypair; QOR ID authorises the public key and never generates,
//! stores or returns a private key (ADR-014). Every endpoint acts only on
//! agents the authenticated user controls, and authorising or widening an
//! agent also needs the controller's own key to sign for it (ADR-112): an
//! access token alone is not enough.

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use rand::Rng;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::handlers::auth::{
    ChainAccount, consume_challenge_signed_over, consume_signed_challenge,
};
use crate::models::{
    AgentInfo, AgentRegistrationResponse, RegisterAgentRequest, UpdateAgentCapabilitiesRequest,
};
use crate::services::auth_service::AuthService;
use crate::state::AppState;

/// Valid agent capabilities
const VALID_CAPABILITIES: &[&str] = &[
    "read",     // Read blockchain data
    "analyze",  // Analyze patterns
    "trade",    // Execute trades
    "transfer", // Transfer CGT
    "bounty",   // Submit bounty bids
    "nft",      // Interact with NFTs
    "vote",     // Participate in governance
    "stake",    // Staking operations
];

/// Validate agent capabilities
fn validate_capabilities(capabilities: &[String]) -> Result<(), AppError> {
    for cap in capabilities {
        if !VALID_CAPABILITIES.contains(&cap.as_str()) {
            return Err(AppError::ValidationError(format!(
                "Invalid capability '{}'. Valid options: {:?}",
                cap, VALID_CAPABILITIES
            )));
        }
    }
    Ok(())
}

/// Validate autonomy level
fn validate_autonomy(autonomy: &str) -> Result<(), AppError> {
    match autonomy {
        "supervised" | "bounded" | "autonomous" | "sovereign" => Ok(()),
        _ => Err(AppError::ValidationError(
            "Autonomy must be: supervised, bounded, autonomous, or sovereign".into(),
        )),
    }
}

/// Validate a spending limit: a whole number of CGT, never negative, and required of a bounded agent.
///
/// It is recorded, not enforced: nothing in QOR ID or on chain holds an agent to it yet (Q-9). Enforcement belongs to
/// `pallet-agent-caps` (M5.2), which will count in embers on chain. Until then a negative limit is refused rather than
/// stored, so no record suggests a cap that could never mean anything.
fn validate_spending_limit(autonomy: &str, limit: Option<i64>) -> Result<(), AppError> {
    match limit {
        None if autonomy == "bounded" => Err(AppError::ValidationError(
            "Bounded agents require a spending_limit".into(),
        )),
        Some(limit) if limit < 0 => Err(AppError::ValidationError(
            "spending_limit is a whole number of CGT and cannot be negative".into(),
        )),
        _ => Ok(()),
    }
}

/// The first line of what a controller's key signs to authorise an agent (ADR-112). Not the challenge's domain
/// (`demiurge:qor-id:challenge:v1:`), so neither signature can be offered as the other, nor as a transaction.
const AUTHORISATION_DOMAIN: &str = "demiurge:qor-id:agent-authorisation:v1";

/// The first line of what a controller's key signs to change an agent's capabilities (ADR-112).
const CHANGE_DOMAIN: &str = "demiurge:qor-id:agent-change:v1";

/// The QOR ID an agent called `name` is given, `agent_<name>`, or a refusal: it must be a QOR ID like any other
/// (`QorId::is_valid_username`: 3 to 20 letters, digits and underscores). Until 22 October 2026 a name of up to 32
/// characters of any kind was taken, so a longer one failed in the database and one with other characters was stored
/// as a QOR ID no person could have.
fn agent_qor_id(name: &str) -> Result<String, AppError> {
    let qor_id = format!("agent_{}", name.to_lowercase().replace(' ', "_"));
    if name.len() < 3 || !crate::models::QorId::is_valid_username(&qor_id) {
        return Err(AppError::ValidationError(
            "An agent's name is 3 to 14 letters, digits, spaces or underscores: its QOR ID is agent_ and the name".into(),
        ));
    }
    Ok(qor_id)
}

/// A field of a statement is one line: a line break or another control character in it could pose as a further field.
fn one_line(what: &str, value: &str) -> Result<(), AppError> {
    if value.chars().any(char::is_control) {
        return Err(AppError::ValidationError(format!(
            "{what} cannot hold a line break or a control character"
        )));
    }
    Ok(())
}

/// Capabilities as a statement names them: each one QOR ID knows, none twice, in order, comma-separated.
fn capabilities_line(capabilities: &[String]) -> Result<String, AppError> {
    validate_capabilities(capabilities)?;
    let mut sorted: Vec<&str> = capabilities.iter().map(String::as_str).collect();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(AppError::ValidationError(
            "A capability is named twice".into(),
        ));
    }
    Ok(sorted.join(","))
}

/// An agent's model as it is recorded and signed for: absent, or one line with something in it.
fn model_of(req: &RegisterAgentRequest) -> Result<Option<&str>, AppError> {
    let model = req.model.as_deref().filter(|m| !m.is_empty());
    if let Some(model) = model {
        one_line("The model", model)?;
    }
    Ok(model)
}

/// What the controller's key signs to authorise `agent` as `req` describes it (ADR-112): every field QOR ID records,
/// and the agent's single-use challenge, so the signature authorises this registration once and nothing else.
pub(crate) fn authorisation_statement(
    controller: &ChainAccount,
    agent: &ChainAccount,
    req: &RegisterAgentRequest,
) -> Result<String, AppError> {
    one_line("The challenge", &req.challenge)?;
    one_line("The name", &req.name)?;
    let capabilities = capabilities_line(&req.capabilities)?;
    validate_autonomy(&req.autonomy)?;
    let spending_limit = req
        .spending_limit
        .map(|limit| limit.to_string())
        .unwrap_or_default();
    Ok(format!(
        "{AUTHORISATION_DOMAIN}\ncontroller:{}\nagent:{}\nchallenge:{}\nname:{}\ncapabilities:{capabilities}\n\
         autonomy:{}\nspending_limit:{spending_limit}\nmodel:{}",
        hex::encode(controller.as_bytes()),
        hex::encode(agent.as_bytes()),
        req.challenge,
        req.name,
        req.autonomy,
        model_of(req)?.unwrap_or_default(),
    ))
}

/// What the controller's key signs to give the agent `did` these capabilities (ADR-112), around a challenge issued for
/// the controller's own account.
pub(crate) fn change_statement(
    controller: &ChainAccount,
    did: &str,
    challenge: &str,
    capabilities: &[String],
) -> Result<String, AppError> {
    one_line("The agent", did)?;
    one_line("The challenge", challenge)?;
    Ok(format!(
        "{CHANGE_DOMAIN}\ncontroller:{}\nagent:{did}\nchallenge:{challenge}\ncapabilities:{}",
        hex::encode(controller.as_bytes()),
        capabilities_line(capabilities)?
    ))
}

/// The key the controller's QOR ID has proved, for a controller allowed to have agents: an active account, not itself
/// an agent, which would let delegated authority delegate itself again.
async fn controller_key(db: &sqlx::PgPool, controller_id: Uuid) -> AppResult<ChainAccount> {
    let controller: Option<(Option<String>, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT account_type, chain_account_id FROM users WHERE id = $1 AND status = 'active'",
    )
    .bind(controller_id)
    .fetch_optional(db)
    .await?;

    let (kind, key) = controller.ok_or(AppError::InsufficientPermissions)?;
    if kind.is_some_and(|kind| kind != "human") {
        return Err(AppError::InsufficientPermissions);
    }
    let key = key.ok_or_else(|| {
        AppError::ValidationError(
            "An agent is authorised by your own key, and this QOR ID has proved none: sign in with your key, or link \
             one, first"
                .into(),
        )
    })?;
    ChainAccount::from_stored(&key)
}

/// The controller's signature, which ADR-112 requires, or a refusal that says so.
fn controller_signature(signature: Option<&str>) -> Result<&str, AppError> {
    signature.ok_or_else(|| {
        AppError::ValidationError(
            "An agent is authorised by the controller's own key: send controller_signature (ADR-112)".into(),
        )
    })
}

/// Generate agent DID
fn generate_agent_did() -> String {
    let random_bytes: [u8; 16] = rand::thread_rng().r#gen();
    format!("did:demiurge:agent:mainnet:{}", hex::encode(random_bytes))
}

fn to_agent_info(agent: crate::models::User, ss58_prefix: u16) -> AgentInfo {
    let capabilities: Vec<String> = agent
        .agent_capabilities
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let account = agent
        .chain_account_id
        .as_deref()
        .and_then(|bytes| ChainAccount::from_stored(bytes).ok());

    AgentInfo {
        id: agent.id,
        qor_id: agent.qor_id(),
        did: agent.agent_did.unwrap_or_default(),
        address: account.map(|a| a.address(ss58_prefix)),
        account_id: account.map(|a| a.account_id_hex()),
        capabilities,
        autonomy: agent
            .agent_autonomy
            .unwrap_or_else(|| "supervised".to_string()),
        spending_limit: agent.agent_spending_limit,
        model: agent.agent_model,
        status: format!("{:?}", agent.status).to_lowercase(),
        controller_id: agent.controller_id.unwrap_or(Uuid::nil()),
        created_at: agent.created_at,
    }
}

/// Register a new AI agent
/// POST /api/v1/agents/register
///
/// The authenticated user becomes the agent's controller. The request carries
/// the agent's public key and the agent key's signature over a challenge issued
/// for it, proving the agent holds the private key; and the controller's own
/// key's signature over the whole authorisation (ADR-112), so an access token
/// alone cannot make an agent.
pub async fn register_agent(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Json(req): Json<RegisterAgentRequest>,
) -> AppResult<(StatusCode, Json<AgentRegistrationResponse>)> {
    // Validate input
    let agent_name = agent_qor_id(&req.name)?;
    validate_capabilities(&req.capabilities)?;
    validate_autonomy(&req.autonomy)?;

    validate_spending_limit(&req.autonomy, req.spending_limit)?;
    let model = model_of(&req)?;
    let signed_by_controller = controller_signature(req.controller_signature.as_deref())?;

    // Only an active human account can control agents, and only with a key it has proved.
    let controller = controller_key(&state.db, controller_id).await?;

    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

    // The controller's own key signed for exactly this agent, as asked (ADR-112).
    let statement = authorisation_statement(&controller, &account, &req)?;
    if !controller.verify(statement.as_bytes(), signed_by_controller) {
        return Err(AppError::ValidationError(
            "The controller's signature does not cover this authorisation".into(),
        ));
    }

    // Proof of possession: the agent's own key signed a challenge issued for it. Spending it spends the controller's
    // signature too, which names it.
    consume_signed_challenge(&state.db, account, &req.challenge, &req.signature).await?;

    let already_registered: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE chain_account_id = $1 LIMIT 1")
            .bind(account.as_bytes())
            .fetch_optional(&state.db)
            .await?;

    if already_registered.is_some() {
        return Err(AppError::ValidationError(
            "This account is already registered".into(),
        ));
    }

    let auth_service = AuthService::new(state.db.clone());

    // One name per account (ADR-075): an agent's name is refused if any account holds it.
    if auth_service.find_by_username(&agent_name).await?.is_some() {
        return Err(AppError::ValidationError(
            "An account with this name already exists".into(),
        ));
    }
    let discriminator: i16 = 1;
    let agent_did = generate_agent_did();

    // Agents sign in with their key. The password hash covers a random value
    // that is discarded at once, so no password can ever match it.
    let unused: [u8; 32] = rand::thread_rng().r#gen();
    let password_hash = AuthService::hash_password(&hex::encode(unused))?;

    let capabilities_json = serde_json::to_value(&req.capabilities)
        .map_err(|_| AppError::ValidationError("Invalid capabilities".into()))?;

    let agent_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (
            username, discriminator, password_hash,
            email_verified, role, status,
            auth_method, chain_account_id,
            account_type, controller_id, agent_did,
            agent_capabilities, agent_autonomy, agent_spending_limit, agent_model
        )
        VALUES ($1, $2, $3, TRUE, 'user', 'active', 'keypair', $4,
                'agent', $5, $6, $7, $8, $9, $10)
        RETURNING id
        "#,
    )
    .bind(&agent_name)
    .bind(discriminator)
    .bind(&password_hash)
    .bind(account.as_bytes())
    .bind(controller_id)
    .bind(&agent_did)
    .bind(&capabilities_json)
    .bind(&req.autonomy)
    .bind(req.spending_limit)
    .bind(model)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if crate::handlers::auth::name_taken(&e) {
            AppError::ValidationError("An account with this name already exists".into())
        } else {
            AppError::DatabaseError(e)
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(AgentRegistrationResponse {
            agent_id,
            qor_id: agent_name.clone(),
            did: agent_did,
            address: account.address(state.config.chain.ss58_prefix),
            account_id: account.account_id_hex(),
            capabilities: req.capabilities,
            autonomy: req.autonomy,
        }),
    ))
}

/// Get one of the caller's agents by DID
/// GET /api/v1/agents/:did
///
/// Another user's agent is reported as not found, so its existence is not
/// disclosed.
pub async fn get_agent(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Path(did): Path<String>,
) -> AppResult<Json<AgentInfo>> {
    let agent: Option<crate::models::User> = sqlx::query_as(
        "SELECT * FROM users WHERE agent_did = $1 AND account_type = 'agent' AND controller_id = $2",
    )
    .bind(&did)
    .bind(controller_id)
    .fetch_optional(&state.db)
    .await?;

    let agent = agent.ok_or(AppError::NotFound("Agent not found".into()))?;

    Ok(Json(to_agent_info(agent, state.config.chain.ss58_prefix)))
}

/// List the caller's agents
/// GET /api/v1/agents
pub async fn list_agents(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
) -> AppResult<Json<Vec<AgentInfo>>> {
    let agents: Vec<crate::models::User> = sqlx::query_as(
        "SELECT * FROM users WHERE account_type = 'agent' AND controller_id = $1 ORDER BY created_at DESC LIMIT 100",
    )
    .bind(controller_id)
    .fetch_all(&state.db)
    .await?;

    let ss58_prefix = state.config.chain.ss58_prefix;
    Ok(Json(
        agents
            .into_iter()
            .map(|agent| to_agent_info(agent, ss58_prefix))
            .collect(),
    ))
}

/// Update one of the caller's agents' capabilities
/// PUT /api/v1/agents/:did/capabilities
///
/// Signed by the controller's own key over the change, around a challenge issued for the controller's account
/// (ADR-112), so an access token alone cannot widen what an agent may do.
pub async fn update_capabilities(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Path(did): Path<String>,
    Json(req): Json<UpdateAgentCapabilitiesRequest>,
) -> AppResult<Json<Value>> {
    validate_capabilities(&req.capabilities)?;
    let signed_by_controller = controller_signature(req.controller_signature.as_deref())?;
    let challenge = req.challenge.as_deref().ok_or_else(|| {
        AppError::ValidationError(
            "A change to an agent is signed around a challenge for your own account: send challenge (ADR-112)"
                .into(),
        )
    })?;

    let controller = controller_key(&state.db, controller_id).await?;
    let statement = change_statement(&controller, &did, challenge, &req.capabilities)?;
    consume_challenge_signed_over(
        &state.db,
        controller,
        challenge,
        statement.as_bytes(),
        signed_by_controller,
        AppError::ValidationError("The controller's signature does not cover this change".into()),
    )
    .await?;

    let capabilities_json = serde_json::to_value(&req.capabilities)
        .map_err(|_| AppError::ValidationError("Invalid capabilities".into()))?;

    let result = sqlx::query(
        "UPDATE users SET agent_capabilities = $1, updated_at = NOW() WHERE agent_did = $2 AND account_type = 'agent' AND controller_id = $3",
    )
    .bind(&capabilities_json)
    .bind(&did)
    .bind(controller_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Agent not found".into()));
    }

    Ok(Json(json!({
        "did": did,
        "capabilities": req.capabilities,
        "message": "Capabilities updated successfully"
    })))
}

/// Deactivate one of the caller's agents
/// DELETE /api/v1/agents/:did
///
/// The session alone suffices (ADR-112 decision 3): it takes authority away, and a stolen token can do no more with it
/// than stop an agent, which then registers a new key.
pub async fn deactivate_agent(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Path(did): Path<String>,
) -> AppResult<Json<Value>> {
    let result = sqlx::query(
        "UPDATE users SET status = 'inactive', updated_at = NOW() WHERE agent_did = $1 AND account_type = 'agent' AND controller_id = $2",
    )
    .bind(&did)
    .bind(controller_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Agent not found".into()));
    }

    Ok(Json(json!({
        "did": did,
        "status": "inactive",
        "message": "Agent deactivated successfully"
    })))
}

#[cfg(test)]
mod spending_limit_tests {
    use super::validate_spending_limit;

    /// A bounded agent needs a limit; none is ever negative, whatever the autonomy; zero is a limit (spend nothing).
    #[test]
    fn a_spending_limit_is_a_whole_number_of_cgt_and_never_negative() {
        assert!(validate_spending_limit("bounded", None).is_err());
        assert!(validate_spending_limit("bounded", Some(0)).is_ok());
        assert!(validate_spending_limit("bounded", Some(250)).is_ok());
        assert!(validate_spending_limit("supervised", None).is_ok());
        for autonomy in ["supervised", "bounded", "autonomous", "sovereign"] {
            assert!(
                validate_spending_limit(autonomy, Some(-1)).is_err(),
                "{autonomy}"
            );
            assert!(validate_spending_limit(autonomy, Some(i64::MIN)).is_err());
        }
    }
}

/// ADR-112: an agent is authorised, and widened, only by the controller's own key.
#[cfg(test)]
mod authorisation_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::handlers::auth::get_challenge;
    use crate::models::ChallengeRequest;
    use crate::services::{EmailConfig, EmailService};
    use axum::extract::Query;
    use sp_core::Pair as _;
    use sqlx::PgPool;

    /// No test here opens a session, so Redis is never reached.
    fn state(db: PgPool) -> Arc<AppState> {
        let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    fn key(seed: u8) -> sp_core::sr25519::Pair {
        sp_core::sr25519::Pair::from_seed(&[seed; 32])
    }

    fn account(pair: &sp_core::sr25519::Pair) -> ChainAccount {
        ChainAccount::from_stored(&pair.public().0).expect("32 bytes")
    }

    fn sign(pair: &sp_core::sr25519::Pair, message: &str) -> String {
        hex::encode(pair.sign(message.as_bytes()).0)
    }

    /// A person's QOR ID, with the key it proved or none (ADR-017).
    async fn person(db: &PgPool, username: &str, key: Option<&sp_core::sr25519::Pair>) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (username, discriminator, password_hash, email_verified, chain_account_id) \
             VALUES ($1, 1, 'x', TRUE, $2) RETURNING id",
        )
        .bind(username)
        .bind(key.map(|k| k.public().0.to_vec()))
        .fetch_one(db)
        .await
        .expect("a person")
    }

    /// A challenge for `pair`'s account, asked for the way a client asks.
    async fn challenge(state: &Arc<AppState>, pair: &sp_core::sr25519::Pair) -> String {
        let Json(issued) = get_challenge(
            State(state.clone()),
            Query(ChallengeRequest {
                address: None,
                account_id: Some(account(pair).account_id_hex()),
            }),
        )
        .await
        .expect("a challenge");
        issued.challenge
    }

    /// What an agent asks for, its own key's proof made; the controller's signature is the test's to add.
    fn asked(agent: &sp_core::sr25519::Pair, challenge: &str) -> RegisterAgentRequest {
        RegisterAgentRequest {
            name: "Lab Assistant".into(),
            capabilities: vec!["trade".into(), "read".into()],
            autonomy: "bounded".into(),
            spending_limit: Some(25),
            model: Some("claude-opus-5-5".into()),
            address: None,
            account_id: Some(account(agent).account_id_hex()),
            challenge: challenge.into(),
            signature: sign(agent, &crate::handlers::auth::challenge_message(challenge)),
            controller_signature: None,
        }
    }

    fn signed_by(
        controller: &sp_core::sr25519::Pair,
        agent: &sp_core::sr25519::Pair,
        mut req: RegisterAgentRequest,
    ) -> RegisterAgentRequest {
        let statement = authorisation_statement(&account(controller), &account(agent), &req)
            .expect("a statement");
        req.controller_signature = Some(sign(controller, &statement));
        req
    }

    fn refused_saying(result: AppResult<impl std::fmt::Debug>, words: &str) {
        match result {
            Err(AppError::ValidationError(message)) => {
                assert!(
                    message.contains(words),
                    "{message:?} does not say {words:?}"
                )
            }
            other => panic!("expected a refusal saying {words:?}, got {other:?}"),
        }
    }

    async fn agents_of(db: &PgPool, controller: Uuid) -> Vec<(String, Value)> {
        sqlx::query_as(
            "SELECT agent_did, agent_capabilities FROM users WHERE account_type = 'agent' AND controller_id = $1",
        )
        .bind(controller)
        .fetch_all(db)
        .await
        .expect("agents")
    }

    /// The launcher builds these exact bytes for the same request (`tools/qor-launcher/core/src/agents.rs`, the
    /// test of the same name): if either side changes, both tests must.
    #[test]
    fn the_statement_is_the_one_the_launcher_signs() {
        let req = asked(&key(2), "demiurge:1761177600:0123456789abcdef");
        let statement =
            authorisation_statement(&account(&key(1)), &account(&key(2)), &req).unwrap();
        assert_eq!(
            statement,
            format!(
                "demiurge:qor-id:agent-authorisation:v1\n\
                 controller:{}\n\
                 agent:{}\n\
                 challenge:demiurge:1761177600:0123456789abcdef\n\
                 name:Lab Assistant\n\
                 capabilities:read,trade\n\
                 autonomy:bounded\n\
                 spending_limit:25\n\
                 model:claude-opus-5-5",
                hex::encode(key(1).public().0),
                hex::encode(key(2).public().0),
            )
        );

        let mut plain = req;
        plain.spending_limit = None;
        plain.model = Some(String::new());
        let statement =
            authorisation_statement(&account(&key(1)), &account(&key(2)), &plain).unwrap();
        assert!(
            statement.ends_with("\nspending_limit:\nmodel:"),
            "{statement}"
        );
    }

    /// An agent's QOR ID is a QOR ID like any other (ADR-075): a name that would make one too long, or of other
    /// characters, is refused in words. Until 22 October 2026 the first failed in the database and the second was stored.
    #[test]
    fn an_agents_name_makes_a_qor_id_like_any_other() {
        assert_eq!(
            agent_qor_id("Lab Assistant1").unwrap(),
            "agent_lab_assistant1"
        );
        assert_eq!(agent_qor_id("bot").unwrap(), "agent_bot");
        for name in ["ab", "Research helper", "help!", "café", "a-b-c"] {
            assert!(agent_qor_id(name).is_err(), "{name}");
        }
    }

    /// A field that holds a line break could pose as another field; a capability named twice could pose as one more.
    #[test]
    fn a_statement_has_one_field_a_line_and_each_capability_once() {
        let (controller, agent) = (account(&key(1)), account(&key(2)));
        let pokes: [fn(&mut RegisterAgentRequest); 5] = [
            |r: &mut RegisterAgentRequest| r.name = "helper\ncapabilities:stake".into(),
            |r: &mut RegisterAgentRequest| r.model = Some("m\nautonomy:sovereign".into()),
            |r: &mut RegisterAgentRequest| r.challenge = "c\r".into(),
            |r: &mut RegisterAgentRequest| r.capabilities = vec!["read".into(), "read".into()],
            |r: &mut RegisterAgentRequest| r.capabilities = vec!["read,stake".into()],
        ];
        for poke in pokes {
            let mut req = asked(&key(2), "demiurge:1:00");
            poke(&mut req);
            assert!(
                authorisation_statement(&controller, &agent, &req).is_err(),
                "{req:?}"
            );
        }
        assert!(change_statement(&controller, "did:x\nagent:y", "c", &["read".into()]).is_err());
    }

    #[sqlx::test]
    async fn an_agent_is_registered_only_with_its_controllers_own_signature(db: PgPool) {
        let state = state(db.clone());
        let (owner, agent, stranger) = (key(1), key(2), key(3));
        let controller = person(&db, "owner", Some(&owner)).await;
        let challenge = challenge(&state, &agent).await;
        let register = |req| register_agent(State(state.clone()), Extension(controller), Json(req));

        refused_saying(
            register(asked(&agent, &challenge)).await,
            "controller_signature",
        );
        refused_saying(
            register(signed_by(&stranger, &agent, asked(&agent, &challenge))).await,
            "does not cover this authorisation",
        );
        // Signed for less than is asked: the person saw "read"; the request asks to trade too.
        let mut seen = asked(&agent, &challenge);
        seen.capabilities = vec!["read".into()];
        let mut asking_more = asked(&agent, &challenge);
        asking_more.controller_signature = signed_by(&owner, &agent, seen).controller_signature;
        refused_saying(
            register(asking_more).await,
            "does not cover this authorisation",
        );
        assert!(
            agents_of(&db, controller).await.is_empty(),
            "nothing refused was recorded"
        );

        let (status, Json(made)) = register(signed_by(&owner, &agent, asked(&agent, &challenge)))
            .await
            .expect("authorised by the controller's own key");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(made.account_id, account(&agent).account_id_hex());
        let recorded = agents_of(&db, controller).await;
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].1, json!(["trade", "read"]));

        // The same signatures again: the challenge they name is spent.
        refused_saying(
            register(signed_by(&owner, &agent, asked(&agent, &challenge))).await,
            "Invalid or expired challenge",
        );
    }

    /// ADR-017: a QOR ID may have proved no key, and then it has nothing to authorise an agent with.
    #[sqlx::test]
    async fn a_qor_id_with_no_proved_key_cannot_authorise_an_agent(db: PgPool) {
        let state = state(db.clone());
        let controller = person(&db, "passwordonly", None).await;
        let challenge = challenge(&state, &key(2)).await;
        refused_saying(
            register_agent(
                State(state.clone()),
                Extension(controller),
                Json(signed_by(&key(1), &key(2), asked(&key(2), &challenge))),
            )
            .await,
            "has proved none",
        );
    }

    /// Delegated authority does not delegate itself again, signature or none.
    #[sqlx::test]
    async fn an_agent_cannot_authorise_an_agent(db: PgPool) {
        let state = state(db.clone());
        let controller = person(&db, "agentlike", Some(&key(1))).await;
        sqlx::query("UPDATE users SET account_type = 'agent' WHERE id = $1")
            .bind(controller)
            .execute(&db)
            .await
            .unwrap();
        let challenge = challenge(&state, &key(2)).await;
        let result = register_agent(
            State(state.clone()),
            Extension(controller),
            Json(signed_by(&key(1), &key(2), asked(&key(2), &challenge))),
        )
        .await;
        assert!(
            matches!(result, Err(AppError::InsufficientPermissions)),
            "{result:?}"
        );
    }

    #[sqlx::test]
    async fn capabilities_change_only_with_the_controllers_own_signature(db: PgPool) {
        let state = state(db.clone());
        let (owner, agent) = (key(1), key(2));
        let controller = person(&db, "owner", Some(&owner)).await;
        let first = challenge(&state, &agent).await;
        let _registered = register_agent(
            State(state.clone()),
            Extension(controller),
            Json(signed_by(&owner, &agent, asked(&agent, &first))),
        )
        .await
        .expect("registered");
        let did = agents_of(&db, controller).await.remove(0).0;
        let wider: Vec<String> = vec!["read".into(), "trade".into(), "transfer".into()];
        let change = |challenge: Option<String>, signature: Option<String>| {
            update_capabilities(
                State(state.clone()),
                Extension(controller),
                Path(did.clone()),
                Json(UpdateAgentCapabilitiesRequest {
                    capabilities: wider.clone(),
                    challenge,
                    controller_signature: signature,
                }),
            )
        };
        let signed = |challenge: &str, capabilities: &[String]| {
            sign(
                &owner,
                &change_statement(&account(&owner), &did, challenge, capabilities).unwrap(),
            )
        };

        refused_saying(change(None, None).await, "controller_signature");
        // A challenge issued for the agent's account is not one for the controller's.
        let agents_challenge = challenge(&state, &agent).await;
        refused_saying(
            change(
                Some(agents_challenge.clone()),
                Some(signed(&agents_challenge, &wider)),
            )
            .await,
            "Invalid or expired challenge",
        );
        // Signed for a narrower change than is asked.
        let mine = challenge(&state, &owner).await;
        refused_saying(
            change(Some(mine.clone()), Some(signed(&mine, &["read".into()]))).await,
            "does not cover this change",
        );
        assert_eq!(
            agents_of(&db, controller).await[0].1,
            json!(["trade", "read"]),
            "unchanged"
        );

        let Json(changed) = change(Some(mine.clone()), Some(signed(&mine, &wider)))
            .await
            .expect("changed with the controller's own signature");
        assert_eq!(changed["capabilities"], json!(wider));
        assert_eq!(agents_of(&db, controller).await[0].1, json!(wider));
        refused_saying(
            change(Some(mine.clone()), Some(signed(&mine, &wider))).await,
            "Invalid or expired challenge",
        );
    }
}
