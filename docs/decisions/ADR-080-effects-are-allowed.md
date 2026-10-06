# ADR-080: Effects are allowed; the rule is distinctiveness

**Status:** **Accepted**, 6 October 2026, by the project owner: "There should not be a bunch of rules stating you
cannot use drawing canvas, glowing, neon. These are all perfectly fine to use, I just didnt want things to turn out
like a cookie-cutter AI template that it usually does." It loosens a check that fails the build, which is the owner's
to decide (`docs/GATES.toml` change log). It answers ADR-078's open question: ring styles may glow. It supersedes the
effect rules of `docs/design/DESIGN_SYSTEM.md` §1, and with them the exceptions ADR-051 and the owner's ceremony decision
of 28 September 2026 carved out of them. It decides no economic value.

## Context

The design system, written from the owner's original direction, ruled glow, neon, gradients, shadows, canvas
backdrops, pointer-following light and looping animation out of every surface, and `check-design.mjs` enforced it.
Exceptions were made twice (QFX; the ceremony). On 6 October 2026, building avatars, a canvas used to show a GIF's first
frame failed the check, and the owner said what the rules had been for: not against effects, but against a generic,
template look.

## Decision

1. **The design system's position is distinctiveness**: every surface intentional and recognisably Demiurge's, never the
   default output of a template or an AI. Glow, neon, gradients, shadows, canvas and shader backdrops, pointer-reactive
   light and looping animation are allowed on every surface.
2. **What stays, because it protects people or keeps themes working:** reduce motion stills or skips every moving
   effect; text is readable over whatever is behind it, measured as painted; colours, sizes and letter-spacing come from
   the theme's tokens and scales.
3. **`check-design.mjs` keeps its three scale rules** (colour tokens, type scale, tracking scale) and drops the five
   effect rules. `check-contrast.mjs` and `check-readability.mjs` are unchanged.
4. **Ring styles may glow** (ADR-078 decision 4).

## Consequences

- The QFX and ceremony exceptions have nothing left to exempt; their obligations (readability as painted, reduced
  motion, a frame-time budget) stay.
- A surface that uses effects carries the reduce-motion and readability obligations itself; the browser checks still
  measure readability across themes.
