//! Qontrol: version control for creators, git-compatible on disk.
//!
//! # The split, and why it is where it is
//!
//! Every **read** — status, history, branches, diffs — is gitoxide, in this
//! process. It is pure Rust, it is fast on the thousands-of-files repositories
//! that game and music projects actually are, and it links into the host that
//! holds the vault without bringing a C parser with it.
//!
//! **Staging** is not gitoxide. Its own `crate-status.md`, checked on
//! 21 September 2026 against `gix` 0.87.1, marks these unimplemented:
//!
//! ```text
//! [ ] add files with `.gitignore` handling
//! [ ] tree from index
//! [ ] add and remove entries          (gix-index)
//! ```
//!
//! while marking `status`, `rev-walk`, diff and `create new commit from tree`
//! done. So the gap is staging, not push or rebase — and staging is the middle
//! of the only chain that matters here:
//!
//! ```text
//! stage  ->  index  ->  write-tree  ->  commit
//! ```
//!
//! gitoxide has the last link and not the two before it. A commit built by
//! bypassing the index would leave `.git/index` stale, and the next plain
//! `git status` in that folder would report files as modified that Qontrol had
//! just committed correctly. The objects would be right and the working state
//! would lie — a silent, delayed failure, which is the column ADR-001 says to
//! stay boring in.
//!
//! So libgit2 stages and writes the tree, gitoxide commits that tree and moves
//! the ref, and the index ends up matching the new HEAD. libgit2 runs in the
//! `qontrol-git` sidecar, never in this process: Qontrol opens folders the user
//! picks, and will open repositories from strangers, and the process holding
//! the keys is not the one that should parse them.
//!
//! # What this module is not
//!
//! It is not a second abstraction for the interface to consume. One port, one
//! set of types, and the gitoxide/libgit2 seam is invisible above it.

pub mod commands;
mod diff;
pub mod guard;
pub mod publish;
pub mod sidecar;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub use diff::{DiffBody, DiffLine, FileDiff};

#[derive(Debug, thiserror::Error)]
pub enum QontrolError {
    #[error("{0}")]
    NotARepository(String),
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Git(String),
    #[error("{0}")]
    Sidecar(String),
    #[error("a commit needs a message")]
    EmptyMessage,
    #[error("there is nothing to commit")]
    NothingToCommit,
    #[error("these need a yes before they are committed: {0}")]
    Unconfirmed(String),
    #[error("{0}")]
    Refused(String),
}

impl From<std::io::Error> for QontrolError {
    fn from(error: std::io::Error) -> Self {
        QontrolError::Io(error.to_string())
    }
}

/// What kind of project a scaffold lays out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scaffold {
    Code,
    Music,
    Game,
}

impl Scaffold {
    pub fn label(self) -> &'static str {
        match self {
            Scaffold::Code => "Code",
            Scaffold::Music => "Music",
            Scaffold::Game => "Game",
        }
    }
}

/// One changed path in the working tree.
#[derive(Debug, Clone, Serialize)]
pub struct Change {
    pub path: String,
    /// `modified`, `added`, `removed`, or `untracked`. Plain words, because a
    /// creator reads this and `??` means nothing to them.
    pub state: String,
}

/// One commit, as the history list shows it.
#[derive(Debug, Clone, Serialize)]
pub struct Commit {
    pub id: String,
    pub short: String,
    pub summary: String,
    pub author: String,
    /// Seconds since the epoch. Formatted in the view, not here.
    pub time: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Branch {
    pub name: String,
    pub head: bool,
}

/// Everything the Projects surface draws, read from the repository on disk.
///
/// The surface holds none of this. It asks again after anything that could have
/// changed it, so what is on screen is what is in the repository rather than
/// what the interface last believed.
#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub branch: Option<String>,
    pub changes: Vec<Change>,
    pub history: Vec<Commit>,
    pub branches: Vec<Branch>,
    /// False when the staging helper is missing, so the surface can say so
    /// instead of letting someone press Commit and get an error.
    pub can_commit: bool,
}

/// Open the repository at `path`; a folder that is not one is refused. QQ checks a project with this before it
/// writes a scene into it.
pub(crate) fn open(path: &Path) -> Result<gix::Repository, QontrolError> {
    gix::open(path)
        .map_err(|e| QontrolError::NotARepository(format!("{path:?} is not a repository: {e}")))
}

/// Open a repository, or create one if the folder has none.
pub fn open_or_init(path: &Path) -> Result<(), QontrolError> {
    if gix::open(path).is_ok() {
        return Ok(());
    }
    gix::init(path)
        .map_err(|e| QontrolError::Git(format!("could not create a repository: {e}")))?;
    Ok(())
}

/// Lay out a project of the given kind, then make it a repository.
///
/// The ignore rules are the point. A music project that commits its renders, or
/// a game project that commits its import cache, is a repository nobody can
/// clone a week later — and by then the history is the problem, not the files.
pub fn scaffold(path: &Path, kind: Scaffold) -> Result<(), QontrolError> {
    std::fs::create_dir_all(path)?;

    let (dirs, ignore, readme) = match kind {
        Scaffold::Code => (
            vec!["src", "tests", "docs"],
            "target/\nnode_modules/\ndist/\n.env\n*.log\n",
            "# A code project\n\n\
             `src/` is the source, `tests/` the tests, `docs/` what a reader needs.\n\n\
             Build output is ignored: `target/`, `node_modules/`, `dist/`. Those are\n\
             made from the source, so committing them makes the history bigger and\n\
             tells nobody anything.\n",
        ),
        Scaffold::Music => (
            vec!["sessions", "stems", "samples", "renders", "reference"],
            // Renders are made from sessions and stems, so they are rebuilt, not
            // versioned. Stems are kept because they are recordings: if a stem is
            // lost, no amount of the project file brings the take back.
            "renders/\n*.wav.peak\n*.asd\n*.reapeaks\n.DS_Store\n",
            "# A music project\n\n\
             `sessions/` holds project files, `stems/` the recorded takes,\n\
             `samples/` what you brought in, `reference/` what you are aiming at.\n\n\
             `renders/` is ignored on purpose. A render is made from the session and\n\
             the stems, so it can be made again. A stem cannot: lose it and the take\n\
             is gone. That is the line this layout draws.\n",
        ),
        Scaffold::Game => (
            vec!["scenes", "assets", "scripts", "audio", "builds"],
            "builds/\n.godot/\n.import/\n*.tmp\n.DS_Store\n",
            "# A game project\n\n\
             `scenes/` holds scenes, `assets/` art and models, `scripts/` code,\n\
             `audio/` sound.\n\n\
             `builds/` and the engine's import cache are ignored. Both are made from\n\
             what is here, and both are large enough to make a repository painful to\n\
             clone if they are not.\n",
        ),
    };

    for dir in dirs {
        std::fs::create_dir_all(path.join(dir))?;
        // Git stores files, not folders. Without this a fresh scaffold would
        // commit as nothing at all and the creator would wonder where it went.
        let keep = path.join(dir).join(".gitkeep");
        if !keep.exists() {
            std::fs::write(keep, "")?;
        }
    }

    let ignore_path = path.join(".gitignore");
    if !ignore_path.exists() {
        std::fs::write(ignore_path, ignore)?;
    }
    let readme_path = path.join("README.md");
    if !readme_path.exists() {
        std::fs::write(readme_path, readme)?;
    }

    open_or_init(path)
}

fn current_branch(repo: &gix::Repository) -> Option<String> {
    repo.head_name()
        .ok()
        .flatten()
        .map(|name| name.shorten().to_string())
}

/// Where a change came from in the index, when it came from the index at all.
///
/// A diff compares the index against the working tree, exactly as `git diff`
/// does, so it needs the blob the index holds for the path — which, for a
/// moved file, is at a different path from the one on disk.
#[derive(Debug, Clone)]
pub(crate) struct IndexSide {
    pub id: gix::ObjectId,
    pub kind: gix::object::tree::EntryKind,
    pub path: String,
}

/// A change, and what the index holds for it. `index` is `None` for a file git
/// has never seen, which is the only honest "before" an untracked file has.
pub(crate) struct Entry {
    pub change: Change,
    pub index: Option<IndexSide>,
}

fn index_side(entry: &gix::index::Entry, path: &str) -> Option<IndexSide> {
    let kind = entry.mode.to_tree_entry_mode()?.kind();
    Some(IndexSide {
        id: entry.id,
        kind,
        path: path.to_owned(),
    })
}

pub(crate) fn read_entries(repo: &gix::Repository) -> Result<Vec<Entry>, QontrolError> {
    let platform = repo
        .status(gix::progress::Discard)
        .map_err(|e| QontrolError::Git(format!("could not read the working tree: {e}")))?;

    let iter = platform
        .into_index_worktree_iter(Vec::new())
        .map_err(|e| QontrolError::Git(format!("could not walk the working tree: {e}")))?;

    let mut entries = Vec::new();
    for item in iter {
        let item = match item {
            Ok(item) => item,
            Err(_) => continue,
        };
        match item {
            gix::status::index_worktree::Item::Modification {
                entry,
                rela_path,
                status,
                ..
            } => {
                // Aliased: this module has its own `Change`, which is what the
                // interface receives, and they are different things.
                use gix::status::plumbing::index_as_worktree::{Change as GitChange, EntryStatus};
                let state = match status {
                    EntryStatus::Conflict { .. } => "conflicted",
                    EntryStatus::Change(GitChange::Removed) => "removed",
                    EntryStatus::IntentToAdd => "added",
                    _ => "modified",
                };
                let path = rela_path.to_string();
                entries.push(Entry {
                    index: index_side(&entry, &path),
                    change: Change {
                        path,
                        state: state.into(),
                    },
                });
            }
            gix::status::index_worktree::Item::DirectoryContents { entry, .. } => {
                entries.push(Entry {
                    index: None,
                    change: Change {
                        path: entry.rela_path.to_string(),
                        state: "untracked".into(),
                    },
                });
            }
            gix::status::index_worktree::Item::Rewrite {
                source,
                dirwalk_entry,
                ..
            } => {
                use gix::status::index_worktree::RewriteSource;
                let index = match &source {
                    RewriteSource::RewriteFromIndex {
                        source_entry,
                        source_rela_path,
                        ..
                    } => index_side(source_entry, &source_rela_path.to_string()),
                    // A copy of an untracked file has no "before" in the index.
                    RewriteSource::CopyFromDirectoryEntry { .. } => None,
                };
                entries.push(Entry {
                    index,
                    change: Change {
                        path: dirwalk_entry.rela_path.to_string(),
                        state: "moved".into(),
                    },
                });
            }
        }
    }

    entries.sort_by(|a, b| a.change.path.cmp(&b.change.path));
    Ok(entries)
}

fn read_changes(repo: &gix::Repository) -> Result<Vec<Change>, QontrolError> {
    Ok(read_entries(repo)?
        .into_iter()
        .map(|entry| entry.change)
        .collect())
}

fn read_history(repo: &gix::Repository, limit: usize) -> Vec<Commit> {
    let Ok(head) = repo.head_id() else {
        // A repository with no commits yet. Not an error: it is where every
        // project starts, and the surface says so.
        return Vec::new();
    };

    let Ok(walk) = repo.rev_walk([head]).all() else {
        return Vec::new();
    };

    let mut history = Vec::new();
    for info in walk.take(limit) {
        let Ok(info) = info else { continue };
        let Ok(object) = info.object() else { continue };

        let id = info.id().to_string();
        let short = id.chars().take(7).collect();
        let summary = object
            .message()
            .map(|m| m.summary().to_string())
            .unwrap_or_default();
        let (author, time) = match object.author() {
            Ok(a) => (
                a.name.to_string(),
                a.time().ok().map(|t| t.seconds).unwrap_or(0),
            ),
            Err(_) => (String::new(), 0),
        };

        history.push(Commit {
            id,
            short,
            summary,
            author,
            time,
        });
    }
    history
}

fn read_branches(repo: &gix::Repository) -> Vec<Branch> {
    let head = current_branch(repo);
    let Ok(platform) = repo.references() else {
        return Vec::new();
    };
    let Ok(iter) = platform.local_branches() else {
        return Vec::new();
    };

    let mut branches: Vec<Branch> = iter
        .filter_map(|r| r.ok())
        .map(|r| {
            let name = r.name().shorten().to_string();
            let is_head = head.as_deref() == Some(name.as_str());
            Branch {
                name,
                head: is_head,
            }
        })
        .collect();
    branches.sort_by(|a, b| a.name.cmp(&b.name));
    branches
}

/// Read everything the surface draws, from disk.
pub fn read(path: &Path) -> Result<Project, QontrolError> {
    let repo = open(path)?;

    Ok(Project {
        path: path.to_string_lossy().to_string(),
        name: path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string()),
        branch: current_branch(&repo),
        changes: read_changes(&repo)?,
        history: read_history(&repo, 50),
        branches: read_branches(&repo),
        can_commit: sidecar::available(),
    })
}

/// The line-by-line diff for one path, or for every changed path when `path`
/// is `None`. Read with gitoxide; nothing here stages anything. See `diff.rs`.
pub fn diff(repo_path: &Path, rela_path: Option<&str>) -> Result<Vec<FileDiff>, QontrolError> {
    diff::diff(&open(repo_path)?, rela_path)
}

/// Stage everything and write a commit. The guard still applies: a file it
/// warns about stops this, because nothing here has said yes to it.
pub fn commit(repo_path: &Path, message: &str) -> Result<String, QontrolError> {
    commit_selected(repo_path, message, None, &[])
}

/// Check that every named file is one of the changes on screen. The interface
/// can send any string; a commit or a discard acts on what the repository says
/// changed, and nothing else.
fn require_changed<'a>(
    changes: &'a [Change],
    files: &[String],
) -> Result<Vec<&'a Change>, QontrolError> {
    if files.is_empty() {
        return Err(QontrolError::Refused("no files were chosen".into()));
    }
    files
        .iter()
        .map(|file| {
            changes.iter().find(|c| &c.path == file).ok_or_else(|| {
                QontrolError::Refused(format!("{file} is not one of the current changes"))
            })
        })
        .collect()
}

/// Write a commit of `files`, or of every change when `files` is `None`.
///
/// Before anything is staged, the guard reads what would be and refuses while
/// a large or credential-shaped file has no yes in `accepted` (see `guard.rs`).
///
/// libgit2 stages and writes the tree in the sidecar; gitoxide writes the commit
/// and moves the ref here. That order is what leaves the index matching the new
/// HEAD, which is what keeps `git status` honest afterwards. A per-file commit
/// leaves the other changes exactly as they were, unstaged.
pub fn commit_selected(
    repo_path: &Path,
    message: &str,
    files: Option<&[String]>,
    accepted: &[guard::Accepted],
) -> Result<String, QontrolError> {
    let message = message.trim();
    if message.is_empty() {
        return Err(QontrolError::EmptyMessage);
    }

    let repo = open(repo_path)?;
    let changes = read_changes(&repo)?;
    if changes.is_empty() {
        return Err(QontrolError::NothingToCommit);
    }
    if let Some(files) = files {
        require_changed(&changes, files)?;
    }

    guard::require_accepted(&guard::check(repo_path, files)?, accepted)?;

    let path = repo_path.to_string_lossy().to_string();
    match files {
        Some(files) => sidecar::stage(&path, files)?,
        None => sidecar::stage_all(&path)?,
    };
    let tree = sidecar::write_tree(&path)?;

    let tree_id = gix::ObjectId::from_hex(tree.as_bytes()).map_err(|e| {
        QontrolError::Git(format!("the helper returned a tree that is not an id: {e}"))
    })?;

    let parents: Vec<gix::ObjectId> = match repo.head_id() {
        Ok(head) => vec![head.detach()],
        Err(_) => Vec::new(),
    };

    let id = repo
        .commit("HEAD", message, tree_id, parents)
        .map_err(|e| QontrolError::Git(format!("could not write the commit: {e}")))?;

    Ok(id.detach().to_string())
}

/// Make a branch at the commit HEAD points at. It is not switched to: that is a
/// separate act, and one that can be refused.
///
/// A ref write, so gitoxide does it here; the name is checked by gitoxide's own
/// ref-name rules, which are git's.
pub fn create_branch(repo_path: &Path, name: &str) -> Result<(), QontrolError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(QontrolError::Refused("a branch needs a name".into()));
    }
    // A leading dash reads as an option to every git tool that meets it later.
    if name.starts_with('-') {
        return Err(QontrolError::Refused(
            "a branch name cannot start with a dash".into(),
        ));
    }

    let repo = open(repo_path)?;
    let head = repo.head_id().map_err(|_| {
        QontrolError::Refused("make a first commit before starting a branch".into())
    })?;
    let full = format!("refs/heads/{name}");
    // Asked first, because `MustNotExist` lets a ref that already exists with
    // the same value through without a word (measured: making `sketch` twice
    // at the same commit succeeded).
    if repo
        .try_find_reference(full.as_str())
        .ok()
        .flatten()
        .is_some()
    {
        return Err(QontrolError::Refused(format!(
            "a branch called {name} already exists"
        )));
    }
    repo.reference(
        full.as_str(),
        head.detach(),
        gix::refs::transaction::PreviousValue::MustNotExist,
        format!("branch: Created from HEAD by Qontrol as {name}"),
    )
    .map_err(|e| match e {
        gix::reference::edit::Error::FileTransactionPrepare(_) => {
            QontrolError::Refused(format!("a branch called {name} already exists"))
        }
        other => QontrolError::Refused(format!("{name} cannot be a branch name: {other}")),
    })?;
    Ok(())
}

/// Switch to a local branch. Uncommitted changes come along when the switch
/// would not overwrite them; when it would, nothing is touched and the refusal
/// says so.
pub fn switch(repo_path: &Path, name: &str) -> Result<(), QontrolError> {
    let repo = open(repo_path)?;
    let branches = read_branches(&repo);
    let Some(branch) = branches.iter().find(|b| b.name == name) else {
        return Err(QontrolError::Refused(format!(
            "there is no branch called {name}"
        )));
    };
    if branch.head {
        return Ok(());
    }

    let path = repo_path.to_string_lossy().to_string();
    sidecar::switch(&path, name).map_err(|e| match e {
        QontrolError::Sidecar(why) if why.contains("conflict") => QontrolError::Refused(format!(
            "switching to {name} would overwrite changes you have not committed. \
             Commit or discard them first; nothing was changed"
        )),
        other => other,
    })
}

/// Put the named files back as they were last committed.
///
/// Only a modified or removed file can be discarded. A new file has no earlier
/// version, so discarding it would mean deleting it, and Qontrol does not
/// delete a creator's file; a moved or conflicted file is refused too, because
/// what "back" means there is not one thing.
pub fn discard(repo_path: &Path, files: &[String]) -> Result<(), QontrolError> {
    let repo = open(repo_path)?;
    let changes = read_changes(&repo)?;
    for change in require_changed(&changes, files)? {
        match change.state.as_str() {
            "modified" | "removed" => {}
            "untracked" | "added" => {
                return Err(QontrolError::Refused(format!(
                    "{} has never been committed, so there is no earlier version to go back \
                     to. Qontrol does not delete files",
                    change.path
                )))
            }
            other => {
                return Err(QontrolError::Refused(format!(
                    "{} is {other}, and cannot be discarded from here",
                    change.path
                )))
            }
        }
    }

    sidecar::discard(&repo_path.to_string_lossy(), files)
}

/// Resolve a path the interface handed over, refusing anything that is not a
/// directory. The interface can pass any string; this is where that stops.
pub fn resolve(path: &str) -> Result<PathBuf, QontrolError> {
    let path = PathBuf::from(path);
    if !path.is_dir() {
        return Err(QontrolError::Io(format!(
            "{} is not a folder",
            path.display()
        )));
    }
    Ok(path)
}

impl From<QontrolError> for crate::error::QorError {
    fn from(error: QontrolError) -> Self {
        crate::error::QorError::Qontrol(error.to_string())
    }
}
