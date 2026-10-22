# ADR-109: Every launcher moves to Obsidian once

**Status:** **Accepted**, 21 October 2026, by the owner ("Everyone moves."), asked whether people who had already picked
a theme should keep it. Built the same day, in ADR-108's fourth slice (DIRECTION P2.9). It decides **no economic value**.

**Relates to:** [ADR-108](ADR-108-one-look-obsidian-and-gilt-in-two-registers.md), whose decision 6 made Obsidian the
launcher's default theme and kept Architect, Sanguine and Numen as choices. This decision says what happens to a
launcher that already had a theme stored. It changes nothing else in ADR-108.

## Context

ADR-108 makes Obsidian, built from `docs/design/tokens.json`, the launcher's default theme. A default reaches only the
launchers that have no theme stored. The launcher stores the theme a person last applied (`qor.theme` in its webview's
storage), and it also stored the old default, Architect, for everyone who never chose, since applying a theme stores it.
So a default alone would have changed almost no installed launcher: the testers would have kept Architect, and the one
look would have reached only new installs.

## Options

**A. Keep what is stored.** Only launchers with nothing stored open on Obsidian. *Costs:* nearly every installed launcher
keeps Architect, whether or not its person ever chose it; the one look does not reach the people already using it.
*Later:* moving them afterwards is this same change, made later.

**B. Everyone moves once (chosen).** The first launch of a launcher with this change stores Obsidian, whatever was
stored, and records that the move was made. A theme picked after that is kept, as before. *Costs:* a person who had
chosen Sanguine or Numen finds Obsidian once and picks their theme again in Settings, one click. *Later:* undoing it is
impossible to do exactly, since what was stored before is not kept; a later move to another theme would use a new
record.

**C. Everyone moves, and the other themes go.** *Costs:* removes a choice ADR-108 kept, which nobody asked to remove.

## Decision

1. **Every launcher moves to Obsidian once.** On a launch that finds no record of the move (`qor.theme.moved` is not
   `obsidian`), the launcher stores Obsidian as its theme and stores the record, then opens on Obsidian
   (`tools/qor-launcher/src/styles/themes.ts`, `storedTheme`).
2. **A theme picked after the move stays.** The record is never cleared, so the move is made once.
3. **Once per launcher, not per profile.** Profiles (ADR-099) share the launcher's webview storage, so the theme, and the
   move, belong to the launcher on a Windows account.
4. **The other themes stay choices,** as ADR-108 decision 6 says.

## Consequences

- Every installed launcher wears the one look after its next update.
- `tools/qor-launcher/scripts/check-look.mjs` holds it, in CI: a launcher that had Sanguine stored and no record opens
  on Obsidian and records the move; a theme stored after the move stays. Each was seen to fail against a planted fault:
  no move, and the move made at every launch.
