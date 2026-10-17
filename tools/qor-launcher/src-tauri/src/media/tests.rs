//! QOR Media's side of the launcher: the queue `media.next` waits on, and what its reports do to the Library.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use super::*;

struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("qor-media-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).unwrap();
        Self(std::fs::canonicalize(path).unwrap())
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn handler(
    wait: Duration,
) -> (
    MediaHandler,
    Arc<Library>,
    Dir,
    Dir,
    String,
    Arc<AtomicUsize>,
) {
    let home = Dir::new();
    let data = Dir::new();
    std::fs::write(home.0.join("film.mkv"), b"a stand-in film").unwrap();
    let library = Library::open(&data.0).unwrap();
    let folder = library.add_folder(&home.0).unwrap();
    library.scan(&folder).unwrap();
    let film = crate::library::display_path(&home.0.join("film.mkv"));
    let told = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&told);
    let handler = MediaHandler {
        queue: Arc::new(Queue::new(wait)),
        library: Arc::clone(&library),
        changed: Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }),
    };
    (handler, library, home, data, film, told)
}

#[tokio::test]
async fn media_next_answers_a_command_at_once_and_nothing_after_the_wait() {
    let (h, _library, _home, _data, film, _) = handler(Duration::from_millis(300));
    assert_eq!(h.capabilities(), ["media.next"]);
    assert!(
        h.quiet("media.next"),
        "asked every few seconds, logged at debug"
    );

    let started = Instant::now();
    assert_eq!(
        h.handle("media.next", Value::Null).await.unwrap(),
        json!({ "command": null })
    );
    assert!(started.elapsed() >= Duration::from_millis(280), "it waited");

    let queue = Arc::clone(&h.queue);
    let pushing = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        queue.push(Command::Open {
            path: film,
            kind: "video".into(),
            position: Some(12.0),
            rect: Some(Rect {
                x: 80,
                y: 60,
                w: 1280,
                h: 720,
            }),
            parent: Some(4242),
        });
    });
    let started = Instant::now();
    let answer = h
        .handle("media.next", json!({ "reports": [] }))
        .await
        .unwrap();
    pushing.await.unwrap();
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "a command is answered as it arrives"
    );
    assert_eq!(answer["command"]["type"], "open");
    assert_eq!(answer["command"]["kind"], "video");
    assert_eq!(answer["command"]["position"], 12.0);
    assert_eq!(
        answer["command"]["rect"],
        json!({ "x": 80, "y": 60, "w": 1280, "h": 720 })
    );
    assert_eq!(answer["command"]["parent"], 4242);
}

#[tokio::test]
async fn a_newer_place_replaces_an_older_one_and_order_is_kept() {
    let (h, _library, _home, _data, _film, _) = handler(Duration::from_millis(100));
    let at = |x| Command::Place {
        rect: Rect {
            x,
            y: 0,
            w: 10,
            h: 10,
        },
        parent: None,
    };
    h.queue.push(at(1));
    h.queue.push(Command::Hide);
    h.queue.push(at(2));
    assert_eq!(h.queue.next().await, Some(Command::Hide));
    assert_eq!(h.queue.next().await, Some(at(2)), "only the newest place");
    assert_eq!(h.queue.next().await, None);
}

#[tokio::test]
async fn reports_record_where_playback_stopped_and_what_was_learned() {
    let (h, library, _home, _data, film, told) = handler(Duration::from_millis(50));
    let answer = h
        .handle(
            "media.next",
            json!({ "reports": [
                { "type": "position", "path": film, "position": 95.5 },
                { "type": "learned", "path": film, "duration": 600.0, "width": 3840, "height": 2160 },
                { "type": "position", "path": "C:/nowhere/else.mkv", "position": 1.0 },
                { "type": "closed" },
            ] }),
        )
        .await
        .unwrap();
    assert_eq!(answer, json!({ "command": null }));
    let item = library.item(&film).unwrap();
    assert_eq!(item.position, Some(95.5));
    assert_eq!(
        (item.duration, item.width, item.height),
        (Some(600.0), Some(3840), Some(2160))
    );
    assert!(
        library.item("C:/nowhere/else.mkv").is_err(),
        "a path the Library does not hold is passed over"
    );
    assert_eq!(
        told.load(Ordering::SeqCst),
        1,
        "the Library's view is told once"
    );

    let refused = h
        .handle("media.next", json!({ "reports": [{ "type": "explode" }] }))
        .await;
    assert!(refused.is_err(), "a report of no known kind is refused");
}

#[test]
fn qor_media_is_found_where_qor_media_points_or_where_it_is_deployed() {
    let dir = Dir::new();
    assert_eq!(find(None, Some(dir.0.clone())), None);
    let deployed = dir.0.join("qor-media").join(if cfg!(windows) {
        "qor-media.exe"
    } else {
        "qor-media"
    });
    std::fs::create_dir_all(deployed.parent().unwrap()).unwrap();
    std::fs::write(&deployed, b"").unwrap();
    assert_eq!(find(None, Some(dir.0.clone())), Some(deployed.clone()));
    let named = dir.0.join("elsewhere.exe");
    assert_eq!(
        find(Some(named.clone().into()), Some(dir.0.clone())),
        None,
        "a name that points at nothing is not swapped for another copy"
    );
    std::fs::write(&named, b"").unwrap();
    assert_eq!(
        find(Some(named.clone().into()), Some(dir.0.clone())),
        Some(named)
    );
}
