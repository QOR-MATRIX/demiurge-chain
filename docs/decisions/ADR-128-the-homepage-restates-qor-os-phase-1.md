# ADR-128: The homepage restates QOR OS: its apps answer the controller and wear Lumen

**Status:** **Accepted**, 30 October 2026, by the owner, who answered "Proceed." to the assistant's recommendation to
restate the page's QOR OS facts together with QOR OS's next Phase 1 step, once it was done. The words are the
assistant's. It decides **no economic value**. **Supersedes [ADR-127](ADR-127-the-homepage-offers-qor-mobile-and-restates-qor-os.md)
decision 3** (what the page says about QOR OS), which it restates whole below, changed only where QOR OS changed.
ADR-127's other decisions stand.

## Context

ADR-127 decision 3 lists what the homepage may say about QOR OS, as its `main` stood at `dd169a7`, and says a change to
it is a later record. Since then QOR OS has done two of the three items the page lists as Phase 1's work left:

- **ARQADE and QOR Media answer the controller** (`8df6dbf`).
  - ARQADE has the pad to itself in Prism, as a game does. On its home the d-pad moves a visible focus between games,
    and A plays one.
  - QOR Media's controls are reached and gone along by the d-pad.
- **QQ, ARQADE and QOR Media wear Lumen** (`7f7f544`, QOR OS's QOR-0016). In QOR OS they take Lumen's colours, the
  person's accent with them, in place of Demiurge's one look. Outside QOR OS they keep it, so this repository's
  launcher, which ships their frozen copies, is unchanged.

These were checked on 30 October 2026 against QOR OS's `main` at `7f7f544`: its `ROADMAP.md`, `docs/CONTROLLERS.md`,
`docs/design/LUMEN.md` and QOR-0016.

## Decision

1. **What the page says about QOR OS is this, and only this** (supersedes ADR-127 decision 3):
   - QOR OS is an operating system for players, game makers and creators: play, make and publish games, apps, themes
     and media on one machine, with one account and one library, driven by a controller as well as a keyboard.
   - **Its parts:**
     - **Prism**, the shell, which is also the compositor, so windows and games are live parts of one scene, drawing
       nothing while nothing changes.
     - **Lumen**, its design language, which every app wears, QQ, ARQADE and QOR Media included, in the accent the
       person chooses.
     - **QQ**, the QOR Engine, with QQ Studio and the QQ Player.
     - **ARQADE** and **QOR Media**, running inside Prism.
     - **Its own controller service**, which reads the common console pads over USB and Bluetooth in one layout and
       drives the whole shell and its apps. A game in front, or ARQADE, has the pad to itself.
     - The **Store**, of signed packages (games, apps, themes and widgets).
     - **The QOR core**, this repository's code for keys, profiles, amounts and the chain, built into QOR OS at a pinned
       commit (ADR-120).
     - A **vault** that keeps keys in a service of its own, outside the shell and every app, whose approvals the
       compositor draws. The vault is designed and not yet built.
   - **Inside Prism**, as built, unchanged from ADR-127:
     - Beam, one search for apps, actions, settings and files, with a calculator; the Rail; the Shelf; Control Center;
     - windows for its own apps and other Linux apps built for Wayland, each opening at its own size;
     - three Spaces and Overview;
     - the controller interface, with a cursor, focus, an on-screen keyboard and the glyphs of the pad in use;
     - themes and widgets from the Store; several displays;
     - three-finger gestures on touchpads and touchscreens; notifications;
     - its own apps (Files, Music, Notes, Terminal, Monitor, Settings and the Store).
   - **Where it stands.**
     - **Phase 0, done:** Prism, Lumen and the shell running nested on Linux with real applications, a controller and
       sound; the Store's first stage; QQ, ARQADE and QOR Media built and tested on Linux.
     - **Phase 1, now. Done:**
       - QQ, ARQADE and QOR Media are developed inside QOR OS;
       - Prism starts all three, lays out and names their windows;
       - every window opens at the size its app asks for;
       - QQ plays with QOR OS's controllers;
       - ARQADE and QOR Media are moved through with a controller's focus;
       - Lumen's look is on every app.
     - **Left:** a Home and a Library made for the controller. **Its goal:** Orb Run, a QQ game, playing inside Prism
       on Linux with a controller.
     - **Then, in order:**
       - one sign-in, chat and publishing from QQ Studio to the devnet, with the vault;
       - booting on its own and playing the PC games people already own;
       - outside developers shipping QQ games to it with public tools.
   - **There is no image or download yet.** Its code is private while it is built. QOR OS 1.0 moves no money: free
     items and test CGT on the devnet only.
   - It uses Demiurge's network as services (ADR-120 decision 4): QOR ID, DRC-369 and CGT.

   Durations, dates beyond those above, hardware and the names of other companies' products stay off the page, as
   ADR-123 and ADR-127 had it, except the platforms a download needs (Windows, Android). A change to any of this is a
   later record.

## Consequences

- The page's Phase 1 checklist shows six done and one left.
- The page says Lumen is worn by every app, which is true in QOR OS only. The page speaks of QOR OS, and this
  repository's launcher keeps Demiurge's look, which ADR-108 governs.
- `site/README.md` and `llms.txt` point to this record for what the page may say about QOR OS.
