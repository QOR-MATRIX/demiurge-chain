//! QQ's scenes on disk (ADR-082, DIRECTION P3.1).
//!
//! A QQ scene is a `.qq.json` file in the `scenes/` folder of a Qontrol project, so it is versioned, diffed and
//! committed by the Projects surface like any other file. The webview has no filesystem access
//! (`capabilities/default.json`), so QQ's editor reads and writes scenes only through these commands, and they take
//! nothing on trust: the project must be a repository the person opened, the name is one plain word, the scene must
//! have QQ's shape and stay within its size, and nothing is written outside the project's own `scenes/` folder.
//!
//! The editor writes the scene in its canonical form (`src/qq/runtime/scene.ts`), with stable key order and one value
//! per line, so a change reads as a change in Projects. This module checks that form; it does not rewrite it.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::QorError;
use crate::qontrol::{self, Project};

/// The folder inside a project that holds its scenes.
pub const SCENES_DIR: &str = "scenes";
/// A scene file's ending.
pub const EXTENSION: &str = ".qq.json";
/// The format this launcher writes and reads.
pub const FORMAT: u64 = 1;
/// Larger than any scene P3.1's editor can make, and small enough that Qontrol's guard never has to hold one back.
pub const MAX_SCENE_BYTES: usize = 2 * 1024 * 1024;
/// The blueprint's target is hundreds of entities; this is the ceiling, not the aim.
pub const MAX_ENTITIES: usize = 4096;

fn refuse(message: impl Into<String>) -> QorError {
    QorError::Qontrol(message.into())
}

/// A scene's name: a lowercase word of letters, digits and dashes, starting with a letter or digit, at most 63 long.
/// It names a file, so it can never be a path.
pub fn check_name(name: &str) -> Result<&str, QorError> {
    let ok = !name.is_empty()
        && name.len() <= 63
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(name)
    } else {
        Err(refuse(
            "a scene's name is lowercase letters, digits and dashes, starting with a letter or digit, at most 63 long",
        ))
    }
}

fn is_entity_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Whether `text` has a QQ scene's shape: format 1, a size, and entities each with a unique id and a transform.
/// The editor checks every field when it reads a scene; the host checks what keeps a file sane on disk.
pub fn check_scene(text: &str) -> Result<(), QorError> {
    if text.len() > MAX_SCENE_BYTES {
        return Err(refuse(format!(
            "a scene is at most {} MB",
            MAX_SCENE_BYTES / (1024 * 1024)
        )));
    }
    let scene: Value =
        serde_json::from_str(text).map_err(|e| refuse(format!("the scene is not JSON: {e}")))?;
    if scene.get("qq").and_then(Value::as_u64) != Some(FORMAT) {
        return Err(refuse(format!(
            "this launcher reads QQ scenes of format {FORMAT}"
        )));
    }
    let size = |key: &str| {
        scene
            .get("size")
            .and_then(|s| s.get(key))
            .and_then(Value::as_f64)
            .is_some_and(|v| (64.0..=4096.0).contains(&v))
    };
    if !size("w") || !size("h") {
        return Err(refuse("a scene's size is 64 to 4096 on each side"));
    }
    let entities = scene
        .get("entities")
        .and_then(Value::as_array)
        .ok_or_else(|| refuse("a scene lists its entities"))?;
    if entities.len() > MAX_ENTITIES {
        return Err(refuse(format!(
            "a scene has at most {MAX_ENTITIES} entities"
        )));
    }
    let mut seen = std::collections::BTreeSet::new();
    for entity in entities {
        let id = entity
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| is_entity_id(id))
            .ok_or_else(|| {
                refuse("every entity has an id of lowercase letters, digits and dashes")
            })?;
        if !seen.insert(id) {
            return Err(refuse(format!("two entities share the id {id}")));
        }
        if !entity.get("transform").is_some_and(Value::is_object) {
            return Err(refuse(format!("entity {id} has no transform")));
        }
    }
    Ok(())
}

/// The project's `scenes/` folder, made if it is missing, and proven to be inside the project.
fn scenes_dir(project: &Path, create: bool) -> Result<PathBuf, QorError> {
    qontrol::open(project)?;
    let root = project
        .canonicalize()
        .map_err(|e| refuse(format!("the project folder could not be read: {e}")))?;
    let dir = root.join(SCENES_DIR);
    if !dir.exists() {
        if !create {
            return Ok(dir);
        }
        std::fs::create_dir(&dir)
            .map_err(|e| refuse(format!("the scenes folder could not be made: {e}")))?;
    }
    // A link named `scenes` that points elsewhere would carry a write out of the project.
    let real = dir
        .canonicalize()
        .map_err(|e| refuse(format!("the scenes folder could not be read: {e}")))?;
    if !real.starts_with(&root) || !real.is_dir() {
        return Err(refuse(
            "the scenes folder must be a folder inside the project",
        ));
    }
    Ok(real)
}

/// Write a scene as `scenes/<name>.qq.json`, replacing one of that name. The file is written beside its final name
/// and moved into place, so a scene on disk is always a whole one.
pub fn save(project: &Path, name: &str, text: &str) -> Result<PathBuf, QorError> {
    let name = check_name(name)?;
    check_scene(text)?;
    let dir = scenes_dir(project, true)?;
    let file = dir.join(format!("{name}{EXTENSION}"));
    if std::fs::symlink_metadata(&file).is_ok_and(|m| !m.is_file()) {
        return Err(refuse(format!("{name}{EXTENSION} is not a plain file")));
    }
    let staged = dir.join(format!(".{name}{EXTENSION}.saving"));
    let mut body = text.to_string();
    if !body.ends_with('\n') {
        body.push('\n');
    }
    std::fs::write(&staged, body)
        .map_err(|e| refuse(format!("the scene could not be written: {e}")))?;
    std::fs::rename(&staged, &file).map_err(|e| {
        let _ = std::fs::remove_file(&staged);
        refuse(format!("the scene could not be put in place: {e}"))
    })?;
    Ok(file)
}

/// Read `scenes/<name>.qq.json`, checked as [`save`] checks it.
pub fn load(project: &Path, name: &str) -> Result<String, QorError> {
    let name = check_name(name)?;
    let file = scenes_dir(project, false)?.join(format!("{name}{EXTENSION}"));
    let meta = std::fs::symlink_metadata(&file)
        .map_err(|_| refuse(format!("there is no scene called {name}")))?;
    if !meta.is_file() {
        return Err(refuse(format!("{name}{EXTENSION} is not a plain file")));
    }
    if meta.len() > MAX_SCENE_BYTES as u64 {
        return Err(refuse(format!(
            "a scene is at most {} MB",
            MAX_SCENE_BYTES / (1024 * 1024)
        )));
    }
    let text = std::fs::read_to_string(&file)
        .map_err(|e| refuse(format!("the scene could not be read: {e}")))?;
    check_scene(&text)?;
    Ok(text)
}

/// The names of the project's scenes, sorted. A file whose name is not a scene name is not listed.
pub fn list(project: &Path) -> Result<Vec<String>, QorError> {
    let dir = scenes_dir(project, false)?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .map_err(|e| refuse(format!("the scenes folder could not be read: {e}")))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|entry| {
            let file = entry.file_name().to_string_lossy().to_string();
            let name = file.strip_suffix(EXTENSION)?.to_string();
            check_name(&name).is_ok().then_some(name)
        })
        .collect();
    names.sort();
    Ok(names)
}

/// Save a scene into an open project, and return the project as it now reads, so Projects shows the change.
#[tauri::command]
pub async fn qq_save_scene(path: String, name: String, scene: String) -> Result<Project, QorError> {
    let project = qontrol::resolve(&path)?;
    save(&project, &name, &scene)?;
    Ok(qontrol::read(&project)?)
}

/// Read one of a project's scenes.
#[tauri::command]
pub async fn qq_load_scene(path: String, name: String) -> Result<String, QorError> {
    load(&qontrol::resolve(&path)?, &name)
}

/// The names of a project's scenes.
#[tauri::command]
pub async fn qq_list_scenes(path: String) -> Result<Vec<String>, QorError> {
    list(&qontrol::resolve(&path)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE: &str = r##"{
  "qq": 1,
  "name": "first-light",
  "size": { "w": 960, "h": 540 },
  "background": "#07080c",
  "entities": [
    { "id": "e1", "name": "Core", "transform": { "x": 0, "y": 0, "rotation": 0, "scale": 1 } }
  ]
}"##;

    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("a folder");
        qontrol::open_or_init(dir.path()).expect("a repository");
        dir
    }

    #[test]
    fn a_scene_is_saved_listed_and_read_back_exactly() {
        let dir = project();
        let file = save(dir.path(), "first-light", SCENE).expect("saved");
        assert!(file.ends_with(Path::new("scenes").join("first-light.qq.json")));
        assert_eq!(
            list(dir.path()).expect("listed"),
            vec!["first-light".to_string()]
        );
        assert_eq!(
            load(dir.path(), "first-light").expect("read"),
            format!("{SCENE}\n")
        );
        // Nothing staged is left behind.
        let leftovers: Vec<_> = std::fs::read_dir(dir.path().join("scenes"))
            .expect("folder")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert_eq!(leftovers, vec!["first-light.qq.json".to_string()]);
    }

    #[test]
    fn saving_again_replaces_the_scene() {
        let dir = project();
        save(dir.path(), "level", SCENE).expect("saved");
        let changed = SCENE.replace("\"Core\"", "\"Heart\"");
        save(dir.path(), "level", &changed).expect("saved again");
        assert!(load(dir.path(), "level").expect("read").contains("Heart"));
        assert_eq!(list(dir.path()).expect("listed").len(), 1);
    }

    #[test]
    fn a_saved_scene_shows_as_a_change_in_the_project() {
        let dir = project();
        save(dir.path(), "first-light", SCENE).expect("saved");
        // A new folder is one untracked change, as git reports it; the file inside it is what a commit takes.
        let read = qontrol::read(dir.path()).expect("project");
        assert!(
            read.changes
                .iter()
                .any(|c| c.path == "scenes" || c.path == "scenes/first-light.qq.json"),
            "{:?}",
            read.changes
        );
    }

    #[test]
    fn a_name_is_one_plain_word() {
        for name in ["level-1", "a", "0x", &"a".repeat(63)] {
            assert!(check_name(name).is_ok(), "{name}");
        }
        for name in [
            "",
            "-lead",
            "Level",
            "../escape",
            "a/b",
            "a\\b",
            "a.b",
            "a b",
            "ä",
            &"a".repeat(64),
        ] {
            assert!(check_name(name).is_err(), "{name:?}");
        }
        let dir = project();
        assert!(save(dir.path(), "../escape", SCENE).is_err());
        assert!(!dir
            .path()
            .parent()
            .expect("parent")
            .join("escape.qq.json")
            .exists());
    }

    #[test]
    fn only_a_scene_of_qq_s_shape_is_written() {
        let bad = [
            "not json".to_string(),
            SCENE.replace("\"qq\": 1", "\"qq\": 2"),
            SCENE.replace("\"w\": 960", "\"w\": 10"),
            SCENE.replace("\"entities\": [", "\"things\": ["),
            SCENE.replace("\"id\": \"e1\"", "\"id\": \"E 1\""),
            SCENE.replace(
                ", \"transform\": { \"x\": 0, \"y\": 0, \"rotation\": 0, \"scale\": 1 }",
                "",
            ),
            SCENE.replace(
                "{ \"id\": \"e1\", \"name\": \"Core\",",
                "{ \"id\": \"e1\", \"transform\": {} }, { \"id\": \"e1\", \"name\": \"Core\",",
            ),
            format!("{SCENE}{}", " ".repeat(MAX_SCENE_BYTES)),
        ];
        let dir = project();
        for text in &bad {
            assert!(
                check_scene(text).is_err(),
                "{}",
                &text[..text.len().min(120)]
            );
            assert!(save(dir.path(), "bad", text).is_err());
        }
        assert!(list(dir.path()).expect("listed").is_empty());
    }

    #[test]
    fn a_folder_that_is_not_a_repository_is_refused() {
        let dir = tempfile::tempdir().expect("a folder");
        assert!(save(dir.path(), "first-light", SCENE).is_err());
        assert!(!dir.path().join("scenes").exists(), "nothing is made in it");
    }

    #[test]
    fn a_missing_scene_is_not_found_and_an_empty_project_lists_none() {
        let dir = project();
        assert!(list(dir.path()).expect("listed").is_empty());
        assert!(load(dir.path(), "nothing-here").is_err());
    }

    #[test]
    fn files_that_are_not_scenes_are_not_listed() {
        let dir = project();
        save(dir.path(), "real", SCENE).expect("saved");
        let scenes = dir.path().join("scenes");
        std::fs::write(scenes.join("Notes.qq.json"), SCENE).expect("write");
        std::fs::write(scenes.join("readme.md"), "hi").expect("write");
        std::fs::create_dir(scenes.join("folder.qq.json")).expect("dir");
        assert_eq!(list(dir.path()).expect("listed"), vec!["real".to_string()]);
    }

    #[cfg(unix)]
    #[test]
    fn a_scenes_link_out_of_the_project_is_refused() {
        let dir = project();
        let outside = tempfile::tempdir().expect("elsewhere");
        std::os::unix::fs::symlink(outside.path(), dir.path().join("scenes")).expect("link");
        assert!(save(dir.path(), "first-light", SCENE).is_err());
        assert!(std::fs::read_dir(outside.path())
            .expect("read")
            .next()
            .is_none());
    }
}
