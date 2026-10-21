# ADR-108: One look across every surface: obsidian and gilt, in two registers

**Status:** **Accepted**, 21 October 2026, by the owner, who chose the recommendation ("Obsidian & gilt, two registers
(Recommended)") after a survey of every surface. **Amends** [ADR-100](ADR-100-the-homepage-at-qorsync-dev.md)
decision 5: the homepage's look becomes every surface's. **Changes** `docs/design/DESIGN_SYSTEM.md`: the launcher stops
being the source of the values, which move to `docs/design/tokens.json`. It decides **no economic value**.

**Relates to:** [ADR-080](ADR-080-effects-are-allowed.md) (effects are allowed, and what they owe people),
[ADR-051](ADR-051-the-shells-rendering-layer-for-qfx.md) (QFX's canvas and its contrast contract), and the Qt surfaces of
[ADR-083](ADR-083-qq-on-qt.md), [ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md),
[ADR-103](ADR-103-the-launcher-moves-to-qt-in-stages.md) and [ADR-104](ADR-104-the-launcher-on-android-on-qt.md).

## Context

On 21 October 2026 the owner asked for "a consistant premium look and feel across the entire ecosystem, with just
enough unique edge to ensure maximum gamer and developer appeal", "a futuristic sleekness which feels god-tier all the
way around".

A survey that day read every surface's colours, faces, shapes and motion, and screenshots were taken of eight of them:
the homepage, ARQADE on the web and in Qt, QOR ID's pages, the launcher, QQ Studio and the QQ Player. It found **three
visual languages and no shared source**:

- **Obsidian and gilt**: the homepage (ADR-100 decision 5), ARQADE's Qt application and QQ's game screens. Cinzel
  capitals in gilt over obsidian, sacred-geometry line work, ember for action. The most distinctive, and in the
  screenshots the one that reads as premium.
- **"Architect"**: the launcher. Carbon grounds, white ink, Barlow Semi Condensed in capitals, 8 and 12 pixel corners.
  Polished, but its own identity, with no gilt.
- **Stock**: ARQADE on the web (a shadcn template's coral orange, Arial and Segoe, monospace labels), QQ Studio
  (Qt's Material Dark, with a green Play button nothing else uses), QOR ID's pages (Segoe, 2 pixel corners) and parts
  of QOR Media (Qt's Basic buttons).

Every Qt application, and QOR ID, copied values by hand, and the copies had drifted: seven near-black backgrounds,
five display faces, four monospace faces, corners from 2 pixels to pills, six button styles, four reds for an error.
`DESIGN_SYSTEM.md` said every surface took the launcher's tokens, and none did. The homepage loaded its faces from
Google's servers. On the way the survey found defects any direction would fix: the faintest ink of ARQADE's Qt
application and QOR Media (`#6b7385`) read 3.4 to 3.8:1, under the 4.5:1 floor; QOR Media had no reduced motion and
buttons a keyboard could not reach; QQ's screen buttons showed no focus; ARQADE's web welcome dialog clipped its last
paragraph behind its button.

## Options

**A. Obsidian and gilt everywhere, in two registers (recommended).** The homepage's identity becomes the one look,
with a register for showing (full ornament) and one for working (dense, ornament as hairlines). Every value from one
source. *Costs:* the launcher, the surface used most, changes its default theme; every other surface is restyled.

**B. Architect everywhere.** The launcher's look becomes the one look. *Costs:* the homepage, ARQADE and the games
lose the gilt and Cinzel that make them distinctive; the result is cleaner but more like other products.

**C. Two languages on shared foundations.** Obsidian and gilt for showing, Architect for tools, one source for grounds,
inks, ember, type, shape and motion. *Costs:* less unified; two identities to keep apart.

## Decision

**Option A**, chosen by the owner on 21 October 2026.

1. **One look.** Obsidian grounds, warm parchment inks, ember the one colour of action, gilt the metal of the brand: its
   engraving, rules and numerals. The values are `docs/design/tokens.json`; how they are used is `DESIGN_SYSTEM.md`.
2. **Two registers, one set of values.** A register decides how much ornament a surface carries, never a value.
   - **Showcase**: the homepage, ARQADE, a game's own screens, ceremonies. Cinzel for wordmarks, heroes and titles;
     gilt engraving and geometry; the living canvas behind.
   - **Tool**: the launcher, QQ Studio, QOR Media, QOR ID's pages, the Android shell. Barlow Semi Condensed for headings
     and controls, Barlow for reading, gilt only as hairlines and numerals, denser spacing. Cinzel only in a wordmark.
3. **One source.** `docs/design/tokens.mjs` writes each surface's own generated file from `tokens.json` and copies the
   faces it bundles. No surface writes a colour, a face, a radius or a duration of its own. Run with `--check` in CI, it
   fails when a generated file is stale, a bundled face differs from the source, or a text colour reads under 4.5:1 on
   any background, or on any filled control.
4. **Faces, bundled and served by each surface itself, never from a font service:** Cinzel (display), Barlow Semi
   Condensed (interface), Barlow (reading) and IBM Plex Mono (numbers), all under the SIL Open Font License, their
   files in `docs/design/fonts/`.
5. **Shape and motion are shared:** 8 pixel corners on controls and 12 on panels, round only for status pills, dots
   and avatars; one easing, `cubic-bezier(0.16, 1, 0.3, 1)`, at 150, 260, 420 and 760 milliseconds; one focus ring, 2
   pixels of bright ember with a 5 pixel halo. Reduced motion stills every moving thing on every surface, and the
   launcher passes its Motion setting to everything docked in it.
6. **The launcher's default theme becomes Obsidian,** made from the tokens. Architect, Sanguine and Numen stay as
   choices; Veridian and Abyss still move to QFX's themes (the owner, 22 September 2026).
7. **The surfaces join in this order** (DIRECTION P2.9), each in the change that makes it read its target, each gaining
   the checks it lacks: the source and the homepage; QQ (the Studio and the game screens); ARQADE (the web, then Qt);
   the launcher's default theme; QOR ID's pages and emails; QOR Media and the Android shell.

## Consequences

- The homepage's colours do not change, since the tokens were drawn from it. Its corners move to the shared scale,
  and its faces are served from its own folder, so its content security policy no longer allows Google's font servers.
- A surface keeps its hand-written values until its slice; `DESIGN_SYSTEM.md` lists which have joined.
- The launcher's checks (`check-design.mjs`, `check-contrast.mjs`, `check-readability.mjs`, `check-accessibility.mjs`)
  keep governing the launcher; the source's check governs the source and every file written from it.
- A later change of look is a new decision that supersedes this one.
