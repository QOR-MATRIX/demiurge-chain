//! Agent management handlers.
//!
//! An agent is an account a human controls. The agent generates its own
//! Ed25519 keypair; QOR ID authorises the public key and never generates,
//! stores or returns a private key (ADR-014). Every endpoint acts only on
//! agents the authenticated user controls.

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
use crate::handlers::auth::{ChainAccount, consume_signed_challenge};
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
/// for it, proving the agent holds the private key.
pub async fn register_agent(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Json(req): Json<RegisterAgentRequest>,
) -> AppResult<(StatusCode, Json<AgentRegistrationResponse>)> {
    // Validate input
    if req.name.len() < 3 || req.name.len() > 32 {
        return Err(AppError::ValidationError(
            "Agent name must be 3-32 characters".into(),
        ));
    }

    validate_capabilities(&req.capabilities)?;
    validate_autonomy(&req.autonomy)?;

    // Bounded agents require spending limit
    if req.autonomy == "bounded" && req.spending_limit.is_none() {
        return Err(AppError::ValidationError(
            "Bounded agents require a spending_limit".into(),
        ));
    }

    // Only an active human account can control agents. An agent cannot create
    // further agents, which would let delegated authority delegate itself again.
    let controller_type: Option<Option<String>> =
        sqlx::query_scalar("SELECT account_type FROM users WHERE id = $1 AND status = 'active'")
            .bind(controller_id)
            .fetch_optional(&state.db)
            .await?;

    match controller_type {
        None => return Err(AppError::InsufficientPermissions),
        Some(Some(kind)) if kind != "human" => return Err(AppError::InsufficientPermissions),
        _ => {}
    }

    // Proof of possession: the agent's own key signed a challenge issued for it.
    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

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

    let agent_name = format!("agent_{}", req.name.to_lowercase().replace(' ', "_"));
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
    .bind(&req.model)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if matches!(&e, sqlx::Error::Database(db) if db.constraint() == Some("users_username_unique")) {
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
pub async fn update_capabilities(
    State(state): State<Arc<AppState>>,
    Extension(controller_id): Extension<Uuid>,
    Path(did): Path<String>,
    Json(req): Json<UpdateAgentCapabilitiesRequest>,
) -> AppResult<Json<Value>> {
    validate_capabilities(&req.capabilities)?;

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
