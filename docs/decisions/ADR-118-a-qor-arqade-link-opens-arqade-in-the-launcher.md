# ADR-118: A `qor://arqade` link opens ARQADE in the launcher, at a game if it names one

**Status:** **Accepted**, 25 October 2026, under the owner's delegation of engineering choices (the owner's words of 24
October 2026: "I authorize full automation and decisions permissable. You are approved to continue development ops
with best recommendations and highest quality when possible."), as the step after P7.23. The owner may amend it. It
decides **no economic value**.

**Relates to:** [ADR-076](ADR-076-arqade-hands-payments-to-the-launcher.md) and
[ADR-077](ADR-077-corrections-to-adr-076.md) (`qor://pay`, the first `qor://` link), and
[ADR-112](ADR-112-the-controllers-own-key-signs-every-agent-authorisation.md) (`qor://agent`).
[ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) decision 6: the website's own games point players to the launcher,
where they are rewarded.

## Context

ARQADE's website points players of its games to the launcher, where each game is rebuilt in 3D (P7.23). It can only
send them to the download: the launcher answers two links, `qor://pay` and `qor://agent`, and a person who already has
the launcher has to open it and find the game by hand. Worse, any other `qor://` link reaches the launcher's payment
path, which refuses it as a malformed payment request and says so in a dialog about paying.

A link that only opens a screen asks nothing of the person's key, account or session, and so needs none of the checks
`qor://pay` and `qor://agent` carry. But any web page can make one, so what it may do has to be written down.

## Decision

1. **`qor://arqade` opens the launcher at its ARQADE screen.** The launcher's window comes to the front and shows
   ARQADE. Before the launcher is unlocked, the screen is the one it opens at once it is.
2. **`qor://arqade?play=<game>` opens that game too,** where `<game>` is one ARQADE's app plays, by its card's name:
   `orbital`, `void`, `synapse`, `rift`, `flux` or `reversi`. The launcher tells ARQADE's app to open it, with a new
   command (`play`) on the channel the app already keeps (`arqade.next`). If the app is not running yet, the game is
   opened as soon as it starts. Any other value, or a link with anything else after `arqade`, opens only the screen:
   nothing the link carries is passed on except one of those six names.
3. **It does nothing else.** It pays nothing, signs nothing, signs nobody in or out, and reads nothing back to the page
   that made it. A game it opens is played as any game opened from its card; one already being played gives way to it,
   as when the person presses another card.
4. **The website uses it only once a launcher release carries it.** Before that, a launcher would refuse the link as a
   payment request, so ARQADE's website offers it beside the download, not in its place, and only from the first
   release that answers it.

## Consequences

- From a release on, the website can offer "Play in the QOR Launcher" for each of its games, and the arenas, and the
  launcher opens straight there.
- The launcher's ARQADE window, which handed every `qor://` link to the payment path, routes a `qor://arqade` link here
  as a browser does; every other `qor://` link goes where it went.
- Built in the launcher (`src-tauri/src/arqade.rs`, `arqade/app.rs`, `lib.rs`, the interface's `App.tsx`) and ARQADE's
  app (`qt/src/arqadelink.*`, `qt/qml/Main.qml`), each tested on its side.
- Not built here: the website's link, which waits for the release (decision 4).
