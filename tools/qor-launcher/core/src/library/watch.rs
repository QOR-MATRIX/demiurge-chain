//! Following the Library's folders as files come, change and go (docs/blueprints/library.md section 2).
//!
//! One watcher for every folder. Each change is applied to the index as it arrives, and `changed` is told once a burst
//! has settled, so the interface refreshes once rather than per file.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
use parking_lot::Mutex;

use super::{display_path, Library};
use crate::error::{QorError, QorResult};

/// How long a burst of changes is let settle before the interface is told.
const SETTLE: Duration = Duration::from_millis(400);

pub struct Watch {
    watcher: Mutex<RecommendedWatcher>,
}

impl Watch {
    /// Watch every folder the Library holds, applying changes to it and calling `changed` after each burst.
    pub fn start(library: Arc<Library>, changed: Arc<dyn Fn() + Send + Sync>) -> QorResult<Self> {
        let (sender, receiver) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let watcher = notify::recommended_watcher(move |event| {
            let _ = sender.send(event);
        })
        .map_err(|e| QorError::Internal(format!("the Library cannot watch folders: {e}")))?;
        let applying = Arc::clone(&library);
        std::thread::Builder::new()
            .name("library-watch".into())
            .spawn(move || {
                while let Ok(first) = receiver.recv() {
                    apply(&applying, first);
                    // Gather the rest of the burst, then tell the interface once.
                    while let Ok(more) = receiver.recv_timeout(SETTLE) {
                        apply(&applying, more);
                    }
                    changed();
                }
            })
            .map_err(|e| QorError::Internal(format!("the Library cannot watch folders: {e}")))?;
        let watch = Self {
            watcher: Mutex::new(watcher),
        };
        for folder in library.folders()? {
            watch.add(&folder);
        }
        Ok(watch)
    }

    /// Watch one more folder. A folder that has gone is passed over; the next scan drops its files.
    pub fn add(&self, folder: &str) {
        let _ = self
            .watcher
            .lock()
            .watch(Path::new(folder), RecursiveMode::Recursive);
    }

    pub fn remove(&self, folder: &str) {
        let _ = self.watcher.lock().unwatch(Path::new(folder));
    }
}

fn apply(library: &Library, event: notify::Result<Event>) {
    let Ok(event) = event else { return };
    for path in &event.paths {
        let gone = matches!(event.kind, EventKind::Remove(_)) || !path.exists();
        if gone {
            let _ = library.forget(&display_path(path));
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if let Ok(Some(folder)) = library.folder_of(path) {
            let _ = library.upsert(&folder, path);
        }
    }
}
