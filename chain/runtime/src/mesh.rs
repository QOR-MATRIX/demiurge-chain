//! The Mesh's hosting escrow in this runtime (ADR-095, ADR-096; `pallet-mesh`).
//!
//! The period, the answer window and the missed-period limit are ADR-096's: engineering values, decided by the owner on
//! 17 October 2026 with MESH_PROOFS.md. No price is set here: every price is the uploader's own, in their escrow.

use frame_support::{
    parameter_types,
    traits::{ConstU32, ConstU8},
};
use polkadot_sdk::*;
use sp_runtime::traits::Verify;

use crate::{AccountId, Balances, Runtime, Signature};

/// Blocks in a period: an hour at six seconds a block (ADR-096 decision 1).
pub const PERIOD: u32 = 600;
/// Blocks after a period's seed block in which its challenge may be answered: ten minutes (ADR-096 decision 1).
pub const ANSWER_WINDOW: u32 = 100;
/// Periods unanswered after which anyone may end an assignment (ADR-096 decision 1).
pub const MAX_MISSED: u32 = 3;

parameter_types! {
    pub const MeshPalletId: frame_support::PalletId = frame_support::PalletId(*b"dmg/mesh");
}

impl pallet_mesh::Config for Runtime {
    /// An escrow holds CGT and pays it, and nothing else.
    type Currency = Balances;
    type PalletId = MeshPalletId;
    type Period = ConstU32<PERIOD>;
    type AnswerWindow = ConstU32<ANSWER_WINDOW>;
    type MaxMissed = ConstU32<MAX_MISSED>;
    /// Engineering bounds: copies an escrow may ask for, and receipts one call submits.
    type MaxCopies = ConstU8<16>;
    type MaxReceipts = ConstU32<64>;
    /// The takedown list is the root origin's (ADR-096 decision 4): the sudo key on the devnet.
    type TakedownOrigin = frame_system::EnsureRoot<AccountId>;
    /// A receipt is signed with the receiving account's own key, as a transaction is.
    type ReceiptSignature = Signature;
    type ReceiptSigner = <Signature as Verify>::Signer;
    type WeightInfo = pallet_mesh::weights::PlaceholderWeight<Runtime>;
}
