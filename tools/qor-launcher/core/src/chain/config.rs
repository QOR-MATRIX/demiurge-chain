//! The `subxt` configuration for the Demiurge chain.
//!
//! Every value here is the Substrate default: `MultiAddress` for the address
//! (ADR-041), `MultiSignature` (the runtime's `Signature`, which is what lets
//! it accept the vault's Sr25519, ADR-023), Blake2-256, and the transaction
//! extensions resolved from the metadata the connected node serves.
//!
//! It is still written out rather than using `SubstrateConfig` or
//! `PolkadotConfig`, for two reasons that are worth keeping: this chain has no
//! transaction payment at all, so its `AssetId` is `()` rather than `u32`, and
//! a configuration that names its own types fails to compile if the runtime
//! moves away from them, where a borrowed one would keep compiling and start
//! producing transactions the node refuses.
//!
//! **Until 20 September 2026 the runtime used `IdentityLookup`**, so the
//! address was a bare `AccountId32` and a client assuming `MultiAddress` had
//! every transaction refused. ADR-041 moved the runtime to the ecosystem's
//! shape. A node built before that commit will refuse what this signs, which is
//! the same failure in the other direction and is why the runtime and this file
//! changed together.

use subxt::config::substrate::{BlakeTwo256, SubstrateHeader};
use subxt::config::{Config, DefaultTransactionExtensions};
use subxt::utils::{AccountId32, MultiAddress, MultiSignature, H256};

/// The chain this launcher talks to.
#[derive(Clone, Debug, Default)]
pub struct DemiurgeConfig;

impl Config for DemiurgeConfig {
    type AccountId = AccountId32;
    /// `AccountIdLookup`: the ecosystem's shape (ADR-041).
    type Address = MultiAddress<AccountId32, ()>;
    type Signature = MultiSignature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<H256>;
    type TransactionExtensions = DefaultTransactionExtensions<Self>;
    /// The runtime has no `ChargeAssetTxPayment`: there is no transaction
    /// payment at all yet, because a `WeightToFee` is a fee decision and fee
    /// classes are OPEN-4 (`chain/README.md`).
    type AssetId = ();
}

/// The same configuration, as the RPC layer wants it.
pub type DemiurgeRpcConfig = subxt::config::RpcConfigFor<DemiurgeConfig>;

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::config::Header as _;

    /// The conversion `subxt` itself makes when it encodes an extrinsic's
    /// address field, so this exercises the real path rather than a hand-made
    /// value.
    fn address_of<T: Config>(account: T::AccountId) -> T::Address {
        account.into()
    }

    /// The thing about this configuration that would fail silently if it were
    /// wrong: the address must encode as `MultiAddress::Id`, a zero variant
    /// byte and then the 32 account bytes. A bare account would be 32 bytes and
    /// every transaction signed with it would be refused (ADR-041).
    #[test]
    fn an_address_is_a_multi_address_id() {
        use codec::Encode;

        let account = AccountId32([0x2au8; 32]);
        let address = address_of::<DemiurgeConfig>(account);
        let encoded = address.encode();

        assert_eq!(
            encoded.len(),
            33,
            "AccountIdLookup takes a MultiAddress; a bare account would be 32 bytes"
        );
        assert_eq!(encoded[0], 0, "Id is MultiAddress's first variant");
        assert_eq!(&encoded[1..], &account.0[..]);
    }

    #[test]
    fn the_header_reports_its_number() {
        let header = SubstrateHeader::<H256> {
            parent_hash: H256::zero(),
            number: 7,
            state_root: H256::zero(),
            extrinsics_root: H256::zero(),
            digest: Default::default(),
        };
        assert_eq!(header.number(), 7);
    }
}
