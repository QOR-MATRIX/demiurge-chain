# QOR Media

The Library's player, viewer and editor ([ADR-094](../../docs/decisions/ADR-094-the-library-and-qor-media.md),
[`docs/blueprints/library.md`](../../docs/blueprints/library.md), L9.3 and L9.4). A Qt 6 program in C++20 and QML that the
QOR Launcher starts when a person opens a file in the Library. Its window becomes a child of the launcher's and sits
over the Library, so it moves, resizes and minimises with the launcher.

| What opens | Where | How |
| --- | --- | --- |
| Video and sound | `qml/Player.qml` | Qt Multimedia on its FFmpeg backend: every container and codec FFmpeg reads. Resumes where it stopped; scrubbing with the time under the pointer; ±10 s; speed; volume; audio and subtitle tracks; full screen |
| Pictures | `qml/Viewer.qml` | Every format Qt's image plugins read (JPEG, PNG, GIF, WebP, TIFF, SVG, BMP, ICO, TGA), turned as the camera wrote it; zoom about the pointer, drag, 1:1 on a double click, the folder's others next and previous, turn, slideshow |
| Text, Markdown and code | `qml/Editor.qml` | Read and saved as written (UTF-8 or UTF-16, byte-order mark and CRLF kept, `src/textfile.cpp`), coloured in the launcher's theme; line numbers for code; Markdown preview beside the words; find; asks before unsaved words are lost |
| PDF, Office documents, anything else | `qml/Elsewhere.qml` | Says so, and opens it in the program Windows chooses. PDF arrives with L9.5 (Qt PDF, which the owner's Qt installation does not have yet), Office documents with L9.7 |

Keys: Space or K play and pause; arrows seek 5 s (player) or move between pictures (viewer); J and L 10 s; Up and Down
the volume; M mutes; F full screen; [ and ] speed; C subtitles; A audio track; R turns a picture; S a slideshow; Ctrl+S
saves; Ctrl+F finds; Ctrl+P previews Markdown; Alt+Z wraps lines; Esc goes back to the Library.

## How the launcher talks to it

Through the QOR product host (ADR-046): a private named pipe and a one-time token on stdin. QOR Media keeps one request
waiting, `media.next`, which the launcher answers with its next command (`open`, `place`, `hide`, `theme`) or with nothing
after its wait (25 s); it asks again at once. What QOR Media has to say (a file's length and picture size, where playback
stopped, that it closed) rides on its next `media.next`, so it reaches the Library within one wait. The launcher's side
is `tools/qor-launcher/src-tauri/src/media.rs`; this side is `src/medialink.cpp`. When the launcher goes, QOR Media goes.

Started by hand with a file (`qor-media.exe film.mkv`) it opens that file in a window of its own.

## Build, test, deploy

Needs the owner's Qt 6.12 (commercial licence) at `C:\Qt\6.12.0\msvc2022_64` with Qt Multimedia, Visual Studio 2022's C++
tools, and CMake and Ninja from `C:\Qt\Tools`. CI has no licensed Qt, so these run on the owner's PC, as QQ's do.

```
pwsh tools/qor-media/build.ps1             # build and run the four Qt Test suites
pwsh tools/qor-media/build.ps1 -Run a.mkv  # build, then open a file
pwsh tools/qor-media/build.ps1 -Deploy     # build, test, then put a self-contained copy in %LOCALAPPDATA%\qor-media
```

The launcher opens files in `%LOCALAPPDATA%\qor-media\qor-media.exe`, or wherever `QOR_MEDIA` points; without either it
hands files to the program Windows chooses. Once deployed, the launcher's own test of the whole path (its product host
starting the real QOR Media, which plays a generated tone and reports its length to a Library) runs with:

```
cd tools/qor-launcher/src-tauri
cargo test --lib media::tests::qor_media -- --ignored
```

| Suite | What it holds |
| --- | --- |
| `tst_medialink` | The hello carries the token once; a launcher without `media.next` is not asked; commands arrive as signals; reports ride on the next request, only the newest position of a file, at most 64; a stale answer is ignored; an oversized frame ends the link; the launcher going away after its welcome is lost, a refused handshake is not |
| `tst_files` | Kinds by extension, as the launcher's index shelves them; which documents the editor opens; a folder's pictures in the order people read numbers |
| `tst_textfile` | LF, CRLF, UTF-8 with a byte-order mark and UTF-16 each written back as they were; a binary, oversized or missing file is refused and never written |
| `tst_playback` | The whole program: a generated tone plays and its length, where it stopped and that it closed are reported; Windows' own sample video decodes through FFmpeg; a picture shows with its size and the arrow key moves to the next; unsaved words hold the window and nothing is written; PDF says what opens it; the launcher's colours are worn; it docks as a child of a window known only by its handle, at the launcher's physical rectangle in logical pixels, and lifts out to full screen and back; hide hides |

Every rule above was checked against a fault planted in it: the save guard, the newest-position rule, the frame limit,
numeric order, the unsaved-words guard, the length report, the closed report, and docking (no parent, physical pixels
taken as logical at 125% scaling, not put back after full screen).

## Not yet

PDF (L9.5); Office documents (L9.7); HEIC, AVIF, JPEG XL and camera RAW pictures (L9.8); video thumbnails on the Library's
tiles; a game controller inside QOR Media (the Library's own view takes one); docking on macOS and Linux.
