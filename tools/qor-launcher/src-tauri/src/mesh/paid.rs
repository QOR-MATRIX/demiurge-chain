//! The Mesh paid for its work: the launcher's half of L10.2 to L10.4 (ADR-096, ADR-097).
//!
//! While seeding is on and the person gave the Mesh grant, a worker runs every few seconds and:
//!
//! 1. **binds** this launcher's node to the account, once;
//! 2. **answers** each period's storage challenge for every file it holds an assignment on: the group the chain names,
//!    and its proof, made a piece at a time from the store with `mesh_proof`, the code the chain checks it with;
//! 3. **claims** payment for serving with the receipts receivers sent it;
//! 4. **takes on** others' published work: a copy an escrow still asks for, of a file that fits the disk limit, fetched
//!    and verified before the assignment is taken.
//!
//! **Finding who holds a root** (L10.2) reads the chain: an escrow's assigned seeders, then its uploader, each dialed by
//! the node it bound. A receiver that fetched from an assigned seeder signs a receipt for the bytes received from it in
//! the period so far and sends it over [`RECEIPT_ALPN`]; the seeder checks it and keeps it until the worker claims it.
//!
//! Nothing here logs a receipt or a signature (AGENTS.md §9).

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Weak};
use std::time::Duration;

use codec::{Decode, Encode};
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{EndpointAddr, EndpointId};
use iroh_blobs::api::proto::BlobStatus;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::Hash;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::{Fetched, Mesh};
use crate::chain::mesh::{receipt_message, Escrow, Granted, Receipt};
use crate::chain::ChainClient;
use crate::error::{QorError, QorResult};
use crate::vault::{derive, MeshGrant, Vault};
use crate::Confirm;

/// The Mesh's own protocol for a receipt, receiver to seeder (ADR-097 decision 5).
pub const RECEIPT_ALPN: &[u8] = b"demiurge/mesh-receipt/1";

/// How often the worker looks at the chain. Answer windows are a hundred blocks on the devnet and five on a development
/// chain, at six seconds a block.
const TICK: Duration = Duration::from_secs(3);
/// How long the worker waits after a round that failed.
const BACK_OFF: Duration = Duration::from_secs(60);
/// Every this many ticks the worker looks for work to take on and drops assignments the chain has ended.
const LOOK_EVERY: u64 = 10;
/// How much of a file is read at a time while a proof is made.
const PIECE: usize = 64 * 1024;
/// The most receipts one claim carries: the runtime's `MaxReceipts`.
const BATCH: usize = 64;
/// How long a peer has to answer.
const DIAL: Duration = Duration::from_secs(20);
/// The payments the Mesh page lists.
const PAYMENTS_KEPT: usize = 50;

/// The vault and the chain, which paid work needs and L10.1's node does not.
pub struct Paid {
    pub vault: Arc<Vault>,
    pub chain: Arc<ChainClient>,
}

/// What the Mesh keeps about its paid work, in its folder.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Ledger {
    /// The account the person gave the Mesh grant for, hex.
    pub grant: Option<String>,
    /// The node bound on chain for it, hex.
    pub bound: Option<String>,
    pub holdings: BTreeMap<u64, Holding>,
    /// Receipts receivers sent, kept until claimed: SCALE, hex.
    pub pending: Vec<Pending>,
    /// What this launcher received from each seeder, per escrow and period, as its receipts said.
    pub received: Vec<Received>,
    pub payments: Vec<Payment>,
    /// Everything paid to the account for the Mesh's work, embers.
    pub paid_embers: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Holding {
    pub root: String,
    pub size: u64,
    pub taken: u32,
    pub answered: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Pending {
    receipt: String,
    signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Received {
    escrow: u64,
    period: u32,
    seeder: String,
    bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    /// `storage` or `serving`.
    pub kind: String,
    pub escrow: Option<u64>,
    pub paid_embers: String,
    pub paid_cgt: String,
    pub block_hash: String,
}

/// What the Mesh page shows of paid work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaidStatus {
    /// The account the Mesh signs for; none until the person allows it.
    pub account: Option<String>,
    pub holdings: Vec<HoldingView>,
    pub pending_receipts: usize,
    pub paid_embers: String,
    pub paid_cgt: String,
    pub payments: Vec<Payment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldingView {
    pub escrow: u64,
    pub root: String,
    pub size: u64,
    pub answered: Option<u32>,
}

impl Ledger {
    pub(crate) fn read(dir: &Path) -> Self {
        std::fs::read(dir.join("paid.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    fn write(&self, dir: &Path) -> QorResult<()> {
        std::fs::create_dir_all(dir).map_err(|e| QorError::Io(e.to_string()))?;
        let bytes =
            serde_json::to_vec_pretty(self).map_err(|e| QorError::Internal(e.to_string()))?;
        std::fs::write(dir.join("paid.json"), bytes).map_err(|e| QorError::Io(e.to_string()))
    }

    fn grant(&self) -> Option<MeshGrant> {
        self.grant
            .as_deref()
            .and_then(account_of)
            .map(MeshGrant::recorded)
    }

    fn paid(&self) -> u128 {
        self.paid_embers.parse().unwrap_or(0)
    }

    fn record(&mut self, kind: &str, escrow: Option<u64>, paid: u128, block_hash: String) {
        if paid == 0 {
            return;
        }
        self.paid_embers = self.paid().saturating_add(paid).to_string();
        self.payments.push(Payment {
            kind: kind.into(),
            escrow,
            paid_embers: paid.to_string(),
            paid_cgt: crate::cgt::format_cgt(paid),
            block_hash,
        });
        if self.payments.len() > PAYMENTS_KEPT {
            self.payments.remove(0);
        }
    }

    fn pending(&self) -> Vec<(Receipt, [u8; 64])> {
        self.pending
            .iter()
            .filter_map(|p| {
                let receipt = Receipt::decode(&mut &hex::decode(&p.receipt).ok()?[..]).ok()?;
                let signature =
                    <[u8; 64]>::try_from(hex::decode(&p.signature).ok()?.as_slice()).ok()?;
                Some((receipt, signature))
            })
            .collect()
    }

    fn set_pending(&mut self, receipts: &[(Receipt, [u8; 64])]) {
        self.pending = receipts
            .iter()
            .map(|(r, s)| Pending {
                receipt: hex::encode(r.encode()),
                signature: hex::encode(s),
            })
            .collect();
    }
}

fn account_of(hex_account: &str) -> Option<[u8; 32]> {
    <[u8; 32]>::try_from(hex::decode(hex_account).ok()?.as_slice()).ok()
}

impl Mesh {
    /// Give the Mesh what paid work needs. Without it the Mesh is L10.1's: serving and fetching, paid nothing.
    pub fn connect(&self, vault: Arc<Vault>, chain: Arc<ChainClient>) {
        let _ = self.paid.set(Paid { vault, chain });
    }

    /// The Mesh grant, if the person gave it.
    pub fn grant(&self) -> Option<MeshGrant> {
        self.ledger.lock().grant()
    }

    /// Ask the person, once, to let the Mesh sign its own work for the vault's active account (ADR-097), and start the
    /// worker if seeding is on. Declining leaves seeding as it was, unpaid.
    pub async fn allow_paid(self: &Arc<Self>, confirm: &dyn Confirm) -> QorResult<PaidStatus> {
        let paid = self.paid()?;
        let address = paid.vault.active_address()?;
        let chain_name = paid
            .chain
            .status()
            .await
            .chain_name
            .unwrap_or_else(|| paid.chain.endpoint());
        // Nobody is asked to allow what the chain cannot pay for: the devnet gains the Mesh by a runtime upgrade.
        if paid.chain.mesh_period().await.is_err() {
            return Err(QorError::Mesh(format!(
                "{chain_name} does not pay for the Mesh yet, so there is nothing to allow. Seeding still serves your \
                 work."
            )));
        }
        let grant = paid
            .vault
            .grant_mesh(confirm, &address, &chain_name)
            .await?;
        {
            let mut ledger = self.ledger.lock();
            ledger.grant = Some(hex::encode(grant.account()));
            ledger.write(&self.dir)?;
        }
        if self.settings().seeding {
            self.start_worker();
        }
        Ok(self.paid_status())
    }

    /// End the Mesh grant: nothing more is signed. Assignments stay on chain until left or lapsed.
    pub(crate) fn withdraw_grant(&self) -> QorResult<()> {
        self.stop_worker();
        let mut ledger = self.ledger.lock();
        if ledger.grant.take().is_some() {
            ledger.bound = None;
            ledger.write(&self.dir)?;
        }
        Ok(())
    }

    pub fn paid_status(&self) -> PaidStatus {
        let ledger = self.ledger.lock();
        let paid = ledger.paid();
        PaidStatus {
            account: ledger
                .grant
                .as_deref()
                .and_then(account_of)
                .map(|a| derive::address_of(&a)),
            holdings: ledger
                .holdings
                .iter()
                .map(|(escrow, h)| HoldingView {
                    escrow: *escrow,
                    root: h.root.clone(),
                    size: h.size,
                    answered: h.answered,
                })
                .collect(),
            pending_receipts: ledger.pending.len(),
            paid_embers: paid.to_string(),
            paid_cgt: crate::cgt::format_cgt(paid),
            payments: ledger.payments.iter().rev().cloned().collect(),
        }
    }

    fn paid(&self) -> QorResult<&Paid> {
        self.paid
            .get()
            .ok_or_else(|| QorError::Mesh("The Mesh is not connected to the chain.".into()))
    }

    /// Start the worker, if seeding with the grant and none runs.
    pub(crate) fn start_worker(self: &Arc<Self>) {
        if self.paid.get().is_none() || self.grant().is_none() {
            return;
        }
        let mut worker = self.worker.lock();
        if worker.as_ref().is_some_and(|w| !w.is_finished()) {
            return;
        }
        let this = Arc::downgrade(self);
        *worker = Some(tokio::spawn(async move {
            let mut n = 0u64;
            loop {
                let Some(mesh) = this.upgrade() else { break };
                let pause = match mesh.tick(n).await {
                    Ok(()) => TICK,
                    // A node that went away, a chain without the Mesh: said once a minute, not every round.
                    Err(e) => {
                        tracing::warn!(error = %e, "the Mesh's paid work paused for a minute");
                        BACK_OFF
                    }
                };
                drop(mesh);
                n += 1;
                tokio::time::sleep(pause).await;
            }
        }));
    }

    pub(crate) fn stop_worker(&self) {
        if let Some(worker) = self.worker.lock().take() {
            worker.abort();
        }
    }

    /// One round of paid work. Public to the crate so a test can drive it without waiting for the timer.
    pub(crate) async fn tick(self: &Arc<Self>, n: u64) -> QorResult<()> {
        let Some(grant) = self.grant() else {
            return Ok(());
        };
        let paid = self.paid()?;
        self.bind(paid, &grant).await?;
        if n.is_multiple_of(LOOK_EVERY) {
            self.forget_ended(paid, &grant).await?;
        }
        self.answer(paid, &grant).await?;
        self.claim(paid, &grant).await?;
        if n.is_multiple_of(LOOK_EVERY) {
            self.take_on(paid, &grant).await?;
        }
        Ok(())
    }

    /// The node's key, while it runs.
    async fn node_id(&self) -> Option<[u8; 32]> {
        let node = self.node.lock().await;
        node.as_ref().map(|n| *n.router.endpoint().id().as_bytes())
    }

    async fn bind(&self, paid: &Paid, grant: &MeshGrant) -> QorResult<()> {
        let Some(node) = self.node_id().await else {
            return Ok(());
        };
        if self.ledger.lock().bound.as_deref() == Some(hex::encode(node).as_str()) {
            return Ok(());
        }
        if paid.chain.mesh_node(grant.account()).await? != Some(node) {
            paid.chain
                .mesh_submit(&paid.vault, grant, &Granted::BindNode { node })
                .await?;
        }
        let mut ledger = self.ledger.lock();
        ledger.bound = Some(hex::encode(node));
        ledger.write(&self.dir)
    }

    /// Drop holdings the chain no longer has: closed escrows and lapsed or left assignments.
    async fn forget_ended(&self, paid: &Paid, grant: &MeshGrant) -> QorResult<()> {
        let held: Vec<u64> = self.ledger.lock().holdings.keys().copied().collect();
        for escrow in held {
            if paid
                .chain
                .mesh_assignment(escrow, grant.account())
                .await?
                .is_none()
            {
                let mut ledger = self.ledger.lock();
                ledger.holdings.remove(&escrow);
                ledger.write(&self.dir)?;
            }
        }
        Ok(())
    }

    /// Answer every open challenge on what this PC holds.
    async fn answer(&self, paid: &Paid, grant: &MeshGrant) -> QorResult<()> {
        let held: Vec<(u64, Holding)> = self
            .ledger
            .lock()
            .holdings
            .iter()
            .map(|(e, h)| (*e, h.clone()))
            .collect();
        for (escrow, holding) in held {
            let Some((period, index)) = paid.chain.mesh_challenge(escrow, grant.account()).await?
            else {
                continue;
            };
            // Answered already, or taken this period, whose first challenge is the next one's.
            if holding.answered == Some(period) || holding.taken >= period {
                continue;
            }
            let (group, proof) = {
                let node = self.node.lock().await;
                let Some(node) = node.as_ref() else {
                    return Ok(());
                };
                let root = super::parse_root(&holding.root)?;
                prove_from_store(&node.store, root, holding.size, index).await?
            };
            let done = paid
                .chain
                .mesh_submit(
                    &paid.vault,
                    grant,
                    &Granted::AnswerChallenge {
                        escrow,
                        group,
                        proof,
                    },
                )
                .await;
            let mut ledger = self.ledger.lock();
            match done {
                Ok(done) => {
                    ledger.record("storage", Some(escrow), done.paid, done.block_hash);
                }
                // Answered by an earlier run of this launcher whose record did not survive.
                Err(e) if e.to_string().contains("AlreadyAnswered") => {}
                Err(e) => return Err(e),
            }
            if let Some(h) = ledger.holdings.get_mut(&escrow) {
                h.answered = Some(period);
            }
            ledger.write(&self.dir)?;
        }
        Ok(())
    }

    /// Claim payment for the receipts receivers sent: all at once, or one by one if any of them is refused.
    async fn claim(&self, paid: &Paid, grant: &MeshGrant) -> QorResult<()> {
        let pending = self.ledger.lock().pending();
        if pending.is_empty() {
            return Ok(());
        }
        let now = paid.chain.mesh_period().await?;
        // The chain pays a receipt for this period or the last.
        let live: Vec<_> = pending
            .into_iter()
            .filter(|(r, _)| r.period.saturating_add(1) >= now)
            .collect();
        let mut kept = Vec::new();
        for batch in live.chunks(BATCH) {
            match paid
                .chain
                .mesh_submit(
                    &paid.vault,
                    grant,
                    &Granted::SubmitReceipts {
                        receipts: batch.to_vec(),
                    },
                )
                .await
            {
                Ok(done) => self
                    .ledger
                    .lock()
                    .record("serving", None, done.paid, done.block_hash),
                Err(_) => {
                    for one in batch {
                        match paid
                            .chain
                            .mesh_submit(
                                &paid.vault,
                                grant,
                                &Granted::SubmitReceipts {
                                    receipts: vec![one.clone()],
                                },
                            )
                            .await
                        {
                            Ok(done) => self.ledger.lock().record(
                                "serving",
                                Some(one.0.escrow),
                                done.paid,
                                done.block_hash,
                            ),
                            // The chain refused it for good (paid already, over a cap, stale): dropped.
                            Err(e) if e.to_string().contains("but failed") => {}
                            // The node or the network: kept for the next round.
                            Err(_) => kept.push(one.clone()),
                        }
                    }
                }
            }
        }
        let mut ledger = self.ledger.lock();
        ledger.set_pending(&kept);
        ledger.write(&self.dir)
    }

    /// Take one copy an escrow still asks for, of a file that fits the disk limit, once it is here and verified.
    async fn take_on(self: &Arc<Self>, paid: &Paid, grant: &MeshGrant) -> QorResult<()> {
        let me = grant.account();
        let limit = self.settings().disk_bytes;
        let (_, used) = self.holdings_now().await;
        for escrow in paid.chain.mesh_escrows().await? {
            let held_already = self.ledger.lock().holdings.contains_key(&escrow.id);
            if escrow.uploader == me
                || escrow.assigned >= escrow.copies
                || held_already
                || paid.chain.mesh_assignment(escrow.id, me).await?.is_some()
                || paid.chain.mesh_taken_down(escrow.root).await?
            {
                continue;
            }
            let hash = Hash::from_bytes(escrow.root);
            if !self.holds(hash).await {
                if used.saturating_add(escrow.size) > limit {
                    continue;
                }
                if let Err(e) = self.fetch_from_holders(paid, &escrow).await {
                    tracing::info!(escrow = escrow.id, error = %e, "the Mesh could not fetch work to take on");
                    continue;
                }
            }
            paid.chain
                .mesh_submit(
                    &paid.vault,
                    grant,
                    &Granted::TakeAssignment { escrow: escrow.id },
                )
                .await?;
            let taken = paid
                .chain
                .mesh_assignment(escrow.id, me)
                .await?
                .map(|a| a.taken)
                .unwrap_or(u32::MAX);
            let mut ledger = self.ledger.lock();
            ledger.holdings.insert(
                escrow.id,
                Holding {
                    root: hex::encode(escrow.root),
                    size: escrow.size,
                    taken,
                    answered: None,
                },
            );
            ledger.write(&self.dir)?;
            // One a round, so a person's limits are read again before the next.
            return Ok(());
        }
        Ok(())
    }

    async fn holds(&self, hash: Hash) -> bool {
        let node = self.node.lock().await;
        match node.as_ref() {
            Some(n) => matches!(
                n.store.blobs().status(hash).await,
                Ok(BlobStatus::Complete { .. })
            ),
            None => false,
        }
    }

    async fn holdings_now(&self) -> (u64, u64) {
        let node = self.node.lock().await;
        match node.as_ref() {
            Some(n) => super::holdings(&n.store).await,
            None => (0, 0),
        }
    }

    /// Fetch a root by its fingerprint alone, from whoever the chain says holds it (L10.2), verified, and keep it.
    pub async fn fetch_root(self: &Arc<Self>, root: &str) -> QorResult<Fetched> {
        let paid = self.paid()?;
        let hash = super::parse_root(root)?;
        if self.settings().disk_bytes == 0 {
            return Err(QorError::Mesh(
                "Choose how much disk the Mesh may use first.".into(),
            ));
        }
        if self.node.lock().await.is_none() {
            self.run(false).await?;
        }
        if let Some(size) = self.complete_size(hash).await {
            return Ok(Fetched {
                root: hash.to_hex().to_string(),
                size,
            });
        }
        let escrows: Vec<Escrow> = paid
            .chain
            .mesh_escrows()
            .await?
            .into_iter()
            .filter(|e| e.root == *hash.as_bytes())
            .collect();
        if escrows.is_empty() {
            return Err(QorError::Mesh(
                "Nobody on the chain is paid to hold that file. Ask for a Mesh ticket instead."
                    .into(),
            ));
        }
        let mut last = None;
        for escrow in &escrows {
            match self.fetch_from_holders(paid, escrow).await {
                Ok(fetched) => return Ok(fetched),
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| QorError::Mesh("Nobody who holds that file answered.".into())))
    }

    async fn complete_size(&self, hash: Hash) -> Option<u64> {
        let node = self.node.lock().await;
        match node.as_ref()?.store.blobs().status(hash).await {
            Ok(BlobStatus::Complete { size }) => Some(size),
            _ => None,
        }
    }

    /// Fetch an escrow's file from its assigned seeders, then its uploader, and send a receipt to the seeder it came from.
    async fn fetch_from_holders(&self, paid: &Paid, escrow: &Escrow) -> QorResult<Fetched> {
        let me = self.grant().map(|g| g.account());
        let seeders: Vec<[u8; 32]> = paid
            .chain
            .mesh_seeders(escrow.id)
            .await?
            .into_iter()
            .map(|(account, _)| account)
            .collect();
        let candidates = seeders
            .iter()
            .map(|s| (*s, true))
            .chain(std::iter::once((escrow.uploader, false)))
            .filter(|(account, _)| Some(*account) != me);
        let hash = Hash::from_bytes(escrow.root);
        let mut last = None;
        for (account, assigned) in candidates {
            let Some(node) = paid.chain.mesh_node(account).await? else {
                continue;
            };
            let Ok(id) = EndpointId::from_bytes(&node) else {
                continue;
            };
            match self.fetch_hash_from(EndpointAddr::from(id), hash).await {
                Ok((fetched, read)) => {
                    if assigned && read > 0 {
                        if let Err(e) = self.send_receipt(paid, escrow.id, account, id, read).await
                        {
                            tracing::info!(escrow = escrow.id, error = %e, "a receipt did not reach its seeder");
                        }
                    }
                    return Ok(fetched);
                }
                Err(e) => last = Some(e),
            }
        }
        Err(last.unwrap_or_else(|| {
            QorError::Mesh("Nobody who holds that file has a node bound.".into())
        }))
    }

    /// Sign a receipt for everything received from `seeder` for `escrow` this period, and send it to that seeder.
    async fn send_receipt(
        &self,
        paid: &Paid,
        escrow: u64,
        seeder: [u8; 32],
        seeder_node: EndpointId,
        read: u64,
    ) -> QorResult<()> {
        let Some(grant) = self.grant() else {
            return Ok(());
        };
        let period = paid.chain.mesh_period().await?;
        let genesis = paid.chain.mesh_genesis().await?;
        let seeder_hex = hex::encode(seeder);
        let bytes = {
            let mut ledger = self.ledger.lock();
            ledger
                .received
                .retain(|r| r.period.saturating_add(1) >= period);
            let so_far = ledger
                .received
                .iter()
                .find(|r| r.escrow == escrow && r.period == period && r.seeder == seeder_hex)
                .map_or(0, |r| r.bytes);
            let bytes = so_far.saturating_add(read);
            ledger
                .received
                .retain(|r| !(r.escrow == escrow && r.period == period && r.seeder == seeder_hex));
            ledger.received.push(Received {
                escrow,
                period,
                seeder: seeder_hex,
                bytes,
            });
            ledger.write(&self.dir)?;
            bytes
        };
        let receipt = Receipt {
            escrow,
            period,
            seeder,
            receiver: grant.account(),
            bytes,
        };
        let signature = paid
            .vault
            .sign_granted(&grant, &receipt_message(&genesis, &receipt))?;
        let endpoint = {
            let node = self.node.lock().await;
            node.as_ref()
                .ok_or_else(|| QorError::Mesh("The Mesh is not running.".into()))?
                .router
                .endpoint()
                .clone()
        };
        let unreachable =
            |e: String| QorError::Mesh(format!("The seeder could not be reached: {e}"));
        let conn = tokio::time::timeout(DIAL, endpoint.connect(seeder_node, RECEIPT_ALPN))
            .await
            .map_err(|_| unreachable("no answer".into()))?
            .map_err(|e| unreachable(e.to_string()))?;
        let (mut send, mut recv) = conn
            .open_bi()
            .await
            .map_err(|e| unreachable(e.to_string()))?;
        send.write_all(&(receipt, signature).encode())
            .await
            .map_err(|e| unreachable(e.to_string()))?;
        send.finish().map_err(|e| unreachable(e.to_string()))?;
        let answer = recv
            .read_to_end(1)
            .await
            .map_err(|e| unreachable(e.to_string()))?;
        conn.close(0u32.into(), b"");
        if answer != [1] {
            return Err(QorError::Mesh("The seeder refused the receipt.".into()));
        }
        Ok(())
    }

    /// A receipt a receiver sent: kept if it is for this account, on a file it holds, from someone else, signed by them
    /// over this chain's receipt message. The newest of a receiver's receipts for an escrow and period replaces the rest.
    async fn take_receipt(&self, receipt: Receipt, signature: [u8; 64]) -> bool {
        let Some(grant) = self.grant() else {
            return false;
        };
        let Some(paid) = self.paid.get() else {
            return false;
        };
        if receipt.seeder != grant.account()
            || receipt.receiver == grant.account()
            || !self.ledger.lock().holdings.contains_key(&receipt.escrow)
        {
            return false;
        }
        let Ok(genesis) = paid.chain.mesh_genesis().await else {
            return false;
        };
        if !derive::verify(
            &receipt.receiver,
            &receipt_message(&genesis, &receipt),
            &signature,
        ) {
            return false;
        }
        let mut ledger = self.ledger.lock();
        let mut pending = ledger.pending();
        if pending.iter().any(|(r, _)| {
            r.escrow == receipt.escrow
                && r.period == receipt.period
                && r.receiver == receipt.receiver
                && r.bytes >= receipt.bytes
        }) {
            return true;
        }
        pending.retain(|(r, _)| {
            !(r.escrow == receipt.escrow
                && r.period == receipt.period
                && r.receiver == receipt.receiver)
        });
        pending.push((receipt, signature));
        ledger.set_pending(&pending);
        ledger.write(&self.dir).is_ok()
    }
}

/// Where receipts arrive (ADR-097 decision 5).
#[derive(Debug, Clone)]
pub(crate) struct ReceiptDesk(pub(crate) Weak<Mesh>);

impl ProtocolHandler for ReceiptDesk {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let Ok((mut send, mut recv)) = connection.accept_bi().await else {
            return Ok(());
        };
        // A receipt and its signature are 149 bytes; anything longer is not one.
        let kept = match recv.read_to_end(256).await {
            Ok(bytes) => match (
                <(Receipt, [u8; 64])>::decode(&mut &bytes[..]),
                self.0.upgrade(),
            ) {
                (Ok((receipt, signature)), Some(mesh)) => {
                    mesh.take_receipt(receipt, signature).await
                }
                _ => false,
            },
            Err(_) => false,
        };
        let _ = send.write_all(&[u8::from(kept)]).await;
        let _ = send.finish();
        let _ = connection.closed().await;
        Ok(())
    }
}

/// Group `index` of the blob `root` in `store`, and its proof, read a piece at a time.
pub(crate) async fn prove_from_store(
    store: &FsStore,
    root: Hash,
    size: u64,
    index: u64,
) -> QorResult<(Vec<u8>, Vec<[u8; 32]>)> {
    let failed =
        |e: String| QorError::Mesh(format!("The Mesh could not read a file it holds: {e}"));
    let (start, len) = mesh_proof::group_range(size, index)
        .ok_or_else(|| failed(format!("the chain asked for group {index} of {size} bytes")))?;
    let mut reader = store.blobs().reader(root);
    let mut group = vec![0u8; len as usize];
    reader
        .seek(std::io::SeekFrom::Start(start))
        .await
        .map_err(|e| failed(e.to_string()))?;
    reader
        .read_exact(&mut group)
        .await
        .map_err(|e| failed(e.to_string()))?;
    let mut proof = Vec::new();
    let mut piece = vec![0u8; PIECE];
    for (start, len) in mesh_proof::siblings(size, index).unwrap_or_default() {
        reader
            .seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|e| failed(e.to_string()))?;
        let mut hasher = mesh_proof::Subtree::at(start);
        let mut left = len;
        while left > 0 {
            let n = left.min(PIECE as u64) as usize;
            reader
                .read_exact(&mut piece[..n])
                .await
                .map_err(|e| failed(e.to_string()))?;
            hasher.update(&piece[..n]);
            left -= n as u64;
        }
        proof.push(hasher.finish());
    }
    Ok((group, proof))
}

#[cfg(test)]
#[path = "paid_tests.rs"]
mod tests;
