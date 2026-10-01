//! DRC-369 from the launcher: mint a project's commit (M4.1), make an asset
//! permanent, read back what an account holds, and trade assets away (L4.5).
//! Selling one and buying one are in [`super::sales`] (L4.6).
//!
//! Every call is built from the connected node's metadata, signed in the vault
//! behind the host dialog, submitted and waited on until GRANDPA finalises it —
//! the same path a transfer takes (ADR-040). Nothing here is compiled against
//! the runtime.
//!
//! **What an account holds is read from chain storage, every time.** The owner
//! index is `pallet-nfts`'s `Account` map; each item's record is
//! `Drc369::Assets`; its name is `Nfts::ItemMetadataOf` (ADR-052); whether it is
//! offered for sale is `Drc369Royalties::Listings` (ADR-061). The launcher keeps
//! no list of its own, so what the Inventory shows is what the chain says, not
//! what the launcher last believed.

use scale_decode::DecodeAsType;
use scale_encode::EncodeAsType;
use serde::{Deserialize, Serialize};
use subxt::config::DefaultExtrinsicParamsBuilder;
use subxt::dynamic::{self, Value};
use subxt::extrinsics::ExtrinsicEvents;
use subxt::utils::{AccountId32, MultiSignature, H256};

use crate::content::{CommitId, ContentRef, HashAlgo};
use crate::error::{QorError, QorResult};
use crate::vault::{canonical_address, normalise_address, Vault};
use crate::{Confirm, Prompt};

use super::config::DemiurgeConfig;
use super::sales::ListingView;
use super::{decode_signature, plainly, ChainClient, Connection, FINALITY_TIMEOUT};

/// The longest name an asset may carry: `pallet-nfts`'s `StringLimit`, 256 bytes
/// (ADR-047 decision 13 row 6). Checked here so a long folder name is refused
/// before anyone is asked to sign it.
pub const NAME_LIMIT: usize = 256;

/// The most assets one trade may carry.
///
/// Not a chain limit: `pallet-utility` would take thousands, and the real
/// ceiling is the block's weight (ADR-053 decision 6). Sixteen is what a person
/// can read in a dialog and check before approving, and a trade nobody read is
/// the failure this path is built to avoid. Raising it is a product decision
/// with a weight measurement behind it, not a constant to nudge.
pub const TRADE_LIMIT: usize = 16;

/// The longest message a trade may carry.
///
/// The message is a `System::remark_with_event` in the same transaction: it is
/// stored in a block for ever and readable by anyone, which is why the dialog
/// says so and why it is short. 256 bytes matches an asset's name limit rather
/// than inventing a second number.
pub const MESSAGE_LIMIT: usize = 256;

// ── What the launcher sends, encoded against the node's metadata ────────────

#[derive(EncodeAsType)]
#[allow(non_camel_case_types)]
pub(super) enum HashAlgoArg {
    Blake3_256,
    Sha2_256,
    Blake2_256,
}

#[derive(EncodeAsType)]
pub(super) struct ContentRefArg {
    algo: HashAlgoArg,
    root: H256,
    size: u64,
}

impl From<&ContentRef> for ContentRefArg {
    fn from(reference: &ContentRef) -> Self {
        Self {
            algo: match reference.algo {
                HashAlgo::Blake3_256 => HashAlgoArg::Blake3_256,
                HashAlgo::Sha2_256 => HashAlgoArg::Sha2_256,
                HashAlgo::Blake2_256 => HashAlgoArg::Blake2_256,
            },
            root: H256(reference.root),
            size: reference.size,
        }
    }
}

#[derive(EncodeAsType)]
pub(super) enum CommitIdArg {
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}

impl From<&CommitId> for CommitIdArg {
    fn from(commit: &CommitId) -> Self {
        match commit {
            CommitId::Sha1(bytes) => CommitIdArg::Sha1(*bytes),
            CommitId::Sha256(bytes) => CommitIdArg::Sha256(*bytes),
        }
    }
}

// ── What the launcher reads, decoded by name ─────────────────────────────────

#[derive(Debug, DecodeAsType)]
#[allow(non_camel_case_types)]
enum HashAlgoRead {
    Blake3_256,
    Sha2_256,
    Blake2_256,
}

#[derive(Debug, DecodeAsType)]
struct ContentRefRead {
    algo: HashAlgoRead,
    root: H256,
    size: u64,
}

#[derive(Debug, DecodeAsType)]
enum CommitIdRead {
    Sha1([u8; 20]),
    Sha256([u8; 32]),
}

#[derive(Debug, DecodeAsType)]
struct AssetRead {
    origin: ContentRefRead,
    current: ContentRefRead,
    commit: Option<CommitIdRead>,
    revisable: bool,
}

#[derive(Debug, DecodeAsType)]
struct ItemMetadataRead {
    data: Vec<u8>,
}

#[derive(Debug, DecodeAsType)]
struct MintedRead {
    collection: u32,
    item: u32,
}

// ── What the interface is given ─────────────────────────────────────────────

/// A content reference, as a person reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContentView {
    /// `BLAKE3-256`, `SHA-256` or `BLAKE2-256`.
    pub algo: String,
    /// The 32-byte root, as 64 hex characters.
    pub root: String,
    /// The length of what the root hashes, in bytes.
    pub size: u64,
}

impl From<&ContentRef> for ContentView {
    fn from(reference: &ContentRef) -> Self {
        Self {
            algo: reference.algo.label().into(),
            root: reference.root_hex(),
            size: reference.size,
        }
    }
}

impl From<&ContentRefRead> for ContentView {
    fn from(read: &ContentRefRead) -> Self {
        Self {
            algo: match read.algo {
                HashAlgoRead::Blake3_256 => "BLAKE3-256",
                HashAlgoRead::Sha2_256 => "SHA-256",
                HashAlgoRead::Blake2_256 => "BLAKE2-256",
            }
            .into(),
            root: hex::encode(read.root.0),
            size: read.size,
        }
    }
}

/// A pinned commit, as a person reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommitView {
    /// `SHA-1` or `SHA-256`.
    pub kind: String,
    pub id: String,
}

impl From<&CommitId> for CommitView {
    fn from(commit: &CommitId) -> Self {
        Self {
            kind: commit.kind().into(),
            id: commit.hex(),
        }
    }
}

impl From<&CommitIdRead> for CommitView {
    fn from(read: &CommitIdRead) -> Self {
        match read {
            CommitIdRead::Sha1(bytes) => Self {
                kind: "SHA-1".into(),
                id: hex::encode(bytes),
            },
            CommitIdRead::Sha256(bytes) => Self {
                kind: "SHA-256".into(),
                id: hex::encode(bytes),
            },
        }
    }
}

/// One DRC-369 asset an account holds, read from chain storage.
#[derive(Debug, Clone, Serialize)]
pub struct OwnedAsset {
    pub collection: u32,
    pub item: u32,
    /// The item's `pallet-nfts` metadata, which a mint sets to the project's
    /// name. Shown lossily if it is not UTF-8; the chain stores bytes.
    pub name: String,
    pub origin: ContentView,
    pub current: ContentView,
    pub commit: Option<CommitView>,
    pub revisable: bool,
    /// Its listing, if it is offered for sale (`Drc369Royalties::Listings`).
    pub listing: Option<ListingView>,
}

/// What a mint needs from the project, computed before anyone is asked.
#[derive(Debug, Clone)]
pub struct MintRequest {
    pub name: String,
    pub reference: ContentRef,
    pub commit: CommitId,
    pub branch: Option<String>,
    pub files: usize,
}

/// A finalised mint.
#[derive(Debug, Clone, Serialize)]
pub struct MintReceipt {
    pub collection: u32,
    pub item: u32,
    pub name: String,
    pub reference: ContentView,
    pub commit: CommitView,
    pub tx_hash: String,
    /// The finalised block the mint is in.
    pub block_hash: String,
}

/// One asset in a trade, as the interface names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TradeItem {
    pub collection: u32,
    pub item: u32,
}

/// A finalised trade: what moved, to whom, and where it was recorded.
#[derive(Debug, Clone, Serialize)]
pub struct TradeReceipt {
    pub moved: Vec<TradeItem>,
    /// The recipient in this chain's own address format, not as it was typed.
    pub to: String,
    pub tx_hash: String,
    pub block_hash: String,
}

/// A finalised switch to permanent.
#[derive(Debug, Clone, Serialize)]
pub struct PermanenceReceipt {
    pub collection: u32,
    pub item: u32,
    pub tx_hash: String,
    pub block_hash: String,
}

/// A transaction the node finalised and reported as successful.
pub(super) struct Finalised {
    pub(super) tx_hash: String,
    pub(super) block_hash: String,
    pub(super) events: ExtrinsicEvents<DemiurgeConfig>,
}

impl ChainClient {
    /// Mint a project's commit as a DRC-369 asset.
    ///
    /// `before_submit` runs after the person has approved and the vault has
    /// signed, and before anything is sent: it is where the bytes go into the
    /// temporary content store, so a declined mint writes nothing anywhere.
    pub async fn mint(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        request: &MintRequest,
        before_submit: impl FnOnce() -> QorResult<()>,
    ) -> QorResult<MintReceipt> {
        let owner = normalise_address(from)?;
        let owner_address = canonical_address(from)?;
        if request.name.len() > NAME_LIMIT {
            return Err(QorError::Qontrol(format!(
                "an asset's name is at most {NAME_LIMIT} bytes, and \"{}\" is {}",
                request.name,
                request.name.len()
            )));
        }
        if request.reference.algo != HashAlgo::Blake3_256 {
            return Err(QorError::Internal(
                "only a BLAKE3-256 reference can be minted (ADR-047 decision 1)".into(),
            ));
        }

        let connection = self.connected().await?;
        let deposits = self
            .mint_deposits(&connection, owner, request.name.len())
            .await?;

        let call = dynamic::transaction(
            "Drc369",
            "mint",
            (
                ContentRefArg::from(&request.reference),
                Some(CommitIdArg::from(&request.commit)),
                request.name.as_bytes().to_vec(),
                true,
                // `derived_from`: a mint from Projects names no source. Minting
                // a remix is not offered yet (M4.2, ADR-061).
                None::<(u32, u32)>,
            ),
        );

        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    mint_prompt(request, &owner_address, deposits, chain_name, endpoint)
                },
                before_submit,
                "mint",
            )
            .await?;

        let minted = finalised
            .events
            .iter()
            .filter_map(Result::ok)
            .find(|event| event.pallet_name() == "Drc369" && event.event_name() == "Minted")
            .ok_or_else(|| {
                QorError::Rpc(format!(
                    "the mint {} was finalised but reported no Minted event",
                    finalised.tx_hash
                ))
            })?
            .decode_fields_unchecked_as::<MintedRead>()
            .map_err(|e| QorError::Rpc(format!("the Minted event could not be read: {e}")))?;

        Ok(MintReceipt {
            collection: minted.collection,
            item: minted.item,
            name: request.name.clone(),
            reference: ContentView::from(&request.reference),
            commit: CommitView::from(&request.commit),
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }

    /// Make an asset permanent. One-way: there is no call that undoes it.
    pub async fn make_permanent(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        collection: u32,
        item: u32,
    ) -> QorResult<PermanenceReceipt> {
        let owner = normalise_address(from)?;
        let owner_address = canonical_address(from)?;
        let connection = self.connected().await?;

        // The mistakes a person can make are caught before anyone is asked.
        let asset = self
            .asset(&connection, owner, collection, item)
            .await?
            .ok_or_else(|| {
                QorError::Qontrol(format!(
                    "this account holds no DRC-369 asset {collection}/{item}"
                ))
            })?;
        if !asset.revisable {
            return Err(QorError::Qontrol(format!(
                "{} is already permanent",
                asset.name
            )));
        }

        let call = dynamic::transaction("Drc369", "make_permanent", (collection, item));
        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    permanence_prompt(&asset, &owner_address, chain_name, endpoint)
                },
                || Ok(()),
                "change",
            )
            .await?;

        Ok(PermanenceReceipt {
            collection,
            item,
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }

    /// Trade assets away: several assets to one account, in one transaction,
    /// with an optional public message (L4.5).
    ///
    /// **All of it happens or none of it does.** The transaction is
    /// `Utility::batch_all` (ADR-053), so a trade that cannot finish moves
    /// nothing — there is no state in which the recipient keeps three of four
    /// assets and the sender has no way back. A single-asset trade takes the
    /// same path, because one code path is one dialog and one receipt.
    ///
    /// **Everything that can be refused is refused before anyone is asked**: an
    /// address that is not an address, this account's own address, an empty
    /// trade, more than [`TRADE_LIMIT`], the same asset twice, an asset this
    /// account does not hold at the latest block, and a message over
    /// [`MESSAGE_LIMIT`]. What survives that is what the dialog describes.
    pub async fn trade(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        to: &str,
        items: &[TradeItem],
        message: Option<&str>,
    ) -> QorResult<TradeReceipt> {
        let owner = normalise_address(from)?;
        let owner_address = canonical_address(from)?;
        let recipient = normalise_address(to).map_err(|_| {
            QorError::Qontrol(
                "that is not an address on this chain. Paste the recipient's address, which \
                 starts with 5 and is 47 or 48 characters long."
                    .into(),
            )
        })?;
        let recipient_address = canonical_address(to)?;
        if recipient == owner {
            return Err(QorError::Qontrol(
                "that is this account's own address. A trade sends assets to someone else.".into(),
            ));
        }
        if items.is_empty() {
            return Err(QorError::Qontrol("a trade needs at least one asset".into()));
        }
        if items.len() > TRADE_LIMIT {
            return Err(QorError::Qontrol(format!(
                "a trade carries at most {TRADE_LIMIT} assets, and this one carries {}",
                items.len()
            )));
        }
        for (i, item) in items.iter().enumerate() {
            if items[..i].contains(item) {
                return Err(QorError::Qontrol(format!(
                    "asset {}/{} is in this trade twice",
                    item.collection, item.item
                )));
            }
        }
        let message = message.map(str::trim).filter(|m| !m.is_empty());
        if let Some(message) = message {
            if message.len() > MESSAGE_LIMIT {
                return Err(QorError::Qontrol(format!(
                    "a message is at most {MESSAGE_LIMIT} bytes, and this one is {}",
                    message.len()
                )));
            }
        }

        let connection = self.connected().await?;

        // Held by this account, at the latest block, one by one. The Inventory
        // may have been read minutes ago.
        let mut held = Vec::with_capacity(items.len());
        for item in items {
            held.push(
                self.asset(&connection, owner, item.collection, item.item)
                    .await?
                    .ok_or_else(|| {
                        QorError::Qontrol(format!(
                            "this account holds no DRC-369 asset {}/{}",
                            item.collection, item.item
                        ))
                    })?,
            );
        }

        let mut calls: Vec<Value> = items
            .iter()
            .map(|item| transfer_call(item, recipient))
            .collect();
        if let Some(message) = message {
            calls.push(remark_call(message));
        }
        let call = dynamic::transaction(
            "Utility",
            "batch_all",
            vec![Value::unnamed_composite(calls)],
        );

        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    trade_prompt(
                        &held,
                        &owner_address,
                        &recipient_address,
                        message,
                        chain_name,
                        endpoint,
                    )
                },
                || Ok(()),
                "trade",
            )
            .await?;

        // The chain's answer, not the launcher's: one Transferred event per
        // asset, or no receipt is written.
        let transferred = finalised
            .events
            .iter()
            .filter_map(Result::ok)
            .filter(|event| event.pallet_name() == "Nfts" && event.event_name() == "Transferred")
            .count();
        if transferred != items.len() {
            return Err(QorError::Rpc(format!(
                "the trade {} was finalised, but the chain reported {transferred} assets moved \
                 and not {}",
                finalised.tx_hash,
                items.len()
            )));
        }

        Ok(TradeReceipt {
            moved: items.to_vec(),
            to: recipient_address,
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }

    /// Every DRC-369 asset an account holds, read from chain storage at the
    /// latest block.
    pub async fn assets_of(&self, address: &str) -> QorResult<Vec<OwnedAsset>> {
        let owner = normalise_address(address)?;
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };

        // `pallet-nfts`' owner index: (owner, collection, item) -> ().
        let index = dynamic::storage::<([u8; 32], u32, u32), ()>("Nfts", "Account");
        let mut entries =
            at.storage().iter(index, (owner,)).await.map_err(|e| {
                QorError::Rpc(format!("could not read what the account holds: {e}"))
            })?;

        let mut held = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(|e| {
                QorError::Rpc(format!("could not read what the account holds: {e}"))
            })?;
            let key = entry
                .key()
                .map_err(|e| QorError::Rpc(format!("an owner index key could not be read: {e}")))?;
            let part = |i: usize| -> QorResult<u32> {
                key.part(i)
                    .and_then(|p| p.decode_as::<u32>().ok().flatten())
                    .ok_or_else(|| QorError::Rpc("an owner index key did not name an item".into()))
            };
            held.push((part(1)?, part(2)?));
        }
        held.sort_unstable();

        let mut assets = Vec::new();
        for (collection, item) in held {
            // An item without a DRC-369 record is not a DRC-369 asset.
            if let Some(asset) = self.asset(&connection, owner, collection, item).await? {
                assets.push(asset);
            }
        }
        Ok(assets)
    }

    /// One asset, if `owner` holds it and it is a DRC-369 asset.
    pub(super) async fn asset(
        &self,
        connection: &Connection,
        owner: [u8; 32],
        collection: u32,
        item: u32,
    ) -> QorResult<Option<OwnedAsset>> {
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };

        let held = at
            .storage()
            .try_fetch(
                dynamic::storage::<([u8; 32], u32, u32), ()>("Nfts", "Account"),
                (owner, collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the owner index: {e}")))?;
        if held.is_none() {
            return Ok(None);
        }

        let Some(record) = at
            .storage()
            .try_fetch(
                dynamic::storage::<(u32, u32), AssetRead>("Drc369", "Assets"),
                (collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the asset: {e}")))?
        else {
            return Ok(None);
        };
        let record = record
            .decode()
            .map_err(|e| QorError::Rpc(format!("the asset could not be decoded: {e}")))?;

        let name = match at
            .storage()
            .try_fetch(
                dynamic::storage::<(u32, u32), ItemMetadataRead>("Nfts", "ItemMetadataOf"),
                (collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the asset's name: {e}")))?
        {
            Some(value) => {
                let metadata = value.decode().map_err(|e| {
                    QorError::Rpc(format!("the asset's name could not be decoded: {e}"))
                })?;
                String::from_utf8_lossy(&metadata.data).into_owned()
            }
            None => String::new(),
        };

        let listing = self.listing(connection, owner, collection, item).await?;

        Ok(Some(OwnedAsset {
            collection,
            item,
            name,
            origin: ContentView::from(&record.origin),
            current: ContentView::from(&record.current),
            commit: record.commit.as_ref().map(CommitView::from),
            revisable: record.revisable,
            listing,
        }))
    }

    /// What a mint of a name this long would hold from `owner`, in Sparks: the
    /// item, the name, and the singles collection if this is their first.
    /// Read from the node's own constants, so the dialog says what this chain
    /// will actually hold rather than what the launcher was built believing.
    async fn mint_deposits(
        &self,
        connection: &Connection,
        owner: [u8; 32],
        name_len: usize,
    ) -> QorResult<MintDeposits> {
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let constant = |name: &'static str| -> QorResult<u128> {
            at.constants()
                .entry(dynamic::constant::<u128>("Nfts", name))
                .map_err(|e| {
                    QorError::Rpc(format!(
                        "this node has no DRC-369 asset pallets ({name}: {e}). It is \
                         running a runtime older than M4.1."
                    ))
                })
        };
        let collection = constant("CollectionDeposit")?;
        let item = constant("ItemDeposit")?;
        let metadata_base = constant("MetadataDepositBase")?;
        let per_byte = constant("DepositPerByte")?;

        let first = at
            .storage()
            .try_fetch(
                dynamic::storage::<([u8; 32],), ()>("Drc369", "Singles").unvalidated(),
                (owner,),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the singles collection: {e}")))?
            .is_none();

        let name = if name_len == 0 {
            0
        } else {
            per_byte
                .checked_mul(name_len as u128)
                .and_then(|bytes| bytes.checked_add(metadata_base))
                .ok_or_else(|| QorError::Rpc("the name's deposit overflows".into()))?
        };
        let total = item
            .checked_add(name)
            .and_then(|sum| sum.checked_add(if first { collection } else { 0 }))
            .ok_or_else(|| QorError::Rpc("the mint's deposits overflow".into()))?;

        Ok(MintDeposits { total, first })
    }

    /// Build the call from metadata, ask, sign in the vault, run
    /// `before_submit`, submit, and wait for finality and success.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn sign_and_finalise<Call: subxt::transactions::Payload>(
        &self,
        connection: &Connection,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        call: &Call,
        prompt: impl FnOnce(&str, &str) -> Prompt,
        before_submit: impl FnOnce() -> QorResult<()>,
        what: &str,
    ) -> QorResult<Finalised> {
        let signer = AccountId32(normalise_address(from)?);
        let nonce = connection
            .rpc
            .system_account_next_index(&signer)
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the account's nonce: {e}")))?;

        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let params = DefaultExtrinsicParamsBuilder::<DemiurgeConfig>::new()
            .nonce(nonce)
            .build();
        let mut signable = at
            .transactions()
            .create_signable(call, &signer, params)
            .await
            .map_err(|e| QorError::Rpc(format!("the {what} could not be built: {e}")))?;
        let payload = signable
            .signer_payload()
            .map_err(|e| QorError::Rpc(format!("the {what} could not be prepared: {e}")))?;

        let prompt = prompt(&connection.chain_name, &connection.endpoint);
        let signature = vault.sign(confirm, &prompt, from, &payload).await?;
        let signature = decode_signature(&signature)?;
        let submittable = signable
            .sign_with_account_and_signature(&signer, &MultiSignature::Sr25519(signature))
            .map_err(|e| QorError::Rpc(format!("the {what} could not be signed: {e}")))?;
        let tx_hash = format!("{:?}", submittable.hash());

        before_submit()?;

        let progress = submittable
            .submit_and_watch()
            .await
            .map_err(|e| QorError::Rpc(format!("the node refused the {what}: {}", plainly(&e))))?;
        let in_block =
            match tokio::time::timeout(FINALITY_TIMEOUT, progress.wait_for_finalized()).await {
                Ok(Ok(in_block)) => in_block,
                Ok(Err(e)) => {
                    return Err(QorError::Rpc(format!(
                        "the {what} {tx_hash} was not accepted: {}",
                        plainly(&e)
                    )))
                }
                Err(_) => {
                    return Err(QorError::Rpc(format!(
                        "the {what} {tx_hash} was submitted but has not finalised within {} \
                         seconds. It may still finalise; look in the Inventory before trying \
                         again.",
                        FINALITY_TIMEOUT.as_secs()
                    )))
                }
            };

        let block_hash = format!("{:?}", in_block.block_hash());
        let events = in_block.wait_for_success().await.map_err(|e| {
            QorError::Rpc(format!(
                "the {what} {tx_hash} was finalised in {block_hash} but failed: {}",
                plainly(&e)
            ))
        })?;

        Ok(Finalised {
            tx_hash,
            block_hash,
            events,
        })
    }
}

/// What a mint would hold, for the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MintDeposits {
    total: u128,
    /// This account has no singles collection yet, so the mint creates one.
    first: bool,
}

/// What the person approving a mint is shown.
fn mint_prompt(
    request: &MintRequest,
    owner: &str,
    deposits: MintDeposits,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let branch = request
        .branch
        .as_deref()
        .map(|b| format!(", on {b}"))
        .unwrap_or_default();
    let first = if deposits.first {
        "This is this account's first mint, so it also creates its singles collection.\n"
    } else {
        ""
    };
    Prompt {
        title: "Approve this mint".into(),
        body: format!(
            "Mint \"{name}\" as a DRC-369 asset\n\n\
             Commit: {commit} ({kind}){branch}\n\
             Files: {files}\n\
             Fingerprint: {algo} {root}\n\
             Manifest: {size} bytes\n\n\
             It can be revised later, until you make it permanent.\n\
             {first}\
             Held from your balance as deposits: {held} {symbol}. A deposit is held, not spent.\n\n\
             Owner: {owner}\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this mint with your key and sends it.",
            name = request.name,
            commit = request.commit.hex(),
            kind = request.commit.kind(),
            files = request.files,
            algo = request.reference.algo.label(),
            root = request.reference.root_hex(),
            size = request.reference.size,
            held = crate::cgt::format_cgt_grouped(deposits.total),
            symbol = crate::cgt::SYMBOL,
        ),
        approve: "Mint".into(),
    }
}

/// What the person making an asset permanent is shown. It says, before they
/// approve, that it cannot be undone.
/// `Nfts::transfer`, as one call inside a batch. The outer variant is the
/// runtime's own call enum, which is why the pallet's name wraps the call's.
fn transfer_call(item: &TradeItem, to: [u8; 32]) -> Value {
    Value::unnamed_variant(
        "Nfts",
        [Value::named_variant(
            "transfer",
            [
                ("collection", Value::u128(item.collection.into())),
                ("item", Value::u128(item.item.into())),
                // `AccountIdLookup`, so the destination is a `MultiAddress`
                // (ADR-041).
                (
                    "dest",
                    Value::unnamed_variant("Id", [Value::from_bytes(to)]),
                ),
            ],
        )],
    )
}

/// The message, as a remark in the same transaction. `remark_with_event` rather
/// than `remark`, because a message nobody can find is not a message: the event
/// is what an indexer or the recipient's client reads it from.
fn remark_call(message: &str) -> Value {
    Value::unnamed_variant(
        "System",
        [Value::named_variant(
            "remark_with_event",
            [("remark", Value::from_bytes(message.as_bytes()))],
        )],
    )
}

/// The dialog before a trade. It names every asset, because approving a list
/// nobody read is the failure this path exists to prevent, and it says plainly
/// that the assets leave this account for good.
fn trade_prompt(
    assets: &[OwnedAsset],
    owner: &str,
    recipient: &str,
    message: Option<&str>,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let list = assets
        .iter()
        .map(|asset| {
            format!(
                "  {name} ({collection}/{item}) {algo} {root}",
                name = asset.name,
                collection = asset.collection,
                item = asset.item,
                algo = asset.current.algo,
                root = asset.current.root,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let count = assets.len();
    let heading = if count == 1 {
        "Send 1 asset".to_string()
    } else {
        format!("Send {count} assets")
    };
    let message = match message {
        Some(message) => {
            format!("\n\nMessage, stored on chain for ever and readable by anyone:\n  {message}")
        }
        None => String::new(),
    };
    Prompt {
        title: "Send these assets".into(),
        body: format!(
            "{heading} to\n{recipient}\n\n\
             {list}{message}\n\n\
             This cannot be undone. Once this is in a block the assets belong to that account, \
             they leave this Inventory, and only that account can send them back.\n\n\
             All of it happens or none of it does: if one asset cannot move, nothing moves.\n\n\
             From: {owner}\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this with your key and sends it.",
        ),
        approve: if count == 1 {
            "Send the asset".into()
        } else {
            "Send the assets".into()
        },
    }
}

fn permanence_prompt(asset: &OwnedAsset, owner: &str, chain_name: &str, endpoint: &str) -> Prompt {
    Prompt {
        title: "Make this asset permanent".into(),
        body: format!(
            "Make \"{name}\" permanent\n\n\
             Asset: {collection}/{item}\n\
             Fingerprint: {algo} {root}\n\n\
             This cannot be undone. Once it is permanent nobody can revise it: not you, and \
             not anyone who owns it later.\n\n\
             Owner: {owner}\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this with your key and sends it.",
            name = asset.name,
            collection = asset.collection,
            item = asset.item,
            algo = asset.current.algo,
            root = asset.current.root,
        ),
        approve: "Make permanent".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::Scripted;

    const NOWHERE: &str = "ws://127.0.0.1:1";

    fn request() -> MintRequest {
        MintRequest {
            name: "a-song".into(),
            reference: ContentRef::of(b"a manifest"),
            commit: CommitId::Sha1([0xab; 20]),
            branch: Some("main".into()),
            files: 7,
        }
    }

    /// The dialog names what is being minted: the project, the commit by hash,
    /// the fingerprint, what is held, and where it goes.
    #[test]
    fn the_mint_prompt_names_what_it_is_approving() {
        let owner = crate::vault::derive::address_of(&[0x04u8; 32]);
        let deposits = MintDeposits {
            total: 1_002 * crate::cgt::SPARKS_PER_CGT + crate::cgt::SPARKS_PER_CGT / 2,
            first: true,
        };
        let prompt = mint_prompt(
            &request(),
            &owner,
            deposits,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );

        assert_eq!(prompt.approve, "Mint");
        for expected in [
            "a-song",
            &"ab".repeat(20),
            "SHA-1",
            "on main",
            "Files: 7",
            "BLAKE3-256",
            &request().reference.root_hex(),
            "1,002.50 CGT",
            "creates its singles collection",
            owner.as_str(),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        ] {
            assert!(
                prompt.body.contains(expected),
                "missing {expected:?} in\n{}",
                prompt.body
            );
        }

        let later = mint_prompt(
            &request(),
            &owner,
            MintDeposits {
                total: 1,
                first: false,
            },
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert!(!later.body.contains("singles collection"));
    }

    #[test]
    fn the_permanence_prompt_says_it_cannot_be_undone() {
        let owner = crate::vault::derive::address_of(&[0x05u8; 32]);
        let asset = OwnedAsset {
            collection: 3,
            item: 9,
            name: "a-song".into(),
            origin: ContentView::from(&ContentRef::of(b"one")),
            current: ContentView::from(&ContentRef::of(b"two")),
            commit: None,
            revisable: true,
            listing: None,
        };
        let prompt = permanence_prompt(
            &asset,
            &owner,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert_eq!(prompt.approve, "Make permanent");
        assert!(prompt.body.contains("cannot be undone"));
        assert!(prompt.body.contains("3/9"));
        assert!(prompt.body.contains(&ContentRef::of(b"two").root_hex()));
    }

    fn asset(collection: u32, item: u32, name: &str) -> OwnedAsset {
        OwnedAsset {
            collection,
            item,
            name: name.into(),
            origin: ContentView::from(&ContentRef::of(b"one")),
            current: ContentView::from(&ContentRef::of(b"one")),
            commit: None,
            revisable: true,
            listing: None,
        }
    }

    /// The dialog before a trade names every asset and says, in words, that the
    /// assets are gone from this account.
    #[test]
    fn the_trade_prompt_says_the_assets_are_gone() {
        let owner = crate::vault::derive::address_of(&[0x07u8; 32]);
        let recipient = crate::vault::derive::address_of(&[0x08u8; 32]);
        let prompt = trade_prompt(
            &[asset(1, 0, "a-song"), asset(1, 4, "a-remix")],
            &owner,
            &recipient,
            Some("for the album"),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );

        assert_eq!(prompt.approve, "Send the assets");
        for expected in [
            "Send 2 assets",
            "a-song",
            "(1/0)",
            "a-remix",
            "(1/4)",
            "cannot be undone",
            "they leave this Inventory",
            "All of it happens or none of it does",
            "readable by anyone",
            "for the album",
            recipient.as_str(),
            owner.as_str(),
        ] {
            assert!(
                prompt.body.contains(expected),
                "missing {expected:?} in\n{}",
                prompt.body
            );
        }

        // One asset is counted as one, and a trade with no message says nothing
        // about messages.
        let one = trade_prompt(
            &[asset(1, 0, "a-song")],
            &owner,
            &recipient,
            None,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert_eq!(one.approve, "Send the asset");
        assert!(one.body.contains("Send 1 asset"));
        assert!(!one.body.contains("readable by anyone"));
    }

    /// Everything a person can get wrong about a trade is refused before the
    /// vault is asked for anything, and before any node is needed.
    #[tokio::test]
    async fn a_trade_nobody_could_finish_asks_nothing() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let dir = std::env::temp_dir().join(format!("qor-trade-nowhere-{}", std::process::id()));
        let vault = crate::vault::testing::vault_in(&dir);
        let confirm = Scripted::approving();
        let from = crate::vault::derive::address_of(&[0x09u8; 32]);
        let to = crate::vault::derive::address_of(&[0x0au8; 32]);
        let one = TradeItem {
            collection: 1,
            item: 0,
        };

        let refusals: Vec<(&str, String, Vec<TradeItem>, Option<String>)> = vec![
            ("not an address", "not-an-address".into(), vec![one], None),
            ("this account", from.clone(), vec![one], None),
            ("at least one asset", to.clone(), vec![], None),
            (
                "at most 16 assets",
                to.clone(),
                (0..TRADE_LIMIT as u32 + 1)
                    .map(|item| TradeItem {
                        collection: 1,
                        item,
                    })
                    .collect(),
                None,
            ),
            ("in this trade twice", to.clone(), vec![one, one], None),
            (
                "at most 256 bytes",
                to.clone(),
                vec![one],
                Some("m".repeat(MESSAGE_LIMIT + 1)),
            ),
        ];

        for (expected, to, items, message) in refusals {
            let error = client
                .trade(&vault, &confirm, &from, &to, &items, message.as_deref())
                .await
                .unwrap_err();
            assert!(
                error.to_string().contains(expected),
                "expected {expected:?}, got {error}"
            );
        }
        assert_eq!(confirm.times_asked(), 0);

        // A trade that IS well formed gets as far as the node, and no further,
        // because there is no node. Still nothing signed.
        let error = client
            .trade(&vault, &confirm, &from, &to, &[one], None)
            .await
            .unwrap_err();
        assert!(matches!(error.kind(), "network" | "rpc"), "{error}");
        assert_eq!(confirm.times_asked(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Nothing is asked, signed or written before the node has answered, and a
    /// name the chain would refuse is refused first.
    #[tokio::test]
    async fn a_mint_that_cannot_be_built_asks_nothing_and_writes_nothing() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let dir = std::env::temp_dir().join(format!("qor-mint-nowhere-{}", std::process::id()));
        let vault = crate::vault::testing::vault_in(&dir);
        let confirm = Scripted::approving();
        let from = crate::vault::derive::address_of(&[0x06u8; 32]);
        let mut wrote = false;

        let error = client
            .mint(&vault, &confirm, &from, &request(), || {
                wrote = true;
                Ok(())
            })
            .await
            .unwrap_err();
        assert!(matches!(error.kind(), "network" | "rpc"), "{error}");
        assert_eq!(confirm.times_asked(), 0);
        assert!(
            !wrote,
            "nothing reaches the store for a mint that was never approved"
        );

        let long = MintRequest {
            name: "x".repeat(NAME_LIMIT + 1),
            ..request()
        };
        let error = client
            .mint(&vault, &confirm, &from, &long, || Ok(()))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("at most 256 bytes"), "{error}");
        assert_eq!(confirm.times_asked(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
