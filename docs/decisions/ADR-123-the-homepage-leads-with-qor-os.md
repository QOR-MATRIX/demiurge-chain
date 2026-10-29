# ADR-123: The homepage leads with QOR OS

**Status:** **Accepted**, 29 October 2026, by the owner. The owner asked the assistant to "update the entire website to
reflect the new QOR OS branding and info", then chose, from the assistant's questions, to make QOR OS the headline with
the network beneath it ("Rebrand around QOR OS") and to publish it at once. The structure, the words and the look below
are the assistant's, under that instruction. It decides **no economic value**. **Amends [ADR-100](ADR-100-the-homepage-at-qorsync-dev.md)**,
decisions 3, 4, 5 and 7, as each point below says. [ADR-108](ADR-108-one-look-obsidian-and-gilt-in-two-registers.md)'s
one look is unchanged.

## Context

`https://qorsync.dev/` is the project's one homepage (ADR-100). Until now it said what Demiurge is: the chain, its four
primitives, what is live and the products. Since 26 October 2026 the owner's operating system, **QOR OS**, carries
QQ, QOR Media and ARQADE's Qt app and builds the launcher's core from this repository (ADR-120). It is what the owner
now presents first.

ADR-100 decision 4 says the page's words come from current documents only. QOR OS's documents are in its own
repository, `QOR-MATRIX/qor-os`, which is private, so no public document said what the page may say about it. This
record is that document: the facts in decision 2 were checked against QOR OS's tree on 29 October 2026 (its
`main` at `86dad74`: `VISION.md`, `docs/ROADMAP.md` and its decision records) and are written here so a reader of this
repository can check the page against them.

## Decision

1. **The page leads with QOR OS.** Its title, description, share cards, hero, web manifest and `llms.txt` name QOR OS
   first. Demiurge's network follows as what runs today and what QOR OS is built on: the live services, the four
   primitives, the principles and the network's source. The address stays `https://qorsync.dev/`, and the domains
   ADR-100 decision 1 lists still redirect to it.
2. **What the page says about QOR OS is this, and only this** (amends ADR-100 decision 4, which this record joins as a
   source):
   - QOR OS is an operating system for players, game makers and creators: play, make and publish games, apps,
     themes and media on one machine, with one account and one library, driven by a controller as well as a keyboard.
   - Its parts: **Prism**, the shell, which is also the compositor, so windows and games are live parts of one scene;
     **Lumen**, its design language; **QQ**, the QOR Engine, with QQ Studio and the QQ Player; **ARQADE** and **QOR
     Media**, built in; the **Store**, of signed packages; and a **vault** that keeps keys in a service of its own,
     outside the shell and every app, whose approvals the compositor draws.
   - Where it stands. **Built:** Prism, Lumen and the shell, running nested on Linux with real applications, a
     controller and sound; the Store's first stage; QQ, ARQADE and QOR Media built and tested on Linux. **Now:** those
     three running inside Prism, driven by a controller (QOR OS's Phase 1). **Then**, in order: one sign-in, chat and
     publishing from QQ Studio to the devnet; booting on its own and playing the PC games people already own; outside
     developers shipping QQ games to it with public tools. The vault is designed and not yet built.
   - **There is no image or download yet.** Its code is private while it is built. QOR OS 1.0 moves no money: free
     items and test CGT on the devnet only.
   - It uses Demiurge's network as services (ADR-120 decision 4): QOR ID, DRC-369 and CGT.

   Durations, dates beyond those above, hardware and the names of other companies' products stay off the page. A
   change to any of this is a later record.
3. **Structured data** (amends ADR-100 decision 3): the `Organization` is `QOR-MATRIX`, the GitHub organisation that
   builds both, with `Demiurge` as an alternate name; the `WebSite` is `QOR OS`; a `SoftwareApplication` describes QOR
   OS, with no download address and no offer; the source, the launcher and ARQADE keep their nodes.
4. **The look** (amends ADR-100 decision 5, as ADR-108 amended it): ADR-108's showcase register, every value and face
   from `tokens.css`, unchanged. What changes:
   - **The mark is QOR OS's**: Lumen's `QorMark`, a rounded tile holding a ring with an offset tail, a stylised Q that
     also reads as a lens, drawn here in gilt with the ring engraved in obsidian and the tail in ember. It replaces
     Demiurge's sigil in the bar, the footer, the favicon, the icons and the share image. Cinzel sets the wordmark.
   - **The living canvas keeps its lattice, its embers and its rules** (reduced motion draws one still frame; a hidden
     tab draws nothing; the canvas is decoration and every word is HTML). The astrolabe becomes **the lens**: rings and
     ticks with the mark's tail turning on the ring, and **an aurora**, Lumen's signature, as slow ribbons of ember and
     gilt across the top of the page.
5. **`site/scripts/check-site.mjs`** (amends ADR-100 decision 7) holds the page to this record as well: the title
   names QOR OS, the structured data describes QOR OS with nothing to download, and the page says there is no image or
   download yet. Its other checks stay.

## Consequences

- Search engines and AI crawlers describe the homepage as QOR OS's, with Demiurge's network under it.
- The repository's own page (ADR-100 decision 6) stays Demiurge's: this repository is the network, and QOR OS's code is
  elsewhere.
- When QOR OS's facts change, the page changes only through a later record that restates decision 2.
