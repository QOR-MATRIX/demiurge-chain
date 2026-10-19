//! Profiles (ADR-099): each QOR ID on this PC has its own wallet, its own data and a PIN.
//!
//! A profile is a folder, `profiles/<id>/` in the launcher's data directory (and the same under its local data
//! directory), holding everything that is the person's: the wallet, the avatar, trade partners, listing drafts, paid
//! requests, the bytes of their published work, the Mesh and the Library. `profiles.json` beside the folders lists the
//! profiles by id, name and address, and names the one signed in. It holds nothing secret except each profile's wallet
//! key sealed under its PIN.
//!
//! The wallet's key is in the keychain in the clear only while its profile is signed in, so the profile signed in last
//! opens at launch with nothing asked (ADR-056). Signing out removes it from the keychain; opening the profile again
//! takes its PIN, which unseals the key (Argon2id, then XChaCha20-Poly1305: the vault's own seal).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};
use crate::vault::seal::{self, SealedVault, Zeroizing};

/// The list of profiles, in the launcher's data directory.
pub const REGISTRY: &str = "profiles.json";
/// The folder the profiles' own folders are in.
pub const DIR: &str = "profiles";
const VERSION: u32 = 1;

/// Wrong PINs allowed before each further try waits.
const FREE_TRIES: u32 = 5;
/// The first wait, doubled with each wrong PIN after it.
const FIRST_WAIT_SECONDS: i64 = 30;
/// The longest wait, a day.
const LONGEST_WAIT_SECONDS: i64 = 24 * 60 * 60;

/// What the launcher keeps about one profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    /// Its account, once its wallet has opened on this PC.
    #[serde(default)]
    pub address: String,
    /// Its QOR ID's name, once it has signed in to QOR ID.
    #[serde(default)]
    pub name: Option<String>,
    /// When it was made here, Unix seconds.
    #[serde(default)]
    pub created: i64,
    /// The wallet's key, sealed under the profile's PIN. None until a PIN is chosen.
    #[serde(default)]
    pub pin: Option<SealedVault>,
    /// Wrong PINs since the last right one.
    #[serde(default)]
    pub failures: u32,
    /// No PIN is tried before this, Unix seconds.
    #[serde(default)]
    pub wait_until: i64,
}

/// `profiles.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Registry {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// The profile signed in, whose wallet key is in the keychain.
    #[serde(default)]
    pub signed_in: Option<String>,
}

/// A profile as *Who's playing?* shows it: no balance, nothing secret.
#[derive(Debug, Clone, Serialize)]
pub struct ProfileView {
    pub id: String,
    pub name: Option<String>,
    pub address: String,
    pub has_pin: bool,
    pub signed_in: bool,
    /// Seconds before its PIN can be tried again; 0 when it can be now.
    pub wait_seconds: i64,
}

impl Registry {
    pub fn find(&self, id: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    fn find_mut(&mut self, id: &str) -> QorResult<&mut Profile> {
        self.profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| QorError::Pin("That QOR ID is not on this PC any more.".into()))
    }

    pub fn views(&self, now: i64) -> Vec<ProfileView> {
        self.profiles
            .iter()
            .map(|p| ProfileView {
                id: p.id.clone(),
                name: p.name.clone(),
                address: p.address.clone(),
                has_pin: p.pin.is_some(),
                signed_in: self.signed_in.as_deref() == Some(p.id.as_str()),
                wait_seconds: (p.wait_until - now).max(0),
            })
            .collect()
    }
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// A profile's folder in the launcher's data directory.
pub fn dir(root: &Path, id: &str) -> PathBuf {
    root.join(DIR).join(id)
}

/// A profile's folder in the launcher's local data directory: the Library's index.
pub fn local_dir(local_root: &Path, id: &str) -> PathBuf {
    local_root.join(DIR).join(id)
}

pub fn read(root: &Path) -> Registry {
    std::fs::read(root.join(REGISTRY))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn write(root: &Path, registry: &Registry) -> QorResult<()> {
    use std::io::Write;
    std::fs::create_dir_all(root)?;
    let mut registry = registry.clone();
    registry.version = VERSION;
    let bytes =
        serde_json::to_vec_pretty(&registry).map_err(|e| QorError::Internal(e.to_string()))?;
    let path = root.join(REGISTRY);
    let temp = root.join(format!("{REGISTRY}.tmp"));
    {
        let mut file = std::fs::File::create(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
    }
    std::fs::rename(&temp, &path)?;
    Ok(())
}

/// A new profile, listed and not yet signed in. Its folders are made.
pub fn add(root: &Path, local_root: &Path) -> QorResult<Profile> {
    let mut id = [0u8; 8];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut id);
    let profile = Profile {
        id: hex::encode(id),
        address: String::new(),
        name: None,
        created: now(),
        pin: None,
        failures: 0,
        wait_until: 0,
    };
    std::fs::create_dir_all(dir(root, &profile.id))?;
    std::fs::create_dir_all(local_dir(local_root, &profile.id))?;
    let mut registry = read(root);
    registry.profiles.push(profile.clone());
    write(root, &registry)?;
    Ok(profile)
}

/// Note which profile is signed in, or that none is.
pub fn set_signed_in(root: &Path, id: Option<&str>) -> QorResult<()> {
    let mut registry = read(root);
    registry.signed_in = id.map(str::to_string);
    write(root, &registry)
}

/// Note a profile's account and QOR ID name as they become known.
pub fn note(root: &Path, id: &str, address: Option<&str>, name: Option<&str>) -> QorResult<()> {
    let mut registry = read(root);
    let profile = registry.find_mut(id)?;
    let mut changed = false;
    if let Some(address) = address.filter(|a| *a != profile.address) {
        profile.address = address.to_string();
        changed = true;
    }
    if let Some(name) = name.filter(|n| profile.name.as_deref() != Some(*n)) {
        profile.name = Some(name.to_string());
        changed = true;
    }
    if changed {
        write(root, &registry)?;
    }
    Ok(())
}

/// Take a profile off this PC's list and delete its folders. Only for a profile just made whose wallet could not be
/// sealed: it holds nothing.
pub fn remove(root: &Path, local_root: &Path, id: &str) -> QorResult<()> {
    let mut registry = read(root);
    registry.profiles.retain(|p| p.id != id);
    if registry.signed_in.as_deref() == Some(id) {
        registry.signed_in = None;
    }
    write(root, &registry)?;
    let _ = std::fs::remove_dir_all(dir(root, id));
    let _ = std::fs::remove_dir_all(local_dir(local_root, id));
    Ok(())
}

/// Forget a profile's PIN, when its wallet is restored with a new key the old PIN did not seal.
pub fn clear_pin(root: &Path, id: &str) -> QorResult<()> {
    let mut registry = read(root);
    let profile = registry.find_mut(id)?;
    profile.pin = None;
    profile.failures = 0;
    profile.wait_until = 0;
    write(root, &registry)
}

/// A PIN is six to twelve digits.
pub fn validate_pin(pin: &str) -> QorResult<()> {
    if (6..=12).contains(&pin.len()) && pin.bytes().all(|b| b.is_ascii_digit()) {
        Ok(())
    } else {
        Err(QorError::Pin("A PIN is 6 to 12 digits.".into()))
    }
}

/// Seal `key`, the wallet's key, under `pin` and keep it in the profile's entry. A new PIN replaces the old one.
pub fn set_pin(root: &Path, id: &str, key: &str, pin: &str) -> QorResult<()> {
    validate_pin(pin)?;
    let sealed = seal::seal(key.as_bytes(), pin)?;
    let mut registry = read(root);
    let profile = registry.find_mut(id)?;
    profile.pin = Some(sealed);
    profile.failures = 0;
    profile.wait_until = 0;
    write(root, &registry)
}

/// Unseal a profile's wallet key with its PIN. A wrong PIN is counted; after [`FREE_TRIES`] each further try waits,
/// doubling from [`FIRST_WAIT_SECONDS`]. A right one clears the count.
pub fn open_with_pin(root: &Path, id: &str, pin: &str, now: i64) -> QorResult<Zeroizing> {
    let mut registry = read(root);
    let profile = registry.find_mut(id)?;
    if profile.wait_until > now {
        return Err(QorError::Pin(format!(
            "Too many wrong PINs. Try again in {}.",
            wait_words(profile.wait_until - now)
        )));
    }
    let sealed = profile.pin.clone().ok_or_else(|| {
        QorError::Pin(
            "This QOR ID has no PIN on this PC. Restore it from its recovery phrase.".into(),
        )
    })?;
    if validate_pin(pin).is_err() {
        return Err(QorError::Pin("That PIN is not right.".into()));
    }
    match seal::open(&sealed, pin) {
        Ok(key) => {
            if profile.failures != 0 || profile.wait_until != 0 {
                profile.failures = 0;
                profile.wait_until = 0;
                write(root, &registry)?;
            }
            Ok(key)
        }
        Err(QorError::BadPassphrase) => {
            profile.failures = profile.failures.saturating_add(1);
            let message = if profile.failures >= FREE_TRIES {
                let wait = wait_after(profile.failures);
                profile.wait_until = now + wait;
                format!("That PIN is not right. Try again in {}.", wait_words(wait))
            } else {
                "That PIN is not right.".to_string()
            };
            write(root, &registry)?;
            Err(QorError::Pin(message))
        }
        Err(other) => Err(other),
    }
}

/// How long the next try waits after `failures` wrong PINs.
fn wait_after(failures: u32) -> i64 {
    let doublings = failures.saturating_sub(FREE_TRIES).min(20);
    (FIRST_WAIT_SECONDS << doublings).min(LONGEST_WAIT_SECONDS)
}

fn wait_words(seconds: i64) -> String {
    match seconds {
        s if s < 60 => format!("{s} seconds"),
        s if s < 3600 => format!("{} minutes", (s + 59) / 60),
        s => format!("{} hours", (s + 3599) / 3600),
    }
}

/// Everything personal that sat in the data directory before profiles, moved into the first profile's folder. The
/// last is the content store's folder, `content::STORE_DIR` in the Windows launcher, which a test there holds equal
/// while the content store is not yet in this core.
pub const PERSONAL: &[&str] = &[
    "vault.qor",
    crate::vault::hello::LEGACY_COPY_FILE,
    "avatar.bin",
    "avatar.type",
    "trade-partners.json",
    "listing-drafts",
    "paid-requests.json",
    "mesh",
    "temporary-content-store",
];
/// The same, in the local data directory.
const PERSONAL_LOCAL: &[&str] = &["library"];
/// ADR-098's file, which described one wallet holding every QOR ID's account. ADR-099 removed that shape.
const RETIRED: &[&str] = &["account-owners.json"];

/// Make the wallet from before profiles the first profile, signed in, with its personal files moved into its folder.
/// Does nothing once `profiles.json` exists, or when there is no wallet to move. Returns the new profile's id.
pub fn migrate(root: &Path, local_root: &Path) -> QorResult<Option<String>> {
    if root.join(REGISTRY).exists() || !root.join("vault.qor").is_file() {
        return Ok(None);
    }
    let profile = add(root, local_root)?;
    let home = dir(root, &profile.id);
    let local_home = local_dir(local_root, &profile.id);
    for name in PERSONAL {
        move_if_there(&root.join(name), &home.join(name))?;
    }
    for name in PERSONAL_LOCAL {
        move_if_there(&local_root.join(name), &local_home.join(name))?;
    }
    for name in RETIRED {
        let _ = std::fs::remove_file(root.join(name));
    }
    set_signed_in(root, Some(&profile.id))?;
    tracing::info!(profile = %profile.id, "the wallet from before profiles is now the first profile");
    Ok(Some(profile.id))
}

fn move_if_there(from: &Path, to: &Path) -> QorResult<()> {
    if from.exists() {
        std::fs::rename(from, to).map_err(|e| {
            QorError::Io(format!(
                "could not move {} into its profile: {e}",
                from.display()
            ))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs() -> (PathBuf, PathBuf) {
        let base =
            std::env::temp_dir().join(format!("qor-profiles-{}", uuid::Uuid::new_v4().simple()));
        let (root, local) = (base.join("roaming"), base.join("local"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&local).unwrap();
        (root, local)
    }

    #[test]
    fn a_pin_is_six_to_twelve_digits() {
        for good in ["123456", "000000000000"] {
            assert!(validate_pin(good).is_ok(), "{good}");
        }
        for bad in ["12345", "1234567890123", "12345a", "", "１２３４５６"] {
            assert_eq!(validate_pin(bad).unwrap_err().kind(), "pin", "{bad}");
        }
    }

    /// The key comes back only with the right PIN; wrong ones are counted, and after five each try waits, doubling.
    /// A right PIN, once the wait is over, clears the count.
    #[test]
    fn the_wallet_key_opens_only_with_its_pin_and_wrong_pins_wait() {
        let (root, local) = dirs();
        let profile = add(&root, &local).unwrap();
        assert!(
            open_with_pin(&root, &profile.id, "123456", 0).is_err(),
            "no PIN yet"
        );
        set_pin(&root, &profile.id, "the wallet key", "123456").unwrap();
        assert_eq!(
            &open_with_pin(&root, &profile.id, "123456", 0).unwrap().0[..],
            b"the wallet key"
        );

        let t = 1_000;
        for _ in 0..4 {
            assert_eq!(
                open_with_pin(&root, &profile.id, "654321", t)
                    .unwrap_err()
                    .to_string(),
                "That PIN is not right."
            );
        }
        let fifth = open_with_pin(&root, &profile.id, "654321", t)
            .unwrap_err()
            .to_string();
        assert!(fifth.contains("30 seconds"), "{fifth}");
        // Waiting: even the right PIN is not tried.
        assert!(open_with_pin(&root, &profile.id, "123456", t + 10)
            .unwrap_err()
            .to_string()
            .contains("Too many"));
        let sixth = open_with_pin(&root, &profile.id, "654321", t + 31)
            .unwrap_err()
            .to_string();
        assert!(sixth.contains("minutes"), "the wait doubles: {sixth}");
        assert_eq!(
            read(&root).find(&profile.id).unwrap().wait_until,
            t + 31 + 60
        );
        assert!(open_with_pin(&root, &profile.id, "123456", t + 92).is_ok());
        assert_eq!(read(&root).find(&profile.id).unwrap().failures, 0);
        assert_eq!(wait_after(100), LONGEST_WAIT_SECONDS);
    }

    /// The wallet from before profiles becomes the first profile, signed in, with every personal file in its folder,
    /// the Library's index in its local folder, ADR-098's file gone and the device's settings left where they were.
    /// A second run changes nothing.
    #[test]
    fn the_wallet_from_before_profiles_becomes_the_first_profile() {
        let (root, local) = dirs();
        assert_eq!(
            migrate(&root, &local).unwrap(),
            None,
            "no wallet, nothing to move"
        );
        for file in [
            "vault.qor",
            "avatar.bin",
            "trade-partners.json",
            "paid-requests.json",
            "account-owners.json",
            "settings.json",
        ] {
            std::fs::write(root.join(file), file).unwrap();
        }
        std::fs::create_dir_all(root.join("mesh")).unwrap();
        std::fs::write(root.join("mesh").join("settings.json"), "mesh").unwrap();
        std::fs::create_dir_all(local.join("library")).unwrap();
        std::fs::create_dir_all(local.join("translations")).unwrap();

        let id = migrate(&root, &local).unwrap().expect("a profile");
        let home = dir(&root, &id);
        for file in [
            "vault.qor",
            "avatar.bin",
            "trade-partners.json",
            "paid-requests.json",
        ] {
            assert_eq!(std::fs::read_to_string(home.join(file)).unwrap(), file);
            assert!(!root.join(file).exists(), "{file} moved");
        }
        assert!(home.join("mesh").join("settings.json").is_file());
        assert!(local_dir(&local, &id).join("library").is_dir());
        assert!(
            local.join("translations").is_dir(),
            "translation models are the device's"
        );
        assert!(root.join("settings.json").is_file(), "device settings stay");
        assert!(!root.join("account-owners.json").exists());
        let registry = read(&root);
        assert_eq!(registry.signed_in.as_deref(), Some(id.as_str()));
        assert_eq!(registry.profiles.len(), 1);
        assert_eq!(migrate(&root, &local).unwrap(), None);
    }

    #[test]
    fn a_profile_is_named_and_listed_without_anything_secret() {
        let (root, local) = dirs();
        let a = add(&root, &local).unwrap();
        let b = add(&root, &local).unwrap();
        note(&root, &a.id, Some("5Address"), Some("yaldobaoth")).unwrap();
        set_pin(&root, &a.id, "key", "123456").unwrap();
        set_signed_in(&root, Some(&b.id)).unwrap();
        let views = read(&root).views(now());
        assert_eq!(views.len(), 2);
        assert_eq!(views[0].name.as_deref(), Some("yaldobaoth"));
        assert!(views[0].has_pin && !views[0].signed_in);
        assert!(!views[1].has_pin && views[1].signed_in);
        let json = serde_json::to_string(&views).unwrap();
        assert!(
            !json.contains("sealed") && !json.contains("ciphertext"),
            "{json}"
        );
    }
}
