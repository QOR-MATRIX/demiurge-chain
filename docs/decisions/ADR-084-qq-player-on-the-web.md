# ADR-084: The QQ Player on the web — Qt's threaded WebAssembly build, cross-origin isolated, with sound unpanned

**Status:** **Accepted**, 10 October 2026, under the delegation of ADR-081 (QQ's engineering and product choices are delegated to its builder without stopping for approval; choices are recorded as ADRs accepted under that delegation and reported to the owner).

## Context

ADR-083 makes the QQ Player run natively and as WebAssembly in ARQADE's browser, and DIRECTION P3.2 asks for it to
play a scene in a browser with its size and first-frame time measured. Building it (HANDOFF §4 item 65) found three
facts about Qt 6.12 for WebAssembly that decide how:

- **Qt's single-threaded WebAssembly kit has no Qt Multimedia**, so Qt Spatial Audio, which a QQ `Sound` and `World`
  use, cannot be linked. Built with that kit the Player loads no game at all ("module QtQuick3D.SpatialAudio is not
  installed").
- **The multithreaded kit has Qt Multimedia, and threads.** A page gets the shared memory threads need only when it is
  cross-origin isolated: served with `Cross-Origin-Opener-Policy: same-origin` and
  `Cross-Origin-Embedder-Policy: require-corp`.
- **Qt Spatial Audio makes its sound on a thread of its own**, and in a browser each of Qt's threads is a Web Worker,
  where there is no Web Audio. The audio thread throws on creating its output (`QWasmAudioDevices::createAudioSink`),
  dies, and takes the application's console output with it.

## Decision

1. **The QQ Player's web build uses Qt's multithreaded WebAssembly kit** (`wasm_multithread`), with the Emscripten SDK
   that kit names (5.0.5 for Qt 6.12). `products/qq/build.ps1 -Web` builds it.
2. **Whatever serves the Player sends `Cross-Origin-Opener-Policy: same-origin` and
   `Cross-Origin-Embedder-Policy: require-corp`.** This binds ARQADE when it delivers games (P3.5): the route that
   serves the Player is cross-origin isolated, and anything that page loads from another origin must allow it
   (CORP or CORS).
3. **In a browser, a `Sound` is a plain sound effect on the page's main thread, not spatial audio.** Its volume follows
   the distance from the scene's player as Qt's logarithmic model does (full within a metre, halving with each
   doubling, none beyond its `reach`), and it is not panned; with no player in the scene it plays at its own volume.
   `World` opens no spatial listener in a browser. Natively, sound stays spatial. This is revisited when Qt's spatial
   audio can make its sound on the page's thread.
4. **The browser build is checked by playing it**, not only by building it: `products/qq/player/measure-web.mjs` serves
   it cross-origin isolated, plays it in a headless Chromium-family browser on the machine's faster GPU, reads the
   Player's measurement (first frame from the page starting to load, and the frame rate), captures the first frame,
   and fails unless the frame is the game and no error reached the page's console or any of its workers'.

## Consequences

- The Player in a browser is a 43.8 MB `.wasm` (44.1 MB served; 15.9 MB gzip, 11.7 MB brotli) and shows its first frame
  5.8 to 6.0 s after the page starts to load, cold, served from the same machine (10 October 2026, RTX 4060 laptop).
  ARQADE should serve it compressed and cacheable; size is a target for later work, not a gate yet.
- Spatial placement of sound is native-only for now. A game that depends on hearing where a sound is from plays
  differently in a browser; the README's Known limits says so.
- A threaded page cannot be embedded in, or embed, pages that are not cross-origin isolated themselves without their
  consent. ARQADE's game page is designed around that when P3.5 is built.
- Nothing here touches CGT, keys or the chain.
