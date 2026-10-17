//! The Mesh on chain, the launcher's side (ADR-095, ADR-096, ADR-097; `pallet-mesh` in `chain/pallets/mesh`).
//!
//! - **Reads:** escrows, who holds each, each account's bound node, the takedown list, and the chain's answer to which
//!   group a seeder must show now (`MeshApi_challenge`).
//! - **Calls the Mesh grant signs** (ADR-097 decision 2): binding a node, taking and leaving an assignment, answering a
//!   challenge, submitting receipts. None moves the person's CGT anywhere. Each is encoded here, byte by byte, from the
//!   pallet's index and the call's, and the node's metadata must encode the call it names back to exactly those bytes
//!   before anything is signed (decision 3). A node that lied about its metadata gets a refusal, not a signature.
//! - **Opening an escrow** moves the uploader's CGT, so it keeps its dialog, like every other transaction.
//! - **Receipts** are built here from the chain's genesis hash, exactly as the pallet builds them, never asked of a node.

use codec::{Compact, Decode, Encode};
use scale_decode::DecodeAsType;
use serde::Serialize;
use subxt::config::DefaultExtrinsicParamsBuilder;
use subxt::dynamic;
use subxt::utils::{AccountId32, MultiSignature};

use super::config::DemiurgeConfig;
use super::{plainly, ChainClient, FINALITY_TIMEOUT};
use crate::error::{QorError, QorResult};
use crate::vault::{canonical_address, derive::address_of, normalise_address, MeshGrant, Vault};
use crate::{Confirm, Prompt};

/// The Mesh's index in the runtime (`chain/runtime/src/lib.rs`: `Mesh: pallet_mesh = 12`). Compiled in on purpose: a
/// runtime that moves it makes the grant refuse until a launcher that knows the new index ships (ADR-097 decision 3).
pub const PALLET_INDEX: u8 = 12;

/// What every receipt's signed bytes begin with (`pallet_mesh::RECEIPT_CONTEXT`).
pub const RECEIPT_CONTEXT: &[u8] = b"demiurge/mesh-receipt/v1";

/// A storage challenge's group, as `mesh_proof` and the pallet count it.
pub const GROUP: u64 = mesh_proof::GROUP;

/// A call the Mesh grant signs, with its arguments. Nothing else can be signed under the grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Granted {
    BindNode {
        node: [u8; 32],
    },
    TakeAssignment {
        escrow: u64,
    },
    LeaveAssignment {
        escrow: u64,
    },
    AnswerChallenge {
        escrow: u64,
        group: Vec<u8>,
        proof: Vec<[u8; 32]>,
    },
    SubmitReceipts {
        receipts: Vec<(Receipt, [u8; 64])>,
    },
}

impl Granted {
    /// The call's index in `pallet-mesh` and its name there.
    fn index_and_name(&self) -> (u8, &'static str) {
        match self {
            Self::BindNode { .. } => (0, "bind_node"),
            Self::TakeAssignment { .. } => (4, "take_assignment"),
            Self::LeaveAssignment { .. } => (5, "leave_assignment"),
            Self::AnswerChallenge { .. } => (7, "answer_challenge"),
            Self::SubmitReceipts { .. } => (8, "submit_receipts"),
        }
    }

    /// The call's bytes, as the runtime decodes them: the pallet's index, the call's, then its arguments in SCALE.
    pub fn call_data(&self) -> Vec<u8> {
        let mut out = vec![PALLET_INDEX, self.index_and_name().0];
        match self {
            Self::BindNode { node } => node.encode_to(&mut out),
            Self::TakeAssignment { escrow } | Self::LeaveAssignment { escrow } => {
                escrow.encode_to(&mut out)
            }
            Self::AnswerChallenge {
                escrow,
                group,
                proof,
            } => {
                escrow.encode_to(&mut out);
                group.encode_to(&mut out);
                proof.encode_to(&mut out);
            }
            Self::SubmitReceipts { receipts } => {
                Compact(receipts.len() as u32).encode_to(&mut out);
                for (receipt, signature) in receipts {
                    receipt.encode_to(&mut out);
                    // `MultiSignature::Sr25519`, the runtime's signature type's second variant.
                    1u8.encode_to(&mut out);
                    signature.encode_to(&mut out);
                }
            }
        }
        out
    }

    fn what(&self) -> &'static str {
        match self {
            Self::BindNode { .. } => "Mesh node binding",
            Self::TakeAssignment { .. } => "Mesh assignment",
            Self::LeaveAssignment { .. } => "Mesh assignment's end",
            Self::AnswerChallenge { .. } => "Mesh storage answer",
            Self::SubmitReceipts { .. } => "Mesh serving claim",
        }
    }
}

/// A granted call, finalised: where, and what the chain paid the signer for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Done {
    pub tx_hash: String,
    pub block_hash: String,
    /// Sparks.
    pub paid: u128,
}

/// What a receiver signs (`pallet_mesh::Receipt`): everything received from one seeder for one escrow in one period, so
/// far. Encoded field for field as the pallet's.
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub struct Receipt {
    pub escrow: u64,
    pub period: u32,
    pub seeder: [u8; 32],
    pub receiver: [u8; 32],
    pub bytes: u64,
}

/// The bytes a receiver signs for `receipt` on the chain whose genesis hash is `genesis`, as `pallet_mesh` builds them.
pub fn receipt_message(genesis: &[u8; 32], receipt: &Receipt) -> Vec<u8> {
    (RECEIPT_CONTEXT, genesis, receipt).encode()
}

/// One escrow, as the chain holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Escrow {
    pub id: u64,
    pub uploader: [u8; 32],
    pub root: [u8; 32],
    pub size: u64,
    pub copies: u8,
    pub storage_price: u128,
    pub serving_price_per_mib: u128,
    pub serving_cap_per_period: u64,
    pub assigned: u8,
}

/// A seeder's hold on an escrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assignment {
    pub taken: u32,
    pub answered: Option<u32>,
}

/// What an uploader asks of an escrow. Every price is theirs (ADR-095 decision 4): nothing here suggests one.
#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EscrowTerms {
    pub root: String,
    pub size: u64,
    pub copies: u8,
    /// Sparks, as decimal strings: a `u128` does not survive JavaScript's `number`.
    pub storage_price_sparks: String,
    pub serving_price_per_mib_sparks: String,
    pub serving_cap_per_period: u64,
    pub deposit_sparks: String,
}

#[derive(Debug, DecodeAsType)]
struct EscrowRead {
    uploader: AccountId32,
    root: [u8; 32],
    size: u64,
    copies: u8,
    storage_price: u128,
    serving_price_per_mib: u128,
    serving_cap_per_period: u64,
    assigned: u8,
}

#[derive(Debug, DecodeAsType)]
struct AssignmentRead {
    taken: u32,
    answered: Option<u32>,
}

fn escrow_of(id: u64, read: EscrowRead) -> Escrow {
    Escrow {
        id,
        uploader: read.uploader.0,
        root: read.root,
        size: read.size,
        copies: read.copies,
        storage_price: read.storage_price,
        serving_price_per_mib: read.serving_price_per_mib,
        serving_cap_per_period: read.serving_cap_per_period,
        assigned: read.assigned,
    }
}

fn read_error(what: &str) -> impl Fn(subxt::error::StorageError) -> QorError + '_ {
    move |e| QorError::Rpc(format!("could not read the Mesh's {what}: {e}"))
}

impl ChainClient {
    /// The chain's genesis hash, which every receipt names.
    pub async fn mesh_genesis(&self) -> QorResult<[u8; 32]> {
        Ok(self.connected().await?.api.genesis_hash().0)
    }

    /// The period the chain is in.
    pub async fn mesh_period(&self) -> QorResult<u32> {
        let bytes = self.mesh_api("MeshApi_period", Vec::new()).await?;
        u32::decode(&mut &bytes[..]).map_err(|e| QorError::Rpc(format!("the period: {e}")))
    }

    /// The period and the group `seeder` must show for `escrow` now, if a window is open and it holds an assignment.
    pub async fn mesh_challenge(
        &self,
        escrow: u64,
        seeder: [u8; 32],
    ) -> QorResult<Option<(u32, u64)>> {
        let bytes = self
            .mesh_api("MeshApi_challenge", (escrow, seeder).encode())
            .await?;
        Option::<(u32, u64)>::decode(&mut &bytes[..])
            .map_err(|e| QorError::Rpc(format!("the challenge: {e}")))
    }

    /// The bytes the chain says a receiver signs: for checking [`receipt_message`] against, never for signing.
    pub async fn mesh_receipt_message_on_chain(&self, receipt: &Receipt) -> QorResult<Vec<u8>> {
        let bytes = self
            .mesh_api("MeshApi_receipt_message", receipt.encode())
            .await?;
        Vec::<u8>::decode(&mut &bytes[..]).map_err(|e| QorError::Rpc(format!("the receipt: {e}")))
    }

    async fn mesh_api(&self, function: &str, args: Vec<u8>) -> QorResult<Vec<u8>> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        at.runtime_apis()
            .call_raw(function, Some(&args))
            .await
            .map_err(|e| QorError::Rpc(format!("the chain did not answer {function}: {e}")))
    }

    /// Every escrow the chain holds, in order.
    pub async fn mesh_escrows(&self) -> QorResult<Vec<Escrow>> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let mut entries = at
            .storage()
            .iter(
                dynamic::storage::<(u64,), EscrowRead>("Mesh", "Escrows"),
                (),
            )
            .await
            .map_err(read_error("escrows"))?;
        let mut escrows = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(read_error("escrows"))?;
            let id = entry
                .key()
                .ok()
                .and_then(|k| k.part(0)?.decode_as::<u64>().ok().flatten())
                .ok_or_else(|| QorError::Rpc("an escrow's number could not be read".into()))?;
            let read = entry
                .value()
                .decode_as::<EscrowRead>()
                .map_err(|e| QorError::Rpc(format!("escrow {id} could not be decoded: {e}")))?;
            escrows.push(escrow_of(id, read));
        }
        escrows.sort_by_key(|e| e.id);
        Ok(escrows)
    }

    /// The seeders holding `escrow`, with their assignments.
    pub async fn mesh_seeders(&self, escrow: u64) -> QorResult<Vec<([u8; 32], Assignment)>> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let mut entries = at
            .storage()
            .iter(
                dynamic::storage::<(u64, AccountId32), AssignmentRead>("Mesh", "Assignments"),
                (escrow,),
            )
            .await
            .map_err(read_error("assignments"))?;
        let mut seeders = Vec::new();
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(read_error("assignments"))?;
            let seeder = entry
                .key()
                .ok()
                .and_then(|k| k.part(1)?.decode_as::<AccountId32>().ok().flatten())
                .ok_or_else(|| QorError::Rpc("a seeder's account could not be read".into()))?;
            let read = entry
                .value()
                .decode_as::<AssignmentRead>()
                .map_err(|e| QorError::Rpc(format!("an assignment could not be decoded: {e}")))?;
            seeders.push((
                seeder.0,
                Assignment {
                    taken: read.taken,
                    answered: read.answered,
                },
            ));
        }
        Ok(seeders)
    }

    /// `seeder`'s assignment on `escrow`, if it holds one.
    pub async fn mesh_assignment(
        &self,
        escrow: u64,
        seeder: [u8; 32],
    ) -> QorResult<Option<Assignment>> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let Some(value) = at
            .storage()
            .try_fetch(
                dynamic::storage::<(u64, AccountId32), AssignmentRead>("Mesh", "Assignments"),
                (escrow, AccountId32(seeder)),
            )
            .await
            .map_err(read_error("assignment"))?
        else {
            return Ok(None);
        };
        let read = value
            .decode()
            .map_err(|e| QorError::Rpc(format!("an assignment could not be decoded: {e}")))?;
        Ok(Some(Assignment {
            taken: read.taken,
            answered: read.answered,
        }))
    }

    /// The Mesh node `account` bound, if any.
    pub async fn mesh_node(&self, account: [u8; 32]) -> QorResult<Option<[u8; 32]>> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let Some(value) = at
            .storage()
            .try_fetch(
                dynamic::storage::<(AccountId32,), [u8; 32]>("Mesh", "Nodes"),
                (AccountId32(account),),
            )
            .await
            .map_err(read_error("nodes"))?
        else {
            return Ok(None);
        };
        value
            .decode()
            .map(Some)
            .map_err(|e| QorError::Rpc(format!("a node could not be decoded: {e}")))
    }

    /// Whether `root` is on the takedown list.
    pub async fn mesh_taken_down(&self, root: [u8; 32]) -> QorResult<bool> {
        let connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        Ok(at
            .storage()
            .try_fetch(
                dynamic::storage::<([u8; 32],), ()>("Mesh", "Takedowns"),
                (root,),
            )
            .await
            .map_err(read_error("takedown list"))?
            .is_some())
    }

    /// Sign `call` under the Mesh grant, submit it and wait for it to finalise and succeed (ADR-097).
    ///
    /// The call's bytes are the launcher's own; the node's metadata must decode them as the Mesh call they are and encode
    /// that call back to the same bytes, or nothing is signed.
    pub async fn mesh_submit(
        &self,
        vault: &Vault,
        grant: &MeshGrant,
        call: &Granted,
    ) -> QorResult<Done> {
        let what = call.what();
        let connection = self.connected().await?;
        let signer = AccountId32(grant.account());
        let from = address_of(&grant.account());
        if self.balance(&from).await? == 0 {
            return Err(QorError::Unfunded(format!(
                "{from} holds nothing on {chain}, so it cannot send a {what}. Fund this account on {chain}.",
                chain = connection.chain_name,
            )));
        }

        let expected = call.call_data();
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let refused = |why: String| {
            QorError::Rpc(format!(
                "{} does not describe the Mesh as this launcher knows it ({why}), so nothing was signed. A launcher \
                 update may be needed.",
                connection.endpoint
            ))
        };
        let payload = at
            .transactions()
            .from_call_data_bytes(&expected)
            .map_err(|e| refused(e.to_string()))?;
        let (_, name) = call.index_and_name();
        if payload.pallet_name() != "Mesh" || payload.call_name() != name {
            return Err(refused(format!(
                "it names {}.{}",
                payload.pallet_name(),
                payload.call_name()
            )));
        }
        let again = at
            .transactions()
            .call_data(&payload)
            .map_err(|e| refused(e.to_string()))?;
        if again != expected {
            return Err(refused("it encodes the call differently".into()));
        }

        let nonce = connection
            .rpc
            .system_account_next_index(&signer)
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the account's nonce: {e}")))?;
        let params = DefaultExtrinsicParamsBuilder::<DemiurgeConfig>::new()
            .nonce(nonce)
            .build();
        let mut signable = at
            .transactions()
            .create_signable(&payload, &signer, params)
            .await
            .map_err(|e| QorError::Rpc(format!("the {what} could not be built: {e}")))?;
        let to_sign = signable
            .signer_payload()
            .map_err(|e| QorError::Rpc(format!("the {what} could not be prepared: {e}")))?;
        // Up to 256 bytes the call is signed as it stands, so it must lead; above, subxt hashes the same encoding.
        if to_sign.len() > 32 && !to_sign.starts_with(&expected) {
            return Err(refused("what it asks to sign is not the call".into()));
        }
        let signature = vault.sign_granted(grant, &to_sign)?;
        let submittable = signable
            .sign_with_account_and_signature(&signer, &MultiSignature::Sr25519(signature))
            .map_err(|e| QorError::Rpc(format!("the {what} could not be signed: {e}")))?;
        let tx_hash = format!("{:?}", submittable.hash());
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
                        "the {what} {tx_hash} has not finalised within {} seconds",
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
        // What the chain paid the signer, from its own events: an answer's storage price, or each receipt's share.
        let mut paid = 0u128;
        for event in events.iter().filter_map(Result::ok) {
            if event.pallet_name() != "Mesh" {
                continue;
            }
            let mut fields = event.field_bytes();
            let amount = match event.event_name() {
                "ChallengeAnswered" => {
                    <(u64, [u8; 32], u32, u128)>::decode(&mut fields).map(|f| f.3)
                }
                "ServingPaid" => {
                    <(u64, [u8; 32], [u8; 32], u32, u64, u128)>::decode(&mut fields).map(|f| f.5)
                }
                _ => continue,
            }
            .map_err(|e| QorError::Rpc(format!("the {what}'s payment could not be read: {e}")))?;
            paid = paid
                .checked_add(amount)
                .ok_or_else(|| QorError::Rpc(format!("the {what}'s payments overflow")))?;
        }
        Ok(Done {
            tx_hash,
            block_hash,
            paid,
        })
    }

    /// Open an escrow for one's own published work, at one's own prices, after the person approves it in a dialog.
    /// Returns the escrow's number.
    pub async fn mesh_open_escrow(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        terms: &EscrowTerms,
    ) -> QorResult<u64> {
        let root = parse_root(&terms.root)?;
        let sparks = |v: &str, what: &str| {
            // Integer Sparks, never rounded (AGENTS.md §5).
            v.trim().parse::<u128>().map_err(|_| {
                QorError::BadAmount(format!("the {what} is not a whole number of Sparks"))
            })
        };
        let storage_price = sparks(&terms.storage_price_sparks, "storage price")?;
        let serving_price = sparks(&terms.serving_price_per_mib_sparks, "serving price")?;
        let deposit = sparks(&terms.deposit_sparks, "deposit")?;
        if terms.size == 0 || terms.copies == 0 {
            return Err(QorError::Mesh(
                "An escrow holds a file of at least one byte, for at least one copy.".into(),
            ));
        }
        normalise_address(from)?;
        let from_address = canonical_address(from)?;

        let call = dynamic::transaction(
            "Mesh",
            "open_escrow",
            (
                root,
                terms.size,
                terms.copies,
                storage_price,
                serving_price,
                terms.serving_cap_per_period,
                deposit,
            ),
        );
        let connection = self.connected().await?;
        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    escrow_prompt(
                        &from_address,
                        terms,
                        storage_price,
                        serving_price,
                        deposit,
                        chain_name,
                        endpoint,
                    )
                },
                || Ok(()),
                "Mesh escrow",
            )
            .await?;
        for event in finalised.events.iter().filter_map(Result::ok) {
            if event.pallet_name() == "Mesh" && event.event_name() == "EscrowOpened" {
                let id = u64::decode(&mut event.field_bytes())
                    .map_err(|e| QorError::Rpc(format!("the escrow's number: {e}")))?;
                return Ok(id);
            }
        }
        Err(QorError::Rpc(format!(
            "the escrow {} was finalised but the chain reported no EscrowOpened event",
            finalised.tx_hash
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn escrow_prompt(
    from: &str,
    terms: &EscrowTerms,
    storage_price: u128,
    serving_price: u128,
    deposit: u128,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    use crate::cgt::format_cgt;
    Prompt {
        title: "Pay seeders to hold this file?".into(),
        body: format!(
            "From {from}, {deposit} goes into an escrow for file {root} ({size} bytes), on {chain_name} ({endpoint}).\n\n\
             Up to {copies} seeders are paid {storage} each time they prove they hold it, and {serving} per MiB they \
             serve, for at most {cap} bytes each hour. What is left comes back when you close the escrow.\n\n\
             Two people working together can draw on it by downloading from each other, up to that limit.",
            deposit = format_cgt(deposit),
            root = terms.root,
            size = terms.size,
            copies = terms.copies,
            storage = format_cgt(storage_price),
            serving = format_cgt(serving_price),
            cap = terms.serving_cap_per_period,
        ),
        approve: "Open the escrow".into(),
    }
}

fn parse_root(root: &str) -> QorResult<[u8; 32]> {
    hex::decode(root.trim())
        .ok()
        .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
        .ok_or_else(|| QorError::Mesh("That is not a BLAKE3 root.".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bytes a granted call is signed as are the pallet's index, the call's, and SCALE: pinned here so a change to
    /// the encoding is a change someone sees.
    #[test]
    fn granted_calls_encode_as_the_runtime_decodes_them() {
        assert_eq!(
            Granted::BindNode { node: [7; 32] }.call_data(),
            [&[12u8, 0][..], &[7; 32]].concat()
        );
        assert_eq!(
            Granted::TakeAssignment { escrow: 3 }.call_data(),
            vec![12, 4, 3, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            Granted::LeaveAssignment { escrow: 3 }.call_data()[..2],
            [12, 5]
        );
        let answer = Granted::AnswerChallenge {
            escrow: 1,
            group: vec![9; 3],
            proof: vec![[8; 32]],
        }
        .call_data();
        assert_eq!(&answer[..10], &[12, 7, 1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(&answer[10..14], &[3 << 2, 9, 9, 9]);
        assert_eq!(answer[14], 1 << 2);
        assert_eq!(answer.len(), 15 + 32);

        let receipt = Receipt {
            escrow: 2,
            period: 5,
            seeder: [1; 32],
            receiver: [2; 32],
            bytes: 1000,
        };
        let claim = Granted::SubmitReceipts {
            receipts: vec![(receipt.clone(), [4; 64])],
        }
        .call_data();
        assert_eq!(&claim[..3], &[12, 8, 1 << 2]);
        assert_eq!(&claim[3..3 + 84], &receipt.encode()[..]);
        assert_eq!(
            claim[3 + 84],
            1,
            "Sr25519 is the signature's second variant"
        );
        assert_eq!(claim.len(), 3 + 84 + 1 + 64);
    }

    /// A receipt's message starts with its context, so no signature made for a receipt is a transaction's: a
    /// transaction's payload starts with a pallet's index, and the runtime has no pallet 96.
    #[test]
    fn a_receipt_message_is_its_context_the_genesis_and_the_receipt() {
        let receipt = Receipt {
            escrow: 1,
            period: 2,
            seeder: [3; 32],
            receiver: [4; 32],
            bytes: 5,
        };
        let message = receipt_message(&[9; 32], &receipt);
        assert_eq!(message[0], (RECEIPT_CONTEXT.len() as u8) << 2);
        assert_eq!(&message[1..25], RECEIPT_CONTEXT);
        assert_eq!(&message[25..57], &[9; 32]);
        assert_eq!(&message[57..], &receipt.encode()[..]);
        assert_eq!(message[0], 96);
    }
}
