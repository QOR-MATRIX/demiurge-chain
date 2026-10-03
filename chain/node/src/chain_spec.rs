//! Chain specifications.
//!
//! # These are development and test networks only
//!
//! There is no mainnet specification here, and there will not be one until the
//! genesis allocation split (OPEN-2) and the issuance rate (OPEN-1) are decided.
//! Every endowment below is a **development** endowment: it exists so a local
//! node is usable, and it is not a genesis allocation of the base supply.
//!
//! The address prefix is 42 and the ticker is `CGT`, from the runtime's own
//! constants (ADR-024, ADR-034), so the node cannot disagree with the runtime
//! about either.

use demiurge_runtime::{denomination, WASM_BINARY};
use polkadot_sdk::*;
use sc_service::{ChainType, Properties};
use sp_keyring::{Ed25519Keyring, Sr25519Keyring};

type AccountId = demiurge_runtime::AccountId;
type AuraId = sp_consensus_aura::sr25519::AuthorityId;
type GrandpaId = sp_consensus_grandpa::AuthorityId;

/// The chain specification type for this chain.
pub type ChainSpec = sc_service::GenericChainSpec;

/// How a development node names itself, so nobody has to guess which chain they
/// reached. `system_chain` answers with the name below, which is how a client
/// confirms what it is talking to (see `chain/README.md`).
pub const DEV_CHAIN_ID: &str = "demiurge_dev";
pub const DEV_CHAIN_NAME: &str = "Demiurge Development";
pub const LOCAL_CHAIN_ID: &str = "demiurge_local";
pub const LOCAL_CHAIN_NAME: &str = "Demiurge Local Testnet";

/// What wallets and explorers read to display amounts: the unit and prefix the
/// runtime itself defines.
fn properties() -> Properties {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), denomination::TOKEN_SYMBOL.into());
    properties.insert("tokenDecimals".into(), denomination::DECIMALS.into());
    properties.insert("ss58Format".into(), denomination::SS58_PREFIX.into());
    properties
}

/// A single-validator development chain: Alice authors and finalises.
pub fn development_chain_spec() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "the development runtime wasm is not available".to_string())?,
        None,
    )
    .with_name(DEV_CHAIN_NAME)
    .with_id(DEV_CHAIN_ID)
    .with_chain_type(ChainType::Development)
    .with_properties(properties())
    .with_genesis_config_patch(genesis(
        // The account identifies the validator (ADR-018); Aura authors with an
        // Sr25519 key and GRANDPA votes with an Ed25519 one.
        vec![(
            Sr25519Keyring::Alice.to_account_id(),
            Sr25519Keyring::Alice.public().into(),
            Ed25519Keyring::Alice.public().into(),
        )],
        development_endowed_accounts(),
        Sr25519Keyring::Alice.to_account_id(),
    ))
    .build())
}

/// A two-validator local test chain: Alice and Bob.
pub fn local_chain_spec() -> Result<ChainSpec, String> {
    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "the development runtime wasm is not available".to_string())?,
        None,
    )
    .with_name(LOCAL_CHAIN_NAME)
    .with_id(LOCAL_CHAIN_ID)
    .with_chain_type(ChainType::Local)
    .with_properties(properties())
    .with_genesis_config_patch(genesis(
        vec![
            (
                Sr25519Keyring::Alice.to_account_id(),
                Sr25519Keyring::Alice.public().into(),
                Ed25519Keyring::Alice.public().into(),
            ),
            (
                Sr25519Keyring::Bob.to_account_id(),
                Sr25519Keyring::Bob.public().into(),
                Ed25519Keyring::Bob.public().into(),
            ),
        ],
        development_endowed_accounts(),
        Sr25519Keyring::Alice.to_account_id(),
    ))
    .build())
}

/// The well-known development accounts, endowed so a local node is usable.
///
/// **Not a genesis allocation.** The base supply's split is OPEN-2 and is not
/// invented here; these are the SDK's public test accounts, whose keys everyone
/// has, on a network that holds nothing.
fn development_endowed_accounts() -> Vec<(AccountId, u128)> {
    Sr25519Keyring::well_known()
        .map(|k| (k.to_account_id(), DEVELOPMENT_ENDOWMENT))
        .collect()
}

/// A development endowment per account.
///
/// **A placeholder, clearly marked**, not a decided value: it is large enough to
/// be useful on a network whose keys are public, and it says nothing about the
/// base supply or its split (OPEN-2). It is far below the base supply so that
/// no development genesis can be mistaken for a real one.
///
/// The devnet's faucet holds the same placeholder (`devnet_chain_spec`): the
/// owner decided on 3 October 2026 to reuse this value rather than name a new one.
const DEVELOPMENT_ENDOWMENT: u128 = 1_000_000 * denomination::CGT;

// Until the devnet's public keys exist, nothing outside the tests calls
// `devnet_chain_spec`: a built-in `--chain demiurge_devnet` is wired in
// `command.rs` once they are known (the plan's §6 step 8), and these `expect`s
// then fail the build, so they cannot outlive their reason.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired once the devnet's keys exist")
)]
pub const DEVNET_CHAIN_ID: &str = "demiurge_devnet";
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired once the devnet's keys exist")
)]
pub const DEVNET_CHAIN_NAME: &str = "Demiurge Devnet";

/// The hosted devnet (`docs/architecture/DEVNET_PLAN.md`): validators, a sudo
/// account and a faucet account whose keys are **not** well-known.
///
/// Public keys and addresses only. The validators' keys are made on their own
/// disks at first boot (`chain/docker/entrypoint.sh`) and the sudo account in the
/// owner's wallet, so this function takes them as arguments; no built-in
/// `--chain demiurge_devnet` exists until those public values are known (the
/// plan's §1.4 step order).
///
/// Genesis, as the owner decided on 3 October 2026, and nothing else:
///
/// - the **faucet** holds `DEVELOPMENT_ENDOWMENT` of test CGT. **A placeholder,
///   marked exactly as the development endowment is**: it is the same value, and
///   it is not a genesis allocation of the base supply (OPEN-2);
/// - the **sudo** account holds exactly the existential deposit, so that it
///   exists and can sign (the chain charges no fees);
/// - the validators hold nothing.
///
/// Built with the `sudo` feature, like every development and test network
/// (ADR-037). Without it the sudo account is endowed but is not the root key.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "wired once the devnet's keys exist")
)]
pub fn devnet_chain_spec(
    validators: &[(AuraId, GrandpaId)],
    sudo: AccountId,
    faucet: AccountId,
) -> Result<ChainSpec, String> {
    if validators.is_empty() {
        return Err("a devnet needs at least one validator".into());
    }
    for (i, (aura, _)) in validators.iter().enumerate() {
        if validators[..i].iter().any(|(earlier, _)| earlier == aura) {
            return Err(format!("validator {aura} is listed twice"));
        }
    }
    if sudo == faucet {
        return Err("the sudo account and the faucet account must be different accounts".into());
    }

    // The account identifies the validator (ADR-018). A validator's account is
    // its Aura key's, as on the development chains; it holds no balance.
    let authorities = validators
        .iter()
        .map(|(aura, grandpa)| {
            let account: AccountId = aura.clone().into_inner().into();
            (account, aura.clone(), grandpa.clone())
        })
        .collect();

    Ok(ChainSpec::builder(
        WASM_BINARY.ok_or_else(|| "the runtime wasm is not available".to_string())?,
        None,
    )
    .with_name(DEVNET_CHAIN_NAME)
    .with_id(DEVNET_CHAIN_ID)
    .with_chain_type(ChainType::Live)
    .with_properties(properties())
    .with_genesis_config_patch(genesis(
        authorities,
        vec![
            (faucet, DEVELOPMENT_ENDOWMENT),
            (sudo.clone(), denomination::EXISTENTIAL_DEPOSIT),
        ],
        sudo,
    ))
    .build())
}

fn genesis(
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    balances: Vec<(AccountId, u128)>,
    root: AccountId,
) -> serde_json::Value {
    // Aura and GRANDPA take their authorities from `pallet-session`, which takes
    // the set from `pallet-validator-set` (ADR-020). So their own genesis lists
    // stay empty: setting both would be two sources of truth for who authors.
    let session_keys: Vec<_> = initial_authorities
        .iter()
        .map(|(account, aura, grandpa)| {
            serde_json::json!([
                account,
                account,
                { "aura": aura, "grandpa": grandpa },
            ])
        })
        .collect();

    let validators: Vec<_> = initial_authorities
        .iter()
        .map(|(account, _, _)| account.clone())
        .collect();

    let patch = serde_json::json!({
        "balances": { "balances": balances },
        "session": { "keys": session_keys },
        "validatorSet": { "validators": validators },
    });

    // Only a runtime built with the `sudo` feature has this key, and only
    // development and test networks are (ADR-037). Written as two shadowing
    // bindings so neither configuration carries an unused `mut` or argument:
    // both are built, so both must compile clean.
    #[cfg(feature = "sudo")]
    let patch = {
        let mut patch = patch;
        patch["sudo"] = serde_json::json!({ "key": Some(root) });
        patch
    };
    #[cfg(not(feature = "sudo"))]
    let _ = root;

    patch
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The specification tells a person which chain they are on. Every
    /// Substrate development node looks alike on port 9944, so the identifiers
    /// must not be vague: `system_chain` is what a client asks, and the launcher
    /// asks automatically (ADR-040).
    #[test]
    fn the_development_chain_says_which_chain_it_is() {
        let spec = development_chain_spec().expect("the development spec builds");
        assert_eq!(spec.id(), DEV_CHAIN_ID);
        assert_eq!(spec.name(), DEV_CHAIN_NAME);
        assert!(spec.id().starts_with("demiurge"));
    }

    /// The unit a wallet displays comes from the runtime, so the node cannot
    /// disagree with the runtime about the ticker, the decimals or the prefix.
    #[test]
    fn the_properties_come_from_the_runtime() {
        let props = properties();
        assert_eq!(
            props.get("tokenSymbol").and_then(|v| v.as_str()),
            Some(denomination::TOKEN_SYMBOL)
        );
        assert_eq!(
            props.get("tokenDecimals").and_then(|v| v.as_u64()),
            Some(denomination::DECIMALS as u64)
        );
        assert_eq!(
            props.get("ss58Format").and_then(|v| v.as_u64()),
            Some(denomination::SS58_PREFIX as u64)
        );
    }

    /// A development endowment is not a genesis allocation. If one ever
    /// approached the base supply, somebody would be inventing OPEN-2.
    #[test]
    fn a_development_endowment_is_nowhere_near_the_base_supply() {
        const BASE_SUPPLY: u128 = 100_000_000_000_000 * denomination::CGT;
        let handed_out = DEVELOPMENT_ENDOWMENT * Sr25519Keyring::well_known().count() as u128;
        assert!(
            handed_out * 1_000_000 < BASE_SUPPLY,
            "development genesis hands out {handed_out} Sparks, too close to the base supply to be \
             obviously not a genesis allocation"
        );
    }

    // ---- The devnet --------------------------------------------------------

    use sc_service::ChainSpec as _;
    use sp_core::{ed25519, sr25519, Pair};

    /// Validators made from fixed seeds that no keyring uses, so these tests are
    /// deterministic and their keys are not well-known.
    fn test_validators() -> Vec<(AuraId, GrandpaId)> {
        [[0x11u8; 32], [0x22u8; 32]]
            .iter()
            .map(|seed| {
                (
                    sr25519::Pair::from_seed(seed).public().into(),
                    ed25519::Pair::from_seed(seed).public().into(),
                )
            })
            .collect()
    }

    fn test_sudo() -> AccountId {
        AccountId::new([0x5d; 32])
    }

    fn test_faucet() -> AccountId {
        AccountId::new([0xfa; 32])
    }

    fn test_devnet() -> ChainSpec {
        devnet_chain_spec(&test_validators(), test_sudo(), test_faucet())
            .expect("the devnet spec builds")
    }

    /// The genesis patch the specification carries, as JSON.
    fn patch_of(spec: &ChainSpec) -> serde_json::Value {
        let json: serde_json::Value =
            serde_json::from_str(&spec.as_json(false).expect("the spec serialises"))
                .expect("the spec is JSON");
        json["genesis"]["runtimeGenesis"]["patch"].clone()
    }

    /// `Balances::TotalIssuance` as the runtime's genesis build writes it.
    fn total_issuance_at_genesis(spec: &ChainSpec) -> u128 {
        use sp_runtime::BuildStorage;
        let storage = spec.build_storage().expect("the devnet genesis builds");
        let key = [
            sp_io::hashing::twox_128(b"Balances"),
            sp_io::hashing::twox_128(b"TotalIssuance"),
        ]
        .concat();
        let raw = storage
            .top
            .get(&key)
            .expect("total issuance is written at genesis");
        u128::from_le_bytes(raw[..].try_into().expect("a u128"))
    }

    /// The name, id and type the owner approved on 3 October 2026. `system_chain`
    /// answers with the name, and `Live` keeps `dev-fund.mjs` from treating it as
    /// a development chain.
    #[test]
    fn the_devnet_says_which_chain_it_is() {
        let spec = test_devnet();
        assert_eq!(spec.name(), "Demiurge Devnet");
        assert_eq!(spec.id(), "demiurge_devnet");
        assert_eq!(spec.chain_type(), ChainType::Live);
    }

    /// Exactly two balances: the faucet's placeholder and the sudo account's
    /// existential deposit. Nothing else holds CGT, and the runtime's own genesis
    /// build agrees on the total.
    #[test]
    fn the_devnet_endows_the_faucet_and_the_sudo_account_and_nothing_else() {
        let spec = test_devnet();
        assert_eq!(
            patch_of(&spec)["balances"]["balances"],
            serde_json::json!([
                [serde_json::json!(test_faucet()), DEVELOPMENT_ENDOWMENT],
                [
                    serde_json::json!(test_sudo()),
                    denomination::EXISTENTIAL_DEPOSIT
                ],
            ])
        );
        assert_eq!(
            total_issuance_at_genesis(&spec),
            DEVELOPMENT_ENDOWMENT + denomination::EXISTENTIAL_DEPOSIT
        );
    }

    /// The validators are the keys passed in, in order, each identified by its
    /// Aura key's account; the sudo key is the account passed in.
    #[test]
    fn the_devnet_uses_the_validators_and_sudo_account_it_is_given() {
        let validators = test_validators();
        let patch = patch_of(&test_devnet());

        let accounts: Vec<AccountId> = validators
            .iter()
            .map(|(aura, _)| aura.clone().into_inner().into())
            .collect();
        assert_eq!(
            patch["validatorSet"]["validators"],
            serde_json::json!(accounts)
        );

        let keys: Vec<_> = validators
            .iter()
            .zip(&accounts)
            .map(|((aura, grandpa), account)| {
                serde_json::json!([account, account, { "aura": aura, "grandpa": grandpa }])
            })
            .collect();
        assert_eq!(patch["session"]["keys"], serde_json::Value::Array(keys));

        #[cfg(feature = "sudo")]
        assert_eq!(patch["sudo"]["key"], serde_json::json!(test_sudo()));
        #[cfg(not(feature = "sudo"))]
        assert!(patch.get("sudo").is_none());
    }

    #[test]
    fn the_devnet_refuses_a_specification_it_cannot_run() {
        let validators = test_validators();
        assert!(devnet_chain_spec(&[], test_sudo(), test_faucet()).is_err());
        assert!(devnet_chain_spec(
            &[validators[0].clone(), validators[0].clone()],
            test_sudo(),
            test_faucet()
        )
        .is_err());
        assert!(devnet_chain_spec(&validators, test_sudo(), test_sudo()).is_err());
    }

    /// A public network must not contain a key everyone has. Every SDK keyring
    /// account, Sr25519 and Ed25519, stash accounts included, is looked for by its
    /// address and by its public key in hex, in the readable specification and in
    /// the raw one, whose storage keys carry account bytes.
    #[test]
    fn no_well_known_key_appears_in_the_devnet() {
        let spec = test_devnet();
        let readable = spec.as_json(false).expect("serialises");
        let raw = spec.as_json(true).expect("serialises raw");

        let sr = Sr25519Keyring::iter().map(|k| {
            let public = k.public();
            (format!("{k:?}"), public.to_string(), hex(public.as_ref()))
        });
        let ed = Ed25519Keyring::iter().map(|k| {
            let public = k.public();
            (format!("{k:?}"), public.to_string(), hex(public.as_ref()))
        });
        for (name, address, public_hex) in sr.chain(ed) {
            for (form, text) in [("readable", &readable), ("raw", &raw)] {
                assert!(
                    !text.contains(&address),
                    "{name}'s address is in the {form} devnet spec"
                );
                assert!(
                    !text.contains(&public_hex),
                    "{name}'s key is in the {form} devnet spec"
                );
            }
        }
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn the_local_chain_is_distinct_from_the_development_chain() {
        let dev = development_chain_spec().expect("dev spec builds");
        let local = local_chain_spec().expect("local spec builds");
        assert_ne!(dev.id(), local.id());
        assert_ne!(dev.name(), local.name());
    }
}
