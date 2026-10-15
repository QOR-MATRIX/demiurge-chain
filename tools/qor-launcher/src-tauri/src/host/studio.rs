//! What QQ Studio may ask of the launcher (P3.4, ADR-088): commit a project, and publish it as its Cartridge.
//!
//! - `qontrol.commit` `{project, message}`: everything changed in the project, committed through Qontrol with the
//!   same guard as the launcher's Projects surface. A file the guard warns about (a key, a large file) is not committed
//!   from here: the answer names it, and the person commits it in Projects, where each warning is read and accepted.
//!   Nothing changed is not an error: the answer's `commit` is null.
//! - `cartridge.publish` `{project}`: the commit HEAD points at, minted as the project's Cartridge the first time and
//!   revising it each time after (ADR-088 decisions 1 and 2), behind the launcher's dialog, which names QQ Studio and
//!   the account that signs. The Cartridge is the asset any of the vault's accounts holds whose commit is nearest HEAD
//!   in the project's history; if that is already HEAD, nothing is asked and the answer says `unchanged`. A revision is
//!   signed by the account holding the Cartridge, the only one that can revise it; a first publish by the account the
//!   launcher's rail shows (ADR-090).

use std::path::PathBuf;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::{json, Value};

use super::session::params;
use super::{Handler, ProductDialog, ProductRecord, Refusal};
use crate::chain::assets::{MintRequest, OwnedAsset};
use crate::chain::ChainClient;
use crate::error::QorError;
use crate::qontrol::{self, QontrolError};
use crate::vault::{Vault, VaultStatus};
use crate::{content, Confirm};

/// The methods QQ Studio is offered, announced in the handshake.
pub const CAPABILITIES: &[&str] = &["qontrol.commit", "cartridge.publish"];

/// What QQ Studio's requests reach: the launcher's own vault, chain client and data folder, and its dialog (the host's,
/// in the launcher; a scripted one in the tests).
pub struct StudioHandler {
    pub product: ProductRecord,
    pub vault: Arc<Vault>,
    pub chain: Arc<ChainClient>,
    pub data_dir: PathBuf,
    pub confirm: Arc<dyn Confirm>,
}

#[derive(Deserialize)]
struct CommitParams {
    project: String,
    message: String,
}

#[derive(Deserialize)]
struct PublishParams {
    project: String,
}

fn refused(error: QontrolError) -> Refusal {
    Refusal::from(QorError::from(error))
}

impl Handler for StudioHandler {
    fn capabilities(&self) -> &'static [&'static str] {
        CAPABILITIES
    }

    async fn handle(&self, method: &str, params_value: Value) -> Result<Value, Refusal> {
        match method {
            "qontrol.commit" => commit(params(params_value)?),
            "cartridge.publish" => self.publish(params(params_value)?).await,
            other => Err(Refusal::new(
                super::protocol::code::UNSUPPORTED_METHOD,
                format!("{other:?} is not offered to QQ Studio"),
            )),
        }
    }
}

fn commit(request: CommitParams) -> Result<Value, Refusal> {
    let path = qontrol::resolve(&request.project).map_err(refused)?;
    match qontrol::commit_selected(&path, &request.message, None, &[]) {
        Ok(id) => Ok(json!({ "commit": id })),
        Err(QontrolError::NothingToCommit) => Ok(json!({ "commit": null })),
        Err(QontrolError::Unconfirmed(which)) => Err(Refusal::new(
            "qontrol",
            format!(
                "these need a look before they are committed: {which}. Commit them in the launcher's Projects, \
                 where each warning can be read and accepted, then publish again."
            ),
        )),
        Err(other) => Err(refused(other)),
    }
}

/// Of the assets `held` (each with the account holding it), the one whose commit is nearest HEAD in the project's
/// history, if any.
fn cartridge_of<'a>(
    path: &std::path::Path,
    held: &'a [(String, OwnedAsset)],
) -> Result<Option<&'a (String, OwnedAsset)>, Refusal> {
    let pinned: Vec<&(String, OwnedAsset)> = held
        .iter()
        .filter(|(_, asset)| asset.commit.is_some())
        .collect();
    let commits: Vec<String> = pinned
        .iter()
        .filter_map(|(_, asset)| asset.commit.as_ref().map(|c| c.id.clone()))
        .collect();
    Ok(qontrol::publish::nearest_in_history(path, &commits)
        .map_err(refused)?
        .map(|index| pinned[index]))
}

impl StudioHandler {
    async fn publish(&self, request: PublishParams) -> Result<Value, Refusal> {
        let path = qontrol::resolve(&request.project).map_err(refused)?;
        let snapshot = qontrol::publish::snapshot(&path).map_err(refused)?;

        let accounts = match self.vault.status() {
            VaultStatus::Unlocked { accounts } => accounts,
            _ => return Err(Refusal::new(
                "vault_locked",
                "the launcher's vault is not open. Open it in the launcher, then publish again.",
            )),
        };

        // Whichever of the vault's accounts holds the Cartridge revises it; a first publish signs as the account the
        // rail shows (ADR-090).
        let mut held = Vec::new();
        for account in &accounts {
            for asset in self.chain.assets_of(&account.address).await? {
                held.push((account.address.clone(), asset));
            }
        }
        let cartridge = cartridge_of(&path, &held)?;
        let from = match cartridge {
            Some((holder, _)) => holder.clone(),
            None => self.vault.active_address()?,
        };
        let cartridge = cartridge.map(|(_, asset)| asset);
        let request = MintRequest {
            name: snapshot.name.clone(),
            reference: snapshot.reference,
            commit: snapshot.commit,
            branch: snapshot.branch.clone(),
            files: snapshot.files(),
        };
        let store = content::TemporaryStore::in_data_dir(&self.data_dir);
        let dialog = ProductDialog {
            inner: &*self.confirm,
            product: &self.product,
        };
        let write = || snapshot.write_to(&path, &store).map_err(QorError::from);

        match cartridge {
            Some(asset) if asset.commit.as_ref().map(|c| c.id.as_str()) == Some(snapshot.commit.hex().as_str()) => {
                Ok(json!({
                    "action": "unchanged",
                    "collection": asset.collection,
                    "item": asset.item,
                    "name": asset.name,
                    "commit": snapshot.commit.hex(),
                }))
            }
            Some(asset) if !asset.revisable => Err(Refusal::new(
                "qontrol",
                format!(
                    "this project's Cartridge, {} ({}/{}), is permanent, so it cannot be published again.",
                    asset.name, asset.collection, asset.item
                ),
            )),
            Some(asset) => {
                let receipt = self.chain.revise(&self.vault, &dialog, &from, asset, &request, write).await?;
                Ok(json!({ "action": "revised", "receipt": receipt }))
            }
            None => {
                let receipt = self.chain.mint(&self.vault, &dialog, &from, &request, write).await?;
                Ok(json!({ "action": "minted", "receipt": receipt }))
            }
        }
    }
}
