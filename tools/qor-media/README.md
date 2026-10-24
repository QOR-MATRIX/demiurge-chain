# QOR Media

The Library's player, viewer, editor and reader ([ADR-094](../../docs/decisions/ADR-094-the-library-and-qor-media.md),
[`docs/blueprints/library.md`](../../docs/blueprints/library.md), L9.3 to L9.5). A Qt 6 program in C++20 and QML that the
QOR Launcher starts when a person opens a file in the Library. Its window becomes a child of the launcher's and sits
over the Library, so it moves, resizes and minimises with the launcher.

| What opens | Where | How |
| --- | --- | --- |
| Video and sound | `qml/Player.qml` | Qt Multimedia on its FFmpeg backend: every container and codec FFmpeg reads. Resumes where it stopped; scrubbing with the time under the pointer; ±10 s; speed; volume; audio and subtitle tracks; full screen |
| Pictures | `qml/Viewer.qml` | Every format Qt's image plugins read (JPEG, PNG, GIF, WebP, TIFF, SVG, BMP, ICO, TGA), turned as the camera wrote it; zoom about the pointer, drag, 1:1 on a double click, the folder's others next and previous, turn, slideshow |
| Text, Markdown and code | `qml/Editor.qml` | Read and saved as written (UTF-8 or UTF-16, byte-order mark and CRLF kept, `src/textfile.cpp`), coloured in the launcher's theme; line numbers for code; Markdown preview beside the words; find; asks before unsaved words are lost |
| PDF | `qml/Reader.qml` | Read with Qt PDF: the pages one under another on their own paper, drawn with their marks (highlights, notes, stamps) and their form's fields by `src/pdfdrawing.cpp`, since Qt PDF's page image leaves the marks out and nothing in Qt PDF draws a field: a PDF without a form by Qt PDF's own engine, one with a form by QOR Media's PDFium through that form; fitted to the width or the whole page, zoomed about the pointer; find with every match marked; text selected and copied; links followed to their page, or out to the web or mail and nowhere else; the pages and the contents beside it; a locked PDF opens with its password, kept only while it is open; a broken one says so. Its pages turned, deleted and moved, shown at once, undone and redone, and saved through PDFium (`src/pdfpages.cpp`) into a temporary file renamed into place; asks before unsaved changes are lost; a locked or protected PDF is read, never changed. **Marked up** (`qml/MarkupBar.qml`, `qml/MarkItem.qml`, `qml/SignaturePad.qml`, `src/pdfmarks.cpp`): words highlighted, underlined and struck through, notes, ink drawn by hand, and a picture of a signature, drawn or typed, which certifies nothing (ADR-114 decision 1); the PDF's own marks chosen and taken away, and a note's words read by a tap; all shown at once, undone and redone with the pages, and written in the same save. **Filled** (`qml/FieldItem.qml`, `src/pdfforms.cpp`): a form's fields tinted, and filled where they are, through PDFium's own form as a person fills one: words typed in place, boxes ticked, a group's button chosen, a list's options chosen from under it, Tab from field to field; a tap on a signature field places a picture of a signature in it; each drawn at once by PDFium as it will be saved, undone and redone with the rest, and written in the same save. Shown and not filled: a form that keeps its answers in XFA (the IRS's), a password field, a field read only; a form's scripts do not run |
| Office documents, anything else | `qml/Elsewhere.qml` | Says so, and opens it in the program Windows chooses. Office documents arrive with L9.7 |

Keys: Space or K play and pause; arrows seek 5 s (player) or move between pictures (viewer); J and L 10 s; Up and Down
the volume; M mutes; F full screen; [ and ] speed; C subtitles; A audio track; R turns a picture; S a slideshow; Ctrl+S
saves; Ctrl+F finds; Ctrl+P previews Markdown; Alt+Z wraps lines; Esc goes back to the Library. In a PDF: Page Up, Page
Down, Space, the arrows, Home and End go through it; Ctrl+G goes to a page; + and -, or Ctrl and the wheel, zoom; 0 fits
the width or the whole page; Ctrl+F and F3 find; Ctrl+A selects a page's text and Ctrl+C copies it; F9 shows the pages
and contents, whose entries the arrows choose and Return opens; R turns the page (Shift+R the other way), Delete deletes
it, Alt+Up and Alt+Down move it, Ctrl+Z and Ctrl+Y undo and redo, Ctrl+S saves. M shows the marks; H, U and X highlight,
underline and strike through the words selected; N writes a note (Ctrl+Return keeps it, Esc leaves it); D draws by hand;
a tap chooses a mark and Delete takes it away; the arrows move a chosen note or signature (Shift for ten points), and [
and ] size a signature. In a form, Tab from the pages goes into the first field on the page in view, and from field to
field after it (Shift+Tab back), on to the next pages'; Return keeps words typed (Ctrl+Return in a field of lines), Esc
leaves them as they were and Esc again lets go of the field; Space or Return ticks a box, chooses a button or opens a
list, whose options the arrows choose and Return takes. Tab reaches every button, the volume and the speed, shown by the
focus ring, and Return or Space presses it; while the keyboard is on the player's controls they stay on screen.

## Its look

Demiurge's one look in the tool register (ADR-108). `qml/Theme.qml` binds every value to `qml/Tokens.qml`, and
`src/tokens.h` gives the highlighter its colours, both generated by `docs/design/tokens.mjs` from
`docs/design/tokens.json`; the faces (Barlow, Barlow Semi Condensed and IBM Plex Mono) are in `fonts/`, copied there by the
same script and built into the executable (`src/look.cpp`). Docked in the launcher it wears the launcher's theme, sent
with every change (`theme`). Less motion comes from the launcher's Motion setting (`QOR_REDUCED_MOTION`) or, started by
hand, Windows' Animation effects: every transition is then immediate and the music's glow stays still. CI reads the QML
with `docs/design/check-qml.mjs`; `tests/tst_look.cpp` checks the same as painted.

Three things on a PDF's page are the page's, not the look's. The six colours a mark can be (a highlighter's yellow, green,
blue and pink, a pen's red and black) are what the PDF holds, the same in every theme, so they are kept in
`src/pdfmarks.cpp` and reach the QML from there (`PdfMarks.colours`), never written in it. A typed signature is drawn in
Great Vibes (SIL Open Font License 1.1, `fonts/GreatVibes-Regular.ttf` with `fonts/GreatVibes-OFL.txt`), as outlines made
in C++ (`PdfMarks.typed`); `docs/design/tokens.mjs` does not manage it, and no QML names it. And words typed in a form's
field are in the field's own ink, read from the PDF (`PdfForms.read`'s `colour`), on the paper's white; they are set in
the look's face and size, scaled with the page as everything on it is, and once kept PDFium draws them in the field's own
font. A field's tint, its focus ring and a list's options are the look's.

## How the launcher talks to it

Through the QOR product host (ADR-046): a private named pipe and a one-time token on stdin. QOR Media keeps one request
waiting, `media.next`, which the launcher answers with its next command (`open`, `place`, `hide`, `theme`) or with nothing
after its wait (25 s); it asks again at once. What QOR Media has to say (a file's length and picture size, where playback
stopped, that it closed) rides on its next `media.next`, so it reaches the Library within one wait. The launcher's side
is `tools/qor-launcher/src-tauri/src/media.rs`; this side is `src/medialink.cpp`. When the launcher goes, QOR Media goes.

Started by hand with a file (`qor-media.exe film.mkv`) it opens that file in a window of its own.

## Build, test, deploy

Needs the owner's Qt 6.12 (commercial licence) at `C:\Qt\6.12.0\msvc2022_64` with Qt Multimedia and Qt PDF, Visual Studio
2022's C++ tools, and CMake and Ninja from `C:\Qt\Tools`. CI has no licensed Qt, so these run on the owner's PC, as QQ's
do.

**PDFium**, for changing a PDF, is the prebuilt release `build.ps1` pins (`bblanchon/pdfium-binaries`, `chromium/8111`,
with its SHA-256). It is fetched once into `%LOCALAPPDATA%\qor-media-pdfium\`, checked against that hash at every build,
and refused if it differs. A deployed copy carries `pdfium.dll` with its licences (`pdfium-LICENSE.txt`,
`pdfium-licenses\`). A newer release is a change to those two lines, and its hash is the release's own.

```
pwsh tools/qor-media/build.ps1             # build and run the nine Qt Test suites
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
| `tst_files` | Kinds by extension, as the launcher's index shelves them; which documents the editor and the reader open; a folder's pictures in the order people read numbers |
| `tst_textfile` | LF, CRLF, UTF-8 with a byte-order mark and UTF-16 each written back as they were; a binary, oversized or missing file is refused and never written |
| `tst_playback` | The whole program: a generated tone plays and its length, where it stopped and that it closed are reported; Windows' own sample video decodes through FFmpeg; a picture shows with its size and the arrow key moves to the next; unsaved words hold the window and nothing is written; an Office document says what opens it; the launcher's colours are worn; it docks as a child of a window known only by its handle, at the launcher's physical rectangle in logical pixels, and lifts out to full screen and back; hide hides |
| `tst_reader` | PDFs the test writes byte by byte: one opens with every page, fitted to the width; the keys go through it, two jumps at once leave the view at the last, and a page's number typed goes there and gives the keyboard back; zoom stays between 10% and 800% and fitting returns; find marks every match and goes to each, around and back; a page's text is selected and copied, and another page's replaces it; a link goes to its page, and only the web or mail may leave; the pages and contents sit beside it and each goes to its page; a PDF locked with the standard handler (RC4) opens with its password and not without, the field emptied each time; a second locked PDF asks again; a broken one says so; pages turned, deleted, moved, undone, redone and saved, the file holding them exactly and the copy shown while saving gone; a PDF's own marks shown, and nothing left open for drawing once the reader goes; the last page stays; going with changes unsaved asks, and "Don't save" leaves the file as it was; a save that fails keeps the changes and the file; a locked or protected PDF is read, not changed, nor marked; marking up: the page's words highlighted from a selection, in yellow, shown before saving, then underlined in the green chosen, with nothing marked while no words are selected, undone and redone with a page's turn, and saved, the file holding both; a note written, left with Esc, saved, and read by a tap as the PDF's own, to read and not to change, put away by a tap beside it; ink drawn with the pointer, neither scrolling nor selecting, and saved as ink; a signature typed, placed saying it certifies nothing, moved by the arrows and by a drag (undone in one step), sized, and saved as a stamp; one drawn on the pad keeping its drawing's shape; a PDF's own mark chosen only while the marks show, taken away at once by Delete (not the page), drawn without it, back on undo, and gone from the file once saved; a form filled: words typed where the field is, held as unsaved before they are kept, kept by Return with the keyboard left on the field, the page then drawn filled; a box ticked and a group's button chosen, each drawn at once, the box keeping the keyboard without the focus ring after a tap; an option chosen from under a list by the keys; undone and redone, nothing reaching the file; a box ticked back as the file has it leaving nothing to save; a field read only saying so; saved, and read back from the file; Tab from the pages into the first field, Esc leaving the words typed and Esc again letting go, Tab keeping words and going on (the focus ring on), Space ticking, Tab on through the group and the list past the field read only, back to the pages after the last, Shift+Tab going back, and Ctrl+S keeping and saving words not yet kept; a locked PDF's form drawn, read and not filled; a form kept in XFA saying why as it opens and on a tap; a signature field taking a picture of a signature in its middle; words half typed kept, and asked about, before going back; Tab reaches every control |
| `tst_pdfforms` | A form's fields drawn as the reader draws its pages (words, a group's button chosen, a drop-down list's option, a field with no appearance of its own; nothing for a box not ticked or a hidden field), which Qt PDF draws not one of; its widgets read (each kind, where it is on the page as laid out, its value, state, options and limits, and why one is not filled: read only, a password, a button, a signature field), a page at a time, a hidden one left out; filled as a person fills a form and saved, read back and drawn (words, a tick, a group's third button, an option, two of a list's, lines, a field's limit, any script, a field given an appearance), what was not filled as it was; drawn filled before saving, with a mark being taken away too, writing nothing; fields, marks and a page's turn in one save, the fields filled where the reader read them though a note before them is taken away; fields on a turned page read where they show, and filled; a locked PDF's form read and drawn with its password only, and never filled; a form kept in XFA read, not filled; 29 entries it cannot fill (a page or place it lacks, a mark, no way or two ways of filling, the wrong value for the field, a field read only, a password, a button, a signature field, a hidden field, too many words, a line break on one line, too many or too few options, an option it lacks or twice, a good entry beside a bad one) leaving the file's bytes as they were |
| `tst_pdfmarks` | Through PDFium, read back and drawn as the reader draws them: words highlighted where Qt PDF's selection puts them, the words still reading through it, within the annotation's rectangle; underlined along their foot and struck through their middle; a note's words (any script, and a new line) where it was put, read back as a mark; ink keeping its strokes and its width, a single point a dot; a signature drawn and one typed in Great Vibes, each a stamp in its box; a mark on a turned page landing on its words; marks and page changes in one save, a mark on a deleted page going with it; a PDF's own marks read (links and pop-ups are not marks), a page at a time, and taken away, a note with its pop-up; a page drawn without the marks being taken away, writing nothing; a locked PDF's marks read with its password only; a list it cannot follow (22 kinds: a kind, page, colour or place it cannot use, too much, a link, a mark it lacks, a good mark beside a bad one) leaving the file's bytes as they were |
| `tst_pdfpages` | A page drawn with its marks, which Qt PDF's page image leaves out; its width as asked and never past the limit; a locked PDF drawn with its password only; a closed key drawing nothing, leaving nothing open and letting go of the file. Through PDFium, read back: turns, deletions and moves written; every page in its place; the order reversed; a turn added to the page's own; a list it cannot follow (nothing kept, a page twice, a page it lacks, a turn too far) leaving the file's bytes as they were; a locked PDF and one protected without a password left as they were; a file that cannot be replaced left as it was, with no stray file beside it; which PDFs can be changed; the copy for saving made and only such a copy deleted, a read-only one too |

Every rule above was checked against a fault planted in it: the save guard, the newest-position rule, the frame limit,
numeric order, the unsaved-words guard, the length report, the closed report, and docking (no parent, physical pixels
taken as logical at 125% scaling, not put back after full screen). In the reader: links allowed anywhere, the password
left in its field, the same wrong password not said to be wrong, zoom without bounds, the keyboard handed back to the
field, a new search not going to its match, and fitting without the margin, each caught. One was not: the reader's own
clearing of the last PDF's password, since Qt PDF clears it too on closing a file; the test holds that a second PDF asks
again whichever of them does it. In changing a PDF, each caught: a turn not written, a page left out kept, the order not
written, a protected PDF changed, a page listed twice, a read-only copy not deleted; a locked PDF's pages changing, undo
doing nothing, a failed save losing the changes, the copy left behind, the file swapped while pages still show, and a
jump landing at the view's next frame, after a later one. The reader's own question on Esc was not, since the window asks
before closing anything unsaved too. In drawing pages, each caught: pages drawn without their marks, no width limit, a
closed key left open, the reader keeping its drawing when it goes, saving while the pages' file is open for drawing, and
a locked PDF drawn without its password. In marking up, nineteen, each caught: a highlight drawn opaque over its words,
no quadrilaterals for the words, the page's turn left out of the mapping, a note without its words, ink without its list
of strokes, a note taken away leaving its pop-up, a link taken away as a mark, no limit on points, marks written after
the pages are turned, a signature drawn without its box, marks being taken away still drawn; in the reader, undo
forgetting the marks, saving without them, Delete taking the page though a mark is chosen, a mark taken away still
drawn, a locked PDF marked, a mark's drag left for the view to take, Esc keeping a note it should leave, and a typed
signature placed as nothing. Two of the first nineteen missed, and changed the code, not the tests: a second limit on
points doing the first's work (one now), and a release taken as a drag's last move, which Qt already gives (gone). In
filling forms, twenty-five, each caught: fields drawn without their form, a PDF with a form drawn by Qt PDF, fields
filled after the marks are taken away, a field read only, a button or a signature field filled, a password filled,
widgets read in the PDF's own space, hidden widgets read, a page drawn without what was filled, an XFA form filled, a box
left unticked (and again with nothing checking it after), a page's form not told the page is loaded; in the reader, undo
forgetting what was filled, Esc keeping words typed, saving without them, Tab in a field not going on, Tab from the pages
not going in, a locked PDF filled, the pages drawn without what was filled, a signature placed beside its field, a tap on
a field doing nothing, an XFA form saying nothing, a field filled back as the file has it still to save, the focus ring
after a tap, and words typed and not kept going without a question. A field's limit not checked first, a radio button
unchosen and two options for a drop-down list were not planted: the check after filling should refuse each anyway
(reasoned, not tried).

## Not yet

Filling a form that keeps its answers in XFA, as the IRS's do (taking its XFA away, as other tools do, needs a change to
the PDF's catalog that PDFium's interface does not make); a form's scripts (its totals, formats and checks), since this
PDFium has no JavaScript; a list of a form's fields to go through beside the pages; certified signatures (ADR-114
decision 2, proposed, not decided); a PDF that carries certified signatures is saved whole, which breaks them; changing
the words of a PDF's own note; words in a box, shapes and squiggles; a signature kept in the person's profile to place again
(ADR-114 allows it); dragging pages to move them; changing or marking a locked PDF; Office documents (L9.7); HEIC, AVIF, JPEG XL and camera RAW pictures (L9.8);
video thumbnails on the Library's tiles; a game controller inside QOR Media (the Library's own view takes one); docking
on macOS and Linux.
