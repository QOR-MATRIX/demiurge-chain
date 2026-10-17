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
fn only_colours_written_rrggbb_make_a_theme() {
    let given: std::collections::BTreeMap<String, String> = [
        ("void", "#06070A"),
        ("accent-bright", "#ff8a33"),
        ("ink", "red"),
        ("surface", "#12151C; background: url(x)"),
        ("Bad Name", "#000000"),
        ("edge", "#12g45z"),
        ("well", "#1);x:y"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();
    let kept = theme(given);
    assert_eq!(kept.keys().collect::<Vec<_>>(), ["accent-bright", "void"]);
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

/// One second of a 440 Hz tone: 16-bit mono PCM at 8 kHz in a WAV file.
fn tone() -> Vec<u8> {
    let rate = 8000u32;
    let mut pcm = Vec::with_capacity(rate as usize * 2);
    for i in 0..rate {
        let sample = (8000.0
            * (2.0 * std::f64::consts::PI * 440.0 * f64::from(i) / f64::from(rate)).sin())
            as i16;
        pcm.extend_from_slice(&sample.to_le_bytes());
    }
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    for field in [
        16u32.to_le_bytes().to_vec(),
        1u16.to_le_bytes().to_vec(),
        1u16.to_le_bytes().to_vec(),
        rate.to_le_bytes().to_vec(),
        (rate * 2).to_le_bytes().to_vec(),
        2u16.to_le_bytes().to_vec(),
        16u16.to_le_bytes().to_vec(),
    ] {
        wav.extend_from_slice(&field);
    }
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(pcm.len() as u32).to_le_bytes());
    wav.extend_from_slice(&pcm);
    wav
}

/// The real QOR Media, started through the product host as the Library starts it: it is welcomed, opens what it is
/// sent, plays it, and its report of the file's length reaches the Library. Needs QOR Media built and deployed
/// (`pwsh tools/qor-media/build.ps1 -Deploy`, which needs the owner's Qt), so it runs only when asked:
/// `cargo test --lib media::tests::qor_media -- --ignored`.
#[test]
#[ignore = "needs QOR Media deployed by tools/qor-media/build.ps1 -Deploy"]
fn qor_media_started_by_the_launcher_plays_and_reports_to_the_library() {
    // Its own runtime, shut without waiting on the channel: QOR Media ends with this process, as with the launcher.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(played_and_reported());
    runtime.shutdown_timeout(Duration::from_secs(1));
}

async fn played_and_reported() {
    let program = find(
        std::env::var_os("QOR_MEDIA"),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    )
    .expect("QOR Media is deployed");
    let home = Dir::new();
    let data = Dir::new();
    std::fs::write(home.0.join("tone.wav"), tone()).unwrap();
    let library = Library::open(&data.0).unwrap();
    let folder = library.add_folder(&home.0).unwrap();
    library.scan(&folder).unwrap();
    let path = crate::library::display_path(&home.0.join("tone.wav"));
    let item = library.item(&path).unwrap();
    assert_eq!(item.position, None, "it has not been played");

    let queue = Arc::new(Queue::new(Duration::from_secs(2)));
    let told = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&told);
    let handler = Arc::new(MediaHandler {
        queue: Arc::clone(&queue),
        library: Arc::clone(&library),
        changed: Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }),
    });
    let exited = Arc::new(parking_lot::Mutex::new(None));
    let noted = Arc::clone(&exited);
    host::launch(
        host::Launch {
            product: host::ProductRecord {
                id: "qor-media",
                name: "QOR Media",
                path: program.clone(),
            },
            args: vec![],
            cwd: program.parent().unwrap().to_path_buf(),
        },
        handler,
        move |e| *noted.lock() = Some(e),
    )
    .await
    .expect("QOR Media starts");
    queue.push(Command::Open {
        path: item.path.clone(),
        kind: item.kind.clone(),
        position: None,
        rect: None,
        parent: None,
    });
    let started = Instant::now();
    while !queue.commands.lock().is_empty() {
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "QOR Media never asked"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }

    // Its report of what it learned arrives on its next media.next, and the Library's view is told.
    let started = Instant::now();
    while told.load(Ordering::SeqCst) == 0 {
        if let Some(e) = exited.lock().clone() {
            panic!("QOR Media ended: {e:?}");
        }
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "QOR Media reported nothing"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let duration = library.item(&path).unwrap().duration.unwrap();
    assert!(
        (duration - 1.0).abs() < 0.1,
        "the tone is a second long, not {duration}"
    );

    // Hidden, it says where it stopped.
    queue.push(Command::Hide);
    let started = Instant::now();
    while library.item(&path).unwrap().position.is_none() {
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "no position was reported"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
