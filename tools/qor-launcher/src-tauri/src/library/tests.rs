//! The Library's index against real files in a folder of its own (ADR-094 decision 5).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::*;

/// A folder of its own under the system's temporary folder, removed when dropped.
struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        let path =
            std::env::temp_dir().join(format!("qor-library-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).unwrap();
        Self(std::fs::canonicalize(path).unwrap())
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, bytes: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
}

fn picture(path: &Path, w: u32, h: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    image::RgbImage::from_fn(w, h, |x, y| image::Rgb([x as u8, y as u8, 128]))
        .save(path)
        .unwrap();
}

/// A short, silent WAV, with title, artist and album in its RIFF INFO list.
fn song(path: &Path) {
    let samples = 8000u32 * 2; // one second of 16-bit mono at 8 kHz
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(36 + samples).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&8000u32.to_le_bytes());
    wav.extend_from_slice(&16000u32.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&samples.to_le_bytes());
    wav.extend(std::iter::repeat_n(0u8, samples as usize));
    write(path, &wav);
    use lofty::config::WriteOptions;
    use lofty::tag::{Accessor as _, Tag, TagExt as _, TagType};
    let mut tag = Tag::new(TagType::RiffInfo);
    tag.set_title("Midnight Lattice".into());
    tag.set_artist("Fixture Band".into());
    tag.set_album("Stand-ins".into());
    tag.save_to_path(path, WriteOptions::default()).unwrap();
}

fn setup() -> (Dir, Dir, Arc<Library>, String) {
    let home = Dir::new();
    let data = Dir::new();
    let media = home.0.join("Media");
    write(&media.join("Holiday clip.mkv"), b"not really a film");
    write(&media.join("films/Night Drive.MP4"), b"also not a film");
    write(
        &media.join("Café notes.txt"),
        "Shopping list: lanterns, saffron and a Bergamot candle.".as_bytes(),
    );
    write(&media.join("report.pdf"), b"%PDF-1.4 stand-in");
    write(&media.join(".hidden.txt"), b"never indexed");
    write(&media.join("desktop.ini"), b"[.ShellClassInfo]");
    picture(&media.join("pictures/sunset.png"), 640, 360);
    song(&media.join("music/lattice.wav"));
    let library = Library::open(&data.0).unwrap();
    let folder = library.add_folder(&media).unwrap();
    (home, data, library, folder)
}

#[test]
fn a_scan_shelves_each_file_by_kind_with_what_can_be_read_of_it() {
    let (_home, _data, library, folder) = setup();
    let scanned = library.scan(&folder).unwrap();
    assert_eq!(scanned.seen, 6, "hidden and system files are passed over");
    assert_eq!(scanned.added, 6);

    let mut counts = library.counts().unwrap();
    counts.sort();
    assert_eq!(
        counts,
        [
            ("audio".into(), 1),
            ("document".into(), 2),
            ("picture".into(), 1),
            ("video".into(), 2)
        ]
    );
    let sunset = library.shelf(Some("picture"), "added", 10).unwrap();
    assert_eq!((sunset[0].width, sunset[0].height), (Some(640), Some(360)));
    let song = &library.shelf(Some("audio"), "added", 10).unwrap()[0];
    assert_eq!(song.title.as_deref(), Some("Midnight Lattice"));
    assert_eq!(song.artist.as_deref(), Some("Fixture Band"));
    assert_eq!(song.album.as_deref(), Some("Stand-ins"));
    assert!(
        song.duration.is_some_and(|d| (d - 1.0).abs() < 0.1),
        "{:?}",
        song.duration
    );

    let again = library.scan(&folder).unwrap();
    assert_eq!(
        (again.added, again.changed, again.removed),
        (0, 0, 0),
        "nothing new, nothing re-read"
    );
}

#[test]
fn search_finds_names_tags_and_words_as_they_are_typed() {
    let (_home, _data, library, folder) = setup();
    library.scan(&folder).unwrap();
    let names = |found: Vec<Item>| found.into_iter().map(|i| i.name).collect::<Vec<_>>();

    assert_eq!(
        names(library.search("holi", None, 10).unwrap()),
        ["Holiday clip.mkv"],
        "a prefix"
    );
    assert_eq!(
        names(library.search("cafe", None, 10).unwrap()),
        ["Café notes.txt"],
        "without its accent"
    );
    assert_eq!(
        names(library.search("saffron", None, 10).unwrap()),
        ["Café notes.txt"],
        "a document's own words"
    );
    assert_eq!(
        names(library.search("fixture band", None, 10).unwrap()),
        ["lattice.wav"],
        "music tags"
    );
    assert_eq!(
        names(library.search("films", None, 10).unwrap()),
        ["Night Drive.MP4"],
        "its folder"
    );
    assert!(
        library
            .search("night", Some("audio"), 10)
            .unwrap()
            .is_empty(),
        "a kind narrows it"
    );
    assert!(library.search("  ", None, 10).unwrap().is_empty());
    assert!(
        library
            .search("\"; DROP TABLE items; --", None, 10)
            .unwrap()
            .is_empty(),
        "anything typed is words, never a query"
    );
    // Words that are the search syntax's own operators are searched as words, never read as syntax.
    for typed in ["NOT", "OR", "NEAR", "AND notes", "café OR"] {
        assert!(library.search(typed, None, 10).is_ok(), "{typed}");
    }
    assert_eq!(
        library.counts().unwrap().len(),
        4,
        "and the index is still there"
    );
}

#[test]
fn a_rescan_takes_in_what_changed_came_and_went() {
    let (home, _data, library, folder) = setup();
    library.scan(&folder).unwrap();
    let media = home.0.join("Media");
    write(&media.join("Café notes.txt"), b"rewritten: no spices left");
    std::fs::remove_file(media.join("report.pdf")).unwrap();
    write(&media.join("new.txt"), b"fresh");
    let scanned = library.scan(&folder).unwrap();
    assert_eq!((scanned.added, scanned.changed, scanned.removed), (1, 1, 1));
    assert!(
        library.search("saffron", None, 10).unwrap().is_empty(),
        "old words are gone"
    );
    assert_eq!(library.search("spices", None, 10).unwrap().len(), 1);
    assert!(library.search("report", None, 10).unwrap().is_empty());
}

#[test]
fn folders_are_chosen_once_without_overlap_and_taking_one_out_touches_no_file() {
    let (home, _data, library, folder) = setup();
    library.scan(&folder).unwrap();
    assert!(
        library.add_folder(&home.0.join("Media/films")).is_err(),
        "inside one already added"
    );
    assert!(
        library.add_folder(&home.0).is_err(),
        "holding one already added"
    );
    assert!(library.add_folder(&home.0.join("nowhere")).is_err());
    assert_eq!(library.folders().unwrap(), std::slice::from_ref(&folder));

    library.remove_folder(&folder).unwrap();
    assert!(library.folders().unwrap().is_empty());
    assert!(library.counts().unwrap().is_empty());
    assert!(
        home.0.join("Media/Holiday clip.mkv").exists(),
        "the files stay where they are"
    );
}

#[test]
fn a_thumbnail_is_made_once_and_only_for_a_picture_the_library_holds() {
    let (home, data, library, folder) = setup();
    library.scan(&folder).unwrap();
    let sunset = display_path(&home.0.join("Media/pictures/sunset.png"));
    let bytes = library.thumbnail(&sunset).unwrap();
    let small = image::load_from_memory(&bytes).unwrap();
    assert_eq!(
        (small.width(), small.height()),
        (384, 216),
        "fitted inside the thumbnail's edge"
    );
    assert_eq!(
        std::fs::read_dir(data.0.join("thumbnails"))
            .unwrap()
            .count(),
        1,
        "kept"
    );
    assert_eq!(library.thumbnail(&sunset).unwrap(), bytes);

    let outside = Dir::new();
    picture(&outside.0.join("secret.png"), 10, 10);
    assert!(
        library
            .thumbnail(&display_path(&outside.0.join("secret.png")))
            .is_err(),
        "not the Library's"
    );
    assert!(
        library
            .thumbnail(&display_path(&home.0.join("Media/report.pdf")))
            .is_err(),
        "not a picture"
    );
}

#[test]
fn what_was_opened_is_remembered_with_where_it_stopped() {
    let (home, _data, library, folder) = setup();
    library.scan(&folder).unwrap();
    let clip = display_path(&home.0.join("Media/Holiday clip.mkv"));
    assert!(library.shelf(None, "opened", 10).unwrap().is_empty());
    library.opened(&clip, Some(73.5)).unwrap();
    library
        .learned(&clip, Some(600.0), Some(1920), Some(1080))
        .unwrap();
    let shelf = library.shelf(None, "opened", 10).unwrap();
    assert_eq!(shelf.len(), 1);
    assert_eq!(shelf[0].position, Some(73.5));
    assert_eq!(
        (shelf[0].duration, shelf[0].width),
        (Some(600.0), Some(1920))
    );
    assert!(library.opened("C:/not/indexed.mkv", None).is_err());
}

#[test]
fn a_file_that_arrives_in_a_watched_folder_is_found_without_a_scan() {
    let (home, _data, library, folder) = setup();
    library.scan(&folder).unwrap();
    let told = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&told);
    let _watch = watch::Watch::start(
        Arc::clone(&library),
        Arc::new(move || {
            counter.fetch_add(1, Ordering::SeqCst);
        }),
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(200));
    write(&home.0.join("Media/arrived later.txt"), b"a late arrival");
    let deadline = Instant::now() + Duration::from_secs(10);
    while library.search("arrived", None, 10).unwrap().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        library.search("arrived", None, 10).unwrap().len(),
        1,
        "indexed as it arrived"
    );
    std::fs::remove_file(home.0.join("Media/arrived later.txt")).unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    while !library.search("arrived", None, 10).unwrap().is_empty() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        library.search("arrived", None, 10).unwrap().is_empty(),
        "and dropped as it went"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    while told.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(told.load(Ordering::SeqCst) >= 1, "the interface is told");
}

#[test]
fn every_extension_is_shelved_once() {
    for (name, kind) in [
        ("a.MKV", "video"),
        ("a.ts", "video"),
        ("a.flac", "audio"),
        ("a.HEIC", "picture"),
        ("a.pdf", "document"),
        ("a.docx", "document"),
        ("a.tsx", "document"),
        ("a.exe", "other"),
        ("noextension", "other"),
    ] {
        assert_eq!(kind_of(Path::new(name)), kind, "{name}");
    }
}
