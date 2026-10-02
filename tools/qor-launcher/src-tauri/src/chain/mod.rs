//! The launcher's client for the **Demiurge Substrate chain** in `chain/`
//! (roadmap L3.1, [ADR-040]).
//!
//! # What it talks to, and what it does not
//!
//! This client speaks standard Substrate RPC to the chain in `chain/`, which
//! since M3.5 is the only chain in this repository. The custom devnet that
//! preceded it left two things worth knowing here: its private vocabulary
//! (`chain_getBlockNumber`, `balances_getBalance`, `account_getTransactionNonce`,
//! `chain_getTransactionHistory`) is gone from the launcher with this module,
//! and so is its distinction between a request nonce and a transaction nonce.
//! A Substrate account has one nonce.
//!
//! [`status`](ChainClient::status) reads `system_chain`, so the launcher shows
//! which chain answered rather than leaving that to a person with `curl`.
//!
//! # How a transaction is made
//!
//! Every call is addressed dynamically, against the metadata the connected node
//! serves: nothing about the runtime is compiled in, so a runtime upgrade does
//! not require a launcher release.
//!
//! Signing is deliberately not `subxt`'s [`subxt::tx::Signer`], which is
//! synchronous and infallible. The launcher builds the transaction, reads the
//! bytes that must be signed, and hands *those* to the vault, which draws the
//! host dialog (roadmap L1.4) and signs inside its own lock. A vault that has
//! locked meanwhile, and a person who declines, both stop the transaction; an
//! infallible signer could only have returned nonsense for the node to refuse.
//!
//! `signer_payload()` already applies Substrate's rule that a payload longer
//! than 256 bytes is signed as its Blake2-256 hash, so the vault signs the
//! bytes it is given, verbatim, and must not hash them again (ADR-040, finding
//! 5).
//!
//! [ADR-040]: ../../../../docs/decisions/ADR-040-the-launchers-chain-client-is-subxt.md

pub mod assets;
pub mod config;
pub mod market;
pub mod sales;

use std::time::{Duration, Instant};

use scale_decode::DecodeAsType;
use serde::{Deserialize, Serialize};
use subxt::config::DefaultExtrinsicParamsBuilder;
use subxt::rpcs::{LegacyRpcMethods, RpcClient};
use subxt::utils::{AccountId32, MultiAddress, MultiSignature};
use subxt::{dynamic, OnlineClient};

use crate::error::{QorError, QorResult};
use crate::vault::{canonical_address, normalise_address, Vault};
use crate::{Confirm, Prompt};

use config::{DemiurgeConfig, DemiurgeRpcConfig};

/// Default endpoint. Overridable from settings.
///
/// A WebSocket address: the RPC client builds a WebSocket transport and nothing
/// else (ADR-040, finding 6). Nothing answers here yet; nothing is deployed.
pub const DEFAULT_RPC: &str = "wss://rpc.demiurge.cloud";

/// A local node started by the launcher listens here.
pub const LOCAL_RPC: &str = "ws://127.0.0.1:9944";

/// How long a transfer waits for finality before it is reported as a timeout
/// rather than a failure (ADR-040, decision 7).
const FINALITY_TIMEOUT: Duration = Duration::from_secs(120);

/// How long to wait for a node to answer at all.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

/// Live chain summary shown in the launcher shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStatus {
    pub endpoint: String,
    pub reachable: bool,
    /// What the node calls itself, from `system_chain`. This is what tells the
    /// Substrate chain apart from the custom devnet (`HANDOFF.md` §2.0).
    pub chain_name: Option<String>,
    pub block_number: Option<u64>,
    /// The highest finalised block. The custom devnet had no finality at all.
    pub finalized_number: Option<u64>,
    pub latency_ms: Option<u64>,
    /// Populated when the node is unreachable or misbehaving.
    pub detail: Option<String>,
}

impl ChainStatus {
    fn unreachable(endpoint: String, detail: String) -> Self {
        Self {
            endpoint,
            reachable: false,
            chain_name: None,
            block_number: None,
            finalized_number: None,
            latency_ms: None,
            detail: Some(detail),
        }
    }
}

/// Result of a submitted transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferReceipt {
    pub tx_hash: String,
    pub from: String,
    pub to: String,
    /// Amount in Sparks, as a decimal string, because `u128` does not survive
    /// a round trip through JavaScript's `number`.
    pub amount_sparks: String,
    pub amount_cgt: String,
    /// The finalised block the transfer is in. Reported only once GRANDPA has
    /// finalised it (ADR-018, ADR-040 decision 7).
    pub block_hash: String,
}

/// One entry in an account's history.
///
/// Kept as the shape the launcher's history surface expects. Nothing fills it
/// yet: see [`ChainClient::history`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub hash: String,
    pub from: String,
    pub to: Option<String>,
    pub amount_sparks: Option<String>,
    /// The amount rendered in CGT, so the UI never does denomination maths.
    pub amount_cgt: Option<String>,
    pub nonce: u64,
    pub block_number: Option<u64>,
    /// `"in"` or `"out"`, relative to the queried account.
    pub direction: Option<String>,
    pub status: String,
}

/// Outcome of a starter-grant claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimResult {
    pub success: bool,
    pub amount_sparks: String,
    pub amount_cgt: String,
    pub message: String,
}

/// The part of `System::Account` the launcher reads.
///
/// Decoded by name against the node's own metadata, so a runtime that adds
/// fields does not break it.
#[derive(Debug, DecodeAsType)]
struct AccountInfo {
    data: AccountData,
}

#[derive(Debug, DecodeAsType)]
struct AccountData {
    free: u128,
}

/// A live connection, and what it reported about itself when it was made.
#[derive(Clone)]
struct Connection {
    endpoint: String,
    chain_name: String,
    api: OnlineClient<DemiurgeConfig>,
    rpc: LegacyRpcMethods<DemiurgeRpcConfig>,
}

/// The launcher's chain client.
pub struct ChainClient {
    endpoint: parking_lot::RwLock<String>,
    /// The current connection, made on first use and remade when the endpoint
    /// changes or the last call failed.
    connection: tokio::sync::Mutex<Option<Connection>>,
}

impl ChainClient {
    pub fn new(endpoint: impl Into<String>) -> QorResult<Self> {
        Ok(Self {
            endpoint: parking_lot::RwLock::new(normalise_endpoint(&endpoint.into())?),
            connection: tokio::sync::Mutex::new(None),
        })
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.read().clone()
    }

    /// Point the client somewhere else. The next call connects there.
    ///
    /// Approving the change is the caller's job, host-side (roadmap L1.4).
    pub fn set_endpoint(&self, endpoint: impl Into<String>) -> QorResult<()> {
        *self.endpoint.write() = normalise_endpoint(&endpoint.into())?;
        Ok(())
    }

    /// The current connection, made if there is not one already.
    async fn connected(&self) -> QorResult<Connection> {
        let wanted = self.endpoint();
        let mut held = self.connection.lock().await;

        if let Some(existing) = held.as_ref() {
            if existing.endpoint == wanted {
                return Ok(existing.clone());
            }
        }
        // Either nothing is connected or it is connected somewhere else.
        *held = None;

        let made = tokio::time::timeout(CONNECT_TIMEOUT, connect(&wanted))
            .await
            .map_err(|_| QorError::Network(format!("{wanted} did not answer in time")))??;

        *held = Some(made.clone());
        Ok(made)
    }

    /// Drop the connection, so the next call makes a fresh one.
    async fn drop_connection(&self) {
        *self.connection.lock().await = None;
    }

    /// Probe the node: which chain it is, how far it has got, and how fast it
    /// answered.
    pub async fn status(&self) -> ChainStatus {
        let endpoint = self.endpoint();
        let started = Instant::now();

        let connection = match self.connected().await {
            Ok(connection) => connection,
            Err(e) => return ChainStatus::unreachable(endpoint, e.to_string()),
        };

        let best = connection.rpc.chain_get_header(None).await;
        let latency = started.elapsed().as_millis() as u64;

        let best = match best {
            Ok(header) => header,
            Err(e) => {
                self.drop_connection().await;
                return ChainStatus::unreachable(endpoint, describe_rpc_error(&e));
            }
        };

        // A missing finalised head is worth showing, not worth failing over.
        let finalized_number = match connection.rpc.chain_get_finalized_head().await {
            Ok(hash) => match connection.rpc.chain_get_header(Some(hash)).await {
                Ok(Some(header)) => Some(header.number),
                _ => None,
            },
            Err(_) => None,
        };

        ChainStatus {
            endpoint,
            reachable: true,
            chain_name: Some(connection.chain_name.clone()),
            block_number: best.as_ref().map(|header| header.number),
            finalized_number,
            latency_ms: Some(latency),
            detail: best
                .is_none()
                .then(|| "the node answered but reported no best header".to_string()),
        }
    }

    /// Free balance in Sparks.
    pub async fn balance(&self, address: &str) -> QorResult<u128> {
        let account = normalise_address(address)?;
        let connection = self.connected().await?;

        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };

        let entry = dynamic::storage::<([u8; 32],), AccountInfo>("System", "Account");
        let value = at
            .storage()
            .fetch(entry, (account,))
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the account: {e}")))?;

        // An account the chain has never seen has no entry, and no balance.
        match value.decode() {
            Ok(info) => Ok(info.data.free),
            Err(e) => Err(QorError::Rpc(format!(
                "the account's balance could not be decoded: {e}"
            ))),
        }
    }

    /// The nonce the account's next transaction must carry.
    ///
    /// `system_accountNextIndex` counts what is waiting in the pool as well as
    /// what is on chain, which is what lets two transfers be sent in a row
    /// (ADR-040, decision 5).
    pub async fn nonce(&self, address: &str) -> QorResult<u64> {
        let account = AccountId32(normalise_address(address)?);
        let connection = self.connected().await?;

        connection
            .rpc
            .system_account_next_index(&account)
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the account's nonce: {e}")))
    }

    /// Send CGT.
    ///
    /// The mistakes a person can make are caught before anything reaches the
    /// network and before anyone is asked to approve anything.
    pub async fn transfer(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        to: &str,
        amount_sparks: u128,
    ) -> QorResult<TransferReceipt> {
        if amount_sparks == 0 {
            return Err(QorError::BadAmount(
                "amount must be greater than zero".into(),
            ));
        }

        let from_account = normalise_address(from)?;
        let to_account = normalise_address(to)?;

        if from_account == to_account {
            return Err(QorError::BadAddress(
                "cannot send CGT to the sending account".into(),
            ));
        }

        // Shown in the dialog and in the receipt, so both name the accounts the
        // same way however the caller spelled them.
        let from_address = canonical_address(from)?;
        let to_address = canonical_address(to)?;

        let connection = self.connected().await?;
        let signer = AccountId32(from_account);
        let nonce = connection
            .rpc
            .system_account_next_index(&signer)
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the account's nonce: {e}")))?;

        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };

        // `transfer_keep_alive`: a wallet does not reap its own account by
        // accident (ADR-036, ADR-040 decision 6). `dest` is a `MultiAddress`,
        // because the runtime looks accounts up with `AccountIdLookup`
        // (ADR-041).
        let call = dynamic::transaction(
            "Balances",
            "transfer_keep_alive",
            (
                MultiAddress::<AccountId32, ()>::Id(AccountId32(to_account)),
                amount_sparks,
            ),
        );
        let params = DefaultExtrinsicParamsBuilder::<DemiurgeConfig>::new()
            .nonce(nonce)
            .build();

        let mut signable = at
            .transactions()
            .create_signable(&call, &signer, params)
            .await
            .map_err(|e| QorError::Rpc(format!("the transfer could not be built: {e}")))?;

        let payload = signable
            .signer_payload()
            .map_err(|e| QorError::Rpc(format!("the transfer could not be prepared: {e}")))?;

        // The dialog describes the transfer itself, not the bytes.
        let prompt = transfer_prompt(
            &from_address,
            &to_address,
            amount_sparks,
            &connection.chain_name,
            &connection.endpoint,
        );
        let signature = vault.sign(confirm, &prompt, from, &payload).await?;
        let signature = decode_signature(&signature)?;

        let submittable = signable
            .sign_with_account_and_signature(&signer, &MultiSignature::Sr25519(signature))
            .map_err(|e| QorError::Rpc(format!("the transfer could not be signed: {e}")))?;

        let tx_hash = format!("{:?}", submittable.hash());

        let progress = submittable.submit_and_watch().await.map_err(|e| {
            QorError::Rpc(format!("the node refused the transfer: {}", plainly(&e)))
        })?;

        // Finalised first, then checked for success, because the finalised block
        // is what the receipt names and `wait_for_finalized_success` does not
        // hand it back.
        let in_block =
            match tokio::time::timeout(FINALITY_TIMEOUT, progress.wait_for_finalized()).await {
                Ok(Ok(in_block)) => in_block,
                Ok(Err(e)) => {
                    return Err(QorError::Rpc(format!(
                        "the transfer {tx_hash} was not accepted: {}",
                        plainly(&e)
                    )))
                }
                // Not a failure. It may yet finalise, and must not be sent twice.
                Err(_) => {
                    return Err(QorError::Rpc(format!(
                        "the transfer {tx_hash} was submitted but has not finalised within \
                     {} seconds. It may still finalise; check the account's balance before \
                     sending it again.",
                        FINALITY_TIMEOUT.as_secs()
                    )))
                }
            };

        let block_hash = format!("{:?}", in_block.block_hash());
        in_block.wait_for_success().await.map_err(|e| {
            QorError::Rpc(format!(
                "the transfer {tx_hash} was finalised in {block_hash} but failed: {}",
                plainly(&e)
            ))
        })?;

        Ok(TransferReceipt {
            tx_hash,
            from: from_address,
            to: to_address,
            amount_sparks: amount_sparks.to_string(),
            amount_cgt: crate::cgt::format_cgt(amount_sparks),
            block_hash,
        })
    }

    /// Claim the starter grant. **Refused: there is nothing to claim from.**
    ///
    /// The claim belongs in the launcher rather than in QOR ID, because the
    /// launcher is the only party holding the key that proves the account is
    /// the claimer's. QOR ID used to mint it unauthenticated, which is closed
    /// (security track item 3, R-3). That reasoning is unchanged. What is
    /// missing now is the other half: the chain has no issuance mechanism, the
    /// issuance rate is OPEN-1 and the genesis split is OPEN-2, and AGENTS.md
    /// §5 forbids adding any path that creates CGT outside `--dev`.
    pub async fn claim_starter(
        &self,
        _vault: &Vault,
        _confirm: &dyn Confirm,
        address: &str,
    ) -> QorResult<ClaimResult> {
        normalise_address(address)?;
        Err(QorError::Rpc(
            "There is no starter grant to claim. The chain has no issuance mechanism yet: \
             the issuance rate and the genesis split are open economic questions (OPEN-1, \
             OPEN-2), and nothing may create CGT until they are decided."
                .into(),
        ))
    }

    /// Transactions involving an account. **Refused: there is nowhere to read
    /// them from yet.**
    ///
    /// The chain has no history RPC and will not grow one: provenance comes
    /// from events, an archive node and an indexer (ADR-028). Walking blocks
    /// from the launcher would be slow, bounded by pruning, and quietly
    /// incomplete, which is worse than saying so.
    pub async fn history(&self, address: &str, _limit: u64) -> QorResult<Vec<HistoryEntry>> {
        normalise_address(address)?;
        Err(QorError::Rpc(
            "Transaction history is not available yet. A Substrate node serves no history \
             RPC; it comes from an indexer over the chain's events (ADR-028), which is not \
             built."
                .into(),
        ))
    }

    /// Turn a client error into one a person can act on, and forget the
    /// connection, so a node that went away is reconnected to rather than
    /// asked again down a socket that is no longer there.
    async fn lost_the_node(&self, error: impl std::fmt::Display) -> QorError {
        self.drop_connection().await;
        QorError::Rpc(format!("the node did not answer: {error}"))
    }
}

/// Connect, and ask the node which chain it is.
async fn connect(endpoint: &str) -> QorResult<Connection> {
    // `from_insecure_url` is deliberate: `ws://127.0.0.1:9944` is the local
    // node this launcher starts, and refusing it would refuse the only chain
    // that exists today. A remote endpoint is a person's own choice, approved
    // host-side before it is ever used (roadmap L1.4).
    let rpc_client = RpcClient::from_insecure_url(endpoint)
        .await
        .map_err(|e| QorError::Network(describe_connect_error(endpoint, &e)))?;

    let rpc = LegacyRpcMethods::<DemiurgeRpcConfig>::new(rpc_client.clone());

    let chain_name = rpc.system_chain().await.map_err(|e| {
        QorError::Rpc(format!(
            "{endpoint} answered, but not as a Substrate chain: {}. Check that it is a 
             Demiurge node and that the address names its RPC port.",
            plainly(&e)
        ))
    })?;

    let api = OnlineClient::<DemiurgeConfig>::from_rpc_client(rpc_client)
        .await
        .map_err(|e| {
            QorError::Rpc(format!(
                "{endpoint} would not hand over its runtime metadata: {}",
                plainly(&e)
            ))
        })?;

    Ok(Connection {
        endpoint: endpoint.to_string(),
        chain_name,
        api,
        rpc,
    })
}

/// Accept an endpoint, and say why if it cannot be one.
///
/// An `http://` or `https://` endpoint stored by an earlier launcher is
/// upgraded rather than refused: it names the same node on the same port, and
/// the only thing that changed is that the client speaks WebSocket (ADR-040,
/// decision 8).
pub fn normalise_endpoint(raw: &str) -> QorResult<String> {
    let trimmed = raw.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(QorError::Rpc("the chain endpoint is empty".into()));
    }

    let upgraded = if let Some(rest) = trimmed.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = trimmed.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        trimmed.to_string()
    };

    if !upgraded.starts_with("ws://") && !upgraded.starts_with("wss://") {
        return Err(QorError::Rpc(format!(
            "{trimmed} is not a node address: it must start with ws:// or wss://"
        )));
    }

    Ok(upgraded)
}

/// What the person approving a transfer is shown.
fn transfer_prompt(
    from: &str,
    to: &str,
    amount_sparks: u128,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    Prompt {
        title: "Approve this transfer".into(),
        body: format!(
            "Send {} {}\n\nFrom: {from}\nTo: {to}\n\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this transfer with your key and sends it. It cannot be undone.",
            crate::cgt::format_cgt_grouped(amount_sparks),
            crate::cgt::SYMBOL
        ),
        approve: "Send".into(),
    }
}

/// The vault hands back hex; the chain wants 64 bytes.
fn decode_signature(hex_signature: &str) -> QorResult<[u8; 64]> {
    let bytes = hex::decode(hex_signature)
        .map_err(|_| QorError::Internal("the vault returned a signature that is not hex".into()))?;

    bytes.try_into().map_err(|_| {
        QorError::Internal("the vault returned a signature of the wrong length".into())
    })
}

/// Errors from the RPC layer nest their causes; a person needs the innermost.
fn plainly(error: &impl std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(inner) = source {
        message = format!("{message}: {inner}");
        source = inner.source();
    }
    message
}

fn describe_connect_error(endpoint: &str, error: &impl std::error::Error) -> String {
    format!("cannot reach {endpoint}: {}", plainly(error))
}

fn describe_rpc_error(error: &impl std::error::Error) -> String {
    plainly(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Scripted;

    /// An endpoint that is never listening, so every test here fails fast and
    /// none of them needs a node.
    const NOWHERE: &str = "ws://127.0.0.1:1";

    fn temp_vault(tag: &str) -> (Vault, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!("qor-chain-{tag}-{}", std::process::id()));
        (crate::vault::testing::vault_in(&dir), dir)
    }

    #[test]
    fn endpoints_must_name_a_node() {
        assert_eq!(
            normalise_endpoint(" ws://127.0.0.1:9944/ ").unwrap(),
            LOCAL_RPC
        );
        assert_eq!(normalise_endpoint(DEFAULT_RPC).unwrap(), DEFAULT_RPC);

        for bad in [
            "",
            "   ",
            "127.0.0.1:9944",
            "ftp://node",
            "rpc.demiurge.cloud",
        ] {
            assert_eq!(
                normalise_endpoint(bad).unwrap_err().kind(),
                "rpc",
                "{bad:?} is not an endpoint"
            );
        }
    }

    /// A launcher that stored `http://127.0.0.1:9944` before ADR-040 points at
    /// the same node; it should carry on working rather than refuse to start.
    #[test]
    fn a_stored_http_endpoint_is_upgraded_not_refused() {
        assert_eq!(
            normalise_endpoint("http://127.0.0.1:9944").unwrap(),
            LOCAL_RPC
        );
        assert_eq!(
            normalise_endpoint("https://rpc.demiurge.cloud").unwrap(),
            DEFAULT_RPC
        );
    }

    #[test]
    fn the_endpoint_is_swappable_at_runtime() {
        let client = ChainClient::new(DEFAULT_RPC).unwrap();
        assert_eq!(client.endpoint(), DEFAULT_RPC);

        client.set_endpoint(LOCAL_RPC).unwrap();
        assert_eq!(client.endpoint(), LOCAL_RPC);

        assert!(client.set_endpoint("not an endpoint").is_err());
        assert_eq!(
            client.endpoint(),
            LOCAL_RPC,
            "a refused endpoint changes nothing"
        );
    }

    #[tokio::test]
    async fn status_reports_unreachable_without_panicking() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let status = client.status().await;

        assert!(!status.reachable);
        assert!(status.block_number.is_none());
        assert!(status.chain_name.is_none());
        assert!(status.finalized_number.is_none());
        assert!(
            status.detail.is_some(),
            "an unreachable node must explain itself"
        );
    }

    #[tokio::test]
    async fn transfer_rejects_obvious_mistakes_before_touching_the_network() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let (vault, dir) = temp_vault("mistakes");
        let confirm = Scripted::approving();

        let same = crate::vault::derive::address_of(&[0xabu8; 32]);
        assert_eq!(
            client
                .transfer(&vault, &confirm, &same, &same, 100)
                .await
                .unwrap_err()
                .kind(),
            "bad_address",
            "sending to yourself is caught locally"
        );

        let other = crate::vault::derive::address_of(&[0xcdu8; 32]);
        assert_eq!(
            client
                .transfer(&vault, &confirm, &same, &other, 0)
                .await
                .unwrap_err()
                .kind(),
            "bad_amount",
            "zero amounts are caught locally"
        );
        assert_eq!(
            confirm.times_asked(),
            0,
            "nobody is asked to approve a doomed transfer"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Nothing is signed, and nobody is asked, before the node has accepted the
    /// transaction as buildable.
    #[tokio::test]
    async fn an_unreachable_node_stops_a_transfer_before_the_dialog() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let (vault, dir) = temp_vault("unreachable");
        let confirm = Scripted::approving();

        let from = crate::vault::derive::address_of(&[0x01u8; 32]);
        let to = crate::vault::derive::address_of(&[0x02u8; 32]);

        let error = client
            .transfer(&vault, &confirm, &from, &to, 100)
            .await
            .unwrap_err();
        assert!(
            matches!(error.kind(), "network" | "rpc"),
            "an unreachable node is a network failure, not a signing one: {error}"
        );
        assert_eq!(
            confirm.times_asked(),
            0,
            "no dialog is drawn for a transfer that cannot be built"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Both calls refuse for a reason that is true of this chain, and neither
    /// pretends the devnet's answer.
    #[tokio::test]
    async fn history_and_the_starter_claim_refuse_with_their_real_reasons() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let (vault, dir) = temp_vault("refusals");
        let confirm = Scripted::approving();
        let address = crate::vault::derive::address_of(&[0x03u8; 32]);

        let history = client.history(&address, 50).await.unwrap_err();
        assert_eq!(history.kind(), "rpc");
        assert!(
            history.to_string().contains("ADR-028"),
            "history must name where it will come from: {history}"
        );

        let claim = client
            .claim_starter(&vault, &confirm, &address)
            .await
            .unwrap_err();
        assert_eq!(claim.kind(), "rpc");
        assert!(
            claim.to_string().contains("OPEN-1"),
            "the claim must name what it waits on: {claim}"
        );

        assert_eq!(
            confirm.times_asked(),
            0,
            "nothing that refuses asks for approval first"
        );

        // An address that is not an address is still caught first.
        assert_eq!(
            client
                .history("not-an-address", 50)
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_signature_from_the_vault_is_sixty_four_bytes_of_hex() {
        assert!(decode_signature(&"ab".repeat(64)).is_ok());
        assert_eq!(
            decode_signature(&"ab".repeat(63)).unwrap_err().kind(),
            "internal",
            "a short signature is not silently padded"
        );
        assert_eq!(decode_signature("nonsense").unwrap_err().kind(), "internal");
    }

    /// The dialog must say what is being sent, where, and on which chain.
    #[test]
    fn the_transfer_prompt_names_what_it_is_approving() {
        let from = crate::vault::derive::address_of(&[0x04u8; 32]);
        let to = crate::vault::derive::address_of(&[0x05u8; 32]);
        let prompt = transfer_prompt(
            &from,
            &to,
            2 * crate::cgt::SPARKS_PER_CGT,
            "Demiurge Development",
            LOCAL_RPC,
        );

        assert!(prompt.body.contains(&from));
        assert!(prompt.body.contains(&to));
        assert!(prompt.body.contains("2.00 CGT"));
        assert!(prompt.body.contains("Demiurge Development"));
        assert!(prompt.body.contains(LOCAL_RPC));
    }
}

/// The whole path, against a running Demiurge development node.
///
/// Ignored by default, because it needs a node. Everything above this module
/// can be checked without one; this is the part that cannot, and it is the
/// evidence that the client actually works (ADR-040).
///
/// ```text
/// chain/target/release/demiurge-node --dev --tmp
/// cargo test -p qor-launcher --lib chain::live -- --ignored --nocapture
/// ```
///
/// Point it elsewhere with `QOR_LAUNCHER_TEST_RPC`.
#[cfg(test)]
mod live {
    use super::*;
    use crate::testing::Scripted;
    use sp_core::{crypto::Pair as _, sr25519};
    use subxt::config::DefaultExtrinsicParamsBuilder;

    /// Enough to clear the existential deposit of 100 CGT (ADR-036) twice
    /// over, on both sides of the transfer under test.
    const FUNDING: u128 = 1_000 * crate::cgt::SPARKS_PER_CGT;
    const SENDING: u128 = 200 * crate::cgt::SPARKS_PER_CGT;

    fn endpoint() -> String {
        std::env::var("QOR_LAUNCHER_TEST_RPC").unwrap_or_else(|_| LOCAL_RPC.to_string())
    }

    /// Alice signs for every live test, and the tests run in parallel. Two
    /// fundings reading her next nonce at once get the same one, and the node
    /// refuses the second as a low-priority replacement. One at a time.
    static ALICE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    /// Put CGT into `account` from the development chain's Alice, which is the
    /// only source of it that exists: the chain has no issuance (OPEN-1) and
    /// the launcher must never grow a path that creates any.
    async fn fund_from_alice(endpoint: &str, account: [u8; 32], amount: u128) {
        let _one_at_a_time = ALICE.lock().await;
        let alice = sr25519::Pair::from_string("//Alice", None).expect("the dev key");
        let alice_account = AccountId32(alice.public().0);

        let rpc_client = RpcClient::from_insecure_url(endpoint)
            .await
            .expect("a node to fund from");
        let rpc = LegacyRpcMethods::<DemiurgeRpcConfig>::new(rpc_client.clone());
        let api = OnlineClient::<DemiurgeConfig>::from_rpc_client(rpc_client)
            .await
            .expect("metadata");

        let nonce = rpc
            .system_account_next_index(&alice_account)
            .await
            .expect("alice's nonce");
        let call = dynamic::transaction(
            "Balances",
            "transfer_keep_alive",
            (
                MultiAddress::<AccountId32, ()>::Id(AccountId32(account)),
                amount,
            ),
        );
        let params = DefaultExtrinsicParamsBuilder::<DemiurgeConfig>::new()
            .nonce(nonce)
            .build();

        let at = api.at_current_block().await.expect("a block");
        let mut signable = at
            .transactions()
            .create_signable(&call, &alice_account, params)
            .await
            .expect("a signable transfer");
        let payload = signable.signer_payload().expect("a payload");
        let signature = alice.sign(&payload).0;

        signable
            .sign_with_account_and_signature(&alice_account, &MultiSignature::Sr25519(signature))
            .expect("a signed transfer")
            .submit_and_watch()
            .await
            .expect("submission")
            .wait_for_finalized_success()
            .await
            .expect("funding to finalise");
    }

    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_transfer_is_built_from_metadata_signed_by_the_vault_and_finalised() {
        let endpoint = endpoint();
        let client = ChainClient::new(&endpoint).unwrap();

        // 1. The launcher knows which chain answered.
        let status = client.status().await;
        assert!(
            status.reachable,
            "no node at {endpoint}: {:?}",
            status.detail
        );
        let name = status.chain_name.clone().expect("a chain name");
        assert!(
            name.starts_with("Demiurge"),
            "this is not the Demiurge chain, it is {name:?}"
        );
        assert!(
            status.finalized_number.is_some(),
            "this chain has GRANDPA (ADR-018), so it must report a finalised height"
        );

        // 2. A fresh vault, so the run starts from an account nobody has used.
        let dir = std::env::temp_dir().join(format!(
            "qor-chain-live-{}-{}",
            std::process::id(),
            status.block_number.unwrap_or_default()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let vault = crate::vault::testing::vault_in(&dir);
        let phrase = crate::vault::derive::generate_mnemonic().unwrap();
        let accounts = vault.create(&phrase).unwrap();
        let sender = accounts[0].address.clone();
        let recipient = vault.add_account("second").unwrap().address;

        assert_eq!(
            client.balance(&sender).await.unwrap(),
            0,
            "a brand new account holds nothing"
        );
        assert_eq!(client.nonce(&sender).await.unwrap(), 0);

        fund_from_alice(&endpoint, normalise_address(&sender).unwrap(), FUNDING).await;
        assert_eq!(client.balance(&sender).await.unwrap(), FUNDING);

        // 3. A declined transfer moves nothing and signs nothing.
        let declining = Scripted::declining();
        let declined = client
            .transfer(&vault, &declining, &sender, &recipient, SENDING)
            .await
            .unwrap_err();
        assert_eq!(declined.kind(), "declined");
        assert_eq!(declining.times_asked(), 1, "the person was asked once");
        assert_eq!(
            client.balance(&sender).await.unwrap(),
            FUNDING,
            "declining leaves the balance alone"
        );
        assert_eq!(
            client.nonce(&sender).await.unwrap(),
            0,
            "declining sends nothing, so the nonce does not move"
        );

        // 4. An approved transfer is built from metadata, signed in the vault,
        //    submitted and finalised.
        let approving = Scripted::approving();
        let receipt = client
            .transfer(&vault, &approving, &sender, &recipient, SENDING)
            .await
            .unwrap();

        assert_eq!(approving.times_asked(), 1);
        assert!(receipt.tx_hash.starts_with("0x"));
        assert!(
            receipt.block_hash.starts_with("0x"),
            "the receipt names the finalised block"
        );
        assert_eq!(receipt.amount_sparks, SENDING.to_string());
        assert_eq!(receipt.from, sender);
        assert_eq!(receipt.to, recipient);

        // The chain has no transaction payment yet (OPEN-4), so the arithmetic
        // is exact. When fees arrive this assertion is the thing that says so.
        assert_eq!(client.balance(&sender).await.unwrap(), FUNDING - SENDING);
        assert_eq!(client.balance(&recipient).await.unwrap(), SENDING);
        assert_eq!(client.nonce(&sender).await.unwrap(), 1);

        // 5. Two in a row: the nonce comes from the pool, not from the last
        //    finalised block (ADR-040, decision 5). Before that fix the second
        //    of these was refused as "Transaction is outdated".
        let again = Scripted::approving();
        for _ in 0..2 {
            client
                .transfer(&vault, &again, &sender, &recipient, SENDING)
                .await
                .expect("back-to-back transfers");
        }
        assert_eq!(client.nonce(&sender).await.unwrap(), 3);

        // 6. What still has no answer says so, against a live node too.
        assert!(client.history(&sender, 10).await.is_err());
        assert!(client
            .claim_starter(&vault, &approving, &sender)
            .await
            .is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// M4.1, end to end: a mint built from metadata, approved in the dialog,
    /// signed in the vault, finalised by GRANDPA, and then found by reading the
    /// owner's enumeration from chain storage; then made permanent the same way.
    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_mint_finalises_and_appears_in_the_owners_enumeration() {
        use crate::chain::assets::MintRequest;
        use crate::content::{CommitId, ContentRef, Manifest, SourceRef, TemporaryStore};

        // A creator's first mint holds about 1,100 CGT at the placeholder
        // deposits (ADR-052); this leaves room for the existential deposit.
        const MINT_FUNDING: u128 = 5_000 * crate::cgt::SPARKS_PER_CGT;

        let endpoint = endpoint();
        let client = ChainClient::new(&endpoint).unwrap();
        let status = client.status().await;
        assert!(
            status.reachable,
            "no node at {endpoint}: {:?}",
            status.detail
        );

        let dir = std::env::temp_dir().join(format!(
            "qor-mint-live-{}-{}",
            std::process::id(),
            status.block_number.unwrap_or_default()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let vault = crate::vault::testing::vault_in(&dir);
        let phrase = crate::vault::derive::generate_mnemonic().unwrap();
        let creator = vault.create(&phrase).unwrap()[0].address.clone();
        fund_from_alice(
            &endpoint,
            normalise_address(&creator).unwrap(),
            MINT_FUNDING,
        )
        .await;

        // What a Qontrol project's commit makes, built as the Projects surface
        // builds it.
        let bytes = b"RIFF the first take".to_vec();
        let manifest = Manifest::new(
            vec![("stems/take.wav".into(), ContentRef::of(&bytes))],
            1_758_500_000,
            Some(SourceRef {
                commit: CommitId::Sha1([0x5a; 20]),
                branch: Some("main".into()),
            }),
        )
        .unwrap();
        let request = MintRequest {
            name: "live-mint".into(),
            reference: manifest.reference(),
            commit: CommitId::Sha1([0x5a; 20]),
            branch: Some("main".into()),
            files: 1,
        };
        let store = TemporaryStore::in_data_dir(&dir);

        // 1. A declined mint moves nothing, sends nothing and writes nothing.
        let declining = Scripted::declining();
        let mut wrote = false;
        let declined = client
            .mint(&vault, &declining, &creator, &request, || {
                wrote = true;
                Ok(())
            })
            .await
            .unwrap_err();
        assert_eq!(declined.kind(), "declined");
        assert_eq!(declining.times_asked(), 1);
        assert!(!wrote, "a declined mint writes nothing to the store");
        assert_eq!(client.balance(&creator).await.unwrap(), MINT_FUNDING);
        assert_eq!(client.nonce(&creator).await.unwrap(), 0);
        assert!(client.assets_of(&creator).await.unwrap().is_empty());

        // 2. An approved mint finalises, and its bytes reach the store.
        let approving = Scripted::approving();
        let receipt = client
            .mint(&vault, &approving, &creator, &request, || {
                store.put(&manifest.reference(), &manifest.bytes())?;
                store.put(&ContentRef::of(&bytes), &bytes)?;
                Ok(())
            })
            .await
            .unwrap();
        assert_eq!(approving.times_asked(), 1);
        assert!(receipt.block_hash.starts_with("0x"), "a finalised block");
        assert!(approving.asked.lock()[0].body.contains("live-mint"));
        assert!(store.path_of(&manifest.reference()).is_file());
        assert!(
            client.balance(&creator).await.unwrap() < MINT_FUNDING,
            "the deposits are held from the creator's free balance"
        );

        // 3. It is in the owner's enumeration, read from chain storage.
        let held = client.assets_of(&creator).await.unwrap();
        assert_eq!(held.len(), 1, "{held:?}");
        let asset = &held[0];
        assert_eq!(
            (asset.collection, asset.item),
            (receipt.collection, receipt.item)
        );
        assert_eq!(asset.name, "live-mint");
        assert_eq!(asset.origin.root, request.reference.root_hex());
        assert_eq!(asset.origin.size, request.reference.size);
        assert_eq!(asset.origin.algo, "BLAKE3-256");
        assert_eq!(asset.current, asset.origin);
        assert_eq!(asset.commit.as_ref().unwrap().id, "5a".repeat(20));
        assert!(asset.revisable);

        // 4. Made permanent through the same dialog and vault, and it says so.
        let permanent = Scripted::approving();
        client
            .make_permanent(&vault, &permanent, &creator, asset.collection, asset.item)
            .await
            .unwrap();
        assert!(permanent.asked.lock()[0].body.contains("cannot be undone"));
        let after = client.assets_of(&creator).await.unwrap();
        assert!(!after[0].revisable, "it is permanent now");

        // 5. And a second attempt is refused before anyone is asked.
        let again = Scripted::approving();
        assert!(client
            .make_permanent(&vault, &again, &creator, asset.collection, asset.item)
            .await
            .is_err());
        assert_eq!(again.times_asked(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// L4.5, end to end against a real node: three assets minted, two of them
    /// traded away in one transaction with a message, and the chain asked
    /// afterwards who holds what.
    ///
    /// Atomicity is proven where it lives, in `chain/runtime/tests/assets.rs`:
    /// a batch whose second call fails moves nothing. What is proven here is
    /// the launcher's half — the batch is built from the node's own metadata,
    /// the dialog says what leaves, a declined trade moves nothing, and an
    /// asset this account no longer holds is refused before anyone is asked.
    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_trade_moves_the_assets_and_the_chain_says_so() {
        use crate::chain::assets::{MintRequest, TradeItem};
        use crate::content::{CommitId, ContentRef, Manifest, SourceRef, TemporaryStore};

        // Three mints at the placeholder deposits: about 1,002.5 CGT for the
        // first and 502.5 for each of the others (ADR-052), plus room to spare.
        const TRADE_FUNDING: u128 = 8_000 * crate::cgt::SPARKS_PER_CGT;

        let endpoint = endpoint();
        let client = ChainClient::new(&endpoint).unwrap();
        let status = client.status().await;
        assert!(
            status.reachable,
            "no node at {endpoint}: {:?}",
            status.detail
        );

        let dir = std::env::temp_dir().join(format!(
            "qor-trade-live-{}-{}",
            std::process::id(),
            status.block_number.unwrap_or_default()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let vault = crate::vault::testing::vault_in(&dir);
        let phrase = crate::vault::derive::generate_mnemonic().unwrap();
        let creator = vault.create(&phrase).unwrap()[0].address.clone();
        let recipient = vault.add_account("the other side").unwrap().address;
        fund_from_alice(
            &endpoint,
            normalise_address(&creator).unwrap(),
            TRADE_FUNDING,
        )
        .await;
        fund_from_alice(
            &endpoint,
            normalise_address(&recipient).unwrap(),
            200 * crate::cgt::SPARKS_PER_CGT,
        )
        .await;

        let store = TemporaryStore::in_data_dir(&dir);
        let mut minted = Vec::new();
        for (i, name) in ["one", "two", "three"].iter().enumerate() {
            let bytes = format!("take {i}").into_bytes();
            let manifest = Manifest::new(
                vec![(format!("stems/{name}.wav"), ContentRef::of(&bytes))],
                1_758_500_000 + i as u64,
                Some(SourceRef {
                    commit: CommitId::Sha1([0x5b + i as u8; 20]),
                    branch: Some("main".into()),
                }),
            )
            .unwrap();
            let request = MintRequest {
                name: (*name).into(),
                reference: manifest.reference(),
                commit: CommitId::Sha1([0x5b + i as u8; 20]),
                branch: Some("main".into()),
                files: 1,
            };
            let receipt = client
                .mint(&vault, &Scripted::approving(), &creator, &request, || {
                    store.put(&manifest.reference(), &manifest.bytes())?;
                    Ok(())
                })
                .await
                .unwrap();
            minted.push(TradeItem {
                collection: receipt.collection,
                item: receipt.item,
            });
        }
        assert_eq!(client.assets_of(&creator).await.unwrap().len(), 3);

        // 1. A declined trade moves nothing.
        let declining = Scripted::declining();
        let declined = client
            .trade(
                &vault,
                &declining,
                &creator,
                &recipient,
                &minted[..2],
                Some("for the album"),
            )
            .await
            .unwrap_err();
        assert_eq!(declined.kind(), "declined");
        assert_eq!(declining.times_asked(), 1);
        assert!(declining.asked.lock()[0].body.contains("cannot be undone"));
        assert!(declining.asked.lock()[0].body.contains(&recipient));
        assert!(declining.asked.lock()[0].body.contains("for the album"));
        assert_eq!(client.assets_of(&creator).await.unwrap().len(), 3);
        assert!(client.assets_of(&recipient).await.unwrap().is_empty());

        // 2. An approved trade moves exactly what it named, in one transaction.
        let approving = Scripted::approving();
        let receipt = client
            .trade(
                &vault,
                &approving,
                &creator,
                &recipient,
                &minted[..2],
                Some("for the album"),
            )
            .await
            .unwrap();
        assert_eq!(approving.times_asked(), 1);
        assert_eq!(receipt.moved.len(), 2);
        assert!(receipt.block_hash.starts_with("0x"), "a finalised block");

        let theirs = client.assets_of(&recipient).await.unwrap();
        let mine = client.assets_of(&creator).await.unwrap();
        assert_eq!(theirs.len(), 2, "{theirs:?}");
        assert_eq!(mine.len(), 1, "{mine:?}");
        assert_eq!(mine[0].name, "three");
        let mut names: Vec<&str> = theirs.iter().map(|a| a.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(names, ["one", "two"]);

        // 3. What has gone is gone: trading it again is refused before anyone
        // is asked, and the asset still held does not move with it.
        let again = Scripted::approving();
        let refused = client
            .trade(
                &vault,
                &again,
                &creator,
                &recipient,
                &[minted[0], minted[2]],
                None,
            )
            .await
            .unwrap_err();
        assert!(
            refused.to_string().contains("holds no DRC-369 asset"),
            "{refused}"
        );
        assert_eq!(again.times_asked(), 0);
        assert_eq!(client.assets_of(&creator).await.unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// L4.6, end to end against a real node: a remix listed, bought and paid
    /// out — the remix share to its source's recipients, the royalty to its
    /// own, the rest to the seller — and the chain asked afterwards who holds
    /// what, to the Spark.
    ///
    /// The split's arithmetic is proven where it lives, in
    /// `chain/pallets/drc369-royalties`. What is proven here is the launcher's
    /// half: every call is built from the node's own metadata; a declined
    /// listing, purchase or withdrawal moves nothing; what the dialog says a
    /// sale will pay is what the chain's `Sold` event then reports, part for
    /// part; and everything the chain would refuse is refused before anyone is
    /// asked, in words.
    ///
    /// The amounts are written out here rather than computed, so this does not
    /// pass by agreeing with itself.
    #[tokio::test]
    #[ignore = "needs a running Demiurge development node; see the module docs"]
    async fn a_sale_pays_every_part_and_hands_the_asset_over() {
        use crate::cgt::SPARKS_PER_CGT as CGT;
        use crate::chain::assets::{CommitIdArg, ContentRefArg, MintRequest, TradeItem};
        use crate::chain::sales::{Payout, PayoutKind, Seen};
        use crate::content::{CommitId, ContentRef, Manifest, SourceRef, TemporaryStore};
        use crate::Prompt;

        // A mint holds about 1,100 CGT at the placeholder deposits (ADR-052).
        const FUNDING: u128 = 5_000 * CGT;
        // Seven Sparks over, so the rounding has somewhere to go.
        const PRICE: u128 = 2_000 * CGT + 7;

        #[derive(Debug, DecodeAsType)]
        struct Minted {
            collection: u32,
            item: u32,
        }

        let endpoint = endpoint();
        let client = ChainClient::new(&endpoint).unwrap();
        let status = client.status().await;
        assert!(
            status.reachable,
            "no node at {endpoint}: {:?}",
            status.detail
        );

        let dir = std::env::temp_dir().join(format!(
            "qor-sale-live-{}-{}",
            std::process::id(),
            status.block_number.unwrap_or_default()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let vault = crate::vault::testing::vault_in(&dir);
        let phrase = crate::vault::derive::generate_mnemonic().unwrap();
        let creator = vault.create(&phrase).unwrap()[0].address.clone();
        let remixer = vault.add_account("the remixer").unwrap().address;
        let buyer = vault.add_account("the buyer").unwrap().address;
        // Named in both sets of terms, never funded: the sale opens the account.
        let collaborator = vault.add_account("a collaborator").unwrap().address;
        for account in [&creator, &remixer, &buyer] {
            fund_from_alice(&endpoint, normalise_address(account).unwrap(), FUNDING).await;
        }
        let account = |address: &str| AccountId32(normalise_address(address).unwrap());
        let silent = || Prompt {
            title: String::new(),
            body: String::new(),
            approve: String::new(),
        };

        // ── The original, with terms, and a remix of it with terms of its own.
        let store = TemporaryStore::in_data_dir(&dir);
        let bytes = b"the original take".to_vec();
        let manifest = Manifest::new(
            vec![("stems/original.wav".into(), ContentRef::of(&bytes))],
            1_758_900_000,
            Some(SourceRef {
                commit: CommitId::Sha1([0x61; 20]),
                branch: Some("main".into()),
            }),
        )
        .unwrap();
        let original = client
            .mint(
                &vault,
                &Scripted::approving(),
                &creator,
                &MintRequest {
                    name: "the-original".into(),
                    reference: manifest.reference(),
                    commit: CommitId::Sha1([0x61; 20]),
                    branch: Some("main".into()),
                    files: 1,
                },
                || {
                    store.put(&manifest.reference(), &manifest.bytes())?;
                    Ok(())
                },
            )
            .await
            .unwrap();

        let connection = client.connected().await.unwrap();
        // The launcher has no surface for setting terms or minting a remix yet,
        // so the test sends those calls itself, through the same signing path
        // as everything else.
        let terms =
            |collection: u32, item: u32, recipients: Vec<(AccountId32, u32)>, remix: u32| {
                dynamic::transaction(
                    "Drc369Royalties",
                    "set_terms",
                    (collection, item, recipients, remix),
                )
            };
        // A sale of any remix owes this one's recipients 20%, divided one to
        // three between the creator (10%) and the collaborator (30%).
        client
            .sign_and_finalise(
                &connection,
                &vault,
                &Scripted::approving(),
                &creator,
                &terms(
                    original.collection,
                    original.item,
                    vec![
                        (account(&creator), 100_000),
                        (account(&collaborator), 300_000),
                    ],
                    200_000,
                ),
                |_, _| silent(),
                || Ok(()),
                "terms",
            )
            .await
            .map(|_| ())
            .expect("the original's terms");

        let remix_reference = ContentRef::of(b"the remix's manifest");
        let minted = client
            .sign_and_finalise(
                &connection,
                &vault,
                &Scripted::approving(),
                &remixer,
                &dynamic::transaction(
                    "Drc369",
                    "mint",
                    (
                        ContentRefArg::from(&remix_reference),
                        Some(CommitIdArg::from(&CommitId::Sha1([0x62; 20]))),
                        b"the-remix".to_vec(),
                        true,
                        Some((original.collection, original.item)),
                    ),
                ),
                |_, _| silent(),
                || Ok(()),
                "mint",
            )
            .await
            .map(|finalised| finalised.events)
            .expect("the remix");
        let remix = minted
            .iter()
            .filter_map(Result::ok)
            .find(|event| event.pallet_name() == "Drc369" && event.event_name() == "Minted")
            .expect("a Minted event")
            .decode_fields_unchecked_as::<Minted>()
            .unwrap();
        let (collection, item) = (remix.collection, remix.item);
        let number = format!("{collection}/{item}");
        // The remix's own terms: the collaborator again, 2.5% of every sale.
        client
            .sign_and_finalise(
                &connection,
                &vault,
                &Scripted::approving(),
                &remixer,
                &terms(collection, item, vec![(account(&collaborator), 25_000)], 0),
                |_, _| silent(),
                || Ok(()),
                "terms",
            )
            .await
            .map(|_| ())
            .expect("the remix's terms");

        // 1. Looked up by its number, by someone who does not hold it: what it
        //    is, who holds it, its terms and its source's, and not for sale.
        let found = client.sale(&number, Some(&buyer)).await.unwrap();
        assert_eq!(found.asset.name, "the-remix");
        assert_eq!(found.holder, remixer);
        assert!(!found.held_by_viewer);
        assert_eq!(
            found.derived_from,
            Some(TradeItem {
                collection: original.collection,
                item: original.item
            })
        );
        let own = found.terms.as_ref().expect("the remix's terms");
        assert_eq!(own.recipients.len(), 1);
        assert_eq!(own.recipients[0].address, collaborator);
        assert_eq!(own.recipients[0].share, "2.5%");
        let upstream = found.source_terms.as_ref().expect("the original's terms");
        assert_eq!(upstream.remix, "20%");
        assert_eq!(upstream.recipients[0].address, creator);
        assert_eq!(upstream.recipients[1].share, "30%");
        assert!(found.asset.listing.is_none());
        assert!(found.breakdown.is_none());
        assert!(found.cannot_buy.as_deref().unwrap().contains("not listed"));
        // The line an asset's menu copies is read as that asset, and its
        // fingerprint is checked against the chain's.
        let pasted = format!(
            "{} {} (asset {number})",
            found.asset.current.algo, found.asset.current.root
        );
        assert_eq!(
            client
                .sale(&pasted, None)
                .await
                .unwrap()
                .pasted_root_matches,
            Some(true)
        );
        assert!(client
            .sale("4000000/0", None)
            .await
            .unwrap_err()
            .to_string()
            .contains("no DRC-369 asset"));
        let root = found.asset.current.root.clone();
        fn seen(price_sparks: u128, root: &str) -> Seen<'_> {
            Seen { price_sparks, root }
        }

        // 2. What cannot happen is refused before anyone is asked.
        let nobody = Scripted::approving();
        assert!(client
            .buy(
                &vault,
                &nobody,
                &buyer,
                collection,
                item,
                seen(PRICE, &root)
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("not listed"));
        assert!(client
            .unlist(&vault, &nobody, &remixer, collection, item)
            .await
            .unwrap_err()
            .to_string()
            .contains("nothing to withdraw"));
        assert!(client
            .list(&vault, &nobody, &buyer, collection, item, PRICE)
            .await
            .unwrap_err()
            .to_string()
            .contains("holds no DRC-369 asset"));
        // At 200 CGT the source's pool is 40, and the creator's 10 of it is
        // fine, but the collaborator's 30 cannot open an account that does not
        // exist: 100 CGT is the least one can hold (ADR-036). The chain would
        // refuse every sale at that price.
        let too_low = client
            .list(&vault, &nobody, &remixer, collection, item, 200 * CGT)
            .await
            .unwrap_err()
            .to_string();
        assert!(too_low.contains(&collaborator), "{too_low}");
        assert!(too_low.contains("30.00 CGT"), "{too_low}");
        assert_eq!(nobody.times_asked(), 0);

        // 3. A declined listing publishes nothing and sends nothing.
        let nonce = client.nonce(&remixer).await.unwrap();
        let declining = Scripted::declining();
        let declined = client
            .list(&vault, &declining, &remixer, collection, item, 2 * PRICE)
            .await
            .unwrap_err();
        assert_eq!(declined.kind(), "declined");
        assert_eq!(declining.times_asked(), 1);
        assert_eq!(client.nonce(&remixer).await.unwrap(), nonce);
        assert!(client.assets_of(&remixer).await.unwrap()[0]
            .listing
            .is_none());

        // 4. An approved listing is on chain, and the Inventory's own read of
        //    the asset carries it.
        let approving = Scripted::approving();
        let listed = client
            .list(&vault, &approving, &remixer, collection, item, 2 * PRICE)
            .await
            .unwrap();
        assert_eq!(approving.times_asked(), 1);
        assert_eq!(listed.price_sparks, (2 * PRICE).to_string());
        assert!(listed.block_hash.starts_with("0x"), "a finalised block");
        {
            let asked = approving.asked.lock();
            assert_eq!(asked[0].approve, "List for sale");
            assert!(asked[0].body.contains("the-remix"));
            assert!(asked[0].body.contains(&collaborator));
            assert!(asked[0].body.contains("The listing is public"));
        }
        let held = client.assets_of(&remixer).await.unwrap();
        let listing = held[0].listing.as_ref().expect("a listing");
        assert_eq!(listing.seller, remixer);
        assert_eq!(listing.price_sparks, (2 * PRICE).to_string());
        assert!(!listing.void);

        // The same price again is refused without asking; a new price is a
        // change, and the dialog says what it was.
        let again = Scripted::approving();
        assert!(client
            .list(&vault, &again, &remixer, collection, item, 2 * PRICE)
            .await
            .unwrap_err()
            .to_string()
            .contains("already listed"));
        assert_eq!(again.times_asked(), 0);
        client
            .list(&vault, &again, &remixer, collection, item, PRICE)
            .await
            .unwrap();
        assert_eq!(again.asked.lock()[0].approve, "Change the price");

        // 5. The buyer looked while it cost twice as much. Nobody is asked to
        //    approve a price they did not see, and the seller cannot buy.
        let stale = Scripted::approving();
        assert!(client
            .buy(
                &vault,
                &stale,
                &buyer,
                collection,
                item,
                seen(2 * PRICE, &root)
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("The price changed since you looked"));
        assert!(client
            .buy(
                &vault,
                &stale,
                &buyer,
                collection,
                item,
                seen(PRICE, &"0".repeat(64))
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("revised since you looked"));
        assert!(client
            .buy(
                &vault,
                &stale,
                &remixer,
                collection,
                item,
                seen(PRICE, &root)
            )
            .await
            .unwrap_err()
            .to_string()
            .contains("your own listing"));
        assert_eq!(stale.times_asked(), 0);

        // 6. What the launcher says the sale will pay, before anyone signs.
        //    20% of the price is 400 CGT and one Spark; a quarter of that,
        //    rounded down, is the creator's 100 CGT, and three quarters the
        //    collaborator's 300 CGT. Of the 1,600 CGT and seven Sparks left,
        //    2.5% is the collaborator's 40 CGT. The seller has the rest.
        let expected: Vec<(PayoutKind, String, u128)> = vec![
            (PayoutKind::Source, creator.clone(), 100 * CGT),
            (PayoutKind::Source, collaborator.clone(), 300 * CGT),
            (PayoutKind::Royalty, collaborator.clone(), 40 * CGT),
            (PayoutKind::Seller, remixer.clone(), 1_560 * CGT + 7),
        ];
        let parts = |payouts: &[Payout]| -> Vec<(PayoutKind, String, u128)> {
            payouts
                .iter()
                .map(|p| (p.kind, p.address.clone(), p.amount_sparks.parse().unwrap()))
                .collect()
        };
        let looked = client.sale(&number, Some(&buyer)).await.unwrap();
        assert_eq!(looked.cannot_buy, None);
        let foretold = looked.breakdown.as_ref().expect("a live listing");
        assert_eq!(foretold.price_sparks, PRICE.to_string());
        assert_eq!(parts(&foretold.payouts), expected);
        assert_eq!(foretold.blocked, None);
        assert_eq!(
            parts(
                &client
                    .sale_preview(collection, item, PRICE)
                    .await
                    .unwrap()
                    .payouts
            ),
            expected
        );

        // 7. A declined purchase moves nothing: no CGT, no asset, no nonce.
        let before = [
            client.balance(&creator).await.unwrap(),
            client.balance(&remixer).await.unwrap(),
            client.balance(&buyer).await.unwrap(),
            client.balance(&collaborator).await.unwrap(),
        ];
        assert_eq!(before[3], 0, "the collaborator's account does not exist");
        let buyer_nonce = client.nonce(&buyer).await.unwrap();
        let declining = Scripted::declining();
        let declined = client
            .buy(
                &vault,
                &declining,
                &buyer,
                collection,
                item,
                seen(PRICE, &root),
            )
            .await
            .unwrap_err();
        assert_eq!(declined.kind(), "declined");
        assert_eq!(declining.times_asked(), 1);
        {
            let asked = declining.asked.lock();
            assert!(asked[0].body.contains("cannot be undone"));
            assert!(asked[0].body.contains("You will not pay more than"));
            assert!(asked[0].body.contains(&remixer));
        }
        assert_eq!(client.balance(&buyer).await.unwrap(), before[2]);
        assert_eq!(client.balance(&remixer).await.unwrap(), before[1]);
        assert_eq!(client.balance(&creator).await.unwrap(), before[0]);
        assert_eq!(client.balance(&collaborator).await.unwrap(), 0);
        assert_eq!(client.nonce(&buyer).await.unwrap(), buyer_nonce);
        assert!(client.assets_of(&buyer).await.unwrap().is_empty());
        assert_eq!(client.assets_of(&remixer).await.unwrap().len(), 1);

        // 8. An approved purchase settles: the chain's own Sold event names
        //    every part, and it is what the launcher foretold.
        let approving = Scripted::approving();
        let receipt = client
            .buy(
                &vault,
                &approving,
                &buyer,
                collection,
                item,
                seen(PRICE, &root),
            )
            .await
            .unwrap();
        assert_eq!(approving.times_asked(), 1);
        assert!(receipt.block_hash.starts_with("0x"), "a finalised block");
        assert_eq!(receipt.price_sparks, PRICE.to_string());
        assert_eq!(receipt.seller, remixer);
        assert_eq!(receipt.buyer, buyer);
        assert_eq!(parts(&receipt.payouts), expected);

        // The balances, to the Spark. There is no transaction payment yet
        // (OPEN-4); when there is, the buyer's line is the one that says so.
        assert_eq!(
            client.balance(&creator).await.unwrap(),
            before[0] + 100 * CGT
        );
        assert_eq!(
            client.balance(&remixer).await.unwrap(),
            before[1] + 1_560 * CGT + 7
        );
        assert_eq!(client.balance(&buyer).await.unwrap(), before[2] - PRICE);
        assert_eq!(
            client.balance(&collaborator).await.unwrap(),
            340 * CGT,
            "the sale opened the collaborator's account"
        );

        // And the asset: the buyer holds it, the seller does not, and the
        // listing went with the sale.
        let theirs = client.assets_of(&buyer).await.unwrap();
        assert_eq!(theirs.len(), 1, "{theirs:?}");
        assert_eq!((theirs[0].collection, theirs[0].item), (collection, item));
        assert!(theirs[0].listing.is_none(), "a sale clears its listing");
        assert!(client.assets_of(&remixer).await.unwrap().is_empty());
        let after = client.sale(&number, Some(&buyer)).await.unwrap();
        assert!(after.held_by_viewer);
        assert!(after.cannot_buy.as_deref().unwrap().contains("not listed"));

        // 9. Withdrawing: declined changes nothing, approved removes it.
        client
            .list(
                &vault,
                &Scripted::approving(),
                &buyer,
                collection,
                item,
                900 * CGT,
            )
            .await
            .unwrap();
        let declining = Scripted::declining();
        assert_eq!(
            client
                .unlist(&vault, &declining, &buyer, collection, item)
                .await
                .unwrap_err()
                .kind(),
            "declined"
        );
        assert!(declining.asked.lock()[0].body.contains("900.00 CGT"));
        assert!(client.assets_of(&buyer).await.unwrap()[0].listing.is_some());
        // Someone who neither listed it nor holds it cannot withdraw it.
        let stranger = Scripted::approving();
        assert!(client
            .unlist(&vault, &stranger, &creator, collection, item)
            .await
            .unwrap_err()
            .to_string()
            .contains("Only the account that holds this asset"));
        assert_eq!(stranger.times_asked(), 0);
        client
            .unlist(&vault, &Scripted::approving(), &buyer, collection, item)
            .await
            .unwrap();
        assert!(client.assets_of(&buyer).await.unwrap()[0].listing.is_none());

        // 10. A listing is void once its seller no longer holds the asset: the
        //     new holder's card says so, nobody can buy from it, and anyone may
        //     clear it.
        client
            .list(
                &vault,
                &Scripted::approving(),
                &buyer,
                collection,
                item,
                900 * CGT,
            )
            .await
            .unwrap();
        client
            .trade(
                &vault,
                &Scripted::approving(),
                &buyer,
                &remixer,
                &[TradeItem { collection, item }],
                None,
            )
            .await
            .unwrap();
        let back = client.assets_of(&remixer).await.unwrap();
        assert!(back[0].listing.as_ref().expect("the old listing").void);
        let void = client.sale(&number, Some(&creator)).await.unwrap();
        assert!(void.breakdown.is_none());
        assert!(void.cannot_buy.as_deref().unwrap().contains("void"));
        let clearing = Scripted::approving();
        client
            .unlist(&vault, &clearing, &creator, collection, item)
            .await
            .unwrap();
        assert!(clearing.asked.lock()[0]
            .body
            .contains("This listing is void"));
        assert!(client.assets_of(&remixer).await.unwrap()[0]
            .listing
            .is_none());

        // 11. The chain's own refusal, reached by going round the launcher's
        //     checks: a purchase that names less than the price as the most it
        //     will pay, which is what a price raised in between looks like to
        //     the chain. Its refusal is put in words, and nothing moved.
        client
            .list(
                &vault,
                &Scripted::approving(),
                &remixer,
                collection,
                item,
                900 * CGT,
            )
            .await
            .unwrap();
        let buyer_before = client.balance(&buyer).await.unwrap();
        let refused = client
            .sign_and_finalise(
                &connection,
                &vault,
                &Scripted::approving(),
                &buyer,
                &dynamic::transaction("Drc369Royalties", "buy", (collection, item, 899 * CGT)),
                |_, _| silent(),
                || Ok(()),
                "purchase",
            )
            .await
            .map(|_| ())
            .unwrap_err();
        assert!(
            refused.to_string().contains("PriceAboveLimit"),
            "the chain names its own error: {refused}"
        );
        let words = crate::chain::sales::in_words(refused).to_string();
        assert!(words.contains("price was raised"), "{words}");
        assert!(words.contains("the purchase 0x"), "{words}");
        assert_eq!(client.balance(&buyer).await.unwrap(), buyer_before);
        assert_eq!(client.assets_of(&remixer).await.unwrap().len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
