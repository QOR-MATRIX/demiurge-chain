//! Translation's host side, against a stand-in for Mozilla's two services: the catalogue read as Firefox reads it, every
//! file checked packed and unpacked before it is kept, and nothing taken that does not match (ADR-092 decision 3).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use super::*;

// ── a stand-in for Remote Settings and its attachment server ───────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct Mozilla {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    asked: Arc<Mutex<HashMap<String, usize>>>,
    /// Paths answered without a Content-Length: the body ends when the connection does.
    without_length: Arc<Mutex<std::collections::HashSet<String>>>,
}

impl Mozilla {
    fn serve(&self, path: &str, body: Vec<u8>) {
        self.files.lock().unwrap().insert(path.to_string(), body);
    }

    fn serve_without_length(&self, path: &str, body: Vec<u8>) {
        self.serve(path, body);
        self.without_length.lock().unwrap().insert(path.to_string());
    }

    fn asked(&self, path: &str) -> usize {
        self.asked.lock().unwrap().get(path).copied().unwrap_or(0)
    }

    /// Listen on a loopback port, answering each GET from `files`, and 404 otherwise.
    async fn start(&self) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let me = self.clone();
        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let me = me.clone();
                tokio::spawn(async move {
                    let mut request = Vec::new();
                    let mut buffer = [0u8; 4096];
                    while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                        match socket.read(&mut buffer).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => request.extend_from_slice(&buffer[..n]),
                        }
                    }
                    let line = String::from_utf8_lossy(&request);
                    let path = line.split_whitespace().nth(1).unwrap_or("/").to_string();
                    *me.asked.lock().unwrap().entry(path.clone()).or_default() += 1;
                    let body = me.files.lock().unwrap().get(&path).cloned();
                    let (status, body) = match body {
                        Some(b) => ("200 OK", b),
                        None => ("404 Not Found", b"not here".to_vec()),
                    };
                    let head = if me.without_length.lock().unwrap().contains(&path) {
                        format!("HTTP/1.1 {status}\r\nconnection: close\r\n\r\n")
                    } else {
                        format!(
                            "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                            body.len()
                        )
                    };
                    let _ = socket.write_all(head.as_bytes()).await;
                    let _ = socket.write_all(&body).await;
                    let _ = socket.shutdown().await;
                });
            }
        });
        format!("http://{address}")
    }
}

fn hash(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

/// A record for `bytes`, packed with zstd and served at `location`.
fn record(
    name: &str,
    kind: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    version: &str,
    bytes: &[u8],
    location: &str,
) -> (Value, Vec<u8>) {
    let packed = zstd::stream::encode_all(bytes, 3).unwrap();
    let r = json!({
        "name": name, "version": version, "fileType": kind, "sourceLanguage": from, "targetLanguage": to,
        "decompressedHash": hash(bytes), "decompressedSize": bytes.len(), "filter_expression": "",
        "attachment": { "hash": hash(&packed), "size": packed.len(), "location": location },
    });
    (r, packed)
}

fn records(values: Vec<Value>) -> Vec<Record> {
    serde_json::from_value(Value::Array(values)).unwrap()
}

/// A complete pair's three records, with small made-up bytes standing in for a model.
fn pair_records(from: &str, to: &str, version: &str) -> Vec<Value> {
    ["model", "lex", "vocab"]
        .iter()
        .map(|kind| {
            let bytes = format!("{kind} {from}{to} {version}");
            record(
                &format!("{kind}.{from}{to}"),
                Some(kind),
                Some(from),
                Some(to),
                version,
                bytes.as_bytes(),
                &format!("m/{kind}-{from}{to}-{version}.zst"),
            )
            .0
        })
        .collect()
}

// ── the catalogue ──────────────────────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_pair_is_the_newest_released_desktop_files_of_the_major_version_the_engine_reads() {
    let mut values = pair_records("es", "en", "3.0");
    values.extend(pair_records("es", "en", "3.1"));
    // A prerelease, an older major, a newer major and an Android-only model are all passed over.
    values.extend(pair_records("es", "en", "3.2a1"));
    values.extend(pair_records("es", "en", "2.9"));
    values.extend(pair_records("es", "en", "4.0"));
    let mut android = pair_records("es", "en", "3.9");
    for r in &mut android {
        r["filter_expression"] = json!("env.appinfo.OS == 'Android'");
    }
    values.extend(android);
    let catalogue = Catalogue::new(records(values), vec![]);
    let pair = catalogue.pair("es", "en").expect("a pair");
    let kinds: Vec<&str> = pair.files.iter().map(|f| f.kind.as_str()).collect();
    assert_eq!(kinds, ["lex", "model", "vocab"]);
    let model = pair.files.iter().find(|f| f.kind == "model").unwrap();
    assert_eq!(
        model.hash,
        hash(b"model esen 3.1"),
        "the newest released 3.x"
    );
}

#[test]
fn a_pair_needs_a_model_and_its_vocabularies() {
    let mut values = pair_records("de", "en", "3.0");
    values.retain(|r| r["fileType"] != "vocab");
    let catalogue = Catalogue::new(records(values.clone()), vec![]);
    assert!(
        catalogue.pair("de", "en").is_none(),
        "no vocabulary, no pair"
    );

    for kind in ["srcvocab", "trgvocab"] {
        values.push(
            record(
                kind,
                Some(kind),
                Some("de"),
                Some("en"),
                "3.0",
                kind.as_bytes(),
                kind,
            )
            .0,
        );
    }
    let catalogue = Catalogue::new(records(values), vec![]);
    assert_eq!(
        catalogue.pair("de", "en").expect("a pair").files.len(),
        4,
        "a source and a target vocabulary will do"
    );
}

#[test]
fn two_languages_neither_of_them_english_meet_in_english() {
    let mut values = Vec::new();
    for (from, to) in [
        ("es", "en"),
        ("en", "es"),
        ("ja", "en"),
        ("en", "ja"),
        ("fr", "en"),
    ] {
        values.extend(pair_records(from, to, "3.0"));
    }
    let catalogue = Catalogue::new(records(values), vec![]);
    let route = catalogue.route("es", "ja").expect("a route");
    assert_eq!(route.len(), 2);
    assert_eq!((route[0].from.as_str(), route[0].to.as_str()), ("es", "en"));
    assert_eq!((route[1].from.as_str(), route[1].to.as_str()), ("en", "ja"));
    assert_eq!(catalogue.route("es", "en").expect("direct").len(), 1);
    assert!(
        catalogue.route("es", "es").is_none(),
        "nothing to translate"
    );
    assert_eq!(
        catalogue.route("fr", "es").expect("out of French").len(),
        2,
        "French reaches Spanish through English"
    );
    assert!(
        catalogue.route("es", "fr").is_none(),
        "nothing translates into French: it has no model from English"
    );
    assert!(catalogue.route("ko", "en").is_none());
    assert_eq!(
        catalogue.languages(),
        ["en", "es", "ja"],
        "only languages with models both ways"
    );
}

#[test]
fn the_engine_is_the_one_pinned_and_no_other() {
    let pinned = json!({
        "name": "bergamot-translator", "version": "4.0", "decompressedHash": ENGINE_SHA256, "decompressedSize": 10,
        "filter_expression": "", "attachment": { "hash": "00", "size": 5, "location": "e/pinned.zst" },
    });
    let mut newer = pinned.clone();
    newer["version"] = json!("5.0");
    newer["decompressedHash"] = json!("ab".repeat(32));
    assert_eq!(
        Catalogue::new(vec![], records(vec![newer.clone()])).engine(),
        None,
        "a newer engine waits for an update"
    );
    let engine = Catalogue::new(vec![], records(vec![newer, pinned]))
        .engine()
        .expect("the pinned engine");
    assert_eq!(engine.hash, ENGINE_SHA256);
    assert_eq!(engine.kind, "engine");
}

#[test]
fn a_language_tag_reads_as_the_model_language_nearest_it() {
    let known: Vec<String> = ["en", "pt", "zh-Hans", "zh-Hant", "nb", "he", "id", "tl"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    for (tag, model) in [
        ("en-US", Some("en")),
        ("pt-BR", Some("pt")),
        ("pt_PT", Some("pt")),
        ("zh-CN", Some("zh-Hans")),
        ("zh-Hans", Some("zh-Hans")),
        ("zh-TW", Some("zh-Hant")),
        ("zh-Hant-HK", Some("zh-Hant")),
        ("no", Some("nb")),
        ("nb-NO", Some("nb")),
        ("iw", Some("he")),
        ("fil", Some("tl")),
        ("ko", None),
        ("", None),
    ] {
        assert_eq!(model_language(tag, &known).as_deref(), model, "{tag:?}");
    }
}

#[test]
fn the_language_of_words_is_told_when_there_are_enough_of_them() {
    assert_eq!(
        detect("Bonjour à tous, comment allez-vous aujourd'hui ?").as_deref(),
        Some("fr")
    );
    assert_eq!(
        detect("¿Dónde está la biblioteca? Necesito un libro nuevo.").as_deref(),
        Some("es")
    );
    assert_eq!(
        detect("今天天气很好，我们一起去公园散步吧").as_deref(),
        Some("zh-Hans")
    );
    assert_eq!(detect("ok"), None, "too few letters to tell");
    assert_eq!(detect("123 456 !!!"), None);
    // Every language whatlang tells is kept as a tag Social accepts: two or three lower-case letters, or zh-Hans.
    for lang in whatlang::Lang::all() {
        let tag = tag_of(*lang);
        let ok = tag == "zh-Hans"
            || ((2..=3).contains(&tag.len()) && tag.bytes().all(|b| b.is_ascii_lowercase()));
        assert!(ok, "{lang:?} is kept as {tag:?}");
    }
}

#[test]
fn unpacking_holds_to_the_size_mozilla_publishes() {
    let bytes = vec![7u8; 10_000];
    let packed = zstd::stream::encode_all(&bytes[..], 3).unwrap();
    assert_eq!(unpack(&packed, 10_000).unwrap(), bytes);
    assert!(
        unpack(&packed, 9_999).is_err(),
        "larger than published: a bomb is stopped one byte past it"
    );
    assert!(unpack(&packed, 10_001).is_err(), "smaller than published");
    assert!(unpack(b"not zstd at all", 5).is_err());
    // A bomb is read only one byte past what is published, never to its end.
    let endless = std::io::Read::take(std::io::repeat(1), 5_000_000);
    assert_eq!(capped(endless, 10).unwrap().len(), 11);
}

// ── downloads ──────────────────────────────────────────────────────────────────────────────────────────────────────

struct Setup {
    mozilla: Mozilla,
    translator: Translator,
    _dir: tempdir::Dir,
}

mod tempdir {
    /// A folder of its own under the system's temporary folder, removed when dropped.
    pub struct Dir(pub std::path::PathBuf);
    impl Dir {
        pub fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("qor-translate-{}", uuid::Uuid::new_v4().simple()));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// Mozilla, as a stand-in, offering Spanish to and from English and the pinned engine (whose bytes cannot be the real
/// engine's, so its record names a stand-in's hash only in the tests that fetch it).
async fn setup(values: Vec<(Value, Vec<u8>)>, engines: Vec<(Value, Vec<u8>)>) -> Setup {
    let mozilla = Mozilla::default();
    let base = mozilla.start().await;
    for (r, packed) in values.iter().chain(engines.iter()) {
        mozilla.serve(
            &format!("/{}", r["attachment"]["location"].as_str().unwrap()),
            packed.clone(),
        );
    }
    let listing = |v: &[(Value, Vec<u8>)]| {
        serde_json::to_vec(&json!({ "data": v.iter().map(|(r, _)| r.clone()).collect::<Vec<_>>() }))
            .unwrap()
    };
    mozilla.serve(
        "/collections/translations-models-v2/records",
        listing(&values),
    );
    mozilla.serve(
        "/collections/translations-wasm-v2/records",
        listing(&engines),
    );
    let dir = tempdir::Dir::new();
    let translator =
        Translator::with_sources(dir.0.clone(), &format!("{base}/collections"), &base).unwrap();
    Setup {
        mozilla,
        translator,
        _dir: dir,
    }
}

fn spanish() -> Vec<(Value, Vec<u8>)> {
    let mut out = Vec::new();
    for (from, to) in [("es", "en"), ("en", "es")] {
        for kind in ["model", "lex", "vocab"] {
            let bytes = format!("{kind} for {from} to {to} ").repeat(50);
            out.push(record(
                kind,
                Some(kind),
                Some(from),
                Some(to),
                "3.0",
                bytes.as_bytes(),
                &format!("m/{kind}-{from}{to}.zst"),
            ));
        }
    }
    out
}

#[derive(Default)]
struct Counted(Mutex<u64>);
impl Progress for Counted {
    fn progress(&self, _: &File, done: u64) {
        *self.0.lock().unwrap() = done;
    }
}

#[tokio::test]
async fn a_file_is_fetched_once_checked_kept_by_hash_and_read_from_the_folder_after() {
    let s = setup(spanish(), vec![]).await;
    let catalogue = s.translator.catalogue().await.unwrap();
    assert_eq!(catalogue.languages(), ["en", "es"]);
    let file = catalogue
        .pair("es", "en")
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.kind == "model")
        .unwrap();
    assert!(
        s.translator.read(&file.hash).is_err(),
        "nothing is read before it is checked"
    );

    let progress = Counted::default();
    let path = s.translator.ensure(&file, &progress).await.unwrap();
    assert_eq!(
        path.file_name().unwrap().to_str(),
        Some(file.hash.as_str()),
        "kept by its hash"
    );
    assert_eq!(
        *progress.0.lock().unwrap(),
        file.packed_size,
        "progress reaches the whole file"
    );
    let bytes = s.translator.read(&file.hash).unwrap();
    assert_eq!(hash(&bytes), file.hash);
    s.translator.ensure(&file, &Quiet).await.unwrap();
    assert_eq!(s.mozilla.asked("/m/model-esen.zst"), 1, "fetched once");

    // A new run checks what is in the folder again, and fetches nothing.
    let again = Translator::with_sources(
        s.translator.dir().to_path_buf(),
        "http://127.0.0.1:9/collections",
        "http://127.0.0.1:9",
    )
    .unwrap();
    assert!(
        again.read(&file.hash).is_err(),
        "on disk, but not checked in this run: not read"
    );
    again.ensure(&file, &Quiet).await.unwrap();
    assert_eq!(again.read(&file.hash).unwrap(), bytes);
    // A file changed on disk is not trusted: it is fetched again.
    std::fs::write(s.translator.dir().join(&file.hash), b"tampered").unwrap();
    let third = Translator::with_sources(
        s.translator.dir().to_path_buf(),
        "http://127.0.0.1:9/collections",
        &format!("http://{}", "127.0.0.1:9"),
    )
    .unwrap();
    assert!(
        third.ensure(&file, &Quiet).await.is_err(),
        "the tampered copy is refused, and Mozilla is unreachable"
    );
    assert!(!s.translator.dir().join(&file.hash).exists(), "and removed");
}

#[tokio::test]
async fn a_file_that_does_not_match_mozillas_hashes_is_refused_and_not_kept() {
    let mut values = spanish();
    // A record that allows up to 4 KiB, so each file below is refused for its hash, never for its size.
    values[0].0["attachment"]["size"] = json!(4096);
    // Served bytes other than the record's: the packed hash fails.
    let location = format!(
        "/{}",
        values[0].0["attachment"]["location"].as_str().unwrap()
    );
    let s = setup(values.clone(), vec![]).await;
    s.mozilla.serve(
        &location,
        zstd::stream::encode_all(&b"something else"[..], 3).unwrap(),
    );
    let catalogue = s.translator.catalogue().await.unwrap();
    let model = catalogue
        .pair("es", "en")
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.kind == "model")
        .unwrap();
    let refused = s.translator.ensure(&model, &Quiet).await.unwrap_err();
    assert_eq!(refused.kind(), "translation", "{refused}");

    // The right words, packed as two frames instead of one: they unpack to exactly the published bytes, yet are not
    // the file Mozilla published, so the packed hash refuses them before the unpacker reads a byte.
    let words = "model for es to en ".repeat(50).into_bytes();
    let (first, second) = words.split_at(words.len() / 2);
    let mut repacked = zstd::stream::encode_all(first, 3).unwrap();
    repacked.extend(zstd::stream::encode_all(second, 3).unwrap());
    assert_eq!(unpack(&repacked, words.len() as u64).unwrap(), words);
    s.mozilla.serve(&location, repacked);
    let refused = s.translator.ensure(&model, &Quiet).await.unwrap_err();
    assert!(
        refused
            .to_string()
            .contains("did not match the hash Mozilla publishes"),
        "{refused}"
    );
    assert!(!s.translator.dir().join(&model.hash).exists());
    assert!(s.translator.read(&model.hash).is_err());

    // A record whose unpacked hash is wrong: the packed bytes pass, the unpacked fail.
    values[0].0["decompressedHash"] = json!("cd".repeat(32));
    let s = setup(values, vec![]).await;
    let catalogue = s.translator.catalogue().await.unwrap();
    let model = catalogue
        .pair("es", "en")
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.kind == "model")
        .unwrap();
    assert!(s.translator.ensure(&model, &Quiet).await.is_err());
    assert!(!s.translator.dir().join(&model.hash).exists());
}

#[tokio::test]
async fn a_file_larger_than_its_record_is_refused_before_it_is_unpacked() {
    let mut values = spanish();
    values[0].0["attachment"]["size"] = json!(10);
    let s = setup(values, vec![]).await;
    let catalogue = s.translator.catalogue().await.unwrap();
    let model = catalogue
        .pair("es", "en")
        .unwrap()
        .files
        .into_iter()
        .find(|f| f.kind == "model")
        .unwrap();
    let refused = s.translator.ensure(&model, &Quiet).await.unwrap_err();
    assert!(
        refused.to_string().contains("larger than Mozilla says"),
        "{refused}"
    );

    // Without a Content-Length to read first, the count of bytes as they arrive stops it.
    let served = s.mozilla.files.lock().unwrap()["/m/model-esen.zst"].clone();
    s.mozilla.serve_without_length("/m/model-esen.zst", served);
    let refused = s.translator.ensure(&model, &Quiet).await.unwrap_err();
    assert!(
        refused.to_string().contains("larger than Mozilla says"),
        "{refused}"
    );
}

#[tokio::test]
async fn a_plan_says_what_is_left_to_download_and_preparing_fetches_it() {
    let engine_bytes = b"a stand-in for the engine".to_vec();
    let (mut engine, packed) = record(
        "bergamot-translator",
        None,
        None,
        None,
        "4.0",
        &engine_bytes,
        "e/engine.zst",
    );
    // The stand-in's own bytes cannot hash to the pinned engine; the plan is checked up to the engine's download.
    engine["decompressedHash"] = json!(ENGINE_SHA256);
    let s = setup(spanish(), vec![(engine, packed)]).await;

    let plan = s
        .translator
        .plan("es-MX", "en-GB")
        .await
        .unwrap()
        .expect("Spanish to English");
    assert_eq!((plan.from.as_str(), plan.to.as_str()), ("es", "en"));
    assert_eq!(plan.pairs.len(), 1);
    let model_bytes: u64 = plan.pairs[0].files.iter().map(|f| f.packed_size).sum();
    assert_eq!(plan.to_download, model_bytes + plan.engine.packed_size);
    assert!(
        s.translator.plan("en", "en-US").await.unwrap().is_none(),
        "the same language"
    );
    assert!(
        s.translator.plan("ko", "en").await.unwrap().is_none(),
        "no models for it"
    );

    // The models arrive; the stand-in engine is refused, since its bytes are not the pinned engine's.
    assert!(s.translator.prepare(&plan, &Quiet).await.is_err());
    for f in &plan.pairs[0].files {
        s.translator.ensure(f, &Quiet).await.unwrap();
    }
    let plan = s.translator.plan("es", "en").await.unwrap().unwrap();
    assert_eq!(
        plan.to_download, plan.engine.packed_size,
        "only the engine is left"
    );
}

#[tokio::test]
async fn the_catalogue_is_read_from_a_recent_copy_or_mozilla_or_any_copy_at_all() {
    let s = setup(spanish(), vec![]).await;
    s.translator.catalogue().await.unwrap();
    assert_eq!(
        s.mozilla
            .asked("/collections/translations-models-v2/records"),
        1
    );

    // Another run within the day uses the copy, and does not ask.
    let dir = s.translator.dir().to_path_buf();
    let base = format!("http://127.0.0.1:{}", 9);
    let offline =
        Translator::with_sources(dir.clone(), &format!("{base}/collections"), &base).unwrap();
    assert_eq!(offline.languages().await.unwrap(), ["en", "es"]);

    // With no copy and no Mozilla, it says so.
    let empty = tempdir::Dir::new();
    let nowhere =
        Translator::with_sources(empty.0.clone(), &format!("{base}/collections"), &base).unwrap();
    let refused = nowhere.catalogue().await.unwrap_err();
    assert_eq!(refused.kind(), "translation");
}

#[test]
fn only_a_hash_names_a_file() {
    assert!(is_hash(&"a".repeat(64)));
    for bad in [
        "../vault.qor",
        "A".repeat(64).as_str(),
        "a".repeat(63).as_str(),
        "",
        "g".repeat(64).as_str(),
    ] {
        assert!(!is_hash(bad), "{bad}");
    }
    let dir = tempdir::Dir::new();
    let t = Translator::new(dir.0.clone()).unwrap();
    assert!(t.read("../../vault.qor").is_err());
}

/// What a person writes passes through here to have its language told, and a model's bytes pass through to the engine;
/// neither reaches the log, at any level (AGENTS.md section 9, as the launcher keeps it for Social's words).
#[tokio::test]
async fn nothing_a_person_writes_reaches_the_log_at_any_level() {
    let captured = crate::testing::captured_log();
    let from = captured.0.lock().unwrap().len();
    let words = format!(
        "Ceci est une phrase privée que seul le salon devrait lire {}",
        std::process::id()
    );
    assert_eq!(detect(&words).as_deref(), Some("fr"));

    let s = setup(spanish(), vec![]).await;
    let catalogue = s.translator.catalogue().await.unwrap();
    for file in catalogue.pair("es", "en").unwrap().files {
        s.translator.ensure(&file, &Quiet).await.unwrap();
        s.translator.read(&file.hash).unwrap();
    }
    let _ = s.translator.read("../../vault.qor");

    let log = String::from_utf8_lossy(&captured.0.lock().unwrap()[from..]).into_owned();
    assert!(!log.contains(&words), "a person's words reached the log");
    assert!(
        !log.contains("model for es to en"),
        "a model's bytes reached the log"
    );
}

/// Mozilla's real catalogue, downloaded and checked by this module: the pinned engine and Spanish to English, kept in
/// `QOR_TRANSLATE_DIR` (or a temporary folder) with the plan beside them, for `scripts/check-translate-engine.mjs` to
/// run in a browser. It reaches Mozilla, so it is run by hand: `cargo test --lib real_mozilla -- --ignored`.
#[tokio::test]
#[ignore = "downloads about 30 MB from Mozilla"]
async fn real_mozilla_files_download_check_and_plan() {
    let dir = std::env::var("QOR_TRANSLATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir().join("qor-translate-real"));
    let translator = Translator::new(dir.clone()).unwrap();
    let languages = translator.languages().await.unwrap();
    assert!(languages.len() > 40, "{languages:?}");
    let plan = translator
        .plan("es", "en-US")
        .await
        .unwrap()
        .expect("Spanish to English");
    assert_eq!(plan.pairs.len(), 1);
    let pivot = translator
        .plan("es", "ja")
        .await
        .unwrap()
        .expect("Spanish to Japanese");
    assert_eq!(pivot.pairs.len(), 2, "through English");
    translator.prepare(&plan, &Quiet).await.unwrap();
    for f in plan.pairs[0]
        .files
        .iter()
        .chain(std::iter::once(&plan.engine))
    {
        assert_eq!(hash(&translator.read(&f.hash).unwrap()), f.hash);
    }
    std::fs::write(
        dir.join("plan.json"),
        serde_json::to_vec_pretty(&plan).unwrap(),
    )
    .unwrap();
    println!("prepared in {}", dir.display());
}
