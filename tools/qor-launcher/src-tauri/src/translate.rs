//! Translation on this PC (ADR-092 decision 3).
//!
//! Social's World and public rooms are shown in the reader's language by Mozilla's Bergamot engine, the one Firefox
//! translates pages with, running in the webview as WebAssembly with Mozilla's per-language models. This side of it
//! finds what a pair of languages needs in Mozilla's public Remote Settings collections, downloads each file once,
//! checks it against the SHA-256 Mozilla publishes before and after unpacking, keeps it by its hash, and hands its bytes
//! to the interface. Nothing is paid per message, and no message leaves the PC to be translated: the words never reach
//! this module except to find which language they are in ([`detect`]), and are never logged.
//!
//! The engine is pinned by [`ENGINE_SHA256`]: it is the build the loader in `public/translate/` was made with, and a new
//! one arrives only with a launcher update. Models are taken as Mozilla lists them, newest first within major version
//! [`MODEL_MAJOR`], the version that engine reads.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::error::{QorError, QorResult};

/// Mozilla's Remote Settings, where Firefox finds its translation engine and models.
pub const SETTINGS: &str =
    "https://firefox.settings.services.mozilla.com/v1/buckets/main/collections";
/// Where the files those records name are served.
pub const ATTACHMENTS: &str = "https://firefox-settings-attachments.cdn.mozilla.net";
const MODELS: &str = "translations-models-v2";
const ENGINES: &str = "translations-wasm-v2";
/// `bergamot-translator` v0.6.0 (Remote Settings version 4.0), by the SHA-256 of its unpacked bytes: the engine the
/// vendored `public/translate/bergamot-translator.js` was built with.
pub const ENGINE_SHA256: &str = "f38ef807636a7c994afedaab7ff8ffe0d590d21897f05139f747b80fd7bbe926";
/// The major version of the models that engine reads.
pub const MODEL_MAJOR: u32 = 3;
/// The language every model is to or from; two others are joined through it.
pub const PIVOT: &str = "en";
/// No file larger than this is accepted, whatever a record says (the largest model in October 2026 was 50 MB).
const FILE_MAX: u64 = 256 * 1024 * 1024;
/// How long a copy of Mozilla's catalogue is used before it is asked for again.
const CATALOGUE_FRESH: Duration = Duration::from_secs(24 * 3600);
/// The kinds of model file the engine takes; anything else in a record is not downloaded.
const KINDS: [&str; 5] = ["model", "lex", "vocab", "srcvocab", "trgvocab"];

// ── Mozilla's records ──────────────────────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Attachment {
    pub hash: String,
    pub size: u64,
    pub location: String,
}

/// One Remote Settings record: a file, the languages it is for, and the hashes of its packed and unpacked bytes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Record {
    pub name: String,
    pub version: String,
    #[serde(rename = "fileType", default)]
    pub file_type: Option<String>,
    #[serde(rename = "sourceLanguage", default)]
    pub source: Option<String>,
    #[serde(rename = "targetLanguage", default)]
    pub target: Option<String>,
    #[serde(rename = "decompressedHash", default)]
    pub decompressed_hash: Option<String>,
    #[serde(rename = "decompressedSize", default)]
    pub decompressed_size: Option<u64>,
    #[serde(default)]
    pub filter_expression: String,
    pub attachment: Attachment,
}

#[derive(Deserialize, Serialize)]
struct Listing {
    data: Vec<Record>,
}

/// A file the engine needs: what it is, and the hashes and sizes it is checked against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct File {
    /// `engine`, or a model file's kind: `model`, `lex`, `vocab`, `srcvocab` or `trgvocab`.
    pub kind: String,
    /// Mozilla's name for it (`model.esen.intgemm.alphas.bin`), from which the engine takes its arithmetic, as Firefox does.
    pub name: String,
    /// SHA-256 of the unpacked bytes, which is also the name it is kept under.
    pub hash: String,
    pub size: u64,
    #[serde(skip)]
    packed_hash: String,
    #[serde(skip)]
    packed_size: u64,
    #[serde(skip)]
    location: String,
}

impl File {
    /// How many bytes it is to download.
    pub fn download_size(&self) -> u64 {
        self.packed_size
    }

    fn of(kind: &str, r: &Record) -> Option<Self> {
        Some(Self {
            kind: kind.to_string(),
            name: r.name.clone(),
            hash: r.decompressed_hash.clone()?.to_ascii_lowercase(),
            size: r.decompressed_size?,
            packed_hash: r.attachment.hash.to_ascii_lowercase(),
            packed_size: r.attachment.size,
            location: r.attachment.location.clone(),
        })
    }
}

/// The files that translate one language directly into another.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Pair {
    pub from: String,
    pub to: String,
    pub files: Vec<File>,
}

/// Everything a translation from one language to another needs: one pair, or two through English, and the engine.
#[derive(Clone, Debug, Serialize)]
pub struct Plan {
    pub from: String,
    pub to: String,
    pub pairs: Vec<Pair>,
    pub engine: File,
    /// Bytes still to download before it can run.
    pub to_download: u64,
}

/// A version Mozilla has released, as numbers: `3.1` is `[3, 1]`. A prerelease (`3.0a1`) is not one.
fn released(version: &str) -> Option<Vec<u32>> {
    version.split('.').map(|p| p.parse().ok()).collect()
}

/// A record meant for a desktop Firefox on its release channel: no filter, or one that only excludes Android.
fn for_desktop(r: &Record) -> bool {
    matches!(
        r.filter_expression.trim(),
        "" | "env.appinfo.OS != 'Android'"
    )
}

/// What Mozilla offers, as read from its two collections.
#[derive(Clone, Debug, Default)]
pub struct Catalogue {
    models: Vec<Record>,
    engines: Vec<Record>,
}

impl Catalogue {
    pub fn new(models: Vec<Record>, engines: Vec<Record>) -> Self {
        Self { models, engines }
    }

    /// The pinned engine, if Mozilla still lists it.
    pub fn engine(&self) -> Option<File> {
        self.engines
            .iter()
            .filter(|r| r.name == "bergamot-translator" && for_desktop(r))
            .find(|r| {
                r.decompressed_hash
                    .as_deref()
                    .is_some_and(|h| h.eq_ignore_ascii_case(ENGINE_SHA256))
            })
            .and_then(|r| File::of("engine", r))
    }

    /// The newest released files that translate `from` into `to` directly, if Mozilla has a complete set.
    pub fn pair(&self, from: &str, to: &str) -> Option<Pair> {
        let mut newest: BTreeMap<&str, (Vec<u32>, &Record)> = BTreeMap::new();
        for r in &self.models {
            let Some(kind) = r.file_type.as_deref().filter(|k| KINDS.contains(k)) else {
                continue;
            };
            let Some(version) = released(&r.version) else {
                continue;
            };
            let fits = r.source.as_deref() == Some(from)
                && r.target.as_deref() == Some(to)
                && for_desktop(r)
                && version.first() == Some(&MODEL_MAJOR)
                && r.decompressed_size.is_some_and(|s| s <= FILE_MAX)
                && r.attachment.size <= FILE_MAX;
            if fits && newest.get(kind).is_none_or(|(held, _)| version > *held) {
                newest.insert(kind, (version, r));
            }
        }
        let has = |k: &str| newest.contains_key(k);
        if !has("model") || !(has("vocab") || (has("srcvocab") && has("trgvocab"))) {
            return None;
        }
        let files = newest
            .iter()
            .map(|(kind, (_, r))| File::of(kind, r))
            .collect::<Option<Vec<_>>>()?;
        Some(Pair {
            from: from.to_string(),
            to: to.to_string(),
            files,
        })
    }

    /// The pairs that take `from` to `to`: one, or two through English. None for the same language, or one Mozilla
    /// cannot reach.
    pub fn route(&self, from: &str, to: &str) -> Option<Vec<Pair>> {
        if from == to {
            return None;
        }
        if let Some(direct) = self.pair(from, to) {
            return Some(vec![direct]);
        }
        if from == PIVOT || to == PIVOT {
            return None;
        }
        Some(vec![self.pair(from, PIVOT)?, self.pair(PIVOT, to)?])
    }

    /// Every language that can be translated to and from: those with models both ways through English, and English.
    pub fn languages(&self) -> Vec<String> {
        let mut found = BTreeSet::new();
        for r in &self.models {
            if let (Some(s), Some(t)) = (r.source.as_deref(), r.target.as_deref()) {
                for (other, back) in [(s, t), (t, s)] {
                    if back == PIVOT
                        && other != PIVOT
                        && self.pair(other, PIVOT).is_some()
                        && self.pair(PIVOT, other).is_some()
                    {
                        found.insert(other.to_string());
                    }
                }
            }
        }
        if !found.is_empty() {
            found.insert(PIVOT.to_string());
        }
        found.into_iter().collect()
    }
}

// ── languages ──────────────────────────────────────────────────────────────────────────────────────────────────────

/// The model language a language tag reads as, among `known`: `pt-BR` is `pt`, `zh-TW` is `zh-Hant`, `no` is `nb`.
/// None when Mozilla has nothing for it.
pub fn model_language(tag: &str, known: &[String]) -> Option<String> {
    let parts: Vec<String> = tag
        .trim()
        .split(['-', '_'])
        .map(str::to_ascii_lowercase)
        .collect();
    let primary = parts.first()?.as_str();
    let chosen = match primary {
        "zh" => {
            let traditional = parts[1..]
                .iter()
                .any(|p| matches!(p.as_str(), "hant" | "tw" | "hk" | "mo"));
            if traditional { "zh-Hant" } else { "zh-Hans" }.to_string()
        }
        "no" | "nb" => "nb".to_string(),
        "iw" => "he".to_string(),
        "in" => "id".to_string(),
        "fil" => "tl".to_string(),
        other => other.to_string(),
    };
    known.contains(&chosen).then_some(chosen)
}

/// The language a person's PC is set to, as a tag (`en-GB`), if it says.
pub fn system_language() -> Option<String> {
    sys_locale::get_locale()
}

/// The language `text` is written in, as a tag, when there are enough words to tell; None otherwise, and the sender's
/// own language stands in. Runs on this PC; the words are neither kept nor logged.
pub fn detect(text: &str) -> Option<String> {
    if text.chars().filter(|c| c.is_alphabetic()).count() < 4 {
        return None;
    }
    let info = whatlang::detect(text)?;
    if !info.is_reliable() {
        return None;
    }
    Some(tag_of(info.lang()).to_string())
}

/// whatlang's languages as the tags Social keeps: two letters where ISO 639-1 has them.
fn tag_of(lang: whatlang::Lang) -> &'static str {
    match lang.code() {
        "afr" => "af",
        "aka" => "ak",
        "amh" => "am",
        "ara" => "ar",
        "aze" => "az",
        "bel" => "be",
        "ben" => "bn",
        "bul" => "bg",
        "cat" => "ca",
        "ces" => "cs",
        "cmn" => "zh-Hans",
        "cym" => "cy",
        "dan" => "da",
        "deu" => "de",
        "ell" => "el",
        "eng" => "en",
        "epo" => "eo",
        "est" => "et",
        "fin" => "fi",
        "fra" => "fr",
        "guj" => "gu",
        "heb" => "he",
        "hin" => "hi",
        "hrv" => "hr",
        "hun" => "hu",
        "hye" => "hy",
        "ind" => "id",
        "ita" => "it",
        "jav" => "jv",
        "jpn" => "ja",
        "kan" => "kn",
        "kat" => "ka",
        "khm" => "km",
        "kor" => "ko",
        "lat" => "la",
        "lav" => "lv",
        "lit" => "lt",
        "mal" => "ml",
        "mar" => "mr",
        "mkd" => "mk",
        "mya" => "my",
        "nep" => "ne",
        "nld" => "nl",
        "nob" => "nb",
        "ori" => "or",
        "pan" => "pa",
        "pes" => "fa",
        "pol" => "pl",
        "por" => "pt",
        "ron" => "ro",
        "rus" => "ru",
        "sin" => "si",
        "slk" => "sk",
        "slv" => "sl",
        "sna" => "sn",
        "spa" => "es",
        "srp" => "sr",
        "swe" => "sv",
        "tam" => "ta",
        "tel" => "te",
        "tgl" => "tl",
        "tha" => "th",
        "tuk" => "tk",
        "tur" => "tr",
        "ukr" => "uk",
        "urd" => "ur",
        "uzb" => "uz",
        "vie" => "vi",
        "yid" => "yi",
        "zul" => "zu",
        other => other,
    }
}

// ── files on this PC ───────────────────────────────────────────────────────────────────────────────────────────────

/// What a download reports as it goes, for the interface to show.
pub trait Progress: Send + Sync {
    fn progress(&self, file: &File, done: u64);
}

/// Reports nothing.
pub struct Quiet;

impl Progress for Quiet {
    fn progress(&self, _: &File, _: u64) {}
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn refused(what: &str) -> QorError {
    QorError::Translation(what.to_string())
}

/// Unpack a zstd file to exactly `size` bytes; anything that unpacks larger or smaller is refused, so a corrupt or
/// hostile file cannot fill memory or the disk.
pub fn unpack(packed: &[u8], size: u64) -> QorResult<Vec<u8>> {
    let decoder = zstd::stream::read::Decoder::new(packed)
        .map_err(|_| refused("a translation file could not be unpacked"))?;
    let out = capped(decoder, size)?;
    if out.len() as u64 != size {
        return Err(refused(
            "a translation file unpacked to a size other than Mozilla publishes, and was refused",
        ));
    }
    Ok(out)
}

/// Read at most one byte more than `size`: enough to tell a file that unpacks larger than published, and never more,
/// however large it would unpack.
fn capped(reader: impl std::io::Read, size: u64) -> QorResult<Vec<u8>> {
    let mut out = Vec::with_capacity(usize::try_from(size.min(FILE_MAX)).unwrap_or(0));
    reader
        .take(size.saturating_add(1))
        .read_to_end(&mut out)
        .map_err(|_| refused("a translation file could not be unpacked"))?;
    Ok(out)
}

/// A file's name on disk is its hash: 64 lower-case hex digits and nothing else, so no name reaches outside the folder.
fn is_hash(name: &str) -> bool {
    name.len() == 64
        && name
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Mozilla's catalogue, the downloads, and the folder they are kept in.
pub struct Translator {
    http: reqwest::Client,
    settings: String,
    attachments: String,
    dir: PathBuf,
    catalogue: tokio::sync::Mutex<Option<Catalogue>>,
    /// Hashes checked since the launcher started: read without hashing again.
    verified: parking_lot::Mutex<HashSet<String>>,
    /// One download at a time, so two screens asking for one file fetch it once.
    downloading: tokio::sync::Mutex<()>,
}

impl Translator {
    pub fn new(dir: PathBuf) -> QorResult<Self> {
        Self::with_sources(dir, SETTINGS, ATTACHMENTS)
    }

    /// With other addresses for Mozilla's two services: the tests' stand-ins, or a mirror.
    pub fn with_sources(dir: PathBuf, settings: &str, attachments: &str) -> QorResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!("QorLauncher/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| QorError::Internal(format!("cannot build http client: {e}")))?;
        Ok(Self {
            http,
            settings: settings.trim_end_matches('/').to_string(),
            attachments: attachments.trim_end_matches('/').to_string(),
            dir,
            catalogue: tokio::sync::Mutex::new(None),
            verified: parking_lot::Mutex::new(HashSet::new()),
            downloading: tokio::sync::Mutex::new(()),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// One collection's records: a copy younger than a day; else Mozilla's, kept; else any copy, however old.
    async fn listing(&self, collection: &str) -> QorResult<Vec<Record>> {
        let copy = self.dir.join(format!("{collection}.json"));
        let fresh = std::fs::metadata(&copy)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .is_some_and(|age| age < CATALOGUE_FRESH);
        let read_copy = || {
            std::fs::read(&copy)
                .ok()
                .and_then(|b| serde_json::from_slice::<Listing>(&b).ok())
                .map(|l| l.data)
        };
        if fresh {
            if let Some(records) = read_copy() {
                return Ok(records);
            }
        }
        let url = format!("{}/{collection}/records", self.settings);
        let fetched = async {
            let response = self
                .http
                .get(&url)
                .send()
                .await
                .ok()?
                .error_for_status()
                .ok()?;
            let bytes = response.bytes().await.ok()?;
            let listing: Listing = serde_json::from_slice(&bytes).ok()?;
            let _ = std::fs::create_dir_all(&self.dir);
            let _ = std::fs::write(&copy, &bytes);
            Some(listing.data)
        }
        .await;
        fetched
            .or_else(read_copy)
            .ok_or_else(|| refused("Mozilla's list of translation models could not be reached"))
    }

    /// What Mozilla offers, read once per run.
    pub async fn catalogue(&self) -> QorResult<Catalogue> {
        let mut held = self.catalogue.lock().await;
        if let Some(c) = held.as_ref() {
            return Ok(c.clone());
        }
        let catalogue = Catalogue::new(self.listing(MODELS).await?, self.listing(ENGINES).await?);
        *held = Some(catalogue.clone());
        Ok(catalogue)
    }

    /// The languages that can be translated, and which of them this PC is set to.
    pub async fn languages(&self) -> QorResult<Vec<String>> {
        Ok(self.catalogue().await?.languages())
    }

    fn present(&self, file: &File) -> bool {
        self.verified.lock().contains(&file.hash) || self.dir.join(&file.hash).exists()
    }

    /// What translating `from` into `to` needs, by language tags, and how much of it is still to download. None when
    /// the two read as one language, or Mozilla cannot translate between them.
    pub async fn plan(&self, from: &str, to: &str) -> QorResult<Option<Plan>> {
        let catalogue = self.catalogue().await?;
        let known = catalogue.languages();
        let (Some(from), Some(to)) = (model_language(from, &known), model_language(to, &known))
        else {
            return Ok(None);
        };
        let Some(pairs) = catalogue.route(&from, &to) else {
            return Ok(None);
        };
        let engine = catalogue.engine().ok_or_else(|| {
            refused("Mozilla no longer lists the translation engine this launcher was built for; an update brings the new one")
        })?;
        let to_download = pairs
            .iter()
            .flat_map(|p| p.files.iter())
            .chain(std::iter::once(&engine))
            .filter(|f| !self.present(f))
            .map(|f| f.packed_size)
            .sum();
        Ok(Some(Plan {
            from,
            to,
            pairs,
            engine,
            to_download,
        }))
    }

    /// Fetch whatever of a plan is not here yet.
    pub async fn prepare(&self, plan: &Plan, progress: &dyn Progress) -> QorResult<()> {
        self.ensure(&plan.engine, progress).await?;
        for file in plan.pairs.iter().flat_map(|p| p.files.iter()) {
            self.ensure(file, progress).await?;
        }
        Ok(())
    }

    /// The file, here and checked: from the folder if its bytes still match, else from Mozilla, checked packed and
    /// unpacked, then kept.
    pub async fn ensure(&self, file: &File, progress: &dyn Progress) -> QorResult<PathBuf> {
        if !is_hash(&file.hash) || file.size > FILE_MAX || file.packed_size > FILE_MAX {
            return Err(refused(
                "Mozilla's record for a translation file is not one this launcher accepts",
            ));
        }
        let path = self.dir.join(&file.hash);
        if self.verified.lock().contains(&file.hash) && path.exists() {
            return Ok(path);
        }
        let _one = self.downloading.lock().await;
        if let Ok(bytes) = tokio::fs::read(&path).await {
            if sha256_hex(&bytes) == file.hash {
                self.verified.lock().insert(file.hash.clone());
                return Ok(path);
            }
            let _ = tokio::fs::remove_file(&path).await;
        }

        let url = format!(
            "{}/{}",
            self.attachments,
            file.location.trim_start_matches('/')
        );
        let network =
            |_| refused("a translation file could not be downloaded from Mozilla; try again later");
        let mut response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(network)?
            .error_for_status()
            .map_err(network)?;
        let too_large =
            || refused("a translation file is larger than Mozilla says, and was refused");
        if response
            .content_length()
            .is_some_and(|n| n > file.packed_size)
        {
            return Err(too_large());
        }
        let mut packed = Vec::with_capacity(usize::try_from(file.packed_size).unwrap_or(0));
        while let Some(chunk) = response.chunk().await.map_err(network)? {
            packed.extend_from_slice(&chunk);
            if packed.len() as u64 > file.packed_size {
                return Err(too_large());
            }
            progress.progress(file, packed.len() as u64);
        }
        if sha256_hex(&packed) != file.packed_hash {
            return Err(refused(
                "a translation file did not match the hash Mozilla publishes, and was refused",
            ));
        }
        let size = file.size;
        let bytes = tokio::task::spawn_blocking(move || unpack(&packed, size))
            .await
            .map_err(|e| QorError::Internal(format!("unpacking stopped: {e}")))??;
        if sha256_hex(&bytes) != file.hash {
            return Err(refused("a translation file unpacked to bytes other than Mozilla publishes, and was refused"));
        }
        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(|e| QorError::Io(e.to_string()))?;
        let partial = self.dir.join(format!("{}.part", file.hash));
        tokio::fs::write(&partial, &bytes)
            .await
            .map_err(|e| QorError::Io(e.to_string()))?;
        tokio::fs::rename(&partial, &path)
            .await
            .map_err(|e| QorError::Io(e.to_string()))?;
        self.verified.lock().insert(file.hash.clone());
        Ok(path)
    }

    /// A file's bytes, for the engine. Only a file checked since the launcher started is read.
    pub fn read(&self, hash: &str) -> QorResult<Vec<u8>> {
        if !is_hash(hash) || !self.verified.lock().contains(hash) {
            return Err(refused("that translation file has not been checked"));
        }
        std::fs::read(self.dir.join(hash)).map_err(|e| QorError::Io(e.to_string()))
    }
}

#[cfg(test)]
mod tests;
