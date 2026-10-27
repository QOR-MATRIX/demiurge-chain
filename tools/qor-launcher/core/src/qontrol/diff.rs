//! What changed inside a file, line by line.
//!
//! # Why this is not a byte comparison
//!
//! What git stores and what is on disk are not the same bytes. With
//! `core.autocrlf = true` on Windows, git stores `LF` and writes `CRLF`; a
//! `.gitattributes` line such as `*.gd text eol=lf` or `*.wav -diff` changes
//! what counts as a change and what can be shown at all. A diff that compared
//! the raw file against the stored blob would report every line of every file
//! as changed on a machine that converts line endings — changes that are not
//! there, which is exactly what a creator must never be shown.
//!
//! So the working-tree side goes through gitoxide's filter pipeline in
//! `Mode::ToGit`: it is converted to what `git add` would store, under this
//! repository's own configuration and attributes, and only then compared with
//! the blob the index holds. That is what `git diff` does.
//!
//! # What this never does
//!
//! It never runs a program. A `diff` driver can name a text conversion command
//! or an external diff tool; both run whatever the configuration says, and
//! Qontrol opens folders from strangers. `Mode::ToGit` does not apply text
//! conversion, and an external command is ignored in favour of the built-in
//! line diff. A file git would only show through such a program is shown here
//! as its stored text, or as binary.

use gix::bstr::BStr;
use gix::diff::blob::{
    pipeline::{Mode, WorktreeRoots},
    platform::{prepare_diff::Operation, resource::Data},
    unified_diff::{ConsumeHunk, ContextSize, DiffLineKind, HunkHeader},
    Algorithm, ResourceKind, UnifiedDiff,
};
use gix::object::tree::EntryKind;
use serde::Serialize;

use super::{read_entries, QontrolError};

/// The most lines one file's diff returns. A generated file or a reformat can
/// change a hundred thousand lines, and the surface is for reading; past this
/// the diff says it was cut short rather than silently ending.
pub const MAX_LINES: usize = 4000;

/// One changed path, and what changed inside it.
#[derive(Debug, Clone, Serialize)]
pub struct FileDiff {
    pub path: String,
    /// The same plain word the change list shows.
    pub state: String,
    /// Where a moved file came from. `None` for everything else.
    pub from: Option<String>,
    pub body: DiffBody,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffBody {
    /// A text diff. `truncated` is true when it stopped at [`MAX_LINES`].
    Text {
        lines: Vec<DiffLine>,
        truncated: bool,
    },
    /// Git calls this binary — by its content, or because `.gitattributes`
    /// says `-diff` or `binary` — so there are no lines to show. The sizes are
    /// what there is; `None` is a side that does not exist.
    Binary {
        old_size: Option<u64>,
        new_size: Option<u64>,
    },
    /// Status listed the path, and once both sides are read as git reads them
    /// there is no difference in content. A file whose only change is line
    /// endings the repository normalises is the usual case.
    Same,
    /// A file with nothing in it, added or removed: there are no lines to
    /// show, and saying "no difference" would be wrong.
    Empty,
    /// Not a file: a folder of new files, or a submodule. `reason` says which.
    NotAFile { reason: String },
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DiffLine {
    /// `hunk`, `context`, `add` or `remove`.
    pub kind: &'static str,
    pub text: String,
    /// The line's number before the change, where it has one.
    pub old: Option<u32>,
    /// The line's number after the change, where it has one.
    pub new: Option<u32>,
}

/// Collects hunks as lines the view can draw, with line numbers attached,
/// rather than as one string it would have to parse back apart.
struct Lines {
    out: Vec<DiffLine>,
    truncated: bool,
}

impl ConsumeHunk for Lines {
    type Out = Self;

    fn consume_hunk(
        &mut self,
        header: HunkHeader,
        lines: &[(DiffLineKind, &[u8])],
    ) -> std::io::Result<()> {
        if self.truncated {
            return Ok(());
        }
        self.out.push(DiffLine {
            kind: "hunk",
            text: header.to_string(),
            old: None,
            new: None,
        });
        let mut old = header.before_hunk_start;
        let mut new = header.after_hunk_start;
        for (kind, bytes) in lines {
            if self.out.len() >= MAX_LINES {
                self.truncated = true;
                break;
            }
            let text = String::from_utf8_lossy(bytes)
                .trim_end_matches(['\n', '\r'])
                .to_owned();
            let (kind, o, n) = match kind {
                DiffLineKind::Context => {
                    let at = (Some(old), Some(new));
                    old += 1;
                    new += 1;
                    ("context", at.0, at.1)
                }
                DiffLineKind::Remove => {
                    old += 1;
                    ("remove", Some(old - 1), None)
                }
                DiffLineKind::Add => {
                    new += 1;
                    ("add", None, Some(new - 1))
                }
            };
            self.out.push(DiffLine {
                kind,
                text,
                old: o,
                new: n,
            });
        }
        Ok(())
    }

    fn finish(self) -> Self::Out {
        self
    }
}

fn size(data: &Data<'_>) -> Option<u64> {
    match data {
        Data::Missing => None,
        Data::Buffer { buf, .. } => Some(buf.len() as u64),
        Data::Binary { size } => Some(*size),
    }
}

pub(super) fn diff(
    repo: &gix::Repository,
    only: Option<&str>,
) -> Result<Vec<FileDiff>, QontrolError> {
    let root = repo
        .workdir()
        .ok_or_else(|| QontrolError::NotARepository("a repository with no working tree".into()))?
        .to_path_buf();

    // Old side: the index's blob, from the object database. New side: the file
    // on disk, converted to what git would store. Attributes are read from the
    // working tree first, because that is where an edited `.gitattributes` is.
    let mut cache = repo
        .diff_resource_cache(
            Mode::ToGit,
            WorktreeRoots {
                old_root: None,
                new_root: Some(root.clone()),
            },
        )
        .map_err(|e| QontrolError::Git(format!("could not prepare a diff: {e}")))?;

    let null = gix::ObjectId::null(repo.object_hash());
    let mut diffs = Vec::new();

    for entry in read_entries(repo)? {
        if only.is_some_and(|only| only != entry.change.path) {
            continue;
        }
        let path = entry.change.path.clone();
        let from = (entry.change.state == "moved")
            .then(|| entry.index.as_ref().map(|i| i.path.clone()))
            .flatten();

        let body = body(repo, &mut cache, &root, &entry, null)?;
        diffs.push(FileDiff {
            path,
            state: entry.change.state,
            from,
            body,
        });
        cache.clear_resource_cache_keep_allocation();
    }
    Ok(diffs)
}

fn body(
    repo: &gix::Repository,
    cache: &mut gix::diff::blob::Platform,
    root: &std::path::Path,
    entry: &super::Entry,
    null: gix::ObjectId,
) -> Result<DiffBody, QontrolError> {
    let path = entry.change.path.as_str();
    let on_disk = root.join(path.trim_end_matches('/'));

    if entry.change.state == "conflicted" {
        return Ok(DiffBody::NotAFile {
            reason: "This file has a merge conflict. Resolve it before its changes can be shown."
                .into(),
        });
    }
    if matches!(
        entry.index.as_ref().map(|i| i.kind),
        Some(EntryKind::Commit | EntryKind::Tree)
    ) {
        return Ok(DiffBody::NotAFile {
            reason: "This is another repository inside this one (a submodule).".into(),
        });
    }
    if on_disk.is_dir() {
        return Ok(DiffBody::NotAFile {
            reason: "A new folder. Its files are new too; commit to add them.".into(),
        });
    }

    let new_kind = match entry.index.as_ref().map(|i| i.kind) {
        Some(kind) => kind,
        None if on_disk.is_symlink() => EntryKind::Link,
        None => EntryKind::Blob,
    };
    let (old_id, old_kind, old_path) = match &entry.index {
        Some(index) => (index.id, index.kind, index.path.as_str()),
        None => (null, new_kind, path),
    };

    let git = |e: &dyn std::fmt::Display| QontrolError::Git(format!("could not read {path}: {e}"));

    cache
        .set_resource(
            old_id,
            old_kind,
            BStr::new(old_path),
            ResourceKind::OldOrSource,
            &repo.objects,
        )
        .map_err(|e| git(&e))?;
    cache
        .set_resource(
            null,
            new_kind,
            BStr::new(path),
            ResourceKind::NewOrDestination,
            &repo.objects,
        )
        .map_err(|e| git(&e))?;

    let prepared = cache.prepare_diff().map_err(|e| git(&e))?;
    let algorithm = match prepared.operation {
        Operation::InternalDiff { algorithm } => algorithm,
        // Never run: see the module comment. The built-in diff stands in.
        Operation::ExternalCommand { .. } => Algorithm::Histogram,
        Operation::SourceOrDestinationIsBinary => {
            return Ok(DiffBody::Binary {
                old_size: size(&prepared.old.data),
                new_size: size(&prepared.new.data),
            });
        }
    };

    let input = prepared.interned_input();
    let computed = gix::diff::blob::diff_with_slider_heuristics(algorithm, &input);
    if computed.count_additions() == 0 && computed.count_removals() == 0 {
        let one_side_missing = matches!(prepared.old.data, Data::Missing)
            || matches!(prepared.new.data, Data::Missing);
        return Ok(if one_side_missing {
            DiffBody::Empty
        } else {
            DiffBody::Same
        });
    }

    let lines = UnifiedDiff::new(
        &computed,
        &input,
        Lines {
            out: Vec::new(),
            truncated: false,
        },
        ContextSize::symmetrical(3),
    )
    .consume()
    .map_err(|e| git(&e))?;

    Ok(DiffBody::Text {
        lines: lines.out,
        truncated: lines.truncated,
    })
}
