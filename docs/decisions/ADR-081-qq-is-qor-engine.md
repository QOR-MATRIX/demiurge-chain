# ADR-081: QQ is QOR Engine; built by any means necessary

**Status:** **Accepted**, 6 October 2026, by the project owner: "QQ becomes QOR Engine" and "Authorized to build by any
means necessary to produce the results outlined." It supersedes the foundation of `docs/blueprints/qor-engine.md` (a
custom Godot 4.6 build) and withdraws the owner's promise of 22 September 2026 that every QOR Engine project opens in
stock Godot (DIRECTION P3.6, `GATES.toml` criterion `qor-engine.opens-in-stock-godot`). It decides no economic value.

## Context

QOR Engine (P3) was planned as a custom Godot build tracking upstream, chosen because a renderer, editor and import
pipeline written from scratch were judged too costly (ADR-001's innovation budget). On 6 October 2026 the owner
commissioned **QQ**: an engine and editor for small-scale, high-quality, effects-led 2D and 3D games, integrated into
the QOR Launcher, signed in with QOR ID, driveable by an LLM through a plugin or agent interface, and with generative
AI that turns a creator's description into assets and worlds. Offered the choice between QQ replacing QOR Engine and QQ
standing beside it, the owner chose replacement, and authorised the work to be built by any means necessary.

## Decision

1. **QQ is QOR Engine.** Product track P3 is QQ's. The Godot foundation, its custom-build plan and the stock-Godot
   promise are withdrawn.
2. **QQ's target** is small-scale, high-quality, fun and interactive experiences built on visual effects and player
   engagement, not AAA production. One game runs in the QOR Launcher and in a browser on ARQADE.
3. **QQ is fully integrated:** a surface of the QOR Launcher; QOR ID sign-in (the launcher's session, or ADR-073's
   OAuth for the web); Qontrol versioning; publishing to ARQADE.
4. **QQ is autonomous-capable and generative:** an agent interface (for example an MCP server and a plugin API) through
   which an LLM can build with QQ, and generative AI for assets and worlds.
5. **Delegation.** The owner delegates QQ's engineering and product choices to whoever builds it, without stopping for
   approval: architecture, libraries and engines built on, names of QQ's own modules and packages, AI providers, and
   where published games are hosted. Such choices are recorded as ADRs accepted under this delegation and reported to
   the owner.
6. **What the delegation does not change:** `AGENTS.md`'s money rules (no path that creates, prices or charges CGT; no
   invented value from `OPEN_QUESTIONS.md`), the language rules, and the rules that protect people's keys and secrets
   (only the launcher's vault signs, behind its host dialog; nothing secret reaches a log). The owner can change those
   only by saying so.

## Consequences

- `docs/blueprints/qor-engine.md` describes the superseded Godot plan; QQ's blueprint replaces it.
- DIRECTION's P3 items, which describe the Godot build, are rewritten as QQ's items when QQ's work starts; until then
  they stand, marked superseded.
- `GATES.toml` drops `qor-engine.opens-in-stock-godot`, a loosening the owner made by withdrawing the promise it
  enforced, logged in its change log. The `qor-engine` gate is rewritten for QQ, as a tightening, when its items are.
- The innovation-budget argument of ADR-001 is set aside for QQ by the owner's choice: QQ is a place Demiurge spends
  innovation, and its risk is a large, long build.
