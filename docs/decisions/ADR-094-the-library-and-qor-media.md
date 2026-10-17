# ADR-094: The Library, and QOR Media in Qt docked inside the launcher

**Status:** **Accepted**, 16 October 2026, by the owner, who directed: "begin working on the Library. The goal is a
steamOS style in-launcher view. We also need a high quality premium feeling media player that supports all video and
image codecs and formats. Files should be directly openable from within the library. User should be able to browse
their own local file folders with ease and their content should be searchable and playable with zero friction. PDF files
and documents should be viewable and editable from within an editor which is high quality and premium. Consider QT based
options for all things from now on." Asked four questions, the owner chose the recommended option in each: **"Qt player
docked in launcher"**, **"PDF + text now, Office next"**, **"Everything, one place"** and **"Own index of chosen
folders"**. **Extends** L7.1 (the Library: install, delta patch and launch) and P5 (Market and Library). It decides no
economic value.

**Relates to:** [ADR-046](ADR-046-the-launcher-is-an-app-host.md) and [ADR-088](ADR-088-a-cartridge-holds-a-commit.md) (the
product host the launcher already runs QQ Studio under), [ADR-083](ADR-083-qq-on-qt.md) (Qt 6 under the owner's
commercial licence).

## Context

Checked on 16 October 2026: the launcher's Library is a placeholder page. The launcher is Tauri, its screens React in
WebView2, which plays H.264, VP8/9 and AV1 in MP4 and WebM but not most of what a person's own folders hold (MKV, AVI,
HEVC without a paid extension, FLAC in odd containers, ProRes, DTS). The owner's Qt 6.12 (`C:\Qt\6.12.0\msvc2022_64`)
holds Qt Multimedia with its FFmpeg backend and FFmpeg's libraries, Qt Quick, Quick Controls, Svg, Sql and image plugins
for JPEG, GIF, WebP, TIFF, SVG, ICO and TGA. It does not hold Qt PDF (an add-on its maintenance tool installs) or any
engine that edits Office documents. QQ Studio already runs as a Qt process the launcher starts and talks to over the
product host's private channel.

## Decision

1. **The Library lives in the launcher; files open in QOR Media, docked inside it.** The Library's view, its shelves,
   folders and search are the launcher's own screen, in a SteamOS-like layout: a hero for what was last opened, shelves
   by kind, large tiles with art, and keyboard and controller navigation. Opening a file shows **QOR Media**, a native
   Qt 6 application (`tools/qor-media/`, C++ and QML), as a borderless window placed exactly over the launcher's content
   area and kept there as the launcher moves and resizes, so it reads as part of the launcher. The launcher starts it
   under the product host (ADR-046's channel and one-time token) and tells it what to open and where to sit; it never
   holds a key or a token of the person's. `tools/qor-media/` joins active scope (AGENTS.md §4).
2. **QOR Media plays every format FFmpeg reads**, through Qt Multimedia's FFmpeg backend: video and audio of any common
   container and codec, with tracks, subtitles, chapters, speed and a scrubbing preview; and shows images in every
   format Qt's plugins read, with formats beyond them (HEIC, AVIF, JPEG XL, camera RAW) added as plugins later.
3. **Documents, release 1: PDF and text.** PDFs are viewed, annotated, highlighted, form-filled, signed, and their pages
   reordered, rotated and deleted, then saved (Qt PDF to show, PDFium's own interface to change). Plain text, Markdown
   and code are edited in QOR Media's own editor. **Release 2:** Word, Excel, PowerPoint and ODF, viewed and edited
   through LibreOffice's engine (LibreOfficeKit, MPL-2.0), designed in a record of its own first.
4. **One Library for everything a person has.** Games and work they own on the chain (DRC-369, installed and launched
   as L7.1 says, once entitlements exist) and their own files, side by side: Games, Videos, Music, Pictures,
   Documents and Folders.
5. **Search is the launcher's own index of the folders a person chooses**, on this PC only: Videos, Pictures, Music
   and Documents offered by default, others added. Kept in a local database with full-text search, it holds names,
   dates, sizes, and as they are read, durations, resolutions, photo details, music tags and document text, and follows
   changes as they happen. Nothing in it leaves the PC.
6. **Names** (plain, under ADR-032): the application QOR Media (`tools/qor-media/`); the launcher module `library` and its
   commands `library_*`.

## Consequences

- The installer carries Qt's runtime and FFmpeg for QOR Media, licensed under the owner's Qt commercial licence; its
  size grows accordingly (Qt Multimedia, Quick and FFmpeg are in the order of 100 MB).
- A docked native window over a web view is a Windows technique first (a child window of the launcher's); macOS and
  Linux need their own, later.
- Qt PDF must be added to the owner's Qt installation (the maintenance tool, under the owner's Qt account) before PDF
  viewing is built; PDFium's editing interface comes from its own binaries.
- Office documents wait for release 2.
- Nothing here touches CGT, the chain or DRC-369 state; owned work is only read.
