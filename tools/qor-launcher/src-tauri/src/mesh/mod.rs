//! The Mesh, the launcher's side (ADR-095; docs/blueprints/mesh.md; roadmap L10.1).
//!
//! A peer-to-peer node in the launcher's host, on iroh. It holds blobs named by their BLAKE3 hash, which is ADR-047's
//! fingerprint, and streams them to peers verified against that hash in 16 KiB groups, so a peer that sends one wrong
//! byte is refused at once and nothing unverified is kept.
//!
//! What L10.1 does, and nothing more:
//!
//! - **Seeding is off until the person turns it on**, and turning it on needs the two limits they give: disk for the
//!   Mesh's store, and upload in bytes a second. Off, nothing listens and nothing is served.
//! - **One's own published work is seeded**: what the temporary content store holds (ADR-047 question 13) is taken in
//!   at start, when seeding is turned on and once a minute while it runs, referenced where it lies rather than copied.
//! - **A root is fetched from a peer named by a ticket**, verified, and kept within the disk limit.
//!
//! Paid work (L10.2 to L10.4, the launcher's half; ADR-096, ADR-097) is in [`paid`]: once the person gives the Mesh
//! grant, the node is bound to their account, challenges are answered, receipts are claimed, others' published work is
//! taken on, and a root is found by its fingerprint through the chain.
//!
//! The node's key is made on this PC and kept in the Mesh's folder. It is never the vault's key, and it can move no
//! CGT; ADR-095 decision 1 ties it to the person's account by a binding the account signs on chain.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh::address_lookup::memory::MemoryLookup;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr, RelayMode, SecretKey};
use iroh_blobs::api::blobs::{AddPathOptions, ImportMode};
use iroh_blobs::api::proto::BlobStatus;
use iroh_blobs::provider::events::{EventMask, EventSender, ProviderMessage, ThrottleMode};
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::ticket::BlobTicket;
use iroh_blobs::{BlobFormat, BlobsProtocol, Hash};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::content::TemporaryStore;
use crate::error::{QorError, QorResult};

pub mod commands;
pub mod paid;
#[cfg(test)]
mod tests;

/// The Mesh's folder inside the launcher's data directory.
const DIR: &str = "mesh";
/// How often one's own published work is taken in again while seeding.
const RESYNC: Duration = Duration::from_secs(60);
/// The largest burst served before the upload limit applies: one second of it, or one 16 KiB group if that is more.
const GROUP: u64 = 16 * 1024;
/// How long a peer has to answer before a fetch gives up on it.
const DIAL: Duration = Duration::from_secs(20);

/// What the person chose. Off, with no limits, until they say otherwise.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub seeding: bool,
    /// The most the Mesh's store may hold, in bytes.
    pub disk_bytes: u64,
    /// The most served to others each second, in bytes.
    pub upload_bytes_per_second: u64,
}

/// What the Mesh page shows.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub settings: Settings,
    /// This launcher's node, as peers dial it; none while nothing runs.
    pub node: Option<String>,
    /// Blobs the store holds completely, and their bytes.
    pub held: u64,
    pub used_bytes: u64,
    /// Bytes served to others since the launcher started.
    pub served_bytes: u64,
    /// Paid work: what this PC holds for escrows, and what it was paid.
    pub paid: paid::PaidStatus,
}

/// A file the Mesh holds: one's own published work, or something fetched.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Held {
    pub root: String,
    pub size: u64,
    /// `own` or `fetched`.
    pub kind: &'static str,
}

/// What a fetch brought.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Fetched {
    pub root: String,
    pub size: u64,
}

/// Where the node listens: the public network, through n0's relays and address lookup, or this PC alone (tests).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Net {
    Public,
    Local,
}

struct Node {
    router: Router,
    store: FsStore,
    serving: bool,
    resync: Option<tokio::task::JoinHandle<()>>,
}

pub struct Mesh {
    dir: PathBuf,
    own: TemporaryStore,
    net: Net,
    settings: parking_lot::Mutex<Settings>,
    node: Mutex<Option<Node>>,
    served: Arc<AtomicU64>,
    /// The upload limit as the throttle reads it, so a change applies to transfers already under way.
    rate: Arc<AtomicU64>,
    /// Peers' addresses learned out of band; iroh's own lookup finds the rest by node key.
    lookup: MemoryLookup,
    paid: std::sync::OnceLock<paid::Paid>,
    ledger: parking_lot::Mutex<paid::Ledger>,
    worker: parking_lot::Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Mesh {
    pub fn new(data_dir: &Path, net: Net) -> Arc<Self> {
        let dir = data_dir.join(DIR);
        let settings = read_settings(&dir);
        Arc::new(Self {
            own: TemporaryStore::in_data_dir(data_dir),
            rate: Arc::new(AtomicU64::new(settings.upload_bytes_per_second)),
            settings: parking_lot::Mutex::new(settings),
            served: Arc::new(AtomicU64::new(0)),
            node: Mutex::new(None),
            lookup: MemoryLookup::new(),
            paid: std::sync::OnceLock::new(),
            ledger: parking_lot::Mutex::new(paid::Ledger::read(&dir)),
            worker: parking_lot::Mutex::new(None),
            dir,
            net,
        })
    }

    pub fn settings(&self) -> Settings {
        *self.settings.lock()
    }

    /// Start seeding if the person left it on.
    pub async fn resume(self: &Arc<Self>) -> QorResult<()> {
        if self.settings().seeding {
            self.run(true).await?;
        }
        Ok(())
    }

    /// Save what the person chose, and start or stop serving to match.
    pub async fn configure(self: &Arc<Self>, settings: Settings) -> QorResult<Status> {
        if settings.seeding && (settings.disk_bytes == 0 || settings.upload_bytes_per_second == 0) {
            return Err(QorError::Mesh(
                "Choose how much disk and upload to give the Mesh before seeding.".into(),
            ));
        }
        write_settings(&self.dir, &settings)?;
        *self.settings.lock() = settings;
        self.rate
            .store(settings.upload_bytes_per_second, Ordering::Relaxed);
        if settings.seeding {
            self.run(true).await?;
        } else {
            // The Mesh grant lasts while seeding does (ADR-097 decision 1).
            self.withdraw_grant()?;
            self.stop().await;
        }
        Ok(self.status().await)
    }

    pub async fn status(&self) -> Status {
        let node = self.node.lock().await;
        let (held, used_bytes) = match node.as_ref() {
            Some(n) => holdings(&n.store).await,
            None => (0, 0),
        };
        Status {
            settings: self.settings(),
            node: node.as_ref().map(|n| n.router.endpoint().id().to_string()),
            held,
            used_bytes,
            served_bytes: self.served.load(Ordering::Relaxed),
            paid: self.paid_status(),
        }
    }

    /// Where this launcher's node can be reached, while it runs.
    pub async fn addr(&self) -> Option<EndpointAddr> {
        let node = self.node.lock().await;
        node.as_ref().map(|n| n.router.endpoint().addr())
    }

    /// Learn where a peer can be reached, out of band: for two launchers on one PC with no relay (tests).
    pub fn learn(&self, addr: EndpointAddr) {
        self.lookup.add_endpoint_info(addr);
    }

    /// Take in what the temporary content store holds that the Mesh does not, within the disk limit. Returns how many
    /// blobs were taken in.
    pub async fn take_in_own(&self) -> QorResult<u64> {
        let node = self.node.lock().await;
        let Some(node) = node.as_ref() else {
            return Ok(0);
        };
        take_in(&self.own, &node.store, self.settings().disk_bytes).await
    }

    /// What the Mesh holds, own work first, each by its root. Empty while nothing runs.
    pub async fn held(&self) -> Vec<Held> {
        use n0_future::StreamExt;
        let node = self.node.lock().await;
        let Some(node) = node.as_ref() else {
            return Vec::new();
        };
        let Ok(mut tags) = node.store.tags().list().await else {
            return Vec::new();
        };
        let mut held = Vec::new();
        while let Some(tag) = tags.next().await {
            let Ok(tag) = tag else { continue };
            let name = String::from_utf8_lossy(tag.name.as_ref()).into_owned();
            let kind = if name.starts_with("own/") {
                "own"
            } else if name.starts_with("fetched/") {
                "fetched"
            } else {
                continue;
            };
            if let Ok(BlobStatus::Complete { size }) = node.store.blobs().status(tag.hash).await {
                held.push(Held {
                    root: tag.hash.to_hex().to_string(),
                    size,
                    kind,
                });
            }
        }
        held.sort_by(|a, b| (a.kind != "own", &a.root).cmp(&(b.kind != "own", &b.root)));
        held
    }

    /// A ticket naming this launcher and a root it holds, for another to fetch it with.
    pub async fn ticket(&self, root: &str) -> QorResult<String> {
        let hash = parse_root(root)?;
        let node = self.node.lock().await;
        let node = node
            .as_ref()
            .filter(|n| n.serving)
            .ok_or_else(|| QorError::Mesh("Turn seeding on to share from this PC.".into()))?;
        if !matches!(
            node.store.blobs().status(hash).await,
            Ok(BlobStatus::Complete { .. })
        ) {
            return Err(QorError::Mesh("This PC does not hold that file.".into()));
        }
        let addr = node.router.endpoint().addr();
        Ok(BlobTicket::new(addr, hash, BlobFormat::Raw).to_string())
    }

    /// Fetch the root a ticket names from the peer it names, verified, and keep it. Refused before any byte arrives if
    /// the size the peer reports would pass the disk limit.
    pub async fn fetch(self: &Arc<Self>, ticket: &str) -> QorResult<Fetched> {
        let ticket: BlobTicket = ticket
            .trim()
            .parse()
            .map_err(|_| QorError::Mesh("That is not a Mesh ticket.".into()))?;
        if ticket.format() != BlobFormat::Raw {
            return Err(QorError::Mesh("The Mesh fetches single files.".into()));
        }
        let limit = self.settings().disk_bytes;
        if limit == 0 {
            return Err(QorError::Mesh(
                "Choose how much disk the Mesh may use first.".into(),
            ));
        }
        let hash = ticket.hash();
        // A launcher that is not seeding still fetches: its node runs without serving.
        if self.node.lock().await.is_none() {
            self.run(false).await?;
        }
        self.fetch_hash_from(ticket.addr().clone(), hash)
            .await
            .map(|(fetched, _)| fetched)
    }

    /// Fetch `hash` from the peer at `addr`, verified, and keep it within the disk limit. Returns what arrived, and how
    /// many of its bytes came from that peer: none when this launcher held it already.
    pub(crate) async fn fetch_hash_from(
        &self,
        addr: EndpointAddr,
        hash: Hash,
    ) -> QorResult<(Fetched, u64)> {
        let limit = self.settings().disk_bytes;
        let node = self.node.lock().await;
        let node = node
            .as_ref()
            .ok_or_else(|| QorError::Mesh("The Mesh is not running.".into()))?;
        if let Ok(BlobStatus::Complete { size }) = node.store.blobs().status(hash).await {
            return Ok((
                Fetched {
                    root: hash.to_hex().to_string(),
                    size,
                },
                0,
            ));
        }
        let connect = |e: String| QorError::Mesh(format!("The peer could not be reached: {e}"));
        let endpoint = node.router.endpoint();
        let conn = tokio::time::timeout(DIAL, endpoint.connect(addr, iroh_blobs::ALPN))
            .await
            .map_err(|_| connect("no answer".into()))?
            .map_err(|e| connect(e.to_string()))?;
        // The verified size, read with a proof before the blob is taken: a peer cannot claim a small file and send a
        // large one, and nothing is written if it would pass the limit.
        let (_, used) = holdings(&node.store).await;
        let size = iroh_blobs::get::request::get_verified_size(&conn, &hash)
            .await
            .map_err(|e| QorError::Mesh(format!("The peer did not prove the file's size: {e}")))?
            .0;
        if used.saturating_add(size) > limit {
            return Err(QorError::Mesh(format!(
                "That file is {size} bytes, more than the Mesh's disk limit leaves room for."
            )));
        }
        let stats = node.store.remote().fetch(conn, hash).await.map_err(|e| {
            QorError::Mesh(format!(
                "The file did not arrive as its fingerprint says: {e}"
            ))
        })?;
        node.store
            .tags()
            .set(format!("fetched/{}", hash.to_hex()), hash)
            .await
            .map_err(|e| QorError::Mesh(e.to_string()))?;
        Ok((
            Fetched {
                root: hash.to_hex().to_string(),
                size,
            },
            stats.payload_bytes_read,
        ))
    }

    /// Shut the node and its store, so another may open them.
    pub async fn stop(&self) {
        self.stop_worker();
        if let Some(node) = self.node.lock().await.take() {
            close(node).await;
        }
    }

    /// Run the node, serving or not. A running node that already matches is left alone.
    async fn run(self: &Arc<Self>, serving: bool) -> QorResult<()> {
        let mut slot = self.node.lock().await;
        if slot.as_ref().is_some_and(|n| n.serving == serving) {
            return Ok(());
        }
        if let Some(old) = slot.take() {
            close(old).await;
        }
        std::fs::create_dir_all(&self.dir).map_err(|e| QorError::Io(e.to_string()))?;
        let store = FsStore::load(self.dir.join("store"))
            .await
            .map_err(|e| QorError::Mesh(format!("The Mesh's store could not be opened: {e}")))?;
        let endpoint = bind(self.net, node_key(&self.dir)?, self.lookup.clone()).await?;
        let mut router = Router::builder(endpoint);
        if serving {
            let events = throttle(Arc::clone(&self.rate), Arc::clone(&self.served));
            router = router
                .accept(iroh_blobs::ALPN, BlobsProtocol::new(&store, Some(events)))
                .accept(paid::RECEIPT_ALPN, paid::ReceiptDesk(Arc::downgrade(self)));
        }
        let router = router.spawn();
        let resync = if serving {
            take_in(&self.own, &store, self.settings().disk_bytes).await?;
            let this = Arc::downgrade(self);
            Some(tokio::spawn(async move {
                loop {
                    tokio::time::sleep(RESYNC).await;
                    let Some(mesh) = this.upgrade() else { break };
                    if let Err(e) = mesh.take_in_own().await {
                        tracing::warn!(error = %e, "the Mesh could not take in published work");
                    }
                }
            }))
        } else {
            None
        };
        *slot = Some(Node {
            router,
            store,
            serving,
            resync,
        });
        drop(slot);
        if serving {
            self.start_worker();
        }
        Ok(())
    }
}

async fn close(node: Node) {
    if let Some(task) = node.resync {
        task.abort();
    }
    let _ = node.router.shutdown().await;
    let _ = node.store.shutdown().await;
}

async fn bind(net: Net, key: SecretKey, lookup: MemoryLookup) -> QorResult<Endpoint> {
    let failed = |e: String| QorError::Mesh(format!("The Mesh could not start its network: {e}"));
    match net {
        Net::Public => Endpoint::builder(presets::N0)
            .secret_key(key)
            .address_lookup(lookup)
            .bind()
            .await
            .map_err(|e| failed(e.to_string())),
        Net::Local => Endpoint::builder(presets::Minimal)
            .secret_key(key)
            .address_lookup(lookup)
            .relay_mode(RelayMode::Disabled)
            .bind_addr("127.0.0.1:0")
            .map_err(|e| failed(e.to_string()))?
            .bind()
            .await
            .map_err(|e| failed(e.to_string())),
    }
}

/// The node's key: made once on this PC, kept in the Mesh's folder.
fn node_key(dir: &Path) -> QorResult<SecretKey> {
    let path = dir.join("node.key");
    if let Ok(bytes) = std::fs::read(&path) {
        if let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) {
            return Ok(SecretKey::from_bytes(&bytes));
        }
    }
    let key = SecretKey::generate();
    std::fs::write(&path, key.to_bytes()).map_err(|e| QorError::Io(e.to_string()))?;
    Ok(key)
}

/// Serve no faster than the upload limit: every 16 KiB group waits for its share of a bucket that refills at the limit
/// and holds at most one second of it. What passes is counted as served.
fn throttle(rate: Arc<AtomicU64>, served: Arc<AtomicU64>) -> EventSender {
    let mask = EventMask {
        throttle: ThrottleMode::Intercept,
        ..EventMask::DEFAULT
    };
    let (tx, mut rx) = EventSender::channel(64, mask);
    let bucket = Arc::new(parking_lot::Mutex::new(Bucket::new()));
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let ProviderMessage::Throttle(msg) = msg {
                let size = msg.size;
                let wait = bucket.lock().take(size, rate.load(Ordering::Relaxed));
                let served = Arc::clone(&served);
                tokio::spawn(async move {
                    if !wait.is_zero() {
                        tokio::time::sleep(wait).await;
                    }
                    served.fetch_add(size, Ordering::Relaxed);
                    msg.tx.send(Ok(())).await.ok();
                });
            }
        }
    });
    tx
}

/// A token bucket: `take` reserves bytes and says how long to wait before they may go.
#[derive(Debug)]
pub(crate) struct Bucket {
    tokens: f64,
    at: Instant,
}

impl Bucket {
    pub(crate) fn new() -> Self {
        Self {
            tokens: 0.0,
            at: Instant::now(),
        }
    }

    pub(crate) fn take(&mut self, bytes: u64, rate: u64) -> Duration {
        if rate == 0 {
            // Seeding cannot be on with no upload limit; served nothing faster than one group a second if it were.
            return Duration::from_secs(1);
        }
        let now = Instant::now();
        let rate = rate as f64;
        let capacity = rate.max(GROUP as f64);
        self.tokens =
            (self.tokens + now.duration_since(self.at).as_secs_f64() * rate).min(capacity);
        self.at = now;
        self.tokens -= bytes as f64;
        if self.tokens >= 0.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(-self.tokens / rate)
        }
    }
}

/// How many blobs the store holds completely, and their bytes.
async fn holdings(store: &FsStore) -> (u64, u64) {
    let Ok(hashes) = store.blobs().list().hashes().await else {
        return (0, 0);
    };
    let (mut held, mut bytes) = (0u64, 0u64);
    for hash in hashes {
        if let Ok(BlobStatus::Complete { size }) = store.blobs().status(hash).await {
            held += 1;
            bytes = bytes.saturating_add(size);
        }
    }
    (held, bytes)
}

/// Take the temporary store's blobs into the Mesh's store, referenced where they lie, each checked against the root it
/// is filed under, within the disk limit.
async fn take_in(own: &TemporaryStore, store: &FsStore, limit: u64) -> QorResult<u64> {
    let Ok(entries) = std::fs::read_dir(own.dir().join("blake3")) else {
        return Ok(0);
    };
    let (_, mut used) = holdings(store).await;
    let mut added = 0;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Ok(root) = parse_root(&name) else {
            continue;
        };
        if matches!(
            store.blobs().status(root).await,
            Ok(BlobStatus::Complete { .. })
        ) {
            continue;
        }
        let Ok(size) = entry.metadata().map(|m| m.len()) else {
            continue;
        };
        if used.saturating_add(size) > limit {
            tracing::info!(root = %name, "published work left out of the Mesh: the disk limit is reached");
            continue;
        }
        // Filed under a root it does not hash to: not this launcher's to serve.
        if !hashes_to(&entry.path(), &root) {
            tracing::warn!(root = %name, "a file in the temporary store does not hash to its name; not seeded");
            continue;
        }
        store
            .blobs()
            .add_path_with_opts(AddPathOptions {
                path: entry.path(),
                format: BlobFormat::Raw,
                mode: ImportMode::TryReference,
            })
            .with_named_tag(format!("own/{name}"))
            .await
            .map_err(|e| QorError::Mesh(format!("Published work could not be taken in: {e}")))?;
        used = used.saturating_add(size);
        added += 1;
    }
    Ok(added)
}

fn hashes_to(path: &Path, root: &Hash) -> bool {
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    let mut hasher = blake3::Hasher::new();
    hasher.update_reader(file).is_ok() && hasher.finalize().as_bytes() == root.as_bytes()
}

fn parse_root(root: &str) -> QorResult<Hash> {
    let bytes = hex::decode(root.trim())
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
        .ok_or_else(|| QorError::Mesh("That is not a BLAKE3 root.".into()))?;
    Ok(Hash::from_bytes(bytes))
}

fn read_settings(dir: &Path) -> Settings {
    std::fs::read(dir.join("settings.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn write_settings(dir: &Path, settings: &Settings) -> QorResult<()> {
    std::fs::create_dir_all(dir).map_err(|e| QorError::Io(e.to_string()))?;
    let bytes =
        serde_json::to_vec_pretty(settings).map_err(|e| QorError::Internal(e.to_string()))?;
    std::fs::write(dir.join("settings.json"), bytes).map_err(|e| QorError::Io(e.to_string()))
}
