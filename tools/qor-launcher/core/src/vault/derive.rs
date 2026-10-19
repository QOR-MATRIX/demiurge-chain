//! BIP-39 recovery phrases and Sr25519 keys, derived the way the ecosystem
//! derives them (ADR-023).
//!
//! # Why not the launcher's old scheme
//!
//! The vault used to derive Ed25519 keys by SLIP-0010 at
//! `m/44'/369'/account'/0'/index'`. Nothing else in the ecosystem opens those
//! accounts from a recovery phrase: SLIP-0010 starts from the phrase's *seed*,
//! while `sp-core` and Polkadot.js start from its *entropy*. A phrase imported
//! into Polkadot.js, Talisman or Nova would therefore show a different account
//! from the one the launcher showed, which defeats ADR-009: the same phrase must
//! open the same accounts everywhere.
//!
//! # The scheme, exactly
//!
//! [`sp_core::sr25519::Pair`] at the pinned SDK release (ADR-022, ADR-033
//! rule 1), so the launcher, the chain and QOR ID agree byte for byte:
//!
//! - **Root.** PBKDF2-HMAC-SHA512 over the phrase's entropy, salt `"mnemonic"`
//!   followed by the password, 2,048 rounds, first 32 bytes as the Sr25519 mini
//!   secret (`substrate-bip39`). `Pair::from_phrase` does exactly this.
//! - **The first account is the phrase with no derivation path**, which is the
//!   account every ecosystem wallet opens when the phrase is imported.
//! - **Further accounts are the hard junctions `//0`, `//1`, …** (ADR-023 left
//!   the form to this work; the choice and its reasons are in ADR-039). They are
//!   built from the path string a person would type, so the account the launcher
//!   shows second is the one `<phrase>//0` opens in subkey, Polkadot.js or
//!   Talisman.
//! - **The vault's optional BIP-39 passphrase is the Substrate password**, the
//!   `///password` of a secret URI.
//!
//! # What a key signs
//!
//! `Pair::sign` signs under schnorrkel's signing context `b"substrate"`, the
//! context the runtime's `MultiSignature` verifies under. The launcher never
//! signs raw bytes under any other context.

use sp_core::crypto::{DeriveJunction, Ss58AddressFormat, Ss58Codec};
use sp_core::sr25519;
use sp_core::Pair as _;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{QorError, QorResult};

/// The address prefix a person sees (ADR-024).
///
/// 42 is the generic Substrate prefix, which is what development and test
/// networks use; it is also the runtime's own `SS58Prefix`
/// (`chain/runtime/src/denomination.rs`), so the launcher and the chain display
/// the same string for the same account. **It does not identify Demiurge**:
/// many chains share 42, and what tells networks apart is the genesis hash.
/// The mainnet prefix is a Public Release criterion and is not decided, so
/// there is one constant here rather than a guess at a second one. Once the
/// chain client is rebuilt (roadmap L3.1) the launcher reads the prefix from
/// the node's `system_properties` and this becomes the offline fallback.
pub const SS58_PREFIX: u16 = 42;

/// The SS58 format the launcher encodes and accepts.
pub fn ss58_format() -> Ss58AddressFormat {
    Ss58AddressFormat::custom(SS58_PREFIX)
}

/// The root key of a recovery phrase: the account the phrase itself opens, and
/// the mini secret every further account is derived from.
///
/// The mini secret is zeroized on drop, and `sr25519::Pair` wraps a schnorrkel
/// keypair, which zeroizes its own secret on drop.
#[derive(ZeroizeOnDrop)]
pub struct RootKey {
    #[zeroize(skip)]
    pair: sr25519::Pair,
    seed: [u8; 32],
}

impl RootKey {
    /// Derive the root from a recovery phrase and an optional BIP-39
    /// passphrase (the "25th word"), which is the Substrate password. An empty
    /// passphrase is the common case and is the same as none.
    pub fn from_mnemonic(phrase: &str, passphrase: &str) -> QorResult<Self> {
        // Parsed here as well as by `from_phrase` so that a bad phrase is a
        // `BadMnemonic` the Gate can show, rather than an opaque secret-string
        // error.
        validate_mnemonic(phrase)?;

        let (pair, seed) = sr25519::Pair::from_phrase(phrase.trim(), Some(passphrase))
            .map_err(|e| QorError::BadMnemonic(format!("{e:?}")))?;

        Ok(Self { pair, seed })
    }
}

/// A derived account: its key pair, and the path it came from.
pub struct DerivedAccount {
    pair: sr25519::Pair,
    /// The derivation suffix a person can retype elsewhere: empty for the
    /// first account, `//0`, `//1`, … after it.
    pub path: String,
    pub index: u32,
}

impl DerivedAccount {
    /// The 32-byte account ID. For Sr25519 the account ID *is* the public key,
    /// as `MultiSigner` defines it, so verifying a signature needs no lookup.
    pub fn account_id(&self) -> [u8; 32] {
        self.pair.public().0
    }

    /// The address as a person sees it: SS58 at the chain's prefix (ADR-024).
    pub fn address(&self) -> String {
        self.pair.public().to_ss58check_with_version(ss58_format())
    }

    /// The raw account ID in hex, for advanced views only (ADR-024).
    pub fn account_id_hex(&self) -> String {
        format!("0x{}", hex::encode(self.account_id()))
    }

    /// Sign `message` under the ecosystem's signing context.
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.pair.sign(message).0
    }
}

/// The derivation path for an account index: nothing for the first, then the
/// hard junctions `//0`, `//1`, … after it (ADR-039).
///
/// The offset is not a mistake. It is Talisman's enumeration, read from its
/// source on 19 September 2026 (`packages/.../DerivedFromMnemonicAccountPicker`,
/// "for substrate, first account should have an empty derivation path"), so a
/// phrase imported there lists the launcher's accounts in the same order with
/// none in between. Matching the ecosystem's list is the point of ADR-023.
pub fn path_for(index: u32) -> String {
    if index == 0 {
        String::new()
    } else {
        format!("//{}", index - 1)
    }
}

/// Derive the account at `index` from a phrase's root.
///
/// Index 0 is the root itself, the account an ecosystem wallet opens from the
/// bare phrase. Every later index is one hard junction below it.
pub fn derive_account(root: &RootKey, index: u32) -> QorResult<DerivedAccount> {
    let path = path_for(index);

    let pair = if index == 0 {
        root.pair.clone()
    } else {
        // Built from the path string rather than from a number, so that the
        // junction is encoded exactly as `sp-core` encodes what a person types.
        let junctions = suri_junctions(&path);
        root.pair
            .derive(junctions.into_iter(), Some(root.seed))
            .map_err(|e| QorError::Internal(format!("cannot derive {path}: {e:?}")))?
            .0
    };

    Ok(DerivedAccount { pair, path, index })
}

/// Split a derivation suffix such as `//2` into junctions, the way `sp-core`
/// parses a secret URI's path: `//x` is hard, `/x` is soft, and a purely
/// numeric junction is the number rather than the digits.
fn suri_junctions(path: &str) -> Vec<DeriveJunction> {
    let mut junctions = Vec::new();
    let mut rest = path;

    while let Some(stripped) = rest.strip_prefix('/') {
        let (hard, body) = match stripped.strip_prefix('/') {
            Some(body) => (true, body),
            None => (false, stripped),
        };

        let end = body.find('/').unwrap_or(body.len());
        let (value, remainder) = body.split_at(end);

        // `DeriveJunction::from` reads a leading `/` as "harden this".
        junctions.push(DeriveJunction::from(if hard {
            format!("/{value}")
        } else {
            value.to_string()
        }));

        rest = remainder;
    }

    junctions
}

/// Parse an address a person or another program supplied.
///
/// SS58 at the chain's prefix is the form people see and paste. Hex is accepted
/// too, because advanced views show it (ADR-024) and a pasted account ID should
/// not be a dead end.
pub fn account_id_from_address(address: &str) -> QorResult<[u8; 32]> {
    let trimmed = address.trim();

    if trimmed.is_empty() {
        return Err(QorError::BadAddress("no address given".into()));
    }

    // Hex is unambiguous: SS58 is base58, which has neither `0` nor `x`.
    let body = trimmed.strip_prefix("0x").unwrap_or(trimmed);
    if body.len() == 64 && body.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes = hex::decode(body)
            .map_err(|_| QorError::BadAddress("address is not valid hex".into()))?;
        return bytes
            .try_into()
            .map_err(|_| QorError::BadAddress("address is not 32 bytes".into()));
    }

    let (public, format) = sr25519::Public::from_ss58check_with_version(trimmed)
        .map_err(|e| QorError::BadAddress(format!("not an address this network uses: {e}")))?;

    // A prefix check is an input-shape check, not chain identification
    // (ADR-024, clarified 17 September 2026): it catches a Polkadot or Kusama
    // address pasted by mistake, and nothing more. Many chains use 42.
    if format != ss58_format() {
        return Err(QorError::BadAddress(format!(
            "that address is for another network (prefix {}, this network uses {SS58_PREFIX})",
            u16::from(format)
        )));
    }

    Ok(public.0)
}

/// The SS58 form of a 32-byte account ID, for showing an account the vault does
/// not hold.
pub fn address_of(account_id: &[u8; 32]) -> String {
    sr25519::Public::from(*account_id).to_ss58check_with_version(ss58_format())
}

/// Verify a signature the vault made, under the same context the chain uses.
pub fn verify(account_id: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    sr25519::Pair::verify(
        &sr25519::Signature::from(*signature),
        message,
        &sr25519::Public::from(*account_id),
    )
}

/// Generate a fresh 24-word (256-bit) recovery phrase.
///
/// 24 words rather than 12: this vault custodies an asset intended to carry real
/// monetary value, and the extra 128 bits of entropy costs the holder only
/// twelve more words to write down once.
pub fn generate_mnemonic() -> QorResult<String> {
    let mut entropy = [0u8; 32];
    fill_random(&mut entropy)?;
    let mnemonic = bip39::Mnemonic::from_entropy(&entropy)
        .map_err(|e| QorError::Internal(format!("mnemonic generation failed: {e}")))?;
    entropy.zeroize();
    Ok(mnemonic.to_string())
}

/// Validate a phrase without retaining it.
pub fn validate_mnemonic(phrase: &str) -> QorResult<()> {
    bip39::Mnemonic::parse_normalized(phrase.trim())
        .map(|_| ())
        .map_err(|e| QorError::BadMnemonic(e.to_string()))
}

pub(crate) fn fill_random(buf: &mut [u8]) -> QorResult<()> {
    use rand::RngCore;
    rand::rngs::OsRng
        .try_fill_bytes(buf)
        .map_err(|e| QorError::Internal(format!("system entropy unavailable: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon abandon \
                          abandon abandon abandon about";

    fn root(passphrase: &str) -> RootKey {
        RootKey::from_mnemonic(PHRASE, passphrase).expect("the test phrase is valid")
    }

    /// The whole point of ADR-023: the first account is what any ecosystem
    /// wallet opens from the bare phrase. `Pair::from_string` is the path
    /// `subkey` and Polkadot.js take, so if these ever diverge the launcher has
    /// stopped being interoperable.
    #[test]
    fn the_first_account_is_the_phrase_itself() {
        let ours = derive_account(&root(""), 0).unwrap();
        let theirs = sr25519::Pair::from_string(PHRASE, None).expect("subkey parses the phrase");

        assert_eq!(ours.account_id(), theirs.public().0);
        assert_eq!(ours.path, "", "the first account has no derivation path");
    }

    /// The account at index `n` is what `<phrase>//(n-1)` opens elsewhere.
    /// Built from the same string a person would type, and checked against
    /// `sp-core`'s own parser: `DeriveJunction::hard("1")` encodes the *string*
    /// "1", length prefix and all, which is a different account from `//1`, and
    /// the two are indistinguishable by eye.
    #[test]
    fn further_accounts_are_the_hard_junctions_a_person_can_retype() {
        let root = root("");

        for index in 1..=3u32 {
            let ours = derive_account(&root, index).unwrap();
            let suri = format!("{PHRASE}//{}", index - 1);
            let theirs = sr25519::Pair::from_string(&suri, None).expect("subkey parses the suri");

            assert_eq!(
                ours.account_id(),
                theirs.public().0,
                "account {index} must be what {suri} opens"
            );
            assert_eq!(ours.path, format!("//{}", index - 1));
        }

        // The trap this guards against, stated as a test rather than as a
        // comment: the string junction is not the numeric one.
        let as_string = root
            .pair
            .derive(std::iter::once(DeriveJunction::hard("0")), None)
            .expect("derives")
            .0;
        assert_ne!(
            derive_account(&root, 1).unwrap().account_id(),
            as_string.public().0,
            "//0 must be the number 0, not the text \"0\""
        );
    }

    /// A soft junction would leave the child's public key derivable from the
    /// parent's, which is not what separating accounts is for.
    #[test]
    fn the_junctions_are_hard() {
        let ours = derive_account(&root(""), 1).unwrap();
        let soft = sr25519::Pair::from_string(&format!("{PHRASE}/0"), None).unwrap();

        assert_ne!(
            ours.account_id(),
            soft.public().0,
            "//0 and /0 must not be the same account"
        );
    }

    #[test]
    fn derivation_is_deterministic_and_index_separated() {
        let root = root("");
        let a0 = derive_account(&root, 0).unwrap();
        let again = derive_account(&root, 0).unwrap();
        let a1 = derive_account(&root, 1).unwrap();

        assert_eq!(a0.account_id(), again.account_id(), "must be deterministic");
        assert_ne!(a0.account_id(), a1.account_id(), "index separates keys");
    }

    /// ADR-023: the vault's BIP-39 passphrase is the Substrate password, so a
    /// phrase with a passphrase opens the same accounts everywhere.
    #[test]
    fn the_passphrase_is_the_substrate_password() {
        let with_pass = derive_account(&root("correct horse"), 0).unwrap();
        let plain = derive_account(&root(""), 0).unwrap();

        assert_ne!(
            with_pass.account_id(),
            plain.account_id(),
            "the 25th word must produce a different wallet"
        );

        let theirs = sr25519::Pair::from_string(PHRASE, Some("correct horse")).unwrap();
        assert_eq!(
            with_pass.account_id(),
            theirs.public().0,
            "it must be the password a secret URI's ///password supplies"
        );
    }

    /// The address a person sees, at the prefix the runtime declares.
    #[test]
    fn addresses_are_ss58_at_the_chains_prefix() {
        let account = derive_account(&root(""), 0).unwrap();
        let address = account.address();

        assert!(
            !address.starts_with("0x"),
            "an address is SS58, not hex: {address}"
        );
        assert_eq!(
            account_id_from_address(&address).unwrap(),
            account.account_id(),
            "an address must decode back to the account it names"
        );
        assert_eq!(address, address_of(&account.account_id()));

        let (_, format) =
            sr25519::Public::from_ss58check_with_version(&address).expect("valid SS58");
        assert_eq!(u16::from(format), SS58_PREFIX);
    }

    /// Hex is still accepted, because advanced views show it. Both forms of one
    /// account must reach the same bytes.
    #[test]
    fn hex_and_ss58_reach_the_same_account() {
        let account = derive_account(&root(""), 0).unwrap();
        let hex_form = account.account_id_hex();

        assert_eq!(
            account_id_from_address(&hex_form).unwrap(),
            account_id_from_address(&account.address()).unwrap()
        );
        assert_eq!(
            account_id_from_address(hex_form.trim_start_matches("0x")).unwrap(),
            account.account_id(),
            "bare hex is accepted too"
        );
    }

    /// An address from a network with another prefix is refused, so a Polkadot
    /// or Kusama address pasted into the send field fails here rather than
    /// sending CGT to an account nobody on this network holds.
    #[test]
    fn an_address_from_another_prefix_is_refused() {
        let account = derive_account(&root(""), 0).unwrap();
        let public = sr25519::Public::from(account.account_id());

        for prefix in [0u16, 2, 777] {
            let elsewhere = public.to_ss58check_with_version(Ss58AddressFormat::custom(prefix));
            assert!(
                account_id_from_address(&elsewhere).is_err(),
                "prefix {prefix} must be refused: {elsewhere}"
            );
        }
    }

    /// A single altered character fails the checksum, which is the protection
    /// hex never had.
    #[test]
    fn a_bad_checksum_is_refused() {
        let address = derive_account(&root(""), 0).unwrap().address();
        let mut broken: Vec<char> = address.chars().collect();
        let last = broken.len() - 1;
        broken[last] = if broken[last] == 'A' { 'B' } else { 'A' };
        let broken: String = broken.into_iter().collect();

        assert!(
            account_id_from_address(&broken).is_err(),
            "a mistyped address must be refused: {broken}"
        );

        for bad in ["", "   ", "not an address", "0x1234"] {
            assert!(account_id_from_address(bad).is_err(), "{bad:?}");
        }
    }

    /// Signatures verify under the context the chain verifies under.
    #[test]
    fn a_signature_verifies_against_the_account() {
        let account = derive_account(&root(""), 0).unwrap();
        let message = b"demiurge:qor-id:challenge:v1:test";
        let signature = account.sign(message);

        assert!(verify(&account.account_id(), message, &signature));
        assert!(!verify(
            &account.account_id(),
            b"another message",
            &signature
        ));

        let other = derive_account(&root(""), 1).unwrap();
        assert!(!verify(&other.account_id(), message, &signature));
    }

    #[test]
    fn mnemonic_generation_and_validation() {
        assert!(validate_mnemonic("not actually a real recovery phrase at all").is_err());

        let generated = generate_mnemonic().unwrap();
        assert_eq!(generated.split_whitespace().count(), 24, "24-word phrase");
        assert!(validate_mnemonic(&generated).is_ok());
        assert_ne!(generated, generate_mnemonic().unwrap(), "must not repeat");

        // A generated phrase must also be one the ecosystem's derivation
        // accepts, or a new vault would be unopenable anywhere else.
        let root = RootKey::from_mnemonic(&generated, "").unwrap();
        assert_eq!(
            derive_account(&root, 0).unwrap().account_id(),
            sr25519::Pair::from_string(&generated, None)
                .unwrap()
                .public()
                .0
        );
    }
}
