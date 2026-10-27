# The Library and QOR Media

**What it is:** everything a person has, in one SteamOS-like view inside the launcher (games and work they own on the
chain, and their own videos, music, pictures and documents), searched instantly and opened without friction in **QOR
Media**, a native Qt 6 player, viewer and editor docked inside the launcher. **Decided by**
[ADR-094](../decisions/ADR-094-the-library-and-qor-media.md) (accepted by the owner, 16 October 2026). **Roadmap:** L9 in
[`DIRECTION.md`](../DIRECTION.md); buying and installing owned work stays L7.1 and P5 ([`market.md`](market.md)). This
document is the design; what exists is in [`SYSTEMS.md`](../SYSTEMS.md). **Built so far** (in `main` since #37, 17 October
2026): the index, the view with keyboard and controller, and QOR Media's player, viewer and editor
(`tools/qor-media/`, [its README](../../tools/qor-media/README.md)), L9.1 to L9.4.

The owner's brief, 16 October 2026: a SteamOS-style in-launcher view; a premium media player for all video and image
codecs and formats; files openable from the Library; local folders browsed with ease, their content searchable and
playable with zero friction; PDFs and documents viewed and edited in a premium editor; Qt-based options for everything
from now on.

## 1. Shape

```
 QOR Launcher (Tauri)                                                   QOR Media (Qt 6, tools/qor-media)
 ┌───────────────────────────────────────────────┐   product host     ┌──────────────────────────────────────┐
 │ Library view (React): hero, shelves, folders, │   channel + token  │ a borderless window, a child of the   │
 │ search, keyboard and controller navigation    │◄──────────────────►│ launcher's, placed over its content   │
 ├───────────────────────────────────────────────┤  media.next /      │ area: Player (Qt Multimedia, FFmpeg), │
 │ Rust host: `library` — folders, scan, watch,  │  (reports ride on) │ Viewer (images), Editor (text, PDF)   │
 │ index (SQLite FTS5), thumbnails, metadata     │                    └──────────────────────────────────────┘
 └───────────────────────────────────────────────┘
```

- **The view is the launcher's own screen**, so the rail, themes and QFX stay around it.
- **QOR Media is started on demand** under the product host (ADR-046): a private channel and a one-time token, ended
  with the launcher. It keeps one request waiting, `media.next`, which the launcher answers with its next command (open
  a file, sit at a rectangle, hide, wear the launcher's colours) or with nothing after 25 seconds. What QOR Media has to
  say (a file's length and size learned, where playback stopped, that it closed) rides on its next `media.next`, so it
  arrives within one wait. The protocol is unchanged: `media.next` is the one method the launcher announces.
- **Docked:** on Windows QOR Media's window is a child of the launcher's, so it moves, resizes, minimises and stacks
  with it; the launcher sends its content rectangle in physical pixels whenever it changes.

## 2. The index (the launcher's core, `core/src/library/`)

| Part | What it does |
| --- | --- |
| Folders | The person's choice; Videos, Music, Pictures and Documents offered at first run. Kept in the launcher's settings |
| Scan | Walks each folder, skipping hidden and system files, and records each file's kind (video, audio, picture, document, other) by its extension, with name, path, size and modified time |
| Watch | Follows changes as they happen, and rescans a folder at start |
| Metadata | Read as files are first seen: a picture's size and photo details (when taken, camera); music tags (title, artist, album, duration); a text document's first words; video duration and size as QOR Media reports them |
| Search | SQLite's full-text index (FTS5) over names, folders, tags and text, ranked; results as the person types |
| Thumbnails | Pictures scaled in the host and kept in the launcher's local cache; video frames from QOR Media are not built yet |

Nothing in the index leaves the PC, and it holds paths, never file contents beyond a document's first words.

## 3. The view

A **hero** for what was last opened (resume where it stopped); **shelves**: Continue, Recently added, Games (owned
work, when L7.1 exists), Videos, Music, Pictures, Documents; a **Folders** browser with breadcrumbs; and **search**
always one key away. Tiles are large, with art or a thumbnail, a title and one fact (duration, size, pages). Arrow keys
and a game controller move a focus ring; Enter opens; Escape goes back.

## 4. QOR Media

- **Player:** Qt Multimedia's `MediaPlayer` on its FFmpeg backend, so every container and codec FFmpeg reads plays:
  MKV, MP4, MOV, AVI, WebM, TS, FLV, WMV; H.264, HEVC, AV1, VP9, ProRes, MPEG-2; AAC, FLAC, Opus, DTS, AC-3, MP3, WAV.
  Controls: play and pause, a scrubbing bar, skip, speed, volume, audio and subtitle tracks, full screen, resume from
  where it stopped.
- **Viewer:** pictures in every format Qt's image plugins read (JPEG, PNG, GIF, WebP, TIFF, SVG, BMP, ICO, TGA), fitted,
  zoomed and panned, with the folder's others next and previous; HEIC, AVIF, JPEG XL and camera RAW added as plugins.
- **Editor:** plain text, Markdown with a preview, and code with highlighting; saved where it was. **PDF:** shown with Qt
  PDF; annotated, highlighted, filled, signed, and pages reordered, rotated and deleted through PDFium's own interface.
  **Office documents** are release 2 (LibreOfficeKit, its own record first).

## 5. Not in release 1

Office documents; images beyond Qt's plugins; streaming to other devices; playlists and libraries shared between PCs;
macOS and Linux docking; owned games until entitlements exist (L7.1, M8).
