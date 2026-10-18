//! The object model, host side (ADR-047): what an asset *is*, computed here and
//! never on chain.
//!
//! An asset is the BLAKE3-256 hash of a **manifest**: a list of the files it is
//! made of, each named by its own BLAKE3-256 hash. The chain stores 41 bytes
//! about it — an algorithm tag, the 32-byte root, and the manifest's length — and
//! compares them; it never computes one (ADR-047 decisions 1 to 4).
//!
//! # Canonical, or it is two assets
//!
//! Two encoders of the same content must produce the same bytes, or one work has
//! two identities (decision 5). So:
//!
//! - entries are sorted by path, byte by byte, whatever order they arrived in;
//! - a path is relative, `/`-separated, and has no empty, `.` or `..` component;
//! - the encoding is SCALE, the chain's own, field by field in declaration order;
//! - `created` is the pinned commit's time, not the time of the mint, so minting
//!   the same commit twice produces the same manifest;
//! - a media type comes from a fixed table of IANA-registered types, and anything
//!   the table does not know is `application/octet-stream`, which is always true.
//!
//! # The temporary content store
//!
//! ADR-047's thirteenth question, answered by the owner on 22 September 2026: the
//! bytes go in a store **labelled temporary until the Mesh** (M8.1), and the store
//! is kept out of the wire format. [`TemporaryStore`] is that store: a folder in
//! the launcher's own data directory, beside the vault, holding each blob under its
//! BLAKE3 root. Nothing on chain names it, and nothing but this machine can read
//! it. When the Mesh exists, the same roots are what it will serve.

use std::path::{Path, PathBuf};

use codec::Encode;

/// The algorithm that produced a fingerprint (ADR-047 decision 2). The same
/// variants, in the same order, as `pallet_drc369::HashAlgo`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode)]
pub enum HashAlgo {
    #[codec(index = 0)]
    Blake3_256,
    #[codec(index = 1)]
    Sha2_256,
    #[codec(index = 2)]
    Blake2_256,
}

impl HashAlgo {
    pub fn label(self) -> &'static str {
        match self {
            HashAlgo::Blake3_256 => "BLAKE3-256",
            HashAlgo::Sha2_256 => "SHA-256",
            HashAlgo::Blake2_256 => "BLAKE2-256",
        }
    }
}

/// A fingerprint: an algorithm, a root and the length of what it hashes.
/// Encodes to exactly 41 bytes, as `pallet_drc369::ContentRef` does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode)]
pub struct ContentRef {
    pub algo: HashAlgo,
    pub root: [u8; 32],
    pub size: u64,
}

impl ContentRef {
    /// The BLAKE3-256 fingerprint of some bytes.
    pub fn of(bytes: &[u8]) -> Self {
        Self {
            algo: HashAlgo::Blake3_256,
            root: *blake3::hash(bytes).as_bytes(),
            size: bytes.len() as u64,
        }
    }

    /// The reference of the file at `path`, read a piece at a time, so a file larger than memory can be named.
    pub fn of_file(path: &Path) -> Result<Self, ContentError> {
        let file = std::fs::File::open(path).map_err(|e| {
            ContentError::Invalid(format!("{} could not be read: {e}", path.display()))
        })?;
        let mut hasher = blake3::Hasher::new();
        let size = std::io::copy(&mut std::io::BufReader::new(file), &mut hasher).map_err(|e| {
            ContentError::Invalid(format!("{} could not be read: {e}", path.display()))
        })?;
        Ok(Self {
            algo: HashAlgo::Blake3_256,
            root: *hasher.finalize().as_bytes(),
            size,
        })
    }

    pub fn root_hex(&self) -> String {
        hex::encode(self.root)
    }
}

/// What part a file plays in an asset (ADR-047 decision 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode)]
pub enum Role {
    #[codec(index = 0)]
    Primary,
    #[codec(index = 1)]
    Preview,
    #[codec(index = 2)]
    Source,
    #[codec(index = 3)]
    Component,
    #[codec(index = 4)]
    Licence,
}

/// One file in a manifest.
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct Entry {
    pub path: String,
    pub media_type: String,
    pub content: ContentRef,
    pub role: Role,
}

/// A source commit: a git object id, tagged by its hash function, as
/// `pallet_drc369::CommitId` is. Never a branch (ADR-047 decision 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode)]
pub enum CommitId {
    #[codec(index = 0)]
    Sha1([u8; 20]),
    #[codec(index = 1)]
    Sha256([u8; 32]),
}

impl CommitId {
    /// From the raw bytes of a git object id.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ContentError> {
        match bytes.len() {
            20 => Ok(CommitId::Sha1(bytes.try_into().expect("checked"))),
            32 => Ok(CommitId::Sha256(bytes.try_into().expect("checked"))),
            n => Err(ContentError::Invalid(format!(
                "a commit id is 20 or 32 bytes, not {n}"
            ))),
        }
    }

    pub fn hex(&self) -> String {
        match self {
            CommitId::Sha1(bytes) => hex::encode(bytes),
            CommitId::Sha256(bytes) => hex::encode(bytes),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            CommitId::Sha1(_) => "SHA-1",
            CommitId::Sha256(_) => "SHA-256",
        }
    }
}

/// Where the content came from (ADR-047 decision 8): the commit by hash, and,
/// for a person reading the manifest, the branch it was on.
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct SourceRef {
    pub commit: CommitId,
    pub branch: Option<String>,
}

/// The format version this code writes.
pub const MANIFEST_VERSION: u16 = 1;

/// What an asset is made of (ADR-047 decision 5).
#[derive(Clone, Debug, PartialEq, Eq, Encode)]
pub struct Manifest {
    pub version: u16,
    pub entries: Vec<Entry>,
    /// Seconds since the epoch: the pinned commit's time, so the same commit
    /// always makes the same manifest.
    pub created: u64,
    pub source: Option<SourceRef>,
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("{0}")]
    Invalid(String),
    #[error("the temporary content store could not be written: {0}")]
    Store(String),
}

impl From<ContentError> for crate::error::QorError {
    fn from(error: ContentError) -> Self {
        crate::error::QorError::Qontrol(error.to_string())
    }
}

impl Manifest {
    /// Build the canonical manifest for a set of files, in whatever order they
    /// arrive.
    pub fn new(
        files: Vec<(String, ContentRef)>,
        created: u64,
        source: Option<SourceRef>,
    ) -> Result<Self, ContentError> {
        if files.is_empty() {
            return Err(ContentError::Invalid(
                "there are no files to make an asset of".into(),
            ));
        }

        let mut entries = files
            .into_iter()
            .map(|(path, content)| {
                check_path(&path)?;
                Ok(Entry {
                    media_type: media_type(&path).to_string(),
                    role: role(&path),
                    path,
                    content,
                })
            })
            .collect::<Result<Vec<_>, ContentError>>()?;

        // Byte order, which is what `str`'s `Ord` is: independent of locale and
        // of the order the files were read in.
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        if let Some(pair) = entries.windows(2).find(|pair| pair[0].path == pair[1].path) {
            return Err(ContentError::Invalid(format!(
                "{} appears twice",
                pair[0].path
            )));
        }

        Ok(Self {
            version: MANIFEST_VERSION,
            entries,
            created,
            source,
        })
    }

    /// The canonical bytes: SCALE, as the chain encodes.
    pub fn bytes(&self) -> Vec<u8> {
        self.encode()
    }

    /// The asset's reference: the BLAKE3-256 of the canonical bytes, and their
    /// length.
    pub fn reference(&self) -> ContentRef {
        ContentRef::of(&self.bytes())
    }
}

/// A path as a manifest holds it: relative, `/`-separated, nothing that could
/// climb out of the asset or mean two things.
fn check_path(path: &str) -> Result<(), ContentError> {
    let refuse = |why: &str| Err(ContentError::Invalid(format!("{path:?} {why}")));
    if path.is_empty() {
        return refuse("is empty");
    }
    if path.contains('\\') || path.contains('\0') {
        return refuse("is not a relative path with / between its parts");
    }
    if path
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return refuse("has an empty, . or .. part");
    }
    Ok(())
}

/// A media type from a fixed table of IANA-registered types. Anything else is
/// `application/octet-stream`, which is never wrong.
pub fn media_type(path: &str) -> &'static str {
    let name = path.rsplit('/').next().unwrap_or(path);
    let extension = match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => return "application/octet-stream",
    };
    match extension.as_str() {
        // Text and documents.
        "txt" => "text/plain",
        "md" => "text/markdown",
        "csv" => "text/csv",
        "html" | "htm" => "text/html",
        "css" => "text/css",
        "js" | "mjs" => "text/javascript",
        "json" => "application/json",
        "xml" => "application/xml",
        "pdf" => "application/pdf",
        "rtf" => "application/rtf",
        "epub" => "application/epub+zip",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application/vnd.oasis.opendocument.text",
        "ods" => "application/vnd.oasis.opendocument.spreadsheet",
        "odp" => "application/vnd.oasis.opendocument.presentation",
        // Archives, and code that ships as bytes.
        "zip" => "application/zip",
        "gz" => "application/gzip",
        "zst" => "application/zstd",
        "wasm" => "application/wasm",
        // Images.
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "avif" => "image/avif",
        "heic" => "image/heic",
        "tif" | "tiff" => "image/tiff",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        // Sound.
        "wav" => "audio/vnd.wave",
        "flac" => "audio/flac",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "opus" => "audio/ogg",
        "aac" => "audio/aac",
        "m4a" => "audio/mp4",
        // Moving pictures.
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "mpeg" | "mpg" => "video/mpeg",
        // Models and scenes.
        "gltf" => "model/gltf+json",
        "glb" => "model/gltf-binary",
        "obj" => "model/obj",
        "stl" => "model/stl",
        "usdz" => "model/vnd.usdz+zip",
        "dae" => "model/vnd.collada+xml",
        // Fonts.
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        // Everything else, including a project's own formats: .blend, .fbx,
        // .als, .tscn, .psd and the rest have no registered type, and inventing
        // one here would put a name the registry does not know into a manifest
        // that is wire format after the freeze.
        _ => "application/octet-stream",
    }
}

/// A licence file at the top of the project is the asset's licence; every
/// other file is a component of the work.
fn role(path: &str) -> Role {
    if path.contains('/') {
        return Role::Component;
    }
    let stem = path.split('.').next().unwrap_or(path).to_ascii_uppercase();
    match stem.as_str() {
        "LICENSE" | "LICENCE" | "COPYING" => Role::Licence,
        _ => Role::Component,
    }
}

/// The store the owner named for bytes before the Mesh exists (ADR-047 row 13).
pub struct TemporaryStore {
    dir: PathBuf,
}

/// The folder's name says what it is, so nobody mistakes it for somewhere safe.
pub const STORE_DIR: &str = "temporary-content-store";

const STORE_README: &str = "\
TEMPORARY. This folder holds the bytes of assets minted from this launcher, each
file named by its BLAKE3-256 hash, until the Mesh exists (roadmap M8.1).

Nothing on chain points here, and nobody but this machine can read it. An asset's
fingerprint is on chain; its bytes are only here. Deleting this folder does not
change any asset, but until the Mesh exists it is the only copy the launcher
made. Your project's own repository still has every file.

See ADR-047 in the Demiurge repository, question 13.
";

impl TemporaryStore {
    /// The store inside the launcher's data directory.
    pub fn in_data_dir(data_dir: &Path) -> Self {
        Self {
            dir: data_dir.join(STORE_DIR),
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// Where the blob with this reference is, or would be.
    pub fn path_of(&self, reference: &ContentRef) -> PathBuf {
        self.dir.join("blake3").join(reference.root_hex())
    }

    /// Write `bytes` under their root, unless they are already there. Refuses
    /// bytes that are not what `reference` says they are.
    pub fn put(&self, reference: &ContentRef, bytes: &[u8]) -> Result<(), ContentError> {
        if ContentRef::of(bytes) != *reference {
            return Err(ContentError::Store(format!(
                "the bytes for {} do not hash to it",
                reference.root_hex()
            )));
        }

        let target = self.path_of(reference);
        if std::fs::metadata(&target).is_ok_and(|m| m.len() == reference.size) {
            return Ok(());
        }

        let blobs = target.parent().expect("a blob is inside the store");
        std::fs::create_dir_all(blobs).map_err(|e| ContentError::Store(e.to_string()))?;
        let readme = self.dir.join("README.txt");
        if !readme.exists() {
            std::fs::write(&readme, STORE_README)
                .map_err(|e| ContentError::Store(e.to_string()))?;
        }

        // Written beside the target and renamed into place, so a crash leaves a
        // stray temporary file rather than a blob with the wrong bytes.
        let partial = blobs.join(format!(".{}.partial", reference.root_hex()));
        std::fs::write(&partial, bytes).map_err(|e| ContentError::Store(e.to_string()))?;
        std::fs::rename(&partial, &target).map_err(|e| ContentError::Store(e.to_string()))?;
        Ok(())
    }

    /// Copy the file at `source` in under its root, unless it is already there, a piece at a time. The copy is hashed
    /// as it is written and refused if it is not what `reference` says, so a file that changed since it was
    /// fingerprinted is never filed under the old fingerprint.
    pub fn put_file(&self, reference: &ContentRef, source: &Path) -> Result<(), ContentError> {
        let target = self.path_of(reference);
        if std::fs::metadata(&target).is_ok_and(|m| m.len() == reference.size) {
            return Ok(());
        }
        let blobs = target.parent().expect("a blob is inside the store");
        std::fs::create_dir_all(blobs).map_err(|e| ContentError::Store(e.to_string()))?;
        let readme = self.dir.join("README.txt");
        if !readme.exists() {
            std::fs::write(&readme, STORE_README)
                .map_err(|e| ContentError::Store(e.to_string()))?;
        }
        let partial = blobs.join(format!(".{}.partial", reference.root_hex()));
        let copied = (|| -> std::io::Result<(u64, [u8; 32])> {
            let mut from = std::io::BufReader::new(std::fs::File::open(source)?);
            let mut to = std::io::BufWriter::new(std::fs::File::create(&partial)?);
            let mut hasher = blake3::Hasher::new();
            let mut buf = vec![0u8; 64 * 1024];
            let mut size = 0u64;
            loop {
                let n = std::io::Read::read(&mut from, &mut buf)?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
                std::io::Write::write_all(&mut to, &buf[..n])?;
                size += n as u64;
            }
            std::io::Write::flush(&mut to)?;
            Ok((size, *hasher.finalize().as_bytes()))
        })();
        let (size, root) = copied.map_err(|e| {
            let _ = std::fs::remove_file(&partial);
            ContentError::Store(e.to_string())
        })?;
        if size != reference.size || root != reference.root {
            let _ = std::fs::remove_file(&partial);
            return Err(ContentError::Store(format!(
                "{} changed since it was fingerprinted; publish it again",
                source.display()
            )));
        }
        std::fs::rename(&partial, &target).map_err(|e| ContentError::Store(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Library file is fingerprinted and stored a piece at a time, as the same reference its bytes give, and a file
    /// that changed after it was fingerprinted is never filed under the old fingerprint.
    #[test]
    fn a_file_is_fingerprinted_and_stored_a_piece_at_a_time_and_a_changed_one_is_refused() {
        let dir = std::env::temp_dir().join(format!("qor-content-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let bytes: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let source = dir.join("a song.flac");
        std::fs::write(&source, &bytes).unwrap();

        let reference = ContentRef::of_file(&source).unwrap();
        assert_eq!(reference, ContentRef::of(&bytes));
        let store = TemporaryStore::in_data_dir(&dir);
        store.put_file(&reference, &source).unwrap();
        assert_eq!(std::fs::read(store.path_of(&reference)).unwrap(), bytes);
        store.put_file(&reference, &source).unwrap();

        let mut changed = bytes.clone();
        changed[1000] ^= 1;
        std::fs::write(&source, &changed).unwrap();
        let other = ContentRef {
            root: [9; 32],
            ..reference
        };
        assert!(
            store.put_file(&other, &source).is_err(),
            "bytes that are not the fingerprint's"
        );
        assert!(!store.path_of(&other).exists(), "nothing is filed under it");
        let leftovers = std::fs::read_dir(store.path_of(&other).parent().unwrap())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".partial")
            })
            .count();
        assert_eq!(leftovers, 0, "no partial copy is left behind");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn files(list: &[(&str, &[u8])]) -> Vec<(String, ContentRef)> {
        list.iter()
            .map(|(path, bytes)| (path.to_string(), ContentRef::of(bytes)))
            .collect()
    }

    fn source() -> Option<SourceRef> {
        Some(SourceRef {
            commit: CommitId::Sha1([7; 20]),
            branch: Some("main".into()),
        })
    }

    const A: (&str, &[u8]) = ("README.md", b"# A song\n");
    const B: (&str, &[u8]) = ("stems/vox.wav", b"RIFF....WAVEfmt ");
    const C: (&str, &[u8]) = ("sessions/take.als", b"<Ableton/>");

    /// ADR-047 decision 5: two encoders, one identity.
    #[test]
    fn the_same_files_in_a_different_order_make_the_same_reference() {
        let one = Manifest::new(files(&[A, B, C]), 1_758_000_000, source()).unwrap();
        let two = Manifest::new(files(&[C, A, B]), 1_758_000_000, source()).unwrap();
        let three = Manifest::new(files(&[B, C, A]), 1_758_000_000, source()).unwrap();

        assert_eq!(one.bytes(), two.bytes());
        assert_eq!(one.reference(), two.reference());
        assert_eq!(one.reference(), three.reference());
        let paths: Vec<&str> = one.entries.iter().map(|e| e.path.as_str()).collect();
        assert_eq!(paths, ["README.md", "sessions/take.als", "stems/vox.wav"]);
    }

    /// Any change to any file is a different asset.
    #[test]
    fn one_changed_byte_changes_the_reference() {
        let before = Manifest::new(files(&[A, B, C]), 1_758_000_000, source()).unwrap();

        let mut changed = B.1.to_vec();
        changed[0] ^= 1;
        let after = Manifest::new(
            files(&[A, ("stems/vox.wav", &changed), C]),
            1_758_000_000,
            source(),
        )
        .unwrap();
        assert_ne!(before.reference(), after.reference());

        // And so is a file renamed, or a different commit.
        let renamed = Manifest::new(
            files(&[A, ("stems/vox2.wav", B.1), C]),
            1_758_000_000,
            source(),
        )
        .unwrap();
        assert_ne!(before.reference(), renamed.reference());
        let other_commit = Manifest::new(
            files(&[A, B, C]),
            1_758_000_000,
            Some(SourceRef {
                commit: CommitId::Sha1([8; 20]),
                branch: Some("main".into()),
            }),
        )
        .unwrap();
        assert_ne!(before.reference(), other_commit.reference());
    }

    /// The reference is exactly what the chain stores: 41 bytes, tag, root,
    /// size little-endian (ADR-047 decision 3; `pallet_drc369::ContentRef`).
    #[test]
    fn the_reference_is_the_41_bytes_the_chain_stores() {
        let manifest = Manifest::new(files(&[A]), 0, None).unwrap();
        let bytes = manifest.bytes();
        let reference = manifest.reference();

        assert_eq!(reference.algo, HashAlgo::Blake3_256);
        assert_eq!(reference.root, *blake3::hash(&bytes).as_bytes());
        assert_eq!(reference.size, bytes.len() as u64);

        let encoded = reference.encode();
        assert_eq!(encoded.len(), 41);
        assert_eq!(encoded[0], 0, "BLAKE3-256 is tag 0");
        assert_eq!(&encoded[1..33], &reference.root);
        assert_eq!(&encoded[33..], &reference.size.to_le_bytes());

        // A commit id is tagged the same way.
        assert_eq!(CommitId::Sha1([1; 20]).encode()[0], 0);
        assert_eq!(CommitId::Sha256([1; 32]).encode()[0], 1);
    }

    /// The manifest's layout, byte by byte, for a single file with no source.
    /// Pinned so that an edit to the types above cannot change every asset's
    /// identity without a test saying so.
    #[test]
    fn the_manifest_encoding_is_pinned() {
        let content = ContentRef::of(b"x");
        let manifest = Manifest::new(vec![("a.txt".into(), content)], 5, None).unwrap();

        let mut expected = Vec::new();
        expected.extend_from_slice(&1u16.to_le_bytes()); // version
        expected.push(1 << 2); // one entry, compact
        expected.push(5 << 2); // "a.txt", compact length
        expected.extend_from_slice(b"a.txt");
        expected.push(10 << 2); // "text/plain"
        expected.extend_from_slice(b"text/plain");
        expected.push(0); // BLAKE3-256
        expected.extend_from_slice(&content.root);
        expected.extend_from_slice(&1u64.to_le_bytes());
        expected.push(3); // Component
        expected.extend_from_slice(&5u64.to_le_bytes()); // created
        expected.push(0); // no source

        assert_eq!(manifest.bytes(), expected);
    }

    #[test]
    fn a_path_that_could_mean_two_things_is_refused() {
        for bad in [
            "",
            "/abs",
            "a//b",
            "a/./b",
            "../up",
            "a/..",
            "a\\b",
            "trailing/",
        ] {
            assert!(
                Manifest::new(vec![(bad.into(), ContentRef::of(b"x"))], 0, None).is_err(),
                "{bad:?} was accepted"
            );
        }
        assert!(
            Manifest::new(Vec::new(), 0, None).is_err(),
            "nothing is not an asset"
        );
        assert!(
            Manifest::new(files(&[A, A]), 0, None).is_err(),
            "one path twice is refused"
        );
    }

    #[test]
    fn media_types_come_from_the_table_or_say_nothing() {
        assert_eq!(media_type("stems/vox.WAV"), "audio/vnd.wave");
        assert_eq!(media_type("README.md"), "text/markdown");
        assert_eq!(media_type("scenes/main.tscn"), "application/octet-stream");
        assert_eq!(media_type(".gitignore"), "application/octet-stream");
        assert_eq!(media_type("Makefile"), "application/octet-stream");

        // An asset can be any bytes, and the table says what the common ones
        // are: archives, video, sound, images, documents, models and fonts.
        for (path, expected) in [
            ("release/build.zip", "application/zip"),
            ("release/build.tar.gz", "application/gzip"),
            ("cut/final.mov", "video/quicktime"),
            ("cut/final.mp4", "video/mp4"),
            ("cut/teaser.webm", "video/webm"),
            ("master/track.flac", "audio/flac"),
            ("master/track.m4a", "audio/mp4"),
            ("art/cover.tiff", "image/tiff"),
            ("art/cover.avif", "image/avif"),
            ("papers/contract.pdf", "application/pdf"),
            (
                "papers/notes.docx",
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            ),
            ("models/prop.glb", "model/gltf-binary"),
            ("models/prop.obj", "model/obj"),
            ("models/prop.usdz", "model/vnd.usdz+zip"),
            ("type/face.woff2", "font/woff2"),
        ] {
            assert_eq!(media_type(path), expected, "{path}");
        }

        // A project's own formats have no registered type, and none is invented.
        for path in [
            "sessions/take.als",
            "art/cover.psd",
            "models/prop.blend",
            "models/prop.fbx",
        ] {
            assert_eq!(media_type(path), "application/octet-stream", "{path}");
        }
        assert_eq!(role("LICENSE"), Role::Licence);
        assert_eq!(role("LICENCE.md"), Role::Licence);
        assert_eq!(role("docs/LICENSE"), Role::Component);
    }

    #[test]
    fn the_store_keeps_bytes_under_their_root_and_refuses_the_wrong_ones() {
        let dir = std::env::temp_dir().join(format!("qor-store-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = TemporaryStore::in_data_dir(&dir);

        let reference = ContentRef::of(b"the take");
        store.put(&reference, b"the take").unwrap();
        let path = store.path_of(&reference);
        assert_eq!(std::fs::read(&path).unwrap(), b"the take");
        assert!(path.starts_with(dir.join(STORE_DIR)));
        let readme = std::fs::read_to_string(dir.join(STORE_DIR).join("README.txt")).unwrap();
        assert!(readme.starts_with("TEMPORARY."));

        // Idempotent, and it will not store bytes under a root they do not have.
        store.put(&reference, b"the take").unwrap();
        assert!(store.put(&reference, b"another take").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
