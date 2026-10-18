//! The CGT Vault: key custody for the launcher.
//!
//! # Design
//!
//! The recovery phrase never leaves this process and never reaches the webview.
//! The UI can ask the vault to *sign* something; it can never ask for the key.
//! That boundary is the whole point of putting custody in the Tauri host rather
//! than in JavaScript, and it is what the browser extension cannot offer, since
//! there the keys live in a service worker sharing an address space with page
//! script.
//!
//! At rest the phrase is sealed with Argon2id and XChaCha20-Poly1305 (`seal`)
//! under a random key kept in the operating system's keychain (`keychain`,
//! ADR-056). Nothing is typed and nothing is asked: the vault opens whenever the
//! person is signed in to their computer, and stays open while the launcher
//! runs. A copied `vault.qor` opens nothing without that account's keychain.
//!
//! Vaults sealed before ADR-056 still open, once, to move to the keychain: one
//! sealed under Windows Hello (ADR-055) with one last Hello gesture, and one
//! sealed with a passphrase with that passphrase ([`Vault::move_to_keychain`]).
//!
//! # What is deliberately not here
//!
//! There is no "export private key" command. There is an export for the recovery
//! phrase, because a user who cannot get their phrase out does not own their
//! money, and it is approved in a host dialog at the moment of export.

pub mod derive;
pub mod hello;
pub mod keychain;
pub mod seal;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};
use crate::{Confirm, Prompt};
use derive::{DerivedAccount, RootKey};
use hello::{HelloVault, Presence};
use keychain::KeyStore;
use seal::{SealedVault, Zeroizing};

/// Vault file name inside the launcher data directory.
const VAULT_FILE: &str = "vault.qor";

/// The value of `vault.qor`'s `unlock` field for a vault whose key is in the keychain.
const UNLOCK_KEYCHAIN: &str = "os-keychain";
const KEYCHAIN_FORMAT: u32 = 3;

/// Public description of an account. Contains no secret material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountView {
    /// SS58 at the chain's prefix: the form a person reads, copies and pastes
    /// (ADR-024).
    pub address: String,
    /// The same account as raw hex, for advanced views only (ADR-024).
    pub account_id: String,
    /// The derivation suffix: empty for the first account, `//0`, `//1`, …
    /// after it (ADR-023, ADR-039).
    pub path: String,
    pub index: u32,
    pub label: String,
}

/// What seals a vault on disk, as the UI is told it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SealedWith {
    /// The keychain (ADR-056): opens with nothing asked.
    Keychain,
    /// Windows Hello (ADR-055): opens once more with Hello, to move.
    Hello,
    /// A passphrase, before ADR-055: opens once more with it, to move.
    Passphrase,
}

/// Vault status as reported to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum VaultStatus {
    /// No vault file on this device.
    Absent,
    /// A vault exists but is sealed.
    Locked { sealed_with: SealedWith },
    /// Open, with the accounts derived so far.
    Unlocked { accounts: Vec<AccountView> },
}

/// The file in `vault.qor` for a vault whose key is in the keychain.
#[derive(Debug, Serialize, Deserialize)]
struct KeychainVault {
    /// Always [`UNLOCK_KEYCHAIN`].
    unlock: String,
    version: u32,
    /// Names the keychain entry that holds the key: [`keychain::entry_name`].
    key: String,
    /// The recovery phrase, sealed under the key.
    sealed: SealedVault,
    /// Which account of the phrase this wallet is (ADR-099: one account per profile). 0, the first, for every wallet
    /// made from a new phrase; another for a profile made from an account an older wallet's phrase had used.
    #[serde(default)]
    index: u32,
}

/// What `vault.qor` holds.
enum OnDisk {
    Keychain(KeychainVault),
    Hello(HelloVault),
    Passphrase(SealedVault),
}

/// Secret state, present only while open.
struct Unlocked {
    root: RootKey,
    accounts: Vec<DerivedAccount>,
    labels: Vec<String>,
    /// The account the person picked in the launcher (the one its rail shows), if not the first (ADR-090).
    active: Option<String>,
}

impl Unlocked {
    fn views(&self) -> Vec<AccountView> {
        self.accounts
            .iter()
            .zip(&self.labels)
            .map(|(a, label)| AccountView {
                address: a.address(),
                account_id: a.account_id_hex(),
                path: a.path.clone(),
                index: a.index,
                label: label.clone(),
            })
            .collect()
    }
}

/// The vault. Cheap to clone by reference; guarded by a mutex.
/// The Mesh's standing approval to sign its own work for one account (ADR-097). Made by [`Vault::grant_mesh`] once the
/// person approves, or restored from the approval the Mesh recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshGrant {
    account: [u8; 32],
}

impl MeshGrant {
    /// An approval the person gave earlier, as the Mesh's settings recorded it.
    pub(crate) fn recorded(account: [u8; 32]) -> Self {
        Self { account }
    }

    pub fn account(&self) -> [u8; 32] {
        self.account
    }
}

pub struct Vault {
    /// The signed-in profile's folder (ADR-099), or none while no profile is signed in.
    dir: parking_lot::RwLock<Option<PathBuf>>,
    keys: Arc<dyn KeyStore>,
    inner: Mutex<Option<Unlocked>>,
}

impl Vault {
    /// A vault in `dir` whose key goes in the operating system's keychain.
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self::with_keys(dir, keychain::system())
    }

    /// A vault whose key goes in `keys`. Tests use an in-memory keychain.
    pub fn with_keys(dir: impl AsRef<Path>, keys: Arc<dyn KeyStore>) -> Self {
        Self {
            dir: parking_lot::RwLock::new(Some(dir.as_ref().to_path_buf())),
            keys,
            inner: Mutex::new(None),
        }
    }

    /// A vault in no folder yet, for when no profile is signed in (ADR-099). Its key goes in `keys`.
    pub fn unplaced(keys: Arc<dyn KeyStore>) -> Self {
        Self {
            dir: parking_lot::RwLock::new(None),
            keys,
            inner: Mutex::new(None),
        }
    }

    /// Point the vault at a profile's folder, or at none. Refused while it is open.
    pub fn place(&self, dir: Option<PathBuf>) -> QorResult<()> {
        if self.is_open() {
            return Err(QorError::VaultOpen);
        }
        *self.dir.write() = dir;
        Ok(())
    }

    fn dir(&self) -> QorResult<PathBuf> {
        self.dir.read().clone().ok_or(QorError::NoProfile)
    }

    /// The vault file, while a profile is signed in.
    pub fn path(&self) -> Option<PathBuf> {
        self.dir.read().as_ref().map(|d| d.join(VAULT_FILE))
    }

    fn file(&self) -> QorResult<PathBuf> {
        Ok(self.dir()?.join(VAULT_FILE))
    }

    pub fn exists(&self) -> bool {
        self.path().is_some_and(|p| p.is_file())
    }

    pub fn status(&self) -> VaultStatus {
        if let Some(session) = self.inner.lock().as_ref() {
            return VaultStatus::Unlocked {
                accounts: session.views(),
            };
        }
        match self.read_disk() {
            _ if !self.exists() => VaultStatus::Absent,
            Ok(OnDisk::Hello(_)) => VaultStatus::Locked {
                sealed_with: SealedWith::Hello,
            },
            Ok(OnDisk::Passphrase(_)) => VaultStatus::Locked {
                sealed_with: SealedWith::Passphrase,
            },
            _ => VaultStatus::Locked {
                sealed_with: SealedWith::Keychain,
            },
        }
    }

    /// Create a vault from a recovery phrase, its key in the keychain.
    ///
    /// Refuses to overwrite an existing vault. Replacing one is a destructive,
    /// funds-losing act and must be an explicit delete first, never a silent
    /// consequence of running setup twice.
    pub fn create(&self, phrase: &str) -> QorResult<Vec<AccountView>> {
        self.create_at(phrase, 0)
    }

    /// Create a vault for the account at `index` of a recovery phrase (ADR-099: a profile's wallet is one account).
    pub fn create_at(&self, phrase: &str, index: u32) -> QorResult<Vec<AccountView>> {
        if self.exists() {
            return Err(QorError::VaultExists);
        }
        self.dir()?;
        derive::validate_mnemonic(phrase)?;
        let phrase = phrase.trim().as_bytes();
        let file = self.seal_to_keychain(phrase, index)?;
        self.write_vault(&file)?;
        self.open_phrase(phrase, index)
    }

    /// The first account a recovery phrase opens, without storing anything.
    ///
    /// A person restoring a vault sees this address before anything on disk
    /// changes, so they can recognise the account as theirs.
    pub fn preview(phrase: &str) -> QorResult<String> {
        Self::preview_at(phrase, 0)
    }

    /// The account at `index` of a recovery phrase, without storing anything.
    pub fn preview_at(phrase: &str, index: u32) -> QorResult<String> {
        derive::validate_mnemonic(phrase)?;
        let root = RootKey::from_mnemonic(phrase.trim(), "")?;
        Ok(derive::derive_account(&root, index)?.address())
    }

    /// Seal a vault from a recovery phrase the person already holds, for a new
    /// computer or a key the keychain no longer has.
    ///
    /// A vault already on this device is **set aside, never deleted**: it is
    /// renamed beside itself. Everything that can refuse — the phrase, an open
    /// vault — refuses before the person is asked, and the file moves only after
    /// they approve in `confirm`, which in the launcher is a host dialog the
    /// webview cannot answer. Returns the new vault's accounts and, if one was
    /// moved, where the old file went.
    pub async fn restore(
        &self,
        confirm: &dyn Confirm,
        phrase: &str,
    ) -> QorResult<(Vec<AccountView>, Option<PathBuf>)> {
        self.restore_at(confirm, phrase, 0).await
    }

    /// [`Self::restore`] for the account at `index` of the phrase.
    pub async fn restore_at(
        &self,
        confirm: &dyn Confirm,
        phrase: &str,
        index: u32,
    ) -> QorResult<(Vec<AccountView>, Option<PathBuf>)> {
        let address = Self::preview_at(phrase, index)?;
        if self.is_open() {
            return Err(QorError::VaultOpen);
        }
        if !self.exists() {
            return self
                .create_at(phrase, index)
                .map(|accounts| (accounts, None));
        }

        let aside = self.set_aside_path();
        let name = aside
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let prompt = Prompt {
            title: "Replace the vault on this device?".into(),
            body: format!(
                "The vault on this device will be set aside as {name} and a new one sealed \
                 from the recovery phrase you typed, which opens account {address}.\n\n\
                 Nothing is deleted. Accounts the old vault held are reachable only from its \
                 own recovery phrase."
            ),
            approve: "Set it aside and restore".into(),
        };
        if !confirm.confirm(&prompt).await {
            return Err(QorError::Declined);
        }
        // Checked again: the vault may have been opened while the dialog was open.
        if self.is_open() {
            return Err(QorError::VaultOpen);
        }

        let moved = self.set_aside(&aside)?;
        match self.create_at(phrase, index) {
            Ok(accounts) => Ok((accounts, Some(aside))),
            Err(error) => {
                // Put the old vault back rather than leave the device with none.
                self.put_back(&aside, moved);
                Err(error)
            }
        }
    }

    /// Open the vault. A keychain vault opens with nothing asked.
    ///
    /// A Windows Hello vault from ADR-055 asks Hello one last time and moves to
    /// the keychain. A passphrase vault from before that moves with Hello alone
    /// if its ADR-054 copy belongs to it, and is otherwise refused with
    /// [`QorError::PassphraseVault`], with Hello not asked.
    pub fn unlock(&self, presence: &dyn Presence) -> QorResult<Vec<AccountView>> {
        if let Some(session) = self.inner.lock().as_ref() {
            return Ok(session.views());
        }
        match self.read_disk()? {
            OnDisk::Keychain(file) => {
                let phrase = self.open_from_keychain(&file)?;
                self.open_phrase(&phrase.0, file.index)
            }
            OnDisk::Hello(file) => {
                let phrase = open_with_hello(presence, &file)?;
                self.move_phrase(&phrase.0)
            }
            OnDisk::Passphrase(_) => {
                let copy = self.legacy_copy().ok_or(QorError::PassphraseVault)?;
                let salt = hex::decode(&copy.salt).map_err(|_| {
                    QorError::VaultCorrupt("the Windows Hello copy's salt is not hex".into())
                })?;
                let signature = presence.sign(&hello::challenge(&salt))?;
                let phrase =
                    seal::open(&copy.sealed, &hex::encode(&signature)).map_err(|e| match e {
                        QorError::BadPassphrase => QorError::PassphraseVault,
                        other => other,
                    })?;
                self.move_phrase(&phrase.0)
            }
        }
    }

    /// Move a passphrase vault from before ADR-055 to the keychain, with its
    /// passphrase typed once more. The old file is set aside, never deleted.
    pub fn move_to_keychain(&self, passphrase: &str) -> QorResult<Vec<AccountView>> {
        let OnDisk::Passphrase(sealed) = self.read_disk()? else {
            return Err(QorError::Internal(
                "this vault was not sealed with a passphrase".into(),
            ));
        };
        let phrase = seal::open(&sealed, passphrase)?;
        self.move_phrase(&phrase.0)
    }

    /// Seal `phrase` under the keychain in place of the vault on disk, which is
    /// set aside with any ADR-054 copy.
    fn move_phrase(&self, phrase: &[u8]) -> QorResult<Vec<AccountView>> {
        let file = self.seal_to_keychain(phrase, 0)?;
        let aside = self.set_aside_path();
        let moved = self.set_aside(&aside)?;
        if let Err(error) = self.write_vault(&file) {
            self.put_back(&aside, moved);
            return Err(error);
        }
        self.open_phrase(phrase, 0)
    }

    /// A fresh key in the keychain, and `phrase` sealed under it, for the account at `index`.
    fn seal_to_keychain(&self, phrase: &[u8], index: u32) -> QorResult<KeychainVault> {
        let mut id = [0u8; 16];
        let mut secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut id);
        rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut secret);
        let id = hex::encode(id);
        let secret = zeroize::Zeroizing::new(hex::encode(secret));

        let sealed = seal::seal(phrase, &secret)?;
        self.keys.put(&keychain::entry_name(&id), &secret)?;
        Ok(KeychainVault {
            unlock: UNLOCK_KEYCHAIN.into(),
            version: KEYCHAIN_FORMAT,
            key: id,
            sealed,
            index,
        })
    }

    fn open_from_keychain(&self, file: &KeychainVault) -> QorResult<Zeroizing> {
        let secret = zeroize::Zeroizing::new(self.keys.get(&keychain::entry_name(&file.key))?);
        seal::open(&file.sealed, &secret).map_err(|e| match e {
            QorError::BadPassphrase => QorError::Keychain(
                "the key in this computer's keychain does not open this vault. Restore the vault \
                 from your recovery phrase"
                    .into(),
            ),
            other => other,
        })
    }

    fn is_open(&self) -> bool {
        self.inner.lock().is_some()
    }

    // ─────────────────────── passphrase vaults, before ADR-055 ───────────────────────

    fn legacy_copy_path(&self) -> PathBuf {
        self.dir().unwrap_or_default().join(hello::LEGACY_COPY_FILE)
    }

    /// ADR-054's copy, if there is one and it was made from the vault file on
    /// disk now. A copy made from a vault since replaced does not count.
    fn legacy_copy(&self) -> Option<hello::LegacyCopy> {
        let raw = std::fs::read_to_string(self.legacy_copy_path()).ok()?;
        let copy: hello::LegacyCopy = serde_json::from_str(&raw).ok()?;
        let binding = blake3::hash(&std::fs::read(self.path()?).ok()?)
            .to_hex()
            .to_string();
        (copy.bound_to == binding).then_some(copy)
    }

    // ───────────────────────────── setting a vault aside ─────────────────────────────

    /// A free name beside the vault: `vault.qor.set-aside-<date>`, then `-2`, `-3`, …
    fn set_aside_path(&self) -> PathBuf {
        let date = chrono::Local::now().format("%Y-%m-%d");
        let base = format!("{VAULT_FILE}.set-aside-{date}");
        let dir = self.dir().unwrap_or_default();
        let mut candidate = dir.join(&base);
        let mut n = 2;
        while candidate.exists() {
            candidate = dir.join(format!("{base}-{n}"));
            n += 1;
        }
        candidate
    }

    /// Where an ADR-054 copy goes when its vault is set aside as `aside`.
    fn copy_beside(aside: &Path) -> PathBuf {
        let mut name = aside.as_os_str().to_os_string();
        name.push(".hello");
        PathBuf::from(name)
    }

    /// Move the vault to `aside`, and its ADR-054 copy with it. Returns whether
    /// the copy moved.
    fn set_aside(&self, aside: &Path) -> QorResult<bool> {
        std::fs::rename(self.file()?, aside)?;
        Ok(self.legacy_copy_path().is_file()
            && std::fs::rename(self.legacy_copy_path(), Self::copy_beside(aside)).is_ok())
    }

    /// Undo [`Self::set_aside`] after a failure.
    fn put_back(&self, aside: &Path, copy_moved: bool) {
        let Ok(file) = self.file() else { return };
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::rename(aside, &file);
        if copy_moved {
            let _ = std::fs::rename(Self::copy_beside(aside), self.legacy_copy_path());
        }
    }

    /// Derive the wallet's account, the one at `index`, from a decrypted phrase and hold the session.
    fn open_phrase(&self, phrase_bytes: &[u8], index: u32) -> QorResult<Vec<AccountView>> {
        let phrase = std::str::from_utf8(phrase_bytes)
            .map_err(|_| QorError::VaultCorrupt("stored phrase is not valid UTF-8".into()))?;

        // The vault seals the phrase alone, so the BIP-39 passphrase it
        // supports is the empty one. ADR-023 makes that passphrase the
        // Substrate password, so when the vault learns to store one, it is
        // passed here and nothing else changes.
        let root = RootKey::from_mnemonic(phrase, "")?;
        let account = derive::derive_account(&root, index)?;

        let session = Unlocked {
            root,
            accounts: vec![account],
            labels: vec!["Primary".to_string()],
            active: None,
        };

        let views = session.views();
        *self.inner.lock() = Some(session);
        Ok(views)
    }

    /// Wipe secret state. Idempotent.
    pub fn lock(&self) {
        *self.inner.lock() = None;
    }

    /// Make `address` the account the launcher acts as: the one its rail shows, and the one a product's first publish
    /// signs with (ADR-090). Only an account this open vault holds can be chosen.
    pub fn set_active(&self, address: &str) -> QorResult<()> {
        let mut guard = self.inner.lock();
        let session = guard.as_mut().ok_or(QorError::VaultLocked)?;
        let address = canonical_address(address)?;
        if !session.accounts.iter().any(|a| a.address() == address) {
            return Err(QorError::BadAddress(format!(
                "{address} is not an account in this vault"
            )));
        }
        session.active = Some(address);
        Ok(())
    }

    /// The account the launcher acts as: the one chosen with [`Self::set_active`], or the first while none is.
    pub fn active_address(&self) -> QorResult<String> {
        let guard = self.inner.lock();
        let session = guard.as_ref().ok_or(QorError::VaultLocked)?;
        Ok(match &session.active {
            Some(address) => address.clone(),
            None => session
                .accounts
                .first()
                .map(|a| a.address())
                .ok_or(QorError::VaultLocked)?,
        })
    }

    /// Derive the next account in the chain. Tests only: since ADR-099 a profile's wallet holds one account, and tests
    /// that need several parties take them from one phrase this way.
    #[cfg(test)]
    pub fn add_account(&self, label: &str) -> QorResult<AccountView> {
        let mut guard = self.inner.lock();
        let session = guard.as_mut().ok_or(QorError::VaultLocked)?;

        let next_index = session.accounts.last().map_or(0, |a| a.index + 1);
        let account = derive::derive_account(&session.root, next_index)?;

        let label = if label.trim().is_empty() {
            format!("Account {}", next_index + 1)
        } else {
            label.trim().to_string()
        };

        let view = AccountView {
            address: account.address(),
            account_id: account.account_id_hex(),
            path: account.path.clone(),
            index: account.index,
            label: label.clone(),
        };

        session.accounts.push(account);
        session.labels.push(label);

        Ok(view)
    }

    /// Sign `message` with the account at `address`, once the person at the
    /// keyboard approves `prompt` (roadmap L1.4).
    ///
    /// This and [`Vault::sign_granted`] are the only routes by which key material
    /// influences the outside world, and there is no way to sign without an
    /// approval given for it. The caller builds the prompt from the same values it
    /// signs, and the launcher shows it in a host-drawn dialog the webview cannot
    /// reach. Two approvals are not a dialog each time: the arrival grant, by which
    /// a QOR ID challenge for the wallet's account is approved straight after
    /// it opens (ADR-016, ADR-056), and the Mesh grant, given once in a dialog, for
    /// the Mesh's own work (ADR-097).
    ///
    /// Nothing is asked when the vault is locked or does not hold `address`, so a
    /// prompt only ever appears for a signature that can actually be made.
    pub async fn sign(
        &self,
        confirm: &dyn Confirm,
        prompt: &Prompt,
        address: &str,
        message: &[u8],
    ) -> QorResult<String> {
        self.with_account(address, |_| ())?;

        if !confirm.confirm(prompt).await {
            return Err(QorError::Declined);
        }

        // Checked again: the vault may have locked while the prompt was open.
        self.with_account(address, |account| hex::encode(account.sign(message)))
    }

    /// Ask the person, once, whether the Mesh may sign its own work for the account at `address` (ADR-097): proofs of
    /// what this PC holds, claims for payment for what it served, taking and leaving the files it holds, and receipts for
    /// what it downloads. None of it can send their CGT anywhere.
    pub async fn grant_mesh(
        &self,
        confirm: &dyn Confirm,
        address: &str,
        chain_name: &str,
    ) -> QorResult<MeshGrant> {
        self.with_account(address, |_| ())?;
        let prompt = Prompt {
            title: "Let the Mesh be paid for what it does?".into(),
            body: format!(
                "While seeding is on, the Mesh signs its own work for {address} on {chain_name}, without asking each                  time: regular proof that this PC holds the files it took on, claims for payment for what it                  served, taking and leaving files within your limits, and receipts for what this PC downloads, so                  the people who served it are paid.

It cannot send your CGT anywhere. Turning seeding off ends                  this."
            ),
            approve: "Allow".into(),
        };
        if !confirm.confirm(&prompt).await {
            return Err(QorError::Declined);
        }
        self.with_account(address, |account| MeshGrant {
            account: account.account_id(),
        })
    }

    /// Sign `message` under the Mesh grant, with no dialog (ADR-097). Only the Mesh's own code calls this, with bytes it
    /// built itself: a granted call it encoded and checked (`chain::mesh::ChainClient::mesh_submit`), or a receipt
    /// (`chain::mesh::receipt_message`). A test keeps it that way.
    pub(crate) fn sign_granted(&self, grant: &MeshGrant, message: &[u8]) -> QorResult<[u8; 64]> {
        self.with_account(&derive::address_of(&grant.account), |account| {
            account.sign(message)
        })
    }

    /// Run `f` with the open account at `address`.
    fn with_account<T>(&self, address: &str, f: impl FnOnce(&DerivedAccount) -> T) -> QorResult<T> {
        let guard = self.inner.lock();
        let session = guard.as_ref().ok_or(QorError::VaultLocked)?;

        let wanted = normalise_address(address)?;
        let account = session
            .accounts
            .iter()
            .find(|a| a.account_id() == wanted)
            .ok_or_else(|| QorError::BadAddress(format!("{address} is not in this vault")))?;

        Ok(f(account))
    }

    // ─────────────────────────── a profile's key (ADR-099) ───────────────────────────

    /// The key that opens this wallet, from the keychain: what a profile's PIN seals when it signs out.
    pub fn key_secret(&self) -> QorResult<zeroize::Zeroizing<String>> {
        let OnDisk::Keychain(file) = self.read_disk()? else {
            return Err(QorError::VaultLocked);
        };
        Ok(zeroize::Zeroizing::new(
            self.keys.get(&keychain::entry_name(&file.key))?,
        ))
    }

    /// Put the key back in the keychain, unsealed with the profile's PIN, so the wallet opens.
    pub fn restore_key(&self, secret: &str) -> QorResult<()> {
        let OnDisk::Keychain(file) = self.read_disk()? else {
            return Err(QorError::VaultLocked);
        };
        self.keys.put(&keychain::entry_name(&file.key), secret)
    }

    /// Take the key out of the keychain, so this wallet opens only with the profile's PIN. Locks it first.
    pub fn forget_key(&self) -> QorResult<()> {
        self.lock();
        let OnDisk::Keychain(file) = self.read_disk()? else {
            return Ok(());
        };
        self.keys.delete(&keychain::entry_name(&file.key))
    }

    /// The address of the account at `index` of this open wallet's phrase, for offering the accounts an older wallet
    /// used as profiles of their own.
    pub fn address_at(&self, index: u32) -> QorResult<String> {
        let guard = self.inner.lock();
        let session = guard.as_ref().ok_or(QorError::VaultLocked)?;
        Ok(derive::derive_account(&session.root, index)?.address())
    }

    /// The index of this wallet's account in its phrase.
    pub fn index(&self) -> QorResult<u32> {
        let guard = self.inner.lock();
        let session = guard.as_ref().ok_or(QorError::VaultLocked)?;
        session
            .accounts
            .first()
            .map(|a| a.index)
            .ok_or(QorError::VaultLocked)
    }

    /// Seal this wallet's phrase, for the account at `index`, as the wallet of another profile in `dir`, its own key in
    /// the keychain. The phrase never leaves this process.
    pub fn copy_to(&self, dir: &Path, index: u32) -> QorResult<()> {
        if !self.is_open() {
            return Err(QorError::VaultLocked);
        }
        let OnDisk::Keychain(file) = self.read_disk()? else {
            return Err(QorError::VaultLocked);
        };
        let phrase = self.open_from_keychain(&file)?;
        let other = Vault::with_keys(dir, Arc::clone(&self.keys));
        if other.exists() {
            return Err(QorError::VaultExists);
        }
        let sealed = other.seal_to_keychain(&phrase.0, index)?;
        other.write_vault(&sealed)
    }

    /// Re-export the recovery phrase, once the person approves it in `confirm`.
    pub async fn export_phrase(&self, confirm: &dyn Confirm) -> QorResult<String> {
        let OnDisk::Keychain(file) = self.read_disk()? else {
            return Err(QorError::VaultLocked);
        };
        let prompt = Prompt {
            title: "Show your recovery phrase?".into(),
            body: "Anyone who sees these 24 words can spend your CGT from any device. Show them \
                   only to write down a backup, and only when you are alone."
                .into(),
            approve: "Show it".into(),
        };
        if !confirm.confirm(&prompt).await {
            return Err(QorError::Declined);
        }
        let bytes = self.open_from_keychain(&file)?;
        std::str::from_utf8(&bytes.0)
            .map(str::to_owned)
            .map_err(|_| QorError::VaultCorrupt("stored phrase is not valid UTF-8".into()))
    }

    fn read_disk(&self) -> QorResult<OnDisk> {
        if !self.exists() {
            return Err(QorError::NoVault);
        }
        let raw = std::fs::read_to_string(self.file()?)?;
        let corrupt =
            |e: serde_json::Error| QorError::VaultCorrupt(format!("cannot parse vault file: {e}"));
        let value: serde_json::Value = serde_json::from_str(&raw).map_err(corrupt)?;
        match value.get("unlock").and_then(|u| u.as_str()) {
            None => serde_json::from_value(value)
                .map(OnDisk::Passphrase)
                .map_err(corrupt),
            Some(UNLOCK_KEYCHAIN) => {
                let file: KeychainVault = serde_json::from_value(value).map_err(corrupt)?;
                if file.version > KEYCHAIN_FORMAT {
                    return Err(newer(file.version));
                }
                Ok(OnDisk::Keychain(file))
            }
            Some(hello::UNLOCK) => {
                let file: HelloVault = serde_json::from_value(value).map_err(corrupt)?;
                if file.version > hello::FORMAT_VERSION {
                    return Err(newer(file.version));
                }
                Ok(OnDisk::Hello(file))
            }
            Some(other) => Err(QorError::VaultCorrupt(format!(
                "vault opened by {other:?} was written by a newer launcher; upgrade to open it"
            ))),
        }
    }

    /// Write the vault, then fsync, so a crash cannot leave a truncated file
    /// where a recovery phrase used to be.
    fn write_vault(&self, file: &KeychainVault) -> QorResult<()> {
        let json = serde_json::to_vec_pretty(file)
            .map_err(|e| QorError::Internal(format!("cannot serialise vault: {e}")))?;
        self.write_atomically(&self.file()?, &json)
    }

    /// Write beside `path`, fsync, then rename over it.
    fn write_atomically(&self, path: &Path, bytes: &[u8]) -> QorResult<()> {
        use std::io::Write;

        std::fs::create_dir_all(self.dir()?)?;
        let mut temp_name = path.file_name().unwrap_or_default().to_os_string();
        temp_name.push(".tmp");
        let temp = path.with_file_name(temp_name);
        {
            let mut file = std::fs::File::create(&temp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        std::fs::rename(&temp, path)?;
        Ok(())
    }
}

fn newer(version: u32) -> QorError {
    QorError::VaultCorrupt(format!(
        "vault format v{version} was written by a newer launcher; upgrade to open it"
    ))
}

/// Open a vault sealed under Windows Hello (ADR-055). Asks the person.
fn open_with_hello(presence: &dyn Presence, file: &HelloVault) -> QorResult<Zeroizing> {
    let salt = hex::decode(&file.salt)
        .map_err(|_| QorError::VaultCorrupt("the vault's salt is not hex".into()))?;
    let signature = presence.sign(&hello::challenge(&salt))?;
    seal::open(&file.sealed, &hex::encode(&signature)).map_err(|e| match e {
        QorError::BadPassphrase => QorError::Hello(
            "Windows Hello did not open this vault: the key it holds is not the one the vault \
             was sealed with. Restore the vault from your recovery phrase"
                .into(),
        ),
        other => other,
    })
}

/// Parse an address into the 32 bytes that identify the account.
///
/// SS58 at the chain's prefix, or hex from an advanced view (ADR-024). The
/// parsing, the checksum and the prefix rule live in [`derive`], so the vault,
/// the chain client and QOR ID's client all read an address the same way.
pub fn normalise_address(address: &str) -> QorResult<[u8; 32]> {
    derive::account_id_from_address(address)
}

/// The canonical SS58 form of whatever address form was supplied.
///
/// Anything that leaves this process — a request to QOR ID, a prompt a person
/// reads — uses this, so one account is one string wherever it appears.
pub fn canonical_address(address: &str) -> QorResult<String> {
    normalise_address(address).map(|id| derive::address_of(&id))
}

/// Test support: a vault in a scratch directory with an in-memory keychain.
#[cfg(test)]
pub mod testing {
    use super::keychain::testing::MemoryKeychain;
    use super::Vault;
    use std::path::Path;
    use std::sync::Arc;

    pub fn vault_in(dir: impl AsRef<Path>) -> Vault {
        Vault::with_keys(dir, Arc::new(MemoryKeychain::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::keychain::testing::MemoryKeychain;
    use super::*;
    use crate::testing::Scripted;
    use hello::testing::FakeHello;
    use tempdir::Holder;

    const PHRASE: &str = "abandon abandon abandon abandon abandon abandon abandon \
                          abandon abandon abandon abandon about";

    fn temp_vault() -> (Vault, Arc<MemoryKeychain>, tempdir::Holder) {
        let holder = tempdir::Holder::new();
        let keys = Arc::new(MemoryKeychain::default());
        (Vault::with_keys(holder.path(), keys.clone()), keys, holder)
    }

    fn asked(hello: &FakeHello) -> usize {
        *hello.asked.lock()
    }

    /// ADR-099: signing a profile out takes its wallet's key out of the keychain, after which the wallet does not open;
    /// the key the PIN unsealed, put back, opens it again. Unplaced, there is no wallet at all.
    #[test]
    fn a_signed_out_wallet_opens_only_once_its_key_is_put_back() {
        let (vault, keys, _dir) = temp_vault();
        let hello = FakeHello::new();
        let address = vault.create(PHRASE).unwrap()[0].address.clone();
        let key = vault.key_secret().unwrap();

        vault.forget_key().unwrap();
        assert!(!vault.is_open(), "taking the key out locks the wallet");
        assert!(keys.0.lock().is_empty(), "nothing left in the keychain");
        assert_eq!(vault.unlock(&hello).unwrap_err().kind(), "keychain");

        vault.restore_key(&key).unwrap();
        assert_eq!(vault.unlock(&hello).unwrap()[0].address, address);
        assert_eq!(asked(&hello), 0, "nothing was asked");

        vault.lock();
        vault.place(None).unwrap();
        assert!(matches!(vault.status(), VaultStatus::Absent));
        assert_eq!(vault.create(PHRASE).unwrap_err().kind(), "no_profile");
        assert_eq!(
            Vault::unplaced(keys.clone())
                .key_secret()
                .unwrap_err()
                .kind(),
            "no_vault"
        );
    }

    /// ADR-099: a profile's wallet is one account. Another account an older wallet's phrase used becomes a profile of its
    /// own holding that account alone, under its own key, and opens as that account every time.
    #[test]
    fn another_account_of_an_older_phrase_becomes_a_wallet_of_its_own() {
        let (vault, keys, _dir) = temp_vault();
        let hello = FakeHello::new();
        let first = vault.create(PHRASE).unwrap();
        assert_eq!(first.len(), 1);
        let second = vault.address_at(1).unwrap();
        assert_eq!(second, Vault::preview_at(PHRASE, 1).unwrap());
        assert_ne!(second, first[0].address);

        let other_dir = tempdir::Holder::new();
        vault.copy_to(other_dir.path(), 1).unwrap();
        assert_eq!(
            vault.copy_to(other_dir.path(), 1).unwrap_err().kind(),
            "vault_exists"
        );
        let other = Vault::with_keys(other_dir.path(), keys.clone());
        let opened = other.unlock(&hello).unwrap();
        assert_eq!(opened.len(), 1, "one account");
        assert_eq!(opened[0].address, second);
        assert_eq!(other.index().unwrap(), 1);
        assert_ne!(
            *other.key_secret().unwrap(),
            *vault.key_secret().unwrap(),
            "its own key"
        );
        other.lock();
        assert_eq!(
            other.unlock(&hello).unwrap()[0].address,
            second,
            "every time"
        );
    }

    #[test]
    fn lifecycle_absent_to_open_and_back_with_nothing_asked() {
        let (vault, _keys, _dir) = temp_vault();
        let hello = FakeHello::new();
        assert!(matches!(vault.status(), VaultStatus::Absent));

        let accounts = vault.create(PHRASE).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts[0].path, "",
            "the first account is the phrase itself, with no path (ADR-023)"
        );
        assert!(matches!(vault.status(), VaultStatus::Unlocked { .. }));

        vault.lock();
        assert!(matches!(
            vault.status(),
            VaultStatus::Locked {
                sealed_with: SealedWith::Keychain
            }
        ));
        let reopened = vault.unlock(&hello).unwrap();
        assert_eq!(reopened[0].address, accounts[0].address);
        assert_eq!(asked(&hello), 0, "a keychain vault asks nothing");
    }

    /// ADR-090: the launcher acts as the first account until the person picks another the vault holds, and a lock
    /// forgets the choice.
    #[test]
    fn the_active_account_is_the_first_until_another_of_its_own_is_chosen() {
        let (vault, _keys, _dir) = temp_vault();
        assert_eq!(vault.active_address().unwrap_err().kind(), "vault_locked");
        let first = vault.create(PHRASE).unwrap()[0].address.clone();
        assert_eq!(vault.active_address().unwrap(), first);

        let second = vault.add_account("").unwrap().address;
        assert_eq!(
            vault.active_address().unwrap(),
            first,
            "adding is not choosing"
        );
        vault.set_active(&second).unwrap();
        assert_eq!(vault.active_address().unwrap(), second);

        let stranger = Vault::preview(
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        assert_eq!(
            vault.set_active(&stranger).unwrap_err().kind(),
            "bad_address"
        );
        assert_eq!(
            vault.set_active("not an address").unwrap_err().kind(),
            "bad_address"
        );
        assert_eq!(
            vault.active_address().unwrap(),
            second,
            "a refused choice changes nothing"
        );

        vault.lock();
        assert_eq!(
            vault.set_active(&second).unwrap_err().kind(),
            "vault_locked"
        );
        vault.unlock(&FakeHello::new()).unwrap();
        assert_eq!(
            vault.active_address().unwrap(),
            first,
            "a lock forgets the choice"
        );
    }

    #[test]
    fn refuses_to_overwrite_an_existing_vault() {
        let (vault, _keys, _dir) = temp_vault();
        vault.create(PHRASE).unwrap();
        assert_eq!(vault.create(PHRASE).unwrap_err().kind(), "vault_exists");
    }

    #[test]
    fn rejects_a_bad_phrase_and_writes_nothing() {
        let (vault, keys, _dir) = temp_vault();
        assert_eq!(
            vault.create("not a phrase").unwrap_err().kind(),
            "bad_mnemonic"
        );
        assert!(!vault.exists());
        assert!(keys.0.lock().is_empty());
    }

    /// Without its keychain entry, a copied file opens nothing.
    #[test]
    fn a_copied_file_opens_nothing_without_the_keychain() {
        let (vault, _keys, dir) = temp_vault();
        vault.create(PHRASE).unwrap();

        let elsewhere = Holder::new();
        std::fs::copy(
            dir.path().join(VAULT_FILE),
            elsewhere.path().join(VAULT_FILE),
        )
        .unwrap();
        let stranger = testing::vault_in(elsewhere.path());
        assert_eq!(
            stranger.unlock(&FakeHello::new()).unwrap_err().kind(),
            "keychain"
        );
        assert!(!stranger.is_open());
    }

    #[test]
    fn the_vault_file_holds_neither_the_phrase_nor_the_key() {
        let (vault, keys, _dir) = temp_vault();
        vault.create(PHRASE).unwrap();

        let written = std::fs::read_to_string(vault.path().unwrap()).unwrap();
        let file: KeychainVault = serde_json::from_str(&written).unwrap();
        assert_eq!(file.unlock, "os-keychain");
        let secret = keys.get(&keychain::entry_name(&file.key)).unwrap();
        assert!(!written.contains("abandon"));
        assert!(!written.contains(&secret));
        assert!(!written.contains(&secret[..16]));
    }

    fn prompt() -> Prompt {
        Prompt {
            title: "Sign?".into(),
            body: "A test signature.".into(),
            approve: "Sign".into(),
        }
    }

    #[tokio::test]
    async fn signing_requires_an_open_vault_and_a_known_address() {
        let (vault, _keys, _dir) = temp_vault();
        let accounts = vault.create(PHRASE).unwrap();
        let address = &accounts[0].address;
        let confirm = Scripted::approving();

        let signature = vault
            .sign(&confirm, &prompt(), address, b"payload")
            .await
            .unwrap();
        assert_eq!(signature.len(), 128, "64-byte Sr25519 signature as hex");

        let stranger = derive::address_of(&[0x11u8; 32]);
        assert_eq!(
            vault
                .sign(&confirm, &prompt(), &stranger, b"payload")
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );

        vault.lock();
        assert_eq!(
            vault
                .sign(&confirm, &prompt(), address, b"payload")
                .await
                .unwrap_err()
                .kind(),
            "vault_locked"
        );

        assert_eq!(
            confirm.times_asked(),
            1,
            "nobody is asked to approve a signature that cannot be made"
        );
    }

    /// Nothing is signed unless the person at the keyboard approves (L1.4).
    #[tokio::test]
    async fn a_declined_prompt_signs_nothing() {
        let (vault, _keys, _dir) = temp_vault();
        let accounts = vault.create(PHRASE).unwrap();
        let confirm = Scripted::declining();
        let shown = Prompt {
            title: "Send CGT?".into(),
            body: "Amount: 1.00 CGT".into(),
            approve: "Sign and send".into(),
        };

        assert_eq!(
            vault
                .sign(&confirm, &shown, &accounts[0].address, b"payload")
                .await
                .unwrap_err()
                .kind(),
            "declined"
        );
        assert_eq!(
            confirm.asked.lock().as_slice(),
            &[shown][..],
            "the prompt shown is exactly the one the caller built"
        );
    }

    /// The signature the vault produces must verify under the account ID the
    /// chain will use, since for Sr25519 the account ID *is* the public key.
    #[tokio::test]
    async fn signature_verifies_against_the_address() {
        let (vault, _keys, _dir) = temp_vault();
        let accounts = vault.create(PHRASE).unwrap();
        let message = b"demiurge:mainnet:v1 payload";

        let sig_hex = vault
            .sign(
                &Scripted::approving(),
                &prompt(),
                &accounts[0].address,
                message,
            )
            .await
            .unwrap();
        let account_id = normalise_address(&accounts[0].address).unwrap();
        let signature: [u8; 64] = hex::decode(&sig_hex).unwrap().try_into().unwrap();

        assert!(derive::verify(&account_id, message, &signature));
        assert!(
            !derive::verify(&account_id, b"a different message", &signature),
            "a signature must not verify for bytes nobody signed"
        );
    }

    #[test]
    fn additional_accounts_are_distinct_and_labelled() {
        let (vault, _keys, _dir) = temp_vault();
        let first = vault.create(PHRASE).unwrap();

        let second = vault.add_account("Trading").unwrap();
        assert_eq!(second.index, 1);
        assert_eq!(second.label, "Trading");
        assert_ne!(second.address, first[0].address);

        let third = vault.add_account("   ").unwrap();
        assert_eq!(third.label, "Account 3", "blank labels get a default");
    }

    #[tokio::test]
    async fn export_asks_in_a_dialog_first() {
        let (vault, _keys, _dir) = temp_vault();
        vault.create(PHRASE).unwrap();

        let approving = Scripted::approving();
        assert_eq!(
            vault.export_phrase(&approving).await.unwrap(),
            PHRASE.split_whitespace().collect::<Vec<_>>().join(" ")
        );
        assert_eq!(approving.times_asked(), 1);
        assert_eq!(
            vault
                .export_phrase(&Scripted::declining())
                .await
                .unwrap_err()
                .kind(),
            "declined"
        );
    }

    /// The vault reads an address the way every other surface does: SS58 for
    /// people, hex for advanced views, and nothing else (ADR-024).
    #[test]
    fn address_parsing_accepts_both_forms_and_rejects_junk() {
        let account_id = [0xabu8; 32];
        let ss58 = derive::address_of(&account_id);
        let bare = "ab".repeat(32);
        let prefixed = format!("0x{bare}");

        assert_eq!(normalise_address(&ss58).unwrap(), account_id);
        assert_eq!(
            normalise_address(&bare).unwrap(),
            normalise_address(&prefixed).unwrap()
        );

        for bad in ["", "0x", "0xzz", &"ab".repeat(31)] {
            assert!(
                normalise_address(bad).is_err(),
                "{bad:?} should be rejected"
            );
        }
    }

    /// The address a person sees and the hex an advanced view shows are two
    /// forms of one account, and the account view carries both.
    #[test]
    fn an_account_view_carries_both_forms() {
        let (vault, _keys, _dir) = temp_vault();
        let accounts = vault.create(PHRASE).unwrap();
        let view = &accounts[0];

        assert!(!view.address.starts_with("0x"), "the address is SS58");
        assert!(view.account_id.starts_with("0x"), "the account ID is hex");
        assert_eq!(
            normalise_address(&view.address).unwrap(),
            normalise_address(&view.account_id).unwrap(),
            "both forms must name the same account"
        );
    }

    /// A different valid phrase: the old vault's owner and the restorer are
    /// not assumed to be the same key.
    const OTHER: &str =
        "legal winner thank year wave sausage worth useful legal winner thank yellow";

    fn set_aside_files(dir: &Path) -> Vec<PathBuf> {
        let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| {
                let name = p.to_string_lossy();
                name.contains(".set-aside-") && !name.ends_with(".hello")
            })
            .collect();
        found.sort();
        found
    }

    #[tokio::test]
    async fn restoring_with_no_vault_seals_one_and_asks_nothing() {
        let (vault, _keys, _dir) = temp_vault();
        let confirm = Scripted::approving();
        let (accounts, aside) = vault.restore(&confirm, OTHER).await.unwrap();
        assert_eq!(accounts[0].address, Vault::preview(OTHER).unwrap());
        assert!(aside.is_none());
        assert_eq!(
            confirm.times_asked(),
            0,
            "nothing is replaced, so nothing is asked"
        );
    }

    /// The old vault is set aside byte for byte, still opens, and the new one
    /// opens the restored phrase's account.
    #[tokio::test]
    async fn restoring_over_a_vault_sets_it_aside_and_deletes_nothing() {
        let (vault, keys, dir) = temp_vault();
        let old = vault.create(PHRASE).unwrap();
        vault.lock();
        let before = std::fs::read(vault.path().unwrap()).unwrap();

        let confirm = Scripted::approving();
        let (accounts, aside) = vault.restore(&confirm, OTHER).await.unwrap();
        let aside = aside.expect("the old vault was moved");

        assert_eq!(
            std::fs::read(&aside).unwrap(),
            before,
            "set aside unchanged"
        );
        assert_eq!(set_aside_files(dir.path()), vec![aside.clone()]);
        assert_ne!(accounts[0].address, old[0].address);
        assert!(
            confirm.asked.lock()[0].body.contains(&accounts[0].address),
            "the dialog names the account the phrase opens"
        );

        vault.lock();
        let hello = FakeHello::new();
        assert_eq!(
            vault.unlock(&hello).unwrap()[0].address,
            accounts[0].address
        );

        let old_dir = Holder::new();
        std::fs::copy(&aside, old_dir.path().join(VAULT_FILE)).unwrap();
        let reopened = Vault::with_keys(old_dir.path(), keys.clone())
            .unlock(&hello)
            .unwrap();
        assert_eq!(
            reopened[0].address, old[0].address,
            "the old vault still opens: its key is kept"
        );
    }

    #[tokio::test]
    async fn a_declined_restore_moves_nothing() {
        let (vault, _keys, dir) = temp_vault();
        vault.create(PHRASE).unwrap();
        vault.lock();
        let before = std::fs::read(vault.path().unwrap()).unwrap();

        let confirm = Scripted::declining();
        let refused = vault.restore(&confirm, OTHER).await.unwrap_err();
        assert_eq!(refused.kind(), "declined");
        assert_eq!(std::fs::read(vault.path().unwrap()).unwrap(), before);
        assert!(set_aside_files(dir.path()).is_empty());
        assert!(vault.unlock(&FakeHello::new()).is_ok());
    }

    /// A typo in the phrase or an open vault is refused before anyone is asked,
    /// and before anything moves.
    #[tokio::test]
    async fn a_restore_that_cannot_succeed_is_refused_before_the_dialog() {
        let (vault, _keys, dir) = temp_vault();
        vault.create(PHRASE).unwrap();
        let confirm = Scripted::approving();

        assert_eq!(
            vault.restore(&confirm, OTHER).await.unwrap_err().kind(),
            "vault_open"
        );
        vault.lock();
        let typo = OTHER.replace("sausage", "sausages");
        assert_eq!(
            vault.restore(&confirm, &typo).await.unwrap_err().kind(),
            "bad_mnemonic"
        );
        assert_eq!(confirm.times_asked(), 0);
        assert!(set_aside_files(dir.path()).is_empty());
    }

    #[tokio::test]
    async fn a_second_restore_the_same_day_takes_the_next_free_name() {
        let (vault, _keys, dir) = temp_vault();
        vault.create(PHRASE).unwrap();
        vault.lock();
        let confirm = Scripted::approving();
        let (_, first) = vault.restore(&confirm, OTHER).await.unwrap();
        vault.lock();
        let (_, second) = vault.restore(&confirm, PHRASE).await.unwrap();
        assert_ne!(first, second);
        assert_eq!(
            set_aside_files(dir.path()).len(),
            2,
            "neither overwrote the other"
        );
    }

    // ─────────────────── vaults sealed before ADR-056 ───────────────────

    const PASS: &str = "a sufficiently long passphrase";

    /// A vault as ADR-055 sealed it, under `hello`.
    fn hello_vault(vault: &Vault, hello: &FakeHello) {
        let salt = [3u8; 32];
        let signature = hello.sign(&hello::challenge(&salt)).unwrap();
        let file = HelloVault {
            unlock: hello::UNLOCK.into(),
            version: hello::FORMAT_VERSION,
            salt: hex::encode(salt),
            sealed: seal::seal(PHRASE.as_bytes(), &hex::encode(&signature)).unwrap(),
        };
        std::fs::write(
            vault.path().unwrap(),
            serde_json::to_vec_pretty(&file).unwrap(),
        )
        .unwrap();
        *hello.asked.lock() = 0;
    }

    /// A vault as the launcher sealed it before ADR-055.
    fn passphrase_vault(vault: &Vault) {
        let sealed = seal::seal(PHRASE.as_bytes(), PASS).unwrap();
        std::fs::write(
            vault.path().unwrap(),
            serde_json::to_vec_pretty(&sealed).unwrap(),
        )
        .unwrap();
    }

    /// ADR-054's copy of a passphrase vault, made with `hello`.
    fn adr_054_copy(vault: &Vault, hello: &FakeHello) {
        let salt = [5u8; 32];
        let signature = hello.sign(&hello::challenge(&salt)).unwrap();
        let copy = serde_json::json!({
            "version": 1,
            "salt": hex::encode(salt),
            "bound_to": blake3::hash(&std::fs::read(vault.path().unwrap()).unwrap()).to_hex().to_string(),
            "sealed": seal::seal(PHRASE.as_bytes(), &hex::encode(&signature)).unwrap(),
        });
        std::fs::write(vault.legacy_copy_path(), copy.to_string()).unwrap();
        *hello.asked.lock() = 0;
    }

    /// Hello is asked one last time, and never again.
    #[test]
    fn a_hello_vault_moves_to_the_keychain_with_one_last_gesture() {
        let (vault, _keys, dir) = temp_vault();
        let hello = FakeHello::new();
        hello_vault(&vault, &hello);
        let before = std::fs::read(vault.path().unwrap()).unwrap();
        assert!(matches!(
            vault.status(),
            VaultStatus::Locked {
                sealed_with: SealedWith::Hello
            }
        ));

        let opened = vault.unlock(&hello).unwrap();
        assert_eq!(opened[0].address, Vault::preview(PHRASE).unwrap());
        assert_eq!(asked(&hello), 1);
        let aside = set_aside_files(dir.path());
        assert_eq!(aside.len(), 1);
        assert_eq!(
            std::fs::read(&aside[0]).unwrap(),
            before,
            "set aside unchanged"
        );

        vault.lock();
        assert!(vault.unlock(&hello).is_ok());
        assert_eq!(asked(&hello), 1, "from now on nothing is asked");
    }

    #[test]
    fn a_cancelled_hello_moves_nothing() {
        let (vault, _keys, dir) = temp_vault();
        let hello = FakeHello::new();
        hello_vault(&vault, &hello);
        let mut cancelled = FakeHello::new();
        cancelled.decline = true;
        assert_eq!(vault.unlock(&cancelled).unwrap_err().kind(), "declined");
        assert!(set_aside_files(dir.path()).is_empty());
        assert!(!vault.is_open());
    }

    #[test]
    fn a_passphrase_vault_is_reported_without_asking_hello() {
        let (vault, _keys, _dir) = temp_vault();
        passphrase_vault(&vault);
        let hello = FakeHello::new();
        assert!(matches!(
            vault.status(),
            VaultStatus::Locked {
                sealed_with: SealedWith::Passphrase
            }
        ));
        assert_eq!(vault.unlock(&hello).unwrap_err().kind(), "passphrase_vault");
        assert_eq!(asked(&hello), 0);
    }

    #[test]
    fn a_passphrase_vault_moves_with_its_passphrase_once() {
        let (vault, _keys, dir) = temp_vault();
        passphrase_vault(&vault);
        let before = std::fs::read(vault.path().unwrap()).unwrap();

        assert_eq!(
            vault
                .move_to_keychain("wrong passphrase here")
                .unwrap_err()
                .kind(),
            "bad_passphrase"
        );
        assert_eq!(std::fs::read(vault.path().unwrap()).unwrap(), before);

        let moved = vault.move_to_keychain(PASS).unwrap();
        assert_eq!(moved[0].address, Vault::preview(PHRASE).unwrap());
        let aside = set_aside_files(dir.path());
        assert_eq!(aside.len(), 1);
        assert_eq!(std::fs::read(&aside[0]).unwrap(), before, "unchanged");

        vault.lock();
        assert!(
            vault.unlock(&FakeHello::new()).is_ok(),
            "nothing asked from now on"
        );
    }

    #[test]
    fn an_adr_054_copy_moves_the_vault_with_no_passphrase() {
        let (vault, _keys, dir) = temp_vault();
        passphrase_vault(&vault);
        let hello = FakeHello::new();
        adr_054_copy(&vault, &hello);

        let opened = vault.unlock(&hello).unwrap();
        assert_eq!(opened[0].address, Vault::preview(PHRASE).unwrap());
        assert!(
            !vault.legacy_copy_path().exists(),
            "the copy went with its vault"
        );
        assert_eq!(set_aside_files(dir.path()).len(), 1);
    }

    #[test]
    fn an_adr_054_copy_of_another_vault_is_ignored() {
        let (vault, _keys, _dir) = temp_vault();
        passphrase_vault(&vault);
        let hello = FakeHello::new();
        adr_054_copy(&vault, &hello);
        passphrase_vault(&vault); // resealed: a new salt and nonce, a new file

        assert_eq!(vault.unlock(&hello).unwrap_err().kind(), "passphrase_vault");
        assert_eq!(asked(&hello), 0);
    }

    /// Minimal scratch-directory helper so tests do not need a dev-dependency.
    mod tempdir {
        use std::path::{Path, PathBuf};

        pub struct Holder(PathBuf);

        impl Holder {
            pub fn new() -> Self {
                let unique = format!(
                    "qor-vault-test-{}-{:?}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                let path = std::env::temp_dir().join(unique);
                std::fs::create_dir_all(&path).unwrap();
                Self(path)
            }

            pub fn path(&self) -> &Path {
                &self.0
            }
        }

        impl Drop for Holder {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
    }
}
