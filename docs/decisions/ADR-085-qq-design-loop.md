# ADR-085: QQ's design loop — Claude through Anthropic's Messages API, the key in the keychain, the creator keeps the last word

**Status:** **Accepted**, 10 October 2026, under the delegation of ADR-081 (QQ's engineering and product choices are delegated to its builder without stopping for approval; choices are recorded as ADRs accepted under that delegation and reported to the owner). ADR-083 decision 4 said the provider would be chosen and recorded when the agent was built; this is that record.

## Context

DIRECTION P3.3 asks for a design loop in QQ Studio: a description turned into a design brief, built, played, judged by
its captured frames and revised, with the creator approving before anything is committed and the provider's key kept in
the operating system's keychain and never logged. The Studio's fourteen tools already exist and are offered to any MCP
client (HANDOFF §4 item 66). The loop needs a model that can plan, call those tools over many turns and judge images,
and a way to reach it from a C++ desktop program: Anthropic publishes no C++ SDK, so it is raw HTTPS.

## Decision

1. **The provider is Anthropic; the model is Claude Opus 5.5 (`claude-opus-5-5`)**, through the Messages API, with
   adaptive thinking and effort `high` set explicitly (that model's default is `medium`, and building and judging a game
   is long agentic work). Each answer may run to 16,000 tokens. The model is one constant in
   `products/qq/agent/designloop.h`; changing it is a new ADR.
2. **Refusals fall back server-side.** Every request opts in to `fallbacks: "default"` (beta
   `server-side-fallback-2026-07-01`), so a declined request is run again on the model Anthropic recommends for why it
   was declined. A refusal that remains ends the run and says so; what a fallback replaced (thinking and tool calls
   before the last `fallback` block) is neither run nor sent back.
3. **The loop uses the same tools as MCP**, called in the Studio without the protocol, so an agent connected from
   outside and the Studio's own loop do the same things the same way. A captured frame goes back to the model as an
   image. The conversation is append-only: each answer is kept as it came, and each turn's tool results go back together
   in one message. A turn cut off at its token limit is not run. Requests ask for automatic prompt caching, so the
   unchanged history is read back cheaply.
4. **The key lives in Windows Credential Manager** (a generic credential for the user, on this computer), typed once in
   the Agent panel and never shown again. It is read only to make a request, goes only into the `x-api-key` header, and
   is never written to a file, a setting or a log. `QQ_ANTHROPIC_URL` may point the loop elsewhere only over https or to
   this computer, so the key cannot be sent in the clear. Other systems have no keychain support yet, and the loop says
   so rather than storing the key anywhere else.
5. **The creator keeps the last word.** The loop cannot save or commit; what it builds shows as unsaved, every step
   (frames included) is shown in the Agent panel, and **Undo the run** puts the scene back as it was when the run
   started. Logic files it wrote stay in the project's `logic/` folder for the creator to keep or delete.
6. **Limits:** at most 60 answers in a run; rate limits and overload (408, 409, 429, 5xx) are waited out up to five
   attempts, as `retry-after` says or with growing pauses; a refused key, a bad request or a refusal ends the run.

## Consequences

- The provider's charges fall on the creator's own key: Claude Opus 5.5 is $4 per million input tokens and $20 per
  million output tokens, cached input $0.20; every captured frame is an image the model reads. The Agent panel shows
  every step, and the creator can stop at any time.
- The loop is checked against a stand-in for the API on the owner's machine (`tst_agent`): the requests' headers and
  bodies, the tools run, images returned, the history append-only, retries, a refused key, a refusal, an answer cut off,
  a fallback's replaced blocks, the key absent from every log and request body. The real endpoint was reached over TLS
  with an invalid key and refused it. **The proof P3.3 asks for, a scene built from a description with no human edit,
  needs the owner's key** and is not yet made.
- A stranger's logic still runs with the engine's full reach (P3.4 and P3.5 must decide how to confine it); the loop
  adds no new reach, since it writes logic only through the same tool a creator's agent uses.
- Nothing here touches CGT, wallet keys or the chain.
