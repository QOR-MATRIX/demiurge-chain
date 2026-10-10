# ADR-086: Generation is paid for by the person who uses it; Demiurge pays for none

**Status:** **Accepted**, 10 October 2026, by the project owner: "We need to find a way to have the generative models be
free of extra charges, even if it means we just don't have that feature be available at first. Users will need to pay
for the generation they use."

## Context

QQ's design loop (ADR-085) calls a generative model, Claude, on the creator's own Anthropic key: the provider bills that
account for each run. Nothing in the active tree holds a provider key of the project's. But nothing yet said that this
is a rule, and the plans reach further: generative assets for QQ (DIRECTION P3.6), and in the frozen applications a
"Sophia" chat whose server held xAI, Anthropic and OpenAI keys of its own (`apps/hub`, `apps/guru`; the root
`.env.example` documented them), which would bill every user's conversation to whoever ran the server.

## Decision

1. **Demiurge pays for no one's generation.** No service, website, launcher, Studio or build holds a model provider's key
   or account of the project's, and none forwards a user's generation to such an account (no proxy, no shared key, no
   free allowance funded by the project). This covers text, code, images, models, textures, sound and any other output
   of a generative model, and the agents that call them.
2. **The person who uses generation pays for it.** Today that is through their own account with the provider: they give
   their own key, the provider bills them directly, and Demiurge neither sees the bill nor takes a share of it. QQ's
   Agent panel (ADR-085) and any MCP client a creator connects to QQ Studio already work this way.
3. **A generative feature is off until its user turns it on with their own account.** No key is built in, and none is
   taken from the environment; the feature says plainly who pays before a key is given, and shows what was used. A
   feature that could only work on an account of the project's (generation run on a server for players who gave no
   key, a Sophia-style chat paid by the server) is not available, and is not built in that form.
4. **Paying for generation through Demiurge is not built.** Whether Demiurge should ever sell generation itself (in CGT
   or otherwise), at what price, and which demand sink it would belong to (ADR-002, ADR-006) is open:
   `OPEN_QUESTIONS.md` U-19. Until it is decided, generation is bought from providers by the people who use it.

## Consequences

- **The owner is not asked to fund generation.** P3.3's proof, a scene built from a description with no human edit, is
  made by whoever runs it with their own key, billed to that key; it waits until someone chooses to.
- QQ's Agent panel says before any key is given that the agent runs on the creator's own Anthropic account, billed by
  Anthropic, with nothing paid or charged by Demiurge; after a run it shows the tokens used, as the provider counted
  them. A check proves a key in the environment is never used.
- The Sophia keys and Pinecone index are removed from the root `.env.example`. The frozen Sophia code stays frozen
  (AGENTS.md §4); if it is ever brought back, it is under this decision.
- DIRECTION P3.6 (generative assets) is built the same way: on the creator's own provider account.
- Nothing here touches CGT, wallet keys or the chain.
