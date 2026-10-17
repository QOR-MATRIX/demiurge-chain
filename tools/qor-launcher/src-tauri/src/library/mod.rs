//! The Library's index of a person's own folders (ADR-094 decision 5; docs/blueprints/library.md section 2).
//!
//! A person chooses folders (Videos, Music, Pictures and Documents offered at first); the launcher walks them, records
//! each file's kind by its extension with its name, size and when it changed, reads what it can about it (a picture's
//! size and photo details, music tags, a text document's first words), and keeps it all in SQLite with a full-text index
//! (FTS5) so search answers as the person types. It follows changes as they happen. Nothing in it leaves the PC, and it
//! keeps paths, never file contents beyond a document's first words.
//!
//! The interface asks by path, and only for paths this index holds: a webview never reads an arbitrary file through it.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::error::{QorError, QorResult};

pub mod commands;
pub mod watch;

#[cfg(test)]
mod tests;

/// The kinds a file is shelved as.
pub const KINDS: [&str; 5] = ["video", "audio", "picture", "document", "other"];

/// How much of a text document's beginning is kept for search.
const TEXT_KEPT: usize = 16 * 1024;
/// A thumbnail's longest side, in pixels.
pub const THUMBNAIL_EDGE: u32 = 384;

/// A file's kind, by its extension.
pub fn kind_of(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match ext.as_str() {
        "mkv" | "mp4" | "m4v" | "mov" | "avi" | "webm" | "wmv" | "flv" | "ts" | "m2ts" | "mts" | "mpg" | "mpeg"
        | "3gp" | "ogv" | "vob" | "divx" | "f4v" | "asf" => "video",
        "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a" | "aac" | "wma" | "aiff" | "aif" | "alac" | "ape" | "dsf"
        | "mka" | "ac3" | "dts" => "audio",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "svg" | "ico" | "tga" | "heic" | "heif"
        | "avif" | "jxl" | "raw" | "cr2" | "cr3" | "nef" | "arw" | "dng" | "orf" | "rw2" => "picture",
        "pdf" | "txt" | "md" | "markdown" | "rtf" | "doc" | "docx" | "odt" | "xls" | "xlsx" | "ods" | "ppt"
        // `.ts` is shelved as video above: in a person's own folders it is a recorded stream far more often than code.
        | "pptx" | "odp" | "csv" | "json" | "xml" | "html" | "htm" | "log" | "epub" | "rs" | "tsx" | "js" | "py"
        | "cpp" | "c" | "h" | "hpp" | "cs" | "java" | "go" | "toml" | "yaml" | "yml" | "ini" | "sql" | "qml" | "sh"
        | "ps1" | "css" => "document",
        _ => "other",
    }
}

/// Whether a document's words are read for search: plain text, which is cheap and safe to read.
fn readable_text(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some(
            "txt"
                | "md"
                | "markdown"
                | "csv"
                | "json"
                | "xml"
                | "log"
                | "rs"
                | "tsx"
                | "js"
                | "py"
                | "cpp"
                | "c"
                | "h"
                | "hpp"
                | "cs"
                | "java"
                | "go"
                | "toml"
                | "yaml"
                | "yml"
                | "ini"
                | "sql"
                | "qml"
                | "sh"
                | "ps1"
                | "css"
                | "html"
                | "htm"
        )
    )
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn db_error(e: rusqlite::Error) -> QorError {
    QorError::Internal(format!("the Library's index: {e}"))
}

/// One file the Library knows.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Item {
    pub path: String,
    pub folder: String,
    pub name: String,
    pub kind: String,
    pub size: i64,
    /// Seconds since 1970, when the file last changed.
    pub modified: i64,
    /// When the Library first saw it.
    pub added: i64,
    /// When it was last opened from the Library, and where playback stopped.
    pub opened: Option<i64>,
    pub position: Option<f64>,
    pub duration: Option<f64>,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub taken: Option<String>,
    pub camera: Option<String>,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
}

const ITEM_COLUMNS: &str =
    "path, folder, name, kind, size, modified, added, opened, position, duration, width, height, \
                            taken, camera, title, artist, album";

/// The same columns named through `items`, for a query that joins the search index, which has a `folder` of its own.
const ITEM_COLUMNS_JOINED: &str = "items.path, items.folder, items.name, items.kind, items.size, items.modified, \
                                   items.added, items.opened, items.position, items.duration, items.width, \
                                   items.height, items.taken, items.camera, items.title, items.artist, items.album";

fn item(row: &rusqlite::Row<'_>) -> rusqlite::Result<Item> {
    Ok(Item {
        path: row.get(0)?,
        folder: row.get(1)?,
        name: row.get(2)?,
        kind: row.get(3)?,
        size: row.get(4)?,
        modified: row.get(5)?,
        added: row.get(6)?,
        opened: row.get(7)?,
        position: row.get(8)?,
        duration: row.get(9)?,
        width: row.get(10)?,
        height: row.get(11)?,
        taken: row.get(12)?,
        camera: row.get(13)?,
        title: row.get(14)?,
        artist: row.get(15)?,
        album: row.get(16)?,
    })
}

/// What a scan did.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Scanned {
    pub seen: u64,
    pub added: u64,
    pub changed: u64,
    pub removed: u64,
}

/// What is read from a file when it is first seen or has changed.
#[derive(Debug, Default)]
struct Details {
    width: Option<i64>,
    height: Option<i64>,
    taken: Option<String>,
    camera: Option<String>,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    duration: Option<f64>,
    text: Option<String>,
}

fn details(path: &Path, kind: &str) -> Details {
    let mut d = Details::default();
    match kind {
        "picture" => {
            if let Ok((w, h)) = image::image_dimensions(path) {
                d.width = Some(i64::from(w));
                d.height = Some(i64::from(h));
            }
            if let Ok(file) = std::fs::File::open(path) {
                let mut reader = std::io::BufReader::new(file);
                if let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) {
                    let field = |tag| {
                        exif.get_field(tag, exif::In::PRIMARY)
                            .map(|f| {
                                f.display_value()
                                    .to_string()
                                    .trim_matches('"')
                                    .trim()
                                    .to_string()
                            })
                            .filter(|s| !s.is_empty())
                    };
                    d.taken = field(exif::Tag::DateTimeOriginal);
                    d.camera = match (field(exif::Tag::Make), field(exif::Tag::Model)) {
                        (Some(make), Some(model)) if model.starts_with(&make) => Some(model),
                        (Some(make), Some(model)) => Some(format!("{make} {model}")),
                        (one, other) => one.or(other),
                    };
                }
            }
        }
        "audio" => {
            use lofty::file::{AudioFile as _, TaggedFileExt as _};
            use lofty::tag::Accessor as _;
            if let Ok(tagged) = lofty::read_from_path(path) {
                d.duration =
                    Some(tagged.properties().duration().as_secs_f64()).filter(|s| *s > 0.0);
                if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
                    d.title = tag.title().map(|s| s.to_string());
                    d.artist = tag.artist().map(|s| s.to_string());
                    d.album = tag.album().map(|s| s.to_string());
                }
            }
        }
        "document" if readable_text(path) => {
            if let Ok(file) = std::fs::File::open(path) {
                use std::io::Read as _;
                let mut bytes = Vec::with_capacity(TEXT_KEPT);
                if file.take(TEXT_KEPT as u64).read_to_end(&mut bytes).is_ok() {
                    d.text = Some(String::from_utf8_lossy(&bytes).into_owned());
                }
            }
        }
        _ => {}
    }
    d
}

/// Whether a path is worth indexing: not hidden, not a system or temporary file.
fn wanted(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    !(name.starts_with('.')
        || name.starts_with('~')
        || name.eq_ignore_ascii_case("desktop.ini")
        || name.eq_ignore_ascii_case("thumbs.db")
        || name.ends_with(".tmp")
        || name.ends_with(".part")
        || name.ends_with(".crdownload"))
}

/// The person's index.
pub struct Library {
    db: Mutex<Connection>,
    thumbnails: PathBuf,
}

impl Library {
    /// Open (or make) the index in `dir`, with its thumbnails beside it.
    pub fn open(dir: &Path) -> QorResult<Arc<Self>> {
        std::fs::create_dir_all(dir).map_err(|e| QorError::Io(e.to_string()))?;
        let db = Connection::open(dir.join("index.sqlite")).map_err(db_error)?;
        db.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS folders (path TEXT PRIMARY KEY, added INTEGER NOT NULL);
             CREATE TABLE IF NOT EXISTS items (
                 id INTEGER PRIMARY KEY,
                 path TEXT NOT NULL UNIQUE,
                 folder TEXT NOT NULL,
                 dir TEXT NOT NULL,
                 name TEXT NOT NULL,
                 kind TEXT NOT NULL,
                 size INTEGER NOT NULL,
                 modified INTEGER NOT NULL,
                 added INTEGER NOT NULL,
                 opened INTEGER,
                 position REAL,
                 duration REAL,
                 width INTEGER,
                 height INTEGER,
                 taken TEXT,
                 camera TEXT,
                 title TEXT,
                 artist TEXT,
                 album TEXT,
                 text TEXT
             );
             CREATE INDEX IF NOT EXISTS items_by_folder ON items (folder);
             CREATE INDEX IF NOT EXISTS items_by_kind ON items (kind, modified);
             CREATE VIRTUAL TABLE IF NOT EXISTS search USING fts5(
                 name, dir, title, artist, album, camera, text,
                 content = 'items', content_rowid = 'id', tokenize = 'unicode61 remove_diacritics 2'
             );
             CREATE TRIGGER IF NOT EXISTS items_ai AFTER INSERT ON items BEGIN
                 INSERT INTO search (rowid, name, dir, title, artist, album, camera, text)
                 VALUES (new.id, new.name, new.dir, new.title, new.artist, new.album, new.camera, new.text);
             END;
             CREATE TRIGGER IF NOT EXISTS items_ad AFTER DELETE ON items BEGIN
                 INSERT INTO search (search, rowid, name, dir, title, artist, album, camera, text)
                 VALUES ('delete', old.id, old.name, old.dir, old.title, old.artist, old.album, old.camera, old.text);
             END;
             CREATE TRIGGER IF NOT EXISTS items_au AFTER UPDATE ON items BEGIN
                 INSERT INTO search (search, rowid, name, dir, title, artist, album, camera, text)
                 VALUES ('delete', old.id, old.name, old.dir, old.title, old.artist, old.album, old.camera, old.text);
                 INSERT INTO search (rowid, name, dir, title, artist, album, camera, text)
                 VALUES (new.id, new.name, new.dir, new.title, new.artist, new.album, new.camera, new.text);
             END;",
        )
        .map_err(db_error)?;
        let thumbnails = dir.join("thumbnails");
        Ok(Arc::new(Self {
            db: Mutex::new(db),
            thumbnails,
        }))
    }

    /// The folders a person might start with: their own Videos, Music, Pictures and Documents, those that exist.
    pub fn suggested() -> Vec<String> {
        let Some(user) = directories::UserDirs::new() else {
            return vec![];
        };
        [
            user.video_dir(),
            user.audio_dir(),
            user.picture_dir(),
            user.document_dir(),
        ]
        .into_iter()
        .flatten()
        .filter(|p| p.is_dir())
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
    }

    pub fn folders(&self) -> QorResult<Vec<String>> {
        let db = self.db.lock();
        let mut q = db
            .prepare("SELECT path FROM folders ORDER BY added")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| r.get(0))
            .map_err(db_error)?
            .collect::<Result<Vec<String>, _>>()
            .map_err(db_error)?;
        Ok(rows)
    }

    /// Add a folder to the Library. It must exist, and not sit inside one already added (nor hold one).
    pub fn add_folder(&self, path: &Path) -> QorResult<String> {
        let path = std::fs::canonicalize(path)
            .map_err(|_| QorError::Io("that folder could not be found".into()))?;
        if !path.is_dir() {
            return Err(QorError::Io("that is not a folder".into()));
        }
        let shown = display_path(&path);
        // Compared as the index keeps them, without the `\\?\` prefix canonicalising adds on Windows.
        let candidate = Path::new(&shown);
        for existing in self.folders()? {
            let existing = PathBuf::from(existing);
            if candidate.starts_with(&existing) || existing.starts_with(candidate) {
                return Err(QorError::Io(format!(
                    "{} overlaps {}, which the Library has already",
                    shown,
                    existing.display()
                )));
            }
        }
        self.db
            .lock()
            .execute(
                "INSERT OR IGNORE INTO folders (path, added) VALUES (?1, ?2)",
                params![shown, now()],
            )
            .map_err(db_error)?;
        Ok(shown)
    }

    /// Take a folder out of the Library, and everything in it out of the index. The files are not touched.
    pub fn remove_folder(&self, path: &str) -> QorResult<()> {
        let db = self.db.lock();
        db.execute("DELETE FROM folders WHERE path = ?1", params![path])
            .map_err(db_error)?;
        db.execute("DELETE FROM items WHERE folder = ?1", params![path])
            .map_err(db_error)?;
        Ok(())
    }

    /// Walk one folder: add what is new, re-read what changed, drop what has gone.
    pub fn scan(&self, folder: &str) -> QorResult<Scanned> {
        let mut seen_paths = std::collections::HashSet::new();
        let mut result = Scanned::default();
        for entry in walkdir::WalkDir::new(folder)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| e.depth() == 0 || wanted(e.path()))
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            seen_paths.insert(display_path(path));
            match self.upsert(folder, path)? {
                Some(true) => result.added += 1,
                Some(false) => result.changed += 1,
                None => {}
            }
            result.seen += 1;
        }
        let known: Vec<String> = {
            let db = self.db.lock();
            let mut q = db
                .prepare("SELECT path FROM items WHERE folder = ?1")
                .map_err(db_error)?;
            let rows = q
                .query_map(params![folder], |r| r.get(0))
                .map_err(db_error)?
                .collect::<Result<Vec<String>, _>>()
                .map_err(db_error)?;
            rows
        };
        for gone in known.into_iter().filter(|p| !seen_paths.contains(p)) {
            self.forget(&gone)?;
            result.removed += 1;
        }
        Ok(result)
    }

    /// Scan every folder.
    pub fn scan_all(&self) -> QorResult<Scanned> {
        let mut total = Scanned::default();
        for folder in self.folders()? {
            let s = self.scan(&folder)?;
            total.seen += s.seen;
            total.added += s.added;
            total.changed += s.changed;
            total.removed += s.removed;
        }
        Ok(total)
    }

    /// Record one file. Some(true) when new, Some(false) when it had changed, None when it had not.
    pub fn upsert(&self, folder: &str, path: &Path) -> QorResult<Option<bool>> {
        if !wanted(path) {
            return Ok(None);
        }
        let Ok(meta) = std::fs::metadata(path) else {
            return Ok(None);
        };
        let shown = display_path(path);
        let size = meta.len() as i64;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let held: Option<(i64, i64)> = self
            .db
            .lock()
            .query_row(
                "SELECT size, modified FROM items WHERE path = ?1",
                params![shown],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()
            .map_err(db_error)?;
        if held == Some((size, modified)) {
            return Ok(None);
        }
        let kind = kind_of(path);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let d = details(path, kind);
        // Its own folder, so a subfolder's name finds what is in it.
        let dir = path.parent().map(display_path).unwrap_or_default();
        let db = self.db.lock();
        db.execute(
            "INSERT INTO items (path, folder, dir, name, kind, size, modified, added, duration, width, height, taken, camera, \
                                title, artist, album, text) \
             VALUES (?1, ?2, ?17, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16) \
             ON CONFLICT (path) DO UPDATE SET folder = ?2, dir = ?17, name = ?3, kind = ?4, size = ?5, modified = ?6, \
               duration = COALESCE(?8, duration), width = COALESCE(?9, width), height = COALESCE(?10, height), \
               taken = ?11, camera = ?12, title = ?13, artist = ?14, album = ?15, text = ?16",
            params![
                shown, folder, name, kind, size, modified, now(), d.duration, d.width, d.height, d.taken, d.camera,
                d.title, d.artist, d.album, d.text, dir
            ],
        )
        .map_err(db_error)?;
        Ok(Some(held.is_none()))
    }

    /// Drop a path from the index, and its thumbnail.
    pub fn forget(&self, path: &str) -> QorResult<()> {
        self.db
            .lock()
            .execute("DELETE FROM items WHERE path = ?1", params![path])
            .map_err(db_error)?;
        let _ = std::fs::remove_file(self.thumbnail_path(path));
        Ok(())
    }

    /// The folder in the Library that holds `path`, if any.
    pub fn folder_of(&self, path: &Path) -> QorResult<Option<String>> {
        Ok(self
            .folders()?
            .into_iter()
            .find(|f| path.starts_with(Path::new(f))))
    }

    /// Search names, folders, tags, photo details and document text, best first; every word as a prefix, so results
    /// come as the person types.
    pub fn search(&self, query: &str, kind: Option<&str>, limit: u32) -> QorResult<Vec<Item>> {
        let terms: Vec<String> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .take(12)
            .map(|t| format!("\"{}\"*", t.replace('"', "")))
            .collect();
        if terms.is_empty() {
            return Ok(vec![]);
        }
        let db = self.db.lock();
        let mut q = db
            .prepare(&format!(
                "SELECT {ITEM_COLUMNS_JOINED} FROM items JOIN search ON search.rowid = items.id \
                 WHERE search MATCH ?1 AND (?2 IS NULL OR items.kind = ?2) \
                 ORDER BY bm25(search, 10.0, 2.0, 6.0, 5.0, 4.0, 2.0, 1.0) LIMIT ?3"
            ))
            .map_err(db_error)?;
        let rows = q
            .query_map(params![terms.join(" AND "), kind, limit.min(500)], item)
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(rows)
    }

    /// A shelf: one kind (or all), newest added, or last opened.
    pub fn shelf(&self, kind: Option<&str>, order: &str, limit: u32) -> QorResult<Vec<Item>> {
        let (filter, order_by) = match order {
            "opened" => ("opened IS NOT NULL", "opened DESC"),
            "name" => ("1", "name COLLATE NOCASE"),
            _ => ("1", "added DESC, modified DESC"),
        };
        let db = self.db.lock();
        let mut q = db
            .prepare(&format!(
                "SELECT {ITEM_COLUMNS} FROM items WHERE {filter} AND (?1 IS NULL OR kind = ?1) ORDER BY {order_by} \
                 LIMIT ?2"
            ))
            .map_err(db_error)?;
        let rows = q
            .query_map(params![kind, limit.min(500)], item)
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(rows)
    }

    /// How many of each kind the Library holds.
    pub fn counts(&self) -> QorResult<Vec<(String, i64)>> {
        let db = self.db.lock();
        let mut q = db
            .prepare("SELECT kind, COUNT(*) FROM items GROUP BY kind")
            .map_err(db_error)?;
        let rows = q
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .map_err(db_error)?
            .collect::<Result<Vec<_>, _>>()
            .map_err(db_error)?;
        Ok(rows)
    }

    /// One indexed file; not found for any path the Library does not hold.
    pub fn item(&self, path: &str) -> QorResult<Item> {
        self.db
            .lock()
            .query_row(
                &format!("SELECT {ITEM_COLUMNS} FROM items WHERE path = ?1"),
                params![path],
                item,
            )
            .optional()
            .map_err(db_error)?
            .ok_or_else(|| QorError::Io("that file is not in the Library".into()))
    }

    /// Note that a file was opened, and where playback stopped.
    pub fn opened(&self, path: &str, position: Option<f64>) -> QorResult<()> {
        self.item(path)?;
        self.db
            .lock()
            .execute(
                "UPDATE items SET opened = ?2, position = COALESCE(?3, position) WHERE path = ?1",
                params![path, now(), position],
            )
            .map_err(db_error)?;
        Ok(())
    }

    /// What QOR Media learned of a file it played.
    pub fn learned(
        &self,
        path: &str,
        duration: Option<f64>,
        width: Option<i64>,
        height: Option<i64>,
    ) -> QorResult<()> {
        self.item(path)?;
        self.db
            .lock()
            .execute(
                "UPDATE items SET duration = COALESCE(?2, duration), width = COALESCE(?3, width), \
                   height = COALESCE(?4, height) WHERE path = ?1",
                params![path, duration, width, height],
            )
            .map_err(db_error)?;
        Ok(())
    }

    fn thumbnail_path(&self, path: &str) -> PathBuf {
        use sha2::Digest as _;
        let key = hex::encode(sha2::Sha256::digest(path.as_bytes()));
        self.thumbnails.join(format!("{key}.jpg"))
    }

    /// A picture's thumbnail as JPEG bytes, made once and kept until the picture changes. Only for indexed pictures.
    pub fn thumbnail(&self, path: &str) -> QorResult<Vec<u8>> {
        let found = self.item(path)?;
        if found.kind != "picture" {
            return Err(QorError::Io(
                "only pictures have thumbnails made here".into(),
            ));
        }
        let kept = self.thumbnail_path(path);
        let fresh = std::fs::metadata(&kept)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .is_some_and(|t| t.as_secs() as i64 >= found.modified);
        if fresh {
            return std::fs::read(&kept).map_err(|e| QorError::Io(e.to_string()));
        }
        let picture = image::ImageReader::open(path)
            .map_err(|e| QorError::Io(e.to_string()))?
            .with_guessed_format()
            .map_err(|e| QorError::Io(e.to_string()))?
            .decode()
            .map_err(|_| QorError::Io("that picture could not be read here".into()))?;
        let small = picture.thumbnail(THUMBNAIL_EDGE, THUMBNAIL_EDGE).to_rgb8();
        let mut bytes = Vec::new();
        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 82)
            .encode_image(&small)
            .map_err(|e| QorError::Internal(e.to_string()))?;
        std::fs::create_dir_all(&self.thumbnails).map_err(|e| QorError::Io(e.to_string()))?;
        let _ = std::fs::write(&kept, &bytes);
        Ok(bytes)
    }
}

/// A path as the index keeps it: without Windows' `\\?\` prefix, so it reads and compares as a person writes it.
pub fn display_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
}
