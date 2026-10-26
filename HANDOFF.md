# Handoff

**Current state, 25 October 2026.** The code is the public repository `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064).
`main` is at `bf41489`, the merge of #104 (records), after #95 to #103, all merged by the owner on 25 and 26 October
2026 (§4 items 151 to 158); CI passed every job on it. Pull requests #1 to #104 are merged. **P7.20 is done: all four of ARQADE's solo games (Orbital, Void Runner, Synapse and
Rift Survivor) are rebuilt in QQ and play in ARQADE's Qt app, each sound heard from where it happens, deployed on the
owner's PC from `main`; their rules replay a run in QQ and in Node (§4 items 142 to 148).** **P7.24 is done: the two
arenas, Flux Four and Rift Reversi, rebuilt in QQ and played in ARQADE's app on ARQADE's server, through QQ's host
channel (ADR-117), deployed on the owner's PC from `main` (§4 items 149 and 151); and P7.23, the website's games
pointing to the launcher, is live (item 150).** **ADR-116, how a run is confirmed
(P7.22), waits on the owner's decision.** **QOR ID runs ADR-112: an agent is authorised only by the
controller's own key.** **Launcher 0.1.17 is published** (`launcher-v0.1.17`, 25
October 2026): ARQADE's six games in 3D, `qor://arqade`, the level themes, assets' levels and QOR Media's PDFs; the
homepage offers it since #102 merged, and ARQADE's website offers each game in it since #103. 0.1.16, the first release the update channel offered, and 0.1.15, the first that
updates itself, before it. **L6.2 is ticked** (26 October 2026): the owner's launcher took 0.1.17 through *Update and
restart*. **P7.16 is done** (#101). **M4.3's rental is built on `session/asset-rental` (#106, ADR-119, `spec_version` 11), not merged** (§4 item 159). **The devnet runs runtime 11 since the owner's upgrade
of 26 October 2026** (rental, ADR-119; §4 item 160), and ran 10 from 23 October: an asset's level, XP and stats are live. L9.5 is built and merged but certified signatures
(ADR-114 decision 2, proposed). Work goes on a `session/*` branch and reaches `main` by pull request. `origin` is the private archive, not
the current tree. Live, devnet-only and with test CGT: Demiurge Devnet (`wss://rpc.qorsync.dev`, `spec_version` 10),
QOR ID (`https://id.qorsync.dev`), Social, ARQADE (`https://qor-arqade-tau.vercel.app`) and the homepage
(`https://qorsync.dev`). §1 has the detail.

**Newest: §4 item 160 (26 October 2026): #105 merged; the devnet upgraded to runtime 11 by the owner, read back, rental live on it; #106 still open, so `main` trails the devnet until it merges.** Before it: **§4 item 159 (26 October 2026): M4.3's rental built, as the owner chose: ADR-119, renting an asset for a number of blocks paid in CGT with the creator's royalty, held in place while rented; `spec_version` 11, 213 workspace tests with the wasm built; #106, after #105.** Before it: **§4 item 158 (26 October 2026): L6.2 ticked, the owner's launcher having taken 0.1.17 through *Update and restart*; the owner chose M4.3's rental as the next work.** Before it: **§4 item 157 (25 October 2026): #102 and #103 merged, CI green on `main`; the homepage offers 0.1.17 and ARQADE's website each game in the launcher; how one person plays an arena with two QOR IDs.** Before it: **§4 item 156 (25 October 2026): ARQADE's website offers each game in the QOR Launcher beside the download (ADR-118 decision 4), as #103, after #102.** Before it: **§4 item 155 (25 October 2026): #101 merged, ticking P7.16; launcher 0.1.17 published from `5db8349`, after the release build found a rematch lost to a late poll, fixed first, and the window suites were made to take the keyboard back; #102 moves the homepage's download to it.** Before it: **§4 item 154 (25 October 2026): P7.16's last piece built: Dusk and Nocturne, the themes a level unlocks, in the launcher's Settings, chosen only once QOR ID lists them; on `session/level-themes`, opened against `main`.** Before it: **§4 item 153 (25 October 2026): #99 merged, CI green on `main`, ARQADE deployed from `main` with the launcher's `play` command after its suites passed whole on an idle PC.** Before it: **§4 item 152 (25 October 2026): ADR-118 accepted under the owner's delegation and built: a `qor://arqade` link opens the launcher at ARQADE, at a game if it names one; on `session/qor-arqade-link`, opened against `main`, in no release yet.** Before it: **§4 item 151 (25 October 2026): #95 to #97 merged, CI green on `main`, ARQADE deployed from `main` with the arenas and started by the launcher's test; the website's pointer live; P7.23 and P7.24 ticked.** Before it: **§4 item 150 (24 October 2026): P7.23's last part built: the website's games point players to the QOR Launcher, where they are rebuilt in 3D; on `session/arqade-launcher-pointer`, opened against `main` after #95 and #96.** Before it: **§4 item 149 (24 October 2026): P7.24 built: Flux Four and Rift Reversi rebuilt in QQ and played in ARQADE's Qt app against another person on ARQADE's server, through QQ's host channel (ADR-117, accepted under the owner's delegation); on `session/qq-arenas`, opened against `main` after #95.** Before it: **§4 item 148 (24 October 2026): #93 and #94 merged, CI green on `main`, ARQADE deployed again from `main` with its sounds placed and started by the launcher's test; no pull request open.** Before it: **§4 item 147 (24 October 2026): ADR-116 proposed, how ARQADE confirms a run by replay (P7.22), for the owner to decide.** Before it: **§4 item 146 (24 October 2026): #88 to #92 merged, CI green on `main`, ARQADE deployed from `main` and started by the launcher's own test; P7.20's "spatial sound" found unmet and built (each sound that happens somewhere heard from there, placed sounds in mono), and P7.20 ticked, in #93.** Before it: **§4 item 145 (24 October 2026): Rift Survivor rebuilt in QQ and played in ARQADE, the last of P7.20's four games, on `session/arqade-rift-survivor` on #91's head: an arena for ninety seconds, its rules the website's step by step with exact square roots; 29 tests in the games suite, 88 in Node.** Before it: **§4 item 144 (24 October 2026): Synapse rebuilt in QQ and played in ARQADE, on `session/arqade-synapse` on #90's head: four nodes light in a growing signal the player repeats, its rules whole numbers counted in steps; three planted faults caught.** Before it: **§4 item 143 (24 October 2026): Void Runner rebuilt in QQ and played in ARQADE, on `session/arqade-void-runner` on #89's head: three lanes into the collapse, its rules one plain module replayed in QQ and Node; each game's Node test now pins the rules' whole state, since a tiny rule change left a run's outcome unmoved; the owner authorised full automation, and merges wait on the owner because Claude Code's safety check refuses them.** Before it: **§4 item 142 (24 October 2026): P7.20's first game, Orbital, rebuilt in QQ and played in ARQADE's Qt app, on `session/arqade-orbital` on #88's head: its rules one plain module that QQ's logic and Node both run, a run QQ recorded replayed by both to the same outcome, the game played in a test by mouse, keyboard and pad; ARQADE builds QQ's runtime in and keeps each play as a run; two planted faults caught.** Before it: **§4 item 141 (24 October 2026): #86 and #87 merged by the owner; CI's security job red on `main` from a new advisory (RUSTSEC-2026-0387, `crypto-bigint` 0.5.5 through the pinned `sp-core`'s `k256`), which no update fixes and no code of ours reaches; ADR-115, the owner's guarded ignore, with a CI step proving its conditions, pushed as #88. Then P7.20, ARQADE's games in QQ, begins with Orbital.** Before it: **§4 item 140 (24 October 2026): L9.5's last built part on `session/pdf-forms`, kept back until #86 is merged: a PDF's form drawn, which no page in QOR Media had been (Qt PDF never draws a field, and item 137's word that it did was wrong), and filled where it is, through PDFium's own form; a form kept in XFA (the IRS's) and a password field shown, not filled.** Before it: **§4 item 139 (23 October 2026): #85 merged by the owner; `session/pdf-markup` rebased onto `main`, its tree unchanged, built and tested again, and opened as #86; then filling forms.** Before it: **§4 item 138 (23 October 2026): L9.5's third part built on `session/pdf-markup`, kept back until #85 is merged: a PDF marked up in QOR Media (words highlighted, underlined and struck through, notes, ink, and a picture of a signature, drawn or typed, as ADR-114 decision 1 has it), the PDF's own marks read and taken away, all written through PDFium in one save; nineteen planted faults caught.** Before it: **§4 item 137 (23 October 2026): #84 merged; QOR Media showed PDFs without their own marks (highlights, notes, filled fields), found and fixed by drawing pages itself; ADR-114, the owner's "Both, drawn first": a drawn signature now, certified signatures proposed; pushed as #85.** Before it: **§4 item 136 (23 October 2026): #83 and #82 merged; the owner upgraded Demiurge Devnet to `spec_version` 10, read back, so progress records are live; L9.5's second part pushed as #84.** Before it: **§4 item 135 (23 October 2026): L9.5's second part built on `session/pdf-pages`, kept back until #83 is merged: a PDF's pages turned, deleted, moved, undone and saved through PDFium, pinned by its SHA-256; eleven of twelve planted faults caught.** Before it: **§4 item 134 (23 October 2026): L9.5's first part built and pushed as #83: QOR Media reads PDFs with Qt PDF (find, selection, links, contents, locked PDFs); 51 tests in six suites, seven of eight planted faults caught.** Before it: **§4 item 133 (23 October 2026): runtime 10 rehearsed against the devnet's own live runtime 9 and passed, 35 checks; its file in the owner's Downloads, to sign once #82 is merged.** Before it: **§4 item 132 (23 October 2026): #81 merged; ADR-113 decided, the owner's approval of a design left to the assistant, and its first phase built: an asset's level, XP and stats kept by its home game, at `spec_version` 10, shown on the launcher's cards; M4.2 and M4.5 ticked; the devnet's upgrade waits on the owner's signature.** Before it: **§4 item 131 (23 October 2026): #80 merged, the homepage live on 0.1.16; a test agent (`examples/agent_link.rs`), so a person can authorise one and tick L5.1.** Before it: **§4 item 130 (23 October 2026): #78 and #79 merged; QOR ID redeployed with ADR-112, so an agent is authorised only by the controller's own key; launcher 0.1.16 published, the first release offered through the update channel.** Before it: **§4 item 129 (22 October 2026): L5.1 built: the controller's own key signs every agent authorisation (ADR-112); QOR ID and the launcher, proven end to end on this PC; kept back until #78 is merged.** Before it: **§4 item 128 (22 October 2026): #77 merged; launcher 0.1.15 published, the first that updates itself, its installer downloaded back and verified; QQ Studio redeployed on the owner's PC; the Mesh dialog's broken sentences fixed and every tested prompt held to it; DIRECTION §6 a pointer; pull request #78.** Before it: **§4 item 127 (22 October 2026): #76 merged; at the owner's word, every record checked against the tree, CI and the live services, and what was wrong corrected; the browser checks' recurring failure in CI made to say why; with item 126, pull request #77.** Before it: **§4 item 126 (22 October 2026): L6.2 built: the launcher's update channel (ADR-111, under the owner's delegation), a signed manifest on GitHub checked by the host, every installer verified twice, installed only when the person says; proven end to end on this PC with a test product.** Before it: **§4 item 125 (22 October 2026): ADR-110 proposed at the owner's request: emulated play in ARQADE, on a libretro host of its own, licensed and homebrew games first, the economy only on games their makers put there. Not a decision: deferred by the owner the same day, kept for future reference.** Before it: **§4 item 124 (22 October 2026): #75 merged; CI green on the merge; nothing open; what follows ADR-108 is the owner's choice.** Before it: **§4 item 123 (22 October 2026): #74 merged and ADR-108 complete; QOR ID live in the look, checked from outside; QOR Media deployed on the owner's PC; SYSTEMS' table mended.** Before it: **§4 item 122 (22 October 2026): ADR-108's last slice, QOR Media and the Android shell, built and pushed as #74: every value from the tokens, the faces their own, QOR Media's three defects from the survey fixed (faint ink, no reduced motion, buttons a keyboard could not reach), and a QML check in CI.** Before it: **§4 item 121 (22 October 2026): #73 merged; CI green on the merge; QOR ID not redeployed, Railway having started no deploy.** Before it: **§4 item 120 (22 October 2026): #72 merged, the launcher in Obsidian on `main`; ADR-108's fifth slice, QOR ID's pages and emails, pushed as #73.** Before it: **§4 item 119 (22 October 2026): ADR-108's fifth slice, QOR ID's pages and emails, built on `session/qor-id-one-look` and committed there, kept back until #72 is merged: every value from the tokens, the four faces embedded and served by QOR ID itself, the one focus ring on every field; 185 tests, eight planted faults caught, and the pages and a message looked at in Chrome.** Before it: **§4 item 118 (22 October 2026): #71 merged, ARQADE's website live in the one look and its welcome filling the window, checked from outside; the launcher's slice pushed as #72.** Before it: **§4 item 117 (21 October 2026): ADR-108's fourth slice, the launcher, built on `session/launcher-one-look`: Obsidian its default theme, every launcher moved to it once (ADR-109, the owner's: "Everyone moves."), the four faces its own, and the person's Motion setting passed to what it starts; #71 pushed.** Before it: **§4 item 116 (21 October 2026): #70 merged, QQ in the one look on `main`; ADR-108's third slice, ARQADE, pushed as #71.** Before it: **§4 item 115 (21 October 2026): ADR-108's third slice, ARQADE, the web and then its Qt application, built on `session/arqade-one-look`: the showcase register, every colour, face, corner and timing of the interface a token, the welcome filling the window again, and a design check in CI that reads both; #70's CI green.** Before it: **§4 item 114 (21 October 2026): #69 merged, ADR-108's first slice live (the homepage in its own faces, checked from outside); its second slice, QQ, pushed as #70.** Before it: **§4 item 113 (21 October 2026): ADR-108's second slice, QQ, built on `session/qq-one-look`: QQ Studio in the tool register, every value from the tokens, and a game's screen buttons answering the keyboard.** Before it: **§4 item 112 (21 October 2026): #68 merged, P3.16 done and with it every step of ADR-105; ADR-108's first slice pushed as #69.** It began: ADR-108, one look across every surface, the owner's, and its first slice built on `session/one-look`: one source for every colour, face, radius and duration, checked in CI, and the homepage on it with its own faces.** Before it: **§4 item 111 (20 and 21 October 2026): #67 merged and P3.15 done; P3.16, saving, pushed as #68; and the owner's choice of one look for every surface, obsidian and gilt in two registers (ADR-108, next).** It began: P3.16, saving, built and committed on `session/qq-saving`, on P3.15's third part: one small store per game, kept from one play to the next, the one the lockdown allows, which a replay neither reads nor writes; every step of ADR-105 built; and #65 green on its new head (run 39508315025).** Before it: **§4 item 110 (20 and 21 October 2026): #66 merged, CI green on its head and on the merge; P3.15's third part, the Studio's tools, pushed as #67.** It began: P3.15's third part built and committed on `session/qq-studio-tools`, on the second; P3.15 built whole, its parts to follow one at a time. Before it: **§4 item 109 (20 and 21 October 2026): #65 merged, CI green on its head and on the merge; P3.15's second part, groups and prefabs, pushed as #66.** It began: P3.15's second part built and committed on `session/qq-hierarchy`, on #65's head, to follow it. Before it: **§4 item 108 (20 October 2026): #64 merged, CI green on it and on the merge; P3.15's first part, editing (undo and redo of every edit, duplicate, snapping), pushed as #65; three new dependency advisories fixed on it, by updates and, for one wrapper crate that holds no certificates and is never built, by ADR-107's guarded ignore, the owner's choice.** Before it: **§4 item 107 (19 and 20 October 2026): P3.13 pushed as #63 and merged by the owner, CI green on it and on the merge; P3.14, materials (pictures on shapes laid by size, spot lights and HDR skies), built on `session/qq-materials` on #63's head and kept back until #63 is merged.** Before it: **§4 item 106 (19 October 2026, late night): #62 merged; P3.13, sound (one-shots from logic, music that fades from track to track, volume groups, and sounds panned in a browser on the page's own Web Audio), built on `session/qq-sound`.** Before it: **§4 items 103 to 105 (19 October 2026, late night): P3.10, 2D (#60, with ADR-106), P3.11, fixed steps and replay (#61), and P3.12, animation (#62), each built, merged by the owner, and ticked.** Before them: **§4 item 102 (19 October 2026, late night): #57 and #58 merged; P3.9, input actions (named, rebindable, on every device and under a finger, with pointer picking), merged as #59.** Before it: **§4 item 101 (19 October 2026, night): P3.8, the game's interface (screens over the world, pause, QQ's faces and reduced motion), merged as #57.** Before it: **§4 item 100 (19 October 2026, night): #54, #55 and #56 merged by the owner and live: QOR ID redeployed with migrations 024 and 025, ARQADE without Energy, the homepage's corrected line; P3.7 ticked, and the engine's gate counts ADR-105's steps.** Before it: **§4 item 99 (19 October 2026): toward a worldwide beta, the code surveyed (the Beta gate, QQ and how CGT is handled) and three pull requests opened at the owner's word, #54 to #56: the wallet moved into the core shared with Android (L13.2); CGT compliance (QOR ID's old donation scheme retired, a stricter amount parser, ARQADE without Energy, a language check in CI); and QQ making real games (ADR-105) with gameplay in logic (P3.7) and a sample game.** Before it: **§4 item 98 (19 October 2026): CI's backdrop check made to test what it claims, by the owner's decision (a loosening, logged in GATES), merged (#52, at `485eda5`): each backdrop's shaders must compile and its program link, read from WebGL, and it must draw, where it needed 20 frames a second that CI's software renderer did not always give it.** (#51, the records of #50's merge, merged at `809136b`.) Before it: **§4 item 97 (18 October 2026): the QOR Launcher on Android decided (ADR-104, the owner's choices) and its first step built and merged (#50, at `15cfa12`): the core moved out of the Tauri crate with its first two modules, a C bridge, and a Qt shell whose APK runs on an Android 16 emulator, answering through the core.** Before it: **§4 item 96 (18 October 2026): ARQADE's site made for phones and tapping (P7.25) merged (#49, at `943815d`) and live on Vercel: a tab bar along the bottom, games filling the screen and played by tapping, every target 44 pixels or more, held by a new phone check in CI.** Before it: **§4 item 95 (18 October 2026): ARQADE's Qt app (P7.19, ADR-102) merged (#48, at `6cb96fb`) and published for testers in launcher 0.1.14, the GitHub Release `launcher-v0.1.14`: docked in the launcher's ARQADE screen, a Qt Quick 3D home on Direct3D 12, the games and arenas, the player's Sparks and the rankings, signed in to ARQADE's server through the launcher; every suite passes, and the launcher started the real app.** Before it: **§4 item 94 (18 October 2026): ARQADE rebuilt inside the launcher on Qt decided (ADR-102, ADR-103), and its first stage merged (#47, at `8746501`): a Spark is a thousandth of a CGT and the ember the atomic unit (ADR-101), shown and typed in the launcher; every suite passes.** Before it: **§4 item 93 (18 October 2026): launcher 0.1.13 merged (#46, at `2214c17`), its installer on the owner's Desktop, not published: each QOR ID on a PC is a profile of its own, with its own wallet and data, a PIN, and *Who's playing?* after signing out (ADR-099, the owner's, superseding ADR-098); every suite passes.** Before it: **§4 item 92 (18 October 2026): the homepage at `https://qorsync.dev/` is live, written for search engines and AI crawlers, with `demiurge.cloud` and `demiurge.guru` redirecting to it, and the repository's description, topics and website set (ADR-100, #45, merged at `316f83f`).** Before it: §4 item 91 (18 October 2026): launcher 0.1.12 published for testers as the GitHub Release `launcher-v0.1.12`, its download page updated, and #43 merged (`2b09bb0`, every check on it passed): each QOR ID on a PC keeps its own account, and the Vault and rail say whose account is on screen (ADR-098); every launcher suite passes, and two planted faults were caught.** Before it: §4 item 90 (18 October 2026): launcher 0.1.11 published for testers as the GitHub Release `launcher-v0.1.11`, after the owner merged #41 (`3e33c1d`); the download page points at it. Before it: §4 item 89 (17 October 2026): launcher 0.1.11 built, not yet published: a file's menu in the Library publishes it to the Mesh, and a premium look throughout (a bundled display face, a grouped rail with a gliding marker, lit buttons and panels); its installer on the owner's Desktop.** Before it: §4 item 88 (17 October 2026): the owner upgraded Demiurge Devnet to runtime 9; the Mesh is live on it, with its 600-block period, and paid seeding can be tested.** Before it: §4 item 87 (17 October 2026): #40 merged (`580aa9e`, CI green on every job), and launcher 0.1.10 published for testers as the GitHub Release `launcher-v0.1.10`, with QOR Media inside and a download page; the devnet's upgrade to runtime 9 rehearsed against its own live runtime and waiting on the owner's signature.** Before it: §4 item 86 (17 October 2026): the launcher's half of the Mesh's paid work (L10.2 to L10.4), on branch `session/mesh`, not merged: ADR-097's Mesh grant, one host dialog after which the Mesh signs its own proofs, claims, assignments and receipts and nothing a node describes differently; a seeder seen paid for holding and for serving on a development chain whose period is a minute. Not on the devnet, which lacks the pallet.** Before it: §4 item 85 (17 October 2026): the owner's "Go with best and most recommended path forward": ADR-096 accepts the Mesh's proofs, names `pallet-mesh` and gives the takedown list to root; `pallet-mesh` and `mesh-proof` built at `spec_version` 9 on branch `session/mesh`, not merged, not on the devnet.** Before it: §4 item 84 (17 October 2026): the Mesh decided (ADR-095) and L10.1 built on branch `session/mesh`, not merged: a node in the launcher on iroh, seeding opt-in with the person's limits, own published work served, files fetched verified; no CGT moves yet.** Before it: §4 item 83 (17 October 2026): launcher 0.1.9 built from `main`, with Social and the Library, its installer on the owner's Desktop.** Before it: §4 item 82 (17 October 2026): #37 merged, the Library and QOR Media in `main`, CI green on it with its images from `mirror.gcr.io`; Qt PDF was installed all along, so PDF (L9.5) can start.** Before it: §4 item 81 (16 and 17 October 2026): the Library and QOR Media, release 1's first four items (L9.1 to L9.4), built on branch `session/library`. Before it: §4 item 80 (17 October 2026): #35 merged and live on Railway, World, friends, translation and levels with it; the owner tests later; next, the Library.** Before it: §4 item 79 (16 October 2026): room levels, Elite accounts and godmode (ADR-093), added to #35. Before it: §4 item 78 (16 October 2026): the owner's first conversation on Social, and World, friends and circles, and translation on the reader's PC built (ADR-092), not merged. Before it: §4 item 77 (16 October 2026): Social deployed on Railway beside QOR ID, checked from outside, and the launcher pointed at it. Before it: §4 item 76 (15 October 2026): #31 merged, QOR ID's side of Social live, and the launcher's Social surface built and merged as #33 (16 October 2026). Before it: §4 item 75 (15 October 2026): Social decided (ADR-091), QOR ID ready for it, and the Social service built and tested, now in `main`. Before it: §4 item 74 (15 October 2026): the owner's first publish from QQ Studio was refused by the node before any
block; an account that cannot pay is now told so before anyone is asked, naming the account and the chain, and the
whole reason reaches the Studio and the launcher's log.** Before it: item 73 (10 October 2026): P3.4 built. QQ Studio
publishes a project as its Cartridge through the launcher that started it (ADR-088, the owner's: a Cartridge holds the
commit; later publishes revise it), over ADR-046's channel, built for its first product. Both on branch
`session/qq-p3.4-publish`; item 73 is commit `65bebff`. Before them: item 72 the same day: L1.4's transfer and
RPC endpoint dialogs exercised by the owner, each approved and declined; what remains of L1.4 concerns QOR ID; with
item 71 the same day: the lockdown measured on mains power, costing nothing measurable; the Player's bundled game had been playing
without its model, and now a model that fails to load is reported and fails the checks. Before them: item 70 the same day: the QQ Player locks down what a game's logic
can reach (ADR-087 decision 2), with two known limits; merged as #27. Before it: item 69 the same day: how a stranger's QQ
game is confined, proposed and decided (ADR-087): in the browser only at first, from a site of its own; the Player
locked down in every mode; native play of others' games deferred. On branch
`session/qq-logic-sandbox`. Before it: item 68 the same day: **generation is paid by whoever uses it, and Demiurge pays for none (ADR-086,
the owner's decision); and P3.3, which merged into #21's branch rather than `main`, is brought to `main` by pull request
#23.** Both merged (#23 and #24), and CI passed on `main` at `b99c689`. Before it: §4 items 66 and 67 the same day: P3.3 all but its proof. QQ Studio offers fourteen tools to agents over
MCP through `qq-mcp` (driven end to end by the official MCP Python SDK), and its Agent panel designs and builds a game
from a description on Claude Opus 5.5 (ADR-085), the key in Windows Credential Manager; the proof needs the owner's
key.** Merged as pull request #22 into #21's branch; to `main` by #23. Before them: item 65 the same day: **P3.2 done. Worlds
play: physics, a player under keyboard, mouse and gamepad, particles, sound and logic rebuilt on each save, in QQ Studio
(Play and Stop, the scene restored exactly) and in the QQ Player, natively and in a browser, measured (ADR-084).** In
pull request #21 (`session/qq-p3.2-play`), whose checks all pass. Before it:
item 64 (9 October 2026): P3.1 done, QQ Studio edits a lit 3D world and keeps it in a Qontrol project (merged as pull
request #19). Before it: item 63 the same day: QQ's native engine
started on Qt 6.12 (merged as pull request #18). Before them: item 62 (8 October 2026): QQ moves to Qt 6 (ADR-083, the owner's decision); ARQADE's
window is signed in with the launcher's session; the accessibility flake is found and fixed; item 61 the same day: QQ's
TypeScript preview, and IBM Bob's unfinished work checked, fixed and finished. Both merged as pull request #17. Before them: item 60 (7 October 2026): the launcher's browser checks
wait 30 seconds for Chrome, not 10; item 59 (6 October 2026): ADR-081 accepted — QQ is QOR Engine, and the Godot plan is withdrawn; item 58 (6 October 2026): documentation brought up to date with the tree after the IBM Bob audit; item 57 (6 October 2026): avatars (ADR-079) and effects allowed (ADR-080), merged to `main` (PR #13); item 56
(6 October 2026): levels and tasks built (ADR-078), merged (PR #11), then migration 021 and launcher 0.1.8 (PR #12); item
55 (6 October 2026): ADR-078 accepted, levels, tasks and a 100 CGT welcome grant (PR #10), with "Play Now" and the fix
for a flaky wait in the launcher's browser checks (PR #9); item 54 (5 October 2026): P7.4 built, tips through the QOR
Launcher (ADR-076, ADR-077), merged (PR #8); item 53: QOR ID account page, merged (PR #7); ADR-076 accepted (cap 100,000
CGT); item 52: one name per QOR ID, no #0001 (ADR-075), merged (PR #5); item 51: QOR ID sign-in live for apps; ARQADE
moved to Vercel with Postgres (ADR-074), merged (PR #2, then PRs #3, #4 and #6); item 50: P7.3 built, QOR ID signs people
in to ARQADE, merged (PR #1); item 49: P7.1 done, ARQADE renamed, inventory written, lint a gate; item 48: P7.2 done,
ARQADE reads any account from the devnet; item 47: Demiurge Devnet at spec_version 8, rounds live; item 46: rounds with
held prizes and the SDK's wallet module; item 45: Demiurge Devnet upgraded to spec_version 7 by the owner, the ARQ Wallet
live; item 44: the upgrade rehearsed; item 43: the chain pinned to Rust 1.98.1 (ADR-072, the owner's option A); item 42:
merged to `main`; the chain CI job red since Rust 1.99, the owner's call; item 41: `pallet-arq-wallet` built and tested,
before it reached the devnet; item 40: ADR-070 and ADR-071 accepted, the wallet named ARQ Wallet; item 39: ARQADE as a
self-publishing store (ADR-071); item 38: Game Vaults designed (ADR-070) and the ARQADE SDK started; item 37: ARQADE in
`products/arqade/`, reading the devnet; item 36 the same day: ARQADE, the gaming platform the owner named, reconciled
with the tree — a corrected brief (`docs/blueprints/arqade.md`), ADR-069, track P7, gate `arqade`, U-16 and gaps G-14 to
G-16. Before those, §4 item 33 (2 October 2026): half-finished, unrecorded work found in the tree was finished and
verified (chain `spec_version` 6 with `buy_exact`; the launcher's Market host read; QOR ID's last-used sessions and the
e2e email guard). It is on `main`, and the chain has since moved to `spec_version` 8.

**History, 13 to 30 September 2026.** §4 item 32 (29 and 30 September) made the code public, moved CI back to GitHub
Actions and began moving QOR ID to Railway; all three are done (ADR-063, ADR-064), and QOR ID is live on Railway. Before
it, §4 item 31: M4.2's royalty half — royalties, remix royalties and a sale settled in CGT (ADR-061), proven live on a
development node. Before it, §4 items 27 to 30: CI on Woodpecker and the operations stack, both since retired. Before
those, §4 item 25 (28 September): onboarding — a QOR ID bubble, a glowing notification and a tutorial, an intro splash
and a first-run slot, Agora renamed Social, explanations behind information icons, and QOR ID and a chain running on
this PC. Before it, §4 item 24: no lock screen. The vault's key is in the OS keychain, nothing is asked to open it, and
QOR ID never blocks the launcher (ADR-056, the owner's decision, superseding ADR-055 the same day). Before it, §4 item 23:
Windows Hello as the vault's only seal (ADR-055). Before that, §4 item 22: a view check for the Gate's restore screens
and Windows Hello. Before that, §4 item 21: unlocking the vault with Windows Hello as an opt-in (ADR-054). Before it, §4
item 20: restoring a vault from its recovery phrase when the passphrase is lost. Before that, §4 item 19: Qontrol's git
layer completed — branch, switch, discard, per-file commit, a guard before staging, and the helper bundled with the
launcher — which ticks P1.2. Before it, §4 item 18: Qontrol's line-by-line diffs, reading line endings and
`.gitattributes` as git does, which ticked P1.1. Before that, §4 item 17: the trade window reworked into two sides with a
lane, "recently traded with", and Sell drafting a listing that publishes nothing. Before that, §4 item 14: M4.1, the
first asset pallet, and the first asset minted from a Qontrol project and shown in the launcher's Inventory. M2.1 was
ticked the same day against the owner's review of a ten-line summary.

**Read this first, especially §2.** Then:
1. [`docs/DIRECTION.md`](docs/DIRECTION.md): what Demiurge is, and the only roadmap, including the launcher
   track.
2. [`docs/DECISIONS.md`](docs/DECISIONS.md): the only decision log, with the ADR index.
3. [`docs/GATES.toml`](docs/GATES.toml): the accepted release gate criteria. Read its rule on changing
   evidence before touching it.
4. [`docs/architecture/MIGRATION_INVENTORY.md`](docs/architecture/MIGRATION_INVENTORY.md): **all but
   Q-19 and Q-20 of its twenty questions are decided** — Q-1 to Q-16 by ADR-017 to ADR-032, confirmed by
   the owner on 2026-09-17, **Q-17 by ADR-041 on 2026-09-20**
   ([`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md) is the write-up it was
   decided from), and **Q-18 by ADR-057 on 2026-09-28** (eight royalty recipients stay). **Q-19 and Q-20
   are open until before the freeze** (§4 item 14). **M2.1, the owner's
   review of it, was ticked on 2026-09-22** against a ten-line summary, kept verbatim in the document.
5. [`docs/audit/RECONCILIATION.md`](docs/audit/RECONCILIATION.md): the code measured against the
   direction, with a remediation table.

---

## 1. Status at a glance

Re-checked on 22 October 2026 against `main` at `c401aee` and pull request #77 on it (§4 item 127), and the launcher
and CI rows again after #77's merge and the release of 0.1.15 (§4 item 128): test counts are
CI's on `main` (run 39865861460) unless a row says otherwise, and the live services were read from outside that day.
`docs/SYSTEMS.md` says what each system is; this table says where it stands.

| Area | State |
| --- | --- |
| Direction | Settled. Demiurge is a creative engine on a purpose-built Substrate L1, with four primitives. The launcher is a first-class deliverable with its own track (L0 to L13). **The active product track is P7, ARQADE** (ADR-069, active scope since 4 October 2026), with QQ (P3, `products/qq/`, ADR-083) as its engine. |
| Base layer | Migration to the Polkadot SDK decided (ADR-013) and complete since M3.5 (20 September 2026). **Q-1 to Q-16 decided** (ADR-017 to ADR-032), confirmed by the owner on 2026-09-17; Q-17 (ADR-041) and Q-18 (ADR-057) decided since; Q-19 and Q-20 open before the freeze. Pinned to `polkadot-stable2606-1` (ADR-022), built with Rust 1.98.1 (ADR-072). |
| The Substrate chain (`chain/`) | **`spec_version` 11 on `session/asset-rental` (#106, not merged; §4 item 159)**: M4.3's rental (ADR-119); 213 workspace tests pass with the wasm built on the owner's PC, 26 October 2026. **`spec_version` 10 on `main` since #82** (ADR-113's progress records, §4 items 132 and 136). **190 workspace tests pass with the wasm built** (CI on #82's head `522b7aa`, 23 October 2026); its upgrade was rehearsed against the devnet's live runtime 9 (§4 item 133). Pallets: System, Timestamp, Aura, GRANDPA, Balances, Session, ValidatorSet, `pallet-nfts` (7), `pallet-drc369` (8, ADR-052, nesting ADR-065), `pallet-utility` (9, only `batch_all` reachable, ADR-053), `pallet-drc369-royalties` (10, ADR-061), `pallet-arq-wallet` (11, ADR-070) and `pallet-mesh` (12, ADR-096), with Sudo (100) behind a build feature a public release is built without. **No transaction payment (OPEN-4), no treasury (OPEN-2) and no issuance (OPEN-1)**; deposits are placeholders (U-14). **Live as Demiurge Devnet** at `wss://rpc.qorsync.dev`, genesis `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`: two validators and an RPC node on Railway (ADR-068), the owner holding the sudo key; at runtime 10 since the owner's upgrade of 23 October, at block #289,065 (read back that day: the rehearsed file, two peers, finality advancing). History: first finality 17 September (M3), M4.1 22 September, royalties 29 September, the ARQ Wallet 4 October, the Mesh 17 October (§4 items 14, 31, 41 to 47, 84 to 88). |
| The custom devnet (`framework/`) | **Retired and deleted at M3.5, 2026-09-20.** 150 files, 49,058 lines, after everything that referred to it had been rewritten and verified with the tree still present. Its last run passed 285 tests. Untrusted, and nothing of it is carried forward (§2.0). **M3 is not complete:** M3.2 waits on OPEN-2. |
| Security track | Items 1 to 7 done. **The owner confirmed credential rotation on 2026-09-14:** the Postgres password and both JWT secret pairs. **Nine credential-shaped values** (from `docker/n8n/docker-compose.yml` and `docker/docker-compose.testnet.yml`) are not in the current tree or in the public repository's history; they exist only in the private archive's history, and rotating them is still advised if those services ever ran (an owner step in `OWNER.md`). CI's security job scans for committed credentials. **A recovery phrase was pasted into a chat on 5 October; treat that key as exposed and replace it.** Sessions record a placeholder IP address. §3. |
| `services/qor-auth` (QOR ID) | **Live at `https://id.qorsync.dev`**, the Railway service `qor-auth`, deployed from `main` (`/ready` 200 on 22 October 2026). **185 tests pass** against Postgres 16 and Redis 7.4 with `--include-ignored`, the log and SQL checks among them (CI, 22 October 2026). Latest migration: 025. Password and key sign-in (Sr25519 and SS58; ADR-023, ADR-024, ADR-039); email verification, reset and address change through Resend; the account page; sign-in for other apps by OAuth 2.1 with PKCE (ADR-073), ARQADE registered, and introspection for Social (ADR-091); one name per account (ADR-075); levels, XP and tasks, the welcome grant recorded as owed and never paid (ADR-078); avatars (ADR-079); the launcher's handoff into an app it opened; its pages and emails in the one look (#73). Registration creates no CGT and calls no chain (R-3). Agent keys are authorised, never created (ADR-014). **The Resend bounce webhook is an owner step, and delivery to a real inbox is unconfirmed** (§4.0). **Since #79, live since QOR ID's redeploy of 23 October 2026** (ADR-112, §4 items 129 and 130): an agent, or a change to what it may do, only with the controller's own key's signature; agent names that make valid QOR IDs; 192 tests. History: §4 items 5 to 8, 33, 50 to 57, 99, 100, 119 to 123 and 129. |
| `services/social` (Social) | **Live since 16 October 2026** at `wss://social-production-e05c.up.railway.app/v1/socket` (ADR-091; `/health` 200 on 22 October 2026): rooms, direct messages, sealed messages, reactions, replies, edits, deletes, unread counts, mentions, blocks, reports, presence and profiles over one WebSocket, identity from QOR ID's introspection; World, friends and circles, message languages (ADR-092); room levels, Elite and godmode (ADR-093). **40 tests** against Postgres 16 (CI, 22 October 2026); migrations 001 to 003. Its client, the launcher's Social surface, is in every published launcher since 0.1.10; the owner's first conversation worked on 16 October 2026 (L8.3 to L8.5 ticked), and the owner's look at World, translation and levels is to come. Audited in CI. History: §4 items 75 to 80. |
| `tools/qor-launcher` | **Version 0.1.17 is the tester download** (the GitHub pre-release `launcher-v0.1.17`, 25 October 2026, from `5db8349`; 88,784,650 bytes, downloaded back and verified against the channel's key, whose `latest.json` announces it; §4 item 155), carrying ARQADE's six games in 3D, `qor://arqade` (ADR-118, #99), the level themes (P7.16, #101), assets' levels and QOR Media's PDFs. 0.1.16, the first release the update channel offered (23 October), 0.1.15, the first that updates itself (22 October), and 0.1.10 to 0.1.12 and 0.1.14 stay published. Unsigned (L6.1). It carries the wallet in the core shared with Android (#56), the launcher in Obsidian (#72) and the update channel (#77). **On `main`** (CI, 22 October 2026): 257 host tests pass (13 ignored), the core's 72 (1 ignored), and sixteen browser checks. **The update channel** (L6.2, ADR-111, §4 items 126 and 128), merged with #77: 264 host tests (14 ignored) and the core's 72 (2 ignored) on the owner's PC, and eighteen steps in `npm run check`. Features: profiles with a PIN (ADR-099); the vault's key in the OS keychain with no lock screen (ADR-056); QOR ID sign-in by key; CGT and Sparks (ADR-101); send, mint, Inventory, trade, sell and buy; the Market (no search, no indexer); Projects and Qontrol; the gates dashboard; `qor://pay` behind the host dialog (ADR-077); levels and avatars; Social; the Library with QOR Media; the Mesh; ARQADE's Qt app docked (ADR-102); QQ Studio through the product host (ADR-046, ADR-088). The Nexus still shows nine frozen-app tiles. **L1.4 is mostly exercised:** the QOR ID endpoint change and a sign-in after QOR ID was unreachable are not (§4.0). **Since #79 and in 0.1.16** (ADR-112, §4 items 129 and 130): `qor://agent`, an agent authorised by the profile's own key after the host's dialog; 274 host tests (14 ignored) and the core's 73 (1 ignored) in CI. **Since #82, and in 0.1.17** (ADR-113, §4 items 132 and 136): the Inventory and Market cards show an asset's level and XP as its home game keeps them, which the devnet now serves; 275 host tests (15 ignored) on the owner's PC. History: §4 items 9, 14 to 24, 54 to 57, 81 to 97, 117, 126, 129 and 132. |
| QOR Launcher for Android (`tools/qor-launcher/shell/`, `core/`) | Merged (#50, #56, #74; ADR-104): the core with Sparks, errors and the wallet moved into it (L13.2), a C bridge, and a first Qt screen in the one look (`tst_core` 6 tests); its x86_64 APK runs on an Android 16 emulator, and an arm64 APK builds, not yet run on a phone. No Keystore yet (L13.3), so it makes no wallet. History: §4 items 97, 99, 100 and 122. |
| **The Mesh** (`src-tauri/src/mesh/`) | **Merged with #40 (`580aa9e`, 17 October 2026) and live on Demiurge Devnet since the owner's upgrade to runtime 9 (§4 items 84 to 88; ADR-095 to ADR-097).** A node in the launcher's host on iroh 1.3 and iroh-blobs 0.103 (whose own README calls it not yet production quality). Seeding off until the person turns it on with disk and upload limits; own published work served; files fetched by ticket, verified against their BLAKE3 root. Paid seeding: `pallet-mesh` and `mesh-proof` on the chain, and the Mesh grant (one host dialog, after which the Mesh signs its own proofs, claims and receipts). Not yet: two PCs over the internet, private backup (L10.5), Demiurge's own relays (L10.6) and the takedown list (L10.7). |
| `tools/qor-media` (QOR Media) | **In `main` since #37 (17 October 2026; ADR-094, L9.3 and L9.4), in the one look since #74.** The Library's player, viewer and editor: Qt 6.12, C++20 and QML, started by the launcher under the product host and docked as a child of its window. **39 tests in five Qt Test suites** pass on the owner's PC (22 October 2026; CI has no licensed Qt), and the launcher's opt-in test starts the deployed copy. Deployed to `%LOCALAPPDATA%\qor-media` by `build.ps1 -Deploy`. **In `main` since #83 and #84 (§4 items 134 to 136):** the reader, PDFs read with Qt PDF, and their pages turned, deleted, moved and saved through PDFium; seven suites on the owner's PC (23 October 2026). **In `main` since #85 (§4 items 137 and 139):** a PDF's own marks drawn on its pages (`src/pdfdrawing.cpp`), which Qt PDF's page image left out. **In `main` since #86 (§4 items 138, 139 and 141):** a PDF marked up, words highlighted, underlined and struck through, notes, ink, and a picture of a signature drawn or typed, and its own marks read and taken away (`src/pdfmarks.cpp`); eight suites on the owner's PC. **In `main` since #87 (§4 items 140 and 141):** a PDF's form drawn (no field had been) and filled (`src/pdfforms.cpp`, `qml/FieldItem.qml`); nine suites on the owner's PC. **Not yet:** forms kept in XFA (the IRS's) and a form's scripts, certified signatures (ADR-114 decision 2, proposed), Office documents (L9.7), HEIC, AVIF, JPEG XL and RAW (L9.8), video thumbnails, a controller inside QOR Media, and the owner's first look. |
| QQ (`products/qq/`) | **Every step of ADR-105 is done: P3.1, P3.2 and P3.7 to P3.16** (9 to 21 October 2026), in the one look since #70. QQ Studio and the QQ Player on Qt 6.12, natively and as WebAssembly; **160 cases in eight Qt Test suites** pass on the owner's PC (21 October 2026; CI has no licensed Qt, ADR-083). Open: P3.3's proof (made on its user's own key, ADR-086), P3.4 (built; ticked when the owner has published from QQ Studio), P3.5 (ARQADE delivery) and P3.6. History: §4 items 62 to 74 and 99 to 113. |
| Products | **Seven, each with a blueprint, a track and a gate**: P1 to P7. The shared substrate decisions ADR-046 to ADR-051 are all accepted. Built: Qontrol's Projects (P1.1, P1.2); QFX's six backdrops (P2.1) and the one look on every surface (P2.9); the Market view (no indexer); QQ (row above); ARQADE (row below). GNOSIS and Stream have not started. |
| Release gates | **`GATES.toml` accepted (2026-09-14).** Measured on 22 October 2026 as the launcher's dashboard measures them, with CI and the live services read: **Alpha** is short of M3.2 (OPEN-2) and L1.4's last two dialogs; its suites' units count once they are run from the dashboard, and CI runs them green. **Beta**: 8 units met, 35 not, 2 unmeasurable (`beta.chain` 2 of 22, `beta.launcher` 3 of 12), most of it waiting on the owner (§4.0, "The Beta gate"). **Loosening any evidence rule needs the owner's separate decision** (the rule is in the file); every change is logged in its change log. |
| CI | **"Pleroma CI", GitHub Actions on the public `QOR-MATRIX/demiurge-chain` (ADR-063, ADR-064), and green on `main`** (run 39865861460, the merge of #76, its launcher job on a second attempt). Jobs: `chain` (format, lints and tests, without `SKIP_WASM_BUILD`); `chain-newest-clippy` (scheduled, a report that never fails the run, ADR-072); `two-validators` (scheduled); `qor-auth` (with Postgres and Redis, `--include-ignored`); `social`; `launcher` (the Qontrol helper, the product host checks, the core, the host, and `npm run check`); `arqade` (with its phone and design checks); `coverage`; and `security` (dependency audits with ADR-089's and ADR-107's ignores proven by their own steps, the committed-credential scan, the `[patch]` and `framework/` guards, the homepage's check, and the one look's check). **A recurring failure:** a browser check stopping with *The browser did not expose a page to inspect* (pull request #34, the merge of #47, and twice on 22 October 2026, once in ARQADE's phone check in the same minute); #77 made Chrome choose its own port and the failure say why (§4 item 127). On #77's merge the launcher job hung 30 minutes restoring its cache; every cache step now stops after ten minutes and the job builds without it (§4 item 128). Woodpecker and `ci.qorsync.dev` are retired; deleting the `ci` DNS record in Cloudflare is an owner step. History: §4 items 1, 27 to 32, 42, 60, 98, 108 and 127. |
| Infrastructure | **Railway** (project `demiurge`): Postgres, Redis, QOR ID (`id.qorsync.dev`), Social with its own Postgres (ADR-091), and Demiurge Devnet (two validators and an RPC node, ADR-068). **Vercel** (team Astra Matrix): ARQADE, project `qor-arqade` on the Hobby plan, with Neon Postgres (ADR-074); the homepage, project `demiurge-site` (ADR-100). Fly.io is not used: ADR-015's Fly plan is superseded by ADR-063 and ADR-068. |
| Production | No production or test network. Live, devnet-only and with test CGT: Demiurge Devnet, QOR ID, Social, ARQADE and the homepage. |
| ARQADE (`products/arqade/`) | **Live at `https://qor-arqade-tau.vercel.app`** (200 on 22 October 2026): standard Next.js 16 on Neon Postgres, migrations 0001 to 0003 applied on production builds. **49 tests**, its phone check (17) and its design check pass (CI, 22 October 2026). QOR ID is the only sign-in; Play Now, the solo games, Flux Four and Rift Reversi multiplayer, rankings, chat, devnet reads, tips through the launcher (ADR-076, ADR-077), levels and QOR ID avatars; free play with no Energy (#54); made for phones (P7.25); in the one look (#71). **Its Qt app** (P7.19, ADR-102) is docked in launcher 0.1.14; 35 Qt tests on the owner's PC (21 October 2026). **All four solo games rebuilt in QQ** (P7.20, done) play in it, merged as #89 to #92 and deployed to the launcher's copy from `main` (§4 items 142 to 146). **The two arenas rebuilt in QQ** (P7.24, done) play in it on ARQADE's server, merged as #96 and deployed to the launcher's copy from `main`, last from `20d505f` with the launcher's `play` command (ADR-118): 79 Qt tests on the owner's PC, and 105 Node tests and 18 phone checks in CI (25 October 2026, §4 items 149, 151 and 153). The website's games point players to the launcher (P7.23, done, live). P7.1 to P7.4 are ticked. Emulated play is deferred by the owner for future reference (ADR-110). |
| Decisions | **119 ADRs** (ADR-001 to ADR-119), indexed in `docs/DECISIONS.md`. All are accepted except ADR-098 (superseded by ADR-099), ADR-103 (accepted in direction; its decision 2's bridge superseded by ADR-104), ADR-110 (deferred by the owner, 22 October 2026), ADR-114's decision 2 (certified PDF signatures, proposed) and ADR-116 (confirming an ARQADE run by replay, proposed for the owner, 24 October 2026). The newest accepted: ADR-118, a `qor://arqade` link opening the launcher at ARQADE, at a game if it names one (under the owner's delegation, 25 October 2026), after ADR-117, a QQ game and its host speaking through one channel kept in the run (under the owner's delegation, 24 October 2026), after ADR-115, the security audits of QOR ID and the launcher ignoring an advisory in the SDK's secp256k1 stack while CI proves none of them uses ECDSA (accepted by the owner, 24 October 2026), after ADR-114, signing a PDF in QOR Media (decision 1 accepted by the owner, 23 October 2026), after ADR-113, an asset's progress kept by each game that earns it (accepted by the owner, 23 October 2026), after ADR-112, the controller's own key signing every agent authorisation (22 October 2026). The owner's open questions are in §4.0. |
| Not live | Paying the welcome grant (recorded as owed, never paid), the indexer, fees and issuance, and private backup on the Mesh. |

## 2. Known gaps: do not build on these

### 2.0 There is one chain

**`chain/` is the chain.** `framework/`, the custom Rust devnet it replaced, was retired at M3.5 on 20
September 2026: everything that referred to it was rewritten first and verified with the tree still
present, then the directory was deleted in a commit of its own.

Everything below about two chains is history, kept because a lot of this file and a lot of muscle memory
were shaped by it.

| | `chain/` |
| --- | --- |
| What | The Substrate L1 (ADR-013), located and named by ADR-032 |
| Chain identity | `demiurge_dev` / "Demiurge Development", `demiurge_local` / "Demiurge Local Testnet", or `demiurge_devnet` / "Demiurge Devnet", the live devnet (ADR-068; `chain/node/src/chain_spec.rs`) |
| Finality | GRANDPA (ADR-018) |
| RPC | Standard Substrate. The launcher speaks it over WebSocket (ADR-040); its default is `wss://rpc.qorsync.dev`, and `ws://127.0.0.1:9944` is `LOCAL_RPC`, for a local node |
| Crate and binary | `demiurge-node` |
| Address type | `AccountIdLookup`; an extrinsic's address is a `MultiAddress` (ADR-041) |

**What went, and what remains of it.**

- The custom Rust devnet in `framework/`, its `demiurge-node-legacy` binary, its private RPC vocabulary
  (`chain_getHealth`, `chain_getBlockNumber`, `account_getTransactionNonce`, …), its 285 tests and its
  six exempted advisories went together, in one commit.
- **It was untrusted, and that is why nothing of it is carried forward.** Its transaction validation used
  non-strict Ed25519 verification, and a test on 2026-09-14 showed it accepting a signature nobody made
  for small-order public keys: the identity-point key on 64 of 64 messages, the all-zero key on 13 of 64.
  Any account with such a key could be spent from by anyone.
- **What it established is not lost.** Its behaviour is in `chain/`'s acceptance tests and the
  two-validator script; its module-by-module mapping is in the migration inventory; what it did on the
  wire is in `PROTOCOL.md`'s history. The inventory's requirements outlived it: **R-1** is closed as met
  by standard behaviour (ADR-038), **R-2** is met in `chain/` (`pallet-drc369` refuses a nesting cycle with
  `NestingCycle`), **R-3** is
  closed in QOR ID, and **R-4** is met by `chain/` having been written with `CGT` from the start.
- **Do not bring it back.** CI fails if the directory returns. Reading it in git history is not forbidden
  the way the pre-realignment documentation is (AGENTS.md §1) — it is code, not a contradicting plan —
  but nothing in it should be copied forward.

**Two traps it left behind, which will outlive it by a while.**

- **`demiurge-node` means one thing.** For a few days two crates built a binary of that name, and from
  2026-09-17 the old one was renamed `demiurge-node-legacy` to stop it. Both names now resolve to one
  place or to nothing: `cargo build -p demiurge-node` in `chain/`, and `demiurge-node-legacy` nowhere.
- **Port 9944 is no longer contested**, so a node answering there is `chain/`. `system_chain` is still the
  way to be sure, and the launcher asks automatically and shows the answer.

Two records mention the old chain and are deliberately unchanged: `PLATFORM_REALIGNMENT.md`, which
records what was launched on a date, and every ADR that referred to it. An ADR is never edited.

**The key scheme changed on 19 September 2026, and no key stored before it still works.**
- Accounts are Sr25519, derived as the ecosystem derives them (ADR-023), and addresses are SS58 (ADR-024).
- Every key QOR ID held was an Ed25519 key. Those same 32 bytes now name an account nobody holds a secret
  for, so migration 018 cleared them rather than binding a QOR ID to an account it could never prove again
  (ADR-039).
- **A password account links its new key** through `link-keypair`. **An account created by key sign-in
  before that day cannot sign in at all** and has to be created again: its password is a random value
  nobody knows. Nothing was deployed on 19 September, so the accounts this touched were development accounts.
- **A vault created before that day opens different accounts.** The phrase is unchanged and nothing is
  lost; the addresses it derives are new ones, which is what ADR-023 accepted while nobody holds anything.
- **The wire changed with it.** `pubkey` is gone from every request; an account is named by `address`
  (SS58) or by `account_id` (hex), exactly one of the two.

**An access token alone could register an agent** (ADR-014, "Known gap"): **closed by ADR-112, live since QOR ID's
redeploy of 23 October 2026** (§4 item 130).
- QOR ID takes an agent, or a change to what it may do, only with the controller's own key's signature over a
  statement naming every field and a single-use challenge, and the launcher signs it in the vault after its own
  dialog (`qor://agent`, L5.1, in 0.1.16).
- L5.1 is a Beta gate criterion, ticked when a person has authorised an agent from an installed launcher against the
  live QOR ID.

**Other gaps, as of 6 October 2026.**
- **Welcome grants are recorded as owed and never paid** (ADR-078). Who funds them with real CGT is U-18.
- **No fees and no issuance.** The chain has no transaction payment (OPEN-4) and no issuance (OPEN-1).
- **The Market view has no search and no indexer**, and transaction history waits on the indexer (ADR-028).
- **Sessions record a placeholder IP address** (`0.0.0.0`, "not yet taken from the request", in
  `src/handlers/auth.rs`).
- **The launcher is unsigned and has no update channel** (L6). A new version is installed by hand.
- **Avatars are not in ARQADE** (ADR-079 step 3, not built). The launcher's avatars are on `main` and wait for a
  new build.
- **The Nexus still shows nine frozen-app tiles** (Explorer, Staking, Sophia, Scatter, Resonance, Worlds, Agent
  Foundry, Developers, Bounties), marked "local" or "forming".
- **The three-sign-ups-per-address limit trusts Railway's `X-Real-IP`** (ADR-078). If QOR ID ever runs behind
  another proxy, the header, and the limit with it, must be checked again.

## 3. Security track and other open findings

Details are in [`SECURITY.md`](SECURITY.md).

| # | Item | State |
| --- | --- | --- |
| 1 | Committed secrets removed from configuration | Done; **rotation confirmed by the owner on 2026-09-14** |
| 2 | Seeded admin removed (migrations 008, 010) | Done; verified on Postgres 16 |
| 3 | Balances self-transfer created CGT | Done, with tests |
| 4 | DRC-369 mint authorisation; parent-owner check on nesting | Done, with tests |
| 5 | QOR ID created and held agent private keys | Done (ADR-014, migration 011); verified end to end |
| 6 | Agent endpoints unscoped; wallet link unbound and unproven | Done; verified end to end |
| 7 | Domain-separated QOR ID challenge signing (launcher and `qor-auth`) | Done; also L1.5. Unit tests on both sides, and the e2e harness shows a bare-challenge signature refused at login and at key link (28 of 28 checks) |

**Migration requirements:**
- R-1: strict signature verification. Closed as met by standard behaviour (ADR-038; §5).
- R-2: refuse nesting cycles. Met in `chain/`: `pallet-drc369` refuses one with the error `NestingCycle`, tested
  in the pallet and the runtime. The custom chain's `do_nest` never refused them, and it is gone.

**Other open findings:**
- The secrets removed on 2026-09-14 are rotated. **Nine credential-shaped values** (from
  `docker/n8n/docker-compose.yml` and `docker/docker-compose.testnet.yml`) are not in the current tree or in the
  public repository's history; they exist only in the private archive's history. Rotating them is still advised if
  those services ever ran (`OWNER.md`). CI's security job fails on committed key material and on credentials in
  configuration files.
- **A recovery phrase was pasted into a chat on 5 October 2026; treat that key as exposed and replace it**
  (`OWNER.md`).
- **New surfaces since October:** avatar upload (an image parser on untrusted input, under size, frame and
  dimension limits; ADR-079) and the OAuth client secrets in `QOR_OAUTH_CLIENTS` (stored as SHA-256 hashes;
  ADR-073). Both are in the log check. **Since 8 October 2026** (§4 item 61): **ARQADE's window**, a second
  launcher window that loads a website. It is given no capability (`capabilities/default.json` names only `main`), so
  its page reaches no host command; the host routes every address it is asked to load or open as a new window
  (`src-tauri/src/arqade.rs`, five tests), and a `qor://pay` link from it meets the same checks and dialog as one
  from the browser. **QQ's scene commands** write only `scenes/<name>.qq.json` inside an opened repository, refusing a
  `scenes` link that leaves it. **The grant admin routes** are behind the admin middleware, audited, and handle no
  token, key or email address. **The launcher's handoff** (`/api/v1/oauth/handoff`) issues an app's code with a
  person's own session: an app's session is refused, the request is checked by the same function as the sign-in page,
  the code is bound to the app's PKCE challenge and redirect URI, and the flow is in the log check.
- **The per-address sign-up limit trusts `X-Real-IP`**, which Railway sets (ADR-078); `X-Forwarded-For` is not
  trusted.
- ~~The RPC reports `finality: 2000` and every transaction as "finalized".~~ **Gone with `framework/` at
  M3.5.** `chain/` has real finality (ADR-018), and the launcher reports a transfer only once GRANDPA has
  finalised it.
- There is no limit on concurrent QOR ID sessions; the unenforced setting was removed.
- **QOR ID weaknesses that remain** (2026-09-15; details in `SECURITY.md`). Closed on 2026-09-15:
  - a reset ends every session;
  - backup codes are single-use and hashed, and can be regenerated;
  - sign-in discloses nothing;
  - an account whose verification link lapsed can request a new one;
  - an account without an email can add a verified one (§4, item 7);
  - no token, key, link or address reaches a log. This is a standing check at runtime and in CI, GATES
    `alpha.no-secrets-in-logs`. Token material had reached the logs three times, from three directions.

  Still open:
  - some accounts cannot be recovered; they are named in `SECURITY.md` as accepted lockouts;
  - sessions record their last activity, but their IP address is a placeholder (`0.0.0.0`);
  - ~~a password-only account holds an `on_chain_address` no key can sign for~~. **Closed 2026-09-19**
    (ADR-017, ADR-039): registration stores no address, `hash_to_address` is removed, and migration 018
    cleared the derived addresses already stored.
- Agent capabilities and spend caps are recorded, not enforced; enforcement is on chain (Q-9).

## 4. What is next

### 4.0 What is blocked, and on whom

Read this before the detail below. "Ready" means it can start today with nothing outstanding.

| Track | State | Waiting on | What unblocks it |
| --- | --- | --- | --- |
| **P3, QQ on Qt 6** (ADR-083) | **P3.1, P3.2 and P3.7 to P3.16 done** (every step of ADR-105, 19 to 21 October 2026); **P3.3 built but for its proof; P3.4 built** (10 October 2026) | **The owner**, for P3.4's first real publish (§4 item 73) and for CI; **anyone with their own key** for P3.3's proof (ADR-086) | ADR-087 decided how a stranger's game is confined, and the Player's lockdown is built; ADR-088 decided what a Cartridge holds, and P3.4 is built over ADR-046's channel; next, P3.5 (strangers' games in the browser only, from a site of their own). CI cannot build QQ until Qt's installer and the owner's Qt account are repository secrets, or a self-hosted runner exists (ADR-083). |
| **P7.17**, paying the welcome grants | **Blocked, 2026-10-08** | **The owner** | ADR-078 decision 7 wants a server that holds the Welcome account's key and nothing else: a new service, which needs the owner's name for it (AGENTS.md §8), the Welcome account created and funded with test CGT on the devnet, and its key set as that service's only secret on Railway. Listing owed grants and marking one paid are built. |
| **M3, the Substrate chain** | **Unblocked, started 2026-09-17** | Nobody | The owner confirmed ADR-018 to ADR-032 and supplied all four inputs: eighteen decimals (ADR-035), an existential deposit of 100 CGT (ADR-036), the five pallet names, and `pallet-sudo` on development and test networks only (ADR-037). |
| **U-1, the decimal places** | **Decided 2026-09-17** | Nobody | Eighteen, on both recorded conditions (ADR-035). Removed from `OPEN_QUESTIONS.md`, so the gate reads it as decided. |
| **The existential deposit** | **Decided 2026-09-17** | Nobody | 100 CGT (ADR-036), `10^20` Sparks. Revisitable before mainnet only, and only if OPEN-2's sponsorship budget changes the arithmetic. |
| **L3.1**, the launcher's chain client | **Done, 2026-09-20 (ADR-040)**, which completes M3.4 | Nobody | The client is `subxt` at 0.51, under ADR-033 rule 2, with every call built from the metadata the node serves. Proven against a development node: an approved transfer signed in the vault and finalised by GRANDPA, a declined one moving nothing, and two sent back to back. Two calls still refuse, and for reasons that live elsewhere: **transaction history** waits on ADR-028's indexer, and **the starter claim** waits on the economics (OPEN-1, OPEN-2). |
| **Q-17**, the runtime's address type | **Decided and done, 2026-09-20 (ADR-041)** | Nobody | The owner chose `AccountIdLookup`, and it was carried out the same day: runtime and launcher in one commit, because a node and a client on opposite sides of the change refuse each other. 51 chain tests, 13 two-validator checks and the launcher's live test all pass on it. |
| **L4**, the studio | **L4.2, L4.4, L4.5 done**; **half of L4.6 built 2026-09-23, unticked**; L4.1 started | Nobody, to continue; **the owner** for the listing vocabulary | Owned assets are listed from on-chain enumeration (L4.2), traded (L4.4, L4.5), and since 2026-09-23 a listing can be drafted for one — saved on this machine and published nowhere, so **L4.6 stays unticked** until M4.2, P5.5 and M5.4 give it somewhere to publish. The six categories it offers are a proposal for the owner to confirm (§4 item 17). Minting any file, and collections beyond a creator's singles, are L4.1's remainder. |
| **L1.4**, host-side confirmation | **Mostly exercised** | **The owner**, for the last two | Tips approved on 5 and 6 October 2026; a transfer and an RPC endpoint change each approved and declined on 10 October (§4 item 72). Still needed in a running launcher: the QOR ID endpoint change approved once and declined once, and a key sign-in after QOR ID was unreachable at unlock (item 4 below). |
| **L2.2**, the gates dashboard | Part verified, 2026-09-17 | Nobody | The numbers are now covered from both sides: the host against the real `GATES.toml`, and the view against a known fixture in a real rendering engine (`scripts/check-gates-view.mjs`, 34 checks, wired into CI and `npm run check`). What is left needs a person: running a suite with one approval and one refusal, and the `gh` readings (item 4 below). |
| **L1.6**, launcher tests in CI | **Ticked 6 October 2026** | Nobody | The `launcher` job runs and passes in CI on `main` (every completed run since the merge of #6, 5 October 2026, except two browser-start failures on the merge of #15, §4 item 60; on the merge of #13, run `37496555161`, 204 host tests passed and 7 were ignored). Ticked in `DIRECTION.md` against that evidence. |
| **L1.7**, CI runs at all | **Ticked 6 October 2026** | Nobody | CI starts and passes on `main` (the merges of #12 and #13), and the `coverage` job uploads the `coverage` artifact `GATES.toml` reads. Ticked in `DIRECTION.md` against that evidence. |
| **L6, distribution** | Partly done | **Owner**, for signing | Unsigned installers are published for testers as GitHub pre-releases: 0.1.10, 0.1.11, 0.1.12, 0.1.14, 0.1.15, 0.1.16 and 0.1.17 (17 to 25 October 2026). **Signing is not done. The update channel is merged (ADR-111, #77); 0.1.15 carries it, and 0.1.16 is the first release it offers** (§4 items 128 and 130); **L6.2 is ticked** (26 October 2026): the owner's launcher took 0.1.17 through it (§4 item 158). Signing needs a Windows code-signing identity and an Apple Developer ID, which the owner was obtaining on 2026-09-17. |
| **The landing page** | **Deferred, 2026-09-17** | Nobody, until the owner wants it | The owner's decision: do not start it and do not extend frozen scope. If it is wanted it gets a roadmap item and its own decision first (item 11 below). |
| **Live email to a real inbox** | Blocked | **Owner** | The Resend webhook to `https://id.qorsync.dev/api/v1/webhooks/resend`, with its signing secret in Railway as `RESEND_WEBHOOK_SECRET` (item 7 below). Delivery to a real inbox is unconfirmed. |
| **Deployment** | **Devnet stage done** | Nobody, for the devnet | QOR ID and Social on Railway (ADR-063, ADR-091), Demiurge Devnet on Railway at runtime 10 (ADR-068), ARQADE and the homepage on Vercel (ADR-074, ADR-100), all devnet-only with test CGT. No production network exists. |
| **M2.1**, the owner's review of the inventory | **Ticked 2026-09-22** | Nobody | Against the owner's acknowledgement of a ten-line summary of the DRC-369 section, kept verbatim in the inventory. M2 is done. |
| **M4.1**, the asset primitive | **Done 2026-09-22** (ADR-052) | Nobody | §4 item 14. Its deposits are placeholders (U-14) and its weights are placeholders owed to M7.2. |
| **M4.2, M4.5** | **Ticked 2026-10-23** with state and XP (ADR-113, §4 item 132): royalties (ADR-061), nesting (ADR-065) and progress records built | Nobody | Merged (#82) and live on the devnet since the owner's upgrade of 23 October 2026 (§4 item 136). ADR-113's second phase, visiting games, is not built. |
| **M4.3** | **Ready** | Nobody | Rental, fractional ownership and physics. None needs an economic value. M4.4 waits on U-4 and OPEN-4. |
| **The freeze** (`beta.wire-format-frozen`) | Blocked | **Owner**, by an ADR each | Q-19 (whether an asset's root depends on its files alone) and Q-20 (whether the media-type table is part of an asset's identity). `beta.manifest-identity` and `beta.media-types` count them. Q-18 was answered by ADR-057 on 2026-09-28: eight royalty recipients stay. |
| **ADR-047** (the asset format) and **ADR-051** (QFX) | **Accepted 2026-09-22** | Nobody | M2.3 is ticked. Deciding is not freezing: the open items before the freeze are Q-19 and Q-20 (Q-18 was answered by ADR-057). |
| **ADR-046, ADR-048, ADR-049, ADR-050** | **Accepted 2026-09-28** | Nobody | Accepted by the owner. |
| **The QFX check narrowing** | **Approved 2026-09-22, retroactively**; widened by ADR-080 on 2026-10-06 | Nobody | Since ADR-080, `check-design.mjs` keeps only its colour-token, type-scale and tracking-scale rules, with no QFX or ceremony exemption; reduce motion and readability as painted still bind. Logged in `GATES.toml`'s change log with the owner's approval. |
| **The product tracks** | Ready where an item names no dependency | Nobody | **P1.1 is done (2026-09-23)** and **P1.2 is done (2026-09-26)** (§4 items 18 and 19). P1.3 to P1.5, P2.2 to P2.5, P4.1, P5.1 and P5.2 can start today, and P6.1 is a decision the owner can take today. **P3 (QQ):** the row above. P1.6 is half built. **P7 (ARQADE):** P7.1 to P7.4 are ticked (P7.4, tips, on 6 October 2026); P7.16 is done: levels, tasks, avatars, ring styles and, since #101 (25 October 2026, in 0.1.17), the themes a level unlocks; P7.17 and P7.18 (ADR-078) are partly built — no grant is paid and there is no abuse watcher; ARQADE's Qt app is docked inside the launcher since 0.1.14 (P7.19, ADR-102), and P7.20 is done: the four solo games rebuilt in QQ play in it (#89 to #92, 24 October 2026); P7.24, the two arenas in QQ, is done (§4 items 149 and 151), and P7.23 with it (item 150). |
| **Open owner questions** | Open | **Owner** | Every question `docs/economics/OPEN_QUESTIONS.md` does not mark answered (OPEN-1 to OPEN-4 and the U-numbers), and Q-19 and Q-20 (the migration inventory). |
| **ADR-110, emulated play in ARQADE** | **Deferred**, 22 October 2026, by the owner | Nobody, until the owner returns to it | Kept for future reference. If the owner takes it up: accept, amend or reject it, and answer its six questions. Nothing is built until it is accepted. |
| **Legal review** | Open | **Owner** | A legal opinion on backing, paid games with prizes, the welcome grant, task rewards, ADR-102's Spark rewards for play, and making CGT exchangeable, before any of them goes live (U-15 to U-18). With it: which KYC, AML, sanctions screening, geofencing and age gating apply, and a public statement that devnet balances never carry into a network with value (`docs/audit/RECONCILIATION.md` §8, 19 October 2026). |
| **Trademark clearance** | Open | **Owner** | An attorney's clearance opinion on the project name (`public-release.name-clearance`). |
| **Vercel plan** | Hobby | **Owner** | Vercel's Hobby plan is for non-commercial use. Before ARQADE takes a real payment or pays out real CGT, it moves to a plan that allows commercial use (ADR-074). |
| **The exposed key** | Open | **Owner** | A recovery phrase was pasted into a chat on 5 October 2026; treat that key as exposed and replace it (`OWNER.md`). |
| **The nine credentials** | Open | **Owner** | Rotate them if those services ever ran (§3, `SECURITY.md`). |
| **Cloudflare clean-up** | Open | **Owner** | Delete the `ci` record and the tunnel left from the retired operations stack. |
| **Launchers 0.1.15 to 0.1.17** | **Published 22, 23 and 25 October 2026** (§4 items 128, 130 and 155) | **Testers**, to install 0.1.15 by hand once and take 0.1.17 through *Update and restart* | 0.1.15 is the first that updates itself; 0.1.16, with `qor://agent`, the first release offered through the channel; 0.1.17, with ARQADE's games in 3D, the newest. The owner's launcher took 0.1.17 through the channel, which ticked L6.2 (§4 item 158). |
| **The QFX backdrop library** | **Six backdrops built** (8 October 2026) | Nobody | The owner's request (§4 item 57): reactive backdrops (Drift, Aurora, Nebula, Lattice, Starfield, Liquid, each reacting to the pointer and richer with the level) are built and chosen in Settings (P2.1). Whether they meet the rest of the request (combinable pointer effects, presets) is the owner's to say at a first look. |
| **Terms of Service, a privacy policy, deleting and exporting an account** | Missing | **Owner**, with a lawyer | QOR ID, Social and ARQADE are live and hold emails, messages and sessions, and none of the active scope has terms or a privacy policy; no route deletes or exports an account for the person who owns it, only an admin ban (RECONCILIATION §8). Needed before inviting more people, and before any public beta. The deletion and export routes are engineering once the policy says what they must do. |
| **The Beta gate** | Far from met (measured 22 October 2026): 8 units met, 35 not, 2 unmeasurable; `beta.chain` 2 of 22 items, `beta.launcher` 3 of 12 | **Owner**, mostly | The economics first: OPEN-1 to OPEN-4, U-4 and U-11 block M6 and M4.4. Then Q-19 and Q-20 and a freeze ADR; names for the indexer and viewer, the SDK, the MCP server and the remote console (AGENTS.md §8); the collective's first members (ADR-021); the testnet's hosting and URL; an external security review; code-signing identities (L6.1). What engineering can do meanwhile is §4 item 99's list. Alpha itself still waits on M3.2 (OPEN-2) and L1.4's last two dialogs. |
| **QOR ID's donation scheme, retired** | **Merged (#54) and deployed, 19 October 2026** | Nobody | Migration 024 ran at QOR ID's redeploy that day (§4 item 100): the trigger that set "staking" bonuses by fiat tier goes, and the tables move whole to a schema `retired`, so nothing that might record a payment is lost. Whether production holds donation rows was not read: the database is the owner's. |

Nothing above waits on another track finishing. L1, L2 and L6 run in parallel with M3, and always could.


1. **Resolved: CI runs** (ADR-063, ADR-064). Until late September 2026 no job in the private repository's workflow
   ever executed: every run failed at startup, and a trivial `probe.yml` failed the same way, which put the cause
   above workflow parsing, at the account. CI now runs on the public `QOR-MATRIX/demiurge-chain`, `probe.yml` is
   deleted, and CI is green on `main`.
2. **L1.7's decisions are applied** (2026-09-14, logged in `GATES.toml`):
   - `framework/` audit: six named advisories exempt until M3.5. **Expired with the directory on
     2026-09-20.** CI now fails if `framework/` returns at all.
   - `qor-auth` audit: RUSTSEC-2023-0071 ignored. CI fails if `rsa` enters the compiled graph.
   - `build-blockchain.yml` removed.
3. **ADR-018 to ADR-032 are confirmed** (owner, 2026-09-17): they record the decisions the owner made, for the
   reasons they made them. **M3 is unblocked and started.** All four inputs arrived with the confirmation:

   | Input | Decided | Record |
   | --- | --- | --- |
   | Precision (U-1) | Eighteen decimals, on both conditions: the stale headroom comment corrected, and a written rule that scaled arithmetic uses the SDK's helpers | ADR-035 |
   | Existential deposit | 100 CGT, `10^20` Sparks; revisitable before mainnet if OPEN-2's sponsorship budget changes the arithmetic | ADR-036 |
   | Pallet names | `pallet-drc369`, `pallet-drc369-royalties`, `pallet-sponsorship`, `pallet-agent-caps`, `pallet-validator-set`. **Checked 2026-09-17: none collides.** All five are unregistered on crates.io, which also rules out an SDK collision, since every SDK pallet is published there | ADR-032, confirmed |
   | `pallet-sudo` | Development and test networks only; the owner holds the key; **absent** from any mainnet runtime, with a removal path written now and a gate criterion of its own | ADR-037, `public-release.no-sudo` |

   Open points that remain the owner's, none of them blocking M3:
   - **The address prefix.** Prefix 42 on development and test networks (the owner, correcting Q-7: the registry is
     archived). ADR-024 reports what was found: no registration process and no successor registry. A forum question
     is drafted there and waits for the owner to post it. The mainnet prefix is a Public Release criterion.
   - **The currency's ticker: settled and done (ADR-034).** On 17 September 2026 the owner decided the name stays
     Creator-God Token and the ticker becomes **CGT**, subject to clearance research. That research found nothing
     that bars it: no live trademark on the string anywhere in any class; no crypto asset on any aggregator or chain,
     so a symbol lookup resolves to nothing; absent from the ss58 registry, Talisman's chain data and icon slots, and
     SubWallet's chain list, all three of which carry `CGT`; no regulatory or exploit history; and no collision with
     anything this system already names. The swap is made: the declared symbol, the launcher's nine hard-coded
     fallbacks and the sixteen prose strings. Identifiers, commands, RPC names and the unit constant deliberately
     still read `cgt`, because M3 replaces them; **R-4 in the migration inventory requires the Substrate
     implementation to write `CGT` into names from the start, with no second rename pass.** U-13 has left
     `OPEN_QUESTIONS.md`, and all of its research is the appendix to ADR-034.
     **Carried consciously:** the project's name, Demiurge, has live trademark exposure in classes 9, 41 and 42 (a US
     application by Demiurge Studios, Inc., past its notice of allowance). It does not touch the ticker and no one
     holds the string in class 36, the class a currency occupies. The screening covered identical marks only, by a
     non-lawyer, so an attorney's clearance opinion on the project name is now a Public Release gate criterion,
     `public-release.name-clearance`.
   - **Linking an address is not chain identification (F-Q9).** Prefix 42 is shared, and the same account bytes are
     valid on every Substrate chain, so ADR-024's prefix check validates a shape and nothing more. The inventory
     records the options; the recommendation is that no route stores an address without proof of possession, over a
     challenge bound to the chain's genesis hash. Not implemented: it lands with the SS58 migration.
   - **Sponsorship (U-4):** the design proposal in `docs/architecture/SPONSORSHIP.md`.
   - **U-10** (voting power) is written up in `OPEN_QUESTIONS.md`, to be decided before OpenGov. It carries a known,
     unresolved tension with ADR-020: nominated proof of stake at mainnet could still be secured by genesis-pool holders.
   - **Settled on 17 September 2026:**
     - the fifth pallet is `pallet-validator-set`, the session manager of ADR-020 (instance name `ValidatorSet`);
     - prefix 42 for development and test networks stands, and the forum draft in ADR-024 is the owner's to post.
   - **Settled on 15 September 2026:**
     - `pallet-assets` for fractionalization only, not a reversal of ADR-031;
     - the Ed25519 forgery test stays;
     - the pin stays on `polkadot-stable2606-1`;
     - dependency versions follow ADR-033.
   - **M2.1** (the owner's review of the inventory) and **M2.3** (the DRC-369 wire format) were both ticked on
     22 September 2026 (items 13 and 14).
4. **L2.2, the development dashboard, is implemented and unit-tested.**
   - The host (`src-tauri/src/gates.rs`) applies `GATES.toml`'s counting rules and nothing else. An
     unreadable signal is not met, never unmeasurable, unless the file says so for that kind.
   - The view is the Gates surface, on the rail and in the Nexus.
   - Running a suite is approved in a host dialog first, because it builds and runs repository code.
   - CI and coverage readings need `gh` installed and signed in.

   **The first of the three checks below is now automated** (2026-09-17). `scripts/check-gates-view.mjs`
   opens the built Gates surface in a real rendering engine, answers `gates_report` with a fixture whose
   numbers are known, and reads back what was drawn: the counts, the passed state, every criterion and
   unit, and that no bar appears while anything is unmeasurable. 34 checks. It never reads `GATES.toml`,
   because a check reading the same file as the host would pass if both were wrong together. It was
   proven to fail first, by altering the report handed to the page. It runs in CI and in `npm run check`.

   What still needs a person at a running launcher:
   - run the `launcher` suite, approving once and declining once;
   - read the CI units with `gh` signed in, and again with it signed out.

   Security item 7, L1.1, L1.2, L1.3 and L1.5 are done.
   **Owner:** the Veridian and Abyss themes lean neon, and the Veridian and Sanguine descriptions are not
   restrained (`docs/design/DESIGN_SYSTEM.md` §2). They are unchanged, pending your call.
   L1.4 (host-side confirmation) is implemented and unit-tested. **Exercised so far:** the owner approved real
   `qor://pay` tips in the host dialog (5 and 6 October 2026), and on 10 October approved and declined a transfer and
   an RPC endpoint change against a local `demiurge-node --dev` (§4 item 72). It stays unchecked until the rest are
   exercised in a running launcher: **the QOR ID endpoint change, approved once and declined once, and a key sign-in
   after QOR ID was unreachable at unlock, which still asks.** Since ADR-016, unlocking also signs in to QOR ID with no
   dialog, and a key with no QOR ID goes straight to claiming a name; exercise both too.
5. **The key scheme and the address format landed on 2026-09-19** (L3.2, half of M3.4; ADR-039).
   - The launcher derives Sr25519 keys with `sp-core` at the chain's pinned version: the first account is
     the bare phrase, further accounts are `//0`, `//1`, … as Talisman enumerates them. Sealing, custody
     and the idle lock are untouched.
   - Addresses are SS58 at prefix 42 everywhere a person sees one; the raw account ID appears only in the
     vault's Derivation panel.
   - QOR ID verifies Sr25519, takes `address` (SS58) or `account_id` (hex), stores 32 bytes, and gives a
     password-only account no chain identity until it proves a key.
   - **What a person should try in a running launcher**, since none of it has been: create a vault, add a
     second account, claim a QOR ID, sign in with the key, and link the key to a password account. Then
     import the same phrase into Talisman or Polkadot.js and check that the first two accounts are the
     same two addresses. That last check is the one thing tests cannot do, and it is what ADR-009 is for.
   - **What is deliberately not done:** the challenge still names no network (ADR-039 point 7, inventory
     F-Q9). The launcher's two state-changing chain calls refused until L3.1, which landed the next day
     (item 9 below).
6. *(Retired on 6 October 2026: work reaches `main` by pull request on `QOR-MATRIX/demiurge-chain`, §7.)*
7. **Owner: Resend on `demiurge.cloud`.** QOR ID sends verification and reset messages through Resend's API,
   tested end to end against a local stand-in.
   - `demiurge.cloud` is verified and authorised in Resend (owner, 2026-09-15). Checked in public DNS the same
     day:
     - DKIM (`resend._domainkey`) resolves and matches;
     - SPF (`send`, TXT and MX) resolves and matches;
     - DMARC (`_dmarc`) is published at `p=none`, monitoring only.
   - Demiurge mail is never sent from `vybz.cloud`: separate projects do not share a sending reputation.
   - The owner set the sending-only key on 2026-09-15, in their user environment. None was created from here.
     A key pasted into chat earlier was revoked. The variables:
     - `RESEND_API_KEY`: the key. Resend refuses it anything but sending;
     - `EMAIL_FROM`: `Demiurge-Cloud <noreply@demiurge.cloud>`;
     - `BASE_URL`: QOR ID's own origin. **Since 1 October 2026 QOR ID is at `https://id.qorsync.dev`** on
       Railway, which holds its settings now; the `demiurge.cloud` subdomain planned here was not used.
   - `RESEND_API_URL` stays unset in any live environment.
   - **QOR ID serves the pages its links open on its own subdomain** (owner, 2026-09-15; recorded as a
     clarification of ADR-015).
     - Vercel serves marketing and the web surfaces only, and does not proxy or rewrite QOR ID's paths.
     - Routing paths through Vercel to QOR ID's host would put a proxy and a rewrite layer between people
       and one-use tokens. (The Fly.io plan of the time is superseded: QOR ID runs on Railway, ADR-063.)
     - `BASE_URL` has no default. Email stays unconfigured unless it is an origin: HTTPS, or HTTP to this
       machine, with no path.
   - **Resend webhooks, set up by the owner.** QOR ID has had its public HTTPS address since 1 October 2026:
     1. In Resend, open Webhooks and add an endpoint: `https://id.qorsync.dev/api/v1/webhooks/resend`.
     2. Select the events `email.bounced`, `email.complained` and `email.suppressed`. Others are recorded and
        ignored, so they are harmless but unnecessary.
     3. Copy the endpoint's signing secret (`whsec_…`) into Railway's settings for `qor-auth` as
        `RESEND_WEBHOOK_SECRET`, never in chat.

     Until then every delivery is refused with 503, and the service logs a warning at start. Whether the
     secret is set today is unconfirmed here; it is listed in `OWNER.md` as still to do.
   - **Live run, 2026-09-15, to Resend's test address `delivered@resend.dev`:**
     - registration sent a verification message, which Resend marked delivered, and its link verified the
       address (a second use was refused);
     - `forgot-password` sent a reset link, and it reset the password (a second use was refused). Resend
       marked it sent for at least two minutes, and delivered when checked again later. It was delivery lag,
       not a failure. Both messages go through the same code path, sender and key. Resend's per-message
       status carries no event times, and no webhook is configured, so the exact lag is not known;
     - the new password signs in and the old one is refused;
     - no link and no key reached the service log.
   - **Still to do:** the same run with a real inbox the owner names, now through the link pages. Receiving
     mail at `demiurge.cloud` is not set up, and is not to be as part of a test (§5). A live bounce report
     cannot be part of it until the webhook is configured.
   - **Templates rewritten; copy approved by the owner as drafted** (2026-09-15).
     - All four messages share one layout in the Architect palette: tables and inline styles, no images,
       declared `color-scheme: dark`.
     - Each message has a plain-text part.
     - Each link is shown in full as well as behind the button.

   The lockout for accounts with an unverified email is closed. An account registered while email is
   configured is sent its link. Any account whose link lapsed or never arrived requests a new one through
   `resend-verification`. Neither needed loosening the reset rules.
8. **Recovery for an account without an email address: decided and built** (owner, 2026-09-15).
   - Such an account adds a verified address through `POST /api/v1/profile/email`, with its password.
   - The address counts for recovery only once confirmed, and the account keeps its backup codes.
   - Changing an address follows the same rules and tells the old address.
   - No existing rule was loosened.
   - The accounts that still cannot be recovered are named in `SECURITY.md` ("Accepted lockouts in QOR
     ID"):
     - no verified email, a forgotten password and no code;
     - a lost inbox with a forgotten password;
     - a lost key on a keypair account.

9. **L3.1 landed on 2026-09-20 (ADR-040), and with it M3.4.** This is the first time anything outside
   `chain/` has used the Substrate chain.
   - The client is `subxt` 0.51, chosen under ADR-033 rule 2 and verified before it was decided: it
     resolves alongside the pinned `sp-core =43.0.0` with a single copy of every shared crate, including
     the same `frame-metadata 23.0.1` the chain builds against, and it depends on no SDK crate, so it
     cannot drag the pin forward.
   - Every call is addressed against the metadata the connected node serves. No generated interface is
     checked in and no metadata file is committed, so a runtime upgrade does not need a launcher release.
   - The launcher declares its own `subxt::Config`. On the day, it had to: the runtime still used
     `IdentityLookup`, so an extrinsic's address was a bare `AccountId32` rather than the `MultiAddress`
     every relay chain uses. That became inventory question **F-Q10**, written up for the owner as
     **Q-17** in [`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md), and **it was
     decided and carried out later the same day** (ADR-041, item 10 below). **So the reason in this
     sentence no longer holds:** every value in that `Config` is now the SDK's own default, and it stays
     as the one place a future divergence would be declared.
   - `subxt`'s `Signer` is deliberately not implemented: it is synchronous and infallible, so a locked
     vault or a declined prompt could not have said so. The client reads the bytes to be signed and hands
     those to the vault, which draws the host dialog and signs inside its own lock.
   - **Proven against `chain/target/release/demiurge-node --dev`:** a fresh vault funded from Alice, an
     approved transfer signed and finalised, a declined transfer that moves nothing and leaves the nonce
     where it was, exact balance arithmetic on both sides (there are no fees yet, OPEN-4), and two
     transfers back to back. It is `cargo test --lib chain::live -- --ignored`, ignored without a node.
   - **What is deliberately still refused:** transaction history, which waits on ADR-028's indexer because
     a Substrate node serves none, and the starter claim, which waits on the economics because the chain
     has no issuance and OPEN-1 and OPEN-2 are undecided. Both say exactly that when called.
   - **`cargo audit` on the launcher: no vulnerability**, and 13 warnings, of which `subxt` brings two.
     `smallstr` (RUSTSEC-2026-0215, unmaintained) arrives through `scale-info-legacy` and `frame-decode`
     and is compiled. `lru` (RUSTSEC-2026-0253, unsound) arrives through `smoldot-light` under
     `subxt-lightclient`, which is an optional feature the launcher does not enable, so it is in the
     lockfile and not in the compiled graph. Neither fails the audit, and **nothing was exempted to make
     that so**: the launcher still has no `audit.toml`.
   - **The live test is not in CI, and is not meant to be yet.** It is ignored without a node, so
     `cargo test` — which is what the `launcher` suite in `GATES.toml` runs — passes with or without one.
     Putting it in CI means building and running a chain node in the job, which is a tightening worth
     making once the node's build cost in CI is known.
   - **What still needs a person:** the native dialog itself. The live test answers the prompt in code.
10. **Q-17, the runtime's address type: decided and done** (2026-09-20). Written up in
    [`docs/architecture/ADDRESS_TYPE.md`](docs/architecture/ADDRESS_TYPE.md), decided by the owner the same
    day, recorded in [ADR-041](docs/decisions/ADR-041-multiaddress-and-accountidlookup.md).
    - **The runtime uses `AccountIdLookup<AccountId, ()>` and an address is a `MultiAddress`**, which is the
      SDK's own `SolochainDefaultConfig` value and every relay chain's. The launcher moved in the same
      commit.
    - **What decided it was a correction, not the original finding.** F-Q10 said `IdentityLookup` was the
      minimal template's default. It is not: both SDK templates derive `SolochainDefaultConfig`, which sets
      `AccountIdLookup` (`substrate/frame/system/src/lib.rs:399`); the prelude that sets `IdentityLookup` is
      `TestDefaultConfig`, whose `AccountId` is `u64`. So the runtime line overrode the SDK's default with no
      reason on record, which is what AGENTS.md §7 asks to be justified.
    - **The write-up's cost estimate was wrong in one row, and says so.** It said `acceptance.rs` needed no
      change. Eight call sites needed it: a balances call's `dest` **argument** is `AccountIdLookupOf<T>`,
      which changes with `Lookup` whether or not an extrinsic address is involved. Corrected in place, struck
      through rather than quietly fixed, because the owner decided on that table.
    - **Evidence:** 51 chain tests with the wasm built, 13 two-validator checks, and the launcher's live test
      against a node built from this runtime. The old shape is refused by the new node, observed rather than
      assumed — the live test's funding helper still passed a bare account on the first run and failed with
      `CannotEncodeCallData(… WrongShape)`.
11. **The landing page: deferred by the owner** (2026-09-17). Do not start it, and do not extend frozen scope.
   - The roadmap has no landing-page item, and there is no gate for one. The nearest item is **M5.4, the public
     viewer**, the shareable page per published work, which depends on M5 and on the provenance source chosen in
     M2. A marketing landing page is not that deliverable.
   - `apps/marketing-site` exists but is **frozen scope** (D-011, amended by ADR-011), so building there would extend
     frozen scope, which is the one thing that must not happen by accident.
   - **If it is wanted it gets a roadmap item and its own decision first.** Until then this entry is the whole of its
     status, so it is deferred rather than undefined.
12. **The six products, 2026-09-21.** The owner named QOR Engine, Qontrol, GNOSIS and QFX, expanded Market
    and Library, and gave music streaming the placeholder name Stream. What exists now:
    - **Six substrate decisions, all Proposed on the day** (ADR-047 and ADR-051 accepted on 22 September, item 13): ADR-046 (the launcher hosts products as separate
      processes), ADR-047 (the object model and the DRC-369 wire format, which would carry out M2.3),
      ADR-048 (interchange formats), ADR-049 (a creator's data under their QOR ID directory), ADR-050
      (where products live, and a gate each) and ADR-051 (QFX's rendering layer).
    - **Seven blueprints** in `docs/blueprints/`: `ECOSYSTEM.md` first, then one per product.
    - **A product track each in `DIRECTION.md` §7 (P1 to P6, 34 items) and a gate each in `GATES.toml`**,
      appended after Public Release. Alpha, Beta and Public Release are unchanged. The launcher's gate
      parser reads `P` headings since `d4be885`.
    - **Two first slices, both inside the launcher, both the owner's scope:** Qontrol's Projects surface
      (P1.1, `5a44207`) and QFX layer one's backdrop (P2.1, `75b2ead`). **P1.1 is not ticked:** the owner's
      scope asked for changes with diffs, and the view and the host's `qontrol_diff` show only which files
      changed. A line-by-line diff that honours `core.autocrlf` and `.gitattributes` is the next Qontrol
      work, ahead of the rest of P1.2.
    - **The owner decided the two foundations** before the build: QOR Engine is a custom Godot build tracking
      upstream, never a hard fork ("QOR Engine, built on Godot"); Qontrol is gitoxide for every read and
      libgit2 for staging only, in a sidecar process, never linked into the host that holds the vault.

    **Found and fixed when the work was finished, after an API failure had stopped the session:**
    - **Ambience Off left the backdrop on screen with the scrim removed**, so text sat outside the contrast
      guarantee, and Off could not be turned back on without a restart. Fixed in `6e5bb28`.
    - **Six of the seven QFX accessibility cases could not fail**: they wrote the expected answer and read it
      back. Replaced by 21 that drive the app and count frame callbacks, proven to fail first.
    - **Qontrol's three proven tests would never have run in CI**: nothing built the helper, and a test
      without it returned early and reported ok. `qontrol-no-skips` makes that a failure, and CI builds,
      lints, tests and audits the helper (`7a3c8ea`). CI also runs `npm run check` now, which it had not,
      so the Projects and contrast checks were never in the workflow.
    - **The drafts described the tree before Phases 4 and 5 landed**, and were corrected before their first
      commit (`df3ccce`, `e998faf`).

    **What needs a person:**
    - **Open the two screens.** They sit behind sign-in, so run the stack as `scripts/run-local-stack.md`
      §4 and §5 describe, and build the helper first (the same file says how). Then **Projects** on the
      rail: New (Code, Music or Game, then "Choose a folder and create") or "Open a folder"; type a message;
      Commit. And **Settings → Accessibility → Ambience**: Live, Still or Off. The backdrop is visible from
      the Gate onwards.
    - **Answer ADR-047 yes or no**, after M2.1.
    - **Confirm or reject the check-design narrowing** (§4.0).
    - **Known divergences from the Proposed records**, not fixed because the records are not accepted:
      the canvas mounts behind the Gate (ADR-051 says never), the budget is one step rather than tiers, and
      there is no pause control; the helper is not bundled, so an installed launcher could not commit
      (P1.2). *(The first three were resolved by the owner on 22 September by amending ADR-051: item 13.)*
13. **The owner's acceptances and decisions, 22 September 2026.**
    - **ADR-047 accepted**, with its three owner questions answered as recommended: an asset is revisable
      until a one-way switch makes it permanent; one singles collection per creator; bytes in a store
      labelled temporary until the Mesh. **M2.3 ticked.** Deciding is not freezing:
      `beta.wire-format-frozen` stays unmet, and GNOSIS's requirements check flags `MaxRoyaltyRecipients`
      (8) as possibly too small before the freeze.
    - **The check-design narrowing approved, retroactively**, with the rule: the design system governs the
      default theme and the chrome; QFX governs the canvas; the default must still pass `check-design.mjs`
      unchanged.
    - **ADR-051 amended and accepted:** only the built-in default ambience may run behind the Gate, never
      an installed theme; the scrim is the guarantee, with the values recorded; the one-step budget is
      accepted for v0 and the tiers are P2.2; Still is the pause. `DESIGN_SYSTEM.md` §1, §6 and §7 and
      `DIRECTION.md` §5 are amended with it.
    - **`--ink-faint` fixed** in all five themes (`320be99`), and the scrim raised from 0.86 to 0.90 so faint
      text also holds 4.5:1 over any backdrop. `check-contrast.mjs` now applies every theme through the app:
      54 checks, proven to fail first against the old tokens and against the old scrim.
    - **Every open question in the six blueprints answered**, each recorded in its blueprint as the owner's
      and reversible. QOR Engine's promise that projects open in stock Godot is P3.6 and a criterion of
      its gate; Market's gate counts U-7, deferred to M6; P6.1 is deferred to M6.
    - **P1.1 stays unticked until in-file diffs exist**, and the diff is the next Qontrol item after M4.1.
    - **Next:** the owner says "acknowledged" to the ten-line summary of the inventory's DRC-369 section,
      which ticks M2.1. Then M4.1, and the first asset minted from a Qontrol project, shown in the launcher.
      No M4 code before that. **Both happened the same day: item 14.**
14. **M4.1: the asset primitive, and the first mint, 22 September 2026.**
    - **M2.1 ticked** against the owner's acknowledgement of the ten-line summary, kept verbatim in the
      inventory. **The eight-recipient royalty bound carried forward** by the owner: ADR-047 open item 1,
      inventory Q-18, and `beta.royalty-recipients`, so the format cannot be frozen with it unexamined.
    - **The chain** (ADR-052, accepted under the delegation): `pallet-nfts` mounted with every value's
      source recorded (ADR-047 for identifiers and strings; Asset Hub Westend at the pinned tag for the other
      bounds, fetched and read), and `pallet-drc369` with only M4.1: the 41-byte content reference, the pinned
      commit, revise, one-way make-permanent, a mint signed to the signer's own singles collection, events
      carrying the reference, and owner enumeration from storage and a runtime API. **A call filter makes it
      the only way an asset is created.** Deposits are placeholders derived from the existential deposit
      (U-14, opened; `public-release.economics-decided` counts it). Weights are placeholders, M7.2 debt.
    - **`chain/scripts/dev-fund.mjs`** sends CGT from Alice on a `--dev` node only, creates nothing, and
      refuses any other chain type.
    - **The launcher:** the object model (manifest, BLAKE3 root, 41-byte reference, temporary content store
      in the launcher's data directory), Projects' Mint panel, and an Inventory read from the chain with Make
      permanent. A declined mint moves, sends and writes nothing.
    - **Found and fixed on the way:** `alpha.chain-tests` could never be met, because four tests kept names
      from the old ticker (`5a0378f`); the Projects surface's controls named classes the stylesheet does not
      define and rendered as browser defaults (`3f67870`).
    - **Found and recorded, not decided:** ADR-047 says an asset's identity is "a function of bytes and
      nothing else" while its decisions 5 and 8 put the commit and branch inside the manifest. The launcher
      follows the decisions; **Q-19** and `beta.manifest-identity` hold it open for a later ADR.
    - **Found and recorded, not fixed:** one QFX case in `check-accessibility.mjs` is intermittent on this
      machine and predates this session's changes (§5). *Since fixed, by item 15.*
15. **Readability, 22 September 2026, from the owner's smoke test.**
    - **The launcher was unreadable with the backdrop live.** The canvas and the scrim were rendered inside
      `#qor-root` beside the interface, and the rule meant to lift the interface above them lifted all three
      together, so the scrim was painted over everything but Nexus. Fixed with `.qfx-interface`
      (`1f5f4ed`).
    - **`check-readability.mjs`** measures every run of text as painted on every screen in every theme, with a
      hostile backdrop and with none; it failed first on every theme, and fails 15 of 15 when the bug is put
      back. It joined `npm run check`.
    - **Three colours raised** by the few lightness points AA needed over a bright backdrop: `--color-bad`,
      and the sanguine and numen accents (`67a083d`).
    - **Also that day, from the same smoke test:** a new vault showed QOR ID's raw refusal instead of asking
      for a name, because the launcher matched two refusals by wording the service had changed (`26f2521`).

16. **Assets as cards, and trading them, 22 September 2026, both asked for by the owner.**
    - **The card shell** (`5d8ad74`): every asset the Inventory reads is a card that leans towards the
      pointer, opens at size on its title, and carries a mark drawn from its own fingerprint until it has a
      preview. In `src/qfx/`, under the carve-out **the owner widened that day** from the backdrop to a
      QFX-owned surface — the same directory, the same two rules, logged in `GATES.toml` with its approval.
      **No sheen**: a sheen is a gradient, gradients are forbidden in every directory including that one, and
      the way to have one is for the single canvas to show *through* the card (ADR-051), which is the next
      slice.
    - **The owner's asks written into the roadmap** (`ebb93f3`), with one correction: the chain stores a
      fingerprint, a commit, a name and an ownership record — **not the files, and not the manifest** — so it
      proves which bytes an asset is and cannot produce them until the Mesh (M8). Said in `PROTOCOL.md` §12.
      An asset may be any bytes, and the media-type table is widened at L4.1. Trade became L4.4/L4.5, Sell
      L4.6, the atomic batch M4.6, and the owner's proposed 15% platform share and USD rail **U-15, open**.
    - **M4.6 and ADR-053**: `pallet-utility` mounted, **only `batch_all` reachable**, `spec_version` 3. Four
      runtime tests, each shown to fail first. A batch cannot smuggle a filtered call: for a signed origin
      the inner calls go through this runtime's own base filter, and only root bypasses it (ADR-037).
    - **L4.4 and L4.5**: an asset's menu on right click *and* from the keyboard, Sell offered and refused
      with its reason, and a trade of up to sixteen assets with an optional public message, a warning screen,
      and then the host's own dialog. 2 host tests and 24 view checks, each proven to fail first; a live
      trade against a development node moved two of three minted assets and the chain was asked afterwards.
    - **The development chain was upgraded in place** with `sudo.setCode` rather than purged, so the owner's
      minted asset survived: `spec_version` 2 to 3, verified by reading it back. The first runtime upgrade
      performed in this repository.
    - **The media-type table was widened** — video, archives, documents, models and fonts are named, a
      project's own formats are still `application/octet-stream` — because the owner asked for "an asset can
      be anything" to be clear now. **Doing it found a wire-format question**: a file's media type is inside
      the hashed manifest, so the table is part of an asset's identity, two clients with different tables
      compute two roots for the same bytes, and the format names no table. **Q-20**, ADR-047 open item 3,
      counted by `beta.media-types`. Nothing minted before is affected — the chain never recomputes a root —
      but re-minting the same project now produces a different one.

17. **The trade window, Sell, and three faults found by running it, 23 September 2026.**
    - **The work was already written when this session started, and none of it had been run.** It was in the
      working tree uncommitted. Running it is what this session did, and all three faults below were invisible
      to reading.
    - **L4.5 reworked at the owner's request**: the trade window is two sides with a lane between them — what
      leaves this account, where it goes — and a mark that crosses the lane **once per change**, never a loop,
      and not drawn at all under reduced motion. A loop would fail `check-design.mjs`, which forbids an endless
      decorative animation in every directory; `src/qfx/`'s carve-out is two rules wide and this is not one of
      them.
    - **"Recently traded with"** (`src-tauri/src/partners.rs`): the accounts this machine has sent assets to,
      written only after the chain finalised a trade, capped at twelve, **local and sent nowhere**. A file that
      cannot be read is an empty list rather than an error, because a convenience must never break a trade;
      the failure to write one is a `warn!` that carries the error and not the address. Reaching someone by
      QOR ID needs a directory lookup that does not exist and has no roadmap item yet, so `label` is carried and always
      empty today.
    - **Half of L4.6, and the item stays unticked** (`src-tauri/src/listings.rs`, `src/views/SellDialog.tsx`).
      Sell drafts a listing — title, kind, that kind's questions, price, description — into a file in the
      launcher's data directory. **Nothing is published**, and the form says so in the product twice: a banner
      before anything is typed, and a line after saving. L4.6 is "a listing other people can see", which still
      needs M4.2, P5.5 and M5.4, so **ticking it would be a lie**. The price goes through `cgt::parse_cgt`:
      exact, excess precision refused rather than rounded, no float (AGENTS.md §5). **No economic value is
      invented** — nothing takes a share, because U-15 is undecided.
    - **The vocabulary is a proposal, and the owner's to confirm** (AGENTS.md §8). Six categories, of which the
      owner named three and said "etc.". It is deliberately not Market's payload-kind axis: a payload kind says
      what a buyer's machine does with the bytes, a category says what the thing is. **Physical (offline)
      carries a warning the others do not**, shown before it is chosen, because the chain moves the record and
      not the object and nothing here holds money, confirms delivery or settles a dispute.
    - **Fault 1: `check-inventory-view.mjs` could not be parsed**, so it had never run. It declared `sent`
      twice and Node refused the file. The `GATES.toml` entry written beside it claimed the run was 110 checks;
      no run had produced that number. Logged as a correction in that file rather than edited away.
    - **Fault 2: the Sell checks passed by doing nothing.** They open an asset's menu after the trade block,
      and used the asset the trade had just sent away — so the card was gone, `?.click()` was a no-op, and no
      dialog opened. Six checks then read an absent dialog. **The `?.` is what made it silent**; those clicks
      no longer use it, and the block uses the asset still held. The real run is **126 checks**.
    - **Fault 3, in the host: three `sort_by` calls failed `clippy -D warnings`.** Fixed to `sort_by_key` with
      `Reverse`, which is the same stable order.
    - **Proven to fail first**, three planted faults, one failure each and the right one each time: a banner
      reading "Your listing is live"; a category appended in the view instead of taken from the host; and a
      form that answered a question the person had not, by filling every field with its first option — that
      last one is the failure the check exists for, and it caught a listing silently claiming "Players: One".
    - **Two roadmap items no gate counted**, found by running §5's gate-coverage check rather than by reading:
      **L4.6** and **P2.8**, both appended to their tracks on 22 September after the gate lists that would have
      counted them. Added to `beta.launcher` and `qfx.roadmap`, logged as a tightening. **This is the third
      time**; the check exists because of the first two. Only `L2.1` is uncounted now, which is deliberate.
    - **Evidence:** 125 host tests, `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean,
      `npm run build` (including `tsc --noEmit`) clean, and `npm run check` 8 / 41 / 34 / 43 / 126 / 54 / 181,
      exit 0, read from the file rather than through a pipe.
    - **Not done, and needing a person or the owner:** nobody has opened the Sell form in a running launcher —
      every check above drives the built page in a headless browser against a fixture host, so the Tauri
      commands `listing_save`, `listing_drafts`, `listing_discard`, `listing_vocabulary` and `trade_partners`
      have unit tests but **have never been called from the real interface**. The trade lane and the partner
      list have not been seen by a person either.

18. **Qontrol's diffs, and P1.1 ticked, 23 September 2026.** The owner's sequencing put the diff next after
    M4.1; it is built, in `src-tauri/src/qontrol/diff.rs`, with the view in `Projects.tsx`.
    - **What it compares:** the index's blob against the working-tree file **converted to what git would
      store**, through gitoxide's filter pipeline in `Mode::ToGit`, under the repository's own
      `core.autocrlf` and `.gitattributes`. That is `git diff`'s comparison. A raw byte comparison would show
      every line of every file changed on a Windows machine that converts line endings, which is why the
      first slice shipped without a diff rather than with that one.
    - **What it returns:** a structured diff per file — lines with their kind and the file's own line
      numbers, or binary with both sizes, or "same", "empty", or not a file (a folder of new files, a
      conflict, a submodule) — not a string the view would parse back apart. A moved file names where it
      came from. Past 4,000 lines it stops and says so.
    - **It never runs a program.** A diff driver's text conversion and an external diff command both run
      whatever configuration names, and Qontrol opens folders from strangers, so neither is applied.
    - **Evidence:** 10 new host tests, 24 for Qontrol and 135 in all, passing under `qontrol-no-skips` with
      the helper built. **Five failed against a planted raw-byte diff** — the autocrlf test (which also asks
      `git diff --numstat` for a second opinion), the `text` attribute test with autocrlf explicitly off, the
      `-diff` test, a NUL byte, and an empty new file — and passed restored. Three of them are now required
      by `qontrol.tests` in `GATES.toml`, logged as a tightening. `check-projects-view.mjs` went from 43 to 67
      checks and **failed against three planted view faults**, one each: the late-answer guard removed (the
      first file's lines drawn under the second file's name), a line drawn with the wrong text, and the diff
      kept after a commit. `check-readability.mjs` measures an open diff in every theme and backdrop, 181 to
      191. `cargo fmt --check` and `clippy --all-targets -D warnings` clean; `npm run check` exit 0.
    - **Found on the way:** `check-readability.mjs` failed its first run on the diff, on a screen-reader-only
      "added: " label — invisible text it measured as painted. The check was not changed (that would be a
      loosening, and the owner's); the rows use `<ins>` and `<del>` instead, which say the same thing to
      assistive technology with no hidden text. And the new Projects checks first failed to parse, on a
      duplicate `const` — the same fault that hid the Inventory check for a day (item 17). Caught by running
      it, not by reading.
    - **Published (the owner asked for a download link to test it):**
      - Launcher **0.1.14**: the version in `package.json`, `tauri.conf.json`, `Cargo.toml`, the lockfiles and the
        launcher's README. It carries ARQADE's app, Sparks, and 0.1.13's profiles, never published before.
      - Built by `npm run app:build` from `6826ce4`. Both staging scripts ran their Qt suites, and both passed.
      - Published as the pre-release `launcher-v0.1.14` at that commit:
        `https://github.com/QOR-MATRIX/demiurge-chain/releases/tag/launcher-v0.1.14`. Its notes cover upgrading from
        0.1.12.
      - The asset `QOR-Launcher_0.1.14_x64-setup.exe` is 78,670,649 bytes, SHA-256 `ee40d17b…659b`. It was downloaded
        back from its public address and matched.
      - The download page (the owner's claude.ai artifact) offers it, with tests for ARQADE, profiles and Sparks in
        place of 0.1.12's account tests.
      - The homepage's links point at it since #48 merged (checked from outside, 18 October 2026).
    - **Not done:** nobody has opened a diff in a running launcher; every check above drives the built page
      against a fixture host, and the host tests drive real repositories. *(The helper was bundled on
      2026-09-26, item 19.)*

19. **Qontrol's git layer, and P1.2 ticked, 26 September 2026.**
    - **Branch** is made at HEAD by gitoxide and not switched to. **Switch** and **discard** are libgit2's
      safe checkout in the `qontrol-git` helper, because gitoxide 0.87 can only check out a whole index into
      a folder. A switch carries uncommitted changes across and refuses, touching nothing, when one would be
      overwritten. A discard puts back a modified or removed file and **refuses a new one**, because that
      would be deleting the only copy.
    - **Per-file commit** stages only the ticked changes, matched by exact name: libgit2's pathspec treats
      `take[1].wav` as a glob even under `DISABLE_PATHSPEC_MATCH`, so the helper decides itself.
    - **The guard** (`src-tauri/src/qontrol/guard.rs`) reads what the helper would stage, under the same
      ignore rules as the commit, and holds back any file over 50 MiB, or named or shaped like a credential,
      until the person says yes to it by name. Its warnings never carry the text that matched, and the host
      runs it again at commit rather than trusting the view.
    - **Bundling:** `npm run app:build` runs `scripts/build-helper.mjs`, which builds the helper, places it
      where Tauri takes a sidecar from, and copies libgit2's licence from the exact vendored source; then it
      applies `src-tauri/tauri.bundle.conf.json`, which declares the sidecar. It is declared there and not in
      `tauri.conf.json` because tauri-build refuses to compile when a declared sidecar is missing, which would
      break `cargo test` on a clean checkout. A Windows installer was built:
      `src-tauri/target/release/bundle/nsis/QOR Launcher_0.1.0_x64-setup.exe`.
    - **Evidence, re-run on 2026-09-26 in a fresh session:** 150 host tests pass under `qontrol-no-skips`
      with the helper built (39 of them Qontrol's), three live ones ignored without a node; the helper's 16
      pass; `cargo fmt --check` and `clippy --all-targets -D warnings` clean. The session that built it
      recorded each new refusal failing first against a planted fault where a single fault can reach it
      (`GATES.toml`, change log of 2026-09-26), and three new tests are required by `qontrol.tests`, logged
      as a tightening. `check-projects-view.mjs` 104, `check-readability.mjs` 231.
    - **Two intermittent view-check failures, found while re-running the evidence (2026-09-26), neither in
      P1.2's code:**
      - **Readability** failed one measurement in two of its first three runs. The second named it: the
        trade warning's "Send them for good" at 1.22:1 in `rgb(6, 7, 10)`, the primary button's text colour,
        which Send never has when settled. React reconciled the compose step into the warning and reused
        Review's button as Send, so Send cross-faded out of the primary look for 200ms, and the check's 250ms
        wait sometimes caught it. **Fixed in the product, not the check:** `TradeDialog.tsx` keys its two
        steps so each mounts fresh. `check-inventory-view.mjs` gained a timing-free check (126 to 127) that
        failed with the keys removed and passed restored; logged in `GATES.toml` as a tightening. Readability
        then passed 231 of 231. The first failure's line was not kept, so it is assumed, not shown, to be the
        same one.
      - **Accessibility** failed "stored off, then live: the backdrop starts" once, counting 3 frames in a
        second where it needs 20, then passed 41 of 41 on the next run. It looks like the headless browser
        being starved of frames under load. **The threshold was not touched** (loosening is the owner's);
        if it recurs on a quiet machine, it is a real fault in turning the backdrop back on.
    - **Not done:** nobody has installed that installer and committed from it; the macOS and Linux bundles
      have not been built; every view check drives the built page against a fixture host. **Next in P1:**
      P1.3, scaffolds completed (Unity and Unreal variants, `qontrol.toml` and the lock set).

20. **Restoring a vault from its recovery phrase, 26 September 2026, asked for by the owner** after losing
    the passphrase to the vault on this machine.
    - **Before this, a lost passphrase had no way back inside the launcher.** The host's `vault_create` took
      any phrase, but the Gate only ever offered a freshly generated one, and `create` refuses when a vault
      exists. The only route was renaming `vault.qor` by hand.
    - **Now:** the Gate's unlock and welcome screens offer *Restore from your recovery phrase*. The host first
      returns the address the typed phrase opens (`vault_preview_phrase`, which stores nothing), so the person
      recognises the account before anything changes. `Vault::restore` then refuses a bad phrase, a weak
      passphrase or an open vault **before** any dialog; asks in a host dialog, naming the set-aside file and
      the account; renames the old file to `vault.qor.set-aside-<date>` (then `-2`, `-3`, …), never deleting
      it; seals the new vault; and puts the old file back if sealing fails. A new error kind, `vault_open`.
    - **Evidence:** five new host tests, 155 in all, fmt and clippy clean. Two were proven to fail first: moving
      the file before the dialog failed the decline test and the refuse-before-asking test, and destroying the
      old file instead of renaming it failed the set-aside test (which checks it is byte-identical and still
      opens with its old passphrase). The view checks all pass unchanged — design 8, accessibility 41, gates
      34, Projects 104, Inventory 127, contrast 54, readability 231. **None of them opened the two new screens;
      since 2026-09-28 `check-vault-gate.mjs` does (item 22)**, but the host dialog has not been seen in a
      running launcher. The putting-back on a failed
      seal is not tested: nothing in a test can make the write fail after the rename.
    - **Why it grants nothing new:** anyone at the machine could already rename the file; nothing in the old
      vault becomes readable, and it stays on disk.
    - **For the owner's own vault:** the file from 22 September already set aside by hand,
      `vault.qor.set-aside-2026-09-22`, is left where it is; a restore today names its file
      `vault.qor.set-aside-2026-09-26`.

21. **Windows Hello unlock, 26 September 2026, the owner's decision (ADR-054).**
    - **Asked for** as "a keyfile stored in the client's directory", because two passwords to log in is
      ridiculous. The second password was QOR ID's fallback sign-in, shown because QOR ID was not running
      (below); ADR-016 already makes the passphrase sign in. Three options were put to the owner with their
      risks — Windows Hello, DPAPI auto-unlock, a plain key file — and **the owner chose Windows Hello**.
    - **Built:** `vault/hello.rs` (the `Presence` interface, its Windows implementation on `windows` 0.62 and a
      test stand-in); `Vault::enable_hello`, `unlock_with_hello`, `disable_hello`, `hello_enabled`; four
      commands; a Settings panel; and the Gate's *Unlock with Windows Hello*, asked once on arrival with the
      passphrase underneath. The copy is bound to `vault.qor` by BLAKE3 and moves aside with it on a restore.
    - **Evidence:** seven new vault tests, 162 host tests in all; fmt and clippy clean. With the binding check
      removed, `a_hello_copy_never_opens_a_replaced_vault` failed; restored, it passes. The one ignored live test
      asked the real Windows on this machine and got `available: true`.
    - **`cargo audit` on the launcher: no vulnerability, 14 warnings**, none from `windows`. The one more than the
      13 recorded on 2026-09-20 is `faster-hex` (RUSTSEC-2026-0306, unsound), which arrives through gitoxide
      (`gix-hash`), not through this change: an advisory published since, against a dependency Qontrol already
      had. Nothing was exempted; whether a newer `gix` clears it is for the next Qontrol session.
    - **Not done:** nobody has turned Hello on or unlocked with it in a running launcher, so the Windows
      implementation's prompts, the determinism it relies on, and whether the prompt opens behind the window
      are unobserved. The helper that would raise the prompt needs `unsafe`, which the launcher forbids; it was
      removed rather than the rule loosened. ~~No view check opens the Hello button or panel~~ — **closed
      2026-09-28 by item 22.**
    - **Also found:** the restore dialog's text (item 20) had runs of spaces where line continuations were lost;
      fixed. Its test only checked that the address appeared.
    - **QOR ID is not running on this machine**, which is why sign-in failed after the restore. Docker Desktop was
      started this session and `qor-smoke-redis` is up; `qor-smoke-pg`, which most likely holds the owner's
      account, cannot bind 55432 because Windows now reserves 55376–55475. Starting a second container on its
      volume was refused by the session's permission check and is waiting on the owner (the reply that turn
      has both routes).

22. **The Gate's restore screens and Windows Hello, checked in a browser, 28 September 2026.** Items 20 and
    21 each recorded that no view check opened their screens.
    - **`scripts/check-vault-gate.mjs`, 64 checks**, joins `npm run check` (so CI's launcher job, once CI runs).
      A stand-in host records every command. It asserts that the typed phrase reaches the host normalised; the
      account it opens is shown before anything is restored, and editing the phrase withdraws it; restore is
      asked once, only after *That is my account*, with that phrase and the passphrase typed; replacing a vault
      says it is set aside, not deleted; with Hello on, Hello is asked once on arrival, a cancel is not re-asked,
      the passphrase stays underneath; no Hello button when Hello is off or the host cannot say; Settings turns
      Hello on only with the passphrase typed, and redraws from the host.
    - **Proven to fail first** against five faults planted in the view one at a time (listed in the script's
      header and in `GATES.toml`'s change log of 2026-09-28, logged as a tightening). Each was caught by its
      own checks; the view was restored byte-identical from a backup afterwards and the check passed again.
    - **`check-readability.mjs` 231 → 251**: the unlock screen with Hello on and just cancelled, and a checked
      recovery phrase, in every theme and backdrop. Its fixture host now answers `vault_hello_status`, so the
      Settings surface it measures includes the Hello panel. All AA.
    - **Its first runs were flaky, and the cause was the check, not the app** (§5): a headless page that draws
      no frames left "Opening the gate…" on screen, and a step could act before the next screen was drawn.
      Fixed by keeping the page in front and waiting for each target; the failing walk now reports where it
      stopped. No assertion was loosened.
    - **Still not done:** Hello and the restore dialog have not been used in a running launcher (items 20, 21).

23. **The vault opens with Windows Hello alone, 28 September 2026, the owner's decision (ADR-055).** The owner:
    every visit ran into "my password or the vault key", and Hello "apparently isn't implemented". It was
    implemented (item 21) but opt-in, behind a Settings panel that itself asked for the passphrase, so every
    arrival was still the passphrase screen; and when QOR ID was unreachable the Gate offered *Use a password
    instead*. Three options were put — Hello only, a silent DPAPI unlock, both — and **the owner chose Hello only.**
    - **The host.** `vault.qor` is now sealed under the Hello key's signature and nothing else (`HelloVault`,
      `"unlock": "windows-hello"`). `Presence::ensure` makes the key only if the Windows account has none, so a
      set-aside vault still opens; ADR-054 replaced it each time. **Sealing signs twice and refuses if the
      signatures differ**, because as the only seal a non-deterministic Hello would make a vault that never opens
      again; ADR-054 assumed determinism and nobody had confirmed it on a device. `unlock`, `restore` and
      `export_phrase` take a `Presence`; `move_to_hello` moves a passphrase vault with its passphrase; a vault with
      an ADR-054 copy bound to it moves with Hello alone. `VaultStatus::Locked` carries `passphrase`. New error
      kind `passphrase_vault`. Commands: `vault_create(phrase)`, `vault_restore(phrase)`, `vault_unlock()`,
      `vault_move_to_hello(passphrase)`, `vault_export_phrase()`, `vault_hello_available()`; the four
      `vault_hello_*` commands are gone. The chain client's three live tests seal with the stand-in.
    - **The Gate.** No passphrase step: create is phrase, confirm, *Seal with Windows Hello* (a button, so the
      prompt follows a click). Unlock asks Hello once on arrival. A passphrase vault goes straight to *Move your
      vault to Windows Hello*. Sign-in is the vault key only; the password form is gone (`qor_login` stays in the
      host, unused). A device without Hello is told before any words are written down. Settings loses the Hello
      panel; *Reveal with Windows Hello* replaces the passphrase field. `declined` now reads "You cancelled, so
      nothing was signed, changed or opened."
    - **Evidence.** 165 host tests pass (vault tests rewritten: 26 in `vault::tests`, including a Hello that signs
      differently each time, a passphrase vault moved with and without its ADR-054 copy, and a copy of another
      file ignored); clippy clean. `check-vault-gate.mjs` rewritten, **84 checks**, proven to fail against four
      planted faults; `check-readability.mjs` 251, all AA; all eight view checks pass. `cargo test ... --ignored`
      confirms Windows reports Hello set up on this PC. Logged in `GATES.toml` as a **scope change**, not a
      tightening, because assertions for removed screens were deleted: **the owner is asked whether that is a
      loosening** (`OWNER.md`).
    - **Found by the check, fixed:** a passphrase vault was sent to the unlock screen, refused at once, and the
      swap to the move screen was lost mid cross-fade in 3 of 5 runs, leaving the person stuck. The locked
      status now names such a vault and the Gate goes straight to the move.
    - **Still not done:** nobody has sealed or opened a vault with the real Hello prompt, or moved the owner's
      own passphrase vault. The prompt may open behind the window (`unsafe` is forbidden, as under ADR-054).
      Other platforms can no longer create a vault.

24. **No lock screen, 28 September 2026, the owner's decision (ADR-056).** After item 23 the owner moved their
    vault to Hello and was stopped by the Gate's sign-in: the shell required a QOR ID session, and QOR ID was not
    running (its default endpoint, `demiurge.cloud`, is not deployed; Docker was not running). They demanded the
    vault and Hello be removed. The keys stay; every prompt and every wait went.
    - **Host.** `vault/keychain.rs`: `KeyStore`, the OS keychain via the existing `keyring` dependency, service
      `cloud.demiurge.qor-launcher`, entry `vault-key:<id>`; an in-memory one for tests; an ignored live test that
      passed against this PC's Credential Manager. `vault.qor` v3 (`"unlock": "os-keychain"`) seals the phrase
      under a random key held there. No idle lock, no `touch`. `unlock` opens a keychain vault with nothing asked,
      moves an ADR-055 Hello vault with one last gesture, and a passphrase vault via `move_to_keychain` (or its
      ADR-054 copy). `export_phrase` asks in a host dialog. `VaultStatus::Locked { sealed_with }`. Commands
      `vault_move_to_keychain`, `qor_sign_in`; `touch_vault`, `vault_hello_available`, `vault_move_to_hello` gone.
    - **Frontend.** The store opens a keychain vault before the first frame; `App` shows the shell once the vault
      is open, session or not. The Gate is first run (one button), restore, and the two one-time moves. Settings
      signs in to QOR ID with the vault key and claims a name. No Lock button, no countdown.
    - **Evidence.** 162 host tests pass; clippy clean; tsc clean; all eight view checks pass (Gate 65, readability
      251). `check-vault-gate.mjs` caught the keychain error reading "The system keychain refused the request."
      and the wording was fixed. **It has not been proven to fail against planted faults**: the first plant made
      every check wait out its timeout and the run was stopped to ship the installer. Logged in `GATES.toml` as a
      scope change; the owner is asked whether it is a loosening. Installer rebuilt 28 September 15:18.
    - **Owed:** the planted-fault proof; a person installing and opening it; the keychain message is shared with
      QOR ID's token errors, whose wording ("the vault's key") fits only the vault.

25. **Onboarding, the intro, and a local stack, 28 September 2026, the owner's asks.** Items 20-24 were committed
    first, on the owner's word (`a3109e2`, `928cab5`, not pushed).
    - **Local stack.** `tools/qor-launcher/scripts/start-local.ps1`: Postgres `qor-local-pg` (54329, volume
      `qor-local-pgdata`) and Redis `qor-local-redis` (56389) in Docker, QOR ID on 8080, `demiurge-node --dev --tmp`
      on 9944; secrets in `%LOCALAPPDATA%\qor-local`, never printed. Running as of this session. Captured output
      makes the caller wait on the servers' inherited pipe (§5).
    - **Launcher.** Agora renamed Social (id `social`; DIRECTION L7.3). `InfoTip` replaces view-header bodies,
      field hints and the accessibility and appearance explanations; kept outside headings so a heading's name
      stays its title (the Projects and Inventory checks caught it inside). `Onboarding.tsx`: the claim bubble and a
      card checking names as typed (350 ms debounce, latest answer only), claiming after `qor_sign_in` so no dialog
      appears. `src/qfx/ceremony/`: `Awaken.tsx` (neochrome halo), `Tutorial.tsx` (six chapters, claims inside
      CGT.md), `Intro.tsx` (2 s splash each open, first-run once per version from `src/assets/first-run/`, reduced
      motion skips both), `ceremony.css`.
    - **The owner's decision:** `check-design.mjs` exempts `src/qfx/ceremony/` from the glow, gradient and looping
      rules only (DESIGN_SYSTEM.md; GATES.toml evidence-rule entry). Proven not to leak.
    - **Evidence.** tsc clean; design 8, accessibility 41 (its known intermittent case failed once of three runs),
      gates 34, Projects 104, Inventory 127, Gate 65, contrast 54, readability 281 (onboarding and tutorial measured
      in every theme). Host unchanged (162).
    - **The owner tested it in the installed launcher and accepted it, 28 September 2026.** Owed: the owner's
      first-run file; not committed.

26. **The owner's rulings of 28 September 2026.** VYB merged into Social and its name dropped (`systems.ts`, SYSTEMS,
    ECOSYSTEM and stream blueprints). ADR-046, ADR-048, ADR-049 and ADR-050 accepted. Q-18 decided, eight royalty
    recipients (ADR-057; `beta.royalty-recipients` now reads met). The two check-scope changes ruled not loosenings
    (GATES.toml). **CI must leave GitHub Actions**, which tried to charge; the platform is not yet chosen, and nothing
    in `.github/workflows` was changed. The readability check's tutorial case now waits for the last chapter to be
    fully drawn (it caught chapter V mid-fade once); 57 of 57 on one theme, twice. Committed on
    `session/realignment-2026-09-21`; new work continues on `session/trading-cards-2026-09-28`.

27. **CI moves to Woodpecker; qorsync.dev is the operations domain, 28 September 2026 (ADR-058, ADR-042).**
    - **Found:** `qorsync.dev` was bought through Vercel, its DNS is Vercel's and verified, and its zone held only
      Vercel's managed defaults (`*` and apex ALIAS to Vercel; CAA for Let's Encrypt, Sectigo, Google). Every name
      resolved and none had a certificate. The Vercel team has **no projects**. `demiurge.cloud`'s DNS is still at
      GoDaddy. Vercel cannot host Woodpecker, QOR ID, Postgres, Redis or a node.
    - **Done:** ADR-042 accepted. `.woodpecker/` holds the six former Actions jobs command for command (chain,
      two-validators nightly, qor-auth with Postgres and Redis services, launcher with its view checks run as a
      non-root user for Chromium, coverage to an agent volume, security as always-run steps). `infra/woodpecker/`
      holds the Fly server config, the PC agent's compose file and a seven-step README. `ci.yml` runs only by
      hand; `probe.yml` deleted. GATES `kinds.ci` now reads Woodpecker (evidence-rule entry, owner-approved).
      AGENTS.md §7's CI sentence updated.
    - **Owed:** the owner's GitHub OAuth app; then `fly deploy`, the `ci` A/AAAA records in Vercel, the agent, and
      trusting the repository for volumes; the gates dashboard's CI and coverage readers rewritten for Woodpecker.
      No pipeline has run, so none of `.woodpecker/` is proven yet.

28. **The operations server on Oracle Cloud Always Free, 28 September 2026 (ADR-059).** The owner asked for a free
    option to ADR-015's $50–70 a month and chose Oracle. `infra/oracle/`: `compose.yaml` (Caddy, QOR ID in
    production mode with no email, Postgres, Redis, Woodpecker server), `Caddyfile` (`id`, `ci`, `ci-grpc`),
    `bootstrap.sh` (Docker, the machine's iptables, secrets made on the machine), `deploy.ps1`,
    `set-github-oauth.ps1` (the secret typed by the owner, sent on SSH standard input), and a README for the
    owner's five steps. ADR-058 decision 1 superseded; `infra/woodpecker/fly.toml` deleted; the agent points at
    `ci-grpc.qorsync.dev:443`. `.gitattributes` pins LF for `infra/`. SSH key `~/.ssh/qor-oracle` made on the
    owner's computer; its public half is in the README. `bash -n` and PowerShell's parser pass; **nothing has run
    against a machine**, because none exists yet.
    - **The public RPC node is withheld on purpose:** `--dev`'s sudo is `//Alice`. Owed: a test chain specification
      with a privately generated sudo key in `chain/`, then the node in the stack and `rpc.qorsync.dev`.

29. **Oracle rejected; the operations stack runs on the owner's computer behind a Cloudflare Tunnel, 28 September
    2026 (ADR-060).** `infra/oracle/` became `infra/ops/`: `compose.yaml` (cloudflared, QOR ID production, Postgres,
    Redis, Woodpecker server and agent, no host ports), `start-ops.ps1` (secrets in `%LOCALAPPDATA%\qor-ops\.env`,
    the tunnel token and GitHub app typed by the owner), README. Caddy, `bootstrap.sh`, `deploy.ps1`,
    `set-github-oauth.ps1` and `infra/woodpecker/agent/` deleted; `~/.ssh/qor-oracle` is unused. `docker compose
    config` accepts the stack; the QOR ID image was built. **Waits on the owner:** Cloudflare account, nameservers,
    tunnel, GitHub OAuth app.

30. **The operations stack is running, 28 September 2026 (ADR-060).** `cloudflared` 2026.9.3 installed by winget
    (its MSI reports 1603 at `RegisterProduct` without elevation, but the binary is installed and on the machine
    PATH). The owner made the Cloudflare account and added `qorsync.dev` (Free); Cloudflare assigned
    `georgia.ns.cloudflare.com` and `porter.ns.cloudflare.com`, which were set at the registrar through Vercel's API
    (`PATCH /v1/registrar/domains/qorsync.dev/nameservers`, team `astramatrix`). Before the switch, Vercel's zone
    held only its own system records (ALIAS, CAA, HTTPS for the apex and `*`) and no MX or TXT; Cloudflare's
    import already answered for the apex, `www` and `id`. `cloudflared tunnel login` done; the owner made the
    GitHub OAuth app and ran `start-ops.ps1`.
    - **Found and fixed:** the owner's terminal predated the install, so `cloudflared` was not on its PATH, the
      script carried on, and wrote `config.yml` with an empty tunnel ID; the container restarted for ever and the
      names answered 525 from Cloudflare's imported records. `start-ops.ps1` now finds cloudflared by its install
      path, throws on an empty ID or a failed DNS route, and rewrites a config with no ID. Re-run: tunnel `qor-ops`
      made, both names routed, four connections registered (sjc).
    - **Measured afterwards:** all six containers up and healthy; `https://id.qorsync.dev/health` 200
      `{"service":"qor-auth","status":"healthy","version":"0.1.0"}`; `https://ci.qorsync.dev/healthz` 204 and `/`
      200.
    - **Step 6 done by the owner:** repository active, webhook on GitHub, Trusted → Volumes only (Network and
      Security were ticked at first and unticked), cron `nightly` `0 3 * * *` on `main`.
    - **CI's first runs, 29 September, manual on the session branch.** Pushes to it start nothing: every pipeline
      runs on push to `main`/`develop`, pull requests to `main`, or manual. Run 1 (`73bc20a`): chain passed
      (55 min cold), QOR ID passed; four faults, none in product code: coverage's loop ended at the first
      unwritten pallet in `GATES.toml`; the fork check matched `GATES.toml`'s own description of it; the
      two-validator script looked in `chain/target` while the build used `CARGO_TARGET_DIR`; the launcher lost
      Docker's DNS mid-download (transient). Fixed in `d50c924` and `d9ba806`. Run 2 (`d9ba806`): chain (89 s
      warm), QOR ID, coverage, security and two validators (13 of 13) passed; the launcher failed at `npm ci`
      with no lockfile, fixed by tracking `tools/qor-launcher/package-lock.json`. Run 3 (`80d72d5`): the same five
      passed; the launcher installed, built and passed the design check, then no browser check could start
      Chromium: "No usable sandbox!". Docker's default seccomp refuses its namespaces as an ordinary user and
      with `chromium-sandbox` too (both measured in `node:24-bookworm`). The browser checks now pass
      `--no-sandbox` only when `QOR_CHECK_NO_SANDBOX=1`, which only `.woodpecker/launcher.yaml` sets; the
      alternative, a privileged step, would need Trusted → Security, which the owner turned off. **The owner chose this option on 2026-09-29.** Locally, without
      the variable, `check-accessibility.mjs` passed 41 of 41. Run 4 (`61ff33c`): chain, QOR ID, coverage,
      security and two validators passed, and so did every launcher frontend check (714 across eight scripts, 0
      failed); the host step then lost Docker's DNS during `apt-get install`, the second time in four runs, both
      in a large download while other pipelines ran. Every `apt-get` in `.woodpecker/` now retries
      (`Acquire::Retries=5`). **Run 5 is owed** for the launcher's host format, lints and tests.
    - Woodpecker reports a temporary `WOODPECKER_GRPC_SECRET` at each start; the agent re-authenticates, so it
      is harmless, but persisting one in `start-ops.ps1` would quiet it. Cloudflare still holds the imported apex and `www` records pointing at Vercel;
      nothing is served there.

31. **M4.2's royalty half: royalties, remix royalties and a sale settled in CGT, 29 September 2026 (ADR-061).**
    The owner's list put real selling next, which needs royalties first. ADR-047 had fixed the fields and bounds and
    left the mechanics undecided; ADR-061 decides them under the engineering delegation and invents no value.
    - **`pallet-drc369`:** `mint` gained `derived_from: Option<(collection, item)>`; the record gained `derived_from`
      and `remix_depth`, checked at mint against `MaxRemixDepth` = 16 (ADR-047); `Minted` carries `derived_from`.
      Errors `UnknownSource`, `RemixTooDeep`. 2 new tests, 15 in all.
    - **`pallet-drc369-royalties`** (new crate, `chain/pallets/drc369-royalties/`, mounted as `Drc369Royalties` at
      index 10): `set_terms` (creator only, while holding it, once; up to 8 recipients, `Permill` shares summing to at
      most one whole, and a remix share), `list`, `unlist`, `buy(max_price)`. `split` is one pure function every amount
      comes from: remix pool to the direct source's recipients pro rata, own shares of what is left, the rest to the
      seller; the parts sum to the price. **One level of remix royalty and immutable terms are product-shaped and the
      owner's to reverse before the freeze.** A part a recipient cannot receive (below the existential deposit, no
      account) refuses the whole sale. No platform share (U-15), no fee (OPEN-4), no deposit of its own (noted
      under U-14). 18 tests.
    - **Faults planted, each caught, then removed** (`diff` against the saved file confirmed the restore): own shares
      taken of the price rather than the remainder (4 tests failed); no check that the seller still holds the asset
      (1); terms settable twice (1); no receivability check (1); upstream payments not subtracted (6); and in
      `pallet-drc369`, the remix depth bound removed (1).
    - **Runtime:** `spec_version` 4, `transaction_version` 2 (the mint's encoding changed). `Nfts::buy_item` joins the
      filter test's refused list; the royalty calls pass the filter; bounds test pins 16 and 8; a runtime test runs a
      sale in CGT. 11 runtime asset tests.
    - **Evidence:** `cargo fmt --all`, `cargo clippy --workspace --all-targets -D warnings` clean; **`cargo test
      --workspace` 95 passed** without `SKIP_WASM_BUILD` (the runtime's wasm rebuilt during the run, 09:35). Release
      node with `sudo`: serves `spec_version` 4, metadata **97,710 bytes** with `Sudo` and `Drc369Royalties`; the
      mainnet shape, built without it: **95,802 bytes**, no `Sudo` (`chain/README.md`). **Two validators: 12 of 13, then
      13 of 13** — the first run reached block 3, not 4, inside the 120-second window and passed everything after; the
      re-run passed all. **Live on a `--dev` node, by a throwaway script since deleted:** Alice minted, set terms
      (Charlie 10%, remix 5%), listed at 1,000 CGT; Bob bought: Alice +900, Charlie +100, Bob −1,000, Bob holds it.
      Dave minted a remix of it (`derivedFrom` recorded, depth 1), listed at 200; Eve bought: Charlie +10 (the remix
      share), Dave +190, Eve −200.
    - **Launcher:** the mint passes `None` for `derived_from`. 162 host tests pass (`--features qontrol-no-skips`),
      and **all three live tests pass against the spec-4 node** (transfer, mint, trade), so reading records and events
      that now carry more fields works. The launcher does not call `list` or `buy` yet: that is L4.6's next half.
    - **Documents:** ADR-061 and both index rows; `PROTOCOL.md` §12–13; `DIRECTION.md` M4.2 (unticked, progress
      recorded) and L4.6; `GATES.toml` (`alpha.pallets` gains the pallet and 14 tests, a tightening logged);
      `OPEN_QUESTIONS.md` U-14 and U-15; `chain/README.md`; `SYSTEMS.md`; `OWNER.md`.
    - **The ticker, on the owner's word the same day: "it's CGT and we are NOT going with DMRG."** The code already
      said CGT (ADR-045). Stale `DMRG` prose was rewritten in `MIGRATION_INVENTORY.md` (R-4 now states CGT, with a
      history note correcting its old claim that the check "looks for `dmrg`"), `DECISIONS.md`'s index descriptions
      and `audit/RECONCILIATION.md`. Left as history: ADR-034 and its index row, ADR files and filenames (never
      edited), and `GATES.toml`'s change log.
    - **The owner's ruling on ADR-061's two choices, the same day (ADR-062):** "the one that ensures the best results
      and least amount of friction for users while also ensuring platform stability." One level of remix royalty is
      kept (the stability choice). Terms that never change is **reversed**: a creator may change their terms whenever
      they hold the asset, never while anyone else does; and once any remix names the asset its remix share may fall
      but not rise (`RemixShareLocked`, from a new `Drc369::RemixCount` written at a remix's mint). `TermsAlreadySet`
      is gone. Kept inside `spec_version` 4, since nothing was committed or deployed. Faults planted and caught: remix
      lock removed (1 test failed), creator-*and*-holder weakened to creator-*or*-holder (1), remix count never
      incremented (1). Pallets 15 and 19 tests. `GATES.toml` swaps the old terms test for its replacement and adds the
      remix-lock test (logged). **Evidence after the change:** clippy clean; `cargo test --workspace` **96 passed** with
      the wasm rebuilt (10:45); metadata 98,230 bytes with `Sudo`, 96,322 without (so the 97,710 / 95,802 above is
      superseded); two validators 13 of 13; and live on a `--dev` node, by a throwaway script since deleted: a
      correction to terms accepted, `RemixCount` 1 after a remix, raising the remix share then refused
      (`RemixShareLocked`), the 1,000 CGT sale paying Charlie 100 and Alice 900, Alice's edit after the sale refused
      (`NotCreator`), and the remix's 200 CGT sale paying Charlie 10 and Dave 190. The machine paused for about
      seventeen minutes mid-run (the node's log jumps from 10:48 to 11:05); nothing failed. The launcher's three live
      tests were not re-run on this last runtime; the mint they build is unchanged by ADR-062.
    - **Not done:** CI run 5 (Docker Desktop was off, `ci.qorsync.dev` answered 530; the owner started it after this
      session's work); the launcher's selling surface; nesting and state and XP.

32. **The repository goes public as `demiurge-chain`; CI back to GitHub Actions; QOR ID moving to Railway, 29 September
    2026 (ADR-063). HALF DONE — the owner has three steps, then the assistant has five (below).**
    *Since 1 October 2026 this is done: the repository is `QOR-MATRIX/demiurge-chain` (ADR-064), CI runs there and
    is green, QOR ID is live on Railway at `https://id.qorsync.dev`, and work reaches `main` by pull request. The
    owner steps and the branch names below are history.*
    The owner travels, so nothing may depend on their computer. The owner chose Railway for what must stay online and
    GitHub Actions for CI, with the repository public so Actions is free, published **without history** (option 1).
    - **Why not just flip the old repository public:** old credentials remain in its history, nine values in two
      tracked files were never rotated (`SECURITY.md`), and the deleted pre-realignment docs are in its history.
    - **Done — the public repository.** `ALaustrup/demiurge-chain`, public, default branch `main`: `e611c99` (the
      import: the private tree at `d9bb19a` less `docker/n8n/docker-compose.yml` and
      `docker/docker-compose.testnet.yml`) and `2681a51` (CI and records). Before publishing, gitleaks 8 (installed by
      winget) scanned the whole tree: 6 findings, all false positives (empty keys in `.env.example`, where gitleaks
      flagged the neighbouring region and index names; a storage-key name; a `YOUR_TOKEN` placeholder; a `0123…` test
      token; a deliberately wrong test secret). Verified after the push: public, one root commit, neither credential
      file in `git ls-files`. **The owner ran the first commit, `gh repo create` and the first push by hand**: Claude
      Code's auto-mode classifier refused them twice, even on the owner's explicit instruction. The owner's `printf`
      to `.gitignore` did not take, so `2681a51` adds the two ignore entries (checked with `git check-ignore`).
    - **Local clone:** now on branch **`public-main`**, tracking **`public/main`** (remote `public` =
      `demiurge-chain`). Remote `origin` is still the private archive `demiurge-cloud`; its branches, including
      `session/trading-cards-2026-09-28` at `d9bb19a`, are untouched. **Push new work to `public`, never `origin`.**
      The two credential files are still on disk, ignored. `archive-7gUg78/` (a GitKraken download) is untracked.
    - **Done — CI.** `.github/workflows/ci.yml` runs on push and pull request to `main`, nightly (two validators) and
      by hand, keeps the name "Pleroma CI" so the gates' `workflow` fields still match, and carries Woodpecker's fixes
      (coverage records a pallet not yet written as absent; apt retries). Chromium keeps its sandbox
      (`sysctl kernel.apparmor_restrict_unprivileged_userns=0` on the runner; `QOR_CHECK_NO_SANDBOX` is not set).
      `.woodpecker/` and `infra/woodpecker/` deleted. `GATES.toml` `kinds.ci` reads Actions again (evidence-rule entry,
      owner-approved). **The first run, `36617755694`, did not start: "your account is locked due to a billing
      issue".** That is also what every `startup_failure` since 21 September was. **It is not the workflow.**
    - **Done — Railway** (MCP connector, account `alaustrup`). Project **`demiurge`**
      `449a0842-d46d-416f-bf4c-c418f72c1d79`, environment `production` `7dfe7274-8af4-4202-aafb-38b475b9d123`, region
      `iad`. **`Postgres`** `4986dae9-7cd8-45ea-9a39-1ea3391cf690` and **`Redis`** `f5d7f670-73e2-4a8b-bfc2-e23573696047`
      from Railway's templates (passwords generated by Railway), both deployed SUCCESS. **`qor-auth`**
      `10e620ca-030e-40d4-9eca-d5c09d2ff78c` is **configured, with no source attached and never built**: root
      `/services/qor-auth`, Dockerfile, healthcheck `/ready` 120 s, restart on failure ×3, watch `/services/qor-auth/**`,
      and every non-secret variable (`DEPLOY-RAILWAY.md` lists them). **A temporary TCP proxy on Postgres**,
      `tokaido.proxy.rlwy.net:30413` (proxy `14c665a7-dd5a-4ed7-91d4-f20ea47a2e48`), exists only for the data copy and
      must be removed after it. Railway has deprecated `railway.json`, so it was deleted and the settings recorded in
      `DEPLOY-RAILWAY.md`. The owner's other Railway project, `fractal-node-room`, was not touched.
    - **Data.** The operations stack's QOR ID held **0 users**; the launcher's local one (`qor-local-pg`, started and
      stopped again for this) held **1, `godmode`**, with a chain account and 18 migrations. Its dump is
      `%LOCALAPPDATA%\qor-ops\qor_auth.dump`. The PC's operations QOR ID was stopped for a dump and restarted; the PC
      stack (`qor-ops-*`) is still running and still serves `id.qorsync.dev` and `ci.qorsync.dev` through the tunnel.
    - **Waiting on the owner, in any order:**
      1. **Unlock the GitHub account:** Settings → Billing and plans (a failed payment or a balance). Then re-run CI
         (`gh run rerun 36617755694 -R ALaustrup/demiurge-chain`, or push).
      2. **Paste QOR ID's two JWT secrets** from `%LOCALAPPDATA%\qor-ops\railway-qor-auth-secrets.txt` into Railway →
         `demiurge` → `qor-auth` → Variables, then delete the file. Generated locally and never printed or sent through
         the chat (`DEPLOY-RAILWAY.md`'s rule).
      3. **Run `& "$env:LOCALAPPDATA\qor-ops\restore-to-railway.ps1"`** in PowerShell. It asks for Railway's
         `POSTGRES_PASSWORD` (Postgres → Variables) and should print "1 user(s), 18 migrations". It mounts the folder
         into a `postgres:18` container, so it works in Windows PowerShell 5.1 too.
    - **Then the assistant, in order:** (a) `connect-service-source` on `qor-auth` with repo `ALaustrup/demiurge-chain`,
      branch `main`; watch its first Railway build; check the logs show the migrations already applied and `/ready`
      answers. (b) Delete the Postgres TCP proxy. (c) `generate-domain` `id.qorsync.dev` on `qor-auth` and give the owner
      the DNS record: in Cloudflare the `id` record is the tunnel's today, so the tunnel route is removed and the record
      replaced (the assistant can do it only once the Cloudflare connector is authorised through `/mcp`); wait for the
      certificate (`.dev` is HSTS-preloaded) and check `https://id.qorsync.dev/ready`. (d) Stop the PC's operations
      stack, delete `ci.qorsync.dev`'s record and the tunnel, and delete `infra/ops/` in a commit. (e) Agree a Railway
      usage limit with the owner (expected $5–15 a month) and rewrite `OWNER.md`, `SYSTEMS.md` and this file.
    - **Also owed, done 30 September:**
      - **The gates dashboard read the wrong repository.** It called `gh` without `-R`, and with two remotes `gh`
        picked `origin`: measured, `gh run list` in this clone returned a `demiurge-cloud` run. `[ci]` in
        `GATES.toml` now names `repository = "ALaustrup/demiurge-chain"` and every `gh` call passes it (run list, the
        branch head through `repos/<repository>/branches/<branch>`, the coverage download); a `[ci]` without it fails
        the reading instead of guessing. Logged as a tightening in `GATES.toml`'s change log; pinned by
        `ci_is_read_from_the_named_repository_only`. Checked against the live API: it now reads run `36695500285` on
        `4023451`, not met, the billing lock.
      - **`docs/architecture/HOSTING.md`** now says what ADR-063 changed (QOR ID, Postgres and Redis on Railway;
        CI free; ADR-042 accepted) and leaves the nodes, the indexer and the archive on ADR-015's Fly plan, which
        ADR-063 did not reopen. `docs/README.md`'s row for it said ADR-042 was Proposed; fixed.
      - **The launcher's default QOR ID address was `https://demiurge.cloud/api/v1`, which answers 404** and never
        served QOR ID. It is now `https://id.qorsync.dev/api/v1` (ADR-042, ADR-063), and so is the Settings
        placeholder. A saved `settings.json` still wins, and the owner's points at `127.0.0.1:8080`. The chain
        default `wss://rpc.demiurge.cloud` is unchanged: public RPC is withheld (ADR-063 decision 6) and its name is
        a HOSTING §2 proposal.
      - Launcher: fmt, clippy `-D warnings`, **163 host tests** (one new), frontend build and `npm run check` (281
        view checks) all pass.
    - **Found 30 September:** Docker Desktop was off, so the PC stack (`qor-ops-*`) and its tunnel were down and
      `id.qorsync.dev` answered **530**. It was not restarted: the stack is being retired, and Railway is the fix.
      Owner steps 1 and 2 were not done (the scheduled run `36695500285` at 09:20 UTC was refused for billing; the
      secrets file is still on disk); step 3 could not be checked, since this session had no Railway connector.
      The Railway steps (a) to (c) need that connector back.
    - **Checked 1 October, with the Railway connector back:** `Postgres` and `Redis` online; `qor-auth` offline, no
      source, no deployment, and **no JWT secret among its variable names** (read with `describe-service`, which
      returns names only; `list-variables` returns values in plaintext and was not called). The secrets file is still
      on disk, so owner step 2 is not done. Step 3 was not verified: checking needs the Postgres password. The TCP
      proxy `14c665a7…` is still ACTIVE, as the restore needs. CI: three runs, all refused, none since
      `36695500285`. **Step (a) was deliberately not started:** a first deploy would run the 18 migrations into the
      empty database, and the restore must go in first.
    - **Later on 1 October: the owner did steps 2 and 3; step (a) ran and the service does not start.** Both JWT
      secret names are among `qor-auth`'s variables, and the owner reported the restore printed "1 user(s), 18
      migrations". The source is attached (`ALaustrup/demiurge-chain`, `main`). Deployment `d1f3297f` at `4023451`:
      **the image built** (`cargo build --release --locked`, 1m 03s), then the container exited four times on
      `Error: migration 1 was previously applied but has been modified` and the `/ready` healthcheck failed.
      **Cause, measured, all 18 migrations:** the checksum in the dump equals the SHA-384 of the file with CRLF
      endings (the working tree, `core.autocrlf=true`, a Windows build) and never the LF form git holds and Railway
      builds. The SQL is unchanged. **Not fixed:** correcting the 18 checksums is a write to Railway's Postgres; the
      auto-mode classifier refused the command that generates the `UPDATE`s, so it waits on the owner's say.
      `%LOCALAPPDATA%\qor-ops\fix-migration-checksums.ps1` exists and its `.sql` does not, so it refuses to run.
      The secrets file is still on disk. The TCP proxy stays until the fix is in. Steps (b) to (e) not started.
      **Fixed the same day, and QOR ID RUNS ON RAILWAY:** the owner approved it; `fix-migration-checksums.sql` (18
      `UPDATE`s, each guarded by the CRLF checksum) was applied by the owner through the proxy, and the owner
      redeployed from the dashboard (the classifier refuses `redeploy` from here). Deployment `f0658467` is SUCCESS;
      its log reads "Database migrations applied", "Connected to Redis", "Listening on 0.0.0.0:8080". The owner's
      pasted run showed 18 × `UPDATE 0`, which is the script's second run; the first run's output was not seen, and
      the service starting is the evidence the checksums are right. **Email is not configured there**
      (`RESEND_API_KEY`, `EMAIL_FROM`, `BASE_URL`, `RESEND_WEBHOOK_SECRET` unset; the log warns), so no mail is sent.
      **Step (b) NOT done:** the owner declined the proxy's deletion when asked, so `tokaido.proxy.rlwy.net:30413`
      is still open to the internet, behind the Postgres password. **Step (c) NOT done:** the classifier refuses
      `generate-domain` ("DNS / Domain / Cert Changes"), so the owner adds `id.qorsync.dev` in the dashboard (port
      8080) and enters Railway's record in Cloudflare. The service has no public address yet, so `/ready` has been
      checked only by Railway's healthcheck. (d) and (e) not started.
      **Step (c) DONE later that day, by the owner's hands:** custom domain `id.qorsync.dev` → port 8080 on
      `qor-auth` (domain `711a8beb…`, certificate VALID), and in Cloudflare the `id` record is now a DNS-only CNAME
      to `esk0jahg.up.railway.app`. Measured from this PC: it resolves so at 1.1.1.1, and
      `https://id.qorsync.dev/ready` answers **200** `{"checks":{"cache":"ok","database":"ok"},"status":"ready"}`
      with `server: railway-hikari`; `/health` 200. (Railway's own `domain-status` still read "requires update"
      at that moment; it lags.) Sign-in with the restored account was not exercised from the launcher.
      **Step (d), mostly done on the owner's instruction:** `docker compose -p qor-ops down` removed the six
      `qor-ops-*` containers and the network; **the four `qor-ops_*` volumes are kept** (delete them when nothing in
      them is wanted). No scheduled task or startup entry restarts it. `infra/ops/` is `git rm`'d, **staged and not
      committed**. `id.qorsync.dev/ready` still 200 afterwards; `ci.qorsync.dev` answers 530. **Left for the owner
      in Cloudflare:** delete the `ci` DNS record and the tunnel (Zero Trust → Networks → Tunnels); no connector.
      **Email, half:** `EMAIL_FROM` and `BASE_URL=https://id.qorsync.dev` set on `qor-auth` with `skipDeploys`, so
      they take effect at the next deploy. The owner pastes `RESEND_API_KEY` (which redeploys), then adds the
      Resend webhook `https://id.qorsync.dev/api/v1/webhooks/resend` (bounced, complained, suppressed) and pastes
      its `whsec_…` as `RESEND_WEBHOOK_SECRET`. Not yet done, and no mail has been sent from Railway.
      **Step (b) STILL not done:** `delete-tcp-proxy` was cancelled at the approval prompt twice more.
      **Step (e):** the owner chose a **$10 a month** cap; it is set in Railway's dashboard (Workspace → Usage →
      limits), which no tool here reaches. **The secrets file** was emptied by the owner, not deleted.
      **Checked again later on 1 October:** **step (b) is DONE** — the owner removed the proxy, `list-tcp-proxies`
      on Postgres returns none. **`RESEND_API_KEY` is set** and deployment `87b5ae96` (SUCCESS) logs "Email service
      configured"; the only warning left is `RESEND_WEBHOOK_SECRET`. **No message has been sent from Railway**, so
      live delivery there is unproven. `/ready` 200. **GitHub is still locked:** `gh run rerun 36695500285` created
      six jobs, each refused in 3 s, "your account is locked due to a billing issue". `ci.qorsync.dev` still
      resolves, so the Cloudflare leftovers are not deleted. Whether the $10 cap is set is unknown from here.
      **Committed and pushed on the owner's word: `7e154e1` on `public/main`.** `/ready` 200 after Railway's rebuild.
      **Live mail from Railway is proven to Resend's test address:** `POST /api/v1/auth/register` as
      `mailtest_20261001` with `delivered@resend.dev` answered 201, and Resend lists "Confirm your email address for
      QOR ID" to that address as **delivered** (09:16 UTC, email `01a0f6bf…`). The link was not opened. **That test
      account now exists in the live database**, with a random password nobody kept; delete it or leave it. A real
      inbox is still untried. **Seen in the same Resend listing, not investigated:** two verification messages
      **bounced** on 22 September, to `hasemail@` and `withemail@example.invalid` — some local test run sent real
      mail through the owner's key. Bounces cost the sending domain reputation; find the run and point it at the
      stand-in (`RESEND_API_URL`) or unset the key for it.
    - **CI RUNS: the repository moved to the organisation `QOR-MATRIX` (ADR-064, 1 October).** The owner disputes
      the charge behind the personal account's lock and will not pay it, created the organisation, and by hand
      (`gh repo create QOR-MATRIX/demiurge-chain --public`, `git push matrix public-main:main`) pushed `7e154e1`
      there. **The owner saw its first run in progress**, the first job of this workflow ever to execute. **Nobody
      has seen a run finish**, and its Linux-only steps had never run before, so expect faults.
      - **The classifier refuses the assistant everything about that repository**: creating it, pushing, even
        `gh run list -R QOR-MATRIX/demiurge-chain` ("Data Exfiltration"), and `git remote set-url` ("Remote
        Repoint"). So the owner reads CI and runs the git commands, until they allow them in Claude Code's settings.
      - **Remotes, unchanged by the assistant:** `matrix` = `QOR-MATRIX/demiurge-chain` (push here now);
        `public` = `ALaustrup/demiurge-chain` (no longer pushed to; the owner archives it); `origin` = the private
        archive. `public-main` still tracks `public/main`.
      - **Changed in the tree, uncommitted:** ADR-064 and both indexes; `GATES.toml` `[ci].repository` and
        `kinds.ci` with an evidence-rule entry; the gates test's expected name; the README badge, the issue-template
        links, both `Cargo.toml` `repository` fields, `ci.yml`'s comment, `DIRECTION.md`, `HOSTING.md`,
        `DEPLOY-RAILWAY.md`, `SYSTEMS.md`, `OWNER.md`.
      - **Railway still deploys from `ALaustrup/demiurge-chain`.** Its GitHub app must be given the organisation
        (owner), then `connect-service-source` to `QOR-MATRIX/demiurge-chain`. Until then a push redeploys nothing,
        and `DEPLOY-RAILWAY.md` already names the new source: it is ahead of the service.
      - **Committed `34f21c1`, pushed to `matrix` by the owner.** Its CI run's result is not known yet.
      - **Step (e) closed by the owner: no Railway usage cap.** The $10 figure given earlier that day is withdrawn
        ("I am not capping railway"). Do not set one, and do not raise it again unless the bill does.
    - **Three agents, 1 October, on the owner's instruction** (chain M4.5 nesting; the launcher's Sell and Buy on
      chain; QOR ID's two faults). The QOR ID one is done and spot-checked by the lead:
      - **Line endings: closed.** `.gitattributes` pins `services/qor-auth/migrations/*.sql` to LF (`i/lf w/lf`,
        checked); `src/migration_hygiene.rs` fails the tests on a carriage return in a migration, on disk or
        embedded, seen failing first. `qor-local-pg` was corrected with
        `services/qor-auth/scripts/correct-migration-checksums.ps1` (18 rows; a second run changes nothing) and
        `target/release/qor-auth.exe` rebuilt. Any other local database migrated on Windows before today needs the
        script once (`scripts/run-local-stack.md` §4), then a rebuild. This closes the "Trap for later" above.
      - **The two bounced emails of 22 September: cause found and closed.** `EmailConfig::default()` read the
        environment, seven test helpers used it, and this PC has the real `RESEND_API_KEY`, `EMAIL_FROM` and
        `BASE_URL` set, so `registering_with_an_email_leaves_it_unverified` and
        `an_account_with_an_email_address_is_refused` sent real mail. `Default` is gone, only `main` calls
        `EmailConfig::from_env()`, a test build refuses any API base not on this machine, and two guard tests pin it.
        **Still open:** a *running* local QOR ID started with the ambient key and no `RESEND_API_URL` sends whatever
        the e2e scripts register; `start-local.ps1` inherits the user environment. Owner's call: clear the key for
        that child, or take it out of the user environment now that Railway holds it.
      - **130 tests** (125 + 5) with Postgres, Redis and `--include-ignored`; fmt and clippy clean; log and SQL
        hygiene pass. The five are required by `alpha.qor-auth-tests` (tightening, logged). `migration_hygiene` is a
        new test-only module name, after `sql_hygiene` and `log_hygiene`: AGENTS §8 asks the owner about names.
        Not yet run in CI.
      - **Chain: nesting and R-2 built (ADR-065), `spec_version` 5.** `Drc369::nest` / `unnest` in
        `pallet-drc369`, which is now `pallet-nfts`'s `Locker`: a nested asset and the asset holding it cannot be
        transferred, sold or burned. Depth 8, 64 children (ADR-047). **109 workspace tests with the wasm built**
        (was 96), fmt and clippy clean, as the agent reported; the lead did not re-run the suite. 13 new tests, each
        seen to fail first, are required by `alpha.pallets` and `alpha.chain-tests` (tightening, logged).
        **M4.2 and M4.5 stay unticked: state and XP are not built.** ADR-065 records twelve choices; **1, 2 and 5
        are product choices for the owner** (a parent holding assets cannot move or be nested; a nested asset can
        be listed but not bought). **Not done:** the release node was not rebuilt, so the running
        `demiurge-node.exe` still serves `spec_version` 4 and nesting has never run on a live node;
        `chain/README.md`'s metadata sizes were not re-measured; the launcher has no nesting surface and was not
        built against the new metadata.
      - **Launcher: Sell publishes on chain and Buy settles (L4.6, still unticked).** `src-tauri/src/chain/sales.rs`
        and five commands (`drc369_sale`, `_sale_preview`, `_list`, `_unlist`, `_buy`), each through the host
        dialog; `BuyDialog.tsx`, `SaleBreakdown.tsx`, a rewritten `SellDialog.tsx`. Buying is by an asset's number,
        inside Inventory: there is no catalogue, because there is no indexer. As the agent reported: **182 host
        tests** (was 163), fmt and clippy clean, **5 live tests** against the spec-4 node including
        `a_sale_pays_every_part_and_hands_the_asset_over` (a remix sold at 2,000 CGT + 7 Sparks, every part exact to
        the Spark, declined prompts moving nothing), and `npm run check` exit 0 **on its second run** — Inventory
        238, readability 341, the rest unchanged. The first run failed on the accessibility case "stored off, then
        live: the backdrop starts", the known intermittent one.
        - **Owner's decisions owed:** (1) **`check-readability.mjs`'s measuring rule changed** — text clipped inside
          a scrolling dialog no longer counts as visible. Logged in `GATES.toml` as evidence-rule, **NOT YET
          APPROVED**. (2) The module name `chain/sales.rs` and the five command names (AGENTS §8). (3) Buying by
          asset number as the surface.
        - **Known weaknesses (the first two removed on 2 October, items 33 and 34):** the payout preview is a **second copy of the pallet's `split`** in the host
          (`sp-arithmetic =28.0.1`), pinned by the pallet's vectors and the live `Sold` event; a runtime API would
          remove it. **A revision landing between the dialog and the block is not caught**: the host compares
          fingerprints, the chain's `buy` has only `max_price`. One view check (the re-read after a purchase)
          cannot fail, and the README says so. `market.md` decision 9 allows a zero price and the pallet refuses
          it. Royalty terms and remix mints have no launcher surface. No person has used any of it.
        - **Traps, to move to §5:** a view check waiting for "the chain was asked again" passes on an unrelated
          re-read (Inventory reloads when the active account object is replaced, which may also flicker in the
          real app); `cargo test --lib chain::live -- --ignored` also runs `vault::keychain::live`.
      - **A devnet plan exists, undecided: `docs/architecture/DEVNET_PLAN.md`** (an agent's research, 1 October). It
        recommends Railway with two validators and a separate RPC node, which needs an ADR superseding ADR-015 for
        the nodes; cost is NOT measured (an illustration of about $11-22 a month from Railway's unit prices).
        **Missing in `chain/node/`, read in the code:** no specification without the well-known Alice/Bob keys, no
        `key` subcommand mounted, no container image for `chain/` (the root `Dockerfile` and `fly.toml` still
        describe the deleted `framework/`), no first-boot key script. **The hard question is the owner's:** a
        non-dev genesis with no CGT means nobody can mint and the sudo key cannot sign (it needs the existential
        deposit), and AGENTS §5 forbids creating CGT outside `--dev`. Two validators also stop finalising whenever
        either restarts. Nine owner decisions are listed in the plan.
      - **Launcher 0.1.5 built, on the owner's instruction** (was 0.1.0 everywhere, the installed copy included):
        `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `Cargo.lock` and `tauri.conf.json`.
        `npm run app:build` produced `src-tauri/target/release/bundle/msi/QOR Launcher_0.1.5_x64_en-US.msi` and
        `bundle/nsis/QOR Launcher_0.1.5_x64-setup.exe`, **unsigned** (no code-signing identity yet, L6), carrying
        the selling surface. Not installed and not run by anyone; no check was run on the packaged build.
      - **Integration, done:** the release node was rebuilt at `spec_version` 5 (2m 34s) and restarted on a fresh
        `--dev` chain, and the launcher's five live tests passed against it (378 s), the sale among them
        (`%LOCALAPPDATA%\qor-ops\integration-spec5.log`). So the launcher's sale works on the runtime that has
        nesting. Nesting itself has still not been called on a live node. A `--dev` node keeps its chain in a
        temporary directory, so every restart is a new chain.
      **Trap for later:** nothing pins these files' line endings, so a Windows build and a Linux build of QOR ID
      cannot share a database; a `.gitattributes` `eol=lf` on `services/qor-auth/migrations/*.sql` would end it, and
      would in turn need the same checksum correction on the launcher's local database (`qor-local-pg`).

33. **2 October 2026: unrecorded, half-finished work found in the tree, finished and verified. UNCOMMITTED.** *(Since
    committed; it is on `main`.)* The
    owner said "proceed". The working tree held a previous session's changes in three areas that `HANDOFF.md` did not
    mention, cut off mid-work. What they are, what was wrong, and what was run:
    - **Chain, `spec_version` 6.** `Drc369Royalties::buy_exact(collection, item, max_price, content)`, call index 4:
      `buy` that also refuses with `ContentChanged` if the asset's `current` reference is not `content`. `buy` keeps
      its index and encoding (pinned in `runtime/tests/assets.rs`), so `transaction_version` stays 2. And the runtime
      API `Drc369RoyaltiesApi::sale_preview(collection, item, price, buyer)` — the parts through the same function
      `buy` uses, and `refusal`: with a buyer the real settlement is run inside `with_transaction` and rolled back;
      without one, zero price, unreceivable parts and `ItemLocked` only. **Wrong when found:** one clippy error in
      `runtime/tests/assets.rs` (redundant closure). **Run:** `cargo fmt --check`, `clippy --workspace --all-targets
      -D warnings`, **`cargo test --workspace` 118 passed** without `SKIP_WASM_BUILD` (was 109: royalties 26, runtime
      asset tests 16). Release node rebuilt (no `sudo` feature) and **running now** as `--dev --tmp` (pid 33028 at
      the time, log `%LOCALAPPDATA%\qor-ops\node-spec6.log`); it reports `spec_version` 6 and serves **99,901
      bytes** of metadata without `Sudo` (the mainnet shape; the `sudo` shape was not re-measured). `PROTOCOL.md`
      and `chain/README.md` updated.
    - **Launcher.** `chain/market.rs` (787 lines) and the command `drc369_market`: every listing read by walking
      `Drc369Royalties::Listings` at the finalised block, paged, bounded, details only for the window asked
      (L7.2's host half). Buy now knows nesting (`Drc369::ParentOf` / `ChildCount`; `ItemLocked` in words).
      **Wrong when found:** `market.rs` never formatted, and **four user-facing sentences in `sales.rs` broken** — a
      lost `\` continuation left a run of spaces inside each string. Likely cause: **a heredoc through the Bash tool
      turns `\` + newline into nothing**; it happened again in this session and was fixed with Edit. Trap for §5:
      never write Rust string continuations through a shell heredoc. **Added this session:** Buy sends **`buy_exact`**
      with the content reference that was on screen (`ContentRefArg::try_from(&ContentView)` in `assets.rs`, refusing
      an unknown algorithm or a root that is not 32 bytes), and `ContentChanged` is put in words; this closes the
      "revision landing between the dialog and the block" weakness in item 32. **Run:** fmt, clippy `-D warnings`,
      **193 host tests** (was 182), and **the five live tests against the spec-6 node, all passed (377 s,
      `%LOCALAPPDATA%\qor-ops\integration-spec6.log`)**, the sale through `buy_exact`. A launcher now needs a node at
      spec 6 or later to buy. **Not done:** no Market screen (no `.tsx` change, nothing in `ipc.ts`);
      `drc369_market` has never run against a node; the host still carries its own copy of `split` instead of
      calling `sale_preview`; `npm run check` was not run (no view changed).
    - **QOR ID.** Sessions record `last_activity` at each refresh (`SessionService::record_use`, Redis `SET … XX
      KEEPTTL`, so a revoked session is not revived and expiry never moves; refresh refuses if the session is gone),
      shown as `last_used_at`; `GET /health` gains `email_leaves_this_machine`; every e2e script imports
      `scripts/e2e/_guard.mjs` and exits 2 unless that field is exactly `false`; `start-local.ps1` gained an email
      stand-in mode. The refresh and the session list joined the log check (AGENTS §9). **Wrong when found:**
      `mail_leaves_this_machine` was a stub returning `false` — the unsafe answer, which would have let every e2e
      script run against a real-mail service — with its test failing and `on_this_machine` unused. Implemented as
      `sends && !on_this_machine(api_url)`. **Run** against throwaway `qa1002-pg` (Postgres 16, port 55435) and
      `qa1002-redis` (7.4, 56381), both `--rm`: fmt, clippy `-D warnings`, **136 passed with `--include-ignored`**
      (was 130). **Not run:** the e2e scripts themselves, including the five new `sessions.mjs` checks.
    - **Owner's decisions owed (AGENTS §8, names):** the call `buy_exact`, the error `ContentChanged`, the runtime
      API `Drc369RoyaltiesApi` / `sale_preview`, the launcher module `chain/market.rs` and command `drc369_market`,
      the health field `email_leaves_this_machine`. Also: `/health` now tells anyone one bit of configuration
      (whether a deployment sends real mail); the code comment argues it is already inferable from `forgot-password`.
    - **Documents:** `PROTOCOL.md`, `chain/README.md`, the launcher README, `SYSTEMS.md`, `OWNER.md`. `DIRECTION.md`
      not changed: no roadmap item is ticked by this.
    - **Next:** the launcher's preview through `sale_preview` (removes the host copy of `split`), then the Market
      screen; then CI and the devnet (item 32's `DEVNET_PLAN.md`, waiting on the owner's nine answers).
    - **Committed `2ba0a85`** on the owner's instruction ("commit all and push and deploy when satisfactory").

34. **2–3 October 2026: a sprint with two agents, on the owner's instruction. The preview from the chain, and the
    Market screen.** Each agent worked in its own git worktree from `2ba0a85` (both worktrees were first created at
    an unrelated commit, `7bfcec6`, and each agent reset its own to `2ba0a85`; the main tree was not touched).
    - **`fbe447d`, the launcher asks the chain what a sale pays.** `sales.rs` calls `Drc369RoyaltiesApi_sale_preview`
      at the finalised block and draws the breakdown from the answer; the host's copy of `split`, its four vector
      tests and the `sp-arithmetic` dependency are gone. The answer is refused if its parts do not sum to the price
      (checked add). A refusal is named through the metadata and put in the same words as a refused transaction. A
      node without the API (spec_version below 6) is refused in words, with no fallback. IPC shapes unchanged.
      Eight faults planted, each caught. **Behaviour change:** looking up a listed asset, previewing, listing and
      buying now need spec 6; withdrawing does not.
    - **`09ad4b5`, the Market screen (L7.2's first slice).** A `market` surface on the rail and a Nexus tile;
      `src/views/Market.tsx`; `AssetCard` gained a `listed` mode; IPC `market.page` over `drc369_market`. Filter
      (everyone's, others', yours), order (cheapest, dearest, by number), paging by the host's window, the truncation
      and unreadable notices, an InfoTip saying the list is one node's view with no search, history or "newest".
      Buy re-reads the asset and opens the existing BuyDialog; Withdraw and, for the holder, Clear void listing.
      Copy that said "no storefront" now names the Market. **`check-market-view.mjs`, 127 checks; 24 planted faults,
      all caught.** Two `#[ignore]` live tests. **Product choices the agent made, for the owner:** Clear offered only to
      the holder (the chain lets anyone clear a void listing); the labels Everyone's / Others' / Yours, the eyebrow
      "Exchange", "No search yet" on the Nexus tile; a page past the end jumps to the last page. **Not covered by the
      check:** that jump, and the Refresh spinner.
    - **`3de3d46` merges both** (one README conflict, the test count). Chain and QOR ID untouched by either.
    - **Verified on the merged tree:** host fmt, clippy `-D warnings`, **195 passed, 8 ignored**. **Live, against a
      spec-6 `--dev` node, all seven chain tests passed**: six in one run (296 s: the market read, a listing read back
      by the Market, mint, transfer, trade, keychain) and the sale alone (305 s). **`npm ci`, `npm run build` and all nine view checks passed first time**: design 8, accessibility 41, gates 34, Projects 104, Inventory 238, Market 127, Gate 65, contrast 54, readability 361. The agent saw the accessibility case "stored off, then live" fail more often on its build (7 of 10 alone, against 3 of 8 for `2ba0a85`); cause not established, threshold not touched.
    - **Two false failures on the way, both environmental:** the PC entered Modern Standby 15:31–01:05 during the
      first live run (Kernel-Power 506/507), so two tests timed out on finality after 9.6 hours; and a node relaunched
      through `Win32_Process.Create` with `cmd /c` received Ctrl+C when a shell command ended (`^C` in its log) and
      died mid-test, which surfaced as subxt's "`chainHead_follow` emitted 'stop'". **Trap for §5:** start the
      development node with `Start-Process -WindowStyle Hidden` (its own console), never through `cmd /c` from a tool
      shell, and keep the PC awake during live runs.
    - **QOR ID's end-to-end scripts, run for the first time since item 33** against throwaway Postgres 16 and Redis
      7.4 (removed after) and the release build: **all six pass** — account-and-admin 24 (2 skipped, email off),
      sign-in-enumeration 11 (1 skipped), sessions 26 (the last-used checks included), registration 8, logout 15 (the
      Redis restart included), email-via-resend 50 (2 skipped) through the stand-in. **The guard was seen to refuse:**
      started with a placeholder key and no stand-in, `/health` said `true` and `sessions.mjs` exited 2 before
      registering anything. Trap: a Redis started with `--rm` is deleted by `logout.mjs`'s `docker stop`.
    - **Railway, read on 3 October:** `qor-auth`'s source has **no repository connected** (root directory
      `/services/qor-auth` only), and its live deployment is `7e154e1` (1 October). So a push redeploys nothing until
      the service is connected to `QOR-MATRIX/demiurge-chain`, which needs Railway's GitHub app to see the
      organisation (owner).
    - **Committed `26e3636`. The push was refused** by Claude Code's classifier (`git push matrix public-main:main`),
      and on 3 October so was `git fetch matrix`: the owner pushes. `matrix/main` was `34f21c1`, an ancestor, so the
      push is a fast-forward.
    - **Railway, read again on 3 October:** the owner redeployed `qor-auth` at 09:09 UTC, but it rebuilt the same
      `7e154e1` (no repository is connected), and **`sleepApplication: true` is now set** on the service, so QOR ID
      sleeps when idle and the first request wakes it. `/ready` and `/health` answered 200 (0.44 s, 0.27 s); `/health`
      has no `email_leaves_this_machine`, so the new code is not live. Whether sleeping suits "nothing may block on
      QOR ID" (ADR-056) is the owner's call; the launcher already does not wait on it.
    - **GATES.toml tightened** (two `change_log` entries, 3 October): six chain tests and one QOR ID test join the
      Alpha criteria, each seen to fail against a planted fault first, and the Market check joins `npm run check`'s
      record. Planting showed **the pallet's mock has no `Locker`**, so a preview that ignored nesting passed every
      pallet test and failed only the runtime's. Not listed, not yet seen failing: `buy_exact_keeps_every_rule_buy_has`,
      `a_preview_at_the_largest_price_cannot_overflow`, `buy_exact_is_a_new_call_and_buy_keeps_its_encoding`, and QOR
      ID's health and session last-used tests. The launcher's 15 gates tests pass against the new file.
    - **Deployed, 3 October, by the owner:** pushed to `matrix` and connected `qor-auth` to `QOR-MATRIX/demiurge-chain`
      (branch `main`, root `/services/qor-auth`). Deployment `ca92e0f4` of `342b6c5` **SUCCESS** at 09:53 UTC.
      Measured: `/health` 200 with `email_leaves_this_machine: true` (right for production: it sends real mail, so the
      e2e scripts refuse it), `/ready` 200 with database and cache ok (~0.35 s from here), a forged refresh token 401.
      Production Redis is 8.2, so `SET … XX KEEPTTL` is supported. `sleepApplication` is off again. **The service
      moved region to `us-west2` while Postgres and Redis stay in `iad`**: every query crosses the continent. Owner's
      call whether that was meant; if not, set qor-auth's region back to `iad`.
    - **The owner's decisions, 3 October (ADR-066):** `buy_exact`, `ContentChanged`, `Drc369RoyaltiesApi::sale_preview`
      and `email_leaves_this_machine` approved as named; the Market's Clear on a void listing stays holder-only; its
      labels and internal names stand. **`my-app/` deleted on the owner's word**: an untouched `create-next-app`
      starter that appeared at 02:00 on 3 October, never committed, origin unknown.
    - **The owner took every recommended option (ADR-067):** ADR-065 choices 1, 2 and 5 confirmed; buying by number
      kept beside the Market; the readability measuring rule approved (`GATES.toml` `approved_by`). **qor-auth moved
      back to `iad`** through Railway's `update-service` on the owner's word; the classifier then refused even
      read-only checks of the live service ("Production Deploy"), and **the owner confirmed `/ready` answered
      `{"checks":{"cache":"ok","database":"ok"},"status":"ready"}`**. Launcher bumped to **0.1.6** for an installer
      that upgrades 0.1.5.

35. **3 October 2026: the devnet is decided (ADR-068), and its engineering has started.** The owner funded nothing
    yet; their account `5DMPEXVcNq6ik8jzW4S7TracpEb1mnabmKbge76Xcd75qLxK` got 10,000 CGT on the local `--dev` chain
    through `dev-fund.mjs` (issuance unchanged) to try launcher 0.1.6. An "Agent IQ" idea (private simulated agents in
    the launcher) was raised and **scrapped by the owner the same day**; nothing was built. The owner then took every
    recommended answer to `DEVNET_PLAN.md` §9:
    - **ADR-068:** Railway; two private validators and a public RPC node (accepts ADR-044; supersedes ADR-015 for the
      nodes); `Demiurge Devnet` / `demiurge_devnet` / `Live`; genesis holds **only** a sudo account (existential
      deposit) and a separate faucet account (the development endowment's marked placeholder of test CGT), both the
      owner's, addresses not yet given; `alpha.devnet-live` will read `https://rpc.qorsync.dev/health/readiness`; a
      new **`alpha.devnet-finality`** check (tightening, logged); the assistant creates Railway services where
      permitted, the owner clicks where not.
    - **An agent is building, in a worktree:** the `key` subcommand, a `devnet_chain_spec` function (no raw spec until
      first boot gives real keys), `chain/Dockerfile`, a first-boot script that prints only public values, and a
      manual `devnet-image.yml` workflow publishing to `ghcr.io/qor-matrix/demiurge-node`, verified with three local
      containers. **Owed by the owner:** the sudo and faucet addresses, from a browser wallet.
    - **Merged `4366b81` (agent commit `6c9808c`).** Chain **124 tests** with the wasm built (was 118: five devnet-spec,
      one CLI), fmt and clippy clean, eight planted faults caught. `chain/Dockerfile` (context `chain/`, image 236 MB),
      `chain/docker/entrypoint.sh` (role from `DEMIURGE_ROLE`; drops to user `demiurge` uid 10001 with `setpriv`; keys
      made with `umask 077`, phrases only in 600 files then in the keystore; prints `DEMIURGE_PUBLIC` lines; waits
      without a spec at `DEMIURGE_CHAIN_SPEC`), `.github/workflows/devnet-image.yml` (manual, to
      `ghcr.io/qor-matrix/demiurge-node:devnet` and `:sha-…`), `.gitattributes` forcing LF on both. **Local evidence:**
      first-boot log carried peer id and both public keys and **0 of 27 secret patterns** (positive control 1); a second
      boot reused the keys; two validators and an RPC node on a Docker network authored and finalised (best 19,
      finalised 16), `system_chain` "Demiurge Devnet", `author_rotateKeys` and `author_insertKey` refused, readiness 200.
      **Two departures from the plan, both measured:** `--allow-private-ip` on every node (a `Live` chain refuses
      private addresses, and Railway's network is private), and **`--rpc-cors all` on the RPC node only** (the default
      403s any public Host; plan §2 corrected). **Not yet tried:** the workflow, a Railway volume's ownership, libp2p
      over Railway's private network. The root `Dockerfile`, `.dockerignore` and `fly.toml` still describe the deleted
      chain and were left, as `DIRECTION.md` records; removing them is a separate decision.
    - **THE DEVNET IS LIVE, 3 October 2026 (record: `chain/DEPLOY-RAILWAY.md`).** Owner's sudo
      `5HN6PZA4…Zadn` and faucet `5GuugU7t…4T9` (polkadot{.js} extension); genesis `0x934e2caa…254a` from
      `chain/specs/demiurge_devnet.raw.json` (`f836075`, 125 chain tests), baked into `sha-f836075`; three Railway
      services in `iad` with volumes; `rpc.qorsync.dev` on `devnet-rpc` only. **Measured:** readiness 200, two peers,
      unsafe methods refused, ten minutes finalising #55 → #155 with authors 50/51 and all three nodes agreeing on
      #155. **`alpha.devnet-live` has its URL and `alpha.devnet-finality` is done (tightening + evidence, logged).**
      The launcher's default endpoint is now `wss://rpc.qorsync.dev` and its chain preset "Mainnet" (a dead address)
      became "Devnet"; 195 host tests, and the read-only Market live test passed against the devnet. **Traps, for
      §5:** Railway labels every standard-error line "error"; an image's `sha-` tag is the short commit, so a workflow
      run before a push builds the old commit (it happened once); the organisation had to allow public packages
      before the image could be made public. **Not done:** no faucet page (the owner sends test CGT by hand from
      polkadot.js Apps); the launcher's signing paths have not run against the devnet; RPC rate limits unset
      (unmeasured); a week's cost not yet read.

36. **4 October 2026: ARQADE reconciled with the tree. Documents only.** The owner handed over a master implementation
    prompt for **ARQADE**, the ecosystem's future gaming platform (QOR ID, DRC-369 only, CGT only, a developer SDK),
    starting from an existing arcade on OpenAI Sites, and asked for whatever did not fit the ecosystem to be corrected.
    Three read-only audits, each with file and line evidence: the arcade's source (its claims held: four solo games, two
    server-checked arenas, `localStorage` Energy and tokens, a `qor_id` column nothing writes, `chain_getBlockNumber` at
    `rpc.demiurge.cloud`; and it shows USD prices for Energy and names CRGT and DMRG), the chain, and QOR ID with the
    launcher. **What the prompt got wrong**, all recorded in `docs/blueprints/arqade.md` §0: QOR ID has no browser
    sign-in (ADR-043 Proposed) and **no way for another server to verify a token** (HS256, shared secret, no
    introspection); nothing connects a browser to the vault, and ADR-011 says web surfaces sign with delegated keys
    (M5.2, unbuilt; no `pallet-proxy`); ADR-011 allows two web surfaces; `DESIGN_SYSTEM.md` rules ARQADE's look out on
    every surface; the chain has **no randomness, no account-bound asset and no enforced editions** (new gaps G-14 to
    G-16 in `ECOSYSTEM.md`); state and XP are M4.2; one faucet, one indexer, the SDK is M5.1 and unpublished before the
    freeze; third-party games list through Market (P5.3); prize funding has no source (new **U-16**). **Written:**
    the brief, ADR-069 (decision 1 is the owner's direction; 2 to 9 await the owner), P7.1 to P7.10, the `arqade` gate
    (a tightening, logged), and the indexes. The owner's copy of the prompt (`Downloads/ARQADE.txt`) was replaced by
    the corrected brief, the original kept beside it. **Found, not fixed:** the runtime's `TOKEN_NAME` is
    `"Creator God Token"` (`chain/runtime/src/denomination.rs:55`) where every record writes Creator-God Token;
    changing it changes the chain's properties, so it is the owner's call. **P7.2 (read-only devnet reads in the
    arcade) can start today**; everything else in P7 waits on ADR-069.
    **Later the same day the owner accepted ADR-069 and ADR-043 as written** ("I accept ADR-069 and ADR-043 as
    written"); a first, one-word "yes" was not recorded, because the assistant's tooling refused to mark decisions on
    so little. ADR-069's three further choices (design, hosting, name) and U-16 stay open. ADR-043's items 1 to 3
    (redirect sign-in, rotating browser tokens, per-client sessions) may now start; item 4 was already done. Both
    `arqade.decisions` units are met.

37. **4 October 2026: ARQADE enters active scope (P7.1 started, P7.2's first read). On branch
    `session/arqade-2026-10-04`.** With ADR-069 accepted, the arcade's tracked tree at `4f02baf` was imported verbatim
    into `products/arqade/` (`f647c82`; 114 files, `tsconfig.tsbuildinfo` left out; its two vendored files MIT, from
    OpenAI and shadcn; no credentials). **Baseline before any change:** 7 of 7 tests, `tsc` clean, build succeeds,
    lint fails with 13 errors and 6 warnings already present (React Compiler rules in minified code). **Then:** the
    deleted chain's `chain_getBlockNumber` at `rpc.demiurge.cloud` is replaced by `lib/chain.ts`, a read-only reader of
    Demiurge Devnet over standard Substrate RPC at the fixed `https://rpc.qorsync.dev`, which refuses any chain whose
    genesis or name is not the devnet's and reports finalized and best blocks (`/api/chain`; the QOR panel shows the
    finalized number). Five new tests against a scripted node; two planted faults (genesis check removed, finalized
    past best allowed) each failed one. Read live: finalized #12,028, genesis matched. CRGT, DMRG, the USD prices and the
    "Energy Tokens → DMRG" panel are gone from the copy; Energy is labelled practice and never CGT. `npm test` (12 pass)
    and `npm run typecheck` scripts added. **Scope (ADR-050 decision 10):** `AGENTS.md` §4 and `DIRECTION.md` §8 list
    `products/arqade/`; `[suites.arqade]`, `arqade.tests`, and an `arqade` CI job (install, type check, tests, build)
    in `[ci].quality_gates`; lint under `[ci].reports`, not failing the run — all logged as a tightening. **P7.1 is not
    ticked:** the integration inventory is unwritten and the CI job has not yet run. Not done: the ARQADE rebrand,
    the lint errors, balance and inventory reads (the rest of P7.2), and the arcade site on OpenAI Sites still serves
    the old copy, because its deploy needs the owner's Sites tooling.

38. **4 October 2026: Game Vaults designed, and the ARQADE SDK started. On `session/arqade-2026-10-04`.** The owner:
    games pay CGT rewards from a specialised wallet per game project, available to developers through the SDK, with
    an innovative way to unlock it. **ADR-070 (Proposed)**: a keyless account per game (`PalletId` sub-account, no
    key to leak) in a pallet of its own (placeholder `pallet-game-vaults`; ADR-026's proxies were weighed and
    rejected, because a proxy filter cannot see amounts and a pure proxy's spawner holds an `Any` proxy); the game's
    build minted as a DRC-369 **Cartridge**, whose holder governs the Vault; a payout authority key bounded by an
    on-chain policy; outcome ids paid once; loosening and withdrawals delayed, tightening instant; prizes held before
    paid rounds open; payouts below the existential deposit accrued; and "ignition", the SDK's unlock walk ending in a
    finalised devnet payout. U-16's "what funds a prize" now records the owner's answer; its bounds and paid entry stay
    open. **Written:** the ADR, `products/arqade/sdk/` (README, `docs/game-vaults.md`, `src/amount.ts`,
    `src/vault-policy.ts`, 4 tests), P7.11, the gate's two new units (a tightening, logged), the indexes. `npm test`
    16 of 16; `tsc` clean; two planted faults (excess precision truncated instead of refused; a shorter epoch read as
    tightening) each failed a test. **Nothing of the pallet is built**, and no number is set: every cap is the
    developer's, every bound U-16's.

39. **4 October 2026: ARQADE as a self-publishing store (ADR-071, Proposed). On `session/arqade-2026-10-04`.** The
    owner: an app-store-like process where developers self-publish once requirements are met, a project profile from a
    template as each game's store page, onboarding at any stage to build interest, CGT support from backers (Kickstarter
    and Patreon), **a price of up to 10,000 CGT per game** (the owner's number, recorded as decided), in-game purchases
    as DRC-369 assets, and agentic, "vibe coded" creation through an external LLM. **ADR-071:** the Cartridge's profile
    manifest is the store page and its revisions the devlog; four stages that publish themselves when the SDK's checks
    find nothing missing, people only for payouts, large campaigns and reports; licences and in-game items as DRC-369
    (non-fungible only, ADR-031); backing gives the work and never its proceeds, with all-or-nothing Campaign Vaults and
    milestone release, memberships per period until M5.2; agents through ADR-010's MCP server under an agent key, the
    developer signing anything public or costing CGT; all code sandboxed by build hash; and one interim content store
    for the ecosystem rather than ARQADE's own (ADR-050 decision 11). **New:** gap G-17 (no primary sale), U-17
    (backing), P7.12 to P7.15, seven gate units (a tightening, logged). **Built and tested:** `sdk/src/profile.ts`
    (`checkReadiness`, `readyStage`, `priceProblem`), the template, 5 tests; the first run caught a real gap (the
    template's all-zero QOR id passed) and two planted faults (ceiling doubled; sandbox not tied to the build) each
    failed a test. `npm test` 21 of 21, `tsc` clean. **Legal review now covers backing as well as paid chance.**

40. **4 October 2026: the owner accepts ADR-070 and ADR-071 and names the wallet "ARQ Wallet".** In the owner's words:
    "I accept ADR-070 and ADR-071 as written. The game wallet should be called 'ARQ Wallet'". Statuses and indexes
    updated; each record's status says that where it reads Game Vault the name is ARQ Wallet, and neither record's
    decision changed. **The pallet is `pallet-arq-wallet`** (`ArqWallet` in the runtime), its name taken from the
    owner's. Renamed in current documents and code: the guide is `products/arqade/sdk/docs/arq-wallet.md`, the
    module `src/arq-wallet-policy.ts`, the type `ArqWalletPolicy`. ADR-070's filename and the history in items 38 and
    39 and in `GATES.toml`'s change log keep the old words. `arqade.decisions` now has all five of its units met.

41. **4 October 2026: `pallet-arq-wallet` built — ARQ Wallets on chain (ADR-070, P7.11's first slice). On
    `session/arqade-2026-10-04`. Not on the devnet.** Mounted as `ArqWallet` at index 11, `spec_version` 7
    (`transaction_version` stays 2: no existing call changed). Keyless accounts from `PalletId(*b"dmg/arqw")` and the
    Cartridge's `(collection, item)`; the governor is `Nfts::owner` at each call; `create` (at least the existential
    deposit), `set_policy` (tightening at once, loosening pending until `apply_policy` after `loosen_delay`),
    `cancel_pending`, `set_authority`, `set_paused`, `payout` (authority only; max payout, rule version, outcome
    window, each outcome once, epoch budget and count, cap per recipient, nothing owed or the deposit spent; a payout
    that cannot reach its player accrues), `claim`, `expire_accrual`, `schedule_withdrawal` / `execute_withdrawal`
    (delayed, re-checked against what is owed), `prune_outcome`. Every amount checked, overflow refused. **Not built:**
    held prizes for paid rounds (ADR-070 decision 8), a runtime API, the SDK calls. **Placeholders:** the four U-16
    bounds (600, 600, 432,000, 1,000 blocks) in `runtime/src/assets.rs`, pinned by a runtime test as placeholders, and
    the weights (M7.2). **Evidence:** 15 pallet tests and 3 runtime tests; three planted faults (outcome check removed:
    1 failed; owed total ignored: 3 failed; loosening applied at once: 1 failed); `cargo fmt --check` and `clippy
    -D warnings` clean; **143 workspace tests pass with the runtime's wasm built** (125 before). One `clippy` allow,
    `too_many_arguments` on `payout`, with its reason beside it. The gate gains `arqade.wallet-tests`, eleven of them,
    counted by ARQADE's gate and not Alpha's (ADR-050 decision 8; a tightening, logged). **Trap found:** drive X: was
    completely full (0 bytes of 954 GB), and the linker failed with LNK1318/LNK1201 rather than saying so; 48 GB of
    incremental caches under three `target/debug/incremental` directories were removed and the run repeated with
    `CARGO_INCREMENTAL=0`. **To reach the devnet** the runtime must be built **with the `sudo` feature** (the devnet's
    runtime has it; a wasm without it would remove sudo from the devnet on upgrade) and set by the owner's sudo key.

42. **4 October 2026: ARQADE merged to `main`; the chain's CI job is red, and has been since Rust 1.99.** `main`
    fast-forwarded to `e5b5f7c` (six commits) and pushed; `public-main` now tracks `matrix/main`. CI runs on the
    organisation (no longer blocked): QOR ID and Security pass; **the chain job fails at Lints**, as it has on every
    run since 28 September, because `dtolnay/rust-toolchain@stable` installed Rust 1.99.0, whose clippy flags
    `clone_on_copy` in code `#[pallet::event]` and `#[pallet::call]` generate (reproduced locally: 36 hits in
    `pallet-drc369`, 3 in `pallet-validator-set`). Every fix weakens a gated check, so it is written up for the owner:
    `docs/architecture/RUST_TOOLCHAIN.md`, recommending a 1.98.1 pin with a non-failing newest-clippy report. Nothing
    was changed in CI or in code. `archive-7gUg78/` (a GitKraken installer zip) was left uncommitted on purpose.
    **The ARQADE job failed at first**, at setup: the root `.gitignore` excludes every `package-lock.json` and every
    `build/`, so the import had silently left out the arcade's lockfile and `build/sites-vite-plugin.ts`. Both are now
    tracked through documented exceptions (`6a4c566`); a clean copy of the commit installed, type-checked, passed 21
    tests and built, and **CI run `37260506426` passed the ARQADE job**. Trap: an import into this repository must be
    checked with `git status --ignored`, because the root rules drop files without a word.

43. **4 October 2026: the owner chose option A — the chain builds with Rust 1.98.1 (ADR-072).** "go with option A".
    `chain/rust-toolchain.toml` pins 1.98.1 with `rustfmt`, `clippy` and `wasm32v1-none`; the chain, two-validators and
    coverage jobs install 1.98.1 (coverage's `llvm-tools-preview` for that version, since the toolchain file decides the
    version in `chain/`); a new `chain-newest-clippy` job lints with the newest stable on scheduled and manual runs,
    `continue-on-error`, `SKIP_WASM_BUILD`, and no criterion reads it. ADR-072 amends ADR-033 rule 2 for `chain/` only.
    `GATES.toml`: `[ci].quality_gates` names the pin, `[ci].reports` the new job, and an `evidence-rule` change-log
    entry carries the owner's approval. Verified locally: rustup switched `chain/` to 1.98.1 from the file; fmt and
    clippy `-D warnings` clean. **CI run `37260938168` on `main` (`f3a684e`) concluded success: every job passed** —
    chain (fmt, clippy and every test with the wasm built, its first pass since 28 September), QOR ID, launcher,
    ARQADE, coverage and security; two-validators and the newest-clippy report are scheduled-only and were skipped.

44. **4 October 2026: the ARQ Wallet's runtime upgrade rehearsed; the devnet waits on the owner's signature.** Live
    devnet read first: `demiurge` spec_version 6, `Sudo::Key` set. Built `cargo build -p demiurge-node --release
    --features sudo --locked` at `715f0ee`: the runtime is `demiurge_runtime.compact.compressed.wasm`, **516,222 bytes,
    SHA-256 `0db8455c018df88ad8aac04c86016629467b506aef3c203bb582dac8ba81a901`**, copied to the owner's Downloads as
    `demiurge-runtime-v7.compact.compressed.wasm`. **Rehearsal** (`chain/scripts/rehearse-upgrade.mjs`, new): a
    development specification with its genesis `:code` replaced by the devnet's own (2,614,632 bytes, read with
    `state_getStorage(":code")`), one local validator; a transfer and a DRC-369 mint left behind; then
    `sudo.sudoUncheckedWeight(system.setCode(wasm))` exactly as the owner will sign it. **Passed:** spec_version 7,
    spec_name kept, Sudo kept with the same key, the transfer and the asset kept, `ArqWallet` in the metadata, and a new
    ARQ Wallet paid a player exactly 5 CGT. The client logged one decode warning for the upgrade block's events, read
    with the old metadata; expected at an upgrade. Traps: a `--tmp` validator needs `--node-key`. **Next:** the
    owner signs the upgrade on the devnet in polkadot.js Apps; then spec_version is read back.

45. **4 October 2026: the owner upgraded Demiurge Devnet to `spec_version` 7 — the ARQ Wallet is live.** Signed in
    polkadot.js Apps (`Developer → Sudo`, `system.setCode` with weight override, the wallet's own password) with the
    sudo account `5HN6PZA4…Zadn`. **Read back from `wss://rpc.qorsync.dev`:** `demiurge` spec_version 7,
    `transaction_version` 2; `:code` is 516,222 bytes with SHA-256 `0db8455c…a901`, the rehearsed file; `Sudo::Key`
    unchanged; `ArqWallet` at index 11 with 12 calls; finality advancing (#19,792 → #19,798 in 36 s) with both
    validators authoring; readiness 200. The node image (`sha-f836075`) was not rebuilt and need not be: the runtime
    runs as wasm from state. Trap: Apps showed no admin account on the Sudo page until the wallet's accounts appeared
    under Accounts; the wallet asks for its own per-account password, not QOR ID's.

46. **4 October 2026: rounds with held prizes (ADR-070 decision 8) and the SDK's ARQ Wallet module. `spec_version` 8,
    rehearsed; the devnet still runs 7 until the owner signs.** `pallet-arq-wallet` gains `open_round` (authority;
    prize at most the epoch budget, held at once in `Held`, at most `MaxOpenRounds` 64 open), `settle_round` (authority,
    from close to `OutcomeWindow` after; at most `MaxWinners` 64 distinct winners; pays, accrues what cannot reach,
    releases the rest) and `cancel_round` (authority or governor any time, anyone once unsettleable). `available()`
    subtracts the hold, so payouts and withdrawals cannot touch it. Held totals live in their own map, so **no stored
    encoding changed and no migration is needed**; `transaction_version` stays 2. **Evidence:** 21 pallet tests (6 new,
    one pinning the wallet account's bytes beside the SDK's) and 4 runtime tests; three planted faults caught (hold
    ignored: 2 failed; prize limit removed: 1; anyone may cancel: 1); fmt and clippy clean; **150 workspace tests with
    the wasm built** (143 before). Five more tests in `arqade.wallet-tests` (a tightening, logged). **SDK:**
    `sdk/src/arq-wallet.ts` — `arqWalletAccountId`, `arqWalletAddress`, `ss58` (checked against Alice's known address),
    `outcomeId` and `roundId` (BLAKE2b-256, length-prefixed fields, separate domains), `toChainPolicy`, `payoutArgs`;
    4 tests, `npm test` 25 of 25. New dependency `@noble/hashes` 2.4 (ADR-033 rule 2: current). **Upgrade rehearsal**
    against the devnet's live `:code` (516,222 bytes, the v7 the owner set): spec_version 8, Sudo kept, state kept, a
    payout, and a round holding 20 CGT then settling 15 to a winner — passed. The runtime, built with `--features sudo`
    at this commit, is **516,949 bytes, SHA-256 `fbd85089f7095e0604cd2d1aa07e0aecf32035e63e40845b572e72eed7db0074`**,
    in the owner's Downloads as `demiurge-runtime-v8.compact.compressed.wasm`.

47. **4 October 2026: the owner upgraded Demiurge Devnet to `spec_version` 8 — rounds with held prizes are live.**
    Signed in polkadot.js Apps as before. **Read back:** `demiurge` spec_version 8; `:code` 516,949 bytes, SHA-256
    `fbd85089…0074`, the rehearsed file; `Sudo::Key` unchanged; `ArqWallet` at index 11 with 15 calls, `open_round`
    among them; finality advancing (#20,370 → #20,376 in 36 s) with both validators authoring; readiness 200. CI run
    `37269092178` on `75e8ba8` passed every job before the upgrade.

48. **4 October 2026: P7.2 done — ARQADE reads any account's CGT and DRC-369 assets from the devnet.** `lib/account.ts`:
    `System::Account` from storage (key `twox128(System) ++ twox128(Account) ++ blake2_128_concat(id)`) and the runtime
    API `Drc369Api_assets_of`, both at the finalized head, decoded by a bounded SCALE reader; spendable is
    `free − max(frozen − reserved, existential deposit)`, as `pallet-balances` computes with `Preserve`. The SDK's
    `ss58Decode` refuses another network's prefix and a broken checksum before any read. `/api/chain/account` returns
    amounts as decimal strings (400 for a refused address, 503 for an unreadable chain); `app/account-lookup.tsx` on the
    QOR Identity screen shows them grouped, labelled test CGT, and keeps the last answer marked stale on failure.
    **Evidence:** test vectors captured from a local `--dev` node at `spec_version` 8 after two named mints (one a
    remix, one with a non-ASCII name, both commit kinds), where `@polkadot/api` decoded the same balance; the reader
    against the live devnet matched `@polkadot/api` on `5DMPEX…qLxK` (100,000 test CGT, 0 assets); the route answered
    in the local workerd runtime; headless Chrome drove the panel (a lookup showing 99,900 / 100,000; a refused checksum).
    `npm test` 31 of 31, type check and build clean; lint unchanged at 13 inherited errors. Trap: `npm run build` fails
    with EPERM on `dist` while `npm start` (workerd) is running. **The live Sites website was not redeployed.**

49. **5 October 2026: P7.1 done — ARQADE renamed on screen, its inventory written, its lint a gate.** The 13 React
    Compiler errors fixed without changing behaviour: latest-callback refs (`callback`, `doneRef`, `cb`, the Orbital
    action, Rift Survivor's props) written in an effect after render; the per-game reset as React's previous-value
    pattern; the first "nothing lit" in Synapse scheduled with its timers; three mount-time reads of browser storage and
    the arenas' first poll deferred one tick; Rift Survivor's run created by `useState`; the home link through
    `next/link`; an unused import removed. Lint 0 errors, 4 warnings (three effect dependencies, one expression), and
    now a CI gate (a tightening, logged). **Verified in headless Chrome** against the built worker: each of the four solo
    games opened, began and played with no page exception or console error (a control run showed Synapse's synthetic
    click was the check's fault, and a real key press advanced its round). **Renamed** "ARQADE · powered by Demiurge":
    title, brand, intro, notice, terminal, ledger file, exported games; `localStorage` keys and the `navigate_demiurge`
    tool name kept. **Two more false displays fixed:** the sidebar's "Demiurge Protocol · Chain connection pending",
    which read nothing, and the terminal's `chain`, which printed "block #undefined" after the devnet reader's fields
    changed. **Inventory:** `products/arqade/INTEGRATION_INVENTORY.md`, every control with its source, permission,
    persistence, failure and verification; it and `IMPLEMENTATION.md` are now in `docs/README.md`. `npm test` 31 of 31,
    type check and build clean.

50. **5 October 2026: P7.3 built — QOR ID signs people in to ARQADE (ADR-043 carried out; ADR-073). On branch
    `session/p7.3-qor-sign-in`, not merged: merging deploys QOR ID.** QOR ID: `handlers/oauth.rs` with `/oauth/authorize`
    (its own page; unknown app or unregistered redirect answered there, never redirected), `/oauth/token` (S256 PKCE
    always; client secret by SHA-256 hash; codes single-use in 60 s; refresh tokens rotate, reuse ends the session),
    `/oauth/userinfo`, `/oauth/revoke`; `QOR_OAUTH_CLIENTS` registry validated at start; sessions and tokens carry the
    app (`client_id`, `cid`, `refresh_jti`/`jti`), serde defaults so nothing stored breaks; the password check shared
    with `/api/v1/auth/login` (`authenticate_password`); `/api/v1/auth/refresh` refuses an app's token; the sign-in page's
    CSP `form-action` adds the app's origin, because browsers apply it to the post-submit redirect. **143 tests** (7 new)
    against Postgres 16 and Redis 7.4 in throwaway containers `qor-p73-pg`/`qor-p73-redis`, fmt and clippy clean; four
    planted faults caught; **the log check drives the whole flow** and caught a planted log of the code. ARQADE:
    `lib/qor-session.ts`, `/api/auth/{login,callback,me,logout}`, migration `0001` (`qor_logins`, `qor_sessions`, the
    session keyed by the cookie's SHA-256), `app/qor-panel.tsx` replacing the static rows; 9 tests, two planted faults
    caught; 40 in all. **End to end in Chrome** against a local QOR ID (port 8099) and the local worker: the button, QOR
    ID's page "Sign in to ARQADE · QOR ID", back signed in as `arqtester#0001`, `document.cookie` empty (HttpOnly),
    sign-out revoking at QOR ID. A bug that check found: the return hash must be `#qor-identity`. **To go live:** merge;
    set Railway's `QOR_OAUTH_CLIENTS` for ARQADE's production callback with the hash of a new secret; set
    `QOR_CLIENT_ID`, `QOR_CLIENT_SECRET`, `QOR_REDIRECT_URI` in the site's host; apply ARQADE's migration `0001`;
    redeploy the site.

51. **5 October 2026: P7.3 live on QOR ID's side; ARQADE moved to Vercel (ADR-074). On branch
    `session/arqade-vercel`, not merged then (merged as PR #2).** The owner merged P7.3 (PR #1, `788d3ab`; QOR ID redeployed, `/oauth/authorize`
    answering) and set Railway's `QOR_OAUTH_CLIENTS` for ARQADE (read back: the live page is titled "Sign in to ARQADE
    · QOR ID"; an unknown app still gets 400). The secret is in the session scratchpad, never in the repository. Sites
    then could not be published or checked from here (every route behind its sign-in wall; it moved itself to
    `demiurge-arcade.andyithink.chatgpt.site`), so the owner chose **Vercel, with v0 for screens**. Done: vinext, Vite,
    wrangler, drizzle, the Sites plugin, `.openai/`, `app/chatgpt-auth.ts` and `tests/worker.smoke.mjs` removed; standard
    Next.js 16; `lib/db.ts` (the D1-shaped statement interface over `pg`, `?` numbered, batches as transactions; PGlite in
    tests); `db/migrations/0001_arcade.sql`; `scripts/migrate.mjs` (production builds only, advisory lock); players are QOR
    ID accounts (id = SHA-256 of `sub`); the arcade accepts QOR ID's answer for 30 s (`qor_sessions.checked`); sign-in
    errors name their cause; origin checks use Host, redirects are relative. **41 tests** (two planted faults caught),
    type check, lint (0 errors), `next build`; migrations twice on Postgres 16; **Chrome end to end** against Postgres and
    a local QOR ID: signed in as `arqtester#0001`, a match created by that player, 401 after sign-out. The Vercel
    connector cannot create projects (403), so **the owner creates the project** in Vercel's dashboard: import
    `QOR-MATRIX/demiurge-chain`, root `products/arqade`, add Neon. Then: QOR ID's registry gets the Vercel callback, and
    Vercel gets `QOR_CLIENT_ID`, `QOR_CLIENT_SECRET`, `QOR_REDIRECT_URI`. **An incident:** a `taskkill /IM node.exe`
    and later a kill of whatever listened on port 3000 stopped Docker Desktop twice (port 3000 is a container's); the
    owner restarted it and the `vyb-*` containers came back healthy. Kill only recorded PIDs of processes you started.
    **Later the same day:** the owner created the Vercel project `qor-arqade` (team Astra Matrix), **live at
    `https://qor-arqade-tau.vercel.app`**: the page loads and `/api/chain` reads Demiurge Devnet (genesis checked). Its
    first build failed because Vercel saw the root `turbo.json` and installed the frozen workspace;
    `products/arqade/vercel.json` (PR #3) fixes install and build to the folder. Vercel at first copied the repository into
    a new private `QOR-MATRIX/arqade`; the project now builds from `demiurge-chain`, and deleting the copy is the owner's
    call. With no database or settings the site now says so (not configured, sign in, being connected) instead of a
    generic outage. Still to do: Neon, the Railway callback, the three `QOR_*` settings in Vercel.

52. **5 October 2026: ADR-075, one name per QOR ID, no `#0001` (the owner's rule). On branch
    `session/unique-usernames`, not merged then (merged as PR #5): merging redeploys QOR ID and runs migration 019 on the live database.**
    The owner's three ARQADE sign-ins as `Godmode` were refused by QOR ID's password check (Railway HTTP log, 21:10,
    `POST /oauth/authorize` 401); every sign-up path for people always refused a taken name, so this was the password or
    the account's state, not a shared name (I said otherwise first, and corrected it). QOR ID: `User::qor_id()` and every
    response give the name alone; `find_by_username` has no `ORDER BY discriminator`; `generate_discriminator` removed;
    agent sign-up refuses a taken name; an insert race maps `users_username_unique` to "Username already taken";
    `QorId` reads an old `name#0001` and drops the number; profile returns `renamed_from`; start-up logs the renamed
    count. Migration 019 (oldest keeps the name, later `name_N`, `renamed_from`, discriminator 1, unique index) checked
    on a database holding shared names. 145 tests, fmt, clippy; a planted `#0001` failed a test. Launcher: shows the
    name alone, its name rules now match QOR ID's (no hyphen), a tutorial finished under `name#0001` stays finished;
    196 tests, clippy, build. ARQADE's tests use plain names (41). **A rename flow is not built**: only agent accounts
    could have shared a name; read the count logged at deploy.

53. **5 October 2026: QOR ID's account page; ADR-076 proposed (P7.4). On branch `session/account-page`, not merged then (merged as PR #7).**
    The owner was locked out because their account was created from a key (the launcher signs in with
    `keypair-login`), so it had no password and no email, and could neither reset nor change one; a password was set by
    hand in the database (the owner ran the SQL; a script made the hash locally) and the stored name had a stray space,
    fixed the same way. Built: `handlers/account.rs`, `GET /account` (two forms, each needing the current password),
    `POST /account/password`, `POST /account/email` (reuses `request_email_change`; its limits are now said on the page,
    not "Something went wrong"), `POST /api/v1/profile/password`; a change ends every session and cancels a pending
    email change. All three joined the log check. 149 tests, clippy, fmt; a planted fault (no sign-out) failed a test.
    ARQADE's identity card links to the page (`accountUrl` from `/api/auth/me`) and shows the QOR ID once.
    **ADR-076, accepted by the owner the same day with a cap of 100,000 CGT per request**: a signed `qor://pay` link, the launcher's host dialog, the result read from finalised blocks,
    devnet only. **Also:** the owner pasted a 24-word recovery phrase into the
    chat; whatever account it controls must be treated as exposed and replaced (OWNER.md).

54. **5 October 2026: P7.4 built, tips through the QOR Launcher (ADR-076 with ADR-077). On branch
    `session/p7.4-qor-pay`, not merged then (merged as PR #8).** ADR-077 corrects ADR-076 from the launcher's code: the host dialog approves
    (the owner chose it over Windows Hello, which no signature has used since ADR-056), no fee exists to show (OPEN-4),
    "devnet only" is a genesis check. The owner chose tips to a game's creator as the first use. **Launcher**: `src/pay.rs`
    (parse and check a `qor://pay` link: ARQADE's Ed25519 key in `KNOWN_APPS`, genesis, the 100,000 CGT cap, account,
    label, id, expiry within fifteen minutes, paid-once in `paid-requests.json`), `src/chain/pay.rs` (`pay_request`:
    genesis re-checked, `batch_all` of `transfer_keep_alive` and `remark_with_event`, through `sign_and_finalise`, prompt
    built from checked values), `lib.rs` (deep-link and single-instance plugins, `handle_pay_link`, a native outcome
    message), `QorError::PaymentRefused`. 204 tests (8 new); a planted fault (no signature check) failed one. **ARQADE**:
    `lib/pay.ts` (request in the launcher's field order, Ed25519 by WebCrypto, `paymentIn`: the remark's event and the
    transfer from its sender, to the recipient, of the amount, in one extrinsic that did not fail), `lib/pay-chain.ts`
    (`@polkadot/api` over HTTP, genesis-checked; **`@polkadot/api` renames the event field `hash` to `hash_`**, which
    the first real-chain run caught), `lib/tips.ts`, `/api/pay/tip`, `/api/pay/[id]`, `app/tip-panel.tsx`, migration
    `0002_tips`. 47 tests; a planted fault (no same-extrinsic check) failed one. **The two are pinned together**: a
    link made by ARQADE's code with a test key is a constant in the launcher's tests, and ARQADE's test checks it is
    still that link. ARQADE's signing key was generated locally; its public half is in `KNOWN_APPS`, its private half in
    the session scratchpad until the owner sets `QOR_PAY_SIGNING_KEY` in Vercel. **To go live**: merge; Vercel
    `QOR_PAY_SIGNING_KEY` and `QOR_PAY_TIP_ADDRESS`; build and install the launcher (the installer registers `qor://`).

55. **6 October 2026: ADR-078 accepted, with a welcome grant of 100 CGT; U-18 opened.** One level and XP per QOR ID
    across every app, granted only by QOR ID or an app's server for checked tasks, each once (level L at 50·L·(L+1) XP;
    tutorial 50, email 25, key 25 make level 1); a level bubble and XP bar; cosmetic unlocks (ring styles colour and
    pattern only until the owner decides on glow, which the design system forbids outside `src/qfx/ceremony/`); ARQADE
    inside the launcher; 100 CGT once per new QOR ID when tutorial, email and key are confirmed, from an uncapped
    Welcome account, devnet test CGT first; three accounts per network address (a keyed hash kept 30 days); an abuse
    watcher that flags. **The owner set 100 knowing an account at exactly the existential deposit has nothing to spend.**
    Roadmap P7.16 to P7.18. Also on 5 and 6 October: tips worked end to end (the owner tipped from launcher 0.1.7 after
    funding the vault's own account: the 100,000 test CGT sit in `5DMPEX…qLxK`, which is not this vault's account);
    "The Arcade" renamed "Play Now" (PR #9); the launcher's browser checks stopped giving up before the page existed
    (the flaky "did not expose a page to inspect"). The owner wants CGT to become exchangeable; no current record says
    so yet, and it was offered as a record of its own with a legal review.

56. **6 October 2026: ADR-078's first build, levels and tasks. On branch `session/levels`, not merged then (merged as PR #11).** QOR ID:
    migration 020 (`progress_events` one row per account and task; `welcome_grants` unique per account, email and chain
    account, 100 CGT in Sparks as text; `signup_addresses` keyed hashes); `handlers/progress.rs` (tasks and XP, level
    50·L·(L+1), unlocks named, `award` once and records an owed grant when tutorial, verified email and linked key are all
    there; `summary`; `/api/v1/profile/progress`, `/api/v1/profile/progress/tutorial`; `/oauth/progress` for an app's
    own tasks with its secret and a live token issued to it; `register_limited`/`keypair_register_limited` wrap sign-up
    with the three-per-address limit, the address from Railway's **`X-Real-IP`** (Railway's documentation;
    `X-Forwarded-For` is not trusted), as an HMAC). Hooks: verify-email in both verification paths, link-key in
    `link_verified_key` and key sign-up, sign-in-arqade in `authorize_submit`; userinfo carries `progress`. 157 tests;
    the log check drives the three routes; a planted fault (a grant without a verified email) failed a test. Launcher:
    `components/chrome/Level.tsx` (bubble, XP bar), `qor_progress`, `qor_tutorial_done`, the tutorial reports itself.
    ARQADE: `reportTask` (`lib/qor-session.ts`), a first match (players.first_match_reported, migration 0003) and a
    first paid tip reported; the bubble and the level row. 48 tests. **Checked in Chrome** against a local QOR ID and
    Postgres: signing in to ARQADE granted 10 XP; the bubble showed 0 and the card "Level 0 · 10/100 XP · Next: Ring
    style: Ember". **Not built yet:** drawing ring styles and themes as unlocks, paying owed grants, ARQADE inside the
    launcher, the abuse watcher.
    **Then:** migration 021 credits accounts created before levels with the verify-email and link-key XP they had
    already earned (checked on sample accounts, twice, no change the second time); launcher 0.1.8 built
    (`QOR Launcher_0.1.8_x64-setup.exe`) to show the level.

57. **6 October 2026: avatars (ADR-079) and effects allowed (ADR-080). On branch `session/adr-079`, not merged then (merged as PR #13).**
    ADR-079 accepted ("I accept ADR-07.", replying to it). QOR ID: migration 022 (`avatars` with a still first frame for
    GIFs, `avatar_reports`, `avatar_removals`); `avatar_image.rs` (the `image` crate, pure Rust, under limits: PNG, JPEG,
    WebP, GIF up to 4 MB, orientation applied, centre square, 256 x 256, re-encoded PNG or GIF, at most 120 frames and
    2 MB; metadata does not survive, tested with a planted text chunk); `handlers/avatar.rs` (upload with a 4 MB body
    limit, delete, serve by content hash with immutable caching, `nosniff`, a sandbox CSP and cross-origin resource
    policy, the still route, report once per reporter, the owner's queue and removal, which clears every account showing
    it, tells them and is audited); profile and userinfo carry `avatar_url`. 166 tests; the log check drives upload,
    serve, report, the queue and removal. Launcher: `components/chrome/Avatar.tsx` (picture or letter, ring, level
    bubble, first frame under reduce motion), `qor_choose_avatar`, `qor_remove_avatar`, `qor_avatar` (the offline copy),
    the rail at 34 px and the Nexus header at 64 px. **The header first grew** and pushed three Nexus tiles under the
    fold; `check-readability.mjs` caught it (50 failures) and the header was brought back to its height. **ADR-080**, the
    owner's: glow, neon, gradients, shadows, canvas, pointer-reactive light and looping animation are allowed everywhere;
    the rule is distinctiveness, never a generic template; reduce motion, readability as painted, and theme tokens and
    scales still bind. `check-design.mjs` dropped its five effect rules and the QFX and ceremony exemptions (logged in
    GATES.toml's change log as the owner's loosening). Rings glow from level 1. **Next, asked for by the owner:** a library
    of reactive QFX backdrops (Drift, Aurora, Nebula, Lattice, Starfield, Liquid), combinable pointer effects, presets,
    and backdrops that evolve with the level; and ARQADE's avatars.
    *Merged to `main` the same day (PR #13, `9f8a818`), as were items 51 to 56 (PRs #2 to #12).*

58. **6 October 2026: documentation brought up to date with the tree after the IBM Bob audit.** On branch
    `session/docs-current`; merged as PR #14 (`5e90748`), the owner confirmations below in the pull request after. **What was stale:** the status sections of the current documents had been written
    before the work of ADR-063 to ADR-080 — the public repository and CI on GitHub Actions, QOR ID and the devnet on
    Railway, ARQADE on Vercel, sign-in for apps, unique names, tips, levels and avatars — and still said so. This
    file's §1 called CI blocked by a billing lock, QOR ID configured but not deployed, the launcher at 91 host tests
    and production offline; its opening called item 32 half done, `id.qorsync.dev` down and the work on
    `public-main`; items 51 to 57 said "on a branch, not merged". `SYSTEMS.md` called ARQADE not yet live on Vercel,
    `SECURITY.md` placed the nine credentials in the tree, and QOR ID's `main.rs` module comment still described
    `username#discriminator` names. **What was fixed:** all current documents were checked against the tree at
    `9f8a818` and against live state, and corrected. Here: the opening and the reading list; §1 rewritten, with rows
    for ARQADE, the decisions and what is not live; §2's chain identity, endpoint and a list of current gaps; §3's
    R-1, R-2, credentials, sessions and new surfaces; §4.0 (L1.4 partly exercised; L1.6, L1.7 and P7.4 ticked in `DIRECTION.md` with their evidence; the
    pull-request row removed; the owner's open decisions added); §4 items 1 to 11; §5's traps; §6 and §7. No
    behaviour or ADR decision changed, and no gate criterion's requirement changed. **Next:** the owner re-runs the audit against `main` on GitHub once this
    work is merged.
    **Owner confirmations, 6 October 2026** (recorded here as `GATES.toml`'s `kinds.check` evidence rule allows):
    "Release gates file confirmed" — the two reworded criterion texts above (`alpha.no-dependency-patching`,
    `public-release.single-provider-revisited`) stand; "Mark Dependency-patching gate done" —
    `alpha.no-dependency-patching` is `done = true`, evidence main's run `37496555161`, whose security step
    passed; "Allowed browser origins confirmed" — `server.allowed_origins` stays unset in production and is
    never set from the environment (it stops QOR ID starting; `services/qor-auth/DEPLOY-RAILWAY.md`).
59. **6 October 2026: ADR-081 accepted — QQ is QOR Engine.** On branch `session/docs-current`, pushed after PR #14 merged; merged as PR #15 (`d97fa69`). The
    owner commissioned QQ, an engine and editor for small, effects-led 2D and 3D games, integrated with the
    launcher, QOR ID, Qontrol and ARQADE, driveable by an LLM and generative, and chose that it **replaces** QOR
    Engine: "QQ becomes QOR Engine" and "Authorized to build by any means necessary to produce the results
    outlined." **Recorded:** ADR-081, indexed in `DECISIONS.md` and `decisions/README.md`; DIRECTION's product
    list and P3 heading marked superseded (the six P3 items stand until QQ's replace them, and the gate parser
    still reads them); `blueprints/qor-engine.md` marked superseded; `GATES.toml` loses
    `qor-engine.opens-in-stock-godot` (`a_project_opens_in_stock_godot`), a loosening logged in its change log
    with the owner's words — the criterion was unmeasurable, so no gate's counts moved. The launcher's 15
    gate-reading tests pass against the edited file, `the_repository_gates_file_is_readable_by_this_launcher`
    among them. **The delegation** (ADR-081 §5) lets QQ's builder make its engineering and product choices
    without stopping, as ADRs accepted under it; **it does not reach** the money rules, the language rules or the
    rules protecting keys and secrets (§6). **Next:** QQ's blueprint, its roadmap items replacing P3's, and the
    `qor-engine` gate rewritten for QQ as a tightening; then the build. The prompt for IBM Bob to do that work
    was rewritten without stop-and-ask points.

60. **7 October 2026: the launcher's browser checks wait 30 seconds for Chrome, not 10.** On branch
    `session/ci-browser-wait`. **What failed:** the launcher job on the merge of #15 (`d97fa69`, run `37514224188`),
    on PR #15's own run (`37514040500`) and on the 7 October scheduled run (`37604884104`), each time in
    `check-accessibility.mjs` before any check ran: "The browser did not expose a page to inspect", exactly 10 seconds
    after the design check finished. The runner image (`ubuntu-24.04`, `20260927.320.1`) was the one the green runs
    used, and the commit changed only documentation, `GATES.toml` and `.claude/settings.json`, none of which the
    launcher's build or checks read. Re-running the failed job on the same commit on 7 October passed, so the cause
    was the runner, not the change. **What changed:** all eight browser checks now poll for Chrome's first page up to
    300 times 100 ms apart (about 30 seconds) instead of 100. Only the wait for the browser to start is longer; no check
    is removed or relaxed, and nothing in `GATES.toml` changes. **Verified locally** on 7 October against a fresh
    `npm run build`: accessibility 41, gates view 34, vault gate 65, contrast 54, Projects 104, Inventory 238, Market
    127 and readability 361, all passing.

61. **8 October 2026: QQ's first slice (P3.1) built; IBM Bob's unfinished work found in the tree, checked, fixed and
    finished.** On branch `session/ci-browser-wait`, not committed. **What was found:** Bob (IBM's coding assistant,
    working from the prompt item 59 describes) had written ADR-082 (QQ's architecture, accepted under ADR-081's
    delegation), `docs/blueprints/qq.md`, QQ's six P3 items in `DIRECTION.md` and the `qor-engine` gate rewritten
    for QQ, and had begun code outside QQ, none of it run or tested: admin routes for welcome grants, an ARQADE
    view in the launcher, six QFX backdrops, a theme-unlock panel, avatar plumbing for ARQADE and a version bump
    to 0.1.9. **What was wrong in it, and fixed:** the grant routes read an `id` column `welcome_grants` does not
    have (its key is `user_id`) and accepted a 128-character hash for an 80-character column; they now key on
    the account, take only `0x` and 64 hex digits, and have three tests. The ARQADE view framed the site in an
    `<iframe>`, which the launcher's content security policy (`frame-src 'none'`) refuses and ADR-078 decision 5
    does not ask for; ARQADE now opens in a window of its own made by the host (`src-tauri/src/arqade.rs`), with
    no capability, every address routed by the host (ARQADE and QOR ID stay, `qor://pay` goes straight to the
    host dialog, other web addresses open in the browser, anything else is refused; five tests). The backdrops
    had no way to be chosen and nothing set the level they read; Settings now has a Backdrop picker, the rail
    writes the level, and switching backdrops with ambience off no longer builds the old one. `PlayerAvatar`
    was used nowhere and would have shown any address QOR ID's answer held; ARQADE's sidebar now shows the
    avatar, the site's server only passes on QOR ID's own `/avatars/<64 hex>` paths (a test), the still frame
    is used under reduced motion, and the letter fallback was made readable (dark on orange, not white).
    `qq.md` was missing from `docs/README.md`, so it was not current; it is indexed now, and the blueprint's
    `window.__qor_invoke__` and `window.__qor_session__`, which do not exist, are corrected. The gate rewrite
    dropped P1.2 from `qor-engine.depends`, which would have been a loosening logged as a tightening; P1.2 is
    back (QQ saves through Qontrol) and the change-log entry is corrected before it was ever merged. The 0.1.9
    bump was reverted: no installer was built. **QQ, built:** `tools/qor-launcher/src/qq/` — a WebGL2 renderer
    (shapes as signed-distance fields with glow, particles additive), a deterministic world (motion, a pointer
    follow, emitters), and the editor surface on the rail after Projects (viewport with pick and drag, the
    entities, an inspector per component, Play, Stop, and under reduced motion a held frame with Step). Scenes
    are saved through new host commands (`qq_save_scene`, `qq_load_scene`, `qq_list_scenes`, `src-tauri/src/qq.rs`)
    as `scenes/<name>.qq.json` in a Qontrol project, in a canonical form so one changed value is one changed
    line. No dependency was added (ADR-082 names `gl-matrix`, `resvg-wasm` and WebGPU as things to lean on;
    P3.1 needed none). **Evidence:** `check-qq-view.mjs`, 41 checks in a real browser with Chrome's software
    WebGL2, reading the canvas back as pixels, proven to fail by two planted faults (its first version missed one,
    recorded in its header); 8 `qq::` host tests; `a_qq_scene_commits_and_one_changed_value_is_one_changed_line`,
    a real Qontrol commit and diff. It is the `qor_engine` suite now (`GATES.toml`, a tightening logged with
    the CI change: `npm run check` runs ten checks). **P3.1 is not ticked:** the sprite ADR-082 decision 7
    names is not built, and nobody has used QQ in the running launcher window. **Measured on 8 October 2026:**
    QOR ID 169 tests (`--include-ignored`, Postgres 16, Redis 7.4) and clippy clean; launcher host 218 tests
    (8 ignored) under `qontrol-no-skips`, format and clippy clean; ARQADE 49 tests, type check clean, lint 0
    errors (4 warnings, all in lines this work did not touch); the ten browser checks: design 3, gates view 34, Projects 104, Inventory 238, Market 127, QQ 41, vault gate 65, contrast 54 and readability 361 pass; accessibility passed 3 of 5 runs and failed 2 on "stored off, then live: the backdrop starts", the flake §5 records, which `main`'s own code (`4396afb`, built in a separate worktree) failed 3 of 6 times on this machine the same day, so it is not this change's. **Not done:** P7.18's sign-in with the
    launcher's session; paying the owed grants (P7.17); measuring the five new backdrops as painted
    (`check-readability.mjs` and `check-accessibility.mjs` draw Drift only); the Dusk and Nocturne themes the
    level unlocks name (Settings shows them as coming). **Next:** the sprite and a run in the launcher window,
    then tick P3.1; then P3.2 (3D and glTF).

62. **8 October 2026: QQ moves to Qt 6 (ADR-083); ARQADE's window signed in with the launcher's session; the
    accessibility flake found and fixed; items 61 and 62 are pull request #17.** On branch
    `session/qq-p3.1-bob-fixes` (pushed to `matrix`, pull request #17, whose first CI run passed in full). **The owner's
    direction:** "Utilize QT framework because i have a licence to build with it", an engine "capable of creating
    virtual worlds and experiences", and autonomous game design and development. Asked, the owner chose to install
    **commercial Qt** with their own account, and to keep the TypeScript engine until the Qt build plays. **IBM Bob's
    QQ work was documents only** (ADR-082, the blueprint, the P3 items, the gate rewrite); the engine in item 61 was
    built in this session. **Decided and recorded:** ADR-083 (QQ on Qt 6 at `products/qq/`: Runtime, Studio, Player
    native and WebAssembly, QML scenes, an agent with an MCP server and a design loop; the Studio never holds a key),
    superseding ADR-082 in part; DIRECTION P3 rewritten for it (each item asks more than before); the blueprint
    rewritten; `qor-engine.decisions` counts ADR-083 (a tightening, logged). **Nothing under `products/qq/` is written
    yet**, on purpose: it cannot be compiled here until Qt is installed, and untested engine code is what went wrong
    with Bob's. **ARQADE signed in by the launcher (P7.18):** QOR ID gains `POST /api/v1/oauth/handoff`. A signed-in
    launcher opens ARQADE's window at ARQADE's sign-in, stops the window on its way to `/oauth/authorize`, and sends that
    address with its own token; QOR ID checks it exactly as its sign-in page does (the checks are now one function,
    `check_authorize`, and code issuance another, `issue_code`, shared by both paths), refuses an app's own session,
    and answers with ARQADE's redirect, which the launcher loads only if it lands on ARQADE over https. The code is
    ARQADE's ordinary code, bound to its PKCE challenge, so nothing new travels in an address. This also fixes a gap
    nobody had named: the window's only sign-in was a password, and a person who signed in to the launcher by key has
    none. QOR ID: 3 tests and the flow in the log check; launcher: 3 more `arqade::` tests. **The accessibility flake:**
    "stored off, then live" failed about two runs in five on `main` as on branches since September. Measured: when it
    failed, a bare animation loop beside the backdrop stalled with it, with the main thread idle, with or without the
    GPU, so headless Chrome had stopped producing frames for the page after the backdrop's GL program was rebuilt.
    Bringing the page to the front again (`toFront`), as boot already does after a reload, made the check pass 10
    runs in 10, and planted faults still fail it. The investigation also found the launcher creating a GL context at
    startup with ambience stored as off (the canvas read `data-ambience` before it was applied): `main.tsx` now applies
    the stored settings before the first render, pinned by a new case. Each of the six backdrops is now required to
    compile and run (a planted GLSL error fails exactly its own case); item 61's note that only Drift was measured as
    painted was wrong, since `check-readability.mjs` already measures text over a hostile white panel in the canvas's
    place. **The README** is rewritten as the public front page, every fact checked against this file. **Measured on
    8 October 2026:** QOR ID 172 tests, clippy and format clean; launcher host 221 tests (8 ignored) under `qontrol-no-skips`, format and clippy clean; the ten browser checks all pass: design 3, accessibility 49 (10 runs in 10), gates view 34, Projects 104, Inventory 238, Market 127, QQ 41, vault gate 65, contrast 54, readability 361. **The backdrops, polished:** CI's software renderer drew Bob's Starfield at 11 frames a second, because it looped over every star for every pixel; it now looks at one grid cell per pixel (60 frames where it drew 15, measured full-window in software, with three depth planes of round stars where it had two of stretched ones). Lattice tied its lines into a knot under the pointer and was stretched by the window's aspect ratio; Liquid tore its cells under the pointer and swelled its borders into bright wedges: both now move smoothly through the pointer, Lattice's cells are square, and Liquid's borders are measured to the bisector of the two nearest centres, one pass. **Not done:** the welcome-grant payer
    (P7.17), which needs the owner to name a new service and to create and fund the Welcome account (§4.0); QOR ID
    deployed with the handoff route (it deploys from `main` when #17 merges); a person using QQ's preview and ARQADE's
    window in a running launcher. **Next:** once Qt is installed, P3.1 in `products/qq/`.

63. **9 October 2026: QQ's native engine started on Qt 6.12.** On branch `session/qq-qt-foundation`. The owner merged
    pull request #17 (QOR ID redeployed with the handoff route: `/health` 200, `/api/v1/oauth/handoff` 401 without a
    token) and installed **Qt 6.12.0 under an Enterprise licence** at `C:\Qt`, with the MSVC 2022 64-bit kit, Qt Quick 3D
    (with Physics, XR, particles, effects and the IBL baker), Spatial Audio, Shader Tools, Graphs, gRPC/Protobuf, HTTP
    Server, WebSockets, both WebAssembly kits, CMake 3.30, Ninja and Qt Creator. **Built:** `products/qq/` (CMake, C++20):
    the runtime module `QQ` (`World`: a procedural dusk sky that also lights the scene, filmic tonemapping, bloom from
    light brighter than white, ambient occlusion, 4x multisampling, specular anti-aliasing; `Sun` with soft shadows set by
    elevation and azimuth; `Ground`; `Prop`, a glTF 2.0 model loaded at runtime that reports `Ready` or `Failed`), QQ
    Studio (an orbit-and-zoom viewport, a model chooser, sun, exposure and bloom controls; `--capture <file>` renders,
    saves the window and exits, for checks and later the agent), and `build.ps1`. **Evidence:** `tst_world`, three cases
    that render real frames on the GPU and judge their pixels (the model drawn lit in its own colour, bloom lighting
    pixels past the glowing band, an unreadable model an error not a crash), 3 of 3 passing; proven to fail by two
    planted faults (bloom ignored; a prop that never loads). No compiler warning at `/W4`. Studio captured and looked at.
    **Found while tuning:** the first look blew the orange sphere to yellow-white and the cyan band to white (sun,
    sky light and a bloom threshold below white together); QQ's default look now keeps lit colour and blooms only
    what is brighter than white. **Not in CI:** GitHub Actions has no licensed Qt (ADR-083). **Next:** the rest of
    P3.1: the scene tree, the inspector, transform gizmos, scenes saved as canonical QML in a Qontrol project, `.qq.json`
    import, and **Open in QQ Studio** from the launcher.

64. **9 October 2026: P3.1 done: QQ Studio edits a 3D world and keeps it in a Qontrol project.** On branch
    `session/qq-p3.1-studio`. **Built:** in the runtime, `Scene` (the root a file holds, with the world's look), `Lamp`,
    `Shape` (cube, sphere, cylinder, cone, plane; physically based surface; glow), and every entity declaring `kind` and
    `fields`; `SceneIO` (C++): canonical writing (fields in declared order, one per line; numbers to four places,
    colours `#rrggbb`, `Qt.vector3d`, model files relative to the scene), loading from text every time (the engine's
    cache would hand back a stale scene), atomic saving (`QSaveFile`), adopting models (and the buffers and images a
    `.gltf` names, never overwriting a different file of the same name) into the project's `assets/`, importing `.qq.json`
    format 1 (built as QML and passed through the same writer, so it is canonical by construction; motion, follow and
    emitters named as not yet in 3D), and the editing operations the agent will reuse (P3.3): list, add, remove, discard,
    describe a project. In the Studio (now a library, `QQ.Studio`, so tests load the real window): the scene tree, a
    generic inspector (each field's editor chosen by its type; `ranges` give sliders, `choices` a list), picking in the
    viewport, handles in an overlay view (always on top; move along X, Y, Z, turn about the vertical, scale evenly), the
    project bar (choose, scenes listed, versioned or not), New, Import 2D, Save (Ctrl+S), Delete, F to frame. In the
    launcher: **Open in QQ Studio** (`qq_open_studio`: the project must be a repository, passed as one argument, no
    shell; the Studio is found where `QQ_STUDIO` points or where `build.ps1 -Deploy` puts a self-contained copy, about 1,500
    files and 113 MB with the C++ runtime). **Evidence, on the owner's machine:** `tst_world` 3 cases, `tst_scene` 12,
    `tst_studio` 8 (clicks and drags in the real window), all passing, the Studio's two planted faults (a diagonal drag; a
    save that skips adopting) each failing exactly its case; launcher host 224 tests (9 ignored) under `qontrol-no-skips`, format and clippy clean, and all ten browser checks: design 3, accessibility 49, gates view 34, Projects 104, Inventory 238, Market 127, QQ 42, vault gate 65, contrast 54, readability 361; `check-qq-view.mjs` 42 (Open in QQ
    Studio asks the host with the open project); `qq_studio_starts_on_a_project` (ignored by default) started the
    deployed Studio on a real repository. **Bugs found by the tests and fixed:** a model on another drive would have
    been written as `X:/...`, which QML reads back as a URL with scheme `x`; Qt 6 keeps URL properties unresolved, so
    adopting a model needed the entity's own file to resolve against. **P3.1 ticked** in DIRECTION with each clause's
    evidence. **Next:** P3.2, worlds that play: physics, input, particles, spatial audio, live behaviours, the Player
    native and in a browser.

65. **10 October 2026: P3.2 done: worlds play, in QQ Studio and in the QQ Player, natively and in a browser.** On branch
    `session/qq-p3.2-play`, in pull request #21 (commits `3e15e97` and `1b20318`). The work was found uncommitted in the tree at the start of the session,
    built and tested from scratch, and finished. **Built:** in the runtime, bodies on `Shape` (none, static, dynamic;
    mass, bounce, friction) under the scene's `gravity`; `Player` (a character controller with its own fall and jump,
    and a camera that follows); `Input` (C++: keyboard, mouse and an XInput gamepad as move, look, jump and run; not
    read while a text field has focus; let go when the window loses focus); `Emitter` (particles drawn with `SoftDot`,
    a sprite made in memory); `Sound`; `Behaviour`, `Logic` and `LogicFile` (a logic file built when play starts and
    rebuilt on each save, a broken save leaving the running logic in place and saying why); `Game` (a world playing one
    scene file); `SceneIO.play` (a copy built from the scene's canonical text); `gpu` (draw on the faster of two GPUs).
    In the Studio: Play (F5) and Stop (Esc), the eye following the player, a drag looking around, New behaviour, and
    sounds and logic adopted into the project. The QQ Player (`products/qq/player/`): plays a scene file or its bundled
    playground, `--capture` and `--measure`. The browser build: Qt's multithreaded WebAssembly kit, `build.ps1 -Web`,
    and `player/measure-web.mjs`; **ADR-084** records why (the single-threaded kit has no Qt Multimedia; Qt's spatial
    audio makes its sound on a worker, where the web has none, so in a browser a `Sound` is a plain sound effect whose
    volume follows the player, unpanned; the page must be cross-origin isolated, which binds ARQADE in P3.5).
    **Found and fixed:** `tst_play` crashed in its gamepad case with an access violation. It was the test: `Stage::open`
    counted `frameSwapped` through a lambda capturing a local, with the window as context; the signal is queued from the
    render thread, a late one arrived after `open()` returned and wrote into a stack slot where `qWait`'s event loop had
    saved `r13`, and the next function to rely on `r13` destroyed a string through a pointer of 1. Found with cdb, a
    debug-info build and a hardware watchpoint; fixed by counting through a short-lived `QObject` context, whose
    destruction drops calls still queued (`tst_world` had the same shape). Also: the inspector's "Unable to assign
    [undefined]" warnings when Play swaps the selection; an unused constant clang warned about in `input.cpp`; mouse
    look had no check, and has one now. **Evidence, on the owner's machine** (Qt 6.12, MSVC 2022, RTX 4060 laptop):
    `tst_play` 11 cases, `tst_player` 4, `tst_scene` 14, `tst_studio` 10, `tst_world` 3, all passing; five planted
    faults (no gamepad dead zone; logic not watched; Play on the scene, not a copy; sound active while editing; a
    player that ignores the mouse), each failing its own check; the native Player's first frame
    1,371 and 1,438 ms from the process starting, 229 and 345 frames a second; in headless Chrome, three runs: first
    frame 5,953, 5,812 and 5,779 ms from the page starting to load (cold, served from the same machine), 220 to 227
    frames a second, 44.1 MB served (15.9 MB gzip, 11.7 MB brotli), the first frame the game (looked at), no error on
    the page or in any worker. **Not checked:** what a sound sounds like (the checks see it set up, not heard); a real
    gamepad (the checks give the pad's state). **Deployed:** `build.ps1 -Deploy` reran every suite and put the P3.2
    Studio where the launcher's **Open in QQ Studio** finds it (1,520 files, 182 MB, up from 113 MB with physics,
    audio and Qt Multimedia); started with no Qt on the path, it drew its scene and exited 0. **Open:** a logic file is code with the engine's full reach; before games are
    published (P3.4) and played by others (P3.5), how a stranger's logic is confined needs deciding. **P3.2 ticked** in
    DIRECTION with each clause's evidence. **Not in CI** (ADR-083). **Next:** P3.3, the agent.

66. **10 October 2026: P3.3 under way: QQ Studio's tools for agents, over MCP.** On branch `session/qq-p3.3-agent`,
    branched from pull request #21's tip. **Built** (`products/qq/agent/`): `McpServer`, MCP's JSON-RPC 2.0 written
    here rather than on Qt's `Qt6Mcp`, whose 6.12 API is private headers only (protocol versions 2025-06-18, 2025-03-26
    and 2024-11-05; initialize with instructions for the model, ping, tools/list, tools/call; JSON-RPC's errors), served
    on a Windows named pipe opened with `QLocalServer::UserAccessOption`, so only the user running the Studio can open
    it, and no network port; a second Studio finding the pipe answered stays out and says so. `StudioTools`: fourteen
    tools (`scene_read`, `kinds`, `scene_set`, `entity_add`, `entity_set`, `entity_remove`, `behaviour_write`, `play`,
    `stop`, `input_set`, `wait`, `view_set`, `frame_capture`, `log_read`) that work the Studio's own window functions,
    so what an agent changes is what the creator sees: in the list, the inspector, the notice line, unsaved. Values as
    JSON (vectors `[x, y, z]`, colours `#rrggbb`, files relative to the project), checked against each field's type,
    range and choices, all or nothing per call; edits refused while playing, except logic (rebuilt at once) and the
    controls (the agent's hands are a gamepad's, a real pad set aside until stop); logic checked for whether it builds
    as it is written; `StudioLog` keeps Qt's messages, the Studio's notices and play's problems, a line said every frame
    kept once with a count. The agent cannot save or commit. `qq-mcp`: the program an MCP client starts, relaying
    standard input and output to the pipe, answering "not running" with no Studio and finding one opened later;
    `build.ps1 -Deploy` puts it beside the Studio. **Evidence, on the owner's machine:** `tst_agent` 9 cases (the
    protocol; every tool in the real Studio window through the protocol; the real `qq-mcp` and `qq-studio` end to end),
    with all six suites passing (`tst_agent` 9, `tst_play` 11, `tst_player` 4, `tst_scene` 14, `tst_studio` 10,
    `tst_world` 3); four planted faults (a refused change half kept; edits allowed in play; a bridge that never retries;
    a log keeping every repeat), each failing its own check, the first only after the check was corrected (below); an
    independent client, the official MCP Python SDK 2.3.0, drove a Studio through `qq-mcp`: protocol 2025-06-18, the
    fourteen tools, a crate, an emitter and spinning logic built, played, a frame captured and looked at, stopped, a
    mass out of range refused; the deployed `qq-mcp` runs with no Qt on the path. **Found on the way:** a check that a
    refused value undoes the others passed with the fault planted, because `QJsonObject` takes keys in sorted order and
    the bad value came first; corrected so a good value is set before the bad one is met. A second write of a logic
    file failed "Access is denied" because the test still held the first open; a retry added to `writeLogic` on a
    guessed cause (a virus scanner) was taken out again once the real cause was found. **Not yet:** the design loop
    and the Agent panel, the provider (an ADR when the loop is built), its key in the keychain, and the proof of a
    scene built from a description, which needs a provider key from the owner. **Next:** the design loop.

67. **10 October 2026: P3.3's design loop built: the Agent panel, on Claude Opus 5.5 (ADR-085).** On branch
    `session/qq-p3.3-agent` (pull request #22). **Built** (`products/qq/agent/`): `DesignLoop`, a manual tool-use loop
    over Anthropic's Messages API in raw HTTPS from Qt (no C++ SDK exists), shaped from the Claude API reference: model
    `claude-opus-5-5`, adaptive thinking, effort `high` set explicitly (the model's default is `medium`), 16,000 tokens
    an answer, `fallbacks: "default"` with beta `server-side-fallback-2026-07-01`, automatic prompt caching; the same
    fourteen tools as MCP, called through `McpServer::call`; frames returned as image tool results; the conversation
    append-only, each turn's results in one message; a `refusal` ends the run, a `max_tokens` turn is not run, what a
    `fallback` block replaced is neither run nor echoed; 408, 409, 429 and 5xx waited out up to five attempts as
    `retry-after` says; at most 60 answers; a run counter so nothing from a stopped run acts in the next. `keychain`:
    Windows Credential Manager (`CredWriteW`, `CredReadW`, `CredDeleteW`), the key read only for a request, sent only in
    `x-api-key`; `QQ_ANTHROPIC_URL` accepted only over https or to loopback. In the Studio: an **Agent** button and
    `AgentPanel.qml` (key once, then hidden; description; Design and build; Stop; every step with its frames; Undo the
    run, through a new `restoreText` that keeps what was last saved); the loop stopped when the Studio quits.
    **Evidence, on the owner's machine:** all six suites pass (`tst_agent` 13 run and 1 skipped unless asked, `tst_play`
    11, `tst_player` 4, `tst_scene` 14, `tst_studio` 10, `tst_world` 3). The loop's cases run against a stand-in for the
    API inside the test (an HTTP server on loopback answering from a script): every request's headers and body, the
    history append-only, three results in one message with the frame as a 640-pixel PNG, the crate left unsaved with
    play stopped, the key in no Studio log entry and no request body, undo back to the scene before; 429 and 529 waited
    out, 401 not retried, a refusal, a cut-off answer, a stop dropping the request in flight; a fallback's replaced
    blocks; the panel asking for a key and never showing it again. Four more planted faults, each failing its own check
    (replaced tool calls run; no waiting out; a cut-off call run; the key in the body), the second only after the
    planted fault itself was corrected (it had replaced half an expression). `QQ_LIVE_ENDPOINT=1` ran one request to the
    real endpoint over TLS with a key that is not one: refused with 401, as it should be (deployed: Qt's `schannel` TLS
    backend is in the deployed Studio). **Not yet:** the proof, a scene built from a description with no human edit,
    recorded with its frames: it needs the owner's Anthropic key, typed once into the Agent panel; its runs are billed to
    that key. **Next:** that proof, then P3.4.

68. **10 October 2026: generation is paid by whoever uses it, and Demiurge pays for none (ADR-086, the owner's
    decision).** On branch `session/generation-paid-by-user`, in pull request #24, based on the tip that holds P3.3.
    **First, a merge that missed:** pull request #22 (P3.3) was based on #21's branch and merged 16 seconds after #21,
    before GitHub retargeted it, so it merged into `session/qq-p3.2-play` and P3.3 was not on `main`; pull request #23
    brings that branch to `main` (no new changes). Stacked pull requests are not used again: each targets `main`. **The
    owner asked** that generative models cost the project nothing, even if a feature is unavailable at first, and that
    users pay for the generation they use. **Found by searching the tree:** nothing active holds a provider key of the
    project's (QQ's loop already used the creator's own key, ADR-085); the frozen Sophia chat (`apps/hub`, `apps/guru`)
    ran on the server's own xAI, Anthropic and OpenAI keys, documented with a Pinecone index in the root
    `.env.example`. **Done:** ADR-086 (no project-held provider key or account anywhere, no proxy, no project-funded
    allowance; the user pays their provider directly; a feature is off until its user turns it on with their own
    account, says who pays and shows what was used; selling generation through Demiurge is open, U-19, with U-9 pointed
    at it); the Sophia and Pinecone sections of `.env.example` replaced by a note forbidding provider keys there;
    `DesignLoop` counts each run's tokens from the provider's own `usage` (input, cache written, cache read, output,
    answers) and the Agent panel shows them, and says before a key is given that the agent runs on the creator's own
    Anthropic account, billed by Anthropic, with nothing paid or charged by Demiurge; a stray `undefined` font in the
    panel's step list fixed. **Evidence, on the owner's machine:** all six suites pass (`tst_agent` 14 run and 1
    skipped unless asked, `tst_play` 11, `tst_player` 4, `tst_scene` 14, `tst_studio` 10, `tst_world` 3); the new case
    `onlyTheCreatorsOwnKeyIsEverUsed` sets `ANTHROPIC_API_KEY` in the environment and shows nothing is sent without the
    creator's key, that with it only that key is sent, and that the panel says who pays; a planted fault (falling back
    to the environment's key) fails it; usage totals checked against the stand-in's counts. **P3.3's proof** is made by
    whoever runs it on their own key; the owner is not asked to fund it. **Merged:** #23, then #24, on 10 October 2026;
    Pleroma CI passed on `main` at `b99c689` (run 38058362606).
69. **10 October 2026: confining a stranger's QQ game, proposed for the owner's decision.** On branch
    `session/qq-logic-sandbox`. Asked by the owner after P3.3: before games are published (P3.4) and played by others
    (P3.5), a game's logic must be kept from harming the person who plays it.
    [`docs/architecture/QQ_LOGIC_SANDBOX.md`](docs/architecture/QQ_LOGIC_SANDBOX.md) sets out what a game's logic can
    reach today, read from the code (the same QML engine as the game with nothing taken away; QQ's own file operations
    by path; Qt's general-purpose QML environment), so natively it has the player's own reach; five properties a
    stranger's game must hold; five options (the browser from an origin of its own; an OS-sandboxed native Player; a
    confined QML engine; a fixed set of behaviours; review); and a recommendation: strangers' games in the browser only
    at first, the Player's engine confined in every mode, native play of others' games only once an OS sandbox exists.
    What logic can reach is read from the code, not shown by experiment. **Decided the same day: ADR-087**, the owner
    accepting the recommendation and the advice on its four questions (browser only at first; the Player locked down in
    every mode, built first; native play of others' games deferred; games from a separate `*.vercel.app` project now and
    a registered domain before launch, never a subdomain of the project's domains). In pull request #26 with the
    write-up. **Next:** the Player's lockdown.

70. **10 October 2026: the QQ Player locks down what a game's logic can reach (ADR-087 decision 2).** On branch
    `session/qq-player-lockdown`. **Built** (`products/qq/runtime/`): `Confinement::apply`, which the Player calls before
    a game loads, natively and in a browser: a URL interceptor (QML, scripts and files only from the game's package and
    Qt's and QQ's module folders, the innermost import folder naming a module, version-numbered folders read as their
    module; imports only from an allowed list: QML, QtQml and under it, QtQuick and QtQuick3D themselves, Quick's
    Window, Layouts, Shapes, Effects, Particles, Templates and Controls, Quick 3D's Helpers, Physics, Particles3D,
    SpatialAudio, AssetUtils, Effects and ParticleEffects, and QQ); a network-access factory refusing every request
    (Qt's WorkerScript makes its manager through the same factory, read from Qt's source); `SceneIO::confineTo`, from
    C++ only (reads only inside the package, writes and copies nothing); and URL handlers refusing a list of schemes to
    `Qt.openUrlExternally`. `WebVoice`, QQ's own sound effect for a browser `Sound`, replaces Qt Multimedia's QML
    `SoundEffect` in `Sound.qml`. `Prop` gives its loader the model's whole path, resolved against the scene file.
    **Found on the way:** Qt 6 hands a url property to the interceptor as written, relative, so relative paths are let
    through and checked whole when loaded (a check climbs out of the package to show it); `Qt.openUrlExternally` takes
    its URL through the interceptor first, so a refused URL arrived as the `qrc:` placeholder, unhandled, and Windows
    asked "Pick an app" (four such dialogs opened during the runs and were closed; `qrc` is now refused); and Qt
    Multimedia cannot be refused natively, because `Qt6Quick3DSpatialAudio.dll` links `Qt6MultimediaQuick.dll`, whose
    types register when it loads, and a registered module is imported without its `qmldir`. **Evidence, on the owner's
    machine:** all seven QQ suites pass (`tst_lockdown` 7 cases, 12 checks, Qt Multimedia's an expected failure with its
    reason; `tst_agent` 14 and 1 skipped; `tst_play` 11; `tst_player` 4; `tst_scene` 14; `tst_studio` 10; `tst_world`
    3); four planted faults (no interceptor; no network policy; `SceneIO` unconfined; every module allowed), each
    failing its own check, the URL-handler fault not planted because it would open dialogs on the desktop; the bundled
    game plays with nothing refused; the browser build plays with no error. **Measured on mains power in item 71:** the
    lockdown costs nothing measurable at start-up. **Next:** P3.4, publishing.

71. **10 October 2026: the lockdown measured, and a model that fails to load is no longer hidden.** On branch
    `session/qq-lockdown-measured`. **Measured** on mains power (checked with `Win32_Battery`), the owner's RTX 4060
    laptop, the bundled playground: the native Player's first frame is about 1.7 s (1,709 to 2,023 ms, five runs; 265 to
    346 frames a second); in headless Chrome 5.5 and 5.7 s from the page loading (313 frames a second). The build before
    the lockdown (`fc353d2`, built beside it) drew its first frame in about 1.4 s, and the lockdown's parts, switched off
    one at a time and all together, change nothing measurable: the difference is the orb. **Found:** before the lockdown
    the bundled game's one model (`../assets/orb.gltf`) never loaded in the Player: its loader took the relative path
    against QQ's own module, the file was not there, and the game counted a failed model as loaded and played without
    it (its first frame has no orb; the current one has). Item 70's change to `Prop` (the whole path, against the scene
    file) is what fixed it, so P3.2's 1.4 s was for a scene missing its model. **Changed:** `Game.unloaded` lists the
    models that failed, each with its reason; the Player puts it in its measurement; `tst_player` (the bundled game and
    a scene from disk) and `measure-web.mjs` fail on any. **Evidence:** all seven QQ suites pass; the old `Prop` line put
    back as a planted fault fails both `tst_player` cases with `Orb: IO Error: File not found`; the browser build plays
    with `unloaded` empty. QQ Studio redeployed (with these changes) so **Open in QQ Studio** has them. **Next:** P3.4,
    publishing, once the owner has done the L1.4 dialog checks (done in item 72).

72. **10 October 2026: L1.4's transfer and RPC endpoint dialogs, exercised by the owner.** In the installed launcher
    0.1.8 (6 October; the dialog code has not changed since), against `demiurge-node --dev --tmp` (the release build of
    5 October; the chain's code has changed only in comments since), RPC `ws://127.0.0.1:9944`. **Done by the owner:** a
    change of RPC endpoint to Local node, approved and declined; a transfer, approved and declined; the endpoint changed
    back to Demiurge Devnet, declined and then approved. **Read back:** the launcher's saved endpoint was the local node,
    then `wss://rpc.qorsync.dev` again (`settings.json`, 13:07); on the local chain the owner's account
    (`5DSNDZ…UV34`), funded with 10,000 CGT from Alice by `chain/scripts/dev-fund.mjs` (total issuance unchanged), sent
    three transfers of 100 CGT to Bob's development address, the first finalised in block #676 (nonce 3, 9,700 CGT left:
    the runtime charges no fee yet, M6.4). The declines leave nothing on a chain, so they are the owner's account.
    **Found:** a transfer from an account with nothing in it asks for approval and is then refused by the node: the
    launcher checks the amount and the addresses before the dialog, not the balance. Asking first is harmless (nothing
    is sent), but a balance check before the dialog would say so sooner; not built. **Also:** on Devnet the owner sent
    100 test CGT to Bob's development address, whose key is public, so those 100 are anyone's to take (test CGT; no
    value). **Still to do for L1.4:** the QOR ID endpoint change, approved and declined, and a key sign-in after QOR ID
    was unreachable at unlock. **Next:** P3.4, publishing: its signing dialog is now exercised both ways.

73. **10 October 2026: P3.4 built. QQ Studio publishes a project as its Cartridge, through the launcher that started
    it.** On branch `session/qq-p3.4-publish`. **Decided first (ADR-088, the owner's choices):** a Cartridge holds the
    ADR-047 manifest of the project's whole tree at the commit published, with that commit (amending ADR-071 decision
    2, which said the profile manifest; the profile is a file in the tree); the first Publish mints it, each later one
    revises it (`Drc369::revise`), so one game is one Cartridge; the names `host`, `qor-host-proof`, the QOR host
    protocol and `LauncherLink`. **Built, launcher:** `src-tauri/src/host/`, ADR-046 for its first product: a named
    pipe only this user can open (DACL, first instance, no remote clients; a `0700`/`0600` socket elsewhere) that
    takes one connection, from the spawned process only (client process id, its start time, its user); a 32-byte token
    on the product's stdin, good once for 30 seconds, compared in constant time; frames of a 4-byte length and JSON up
    to 1 MiB; `hello`/`welcome` with a version and capabilities; typed intents only (`qontrol.commit`,
    `cartridge.publish`); `ProductDialog`, naming the product from the launcher's record at the top of every prompt;
    a Windows job that ends products with the launcher; the exit code and the end of stderr shown on the QQ screen
    when the Studio ends badly. The FFI is `tools/qor-launcher/host-proof/` (its own crate; the launcher forbids
    unsafe code), now in CI. `ChainClient::revise` and its dialog; `qontrol::publish::nearest_in_history` finds a
    project's Cartridge as the asset this account holds whose commit is nearest HEAD in its history, so nothing is
    remembered on the machine. The schema and golden frames: `tools/qor-launcher/host-protocol/`. **Built, Studio:**
    `products/qq/studio/launcherlink.*` (the token from stdin, the hello, commit then publish, exit when the channel
    closes) and **Publish** in the toolbar, which saves first and says what happened; a Studio not started by the
    launcher cannot publish and says why. **Evidence, owner's machine:** the launcher's 240 tests pass with
    `qontrol-no-skips` (10 ignored: the 9 as before and the new live one), format and clippy clean, the ten browser
    checks pass (QQ 42); 15 host checks, including **a live one against `demiurge-node --dev`**: a stand-in product
    (`src-tauri/examples/host_fake_product.rs`) started through the real channel commits a project, mints it behind a
    dialog titled "Fake Product asks: Approve this mint", revises the same Cartridge after a new commit, is answered
    `unchanged` with nothing asked, and a declined publish leaves the chain as it was; `qor-host-proof`'s 5 Windows
    checks; all eight QQ suites, with `tst_publish` (9 cases) new. **Eight planted faults, each caught:** the peer
    check skipped, the token check skipped, the token logged, another product's hello accepted, a second publish
    minting instead of revising, the Studio publishing without the commit, the Studio publishing without saving, Publish
    greyed out on an unversioned folder. **Found on the way:** `cargo test` only checks that an example compiles and never builds its executable, so the
    first host runs used a stand-in built by hand; the tests now build it themselves. The log check first passed with
    the token logged when run alongside other tests, because `tracing` caches across threads whether a line is wanted;
    it now installs one process-wide logger. **The owner's first run (10 October) found Publish greyed out:** the Studio
    had been opened on a folder that is not a Qontrol project, and the reason was only in a tooltip on the grey button.
    Publish is now pressable whenever the launcher is linked and says what is missing when pressed, asking the launcher
    nothing (`tst_publish`'s ninth case, which fails with the old condition). **Re-verified 15 October 2026** after a
    restart had cleared the run's node and logs: the launcher's 240 tests (10 ignored) and all eight QQ suites (73
    cases) pass; the deployed Studio is not yet rebuilt with the fix (`build.ps1 -Deploy`, the owner's to run). **Not
    yet:** the owner's first publish from QQ Studio started by a launcher built with this change (P3.4 stays unchecked
    until then); an installer with it (the release bundles the Qontrol helper, `tauri.bundle.conf.json`). **Next:** the owner's run, then P3.5.

74. **15 October 2026: the owner's first publish was refused, and a refusal now says why.** On branch
    `session/qq-p3.4-publish`, after item 73's commit. A restart had cleared item 73's node and logs, so a fresh
    `demiurge-node --dev --tmp` was started, `5DSND…LUV34` funded with 5,000 test CGT from Alice (total issuance
    unchanged) and the test launcher started. **The owner's run:** Publish in QQ Studio, on the Qontrol project
    `first-light`, approved in the launcher's dialog, ended "Not published: chain rpc error: the mint 0xa761cbb4bf…",
    cut off inside the hash: the Studio's notice was one elided line, and the launcher logged only the refusal's code
    (`rpc`). **Found:** the mint is in no block of that chain, so the node refused it at submission. Ruled out by measurement: a
    stale connection (a scratch test held one for ten minutes and its transaction still finalised); the live host check
    on the same node passed (mint, revise, unchanged, decline); payloads over 256 bytes are already hashed by
    `frame-decode` before the vault signs. **Shown:** the node refuses any transaction from an account it holds
    nothing for as `1010: Invalid Transaction: Inability to pay some fees`, on a chain with no fees (OPEN-4); and a
    publish is signed by the vault's **first** account (`host/studio.rs`), which need not be the one that was funded.
    That is the most likely cause; it is confirmed when the owner's retry names the account. **Built:** a new error,
    `unfunded` (`QorError::Unfunded`, a sentence naming the account and the chain): `sign_and_finalise` refuses an
    account holding nothing on the connected chain before anyone is asked, for every path through it (assets, sales,
    trades, `qor://pay`); and a mint whose deposits exceed the free balance is refused the same way, saying both
    amounts. The launcher's log line for a refused product request now carries its message. QQ Studio's notice wraps
    to two lines and shows the whole of it on hover. **Evidence:** the live test
    `an_account_that_cannot_pay_is_told_so_before_anyone_is_asked` (nothing on chain, then 200 CGT against about
    1,100 CGT of deposits: refused, account and chain named, nobody asked, nothing written, nonce 0), proven to fail
    with the deposit check disabled (the 200 CGT mint was asked for and failed on chain with
    `Balances::InsufficientBalance`); `tst_publish`'s window case now hovers a long refusal and reads it whole.
    **Also run on 15 October, after the change:** the launcher's 240 tests (11 ignored) under `qontrol-no-skips`,
    format and clippy clean; the frontend's type check and build; the ten browser checks, counts unchanged; all eight
    live chain tests together against the node (420 s) and the live host check; all eight QQ suites (73 cases), the hover case proven to fail with the tooltip switched off. **Not yet:** the owner's
    retry, on a launcher rebuilt with this change; QQ Studio redeployed with it (`build.ps1 -Deploy`, the owner's to
    run, since this assistant's permissions refuse it). **Next:** the owner retries; if the account named is not
    `5DSND…LUV34`, fund the named one, or choose which account publishes (a decision for the owner: the first account, or the one shown in the rail).
    **Later the same day.** The owner pushed both commits and pull request #29 was opened against `main`. **The retry,
    on the rebuilt launcher, confirmed the cause:** three Publishes were refused as `unfunded`, naming
    `5Gx19bfX6PpiLaWL3JwuFrLWFDhZ6pFDCP8xHu5FyAZ5sPwt` (the vault's first account) holding 0.00 CGT against 1,003.75 CGT
    of deposits, and nothing was asked or signed; that account was then funded with 5,000 test CGT (total issuance
    unchanged). The refusal's sentence had a run of spaces where a line continuation was lost; fixed, and the live test
    now asserts each message is one sentence. **CI on #29:** chain, QOR ID, ARQADE, coverage and **the launcher, the
    product host's first run on Linux, passed**; the **security job failed** on advisories published that week, not on
    this change: `rustls` 0.23.45 (RUSTSEC-2026-0346 to 0349) and `quinn-proto` 0.11.17 (RUSTSEC-2026-0338, 0340,
    0343), both updated to their fixed releases (`rustls` in the launcher, QOR ID and the chain; `quinn-proto` in the
    launcher), and `ruzstd` 0.8.3 (RUSTSEC-2026-0344), which no update removes and the launcher never builds: **ADR-089,
    Proposed**, puts the options to the owner. Until it is decided the security job fails on every branch, `main`
    included. **Next:** the owner publishes again (the account is now funded); the owner decides ADR-089 and which
    account publishes.
    **Verified that afternoon, then committed** (the dependency updates, the one-sentence fix and ADR-089) as the
    branch's third commit, for the owner to push. `cargo test --workspace --locked` in `chain/`, without
    `SKIP_WASM_BUILD`: 150 passed. The launcher's `cargo test --locked --features qontrol-no-skips`: 240 passed, 11
    ignored; of those, against `demiurge-node --dev --tmp` (the release build of 4 October; `chain/` is unchanged
    since but for `rustls` in its lockfile), the seven `chain::live` tests, the publish through the host's channel
    (`host::tests::a_project_is_minted_then_revised_then_left_alone_through_the_channel`), the OS keychain and the
    Hello probe passed; `qq::tests::qq_studio_starts_on_a_project` was not run (it opens QQ Studio on the desktop).
    QOR ID: 62 passed; the other 81 failed only for want of `DATABASE_URL` and 29 need Redis, since this PC runs no
    Postgres, Redis or Docker, so those 110 are left to CI. `cargo audit`: the launcher reports RUSTSEC-2026-0344 alone,
    QOR ID nothing beyond its ignore, the Qontrol helper nothing, and the chain (report-only) no `rustls` or
    `quinn-proto` advisory; `cargo tree -i ruzstd --locked --target all -e all` prints nothing. On Demiurge Devnet
    (block 170,917, `spec_version` 8) `5Gx19…5sPwt` holds **35,000** test CGT free and nothing reserved, not the 5,000
    recorded above; who sent the difference is not established. Nothing reserved means no publish has succeeded.
    **`main` is already red:** its scheduled CI runs failed the security job on 13, 14 and 15 October (the launcher's
    audit each day, QOR ID's on the 15th; the run of 12 October passed), so ADR-089's context now says so.
    **The owner's decisions, the same evening ("Go with best recommendation").** **ADR-089 accepted, option A, not
    built:** writing `tools/qor-launcher/src-tauri/.cargo/audit.toml` (the one-advisory ignore) was refused by this
    assistant's permission system as a weakening of security. The CI condition step and the `GATES.toml` change-log
    entry were written, then taken back out so the tree claims no ignore it lacks; they are kept as a patch outside the
    repository. Until the owner makes the ignore file or allows it to be made, the security job stays red. **ADR-090
    accepted and built:** a Cartridge is revised by the vault account holding it, whatever the rail shows, and a first
    publish signs as the account the rail shows (`Vault::set_active` and `active_address`, the command
    `vault_set_active`, the interface's store telling the host on every change). Verified: the launcher's 241 tests (one
    new: `the_active_account_is_the_first_until_another_of_its_own_is_chosen`), clippy `-D warnings` and fmt clean; the
    frontend's type check and build; all ten browser checks (readability 361); the live publish test, now publishing from
    a picked second account and revising with the first picked, passed against `demiurge-node --dev --tmp` and failed
    against each wrong signer planted in turn (the first account for a first publish: `unfunded`; the rail's account for
    a revision: `unfunded`). **The owner pushed `e540d7c`; CI on #29 (run 38780816958):** chain, QOR ID, the launcher,
    ARQADE and coverage pass; security fails at "Audit launcher host dependencies" on RUSTSEC-2026-0344 (`ruzstd`) alone,
    and QOR ID's audit, red on `main` that morning, passes with the `rustls` update. ADR-090's work is committed after
    it, for the owner to push. **ADR-089 built, the owner having given the permission** ("retry ADR-089 file with
    permission"): `tools/qor-launcher/src-tauri/.cargo/audit.toml` ignores RUSTSEC-2026-0344 alone; the security job's
    new step "Condition of the ruzstd ignore" fails if `cargo tree --locked -i ruzstd --target all -e all` prints
    anything or `ruzstd` leaves `Cargo.lock`; `GATES.toml` logs it (`ci-exemption`, 2026-10-15). Verified on this PC: the
    launcher's `cargo audit` exits 0 (its 15 warnings unchanged); the step, extracted from `ci.yml` and run, passes, and
    fails with its message on each planted condition (a crate that is built, `rustls`, in `ruzstd`'s place; a crate absent
    from the lockfile); `GATES.toml` and `ci.yml` parse; the launcher's 15 gates tests pass. **Next:** the owner pushes
    (CI on #29 is the proof that the security job is green); the owner publishes again.
    **That night.** The owner pushed; **every check on #29's last commit (`1f89400`) passed, security included**, and the
    owner merged #29 (`main` at `455b27d`). For the owner's retry, the launcher's saved chain was the local node
    (`ws://127.0.0.1:9944`, in its `settings.json`), and the node of the afternoon was gone with its `--tmp` state, so a
    fresh `demiurge-node --dev --tmp` was started, `5Gx19…5sPwt` funded with 5,000 test CGT from Alice (total issuance
    unchanged; a script that refuses any chain not named as a development chain) and the launcher started from
    `1f89400` (`npm run app:dev`). **The owner's run:** a project, "QQTest1", minted from the launcher (block 174:
    `drc369.mint`, the account's singles collection created, 1,004.375 CGT held as deposits, nonce 1), so the launcher's
    signing path, the funding and the dialog work. **Publishing from QQ Studio did not:** the launcher's log shows the
    Studio connecting twice over the host channel and closing once with code 0, and **no request reaching the
    launcher** (a refused request is logged with its message since item 74; an accepted one is not logged at all), and
    no further transaction on the chain. **What the Studio said is not recorded**, so the cause is not established.
    Three candidates fit what was measured, none confirmed: (1) the deployed QQ Studio
    (`%LOCALAPPDATA%\qq-studio\qq-studio.exe`, 10 October 12:39) predates two Studio fixes still waiting on
    `build.ps1 -Deploy` (Publish pressable and saying what is missing; a long refusal shown whole), so a precondition
    may have stopped it silently; (2) the Studio was on a project that is not chosen or not versioned, which it refuses
    without asking the launcher; (3) the Studio was on QQTest1 at the commit just minted, which the launcher answers
    `unchanged` with nothing asked and nothing logged. **P3.4 stays unticked.** **Next:** the owner runs
    `pwsh products/qq/build.ps1 -Deploy`, publishes again from the Studio and reports its notice word for word. **Built
    so the next run leaves evidence either way:** the launcher now logs an answered product request as well as a
    refused one, by product and method only, never its parameters or answer (`host/session.rs`; the host's 15 tests,
    its log check among them, pass; clippy and fmt clean).

75. **15 October 2026: Social decided (ADR-091) and its service built, not deployed; QOR ID ready for it.** On branch
    `session/social-foundation`, from `main` at `455b27d`. The owner asked for "an engaging and full featured social
    networking / high quality next gen chatting / socialization system with complete personalized profiles per QOR ID
    (username) and customizable aspects never before seen in a chatting environment", premium and seamless in onboarding,
    and chose the recommendation in each of four questions: **a service of its own** (`services/social/`, now active
    scope in AGENTS.md section 4), **server-held messages sealed at rest with end-to-end encrypted DMs in release 2**,
    **the launcher first**, and **a one-time username claim** for an account on an automatic `key_` name (amending
    ADR-075). ADR-091 records them; `docs/blueprints/social.md` is the design; L7.3 is brought forward of M8 and its steps
    are the new track L8 (L8.1 to L8.7 are release 1), counted by a new `social` gate.
    **QOR ID (L8.1), built and tested, not deployed:** `POST /oauth/introspect` for a registered service (a client with
    `"introspect": true`, a secret and no redirect URI; the registry refuses any other shape); `POST
    /api/v1/profile/username`, the one-time claim, from a person's own session, which renames their live sessions so the
    next tokens carry the name, audited; `GET /api/v1/users/{username}`, another person's name, avatar, level, kind and
    joining date; migration 023 (`username_chosen`, false for the names QOR ID made). **Found on the way:** an exact name
    clash is refused by the older constraint `unique_qor_id`, which no path recognised, so a race reported a database
    error; one helper (`name_taken`) now recognises both constraints everywhere. **Evidence:** 176 tests with Postgres 16
    and Redis 7.4, `--include-ignored` (172 before; four new, one extended), clippy and fmt clean; each new test failed
    against a planted fault (introspection accepting an app's token; a second claim; sessions left on the old name; the
    service secret logged, caught by the log check).
    **Social (L8.2), built and tested, not deployed:** `services/social/`, Rust on QOR ID's stack: one WebSocket per
    client (`hello` with the QOR ID token, checked by introspection under Social's own credential and never kept;
    `welcome`; typed requests; pushed events; closed at the token's expiry); public and private rooms with owner and
    moderator roles, invitations, direct rooms of exactly two; messages sealed at rest (XChaCha20-Poly1305, versioned
    keys, bound to room and sequence), replies, reactions, edits, deletes that keep a message's place and drop its words,
    read positions and unread counts, mentions within a room; blocks that stop messages, presence, invitations and
    direct rooms; reports; limits per person across sockets; NFKC-cleaned text; profiles (display line, bio, accent,
    banner from the QFX presets, a showcase of DRC-369 assets, status), presence (online, away, invisible, playing).
    **Found on the way:** tungstenite prints every WebSocket frame with its payload at debug and trace, which would put
    hello tokens and people's words in a log; the service now holds it to info whatever `RUST_LOG` says (`may_log`), and
    the log check captures through that same filter and also looks for secrets printed as bytes. **Evidence:** 29 tests
    against Postgres 16 and a real WebSocket on a loopback port, run four times, clippy and fmt clean; each rule failed
    against a planted fault first (history read without membership; a blocker still hearing the blocked; the log filter
    off, when tokens, words, bios, notes and status lines reached the log; words stored plain; a made name posting; a
    CRLF migration; a mention reaching someone outside the room, after the test was corrected to give that person a room
    of their own, without which it could not fail). CI gains a `social` job; GATES gains `[suites.social]`,
    `social.tests` and `social.qor-id-tests` (tightening). **Social's dependency audit:** `rsa` (RUSTSEC-2023-0071) is in
    its lockfile through sqlx's optional MySQL driver and not compiled, as in QOR ID; the owner chose QOR ID's terms the
    same night ("Ignore it on QOR ID's terms"), so `services/social/.cargo/audit.toml` ignores that one advisory and a CI
    step fails if `rsa` is ever built (run here: passes, and fails with the crate swapped for one that is built); logged
    in GATES.toml. **Not done:** the launcher's Social surface (L8.5), the deployment (L8.6), the retention rule and
    content policy (L8.7). No L8 item is ticked until CI has run on it. A trap found while proving faults: a helper that
    restores a planted file with its older timestamp leaves cargo's build of the fault in place, so the next run tests
    the fault, not the fix; restores now touch the file. Pull request #31. **Next:** CI on #31; then L8.5, the
    launcher's Social surface.

76. **15 October 2026, late: #31 merged and QOR ID's side of Social live; the launcher's Social surface built (L8.5).**
    **Merged:** the owner merged pull request #31 at `7821e4d` (`main` at `76ffbf4`, CI green). It deployed QOR ID from
    `main`: `POST /oauth/introspect` answers an unregistered caller `invalid_client` at `id.qorsync.dev`, so L8.1 is live,
    migration 023 applied in production; L8.1 and L8.2 are ticked. Social's audit commit was pushed after that merge, so
    it is pull request #32 on its own (`session/social-audit`). Pull request #30's QOR ID job had failed before any step
    (a Docker image pull) and was re-run.
    **The launcher's Social surface, on branch `session/social-launcher`, not merged:** the host holds the one socket and
    the token (`src-tauri/src/social.rs`): jsonrpsee's WebSocket transport, already in the graph through subxt, so the
    lockfile gains no package; reconnects at 1, 2, 4 … 30 s; an expired token is swapped and hello said again at once; a
    session that is not live is refreshed once, then reported `signed_out`; a request never hangs past a lost connection.
    `social_url` joins the settings (`wss://`, or `ws://` on this machine; `QOR_SOCIAL_URL` overrides; default
    `ws://127.0.0.1:8090/v1/socket` until L8.6). Commands `social_connect`, `social_disconnect`, `social_status`,
    `social_request` and `qor_claim_username`; refusals reach the interface as `kind: "social"` with Social's own
    `code`. The interface (`src/social/`): connected while the shell is open, so the rail counts unread from anywhere;
    the claim on entry (a new QOR ID, or a made name once, then Social told again who this is); conversations, direct and
    rooms apart, with invitations; a room's messages with day rules, grouping, replies, reactions, edits, deletes,
    @mentions marked and completed, typing, and older pages; profile cards and an editor with a live preview, banners in
    the six QFX presets drawn in CSS, an accent refused when it would not read on the theme, and a showcase chosen from the
    account's own assets (another person's is shown unchecked: a profile carries no chain account yet); dialogs for a new
    room, finding one, a direct message, inviting, and room settings with ambience. Onboarding's claim card lost copy left
    from before ADR-075 (a hyphen allowed, "a number is added", `name#…`). **Evidence:** the launcher's 249 tests (eight
    new for the socket, among them its own log check, proven to fail with the hello frame logged; one for settings), fmt
    and clippy clean; the ten browser checks, counts unchanged; the new `check-social-view.mjs`, 29 checks, each of three
    planted faults caught (an extra field in a send, the rail counting nothing, every accent accepted); it also caught two
    faults of the build itself, fixed before commit: a room opened automatically on welcome, which marked it read unseen,
    and its history asked for twice. **End to end on this PC, the same night:** a real QOR ID (`cargo run`, Postgres and
    Redis in Docker, Social registered as a service in `QOR_OAUTH_CLIENTS`) and a real Social (`cargo run`, its own
    database), driven over real WebSockets by a script: a token QOR ID never issued refused; two people welcomed by the
    names QOR ID vouches for; a room made and joined; a message sealed, delivered and opened; an @mention delivered; a
    direct message to its one recipient; an account put on a made name told so at hello and refused posting
    (`name_not_chosen`); QOR ID's one-time claim taken and a second refused; and, after a token refresh, Social welcoming
    the chosen name and letting it post: 12 of 12. Two script errors were the script's (it read the sender's own echo, and
    reconnected with the same token inside Social's 30-second introspection cache, which the launcher avoids by
    refreshing after a claim). Social ran with `RUST_LOG` asking tungstenite for trace: its log held no token, no words,
    no client secret and no tungstenite line. **Deployment readiness (L8.6):** `GET /ready` (asks the database; tested to
    answer 503 once the database is closed, while `/health` still says 200), `services/social/Dockerfile` (built here,
    149 MB; started against a fresh database, migrated it and answered `/ready` 200), and `services/social/DEPLOY-RAILWAY.md`,
    the owner's steps: a `Social-Postgres` service, the `social` service, two secrets generated on the owner's machine,
    and Social's entry in QOR ID's `QOR_OAUTH_CLIENTS`. **Not done:** the launcher has not run against a running Social
    (each side of that seam is tested against a stand-in of the other); that waits on L8.6, when the owner's first look
    is the proof. **Next:** CI on #33; then L8.6, deploying Social beside QOR ID by `services/social/DEPLOY-RAILWAY.md`
    (owner steps in Railway).

77. **16 October 2026: Social deployed (L8.6) and the launcher pointed at it.** The owner merged #30, #32 and #33
    (`main` at `fa7433c`). In Railway project `demiurge`, through Railway's connector: Social's database from the
    `postgres` template (Railway named it `Postgres-Yj3Q`, and renaming it would break `social`'s `DATABASE_URL`), the
    `social` service (root `/services/social`, its Dockerfile, healthcheck `/ready`, restart on failure, watch paths,
    `PORT` 8090) with its non-secret variables, and the domain `social-production-e05c.up.railway.app`. The two secrets
    were written to a file on the owner's Desktop, never shown in the session, and pasted by the owner. **Two faults on
    the way, found from outside:** (1) QOR ID still refused Social's credential after several redeploys; the owner pasted
    `QOR_OAUTH_CLIENTS` (it holds only names, addresses and hashes), which showed the Social entry had never been added, so
    it was added through the connector; (2) Social then answered every hello `unavailable`, and QOR ID's HTTP log showed
    Social's call refused with 401: Social held a different secret, one the owner had generated with the guide's own
    command and pasted into the chat. Social's secret was replaced with the registered one, so the secret seen in the
    chat was never registered with QOR ID and authenticates nothing. **Checked from outside:** `/ready` 200, and a hello
    with a token that is not live answered `unauthorized`, which needs QOR ID to accept Social's credential. L8.6 is
    ticked. **The launcher** now defaults to `wss://social-production-e05c.up.railway.app/v1/socket`
    (`DEFAULT_SOCIAL`; a saved `social_url` or `QOR_SOCIAL_URL` still wins). `services/social/DEPLOY-RAILWAY.md` is now the
    record of the setup, with how to rotate each secret and how to tell the two refusals apart. **A trap:** a variable
    changed in Railway's dashboard does not reach a service until it next deploys; the connector's `redeploy` does that.
    **A launcher fault found on the first start against it:** `social_connect` was a synchronous Tauri command, so it ran
    on the main thread, outside Tokio, and `tokio::spawn` panicked there and aborted the launcher at start-up ("there is no
    reactor running"). The command is now async, and `Social::connect` refuses outside a runtime instead of panicking
    (`connecting_outside_a_runtime_is_refused_not_a_panic`). The stand-in server tests could not see it: they run inside a
    runtime. **Not done:** the owner's first conversation in the launcher (it needs a launcher built with this change), and the
    owner's decisions before anyone else uses Social (L8.7). **Next:** the owner's first look; then L8.7, and L8.8 (end-to-end
    encrypted direct messages, its own record first).

78. **16 October 2026: the owner's first conversation, and World, friends and circles, and translation on the reader's PC
    (ADR-092), on branch `session/social-world`, not merged.** The owner merged #34 (`main` at `76a410f`, CI green; the
    PR's own run had one launcher-job failure, a browser check that found no page to inspect, which `main`'s run after the
    merge did not repeat). Then the owner's first conversation in a launcher built from #34 against the deployed Social:
    "Successful. Looking great so far", so L8.3 to L8.5 are ticked. **One more launcher fault on the way:** for two hours
    every debug build's window failed to open (`failed to create webview: WebView2 error … 0x80070057`, "The parameter is
    incorrect"), with the WebView2 runtime that updated the night before or the one before it, with a fresh profile,
    with the code from before the fix and from another path; the installed 0.1.8 opened its window throughout, and by
    16:05 every build did again. No cause was found; it was this PC, not the code. **The owner's direction**, and four
    answers: "Friends + circles", translation in "World and public rooms", World to "Keep everything", and for the
    translator, none of the four offered: "The best possible option with zero costs associated with its use". Recorded
    as ADR-092 (accepted), which meets that with translation on each reader's PC.
    **Social (`services/social/`):** migration 002 (World, made once; `mutes`; `lang` on messages and `language` on
    profiles; `friend_requests`, `friendships`, `circles`, `circle_members`), checked over a database populated under 001
    as well as fresh; World joined at every hello at its newest message, never left, renamed, invited to or removed from,
    making nobody a contact, typing not fanned out, delivered to whoever is connected less those who block the author,
    its members listed as who is here, its own pace (5 per 20 s); its moderators from `SOCIAL_WORLD_MODERATORS`, a name
    only once chosen, who delete and mute (10 min, 1 h, 1 day, 1 week); friends asked, accepted, declined, cancelled
    and removed, a block ending both friendship and requests, friends seeing presence; circles (20, of up to 100
    friends) private to their owner and opening a private room. 37 tests (eight new, the newcomer test changed on
    purpose: World is in every welcome), twelve planted faults each caught, the log check driving the new flows with
    World's words among its secrets. **The launcher:** `src-tauri/src/translate.rs` reads Mozilla's
    `translations-models-v2` and `translations-wasm-v2`, plans a pair (two models through English when neither is
    English), downloads each file once with progress, checks its SHA-256 packed and unpacked with unpacking capped one
    byte past the published size, keeps it by hash in the local data folder, and reads only files checked this run;
    the engine is pinned by its hash to Firefox release's `bergamot-translator.js` v0.6.0, kept with its MPL-2.0 licence
    in `public/translate/`, beside a worker of our own after Firefox's; `whatlang` tells a message's language on the
    sender's PC. New crates `zstd` (the C library, not `ruzstd`), `whatlang`, `sys-locale`. The CSP gains
    `'wasm-unsafe-eval'` and `worker-src 'self'`. The screen: World pinned first and opened with Social; Chats and Friends
    tabs, requests, add by name, friends online first, circles as chips with group chat; messages in another language
    shown translated, "Translated from Spanish", the original a click away, a language without models saying so; a
    switch in the header; what is being downloaded, and how large, in a bar; each author's username and "Lv n" beside the
    name; a moderator's mute menu; a muted reader told in place of the box; the rail leaving World's chatter out; the
    language picker in the profile editor. **Evidence:** 264 host tests (14 new for translation, nine planted faults
    each caught after three of its tests were sharpened, and one by-hand test that downloads Mozilla's real files);
    clippy and fmt clean; `check-social-view.mjs` 52 checks; the other ten browser checks pass; and **the real engine
    translated on this PC** with Mozilla's real files, fetched and checked by the launcher's own code:
    "Hola a todos, ¿qué tal estáis hoy? Me encanta este juego." became "Hi everyone, how are you today? I love this
    game." in 190 ms after a 70 ms load (`check-translate-engine.mjs`, by hand). **Not done:** merging; Social deploys
    from `main` and runs migration 002 at start; `SOCIAL_WORLD_MODERATORS` set on Railway (the owner's username, when
    the owner says which); a launcher built from `main` for the owner's look at World; and the rest of L8.7, the
    content policy, before anyone else uses Social.

79. **16 October 2026, late: room levels, Elite accounts and godmode (ADR-093), on the same branch and pull request
    (#35), not merged.** #35 opened with three commits; its CI passed every job but QOR ID's, which failed twice before
    any step on `toomanyrequests: Data limit exceeded` pulling Postgres from Amazon ECR Public (§5's trap; #35 changes
    nothing in QOR ID). The owner: "godmode is the account that moderates chat", and room levels set by room owners and
    "Elite" accounts, Elite set only by godmode; the recommended answer to each of four questions (levels Owner, Admin,
    Mod, VIP, Member; Elite staff everywhere; godmode above everyone; badges beside names), recorded as ADR-093. **On
    Railway:** `SOCIAL_WORLD_MODERATORS=godmode` and `SOCIAL_GODMODE=godmode` on `social`, set without a redeploy; the
    service reads them when #35 deploys. **Social:** migration 003 (the five levels; `elites`); `authority`, a person's
    standing in a room (godmode 6, an Elite at least an Admin's 4, else the level); each level acts only below itself
    and gives only below its own; only the Owner or godmode hands a room on, the old Owner becoming an Admin; settings an
    Admin's, inviting to a private room a Mod's; deleting, muting and removing below oneself; mutes in any room but a
    direct one (`rooms.mute | unmute`, `room.muted`; World's `world.mute` kept); `elite.grant | revoke | list`, godmode
    only; a profile's `status`; `levels` in a history page, `level` on a new message, `muted_until` per room; World's
    configured moderators a floor at hello, never undoing an appointed level; VIPs free of World's pace; and no level or
    standing reads a room it is not in (ADR-093 decision 5). Two existing tests changed on purpose: a Mod no longer
    deletes the Owner's words, and an Owner who hands on becomes an Admin. 40 tests; thirteen planted faults each
    caught. **The launcher:** `src/social/Levels.tsx` (ranks, what a reader may give, the badges); badges on messages,
    in member lists and on cards; the member list's level menu, Mute and Remove offered only where Social would allow;
    Delete and Mute on a message only below the reader; a muted reader told in any room; godmode offered Make and
    Remove Elite on a card. `check-social-view.mjs` 59 checks, with the reader as godmode; two planted faults caught (a
    third first went unseen because the fault did not compile, and was caught once it did). 264 host tests.
    **Not done:** merging #35 (and QOR ID's job re-run once ECR allows); then a launcher built from `main` for the owner;
    the content policy (L8.7).

80. **17 October 2026 (UTC; the owner's evening of the 16th): #35 merged and live.** CI on #35's last commit: every
    job passed; Social's first failed before any step on `toomanyrequests` pulling Postgres (§5), and passed when re-run.
    The owner merged at 02:13 UTC (`main` at `9a3895f`). **Railway** built and deployed `social` from it, `SUCCESS` at
    02:14:40; its log shows "Social is listening" only after migrations run, so 002 and 003 applied to the live database;
    `SOCIAL_WORLD_MODERATORS` and `SOCIAL_GODMODE` (`godmode`) are in effect from that deploy. **Checked from outside:**
    `/ready` 200; a hello with a token that is not live answered `unauthorized`. A launcher built from `main` on the owner's
    PC (window opened, no WebView2 fault) connected at 02:15:57 (`client=qor-launcher/0.1.8` in Social's log). **Not
    done:** the owner's own test of World, translation, friends and levels ("We'll test later on"), which ticks L8.11 to
    L8.15; the content policy (L8.7). **Next, by the owner's direction the same night:** the Library, a SteamOS-style view
    in the launcher with a media player for every video and image format, local folders browsed and searched, PDFs and
    documents viewed and edited, and "QT based options for all things from now on".

81. **16 and 17 October 2026: the Library and QOR Media (ADR-094), L9.1 to L9.4 built on branch `session/library`, not
    merged.** The owner: "begin working on the Library. The goal is a steamOS style in-launcher view", with "a high
    quality premium feeling media player that supports all video and image codecs and formats", files opened from the
    Library, local folders browsed and searched "with zero friction", PDFs and documents viewed and edited, and "QT based
    options for all things from now on"; the recommended answer to each of four questions (a Qt player docked in the
    launcher; PDF and text now, Office next; everything in one place; the launcher's own index of chosen folders),
    recorded as ADR-094. **The index (L9.1, `9123659`):** `src-tauri/src/library/`, SQLite with FTS5 in the launcher's
    data folder, folders chosen (Videos, Music, Pictures and Documents offered), scanned at start and watched, pictures'
    size and camera, music tags, a text document's first words, thumbnails for pictures; every typed word searched as a
    prefix, quoted so search operators are words. **The view (L9.2, `6183669`):** `src/library/LibraryView.tsx`, a hero
    with Resume, shelves by kind, tabs, Folders, search on `/`, a focus ring moved by the arrow keys; and since this item
    **a game controller** (D-pad or left stick, held directions repeating, A opening, B going back), read through the
    Gamepad API because WebView2 turns controller input into nothing. The view's header had claimed the D-pad "arrives as
    the same keys"; it did not, and the claim is gone. `check-library-view.mjs` 27 checks (22 before), four planted
    controller faults each caught (one only after its plant was made to compile: the first left a variable unused, so
    the old build was what ran). **The launcher's side of QOR Media:** `src-tauri/src/media.rs`, the `media.next` queue,
    `media_place` in physical pixels, `media_hide`, and `media_theme`, the launcher's colour tokens as `#rrggbb`, newest
    kept, sent again when the theme changes (its test missed a planted fault, a colour's digits unchecked, until two
    cases were added); reports (where playback stopped, a file's length and size, closed) ride on
    QOR Media's next `media.next`, so one arrives within one wait (25 s). The blueprint had named a `media.report`
    method; there is none, and it says so now. **QOR Media (L9.3, L9.4):** `tools/qor-media/`, a Qt 6.12 program in
    C++20 and QML: the player (resume, scrubbing with the time under the pointer, skip, speed, volume, audio and
    subtitle tracks, full screen, every key), the viewer (zoom about the pointer, drag, 1:1, the folder's others in the
    order people read numbers, turn, slideshow), the editor (UTF-8 or UTF-16, byte-order mark and CRLF written back as
    found, a file it could not read never saved over, line numbers, Markdown preview, find, a question before unsaved
    words are lost), and a plain page for PDF and Office documents with the program Windows chooses. 34 tests in four
    Qt Test suites, among them the whole program opening a generated tone, a picture, Markdown and a PDF, Windows' own
    sample video decoding through FFmpeg, and docking as a child of a window known only by its handle, in logical pixels
    at this PC's 125% scaling; ten planted faults each caught. Deployed to `%LOCALAPPDATA%\qor-media` (137 MB, with
    FFmpeg's libraries). **The whole path:** `media::tests::qor_media_started_by_the_launcher_plays_and_reports_to_the_library`
    (opt-in, `--ignored`) starts the deployed QOR Media through the product host, opens a generated tone, and sees its
    length and where it stopped reach a Library: passed in 3.5 s. **Launcher:** 277 host tests pass (13 ignored), twelve
    browser checks. **CI:** Social's job could not pull Postgres from Amazon ECR Public five times running on this pull
    request, so CI's service images now come from `mirror.gcr.io` (§5). **Gates:** `library.tests` (thirteen launcher tests) and `library.view` (the view check, as suite
    `library_view`), a tightening logged in `GATES.toml`. **Not done:** the owner's first look in a running launcher (nobody has opened a file from the Library
    on screen yet); PDF (L9.5; this said Qt PDF was missing from the owner's Qt, which was wrong: see item 82); video thumbnails
    on tiles; a controller inside QOR Media (Qt 6 has no gamepad module); the pull request.

82. **17 October 2026: #37 merged; the Library and QOR Media in `main`.** On #37, Social's job failed five times
    running at "Initialize containers", Amazon ECR Public refusing `postgres:16-alpine` ("toomanyrequests: Data limit
    exceeded"); QOR ID's twice. CI's service images moved to `mirror.gcr.io/library/` (`808f7d6`; the same digests for
    both tags on both mirrors, no credential), and the next run passed every job first time. #36 had been merged
    meanwhile: the three conflicting documents were the ones both changed, and #37's side already held #36's text, so
    it was kept (`git diff` against `main` removed nothing of #36's). The owner merged #37 at 12:59 UTC (`95e554e`);
    CI on `main` at `95e554e` passed every job (two scheduled jobs skipped). **A correction:** items 81, OWNER.md and the
    roadmap said Qt PDF was missing from the owner's Qt and asked the owner to add it. It was there all along:
    `Qt6Pdf.dll`, `Qt6PdfQuick` and the `QtQuick.Pdf` module, dated 27 September 2026. The check that said otherwise
    printed nothing and was read as "absent"; read a check's output before reporting what it found. **Not done:** the
    owner's first look at the Library in a running launcher (L9.1 to L9.4 stay unticked until then); PDF (L9.5), which
    can start.

83. **17 October 2026: launcher 0.1.9 built.** The owner: "build an up to date launcher to be installed onto my desktop".
    The version raised from 0.1.8 to 0.1.9 (`package.json`, `tauri.conf.json`, `Cargo.toml` and both lockfiles;
    `package-lock.json` had still said 0.1.6), on `main` at `95e554e` (#37). `npm run app:build` built the release in 42
    minutes and both Windows bundles: `QOR Launcher_0.1.9_x64-setup.exe` (7,240,336 bytes, SHA-256
    `CEF47E34211A719F3ACC3E05CA1957469EA650A3290A7F16D12A749B01D2D1C8`), copied to the owner's Desktop, and the MSI. It
    carries everything in `main` since 0.1.8: avatars and rings, Social with World, friends, translation and levels,
    and the Library. Unsigned (L6), so Windows warns. QOR Media is not in the installer: the Library opens files in the
    copy at `%LOCALAPPDATA%\qor-media`, deployed on this PC, and in the program Windows chooses where there is none.
    **Not done:** the owner installing it and looking; a launcher on another PC has no QOR Media until it is deployed
    there (an installer for QOR Media, or bundling it, is not built).

84. **17 October 2026: the Mesh (ADR-095), and L10.1 built on branch `session/mesh`, not merged.** The owner: "begin
    working on the Mesh and ensure that earning CGT is a main mechanic ... like a in launcher torrenting network which
    allows any QOR ID logged in user to earn CGT for seeding files and data for others to create a sort of cloud file
    storage mesh. please advise". Advised: what ADR-005, ADR-006 and ADR-047 already decide; what is open (U-6, and
    OPEN-1 to OPEN-3 for any issuance); and that seeders can be paid in release 1 from the uploader's escrow without any
    CGT being created. Four answers: uploaders' escrow (recommended), **plus private backup** (beyond the recommended
    published work alone), iroh (recommended), opt-in with limits (recommended). Recorded as **ADR-095**, with the design
    (`blueprints/mesh.md`), the proofs proposed for review (`architecture/MESH_PROOFS.md`: storage challenges against the
    root on chain, serving receipts signed by the receiver, copies counted by distinct QOR ID accounts, and the note that
    Aura gives no unbiased randomness), roadmap **L10** (carrying M8.1 and L7.4), a `mesh` gate, and U-6 pointing at the
    proposal. **iroh checked in its source:** `iroh` 1.3.0 and `iroh-blobs` 0.103.1, MIT or Apache-2.0, Rust 1.91;
    `IROH_BLOCK_SIZE = BlockSize::from_chunk_log(4)`, 16 KiB, ADR-047's granularity; plain BLAKE3, ADR-047's root, which
    is what `ContentRef::of` computes. Its README calls 0.10x "not yet considered production quality" (ADR-095 says so).
    **L10.1:** `src-tauri/src/mesh/` (the node: an iroh endpoint whose key is made on this PC and is never the vault's,
    an iroh-blobs file store in `<data>/mesh/store`, served only while seeding is on; a token bucket on every 16 KiB served
    keeps to the upload limit; the temporary content store's blobs taken in at start, when seeding is turned on and once
    a minute, each hashed first and referenced where it lies; a fetch by ticket proves the size before taking anything,
    so the disk limit is kept, and refuses a peer whose bytes changed), `mesh_status`, `mesh_held`, `mesh_configure`,
    `mesh_ticket` and `mesh_fetch`, a `Mesh` error, and the Mesh page (`src/mesh/MeshView.tsx`), which replaces the last
    placeholder screen: `views/Horizon.tsx` is gone. The launcher's minimum Rust moves from 1.88 to 1.91 for iroh; CI
    builds it on stable. **Tests:** ten host tests, two launchers on one PC over the loopback address with no relay;
    eight planted faults each caught (one, an unset disk limit, only once its test also checked that no peer was
    reached). `check-mesh-view.mjs`, 23 checks, four planted faults caught; it joins `npm run check` (thirteen). The
    gates gain `mesh.tests` and `mesh.view` (logged). 287 host tests pass (13 ignored). `cargo audit` passes; its new
    notices are "unmaintained" ones: `atomic-polyfill` and two old `proc-macro-error`s come with iroh, and `zvariant`,
    `zvariant_derive` and `zbus_names` with Tauri's Linux plugins, newly listed by the advisory database; left unexempted.
    **The disk:** drive X: filled to 0 bytes during this item's first test build (the owner's Steam library and Videos
    changed that morning; the launcher's debug build is 37 GB and the chain's 63 GB); it recovered to 7 to 25 GB as builds
    cleaned up. **`check-accessibility.mjs` flaked** under that load, a different backdrop's frame count each time (two
    failures in four runs), and passed twice running after. **Not done:** two PCs over the internet; L10.2 to L10.7; the owner's
    review of MESH_PROOFS.md, the pallet's name, and who keeps the takedown list.

85. **17 October 2026: ADR-096, and the chain's half of the Mesh (`pallet-mesh`, `mesh-proof`), on branch `session/mesh`,
    not merged.** Asked to review MESH_PROOFS.md, approve the pallet's name and say who keeps the takedown list, the owner
    answered: "Go with best and most recommended path forward." Recorded as **ADR-096**, each decision the option
    recommended: MESH_PROOFS.md accepted with its period (600 blocks), window (100) and missed-period limit (3); of the
    receipt rules, what the chain can see enforced and QOR ID membership waiting on an attestation; the module named
    `pallet-mesh`; the takedown list kept by the root origin; the chain's half built before discovery (L10.2), because
    finding who holds a root reads the escrow's assignments. U-6 is answered. The pending question of crediting godmode
    with 100,000,000 CGT is not a protocol decision and was not taken as one: the recommended path is a transfer from the
    faucet, which the owner signs, and godmode's chain address is still wanted. **Checked first:** the chain keeps the
    last 256 block hashes (frame-system 48's `SolochainDefaultConfig`; ADR-096 first said 2,400, corrected before it was
    committed), so the window closes well inside them; `blake3` 1.8.7, already in the chain's lockfile, has the `hazmat`
    functions a proof of one group needs, without `std`. **`chain/primitives/mesh-proof`** (`no_std`, built for
    `wasm32v1-none`): `prove` and `verify`, a 16 KiB group shown against its BLAKE3 root by the chaining values beside
    its path, split exactly as BLAKE3 splits; four tests over ten sizes, and the finding that the proof binds the group to
    the root but not to the size, so the size must be the chain's, never the seeder's (it is: the escrow's). The launcher
    will depend on it by path, so both sides run the same code. **`chain/pallets/mesh`**: escrows in a keyless account per
    escrow (`dmg/mesh`); `bind_node`, `open_escrow`, `top_up`, `close_escrow`, `take_assignment`, `leave_assignment`,
    `end_lapsed`, `answer_challenge`, `submit_receipts` (cumulative, so never paid twice; capped per receiver and per
    escrow; never the seeder or the uploader as receiver; signed over a context and the genesis hash), `set_takedown`,
    `forget_served`; the one ratio by `multiply_by_rational_with_rounding`, pinned at `u128::MAX`; the `MeshApi` runtime
    API (`challenge`, `receipt_message`, `period`). Sixteen tests with real BLAKE3 proofs and Sr25519 signatures; fourteen
    planted faults (twelve in the pallet, two in the proof crate) each caught. **The runtime:** `Mesh` at index 12,
    `spec_version` 9, ADR-096's values in `runtime/src/mesh.rs`; four tests in `runtime/tests/mesh.rs`; the ARQ Wallet's
    test that pinned spec version 8 now pins 9 and is renamed, on purpose. **The suite:** `cargo test --workspace` with the
    runtime's wasm built, 174 passed (150 before); clippy's three findings (a doubled bound, a complex type, a constant
    assertion) fixed. **The gates:** `mesh.decisions` counts ADR-096, `mesh.chain-tests` requires fifteen tests, and
    `beta.value-pallet-coverage` names `pallet-mesh` (logged). `cargo-llvm-cov` is not installed here, so its coverage is
    first measured by CI. PROTOCOL.md section 13 and `chain/README.md` describe it. **Not done:** the launcher answering
    challenges, signing receipts and taking assignments (L10.2 to L10.4's launcher half); the runtime upgrade to
    `spec_version` 9 on the devnet, which the owner signs with the sudo key after a rehearsal on a local chain.

86. **17 October 2026: the launcher's half of the Mesh's paid work (L10.2 to L10.4), and ADR-097, on branch
    `session/mesh`, not merged.** The owner said "Proceed". Item 85's suite was run first: 174 passed with the runtime
    built. **The decision (ADR-097, under the owner's delegation for engineering choices):** a seeder answers a challenge
    every period while nobody may be at the PC, so the vault gains a second standing approval beside ADR-016's arrival
    grant: the **Mesh grant**, asked once in a host dialog when seeding is turned on, for the active account, ended by
    turning seeding off. Under it the launcher signs `bind_node`, `take_assignment`, `leave_assignment`,
    `answer_challenge` and `submit_receipts`, and receipts, with no dialog; none can send the person's CGT (the chain
    charges no fees, OPEN-4; a ceiling is required before it does). Opening an escrow keeps its dialog. **The node is
    trusted for none of it:** each granted call is encoded by the launcher (pallet 12, call indices compiled in) and
    nothing is signed unless the node's metadata encodes the call it names back to the same bytes; a receipt's bytes are
    built from the genesis hash, never asked of the node. **The chain:** the pallet's period and answer window move to
    storage that defaults to ADR-096's values; only the `--dev` genesis sets 10 and 5 blocks (a minute), refused at
    genesis if they cannot work; the devnet gains the pallet by an upgrade with no genesis, so it keeps 600 and 100.
    `mesh-proof` gains `siblings` and `Subtree`, the same proof made a piece at a time, for a file too large to hold.
    **The launcher:** `src-tauri/src/chain/mesh.rs` (reads of escrows, assignments, nodes and the takedown list;
    `MeshApi` through `call_raw`; the granted calls; `mesh_open_escrow` with its dialog) and `src-tauri/src/mesh/paid.rs`
    (a worker every three seconds while seeding with the grant: bind, answer, claim, and every tenth round take on one
    copy an escrow still asks for, fetched and verified first; a minute's pause after a failed round; finding a root
    through the chain, its seeders first, then its uploader, each dialed by its bound node; receipts sent to the seeder
    over iroh's `demiurge/mesh-receipt/1` and checked there before they are kept). No grant is asked for where the chain
    has no Mesh, so on the devnet nothing is asked and seeding runs unpaid. **The page:** what the Mesh signs for, held for
    escrows, receipts waiting, what was paid; fetch by fingerprint; "Pay seeders to hold it" on one's own files, a form
    that suggests no price and sends exact Sparks. **Evidence:** a release node built with the new genesis; the live
    test `mesh::paid::tests::live` (ignored by default) run three times against it, passing each time in about four
    minutes: an uploader's escrow opened in its dialog, a seeder taking it on, answering the next period's challenge
    and paid exactly the storage price, a receiver finding the file by fingerprint alone and the seeder paid exactly
    1.171875 CGT for 300 KiB at 4 CGT a MiB; the seeder refusing a forged receipt, one for another seeder and one for a
    file it does not hold; a declined grant recording nothing; the local receipt bytes equal to the chain's; every level
    of the log captured and no recovery phrase in it. Three planted faults were each caught (the receipt's signature
    unchecked, by the live test; a proof missing its top sibling; a receipt message without the genesis hash).
    **Suites:** the chain's `cargo test --workspace` with the runtime built, 177 passed (174 before); the launcher's
    `cargo test --locked`, 292 passed, 14 ignored; `npm run check`, all thirteen checks (the Mesh page's 38, up from 23);
    fmt and clippy `-D warnings` clean on both. **Gates (tightening, logged):** `mesh.decisions` counts ADR-097,
    `mesh.chain-tests` requires 18, `mesh.tests` 13. Documents: PROTOCOL.md section 13, `chain/README.md`,
    DIRECTION.md L10.2 to L10.4, the Mesh blueprint (its stale "name awaiting approval" and gossip discovery), both ADR
    indexes, SYSTEMS, OWNER. **Checked from outside:** QOR ID, Social (at its Railway domain; `social.qorsync.dev`
    answers 525 and no document names it), ARQADE, and the devnet at `spec_version` 8. **Not done:** two PCs over the
    internet; closing an escrow from the launcher; refusing to serve a listed root already held (L10.7); the runtime
    upgrade to 9 on the devnet, which the owner signs; `archive-bZLIQp/`, which holds one zip, `gk_3.1.77_windows_amd64.zip`, left untouched.

87. **17 October 2026: launcher 0.1.10 published for testers, #40 merged, and runtime 9 rehearsed.** The owner asked for an
    installer testers download from a download page, holding everything the tests above need, and chose: testers on
    the devnet once it is upgraded, a GitHub Release with a page, and merging before building. **Merged:** #40 at
    `580aa9e`, after CI passed every job on its head `4508283` (the chain's job and the launcher's among them). **The
    installer:** version 0.1.10 (`package.json`, `package-lock.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`).
    The Windows installer now carries QOR Media: `scripts/stage-qor-media.mjs` runs `tools/qor-media/build.ps1 -Deploy`
    (its four Qt Test suites, 42 tests, passed each time) into `src-tauri/media-bundle/` (not tracked), mapped to
    `qor-media/` in the install folder by `tauri.bundle.conf.json`, and `media::find` looks there before a developer's
    deployed copy (its test renamed and extended; the library gate follows, logged). QOR Media and the Qontrol helper
    import `VCRUNTIME140.dll` (the launcher does not), so a post-install hook (`src-tauri/windows/installer-hooks.nsh`)
    runs the `vc_redist.x64.exe` windeployqt ships, quietly. The Mesh page gained a button that copies a file's whole
    fingerprint (39 view checks), which fetching by fingerprint needs. `npm run app:build` built both bundles from the
    tree of `4508283`; `git diff 4508283 matrix/main` is empty, so the installer is `main`'s at `580aa9e`. **The release:**
    `launcher-v0.1.10`, a pre-release on `QOR-MATRIX/demiurge-chain`, asset `QOR-Launcher_0.1.10_x64-setup.exe`,
    **68,976,975 bytes, SHA-256 `7b515d7515cea2bb6a067bbeaf4df69053f4a44b05462d408d1befe156e33489`**, downloaded back
    from its public URL (200) and matched. Its contents listed with 7-Zip: the launcher, the Qontrol helper and
    libgit2's licence, and QOR Media's 186 files with its FFmpeg backend. **The download page:** a private Artifact,
    `https://claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv`, which the owner shares: the download, the checksum and how to
    check it, SmartScreen, the recovery phrase, test CGT from the faucet, and the tests as a checklist (Library and QOR
    Media, Social, the wallet, Mesh seeding, and the Mesh's paid work once runtime 9 is live). **The upgrade:** a node
    built with `--features sudo` at the branch head; runtime 9 is `demiurge_runtime.compact.compressed.wasm`,
    **539,214 bytes, SHA-256 `318643587f6fe625346f4d4cd4a2ac31037d0a2e88225a06999915c84e037cfc`**, in the owner's
    Downloads as `demiurge-runtime-v9.compact.compressed.wasm`. Rehearsed with `chain/scripts/rehearse-upgrade.mjs`
    (now with Mesh checks) on a local chain started from the devnet's own live `:code` (516,949 bytes, v8) and no Mesh
    genesis: spec_version 9, Sudo kept with its key, the transfer, the asset, an ARQ Wallet payout and a round kept
    working, and the Mesh came up with ADR-096's 600-block period and 100-block window, bound a node, opened, took and
    closed an escrow (all 200 CGT back) and set a takedown as root. **Passed.** **Not done:** the owner signing the
    upgrade; installing 0.1.10 on a clean PC (installing needs an administrator prompt; nobody has run it yet); a log
    file testers can send (the launcher logs only to a console it does not have when installed); code signing (L6).

88. **17 October 2026: the owner upgraded Demiurge Devnet to `spec_version` 9 — the Mesh is live.** Walked through it one
    step at a time. The first signature, at #203267, was `sudo.sudoUncheckedWeight(system.remark(...))` with the right
    file as the remark: the Sudo page's second dropdown starts on `remark`. It succeeded and changed nothing (no
    `CodeUpdated`), found by decoding the block. Signed again with `system.setCode`. **Read back from
    `wss://rpc.qorsync.dev`:** `demiurge` spec_version 9, `transaction_version` 2; `:code` 539,214 bytes with SHA-256
    `318643587f6fe625346f4d4cd4a2ac31037d0a2e88225a06999915c84e037cfc`, the rehearsed file; `Sudo::Key` unchanged
    (`5HN6PZA4…Zadn`); `Mesh` with 11 calls, `PeriodLength` 600 and `AnswerWindowLength` 100 (ADR-096's, as rehearsed),
    `MeshApi_period` answering 338; finality advancing (#203,323 → #203,329 in 36 s); readiness 200. **Trap:** on the
    Sudo page, pick `setCode(code)` in the second dropdown; the field must read `code`, not `remark`. **Then the owner
    installed 0.1.10** from the release over 0.1.9 on their PC, and confirmed it working without error: the version 0.1.10,
    and a file from the Library playing inside the launcher in the QOR Media the installer carries. Not yet: an install
    on a PC that never had the launcher or the Visual C++ runtime.

89. **17 October 2026: launcher 0.1.11, built from `130d83c` on branch `session/launcher-0.1.10-release` (#41), not
    merged, not published.** Going through the steps as a newcomer, the owner found no way to put a Library file on the
    Mesh, and asked; told that the Mesh carries published work (ADR-095 decision 3) and offered three ways, they chose
    publishing a file from the Library, and asked for "a high quality and premium feeling gaming style look and feel
    that is easy for gamers and creators to understand", with animated buttons and a more enticing menu. **Publishing a
    Library file:** `library_publish` fingerprints the file a piece at a time (`ContentRef::of_file`), builds a manifest
    naming it alone (`library::commands::publication`: its name cut to the chain's limit on a character, never a path
    with a separator, dated when the file last changed, no commit), and mints it through `ChainClient::publish_file`,
    which shares `mint_core` with Projects' `mint`; the pallet's commit is optional, so none is sent. The dialog says
    first that the file becomes public and to publish only what is one's to share. Approved, the bytes are copied into
    the temporary store a piece at a time and refused if the file changed since it was fingerprinted
    (`TemporaryStore::put_file`), and the Mesh takes them in at once. **The file's menu** (`FileMenu`): right-click,
    Shift+F10, the Menu key, or Y on a controller; Play or Open, Show in folder, Publish to the Mesh…; it takes focus,
    the arrows move in it, Escape gives focus back to the file's tile (first written to give it to whatever had focus,
    which a right-click never moves: the view check caught it). The Mesh page's empty state now says where work comes
    from. **Found on the way:** an account's first mint holds 1,006.875 CGT in deposits, mostly for its collection, so
    the download page's 1,000 test CGT per tester could not publish anything; it says 2,500 now. **The look**
    (DESIGN_SYSTEM.md, under ADR-080, no new decision): Barlow Semi Condensed bundled as the display face
    (`@fontsource/barlow-semi-condensed` 5.3.0, OFL); 8px controls and 12px panels, lit from above with a top highlight
    and a shadow; buttons with a lit accent gradient, a glow, a sheen that crosses once on hover and a press; interactive
    panels that lift 2px and glow; corner brackets that follow the rounding; status pills; view titles in the display
    size with a line of light; and the rail grouped as Play, Own, Create, Connect and Network, each destination an icon
    tile, the current one lit, with one bar that glides to it. Every colour is a theme token, so all five themes
    restyle; seen in Architect and Numen. `check-readability.mjs` can now save every screen it measures
    (`READABILITY_SHOTS`), which is how the redesign was looked at. **Evidence:** the launcher's `cargo test --locked`,
    294 passed, 15 ignored; clippy and fmt clean; `npm run check`, all thirteen checks in every theme (readability as
    painted 371, the Library's view 40, the Mesh's 39); the publish path against a development node,
    `a_file_published_from_the_library_is_minted_and_seeded`, passed (declined writes nothing; the asset is the manifest,
    with no commit; the Mesh holds the file as one's own). Gates: library.tests requires two more (logged). **The
    installer:** `QOR Launcher_0.1.11_x64-setup.exe`, **69,235,782 bytes, SHA-256
    `0783b780502953c551439c0c36cfcf4f7088920d95883cd23fe7c3dd2967b2ce`**, QOR Media inside, copied to the owner's
    Desktop. **Not done:** #41 merged; 0.1.11 published as a release and the download page pointed at it; nobody has
    installed 0.1.11 yet.

90. **18 October 2026: launcher 0.1.11 published.** The owner merged #41 at `3e33c1d` (CI passed every job on its head
    `6ecb6f4`) and said: "commit. push. and publish all recommended. local and remote." `git diff 130d83c 3e33c1d`
    touches only `HANDOFF.md`, `OWNER.md` and `docs/SYSTEMS.md`, so the installer built from `130d83c` is `main`'s code.
    **The release:** `launcher-v0.1.11`, a pre-release on `QOR-MATRIX/demiurge-chain` at `3e33c1d`, asset
    `QOR-Launcher_0.1.11_x64-setup.exe`, 69,235,782 bytes, SHA-256
    `0783b780502953c551439c0c36cfcf4f7088920d95883cd23fe7c3dd2967b2ce`, downloaded back from its public URL (200) and
    matched; its notes give testers 2,500 test CGT each. **The download page**
    (`https://claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv`, private until the owner shares it) points at 0.1.11, with a
    group of tests for it: the file's menu, publishing to the Mesh, and the new look; the Mesh tests now start from
    publishing a Library file. `launcher-v0.1.10` stays published beside it. **Not done:** nobody has installed 0.1.11
    yet; two PCs over the internet; L10.5 to L10.7.

91. **18 October 2026: launcher 0.1.12 built, not merged, not published: each QOR ID on a PC keeps its own account,
    and the launcher says whose account is on screen (ADR-098, under the owner's delegation).** The owner found one
    wallet's balance shown under two QOR IDs on one PC. The cause: opening the vault signed in with the vault's first
    account (ADR-016), the interface showed the first account after every launch, and `qor://pay` paid from it, so a
    sign-in as a second QOR ID left the first account's balance under the second's name, and a send or tip from there
    spent the first account's CGT. **The fix:** `src-tauri/src/identity/owners.rs` notes which QOR ID each account was
    signed in as, and the account last in use, in `account-owners.json` beside the vault (public facts, sent nowhere; a
    QOR ID seen on a new account is forgotten on the old one). `arrive` signs in with the account asked for, else the
    one last in use, else the first; `adopt` makes the signed-in QOR ID's own account the one in use when the vault
    holds it; `qor_sign_in` takes an account; `qor_new_identity` derives a new account and registers the name for it in
    one host dialog; `vault_owners` gives the labels to the webview; `pay_link` pays from the account in use. In the
    interface: the Vault labels each account with its QOR ID and, on another QOR ID's account, says whose it is and that
    sending spends its CGT, with one click to the right one (`Whose`); the rail's account card says *Not <name>'s
    wallet*; Settings, signed out, lists each account to sign in as, claims a name only for the account that has none,
    and offers *A new QOR ID* on a new account. Also in the tree: `partners.rs` puts a partner traded with again in the
    same second first (the clock counts whole seconds, so the existing ordering test failed whenever the second turned
    between its trades), with a test. **Picked up uncommitted, its author not recorded** (an earlier session or IBM
    Bob), and verified rather than trusted: ADR-098 said a key sign-in outside the grant's five minutes still asks in a
    host dialog, which is false. `qor_sign_in` reopens the arrival grant, as it has for the first account since ADR-056,
    so Settings' sign-in asks nothing. The ADR now says so, and also that a new QOR ID's account declined at its dialog
    stays in the vault with no QOR ID. ADR-016 and ADR-090 say they are amended; ADR-098 is indexed in `DECISIONS.md` and
    `decisions/README.md`; the launcher's README has a section on it and pays from the account in use. **Evidence (this
    session):** `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean; `cargo test --features
    qontrol-no-skips`, **296 passed, 15 ignored** (294 before, plus the owners test and the partners test); the frontend
    built with `tsc --noEmit`; `npm run check`, **all fourteen checks pass** (design 3, accessibility 49, gates 34,
    Projects 104, Inventory 238, Market 127, QQ 42, Social 59, Library 40, Mesh 39, **accounts 21**, vault gate 66,
    contrast 54, readability 371), in the default theme. **Two planted faults**, the first account chosen at arrival
    (`setAccounts`) and a name claimed for the first account (Settings' `claim`), built and run: the accounts check
    failed 4 of 21, naming each; restored, 21 of 21. Gates: `alpha.accounts-view` on the suite `accounts_view`, logged as
    a tightening. **Not proven:** a sign-in as two real QOR IDs on one PC against QOR ID (the check uses a stand-in
    host). **What this is not:** a separation between people; every account comes from one phrase, and two people who
    must not reach each other's CGT need separate Windows user accounts. **The installer:** `QOR Launcher_0.1.12_x64-setup.exe`,
    **69,244,187 bytes, SHA-256 `ad970d155c6dda9772207adbf0ba9d37b11b0f1b9d22bb17068e347c17c00488`**, QOR Media inside,
    built from this uncommitted tree and copied to the owner's Desktop (the copy's hash matched). The first attempt stopped
    when QOR Media's `tst_playback` failed `text_opens_in_the_editor_and_unsaved_words_hold_the_window`
    (`qWaitForWindowActive` returned false: its window got no focus while the build ran in the background beside other
    windows); run alone it passed 10 of 10 three times, and the retried build passed all four suites. **The four live
    services answered from outside** (QOR ID's `/health` 200, Social's `/health` and `/ready` 200 at its Railway domain,
    ARQADE 200, the devnet at `spec_version` 9). **Then the owner asked for "an updated web downloader":** the work was
    committed as `5990c4a` on `session/launcher-0.1.12-accounts`, pushed to `matrix`, and opened as **pull request #43**
    against `main`. The download page (`https://claude.ai/artifact/Uw2NRswDw9quvSBxQFbBLv`, version 5, private until the
    owner shares it) now offers 0.1.12, with six tests for two QOR IDs on one PC first and 0.1.11's tests kept. **Creating
    the GitHub Release was refused by Claude Code's auto-mode classifier** ("Create Public Surface"), so the owner ran
    `gh release create` themselves; GitHub refused the short commit ID as `target_commitish` (HTTP 422), and the full one
    worked. **The release:** `launcher-v0.1.12`, a pre-release at `5990c4ae31c962647df90590982d8154249a96e4` (the
    unmerged branch commit, as no earlier release was; #43's later commits are documentation), asset
    `QOR-Launcher_0.1.12_x64-setup.exe`, downloaded back from its public URL (200), 69,244,187 bytes, SHA-256 matched.
    The download page's links now work. `launcher-v0.1.11` stays published beside it. **The owner merged #43** at
    `2b09bb0`; every check on its head passed (the two nightly-only jobs skipped, as on every pull request), and `main`'s
    own run (39143588848) was in progress when this was written. The merge took the branch one commit early, so this
    record reached `main` through a follow-up pull request. **Not done:** the owner's test with two real QOR IDs.

92. **18 October 2026: the homepage at qorsync.dev, live, written for search engines (ADR-100, the owner's), on branch
    `session/homepage`, merged as #45 at `316f83f`.** The owner asked for search to show this project rather than an older copy.
    **Found:** the one public copy is `AuthorPrime/Demiurge-Blockchain`, the owner's brother's (the owner: "Im not trying
    to remove his links, just to ensure that our current efforts are accurately being portrayed"), which is left alone and
    was not read (AGENTS.md §1); the owner's other copies are private. No page of the project's own existed: `qorsync.dev`,
    `demiurge.cloud` and `demiurge.guru` answered 404 (the first two attached to the Vercel project `demiurge`, linked to the
    private archive, every deployment failed since late September), and this repository had no description or topics.
    The owner chose `qorsync.dev`, "Build and publish", and "a high quality, ancient futuristic style with animations and
    effects mouse-over hover and reactive background. Premium only look and feel". **Built:** `site/` (see its README):
    one static page, its words from current documents only, with the title, description, canonical address, share cards
    (`og.png`, 1200 × 630), schema.org structured data naming this repository, `robots.txt`, `sitemap.xml`, `llms.txt`, an
    IndexNow key, a 404, a manifest and `security.txt`; obsidian, gilt and ember, Cinzel over Barlow, engraved cards that
    tilt and light, and a canvas lattice, astrolabe (circle, triangle and square, deliberately no hexagram) and embers that
    react to the pointer; reduced motion draws one still frame. ARQADE gained share cards, a canonical address, structured
    data naming Demiurge as publisher, `robots.txt` and a sitemap, and its dead `demiurge.cloud` link points at the
    homepage. The README was brought up to date (0.1.12, Social, the Mesh, fourteen checks, `site/`). **Published:** the
    Vercel project `demiurge-site` in `astramatrix` (the Vercel connector may not create projects, so the owner's signed-in
    CLI did it), deployed from `site/`; `qorsync.dev`, `www.qorsync.dev`, `demiurge.cloud` and `www.demiurge.cloud` moved
    to it from `demiurge` with Vercel's move call (the CLI's delete needs a confirmation flag, not used), `demiurge.guru`
    and `www.demiurge.guru` added; all but `qorsync.dev` answer 308 to it. Then connected to `QOR-MATRIX/demiurge-chain`,
    root `site`, production branch `main`, a build skipped when `site/` did not change. The repository's description,
    website (`https://qorsync.dev`) and ten topics set. IndexNow answered 202. **Evidence:** `check-site.mjs` 26 of 26, two
    planted faults caught; the page in Edge at 1440 px and at 390 px under phone emulation, no horizontal scroll, hover
    exercised; ARQADE typecheck clean, lint 0 errors (the same 4 warnings as `main`), tests 49 of 49, build with
    `/robots.txt` and `/sitemap.xml`; every address above checked from outside, and QOR ID and the RPC still 200. **Not
    done:** Google Search Console and Bing Webmaster Tools (the owner's accounts); GitHub's social preview
    image, which only the repository's settings page can set; re-crawling takes days to weeks. Launcher 0.1.13 (ADR-099) is item 93.

93. **18 October 2026: launcher 0.1.13 built and committed (#46, `366bbf6`), not published: each QOR ID on a PC is a profile of its own
    (ADR-099, the owner's), superseding ADR-098 the day it shipped.** After 0.1.12 the owner asked why a new wallet made
    while signed in as Yaldobaoth showed about 59,000 CGT. **The cause, read from the PC and the devnet:** the vault never
    stored the accounts it derived, so each launch it held only the first (`5Gx19…5sPwt`, 37,000 test CGT, Yaldobaoth's)
    and *Add account* and ADR-098's *A new QOR ID* derived the second again every time: `5DSND…LUV34`, the account used
    since early October, holding 57,900 test CGT with nonce 2. The owner then decided: "QOR IDs need to be separate
    entirely. A new app sign in should be entirely dependent on the data for that account. If an account logs out, that
    should separate the amounts completely", and, asked what keeps a signed-out QOR ID from someone else at the PC, chose
    a PIN per QOR ID. **Built:** `src-tauri/src/profiles.rs` (`profiles.json`; a folder per profile under the data and
    local data directories; a PIN of 6 to 12 digits sealing the wallet key with the vault's own seal; five free tries, then
    waits doubling from 30 seconds; the move of the old wallet into the first profile). The vault is placed in the
    signed-in profile's folder, holds one account at the index its file names (`index`, default 0), and can give its key
    for sealing, take it out of the keychain and put it back (`KeyStore::delete` added). QOR ID's tokens are kept under
    the profile's id, and the unscoped ones are deleted at the move. `AppState` holds the signed-in profile's things
    (`Personal`: folder, Mesh, Library and its watcher), and every personal path (avatar, trade partners, listing drafts,
    paid requests, published work's bytes, QQ Studio's store, the Library and the Mesh) goes through it; device settings,
    gates runs and translation models stay shared. Commands: `profiles_list`, `profile_open`, `profile_sign_out`,
    `profile_set_pin`, `profile_other_accounts` (the phrase's accounts 0 to 7 used on the chain), `profile_split`;
    `vault_create` and `vault_restore` make or reopen a profile. Removed: `vault_add_account`, `vault_owners`,
    `qor_new_identity`, `qor_logout` and `identity/owners.rs`. The interface: *Who's playing?* and a PIN step in the Gate,
    *This QOR ID on this PC* in Settings (sign out, choosing a PIN the first time; change PIN; other accounts from this
    wallet's phrase), one account in the Vault; the five files ADR-098 changed went back to their versions before 0.1.12
    first. **Found by the new host test:** a profile's account was noted only after QOR ID sign-in, so a split could have
    made an account that already was a profile into a second one; the account is now noted as the wallet opens.
    **Evidence (this session):** `cargo fmt --check` and `cargo clippy --all-targets -D warnings` clean; `cargo test
    --features qontrol-no-skips`, **302 passed, 15 ignored**, among them `a_profile_is_left_completely_and_opens_again_only_with_its_pin`
    (the move, a silent first open, sign-out refused without a PIN, nothing open and no key in the keychain after it, a
    wrong PIN opening nothing, a split, each profile opening as its own account with its own PIN), four in `profiles` and
    two new in `vault`; `npm run check`, all fourteen checks (design 3, accessibility 49, gates 34, Projects 104, Inventory
    238, Market 127, QQ 42, Social 59, Library 40, Mesh 39, **profiles 25**, vault gate 65, contrast 54, readability 371).
    `check-profiles-view.mjs` caught two planted faults (a PIN shorter than six digits accepted; a split sending no PIN).
    The vault gate's restore check, which requires the phrase alone, caught the interface sending `index: 0` beside it;
    the index is now sent only when it is not the first. Gates: `alpha.accounts-view` replaced by `alpha.profiles-view`,
    logged as a scope change for the owner to rule on, as on 28 September. Also: #44 merged at `9f5c8bd`, and CI passed on
    `main` at `2b09bb0` and `9f5c8bd`. **The installer:** `QOR Launcher_0.1.13_x64-setup.exe`, 69,277,353 bytes, SHA-256
    `834ba85ac38b1ef1af9cb0a55e4950947659dd523c95c6eb51e85a29e1df7756`, all four QOR Media suites passing, copied to the
    owner's Desktop (the copy's hash matched). Item 92, the homepage, was written on its own branch (#45) meanwhile.
    **Committed** as `366bbf6` on `session/launcher-0.1.13-profiles` (#46) at the owner's word; its
    homepage preview check failed until #45 brought `site/` to `main`, which was then merged in. **Not done:** #46
    merged, the installer published; the move run on
    the owner's real data, which happens at their first launch of 0.1.13 (the old wallet's folder is moved, not copied: a
    0.1.12 or older launcher opened afterwards finds no wallet and offers to make one).

94. **18 October 2026: ARQADE rebuilt in the launcher on Qt is decided (ADR-102, ADR-103) and its first stage, Sparks
    and embers (ADR-101), is built on branch `session/sparks`, merged as #47 at `8746501` (#46 at `2214c17` before it).** The owner asked to rebuild ARQADE into the
    launcher "as a new and improved version", Energy replaced by Sparks, the games "rebuilt as high quality versions"
    with "an earning mechanic which rewards players with Sparks", Sparks reworked to "1000 Sparks = 1 CGT", and Qt "for
    all aspects of the QOR experience with the most graphically cutting edge options possible". Asked, the owner chose:
    a Spark of 0.001 CGT over an unchanged chain, the atomic unit named the **ember**; **a platform reward pool** to pay
    players; ARQADE as the name, in `products/arqade/qt/`; Qt for the Arcade now and the launcher's screens in stages;
    and 0.1.13 committed first (#46). **Recorded:** ADR-101 (units), ADR-102 (ARQADE on Qt, Energy removed, the pool as
    ARQADE's own ARQ Wallet, rewards only for runs the ARQADE service replays, devnet only; U-16 updated), ADR-103 (the
    launcher's screens to Qt over the Rust host, three stages); DIRECTION L11, L12 and P7.19 to P7.24; AGENTS.md §5 in
    embers; CGT.md, OPEN_QUESTIONS (U-2, U-16), DECISIONS and every current document brought into line (the two
    architecture records of the old devnet keep their words, which were true of it). The pool's policy is the chain's
    own (`pallet-arq-wallet`: largest payout, epoch budget, per-player cap, rule versions), so one pool serves every game
    with no chain change. **Built:** the rename of the atomic unit from Spark to ember in the chain's documentation and
    the launcher, ARQADE and its SDK (375 lines by script, reviewed: QQ's particle sparks, a palette colour, ARQADE
    chat's spark reaction and the display `sub_unit` excluded; a historical constant name and comment capitals fixed by
    hand); `SPARK` (10^15 embers) in `chain/runtime/src/denomination.rs` with a test; in the launcher, `parse_amount`
    (CGT or Sparks, "250 Sparks", finer than an ember refused) behind the send box, a sale's price and its preview, and a
    listing draft (the Mesh's escrow prices are still parsed in the interface, CGT only), `format_sparks`, a Balance's
    `in_sparks` shown under the Vault's figure, and Settings' units line ("1 CGT = 1,000 Sparks = 10^18 embers"); QOR
    ID's `WELCOME_EMBERS`, its grant listing sending `amount_embers` with `amount_sparks` beside it for one release, the
    column `welcome_grants.amount_sparks` keeping its name (documented as embers) so no migration runs on the live
    database; the `qor://pay` link's field was always `amount`, so installed launchers are unaffected. **Evidence (this
    session):** chain `cargo fmt --check` and `cargo clippy --workspace --all-targets -D warnings` clean, `cargo test
    --workspace --locked` **178 passed** in 20 suites, runtime built (177 before, plus the Spark test); launcher fmt and
    clippy clean (two `is_multiple_of` findings fixed), `cargo test --features qontrol-no-skips` **304 passed, 15
    ignored**; `npm run check` all fourteen checks, profiles **26** (a new check that the Vault shows Sparks, which caught
    a planted fault); ARQADE typecheck clean, lint 0 errors, tests 49 of 49; QOR ID fmt and clippy clean, its database
    tests left to CI (Docker Desktop was not running). **Next (ADR-102's order):** P7.19, ARQADE's Qt shell in
    `products/arqade/qt/` docked under the product host; P7.20, the four solo games in QQ; then the pool and replay.

95. **18 October 2026: ARQADE's Qt app, P7.19 (ADR-102), built on branch `session/arqade-qt`, committed as `6826ce4` and pushed at the owner's word ("Commit and push all recommended"), in pull request #48, merged by the owner at `6cb96fb`; published for testers in launcher 0.1.14, and the homepage's download links point at it (checked from outside).** The owner
    merged #46 and #47 ("both are merged."), and stage 2 of ADR-102 followed.
    - **The app**, `products/arqade/qt/` (Qt 6.12, C++20, Qt Quick and Qt Quick 3D; `README.md` there):
      - its home: a 3D astrolabe portal (physically based gilt rings around an ember core with a halo; the homepage's
        triangle and square; embers rising and leaning toward the pointer; filmic tonemapping, bloom, a vignette,
        4x MSAA);
      - the four games and two arenas as cards, each with a small 3D emblem; until P7.20 a card opens today's version
        in the launcher's ARQADE window (`arqade.web`);
      - the player's QOR ID, level and Sparks, with what rewards are on the devnet said beside them (ADR-102
        decision 7);
      - the arenas' rankings from `/api/live`.

      Cinzel and Barlow Semi Condensed, the homepage's faces, are bundled under the SIL Open Font License. It draws on
      Direct3D 12 when a probe finds it. Where a PC has two graphics cards it uses the one Windows calls
      high-performance. On this PC, Qt had taken the Intel UHD first; with the change it took the RTX 4060, as `QSG_INFO`
      showed.
    - **Its link to the launcher** (ADR-046's product host, as QOR Media's):
      - `arqade.next`, with a 3-second wait so a sign-in is not held long behind it;
      - `arqade.signin` and `arqade.web`;
      - the commands place (which shows it), hide, theme, player, prefs and quit.
    - **The sign-in:**
      1. The app asks ARQADE's server (`/api/auth/login`) to start a sign-in and is sent to QOR ID's page.
      2. The launcher checks that page is QOR ID's sign-in on the QOR ID it uses, asks QOR ID with its own token
         (`/api/v1/oauth/handoff`), and checks the answer lands on ARQADE over https.
      3. The app follows only an answer on its own server, which sets `arq_session` in a cookie jar held in memory.

      A session that ends is renewed once, never in a loop.
    - **The launcher's side:**
      - `src-tauri/src/arqade/app.rs`: the handler, the app's state (`App::open` starts it once however often the
        screen is visited, and clears what an ended run left queued) and six commands;
      - the command queue made generic in `media.rs`, so QOR Media and ARQADE share it;
      - `host::find_product`;
      - signing out of a profile quits ARQADE.

      The ARQADE screen (`ArqadeView.tsx`, with `lib/dock.ts` now shared with the Library) starts and docks the app. It
      falls back to the website's window when the app is absent, fails to start or ends. The app also ships with the
      installer:
      - `scripts/stage-arqade.mjs` and the bundle mapping carry it;
      - the installer hook runs whichever Visual C++ runtime installer it finds;
      - `.gitattributes` marks `*.hdr` binary.
    - **Found:**
      - Six Quick 3D views that each made their own sky with `ProceduralSkyTextureData` crashed Qt 6.12 (heap corruption
        or an access violation, three runs in three). The first interface test run lost its graphics device.
      - One HDR image shared by every view (`assets/sky.hdr`, written by `tools/make_light.py`) was stable in five runs
        in five.
      - Qt's torus lies flat, an unlit material shows no emission, and strong bloom turns ember to yellow (§5).
    - **Evidence (this session):**
      - ARQADE's Qt Test suites pass, 32 tests: the launcher link 15, the server 9, the interface 8. The interface tests
        draw on the graphics card and save frames, which were looked at.
      - The QML lint is clean.
      - The launcher is fmt and clippy clean. `cargo test --features qontrol-no-skips`: **317 passed, 16 ignored**, the
        twelve new tests among them. Three planted faults in the sign-in were each caught: the page unchecked, the
        answer passed on wherever it lands, the page logged.
      - The opt-in test that starts the real app, through `App::open` as the screen does, passes on the RTX 4060 (and
        passed earlier on the Intel card). It was welcomed, took every command, asked to be signed in with exactly the
        page its server sent, started once for two visits, and closed with code 0 when told. A review found that a quit
        queued after ARQADE had already gone would close the next ARQADE at once; with the fix removed, the real app did
        exactly that and the test failed.
      - `npm run check` passes all **fifteen** checks: design 3, accessibility 49, gates view 34, Projects 104,
        Inventory 238, Market 127, QQ 42, Social 59, Library 40, Mesh 39, profiles 26, **ARQADE view 17**, vault gate 65,
        contrast 54 and readability 371. A planted fault (less motion not passed on) failed the new check, 16 of 17.
      - `npm run app:build` from this uncommitted tree built an installer, to prove the bundle and the hook. Both staging
        scripts ran their suites. ARQADE was staged as 1,425 files, 154.6 MiB, and `arqade\arqade.exe` was placed
        beside the launcher. makensis took the new hook. The installer is 78,673,130 bytes, against 0.1.12's 69,244,187.
        It is still labelled 0.1.13; it was not kept for testers and not published.
      - From outside, ARQADE's real sign-in start answers 302 to `id.qorsync.dev/oauth/authorize`, and QOR ID's handoff
        route answers 401 without a token, where an unknown route answers 404.
    - **Not done:**
      - A person signing in through the app in a running launcher.
      - The games themselves (P7.20), the pool (P7.21) and replay (P7.22).

96. **18 October 2026: ARQADE's site on phones and by touch (P7.25), on branch `session/arqade-mobile`, in pull request #49, merged by the owner at `943815d`
    and live on Vercel (its production deployment at that commit serves the tab bar, checked from outside).**
    The owner asked to "optimize Arqade for its mobile browser users and screen tapping" (and for an Android launcher,
    item 97 when it starts).
    - **Found, on the live site at phone size:** a 64-pixel icon rail always on the left, 17 targets under 44 pixels (the
      intro's Skip was 14 high, the Energy chip 15), a large chat pill over the content, and games in a dialog sized for a
      desktop.
    - **Built:**
      - one readable block at the end of `app/globals.css`, "Phones and touch": below 760 pixels the rail is a tab bar
        along the bottom (eight tabs, an icon over a short label from `data-short` in `page.tsx`), the content takes the
        whole width, dialogs and games fill the screen, every target is 44 by 44 or more, and the chat's button is round
        above the tab bar;
      - `viewport-fit=cover` and a theme colour in `app/layout.tsx`, with content kept in the safe area;
      - Void Runner changes lane on a tap of the playfield's half, Orbital locks on a tap of its playfield, and both
        games' instructions say so;
      - `agentRules: false` in `next.config.ts`, because `next dev` was writing its own `AGENTS.md` and `CLAUDE.md` into
        `products/arqade/`, a second set of assistant rules beside the code.
    - **Evidence:**
      - The new `scripts/check-phone.mjs` passes 17 of 17 against the production build, as a phone (390 by 844, touch)
        and once as a desktop. With the phone rules switched off as a planted fault, 8 failed.
      - It is a CI step in the arqade job and a criterion, `arqade.phone` (a tightening, logged in GATES).
      - Typecheck is clean; lint has 0 errors and the same 4 warnings; `npm test` passes 49 of 49; `next build`
        succeeds; the launcher's gates tests pass 15 of 15 against the new GATES.toml.
    - **Not done:** tried on the owner's phone; arena boards seen during a live
      match on a phone (they need two signed-in players).

97. **18 October 2026: the QOR Launcher on Android (ADR-104, L13), its first step built on branch `session/android`,
    in pull request #50, merged by the owner at `15cfa12`.**
    - **The owner's request:** "I'd like to build a android varient of the QOR Launcher as well".
    - **The owner's choices, asked:**
      - Qt for Android now, rather than Tauri on Android;
      - a first build with the wallet, QOR ID and Sparks plus Social, ARQADE, the Library and the Mesh;
      - a signed APK on GitHub;
      - the toolchain installed on this PC;
      - the app ID `app.qorsync.dev`: typed `app.qorsync.dev`, then confirmed in Android's usual lowercase form when
        asked, before any tester installed it. It is permanent from the first install.
    - **Decided:**
      - ADR-104 is the record ADR-103's stage 2 asked for. Its bridge is a small C interface of the core's own, not
        CXX-Qt, whose maintainers wrote that Android "is currently not a platform we actively support" (KDAB/cxx-qt#1251).
      - ADR-103's status and index line say so.
      - DIRECTION gains L13, eight steps.
    - **Built:**
      - `tools/qor-launcher/shell/android-toolchain.ps1` installs JDK 21, the SDK (platform and build tools 36), NDK
        r27c (the one Qt 6.12 is built with) and Rust's Android targets, each download checked against its SHA-256.
        2.9 GB, plus the emulator and an Android 16 x86_64 image, installed afterwards.
      - The core, `tools/qor-launcher/core` (crate `qor-launcher-core`). `cgt.rs` and `error.rs` were moved there by
        `git mv`, the error's reqwest conversion behind a `net` feature. The Windows launcher re-exports both at their
        old paths.
      - The core's bridge (`src/bridge.rs`, `include/qor_core.h`): JSON in, JSON out, in the shape of Tauri's `invoke`.
        It is the only module allowed `unsafe`. Its commands are `core_version`, `token_info`, `parse_amount` and
        `format_sparks`.
      - The shell, `tools/qor-launcher/shell` (Qt 6.12, Qt Quick): `Core`, the bridge as QML sees it, and a first screen
        that shows the core answering and reads an amount typed in Sparks or CGT.
        - Corrosion v0.6.1 builds the core for the build's target.
        - `build.ps1` makes the desktop build and its tests, or `-Android [-Abi x86_64]`, which signs the APK with
          Android's debug key.
        - The app leaves room for the keyboard itself: Android 16 draws apps behind it, and Qt reports it in physical
          pixels (section 5).
      - CI runs the core's format, lints and tests in the launcher job. GATES has a suite `launcher_core`, logged as a
        scope change, because 13 tests moved from suite `launcher`.
    - **Evidence:**
      - The core passes 20 of 20 tests, with clippy and fmt clean.
      - The launcher passes 291 tests with 15 ignored (304 before, the 13 moved). Its fmt, clippy with `--locked` and
        gates tests (15) are clean.
      - The shell's `tst_core` passes 4 of 4 on the desktop.
      - The arm64 APK is 55.8 MiB: `app.qorsync.dev`, 0.1.0, min SDK 28, target 36. The x86_64 one ran on an Android 16
        emulator on this PC's RTX 4060: it starts in 0.5 to 0.9 seconds, the core answers, and "1.25 CGT" reads as
        1,250 Sparks and 1,250,000,000,000,000,000 embers above the keyboard.
      - The debug-signed arm64 APK is on the owner's Desktop, `QOR-Launcher-Android-0.1.0-test.apk` (58,490,583 bytes,
        SHA-256 `97d80812...c7c1`; the renamed `app.qorsync.dev` build, run on the emulator again before it was copied).
    - **Not done (L13.2 on):**
      - The rest of the core moved: the vault, profiles, QOR ID, the chain, Social, the Library and the Mesh.
      - Keystore and the wallet's screens; QOR ID and Social; ARQADE; the Library; the Mesh.
      - A release signed with the project's test key. A build signed so does not install over this debug-signed one,
        which must be uninstalled first.
      - A manifest of the app's own: Qt's default asks for `WRITE_EXTERNAL_STORAGE`, which this app does not need.

98. **19 October 2026: CI's backdrop check made to test what it claims (the owner's decision, a loosening logged in
    GATES), on branch `session/backdrop-check`, in pull request #52, merged by the owner at `485eda5`.**
    - **Found:** after #50 merged, CI failed on `main` at `15cfa12`. `backdrop liquid: it compiles and runs` counted 16
      frames in a second against a bar of 20.
      - The same tree had passed on #50 at `802c4db` with 22, and re-running `main`'s failed job passed with 23.
      - Thirteen runs read back gave liquid 16 to 47 frames a second on CI's software renderer, 20 to 22 in six.
      - Below about 41 a second the launcher's own budget (`src/qfx/Canvas.tsx`: 24 ms a frame, 30 strikes) stops the
        loop after thirty slow frames, by design. The count measured the runner, not the backdrop.
      - `main` had failed once before on a browser check, at `8746501` (the merge of #47): headless Chrome did not
        expose a page.
    - **Decided:** asked, the owner chose to make the check test what it claims, a loosening logged in GATES' change log.
    - **Built:** `scripts/check-accessibility.mjs` records every shader compile and program link on a WebGL2 context as
      WebGL reports it. Each backdrop is checked twice: its shaders compile and its program links (2 of 2, 1 of 1), and
      it draws at least one frame in the second. The twenty-a-second bar stays on the positive control and on the
      backdrop starting again from off. 55 cases where there were 49.
    - **Evidence:**
      - 55 of 55, three runs in three.
      - Planted in liquid's shader, a compile error ("1 of 2 compiled, 0 of 0 linked") and a link error ("2 of 2
        compiled, 0 of 1 linked") each failed both new cases, the second with 0 frames.
      - Liquid slowed on purpose drew 11 frames in the second, CI's condition, and passed.
      - Restored and rebuilt, 55 of 55.

99. **19 October 2026: toward a worldwide beta. The code surveyed, and three branches built and tested, each based on
    `main` at `ae99d2c`, then committed and pushed at the owner's word ("commit and push") as pull requests #54, #55
    and #56, for the owner to merge.**
    - **The owner's request:** "Proceed"; then "Merged #52" (#53, its records, was merged too); then: "Analyze the
      entire codebase and proceed with the next steps forward as best recommended in sequence as to complete all missing
      aspects of the most current ecosystem. Ensure that the game engine is high quality and can produce real games in
      3D and 2D … premium look and feel … appeal to gamers and developers … Compliance is a must while working with
      CGT. Lets prepare the ecosystem for a worldwide global beta release." The owner also asked that CGT be developed
      to the standard of a currency. That is how it is handled here, and no document states anything about its value
      (AGENTS.md §6).
    - **Surveyed, by three read-only agents:**
      - **The Beta gate.** Alpha is two units short: M3.2 (OPEN-2) and L1.4's last two dialogs. Beta is far: `beta.chain`
        2 of 22 items, `beta.launcher` 3 of 12, three units unmeasurable. Most of it waits on the owner (§4.0, "The Beta
        gate").
      - **QQ.** 3D only, with no 2D at all. Logic could not hear a collision, cast a ray, make or remove an entity or
        wait. There was no game interface, no input actions, no fixed step, and no animation, mix, textures or editor
        basics. ADR-105 answers this.
      - **CGT.** The chain and the core are sound. The findings, what was fixed and what is open are in
        `docs/audit/RECONCILIATION.md` §8.
    - **#56, `session/core-vault` (this one): L13.2, the wallet moved into the core.**
      - The vault (`vault/`, by `git mv`), the profiles and the confirmation they ask through (`confirm.rs`) moved into
        `tools/qor-launcher/core`, with their 52 tests. The Windows launcher re-exports each at its old path.
      - The test stand-ins sit behind the core's `testing` feature, which only the launcher's dev-dependency turns on.
      - On Android there is no OS keychain in the core (Keystore comes at L13.3).
      - `MeshGrant::recorded` and `Vault::sign_granted` became public across the crate boundary. Two guards cover them:
        the launcher's existing check now also holds restoring a grant to the Mesh, and a new core test shows nothing in
        the core calls either. Each was proven by a planted call.
      - The core's `Cargo.lock` follows the launcher's. CI audits it, a tightening: no vulnerabilities, three
        "unmaintained" warnings.
      - **Evidence:**
        - The core: 71 pass, 2 ignored; clippy and fmt clean, with and without `testing`.
        - The launcher: 255 pass, 14 ignored; clippy and fmt clean.
        - Counted as written: 321 tests to 270 in suite `launcher`, 20 to 73 in `launcher_core` (GATES change log).
        - The arm64 APK builds, with sp-core, schnorrkel, blake3 and secp256k1's C library compiled for Android, and
          `keyring` not.
      - The same branch carries this session's records:
        - this item and §4.0's new rows;
        - HANDOFF's header and the Mesh row;
        - DIRECTION's launcher row (it said 0.1.9);
        - the Spark line in `OPEN_QUESTIONS.md`;
        - `OWNER.md` and `SYSTEMS.md`.
    - **#54, `session/cgt-compliance`: what the CGT audit found that engineering could fix.**
      - **QOR ID:**
        - Migration 024 retires migration 005's donation scheme. Its trigger, which set "staking" bonuses and fee
          discounts by fiat tier, is dropped with its functions and perk columns. The tables move whole to a schema
          `retired`, so nothing that might record a payment is lost.
        - Migration 025, and the service, refuse a negative agent spending limit.
        - 180 tests pass against Postgres 16 and Redis 7.4 with `--include-ignored`, 4 of them new. fmt, clippy and
          the SQL check are clean, after the first draft's tests were caught building SQL with `format!`.
        - A planted fault, 024 keeping the "staking" column, was caught.
      - **The core's parser:**
        - A separator is read only between groups of three digits, and a sign is refused. "1,5" was 15 CGT, and
          "1.+5" was 1.05.
        - `format_sparks` takes digits only.
        - 21 tests.
      - **ARQADE, P7.23's removal:**
        - No Energy store, entry cost, token draw or token wallet. The intro no longer says "you may lose more than
          you could win".
        - The tip panel formats with `BigInt`.
        - CI's sequence passes locally: types, tests, build, lint (the four old warnings) and the phone check, 17 of
          17 with six tabs (a scope change in GATES).
      - **Copy:** the launcher and the homepage no longer promise a starter grant or a faucet.
      - **CI:** a step runs the homepage's own check, which no job ran, and fails on investment language anywhere
        people read in the active scope (a tightening). It was proven both ways, clean on the tree and failing on a
        planted "passive income".
      - On that branch the launcher passes 304 host tests (16 ignored) and all fifteen browser checks.
    - **#55, `session/qq-gameplay`: ADR-105 and P3.7.**
      - **ADR-105** (accepted under the owner's direction): real games in 3D and 2D from one runtime, 2D being the same
        world through an orthographic camera. DIRECTION gains P3.7 to P3.16 in order.
      - **P3.7, built:**
        - Logic gains `onStarted`, `onContact`, `onEntered`, `onExited`, `ray`, `spawn`, `remove`, `after`, `every`
          and `cancel`.
        - Shapes can be kinematic, and dynamic ones can be pushed and turned.
        - `Trigger` is a new kind.
        - `SceneIO.remove` takes a body with its entity.
      - **The proof is Orb Run**, `products/qq/samples/orb-run/`.
      - **Line endings.** `.gitattributes` pins QQ's scene fixtures and samples to LF. With this PC's
        `core.autocrlf=true`, a fresh checkout gave them CRLF, and the scene round trip failed on `main` as well.
      - **Evidence:**
        - `tst_play` passes 18 cases, 7 new.
        - Five planted faults were each caught.
        - All eight suites pass in one run of `build.ps1` on the owner's PC: 83 cases, with the agent suite's
          real-endpoint case skipped, as it is unless asked.
    - **Next, engineering, in order:**
      1. QQ's interface (P3.8), input actions (P3.9), 2D (P3.10), and fixed steps and replay (P3.11), which ARQADE's
         rewarded runs need.
      2. The launcher's update channel (L6.2), so testers worldwide get fixes.
      3. Agent authorisations signed in the vault (L5.1), which closes the gap in SECURITY.md.
      4. The Mesh grant's per-period limit, which ADR-097 wants before fees.
      5. Q-19 and Q-20 drafted as proposed ADRs for the owner.
      6. The open items of RECONCILIATION §8:
         - a welcome grant marked paid only against the chain;
         - the ARQ Wallet refusing rather than saturating, which needs a runtime upgrade;
         - `qor://pay` checking the chain before paying;
         - the Mesh form on the core's parser.
      7. L13.2's next modules: QOR ID's client, then the chain client.
    - **Left on this PC:**
      - The worktrees `X:\REPOZ\cgt-compliance` and `X:\REPOZ\qq-gameplay`.
      - QQ's build folder `%LOCALAPPDATA%\qq-build-gameplay`.
      - Docker Desktop was started for QOR ID's tests and stopped again, its two throwaway containers (`qor-cgt-pg`,
        `qor-cgt-redis`) removed first. The older stopped containers were left as they were.
      - `archive-bZLIQp/` untouched.

100. **19 October 2026, night: #54, #55 and #56 merged by the owner, and live.**
     - **Merged:** #54 at `d93c3c5`, #55 at `ae11498`, #56 at `c4972b0`. They merged in turn with no conflict, as
       checked before they were pushed. CI passed every job on `c4972b0`
       (run 39320068874); the runs on the first two merges were cancelled as each next merge superseded them.
     - **QOR ID:**
       - Railway redeployed it from #54's merge (deployment `63fed846`). It reports SUCCESS, its new container started
         at 20:33:45 UTC, and `/ready` and `/health` answer 200 from outside.
       - QOR ID runs its migrations before it serves, so 024 and 025 are applied. Its own startup lines were not yet
         readable through Railway's API when this was written.
       - #55's and #56's merges were skipped for QOR ID, as its watch paths intend.
     - **ARQADE and the homepage** deployed from `main` on Vercel. ARQADE serves "Free to play" and "Sessions played"
       and no Energy. The homepage serves the corrected faucet line.
     - **Recorded on `session/after-56`:**
       - P3.7 ticked in DIRECTION, and #54's and #56's notes there say merged.
       - The `qor-engine` gate gains P3.7 to P3.16 and ADR-105, and its `means` says the engine makes real games in
         3D and 2D. A tightening, logged in GATES; the launcher's gates tests read the edited file.
       - This item, the header, OWNER.md and SYSTEMS.md.
     - **Removed:** the worktrees `X:\REPOZ\cgt-compliance` and `X:\REPOZ\qq-gameplay`, with their build outputs.
       The branches stay, on this PC and on GitHub.

101. **19 October 2026, night: P3.8, the game's interface, built on branch `session/qq-interface`, pushed at the
     owner's word as pull request #57, and merged by the owner at `f31443f`.**
     - **What:** ADR-105's interface, as five screen kinds on one base (`ScreenPart`): `ScreenText`, `ScreenBar`,
       `ScreenButton`, `ScreenImage` and `ScreenPanel`.
       - Each belongs to a named screen, and each is drawn on the World's overlay, placed by an anchor and insets in
         points of a screen 720 tall.
       - The Scene's new field `screens` names those shown at the start.
       - Logic gains `showScreen`, `hideScreen`, `onlyScreen`, `pause`, `resume` and `onClicked`. A paused world holds
         physics, logic and its timers, the player and particles still.
       - `Look` is the style: Cinzel and Barlow Semi Condensed in the runtime (OFL), the launcher's inks and ARQADE's
         gilt, and Windows' own animation setting for reduced motion.
       - The Studio's Add menu has an Interface submenu, and the agent's instructions name the kinds.
     - **Orb Run** opens on its title with the world paused; Play starts it, the score is on a HUD, and Pause shows a
       pause screen over the held world.
     - **Evidence, on the owner's PC:**
       - `tst_play` passes 23 cases, 5 new.
       - All eight suites pass in one run of `build.ps1`: 88 cases.
       - Five planted faults were each caught.
       - Frames were looked at: the title, the HUD and the pause screen.
     - **Two things the first run found**, fixed before it passed:
       - a part made before its scene was placed in its world never found its layer;
       - parts of hidden screens faded out of view instead of starting hidden.
     - **Next:** P3.9, input actions.

102. **19 October 2026, late night: #57 and #58 merged by the owner; P3.9, input actions, built on `session/qq-input`
     and pushed at the owner's word as pull request #59.**
     - **Merged:** #57 (P3.8) at `f31443f` and #58 (the records of #54 to #56) at `ee374ba`. P3.8 is ticked in
       DIRECTION. CI passed every job on `ee374ba` (run 39329399993); the run on #57's merge was cancelled as
       #58's superseded it.
     - **P3.9, built:**
       - **Declaring actions.** An `InputAction` entity names an action with its keys and mouse buttons ("E, Mouse
         Left") and gamepad buttons ("X, RT"). XInput now reads every button, the triggers among them. A
         `ScreenButton` with an `action` holds it under a finger: the touch control.
       - **Built-ins.** Jump and run are built-in actions with their old bindings, so they can be rebound. Move and
         look stay fixed.
       - **Logic** gains `onPressed`, `onReleased`, `held`, `rebind`, `bindingOf` and `pointing()`.
       - **The agent's** `input_set` holds actions by name, and stop lets them go.
     - **A behaviour change in Input:** keys and mouse buttons are read once, as they reach a window. Qt sends a key on
       to the item with focus, so reading it at both would count it twice. A planted fault proves it.
     - **Orb Run** pauses on P, on the pad's Start, or on its Pause button.
     - **Evidence, on the owner's PC:**
       - `tst_play` passes 26 cases, 3 new.
       - All eight suites pass in one run of `build.ps1`: 91 cases.
       - Five planted faults were each caught.
     - **Next:** P3.10, 2D.

103. **19 October 2026, late night: #59 merged by the owner; P3.10, 2D, built on `session/qq-2d`, with ADR-106,
     committed as `3b84ee4` and pushed at the owner's word as pull request #60.**
     - **Merged:** #59 (P3.9) at `b483e7a`. P3.9 is ticked in DIRECTION.
     - **P3.10, built** (DIRECTION P3.10 has the whole list):
       - `view: "2d"`, the same world through an orthographic camera, with a backdrop of the sky's colours.
       - `Sprite`, `TileMap` (Tiled's JSON, one mesh per layer, colliders merged) and `Player2D`. Bodies stay on the
         plane.
       - The Studio edits flat, and the preview's scenes import as 2D.
     - **ADR-106** (accepted under the owner's delegation, as ADR-105 was) amends ADR-105 decisions 1 and 5:
       - whole-number scaling and painted colours are for pixel art, which a scene says it has (`pixelArt`);
       - a tile layer is one mesh, not instanced.

       The qor-engine gate counts it (change log, a tightening).
     - **Ember Climb** (`products/qq/samples/ember-climb/`): a pixel-art platformer made only of kinds and logic, its art
       and Tiled level written by `make_art.py`.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 109 cases.
       - Comments were then corrected and an unused property removed, and the run was repeated. Seven suites passed;
         `tst_play`'s tile-level case failed at its first line, its window never reported shown, with nothing logged.
         This is the background-focus flake known from QQ Media. `tst_play` then passed whole on its own.
       - `tst_play` has 32 cases, 6 new.
       - Seven planted faults were each caught.
       - Frames were looked at: Ember Climb's title in the real Player, with the eye stopped at the level's edges, and
         its play.
     - **Found on the way, fixed:**
       - bloom brightened every painted colour (`#28c8c8` drew as `#65ffff`);
       - after the Studio's overlay changed camera, the first change to its handles was not drawn until something asked
         for a frame later;
       - Ember Climb's Again gathered the last ember where the hero stood.
     - **Next:** #60's merge, then P3.11, fixed steps and replay.

104. **19 October 2026, late night: #60 merged by the owner; P3.11, fixed steps and replay, built on `session/qq-steps`,
     committed as `93b0bf4` and pushed at the owner's word as pull request #61. CI passed every job on it (run
     39361627242).**
     - **Merged:** #60 (P3.10) at `6b58167`. CI passed every job on it (run 39357583640). P3.10 is ticked in DIRECTION.
     - **P3.11, built** (DIRECTION P3.11 has the whole list):
       - **The clock.** Play moves in ticks of 1/60 s whatever the frame rate. Each tick takes the hands and the clicks,
         then steps.
       - **Logic** has `onStep`, timers in steps, `random()` and `randomInt()` from the scene's new `seed` field,
         `overlaps()` and `outcome`.
       - **QQ's own kinematics** move Player2D, and triggers hear it without PhysX.
       - **`RunRecord` and `replayRun`** keep a run and play it again with no window and no physics world. The Player
         has `--seed`, `--record`, `--ticks` and `--replay`.
     - **A behaviour change:** presses, releases and screen-button clicks reach logic at the next tick (within 1/60 s),
       never between ticks, so a replay sees them at the same moment. Gameplay moves from `onFrame` to `onStep` in
       the Studio's starter logic, the agent's instructions, Ember Climb and the spin fixture.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 117 cases.
       - `tst_play` has 36 cases, 4 new and Ember Climb's extended.
       - Seven planted faults were each caught.
     - **Found on the way, fixed:** the Player's first check used `throws` as a logic property, which QML keeps for
       itself. The logic never built, the game played without it, and the replay matched an empty outcome with an empty
       outcome. A Behaviour now says why its logic does not build, and a replay refuses such a game (exit 3), so an
       empty outcome is never counted as a match.
     - **Next:** #61's merge, then P3.12, animation.

105. **19 October 2026, late night: #61 merged by the owner at `6b03027` (CI passed every job on the merge, run
     39363830555), and P3.11 ticked; P3.12, animation, built
     on `session/qq-animation`, committed as `07cc295` and pushed at the owner's word as pull request #62.** It was built on #61's head, since it needs P3.11's clock, and moved
     onto `main` once #61 was merged (`main`'s tree is #61's head's, so its suites' run stands).
     - **Built** (DIRECTION P3.12 has the whole list):
       - a Prop's glTF clips, skeletal and morph, on the game's steps (`Clips` in C++ takes them from Qt's wall clock);
       - tweens on any number, vector or colour field;
       - state machines;
       - Orb Run's animated lanterns (`make_lantern.py`), tweens and boulder machine.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 121 cases.
       - `tst_play` has 40 cases, 4 new.
       - Six planted faults were each caught, and a seventh's miss led to a fix (below).
     - **Found on the way, fixed:**
       - a Timeline switched off leaves what it moved where it was, so changing clips left joints bent;
       - a vector or colour read from a field is a live reference, so a tween's start followed the field;
       - a machine made in a property's binding reassigned a list the binding read, and ran again;
       - a replay draws no frames, so its models were never posed: each step now poses them.
     - **Next:** #62's merge; then P3.13, sound.

106. **19 October 2026, late night: #62 merged by the owner at `dd5a29d` (CI passed every job on its head and on the
     merge, runs 39368661482 and 39370514734), and P3.12 ticked; P3.13, sound, built on `session/qq-sound` from that
     merge, committed as `9282899` and pushed at the owner's word as pull request #63.**
     - **Built** (DIRECTION P3.13 has the whole list):
       - one-shots from logic (`play(source, { at, volume, group, reach })`), placed or heard everywhere, put away when
         played through (`Voices.qml`, measured by `Mixer.lengthOf`);
       - `Music`, a new kind: a track that fades to the next when logic sets its `source`, on through a pause;
       - volume groups (`Mixer`, a C++ singleton; logic's `setVolume` and `volumeOf`), put back by the Studio's Stop;
       - in a browser, `WebVoice` on the page's own Web Audio (`EM_JS`): each file decoded once and kept for the next
         voice, played through a gain and a stereo panner; placed sounds panned by the eye;
       - no sound in a replay;
       - Orb Run's music, chime, rumble and music switch, its sounds written by `make_sounds.py`.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 126 cases.
       - `tst_play` has 44 cases, 4 new and Orb Run's extended; `tst_scene` 23 (28 checks); `tst_studio`'s Stop also
         checks the levels.
       - Eight planted faults were each caught (DIRECTION P3.13 lists them).
       - `build.ps1 -Web` built the Player for the browser and played it in headless Chrome: the playground's sound
         decoded on the page's Web Audio, through a stereo panner (pan −0.12, the chime a little left of the eye), the
         audio running; first frame at 5.7 s, 313 frames a second; 45.1 MB served, 12.0 MB brotli (11.7 on 10
         October, before P3.7 to P3.13).
       - The language check passes over the active scope.
     - **Found on the way, fixed:**
       - a music voice with no track was given an empty file, and Qt said "cannot load file" at every play: a voice is
         now made only while it has a track;
       - the Mixer is one for the whole program, so a level a game set in a Studio play stayed after Stop; Stop now
         resets it;
       - a sound file replaced under the same name kept its first length; it is measured again when its size or time
         changes;
       - a WebVoice given another file went on playing the old one, and one given none kept it; it now stops first
         (found by reading it: the browser check plays one file once).
     - **Next:** #63's merge; then P3.14, materials.

107. **19 October 2026, late night: P3.13 committed as `9282899` and pushed at the owner's word as pull request #63 (CI
     passed every job on its head, run 39376083521); P3.14, materials, built on `session/qq-materials` on #63's head,
     not yet committed.** It followed #63's records, so it stayed on this PC until #63 was merged (no stacked pull
     requests). **20 October: #63 merged by the owner at `6e37363`; CI passed every job on the merge (run 39399914355). P3.13
     ticked. `main`'s tree is #63's head's, so the branch moved onto it as it was, and its suites' run stands.**
     - **Built** (DIRECTION P3.14 has the whole list):
       - pictures on shapes and the ground (`image`, `imageSize`; a Shape's `normalMap` and `crisp`), one copy every
         `imageSize` units on any face, projected along the shape's axes by a custom material
         (`runtime/shaders/surface.*`) and lit by Qt's own lighting;
       - `Spotlight`, a new kind;
       - HDR skies (the Scene's `sky` and `skyTurn`), with bloom gentler under them;
       - saving brings the scene's own sky into the project;
       - confined, `SceneIO.resolvedUrl` gives nothing outside the game's package, since Qt Quick 3D reads pictures and
         models past the engine's interceptor;
       - Orb Run's stone walls, slate floor, spot lights and dusk sky (`make_looks.py`), and the playground's stone,
         spot light and sky, which the browser check draws.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 133 cases.
       - `tst_world` has 9 cases, 6 new, judged by pixels; `tst_lockdown` 8; `tst_studio`'s save and Orb Run's case
         check the rest.
       - Eleven planted faults were each caught (DIRECTION P3.14 lists them).
       - `build.ps1 -Web` played the playground in headless Chrome with no problem on the page: its stone, relief, spot
         light and dusk sky drawn by WebGL (looked at), its sound panned. First frame at 7.2 s, where P3.13's playground
         took 5.7 s: the sky's lighting is prepared and the new shaders made in the browser before it. 320 frames a
         second; 45.2 MB served, 12.05 MB brotli.
       - Frames were looked at: Orb Run in the real Player, clear of its title and under it; the checks' own frames.
       - The language check passes over the active scope.
     - **Found on the way, fixed:**
       - the Ground was drawn a hundred times as wide as its solid slab;
       - a relief turned by Qt's normal matrix lost most of its tilt on a stretched shape (a 3 by 0.2 face kept a
         fifteenth of it): the pictures are laid in the shape's own units, so the normal is only turned;
       - bloom's whole-frame bleed washed an HDR sky out to white.
     - **Committed** as `7e7408f` and pushed at the owner's word as pull request #64.
     - **Next:** #64's merge; then P3.15, the Studio's basics, in parts.

108. **20 October 2026: #63 merged and P3.14 pushed as #64 at the owner's word ("commit and push all as necessary");
     P3.15's first part, editing, built on `session/qq-studio-edit` on #64's head and committed there, kept back until
     #64 is merged (no stacked pull requests).** P3.15 is built in three parts, each its own pull request: editing; the
     hierarchy with parenting and prefabs; the asset browser, the logic editor and the statistics overlay.
     - **Built** (DIRECTION P3.15 has the whole list):
       - undo and redo through every edit, each step the scene's canonical text (`studio/History.qml`), quick edits of
         one field as one step, saving not a step;
       - duplicate (Ctrl+D), every field copied but the name;
       - snapping on the handles: a grid along the axis dragged, 15-degree turns, scaling by tenths.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 136 cases.
       - `tst_studio` has 14 cases, 3 new.
       - Eight planted faults were each caught. Two of the first eight did not fail at first: an edit after an undo
         that kept the undone steps (the check now undoes once more and must land before the edit), and a duplicate's
         values shared with its original, which was not a fault at all (a value handed to `SceneIO.add` is copied
         there), so a copy named after the copy took its place.
     - **Found on the way, fixed:** naming Ctrl+Shift+Z beside the platform's redo keys made it ambiguous, and an
       ambiguous shortcut does nothing; saving had become a step to undo.
     - **20 October: #64 merged by the owner at `f6d230a`; CI passed every job on the merge (run 39432677902), and on #64's head on
       a rerun (run 39422156020, attempt 2: its first attempt's coverage job failed fetching Microsoft's package repository,
       403, before any of the repository's steps). P3.14 ticked.** `main`'s tree is #64's head's, so this part moved
       onto it as it was (the same tree, so its suites' run stands), and was pushed at the owner's word ("continue") as
       pull request #65.
     - **20 October, evening: #65's CI passed every job but security** (run 39494997610). It failed on three advisories
       published that day and the day before, in crates #65 does not touch, which fail every branch and `main` alike:
       RUSTSEC-2026-0382 (`rustls-webpki` 0.103.15), RUSTSEC-2026-0381 (`webpki-roots` 1.0.9) and RUSTSEC-2026-0383
       (`webpki-root-certs` 1.0.9 and 0.26.11). Updates fix them in QOR ID's, Social's, the launcher host's and the
       launcher core's lockfiles (0.103.16 and 1.0.10), all but `webpki-root-certs` 0.26.11 in the launcher host: no
       0.26 fix exists and subxt 0.51.1, the newest, holds it through jsonrpsee 0.24, though its library only
       re-exports the 1.x crate, now 1.0.10, and it is built only for WebAssembly. **ADR-107, the owner's ("Guarded
       ignore (Recommended)")**: the launcher's audit ignores that advisory while a new CI step, "Condition of the
       webpki-root-certs ignore", proves the 1.x under 0.26 is 1.0.10 or later, 0.26 is built for none of six Windows,
       Linux and macOS targets, and 0.26 is still in the lockfile. The step passed here and was seen to fail each way
       first: WebAssembly in its list, the bar at 1.0.11, the crate renamed, and the lockfile before the update.
       `cargo audit` passes on all four lockfiles here; their builds and tests are left to CI, which runs them. Logged
       in `GATES.toml`; committed on #65's branch as a commit of its own and pushed. `SECURITY.md` is corrected (it
       still said the launcher had no ignores) and records that `chain/`'s report-only audit now lists 20
       vulnerabilities, these among them, against the ten it measured on 20 September: re-measuring them by version,
       and taking what patch releases fix, is a task of its own, since a change to `chain/Cargo.lock` needs the whole
       chain suite.
     - **20 October, late: CI passed every job on #65's new head (run 39508315025), the security job's new step saying "the
       ignore still holds". #65 merged by the owner at `f75d082` (00:27 UTC on 21 October); CI passed every job on the
       merge (run 39521483702).** The first part is done; P3.15 stays open for the other two.
     - **Next:** the second part as its pull request (§4 item 109).

109. **20 October 2026: CI passed every job on #64 on a rerun; P3.15's second part, the hierarchy and prefabs, built
     on `session/qq-hierarchy` on part one and committed there, kept back until the parts before it are merged.** The
     first attempt's coverage job failed before any of the repository's steps: the runner's `apt-get update` got 403
     from Microsoft's package repository (`packages.microsoft.com ... is no longer signed`). The other jobs had passed
     the same step; a rerun of the failed job passed (run 39422156020, attempt 2).
     - **Built** (DIRECTION P3.15 has the whole list):
       - `Group`, entities kept together and written inside it, and `Prefab`, a group kept in `prefabs/<name>.qml` and
         placed as often as wanted, its parts its file's;
       - entities walked as a tree everywhere (`SceneIO.entities`, and `everyEntity` with prefabs' parts for play),
         bodies made at the scene's top from the world place, and QQ's kinematics measured through groups in doubles;
       - the Studio's tree, grouping, ungrouping, moving between groups, Keep as prefab, Add → Prefab…, Unpack and
         keeping back over the file; the agent's `entity_add` with a `group`;
       - Orb Run's orbs a prefab placed by logic, and its walls a group.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 142 cases.
       - `build.ps1 -Web` plays the playground in headless Chrome with no problem on the page (first frame 7.1 s, 318 frames a second, 12.06 MB brotli): kinematic and static bodies made the new way, from the world place.
       - Ten planted faults were each caught (DIRECTION P3.15 lists them and the two that did not fail at first).
     - **Found on the way, fixed:** the QQ README's table of the runtime split over two lines since P3.13; an inspector
       switch reading a field the newly selected kind lacked; Qt Quick 3D's node class is not public API, so SceneIO
       works through a node's properties and invokables.
     - **#65 merged (§4 item 108).** `main`'s tree is #65's head's, so this part moved onto the merge as it was (the
       same tree, so its suites' run stands), and was pushed at the owner's word ("#65 is now merged") as pull request
       #66.
     - **21 October: CI passed every job on #66's head (run 39523761564). #66 merged by the owner at `bc107d8` (01:22 UTC); CI
       passed every job on the merge (run 39526988403).** The second part is done.
     - **Next:** the third part as its pull request (§4 item 110).

110. **20 October 2026: P3.15's third part, the Studio's tools (the project's files, the logic editor, statistics
     with colliders in play), built on `session/qq-studio-tools` on part two and committed there, kept back until the
     parts before it are merged (#65, then the second part).** With it P3.15 is built whole: editing, the hierarchy and prefabs, and the tools.
     - **Built** (DIRECTION P3.15 has the whole list): `studio/ProjectPanel.qml`, `LogicEditor.qml` and
       `LogicHighlighter`, `StatsOverlay.qml`; `SceneIO`'s project assets by kind, `checkLogic` and `readText`; the
       scene's `debugPhysics`.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 146 cases.
       - `build.ps1 -Web` plays the playground in headless Chrome with no problem on the page: first frame 7.1 s; 217 frames a second in that run and 315 measured again straight after (a run's frame rate varies with what else the machine is doing).
       - Eight planted faults were each caught (DIRECTION P3.15 lists them).
       - The Studio looked at with the project's files, the editor open and the statistics showing.
     - **#65 merged (§4 item 108), and this part moved onto #66's head, its products unchanged. #66 merged (§4 item
       109), and this part moved onto its merge as it was (the same tree, so its suites' run stands), and was pushed
       at the owner's word ("#66 is merged successfully") as pull request #67.**
     - **21 October: CI passed every job on #67's head (run 39529063480). #67 merged by the owner at `d2357c1` (04:57 UTC); CI
       passed every job on the merge (run 39546833889). P3.15 is done and ticked.**
     - **Next:** P3.16 as its pull request (§4 item 111).
111. **20 October 2026: P3.16, saving, built on `session/qq-saving` on P3.15's third part and committed there, kept
     back until the pull requests before it are merged; and #65 green on its new head.** With it every step of ADR-105
     (P3.7 to P3.16) is built.
     - **#65's CI** on `f82f899`, the advisory fix (§4 item 108): every job passed (run 39508315025), and the security
       job's new step said "webpki-root-certs 0.26 re-exports 1.0.10 and is built for no target the launcher ships
       on; the ignore still holds." Parts two and three moved onto that head, their products unchanged and their
       records joined with the fix's.
     - **Built** (DIRECTION P3.16 has the whole list): `runtime/store.h` and `store.cpp` (`GameStore`), Logic's `save`,
       `saved` and `forget`, `SceneIO.storeOf`, `RunRecord.store`, the Studio's saved-data line with Forget, Orb Run's
       best and music kept, the playground's visits, and the Player's measurement listing logic that does not build.
     - **Found on the way, fixed:** the playground's logic had not been building (`Component.onCompleted` in a logic
       file), so its orb stood still, and no check said so. Now the native and browser checks fail on any such logic.
     - **Evidence, on the owner's PC:**
       - All eight suites pass in one run of `build.ps1`: 156 cases, 10 new.
       - `build.ps1 -Web`: first frame 7.5 s, 315 frames a second, no problem on the page; the store kept in local
         storage and found again after a reload.
       - Fourteen planted faults were each caught (DIRECTION P3.16 lists them). The first shows the made-up-scene
         route to another game's store is real: without the confinement rule in `SceneIO.storeOf`, the game read it.
       - The checks never write a person's own saves: the suites use Qt's test folders, and the Player's suite, which
         must use the real one, puts back what it found.
     - **#65, #66 and #67 merged (§4 items 108 to 110).** This part moved onto each new head with its products
       unchanged, and onto #67's merge as it was (the same tree, so its suites' run stands), and was pushed at the
       owner's word ("#67 is now merged") as pull request #68.
     - **21 October: where everything stands, and one look.** The owner asked for an assessment and for a consistent
       premium look across the ecosystem. A survey of every surface's tokens, and screenshots of eight surfaces (the
       homepage, ARQADE on the web and in Qt, QOR ID's pages, the launcher, QQ Studio and the Player), found three
       visual languages and no shared source: obsidian and gilt (the homepage, ARQADE's Qt app, QQ's game screens),
       the launcher's "Architect", and stock looks (ARQADE on the web, QQ Studio, QOR ID, parts of QOR Media), with
       values copied by hand and drifted: seven near-black backgrounds, five display faces, radii from 2 pixels to
       pills, six button styles. `docs/design/DESIGN_SYSTEM.md` says every surface takes the launcher's tokens; none
       does. Defects found on the way: ARQADE's web welcome dialog clips its last paragraph; the faintest ink in
       ARQADE's Qt app and QOR Media (#6b7385) is under 4.5:1; QOR Media has no reduced motion and buttons a keyboard
       cannot reach; QQ's screen buttons show no focus; QQ hears the system's motion setting, not the launcher's.
       **The owner chose obsidian and gilt as the one standard, in two registers** ("Obsidian & gilt, two registers
       (Recommended)"): a showcase register for the homepage, ARQADE and games, and a tool register for the launcher,
       the Studio, QOR Media and QOR ID, from one shared source of colours, type, shape and motion. ADR-108 records
       it.
     - **21 October: #68 merged by the owner at `b88185a` (11:25 UTC); CI passed every job on its head (run 39562131024) and on the
       merge (run 39591890110). P3.16 is done and ticked, and with it every step ADR-105 set out (P3.7 to P3.16).**
     - **Next:** ADR-108's first slice as its pull request (§4 item 112).
112. **21 October 2026: ADR-108, one look across every surface, accepted by the owner, and its first slice (the token
     source and the homepage) built on `session/one-look`, on #68's head, and committed there, kept back until #68 is
     merged.** DIRECTION P2.9 lists the slices.
     - **ADR-108** (the owner: "Obsidian & gilt, two registers (Recommended)"): obsidian grounds, parchment inks, ember
       the one colour of action, gilt the brand's metal; a showcase register (the homepage, ARQADE, games) and a tool
       register (the launcher, QQ Studio, QOR Media, QOR ID, the Android shell), which decide how much ornament, never
       a value. It amends ADR-100 decision 5, and `DESIGN_SYSTEM.md` no longer names the launcher as the source of the
       values.
     - **Built:**
       - `docs/design/tokens.json`, the one source: 21 colours, four faces, the type scale, tracking, two radii, one
         easing and four durations, the focus ring.
       - `docs/design/tokens.mjs`: writes each surface's generated file and copies its faces. Its `--check`, a new step
         in CI's security job, fails on a stale file, a face that differs from the source, or a text colour under
         4.5:1 on any of the five grounds or on a filled control. The faint ink is `#868f9d` (4.94:1 on `raised`, the
         weakest pairing), where ARQADE's Qt application and QOR Media carried `#6b7385` (3.4:1).
       - `docs/design/fonts/`: Cinzel, Barlow Semi Condensed, Barlow and IBM Plex Mono as Latin and Latin Extended
         WOFF2 (Fontsource 5.3.0, the homepage's ten weights), and the TrueType faces ARQADE's Qt application and QQ
         bundle, which were identical and are now checked to stay so; each family's licence (SIL OFL 1.1).
       - The homepage on it: `site/tokens.css` and `site/fonts/` generated, `styles.css` mapping its own names onto
         the tokens, the faces served from its own folder (no request to a font service; the content security policy
         now allows styles and fonts only from itself; the fonts cached for a year), the shared 8 and 12 pixel corners
         and round status tags. Its colours are unchanged.
     - **Evidence, on the owner's PC:** `site/scripts/check-site.mjs` passes, 26 checks; the page served locally and
       looked at, the same but for its corners; the share image rendered from the local faces, the same as before (the
       committed one kept). The token check fails as it should, each way: a faint ink of `#6b7385` (five pairings under
       4.5:1), one byte added to a bundled face, and one value changed in `site/tokens.css`.
     - **#68 merged (§4 item 111).** This slice moved onto its merge as it was (the same tree), and was pushed at the
       owner's word ("Merg3d") as pull request #69.
     - **Next:** #69's merge; then QQ's slice, built after this one, as its pull request; then ARQADE.
113. **21 October 2026: ADR-108's second slice, QQ, built on `session/qq-one-look`, on the first, and committed there,
     kept back until the slices before it are merged.** DIRECTION P2.9 has the whole of it.
     - **#68's CI** passed every job (run 39562131024), and #68 was merged (§4 item 111).
     - **Built:** `products/qq/runtime/tokens.h`, generated, from which `Look` takes every value; the Studio's faces,
       Barlow and IBM Plex Mono, copied by the generator into `products/qq/studio/fonts/`; `studio/StudioButton.qml`;
       the Studio restyled from Qt's stock Material Dark to the tool register; a game's screen buttons answering the
       keyboard (Tab, Enter, Space), pressed at the scene's next tick so a replay presses them too; `Look` hearing the
       launcher's word on motion (`QOR_REDUCED_MOTION`) first. A screen's fade is now the standard 260 milliseconds.
     - **Found on the way, fixed:**
       - On Windows Qt names Barlow Semi Condensed "Barlow", the family of every width. With Barlow itself loaded
         (the Studio), "Barlow" drew the wider face, and `Look.ui` had always been "Barlow". `Look` now answers with
         each face's own name, by which each face and weight is found (measured: 348 points for the narrow face, 356
         at its SemiBold, 399 for Barlow).
       - The restyled toolbar first overflowed a 1539-point window (a 1536-point screen at 125%), Play hanging 62 points
         past its edge; the notice now gives way, and a check holds every action inside the window.
       - The Studio's "Playing" pulse ran under reduced motion; it is still now.
     - **Evidence, on the owner's PC:** all eight suites pass, 160 cases, 4 new; the browser build plays (first frame
       7.6 s, 305 frames a second, its store kept across a reload); nine planted faults caught. One was missed at
       first: the Studio's faces unbundled still passed "Barlow is installed", since the narrow face answers to that name
       too. The check now asks for the file and for the reading face's width, and was seen to fail on the fault.
     - **#69 pushed (§4 item 112); this slice moved onto its head, its products unchanged.**
     - **Next:** #69's merge, then this slice's pull request; then ARQADE (the web, with its clipped welcome dialog,
       then Qt).
114. **21 October 2026: #69 merged by the owner; ADR-108's first slice live; its second, QQ, pushed as #70.**
     - **#69** passed every CI job on its head (run 39594549944) and was merged at `b964ce0`; CI passed every job on
       the merge (run 39635556360).
     - **The homepage, checked from outside:** `https://qorsync.dev/` serves `tokens.css` and its faces from its own
       `/fonts/` (`font/woff2`), under a content security policy that allows styles and fonts only from itself, and
       asks nothing of a font service; drawn in headless Edge, it shows Cinzel, Barlow Semi Condensed, Barlow and IBM
       Plex Mono.
     - **The second slice, QQ (§4 item 113), pushed as #70** with these records, on #69's merge, its products
       unchanged. The third, ARQADE on the web, built meanwhile, is committed on `session/arqade-one-look` and waits
       for #70.
     - **Next:** #70's merge, then ARQADE's pull request; then ARQADE's Qt application.
115. **21 October 2026: ADR-108's third slice, ARQADE, the web and then its Qt application, built on
     `session/arqade-one-look`, on the second, and committed there, kept back until #70 is merged.** Built while #69 waited, and moved onto #70's head once
     #70 was pushed, its products unchanged. DIRECTION P2.9 has the whole of it.
     - **Built:** `products/arqade/app/tokens.css` and `public/fonts/`, generated (a new target in
       `docs/design/tokens.mjs`, 56 in all); `app/globals.css` in the showcase register, every colour, face and timing
       of the interface a token, and every panel's and control's corner; `scripts/check-design.mjs`
       (`npm run check:design`, a step in CI's ARQADE job, logged as a tightening in `docs/GATES.toml`);
       `scripts/shots.mjs`, every screen at a desktop's size and a phone's, for review.
     - **Found on the way, fixed:** the welcome drawn in a quarter of the window (the minifier, §5 "ARQADE's
       stylesheet"); the ground rules' last lines cut by the panel's edge, which now fade under it; the hero's headline
       wider than a phone once set in Cinzel, now following the window's width; "Save project" in the mono face beside
       its ember partner; the chat window's and the level bubble's corners written by hand.
     - **Evidence, on the owner's PC:** type check; lint, no errors and the four warnings it had; 49 tests; the build;
       the phone check, 17 of 17; the tokens check; twelve planted faults each caught by the design check. One, an
       `ease-in` among parts on the tokens, passed its first version, which now reads each part. Twenty screens looked
       at before and after.
     - **#70's CI** passed every job (run 39638696981).
     - **Then the Qt application** (`products/arqade/qt/`), a second commit on the branch: `qml/Tokens.qml` generated
       (a QML target in `docs/design/tokens.mjs`), `Theme` bound to it, Barlow carried for reading, buttons and tabs
       at 8 pixel corners, the ember primary with dark ink, every size, corner and timing from the tokens, the shared
       focus ring on buttons, tabs and cards. `inkFaint` moves from `#6b7385` (under 4.5:1) to the source's.
     - **Found on the way, fixed:** a QML property named `onAccent` is a signal handler, and came out black (§5);
       Barlow, wider than the face it replaced, left single words on lines of their own, under the arenas' heading
       and in the devnet note.
     - **Evidence, on the owner's PC:** the three Qt suites pass, 35 tests (32 before, 3 new); the design check
       reads the QML in CI. Thirteen planted faults caught, ten by the check and three by the interface test; one,
       Barlow's Regular unloaded, passed the test's first version, because its Medium answered; the test now asks
       which face is drawn. The build ran in a folder of its own, so the launcher's deployed copy is untouched.
     - **Next:** #70's merge, then this slice's pull request; then the launcher's default theme, Obsidian.
116. **21 October 2026: #70 merged by the owner; ADR-108's third slice, ARQADE, pushed as #71.**
     - **#70** passed every CI job on its head (run 39638696981) and was merged at `e779957`; CI passed every job on
       the merge (run 39655121847). QQ Studio and a game's screens are in the one look on `main`; the Studio on the
       owner's PC takes it when it is next built.
     - **The third slice, ARQADE (§4 item 115), pushed as #71** with these records, its two commits (the website,
       then the Qt application) moved onto #70's merge, their products unchanged (the same tree).
     - **Once #71 is merged:** Vercel deploys the website from `main`, which fixes the live welcome screen; check it
       from outside. The app in the launcher takes the look when it is next built and deployed
       (`products/arqade/qt/build.ps1 -Deploy`) or released with the launcher.
     - **Next:** #71's merge; then the launcher's default theme, Obsidian.
117. **21 October 2026: ADR-108's fourth slice, the launcher, built on `session/launcher-one-look`, on #71's head,
     and committed there, kept back until #71 is merged.** DIRECTION P2.9 has the whole of it.
     - **The owner's decision, ADR-109:** asked whether people who had picked a theme keep it, "Everyone moves."
       Every launcher moves to Obsidian once; a theme picked after that stays; the other themes stay choices. Its
       own record, indexed in `docs/DECISIONS.md`; ADR-108 points to it.
     - **Built:** Obsidian from the generated `tokens.ts`, the default, and the move (`storedTheme`); the four faces
       carried by the launcher (`@fontsource/barlow-semi-condensed` removed); Barlow's SemiBold added to the
       source; the wordmark in gilt Cinzel; the close button's hover the status red with dark ink; the Motion
       setting passed on (`launcher_motion`, `QOR_REDUCED_MOTION` from `host::launch`); `scripts/check-look.mjs`.
     - **Found on the way, fixed:** bold reading text in Barlow was thickened by the browser from its Medium, which
       smeared it, until Barlow's SemiBold was carried; the close button's hover was the theme's `counter`, which in
       Obsidian is the gilt, so it now takes the status red in every theme; `check-accessibility.mjs` named
       Architect as the default.
     - **Evidence, on the owner's PC:** `npm run check`, all sixteen checks on a clean build (the one look 15, accessibility 55, readability 445); the host's library tests, 256 passing and 14 ignored, as before; clippy -D warnings;
       eight planted faults caught by the new check, each in its own build.
     - **#71** (ARQADE) pushed at the owner's word ("All merged" after #70; CI on #70's merge passed every job,
       run 39655121847), and passed every CI job on its head (run 39658442821), the new design step among them.
     - **Next:** #71's merge, then this slice's pull request; then QOR ID's pages and emails.
118. **22 October 2026: #71 merged by the owner; ARQADE's website live in the one look; the launcher's slice pushed as
     #72.**
     - **#71** passed every CI job on its head (run 39658442821), its new step 'Design: one look (ADR-108)' among
       them, and was merged at `a401c0f`; CI passed every job on the merge (run 39712947321).
     - **Deployed:** Vercel's `qor-arqade` built the merge for production (its GitHub status: "Deployment has
       completed"). `demiurge-site` skipped it by its Ignored Build Step, since `site/` did not change.
     - **Checked from outside:** `https://qor-arqade-tau.vercel.app/` serves the stylesheet with the tokens
       (`--qor-void`) and the welcome's fix, and its faces from `/fonts/` (`font/woff2`). Driven over the DevTools
       protocol at a set size, the welcome fills the window: 1440 by 900 at a desktop's, 390 by 844 at a phone's. (A
       headless `--window-size` shot seemed to show it short; that mode draws a viewport shorter than its picture.)
     - **The launcher's slice (§4 item 117), pushed as #72** with these records, on #71's merge, its products
       unchanged (the same tree).
     - **ARQADE's app in the launcher** takes the look when it is next deployed (`products/arqade/qt/build.ps1
       -Deploy`) or released with the launcher.
     - **Next:** #72's merge; then QOR ID's pages and emails.
119. **22 October 2026: ADR-108's fifth slice, QOR ID's pages and emails, built on `session/qor-id-one-look`, on #72's
     head, and committed there, kept back until #72 is merged.** DIRECTION P2.9 has the whole of it.
     - **Found on arrival:** the slice half built and uncommitted in the worktree `X:/REPOZ/qq-interface`, its new
       page test never run green: it refused `rgb` anywhere, which `color-mix(in srgb, …)` contains. It now refuses
       the colour functions themselves.
     - **Built:** a Rust target in `docs/design/tokens.mjs` (`services/qor-auth/src/look/tokens.rs`, and the faces
       copied to `look/fonts/`); `look::font` serving them at `/fonts/{file}`; the pages' stylesheet and the emails on
       the constants, sizes and tracking included; the focus ring's halo; `font-src 'self'` in the pages' policy;
       `DEPLOY-RAILWAY.md` told the faces are in the binary.
     - **Found on the way, fixed:** a field never showed the focus ring: its own `:focus` rule (`outline: none`)
       outranked the shared `:focus-visible` rule, so only its border changed. It predates this slice.
     - **Evidence, on the owner's PC:** QOR ID's 185 tests against Postgres 16 and Redis 7.4 (`--include-ignored`),
       format, clippy with warnings as errors, and the tokens check; eight planted faults caught, each in its own
       build; QOR ID run locally and driven in Chrome over the DevTools protocol, its pages at 1280 by 900 and 390 by
       844, and a verification message caught by a stand-in for Resend and looked at. Docker Desktop was started for
       the throwaway containers `qor-look-pg` and `qor-look-redis`, both removed after, and stopped again.
     - **Checked from outside:** the homepage, QOR ID's `/ready`, Social's `/health` at its Railway domain and ARQADE
       answer 200, and the devnet is at `spec_version` 9. `https://id.qorsync.dev/fonts/…` is 404 until this slice is
       deployed.
     - **Next:** #72's merge; then this slice's pull request, and QOR ID redeployed from its merge; then QOR Media and
       the Android shell, ADR-108's last slice.
120. **22 October 2026: #72 merged by the owner; the launcher in Obsidian on `main`; QOR ID's slice pushed as #73.**
     - **#72** passed every CI job on its head and was merged at `615829e` ("#72 is now merged"); CI passed every job
       on the merge (run 39737334037).
     - **Deployed:** Vercel built the merge for the homepage and ARQADE (#72 carried Barlow's SemiBold into the
       source's web set, which both copy), and both serve it from their own `/fonts/`, checked from outside; Railway skipped QOR ID and Social, whose watched paths did not change.
     - **The launcher in Obsidian** reaches testers in the next launcher release; 0.1.14 is still the download.
     - **QOR ID's slice (§4 item 119)** rebased onto the merge, its tree the one built and tested (the merge's tree
       is #72's head's), and pushed as #73 with these records.
     - **Next:** #73's merge, and QOR ID redeployed from it, checked from outside (`/fonts/…` served); then QOR Media
       and the Android shell, ADR-108's last slice.
121. **22 October 2026: #73 merged by the owner ("#73 is officially merged"); CI green on the merge; QOR ID not
     redeployed.**
     - **#73** passed every CI job on its head (run 39740383394) and was merged at `28a7f71`; CI passed every job on
       the merge (run 39759446451).
     - **Deployed:** Vercel built `qor-arqade` for production at the merge (nothing of ARQADE changed);
       `demiurge-site` skipped it by its Ignored Build Step; Railway said Social needed no deploy.
     - **QOR ID was not redeployed.** Railway posted no status for `qor-auth` on the merge, where on #54's it posted
       "Railway is deploying the service" within seconds and "Success" two minutes later. Checked from outside after
       the merge, and again hours later: `https://id.qorsync.dev/fonts/barlow-latin-400-normal.woff2` is 404 and
       `/account` carries the old content security policy, with no `font-src`. **Owner:** redeploy `qor-auth` from
       `main` in Railway's dashboard; then `/fonts/…` should answer `font/woff2`.
122. **22 October 2026: ADR-108's sixth and last slice, QOR Media and the Android shell, built on
     `session/media-one-look` from #73's merge, and pushed as #74 at the owner's word ("#73 is officially merged").**
     DIRECTION P2.9 has the whole of it.
     - **Found on arrival (a survey of both):** neither read a token; QOR Media still had all three defects ADR-108's
       survey named (faint ink `#6b7385` at 3.5 to 4.2:1, no reduced motion, every icon button `Qt.NoFocus`), and white
       ink on its one ember button (2.9:1); `DESIGN_SYSTEM.md` said their faces were already the source's, where both
       drew Segoe, Roboto and Cascadia.
     - **Built:** the generated `Tokens.qml` for both and `tokens.h` for QOR Media's highlighter; the tool register's
       faces carried by each (`fonts/`, loaded at start); both Themes bound to Tokens; QOR Media's `FocusRing` and
       `TextButton`; the Motion setting read (`QOR_REDUCED_MOTION`, else Windows' Animation effects); Tab reaching
       every control; `docs/design/check-qml.mjs`, in CI's one-look step; `tst_look`, and new cases in
       `tst_playback` and the shell's `tst_core`.
     - **Found on the way, fixed:** a click on the picture gave the keyboard back to the control that last held it,
       so the player's controls never rested again (the new test caught it); an icon button's icons resolved against
       whichever file placed it; and on a phone in its light setting the status bar's clock and icons were drawn dark
       over the obsidian ground.
     - **Evidence, on the owner's PC:** QOR Media's five suites, 39 tests (34 before); the shell's 6 (4 before); the
       tokens check (138 targets) and the QML check (10 files); ten planted faults caught by the QML check and nine by
       the applications' tests, each in its own build; QOR Media looked at playing, editing and on a PDF; the shell on
       the desktop and on the Android 16 emulator; the arm64 APK built from the clean tree (an earlier build overlapped
       the planted faults and was rebuilt), its library holding the faces and the token's status red.
     - **Not yet in people's hands:** QOR Media reaches the launcher when it is next deployed (`tools/qor-media/
       build.ps1 -Deploy`) or released with the launcher; the shell with its next APK.
     - **Next:** #74's merge, which completes ADR-108 and ticks P2.9; QOR ID's redeploy (item 121).
123. **22 October 2026: #74 merged by the owner, before its CI had finished ("I went ahead with unfinished merge"); CI
     then passed every job on its head (run 39764836344) and on the merge (run 39765971469). ADR-108 is complete.**
     - **P2.9 ticked** in DIRECTION: every surface wears the one look, from one source.
     - **QOR ID is live in the look.** It was redeployed after item 121, not by this session (Railway's statuses show
       no automatic deploy of #73's merge, and this PC has no Railway CLI). Checked from outside at 11:15 UTC:
       `/fonts/barlow-latin-400-normal.woff2` answers `font/woff2`, kept a year; `/account` carries the tokens, the
       gilt wordmark and `font-src 'self'`, and was looked at in Chrome; `/ready` 200.
     - **Deployed:** Vercel built `qor-arqade` at the merge and `demiurge-site` skipped it; Railway needed no deploy
       of QOR ID or Social, whose watched paths did not change.
     - **QOR Media deployed on the owner's PC** (`build.ps1 -Deploy`, its five suites passing first), so the
       launcher opens the new copy; the launcher's own test of the whole path (`media::tests::qor_media…`, its product
       host starting the deployed copy, which played a tone and reported to the Library) passed.
     - **Found, fixed:** #74's records broke SYSTEMS' table: two rows (the Library's, the Android launcher's) had line
       breaks inside them, which end a Markdown table row. Rejoined; no other current record has a split row.
     - **Next:** the owner's choice of what comes after ADR-108; the Beta's owner steps in §4.0 stand.
124. **22 October 2026: #75 merged by the owner ("merged"), at `5f52a66`.** CI passed every job on its head (run
     39769389640) and on the merge (run 39827196304). Checked from outside afterwards: the homepage, QOR ID's `/ready`
     and its `/fonts/`, Social's `/health` at its Railway domain and ARQADE answer 200, and the devnet is at
     `spec_version` 9. No pull request is open. These records are committed on `session/after-75` and go with the next
     one.
     - **Next:** the owner's choice; the Beta's owner steps in §4.0 stand.
125. **22 October 2026: ADR-110 proposed: emulated play in ARQADE.** The owner asked for "your absolute best
     recommendation for developmental support for a complete emulation framework within our Arqade system". No current
     record had decided emulation (the only mentions were the Android test emulator and browser emulation in checks).
     - **The recommendation (option C):** a libretro host of ARQADE's own in C++ and Qt, each emulator in a locked-down
       process under the product host; only emulators whose licences allow it and no machine that needs keys; a retro
       game published as a Cartridge (ADR-071, ADR-088); CGT, Sparks, prizes and the Mesh only on games their makers
       put there; a person's own game files a separate mode, off until the legal opinion approves it.
     - **Its roadmap, on acceptance:** P7.26 to P7.31, in the record. DIRECTION is not changed until it is accepted.
     - **Facts relied on and not re-read here** (the court case and settlements, the emulators' licences) are listed in
       the record with what checks each; the licences are read at source in its first step.
     - **Owner:** accept, amend or reject; a name (AGENTS.md §8); the first machines; bring-your-own; the legal
       opinion's scope; permission from the first games' makers. Pushed as #76 with the records of #75's merge.
     - **Deferred by the owner the same day:** "Lets go ahead and log this emulation concept somewhere for future reference. For now lets proceed with our current roadmap objectives". The record's status says Deferred; it waits on nothing and is
       the reference if the owner returns to it. Next, the roadmap: L6.2, the launcher's update channel, the second of
       §4 item 99's list (the first, QQ's steps, is done).

126. **22 October 2026: L6.2 built, the launcher's update channel (ADR-111), on `session/update-channel`, rebased onto
     #76's merge and pushed with item 127 as pull request #77.** The owner: "For now lets proceed with our current
     roadmap objectives"; L6.2 was the next of §4 item 99's list. DIRECTION L6.2 has the whole of it.
     - **ADR-111, accepted under the owner's delegation of engineering choices:** one fixed address
       (`launcher-channel/latest.json` on GitHub, since `releases/latest` skips pre-releases); the installer verified
       twice before it runs; the key on the PC that builds releases; the host checks, the person decides.
     - **Read in the source before choosing:** Tauri's updater checks the signature before running anything, but its
       check that a signature was made for the announced version is opt-in, and the Tauri CLI this launcher is built
       with (2.11.4) signs a file's name, not its version, so turning it on would refuse every update. The host's own
       check (`update.rs`, `verify_release`) does the same work from the signed file's name.
     - **Built:** `src-tauri/src/update.rs` (the channel, its status, four commands, the check every six hours); the
       updater plugin 2.12 without its `zip` feature; `tauri.conf.json`'s updater block and `tauri.release.conf.json`;
       Settings' *Check for updates by itself* (on by default, kept in `settings.json`); the Updates panel in
       Settings and *Update ready* in the title bar; `QorError::Update` in the core; `scripts/updater-key.ps1`,
       `scripts/release.mjs`, `scripts/update-e2e.mjs`; tests and two checks in `npm run check`.
     - **The key was made on this PC** (`updater-key.ps1 -Create`): `%APPDATA%\Demiurge\updater\`, its password
       sealed by Windows beside it; its public key is in `tauri.conf.json`. Nobody has backed it up yet (OWNER).
     - **Found on the way, fixed:** the updater's default `zip` feature brought zip 4 with three open advisories
       (RUSTSEC-2026-0367 to 0369), which CI's audit would have failed on; a Windows update is the installer itself, so
       the feature is off and the audit reads as on `main` (20 allowed warnings, no vulnerability). The bundler reads
       the key from `TAURI_SIGNING_PRIVATE_KEY`, not the `_PATH` variable `tauri signer sign` reads: found by the first
       end-to-end run, in the release script too. The host and the plugin each held the key: the host now reads the one
       built into the launcher's configuration, so they cannot differ.
     - **Evidence, on the owner's PC:** the end-to-end run (`update-e2e.mjs`, 14 checks): a test product of its own
       installed for this user; a tampered installer refused by the plugin, nothing installed, the launcher still
       running; 9.0.2's genuine installer announced as 9.0.3 passed the plugin and was refused by the host's own check;
       the real update installed with no question asked and the launcher reopened on 9.0.2, up to date; then removed
       without a trace. `npm run check`, eighteen steps (the updates view 27, the release script's check 3); the host's
       tests, 264 passing and 14 ignored (256 before), and the core's, 72 and 2 (71 before), with format and clippy
       clean; ten planted faults caught, each in its own build. **Not exercised:**
       Windows' permission prompt, since a real launcher installs for every user and the test product for one; and a
       real release through the channel, which waits on the owner's word.
     - **Next:** #77's merge; the first release through the channel (0.1.15) at the owner's word, installed by hand
       once by testers on 0.1.14; then L5.1 (§4 item 99's list).
127. **22 October 2026: #76 merged by the owner ("Now merged.."), at `c401aee`; then, at the owner's word, every record
     checked against the tree, CI and the live services, what was wrong corrected, and the browser checks' recurring
     failure in CI made to say why. With item 126, pull request #77.** The owner: "Analyze current direction, ensure
     that there is zero hallucination and that all developments are being implemented with the highest quality possible
     and that we are moving forward in the best possible ways."
     - **#76's merge:** CI passed every job on it (run 39865861460), the launcher job on a second attempt; its first
       stopped in `check-profiles-view.mjs` with *The browser did not expose a page to inspect* (below).
       `session/update-channel` was rebased onto the merge.
     - **How it was checked:** three read-only agents, one each for DIRECTION's ticked items against the code, SYSTEMS
       and OWNER against the tree, and this document's tables against the tree; then every finding they reported checked
       again by hand before anything changed. Test counts are CI's on `main` (run 39865861460: the chain 178, QOR ID 185,
       Social 40, ARQADE 49, the launcher's host 257 and core 72, and per pallet DRC-369 25, royalties 26 and the
       runtime's asset tests 16), or this PC's where CI cannot run them. The five live services and the devnet were read
       from outside (the devnet: `specVersion` 9, its genesis, two peers, finality two blocks behind the head). The gates
       were measured by the launcher's own `gates::Dashboard::report`, with CI and the services read, from a test file
       written for the purpose and deleted after.
     - **What held:** every one of DIRECTION's 42 ticked items has the code it names, and the 27 merge commits and pull
       request numbers it cites are real and in `main`. No claim of built work was found without its code.
     - **What was wrong, and is corrected:**
       - The Mesh called unmerged (DIRECTION L10.1 to L10.4, SYSTEMS' chain row, `docs/README.md`), and L10.4 saying it
         was not on the devnet while L10.2 said it was: merged with #40 and live since runtime 9.
       - The chain as it stood on 4 October (`spec_version` 8, 150 tests, "not changed since") in DIRECTION §6, §1 here
         and SYSTEMS: `spec_version` 9 with `pallet-mesh`, 178 tests.
       - QOR ID at 172 tests and migration 022 (§1, DIRECTION §6): 185 and 025.
       - The launcher at 0.1.9 (§1, §4.0 "A new launcher build"); Sparks "not yet in a released launcher", 0.1.13 "not
         committed" and Social "not in an installed launcher" (SYSTEMS); ARQADE "not inside the launcher" (§4.0):
         0.1.14, which has all of them, is what testers download.
       - ARQADE's avatars called not built (§1 "Not live", §4.0): ARQADE shows QOR ID's avatars (`lib/qor-session.ts`).
       - Migration 024 said to run "on QOR ID's next deploy" (§4.0): it ran on 19 October (item 100).
       - QQ's P3.7 to P3.16 missing from §1 and §4.0, whose product tracks still offered P3.1 as ready to start.
       - An item L4.7 cited for a QOR ID directory lookup (DIRECTION L4.5 and here): there is none; the lookup has no
         roadmap item.
       - `.cursorrules` calling the Spark the atomic unit, against ADR-101 and AGENTS.md §5: its currency line now says
         the ember is, in ADR-101's words; and "formerly CGT", a slip, is gone.
       - AGENTS.md §4's list of active directories leaving out `products/qq/`, where ADR-083 put QQ: added.
       - DIRECTION: M3.5 crediting AGENTS.md with pointing at `denomination.rs` (only `.cursorrules` does); M4.6's
         `spec_version` 3 in the present tense; L1.6's "nine browser checks", the count when it was ticked; P7.4's host
         paths written as the interface's; §6's chain, QOR ID and launcher rows.
       - SYSTEMS: a sentence fragment in the QFX row, and the update channel listed as planned. Its rows had grown into
         histories of every session; it is rewritten to say what each system is and where it stands, each row checked,
         with its history left to git.
       - OWNER.md: "#65 also fixed three" advisories (updates fixed two; ADR-107 ignores the third), and #76 still
         awaiting the owner's merge.
     - **The recurring CI failure.** A browser check stopping with *The browser did not expose a page to inspect*: on
       pull request #34, on the merge of #47, and twice on 22 October 2026 (run 39851643093, in the launcher's job and in
       ARQADE's phone check in the same minute on different runners, each the first Chrome its job started; and run
       39865861460, the twelfth). Every re-run passed. The checks drew Chrome's debugging port at random from one shared
       range and discarded Chrome's output, so nothing said why. The cause is not known: two failures were the first
       Chrome on a fresh runner, where no other port was held, so a drawn port's collision explains at most some of them.
       **Changed, nothing loosened:** every browser check, and ARQADE's phone check, starts Chrome with
       `--remote-debugging-port=0`, reads the port Chrome names (`scripts/chrome.mjs`; ARQADE has its own copy), and on
       failure says whether Chrome exited and what it last said. No check is retried, and no assertion changed. Evidence: the
       helper's four answers (a port named across two writes, an exit with its words, silence past the wait, a browser
       that cannot start) against stand-in processes; on this PC, all eighteen steps of `npm run check` with Edge (readability's 445 among them),
       `check-look.mjs` and `check-updates-view.mjs` with Google Chrome while the Edge run went on, and ARQADE's
       phone check (17) after its build. The next such failure will carry Chrome's words.
     - **The gates, measured:** Alpha is short of M3.2 (OPEN-2) and L1.4's last two dialogs, as on 19 October; its
       suites' units count once they are run from the dashboard. Beta: 8 units met, 35 not, 2 unmeasurable.
     - **For the owner, not changed:** three tables say where things stand (DIRECTION §6, §1 here, and SYSTEMS), and
       the two that are not the owner's to read went stale first. Making DIRECTION §6 a pointer to SYSTEMS would leave
       one status table in DIRECTION's place; that changes what the roadmap document holds, so it is the owner's call.
       QQ Studio on this PC is the copy deployed on 15 October, before every step of ADR-105: redeploying it
       (`products/qq/build.ps1 -Deploy`) waits on the owner's word.
128. **22 October 2026: #77 merged by the owner, at `d65b8b9`; launcher 0.1.15 published, the first that updates itself;
     QQ Studio redeployed on the owner's PC; DIRECTION §6 made a pointer.** The owner: "Merged #77. Commit and push all
     recommended. 2 is a big yes (updates within launcher)", and "Proceed with all recommended steps moving forward.
     Redeploy QQ Studio yes."
     - **#77's merge:** CI on it (run 39882463865) passed every job but the launcher's, which hung for 30 minutes
       restoring its cache before any step of its own ran; the run was cancelled and the job re-run, and it passed: every job on the merge is
       green. Every cache step in
       `ci.yml` now stops after ten minutes and lets its job build without the cache, which checks nothing.
     - **Launcher 0.1.15, published** by `scripts/release.mjs --publish` from `31fe7d8` on `session/release-0.1.15`:
       the pre-release `launcher-v0.1.15`, its installer `QOR-Launcher_0.1.15_x64-setup.exe`, 79,155,106 bytes,
       SHA-256 `b8b5e094e8dd6c1adb5fdb413ff668192019dffaef1a68dbc0d8fbcd772e725a`; and the pre-release
       `launcher-channel`, whose `latest.json` announces 0.1.15. The script read the manifest back from its public
       address and downloaded the installer it names, the same bytes as built, its signature verified against the
       channel's key. It carries the update channel, Obsidian, QOR Media and ARQADE's app in the one look, the wallet in
       the core, the stricter amount parser, and the Mesh dialog's fix (below). QOR Media's five suites and ARQADE's
       three passed in the build that staged them. Testers on 0.1.14 or older install it by hand once; every release
       after it reaches them from inside the launcher. L6.2 is ticked when one does. A dry run first (built, signed,
       checked, nothing published) proved the pipeline before anything went out. This PC's launcher was not started on
       the release build, so the owner's own profile did not move to Obsidian ahead of the owner's install.
     - **Found on the way, fixed:**
       - The release build stopped at QOR Media: its default build folder had been configured from the old checkout,
         and CMake refuses a folder made from another source tree. The staging scripts now build QOR Media and ARQADE in
         a folder of the checkout's own (`<app>-stage-<hash of the checkout's path>`).
       - `release.mjs` said release notes were Markdown; the launcher shows them as written. The script says so, and
         0.1.15's notes are plain.
       - `release.mjs` read the manifest back after publishing but not the installer; it now downloads the installer and
         checks its bytes and signature, as every release before was checked by hand.
       - **The Mesh grant's dialog, since 0.1.10, and the message when a chain does not pay for the Mesh, showed runs of
         spaces mid-sentence:** string literals that had lost their line continuations. Both fixed. The core's test
         dialog now refuses any prompt holding a run of spaces, and a new test asks for the Mesh grant directly, shown to
         fail with the old string. The same pattern stands in four chain strings (a genesis check in `pallet-mesh` and
         three test messages), left until the chain next changes, since none reaches a person.
     - **QQ Studio redeployed** (`products/qq/build.ps1 -Deploy`): all eight suites passed (176 test functions with each
       suite's set-up and tear-down, one skipped: the one that reaches the agent's provider only when asked), and the
       launcher's opt-in test started the deployed Studio on a project. The copy of 15 October it replaced predated every
       step of ADR-105.
     - **DIRECTION §6 is a pointer** to SYSTEMS and §1 here, with the four facts its table held that SYSTEMS did not
       (no production network, nothing audited, nothing to hold real value, the economic model decided and not built),
       under the owner's "all recommended steps".
     - **The update key's backup:** the owner ran `pwsh tools/qor-launcher/scripts/updater-key.ps1 -Backup` from a folder
       outside the repository, where that relative path does not exist. The command with the full path is in OWNER.
       It prints the password, so it is the owner's to run.
     - **Next:** this pull request; then L5.1 (ADR-112, drafted): the controller's own key signs each agent
       authorisation. Its review also found `CHALLENGE_DOMAIN`'s comment in QOR ID saying no network is deployed and
       naming the vault's old path, and agent registration missing from the log check (AGENTS.md §9); both go with it.
129. **22 October 2026: L5.1 built: the controller's own key signs every agent authorisation (ADR-112, under the
     owner's delegation), on `session/agent-authorisation`, on #78's head, and kept back until #78 is merged.** The
     owner: "Proceed with all recommended steps moving forward"; L5.1 was the next engineering step, and it closes the
     gap ADR-014 left: an access token alone could make or widen an agent.
     - **QOR ID:** `POST /api/v1/agents/register` needs `controller_signature`, the controller's own key's signature,
       verified against the key its QOR ID proved, over a statement naming every recorded field and the agent's
       single-use challenge (`authorisation_statement`); `PUT /api/v1/agents/{did}/capabilities` needs one over a change
       statement around a challenge for the controller's own account (`change_statement`). A QOR ID that proved no key,
       and an agent, cannot authorise one. `ChainAccount::verify` and `consume_challenge_signed_over` generalise the
       challenge check. Agent registration and the change joined the log check.
     - **Found by the first tests agent registration ever had:** its name rule took 3 to 32 characters of any kind, while
       an agent's QOR ID, `agent_` and the name, lives in a 20-character column under the QOR ID rule. A longer name
       failed in the database (a 500), and one with other characters was stored. Names are now 3 to 14 letters, digits,
       spaces or underscores, refused in words. `CHALLENGE_DOMAIN`'s comment no longer says no network is deployed, and
       names where the launcher builds the message.
     - **The launcher:** `src-tauri/src/agents.rs` reads `qor://agent?r=<hex JSON>` and checks it before anything is
       asked (the agent's signature, a challenge of QOR ID's shape no more than ten minutes old, the capabilities,
       autonomy, limit, name and model as QOR ID takes them); `authorise_agent` requires a signed-in profile whose QOR ID
       proved the open wallet's key, asks in the host dialog, signs the statement in the vault and registers the agent
       through `IdentityClient::register_agent`. `qor://` links are dispatched by kind; `QorError::AgentRefused` is new,
       and the interface's list of error kinds gained it and the three it lacked (`social`, `translation`, `mesh`).
     - **Evidence, on the owner's PC:** QOR ID's 192 tests (185 before) against Postgres 16 and Redis 7.4, the log check
       among them; four planted faults each caught (no controller check; capabilities left out of the statement; the
       change unsigned; the old name rule), and the log check caught the controller's signature logged at debug. The
       launcher's host, 273 tests (264 before) and the core's 73, with format and lints clean; three planted faults each
       caught (capabilities left out; the agent's signature unchecked; any key accepted). End to end, against QOR ID run
       from this branch on a fresh database: a QOR ID made by the vault's key, an agent's link, the dialog approved, the
       agent registered and listed with exactly what it asked, the same link refused again; read back from the database
       (`agent_lab_3498`, read and trade, 25 CGT, controlled by `e2e_3498`, which proved a key; both challenges spent);
       no token left in the keychain. Each side builds the statement; a test in each holds it to one vector, and a
       launcher test reads QOR ID's source for its capability list, domain and name rule.
     - **Not done:** QOR ID deployed with it (the live service still takes a token alone until then); a launcher release
       carrying `qor://agent` (0.1.16); a person authorising an agent from an installed launcher, which ticks L5.1; L5.2.
     - **Next:** #78's merge, then this branch's pull request.
130. **22 and 23 October 2026: #78 merged by the owner (at `6781caa`) and #79 (at `4aab676`), which the owner opened
     from `session/agent-authorisation`; QOR ID redeployed with ADR-112, so the live service no longer takes an access
     token alone for an agent; launcher 0.1.16 published, the first release offered through the update channel.** The
     owner: "#78 is merged. CI working on #79 now, wait for checks", then "#79 is merged."
     - **#78:** CI passed every job on its merge (run 39893784940). The homepage went live on 0.1.15 at it, checked from
       outside: its four download links and its structured data name `launcher-v0.1.15`.
     - **#79:** CI passed every job on its head (run 39894785124): QOR ID's 192 tests, the agent tests and the log check
       among them; the launcher's host 274 (14 ignored) and core 73 (1 ignored) on Linux. Merged at `4aab676`.
     - **QOR ID redeployed by Railway from #79's merge:** "Railway is deploying the service" on `4aab676` at 06:50:53
       UTC, success at 06:57:17; `/ready` 200 after. So ADR-112 is live: registering an agent, or changing what it may
       do, needs the controller's own key's signature, and SECURITY.md's known gap is closed on the live service. Not
       exercised on it from outside, which would need an account there; proven against QOR ID from the same commit on
       the owner's PC (item 129).
     - **Launcher 0.1.16, published** by `scripts/release.mjs --publish` from `1e7ab2a` on `session/after-79`: the
       pre-release `launcher-v0.1.16`, its installer `QOR-Launcher_0.1.16_x64-setup.exe`, 79,155,258 bytes, SHA-256
       `e425bfaa0b20fab186dadc543aa6bf28830bcc1142734ebdde43657ef2158dee`; the channel's `latest.json` announces it. The
       script read the manifest back and downloaded the installer it names, the same bytes as built, verified against
       the channel's key. It carries `qor://agent` (L5.1). A launcher on 0.1.15 is offered it inside the launcher:
       the first release through the channel; L6.2 is ticked when a tester's launcher takes it that way. Its notes are
       plain text, as the launcher shows them.
     - **Still to happen, a person's:** installing 0.1.15 and taking 0.1.16 through *Update and restart* (L6.2); an
       agent allowed from an installed 0.1.16 against the live QOR ID (L5.1).
131. **23 October 2026: #80 merged by the owner ("#80 is merged successfully"), at `1f8ba33`; the homepage live on
     0.1.16; a test agent, so a person can authorise one and tick L5.1.** The owner's standing word: "Proceed with all
     recommended steps moving forward."
     - **#80's merge:** Vercel deployed `demiurge-site` and `qor-arqade` at it; Railway needed no deploy of QOR ID or
       Social. The homepage's four download links name `launcher-v0.1.16`, checked from outside.
     - **No agent made `qor://agent` links**, so L5.1, ticked when a person authorises an agent from an installed
       launcher, could not be. `agents::request_link` is the agent's side, next to the launcher's, held to it by a test
       (`the_agents_side_and_the_launchers_side_agree`: what one builds the other reads, and another key's signature is
       refused); `examples/agent_link.rs` plays a test agent: an Sr25519 key of its own that is never written down or
       shown, a challenge from QOR ID (the live one by default), the link opened for the installed launcher, or printed
       with `--print`. Run against the live QOR ID with `--print` on the owner's PC: a challenge issued and a link made;
       not opened, since the launcher installed there may predate `qor://agent`.
     - **A new advisory failed the security job on #80's merge:** RUSTSEC-2026-0385 (23 October 2026), non-constant-time
       comparisons in `ctutils` 0.4.3, which the Mesh's iroh brings into the launcher's lockfile alone. Fixed by an update,
       `cargo update -p ctutils` to 0.4.5 (the advisory asks for 0.4.4 or later), nothing else moving; `cargo audit` on
       that lockfile passes, and the host's 274 tests, the Mesh's 15 among them, and its lints.
     - **Next:** the owner on 0.1.16 runs the test agent, or asks for it to be run, and allows it: L5.1. Then the asset
       work (M4.2, M4.3, M4.5) and PDF in QOR Media (L9.5).
132. **23 October 2026: #81 merged by the owner (at `e19b923`; CI green on the merge, run 40007628767); ADR-113 decided
     and its first phase built on `session/progress-records`: an asset's level, XP and stats, kept by each game that
     earns them. M4.2 and M4.5 ticked.** Asked who may change an asset's state and XP, the owner weighed four paths,
     found each lacking ("None of these paths seem to be a very good balance") and left the choice to the assistant,
     "while also urging you to consider long term usability alongside developer friendliness". The design was put to
     the owner with the question of supporting several paths by use case, and the owner answered "all is Approved", then
     "Excellent. Proceed."
     - **The chain, `spec_version` 10** (`transaction_version` stays 2): `pallet-drc369` gains calls 5 to 9,
       `set_home_game`, `authorise_writer`, `revoke_writer`, `record_progress` and `correct_progress`; the storage
       `HomeGame`, `Writers`, `WriterCount` and `Records`; the events `HomeGameSet`, `WriterAuthorised`, `WriterRevoked`,
       `StateChanged` and `StateCorrected`; and `Drc369Api` version 2, with `progress_of` and `home_game`. At most 32
       stats (ADR-047 decision 13) and 16 writers per game. The runtime's `ListedForSale` answers from
       `Drc369Royalties::Listings`, so a listed asset's records stay as the buyer saw them. No deposit (ADR-113 decision
       10); weights are placeholders owed to M7.2. `docs/protocol/PROTOCOL.md` §12 describes every rule.
     - **Evidence:** 11 new pallet tests (36 in all) and one new runtime test (17 asset tests); 190 workspace tests with
       the wasm built on the owner's PC, format and lints clean. Seven planted faults, each caught first: a level that
       may fall, no writer check, any game writing any collection, a listed asset's record changing, XP set instead of
       added, no Cartridge-holder check, and the runtime's listing check never answering yes. The pallet's test pinning
       its call names lists ten, and the two runtime tests pinning `spec_version` 9 pin 10.
     - **The launcher:** the Inventory and Market cards, and the close-up, show the home game's level and XP, read
       through `Drc369Api.progress_of` where the node's metadata has it and not at all where it does not (the devnet,
       until 10). `check-inventory-view.mjs` (252 checks) and `check-market-view.mjs` (142) cover it, both catching a
       planted fault; the host's 275 tests pass (15 ignored) with `the_card_shows_the_home_games_record_exactly`, its
       format and lints clean.
     - **The gates, tightened and logged** (`docs/GATES.toml`, 23 October): `alpha.pallets` requires the eleven new
       pallet tests and `alpha.asset-core-tests` the runtime's; M4.2 and M4.5, which `beta.chain` counts, are ticked.
       The dashboard's 15 gate tests read the file.
     - **Records corrected on the way:** PROTOCOL.md's header and §13, and `chain/README.md`, still said the devnet ran
       `spec_version` 8; it has run 9 since 17 October (item 88), read again from `wss://rpc.qorsync.dev` this day.
       HANDOFF §1's decisions row counted 111 ADRs.
     - **The owner's step:** setting `spec_version` 10 on the devnet with the sudo key, after a rehearsal against its
       live runtime as for 9 (items 87 and 88). Until then the devnet keeps no progress records and the launcher shows
       none.
     - **Next:** the rehearsal; ADR-113's second phase (visiting games) and ARQADE's SDK wrapping `record_progress`,
       when the owner wants them; L5.1's test agent (item 131); PDF in QOR Media (L9.5); M4.3.
133. **23 October 2026: runtime 10 rehearsed against the devnet's own live runtime, and passed; the file is in the
     owner's Downloads, to sign once #82 is merged.** At the owner's "Proceed."
     - **The devnet, read first** over `https://rpc.qorsync.dev`: `demiurge` `spec_version` 9, `transaction_version` 2,
       finalised at #287,950, two peers; its `:code` is 539,214 bytes with SHA-256
       `318643587f6fe625346f4d4cd4a2ac31037d0a2e88225a06999915c84e037cfc`, the file the owner set on 17 October (item 88).
     - **The file:** `cargo build -p demiurge-node --release --features sudo` at `b34151f`, #82's head, with
       `chain/Cargo.lock` untouched. `demiurge_runtime.compact.compressed.wasm` is **554,535 bytes, SHA-256
       `6192023bf09a0938ccdf7f8236403663a5269f8e908bae184a831c4c67e45dd5`**, copied to the owner's Downloads as
       `demiurge-runtime-v10.compact.compressed.wasm`.
     - **The rehearsal** (`chain/scripts/rehearse-upgrade.mjs`, which gains progress-record checks): a raw development
       specification whose genesis `:code` is the devnet's live runtime 9, **and without the Mesh's genesis period and
       answer window** (the development chain sets 10 and 5; the devnet gained the Mesh by an upgrade, so it runs the
       storage defaults, 600 and 100, and a rehearsal from 9 must too). One local validator; a transfer and a DRC-369
       mint left behind on 9; then `sudo.sudoUncheckedWeight(system.setCode(wasm))`, as the owner will sign it.
       **Passed, 35 checks:** `spec_version` 10 and `spec_name` kept; Sudo with the same key; the transfer and the asset
       kept; an ARQ Wallet payout and a round; the Mesh at 600 and 100 blocks with a node bound, an escrow opened, taken
       and closed with all 200 CGT back, and a takedown set by root; and progress records: `Drc369Api` at version 2,
       the asset minted on 9 with no home game and no record, its owner binding the home game, a writer recording level
       2, 50 XP and a stat, its holder refused as a writer (`NotWriter`), a lower level refused (`LevelMustRise`), a
       listed asset refused (`ListedForSale`, read from `Drc369Royalties`), and the Cartridge's holder correcting the
       record downwards. The client logged two decode warnings for the upgrade block's events, read with the new
       metadata, as at every rehearsal (item 44).
     - **Live today**, read from outside: the homepage (its four links name `launcher-v0.1.16`), QOR ID's `/ready`,
       Social's `/health` at its Railway domain and ARQADE each answered 200.
     - **The owner's step, after merging #82:** in polkadot.js Apps on the devnet, `Developer → Sudo`,
       `sudo.sudoUncheckedWeight` with `system.setCode` in the second dropdown (the field must read `code`, not
       `remark`: item 88's trap), the file above. Then `spec_version` 10 is read back and `:code` matched to the hash.
     - **Next:** PDF in QOR Media (L9.5), which needs no owner decision (ADR-094 decision 3), on a branch from `main`;
       ADR-113's second phase and M4.3 at the owner's word.
134. **23 October 2026: L9.5's first part built and pushed as #83: QOR Media reads PDFs, with Qt PDF.** Branch
     `session/pdf`, from `main` at `e19b923`, commit `12584f5`; it shares no file with #82 but `docs/DIRECTION.md`, in
     other lines, and `git merge-tree` merges the two cleanly in either order. These records are on #82's branch so the
     two do not conflict.
     - **What a person can do** (`tools/qor-media/qml/Reader.qml`): a PDF from the Library opens in QOR Media instead of
       the page that sent it elsewhere. Its pages one under another on their own paper, fitted to the width or the whole
       page (0), zoomed about the pointer between 10% and 800% (+, -, Ctrl and the wheel); Page Up, Page Down, Space,
       the arrows, Home and End, and Ctrl+G for a page by its number; find with every match marked, from the page in
       view (Ctrl+F, F3); text selected by dragging or Ctrl+A and copied; links followed to their page, or out to
       `http`, `https` or `mailto` and nowhere else; the pages small and the PDF's contents beside it (F9); a locked
       PDF opens with its password, kept only while it is open; a broken one says so and offers another program.
       Everything around the pages in the one look (`check-qml.mjs` passes). `Files.isPdf` routes it.
     - **Three traps of Qt 6.12 met on the way, added to §5:** a `TableView`'s delegates see the file's ids only under
       `pragma ComponentBehavior: Bound` (a ten-line probe in `qml.exe` reproduced it); `PdfSearchModel.count` is
       declared on its base class at a revision the QML type does not export, so the reader counts matches from the
       model's rows; and `forceActiveFocus()` on a `FocusScope` gives the keyboard back to the child that had it last,
       so the reader rests the keyboard on an item of its own.
     - **Evidence, on the owner's PC:** `main`'s five suites passed first (39 tests). After: **51 tests in six suites
       pass**: `tst_reader` new with 11, writing its PDFs byte by byte, one locked with the standard security handler
       (RC4) so the password path meets a real lock; `tst_files` one more (`isPdf`); the player suite's "opens elsewhere"
       test now uses an Office document. **Eight planted faults, seven caught:** links allowed anywhere, the password
       left in its field, the same wrong password not said to be wrong, zoom without bounds, the keyboard handed back to
       the page field, a new search not going to its match, fitting without the margin. The eighth, the reader's own
       clearing of the last PDF's password, was not, because Qt PDF clears it too on closing a file; the test holds that
       a second locked PDF asks again either way. `build.ps1 -Deploy` into a scratch folder carries `Qt6Pdf`,
       `Qt6PdfQuick`, the `QtQuick.Pdf` and `QtQuick.Shapes` modules; that copy opened a real 72-page PDF (Snagit's
       offline help) with its contents, and a search marked all 96 matches, looked at in screenshots. The owner's own
       deployed QOR Media (`%LOCALAPPDATA%\qor-media`) was left as it was, from `main`.
     - **Not yet:** annotating, highlighting, filling, signing, and reordering, turning and deleting pages, then saving,
       through PDFium's own interface (ADR-094 decision 3; the binaries, `bblanchon/pdfium-binaries`, were at
       `chromium/8111` on 19 October). L9.5 stays unticked until then.
     - **Next:** #82's merge and the devnet's upgrade (item 133); then PDFium for changing PDFs, the rest of L9.5.
     - **CI on #83's head `12584f5`:** every job green (the chain, QOR ID, Social, the launcher, ARQADE, coverage, and
       security with the language and design checks). #82's new head `5137b32` the same.
135. **23 October 2026: L9.5's second part built on `session/pdf-pages`, kept back until #83 is merged: a PDF's pages
     turned, deleted, moved and saved through PDFium.** Commit `0f51c12` on #83's head, pushed as a branch with no pull
     request (no stacked pull requests); it goes up against `main` once #83 is in.
     - **What a person can do** (`qml/Reader.qml`): R turns the page in view (Shift+R the other way), Delete deletes it,
       Alt+Up and Alt+Down move it, from the keyboard, the bar or under the small pages. Each change shows at once, and
       Ctrl+Z and Ctrl+Y undo and redo. Ctrl+S saves; going back with changes unsaved asks, as the text editor does. A
       locked or protected PDF is read, never changed, so a save never strips its protection. The contents beside a
       PDF now answer the keyboard: the arrows choose an entry, Return goes there (a gap the first part left).
     - **How it saves** (`src/pdfpages.cpp`, a QML type `PdfPages`): PDFium reads the file a block at a time through a
       `QFile` (any path Windows opens), turns, then deletes from the end, then moves all kept pages to the front in the
       list's order (`FPDF_MovePages`), and writes through a `QSaveFile`, renamed into place only when complete.
       **Qt PDF keeps open the file it shows, even after its source is cleared**, and Windows will not replace an open
       file; so while saving the reader shows a copy in QOR Media's temporary folder, in two steps: its pages let go
       first (the view forced to apply that at once), the file swapped on the next turn of the event loop, then the
       file reopened at the same page and the copy deleted.
     - **PDFium:** the prebuilt release `build.ps1` pins (`bblanchon/pdfium-binaries` `chromium/8111`, SHA-256
       `3452d110…7d23e`, the release's own), fetched once into `%LOCALAPPDATA%\qor-media-pdfium\`, checked every build
       and refused if it differs. A deployed copy carries `pdfium.dll` (and the installer with it, since it maps QOR
       Media's whole folder), with `pdfium-LICENSE.txt` and `pdfium-licenses\`. ADR-094 decision 3 already decided
       PDFium's own binaries; the pin follows ADR-033 (current).
     - **Evidence, on the owner's PC:** seven suites. `tst_pdfpages` new, 18 entries, every saved PDF read back
       through PDFium; `tst_reader` 16 tests (five new). The test PDFs' writer moved to `tests/testpdf.h`. On the final
       code the reader suite passed six runs in a row with no QML warning, and two full runs with deployment: the
       second passed whole, the first stopped once on the window-focus wait (`qWaitForWindowActive`), the known flake
       of a run in the background, a test that passed in every other run. **Twelve planted faults, eleven caught:** a turn not written, a page
       left out kept, the order not written, a protected PDF changed, a page listed twice, a read-only copy not deleted
       (Windows copies the read-only mark), a locked PDF's pages changing, undo doing nothing, a failed save losing the
       changes, the copy left behind, the file swapped while pages still show. The twelfth, the reader's own question
       on Esc, is also asked by the window before it closes anything unsaved. The deployed copy turned and deleted
       pages of the 72-page Snagit help, looked at in a screenshot; nothing was saved over it.
     - **Not yet:** marking up, highlighting, filling and signing (the rest of L9.5); dragging pages to move them;
       changing a locked PDF.
     - **Next:** #82 and #83 merged; then this branch rebased on `main` and opened as a pull request; then annotation.
136. **23 October 2026: #83 and #82 merged by the owner; Demiurge Devnet upgraded to `spec_version` 10 — progress
     records are live; L9.5's second part pushed as #84.** Walked through one step at a time, at the owner's "Walk me
     through devnet upgrade".
     - **Merged:** #83 at `d8c9606` (QOR Media reads PDFs), then #82 at `0b82a33` (progress records, `spec_version` 10,
       and the records of items 133 to 135). On `main`, CI's run for #83's merge was cancelled by #82's, which covers
       both and was still running when this was written.
     - **The upgrade:** read first: `spec_version` 9, the 17 October file, two peers, sudo key `5HN6PZA4…Zadn`. The
       owner merged #82, then in polkadot.js Apps (`?rpc=wss://rpc.qorsync.dev#/sudo`) chose `system.setCode` with the
       weight override, the file `demiurge-runtime-v10.compact.compressed.wasm`, and signed. **Read back from
       `wss://rpc.qorsync.dev`:** `demiurge` `spec_version` 10, `transaction_version` 2; `:code` 554,535 bytes with
       SHA-256 `6192023bf09a0938ccdf7f8236403663a5269f8e908bae184a831c4c67e45dd5`, the rehearsed file; `CodeUpdated` in
       block #289,065; `Sudo::Key` unchanged; `Drc369` with ten calls, the five new among them; `Drc369Api` at version
       2, `progress_of` answering; the Mesh's period and window still 600 and 100; finality advancing (#289,071 to
       #289,076 in 30 s) with two peers.
     - **What it means:** a launcher built from `main` now shows an asset's level and XP from the devnet. No released
       launcher carries that yet: 0.1.16 predates #82.
     - **#84:** `session/pdf-pages` rebased on `main` (one commit, no conflict, now `e87b035`), with these records.
       **On the rebased head the contents test failed every run, and it was a real bug, not the test:**
       `positionViewAtRow` on a page the view has not laid out yet moves the view at its next frame, so a second jump
       in between (End then Home; a link, then a contents entry) applied first and the stale one then landed and took
       the view back. It had passed before by timing alone, and item 135's flaky click on "Start" was this, not the
       click. Fixed at `d9269a7`: the reader forces the view's layout after each jump. The keys test now makes two jumps
       at once and checks the last one wins and stays; it caught the fault planted by removing the fix.
     - **Evidence after the fix, on the owner's PC:** all seven suites pass; the reader suite passed fourteen runs in a
       row with no warning. **Once, before the fix, the reader suite crashed** inside Qt's own image thread
       (`QQuickPixmap` loading a page through `QImageReader` and Qt PDF's image plugin, `0xc0000005`), during a reading
       test; it did not recur in the fourteen runs since. Its cause is not known; it is recorded so a recurrence is
       recognised (§5).
     - **Records corrected:** `chain/README.md`, `docs/protocol/PROTOCOL.md` and DIRECTION said the devnet ran 9 until
       the owner set 10; they now say 10, with the block.
     - **Next:** #84's review and merge; then annotation, the rest of L9.5. At the owner's word: ADR-113's second phase
       (visiting games), M4.3, or a launcher release carrying the progress cards.
137. **23 October 2026: #84 merged by the owner (at `328b70e`); QOR Media showed PDFs without their own marks, found and
     fixed; ADR-114 decided at the owner's word; pushed as #85.** CI on `main` green for #82's merge (`0b82a33`) and
     #84's (`328b70e`), every job.
     - **Asked what "signed" means** in ADR-094 decision 3, the owner chose **"Both, drawn first"**: a drawn or typed
       signature, built with the rest of marking up, and a record for certified signatures to decide later. Recorded as
       [ADR-114](docs/decisions/ADR-114-signing-a-pdf-in-qor-media.md): decision 1 accepted; decision 2 proposed, with
       options (the person's own certificate from Windows' store recommended; a certificate authority for QOR IDs;
       bought certificates; or only reading signatures). Nothing certifies a PDF until it is decided.
     - **Found before marking up began: QOR Media showed a PDF without its own marks.** Qt PDF's page image in QML draws
       no annotations, so highlights, notes, stamps and a form's filled fields already in a PDF were missing, and marks
       QOR Media saved would have been too; a PDF with a highlight and a box showed neither, in a screenshot. Qt PDF's
       page image has no setting for it; `QPdfDocument::render` draws them when asked. **Fixed** (`fba0c63`): the
       reader's pages are images drawn by an image provider of QOR Media's own, `src/pdfdrawing.cpp`
       (`image://pdfpage/<key>/<page>`), through `QPdfDocument` with annotations, on its own threads. A key names a PDF
       opened for drawing, with its password, which stays in C++. It reads through a file it opens itself, since
       `QPdfDocument::close` does not let go of a file it opened by name; closing a key closes it at once, which the
       reader does before a save swaps its file, on opening another and when it goes. Text, search, links and contents
       still come from Qt PDF. The same PDF, after: the highlight (its look made by PDFium, since the PDF gave none)
       and the box both drawn, in a screenshot.
     - **Evidence, on the owner's PC:** seven suites pass, twice in a row. `tst_pdfpages` 20 entries (a page drawn
       with its marks where Qt PDF's plain drawing leaves them out; the width rules; a locked PDF drawn only with its
       password; a closed key leaving nothing open and the file free). `tst_reader` 17 tests (a PDF's own marks on
       screen and nothing open for drawing once the reader goes; a locked PDF's page drawn after unlocking). The test
       PDF writer gains boxes with their own appearance. **Six planted faults, each caught:** pages drawn without their
       marks, no width limit, a closed key left open (caught once the test also counted what stays open), the reader
       keeping its drawing when it goes, saving while the pages' file is open for drawing, a locked PDF drawn without
       its password.
     - **On the way, a check that misled:** a test that the file is free after going back, by renaming it, failed in
       one scripted full run and passed alone and in the reader suite's own runs; the file is held for a moment until
       Qt deletes the closed reader, and on one run longer than five seconds by something not found. The test now
       holds QOR Media's part, that nothing stays open for drawing, and `tst_pdfpages` renames the file right after
       closing a key.
     - **Next:** #85's review and merge; then marking up on it: highlight, underline and strike-through of selected
       text, notes, freehand ink, and the drawn signature, saved through PDFium; then filling forms.
138. **23 October 2026: L9.5's third part built on `session/pdf-markup`, kept back until #85 is merged: a PDF marked
     up in QOR Media, and its own marks read and taken away, all written through PDFium.** Commits `39295c7` (the code)
     and `3aad3a2` (QOR Media's README) on #85's head (`09ffed8`), with these records; not pushed, since #85 is open
     and pull requests are not stacked. The live services read from outside the same evening: the homepage, QOR ID's
     `/ready`, Social's `/health` and ARQADE 200; the devnet `specVersion` 10, two peers, finalised to #291,220.
     - **What a person can do** (`qml/Reader.qml`, with `MarkupBar.qml`, `MarkItem.qml`, `SignaturePad.qml` and
       `Swatch.qml`): M shows a bar of marks under the reader's own. H, U and X highlight, underline and strike through
       the words selected; N writes a note (Ctrl+Return keeps it, Esc leaves it); D draws by hand, with the mouse, a pen
       or a finger; Sign opens a pad to draw a signature or type a name, drawn in a handwriting face. The pad says, and
       the reader says again where it is placed, that it is a picture of a signature and certifies nothing (ADR-114
       decision 1). A note or a signature goes on the page in view; it is dragged into place, or moved by the arrows
       (Shift for ten points), and a signature sized by its corner or [ and ]. Six colours, one for words and notes and
       one for the pen. A tap chooses a mark, the PDF's own too while the marks show, and Delete takes it away; a PDF's
       own note shows its words when tapped at any time, a locked PDF's too. Every change shows at once, is undone and
       redone with the pages' own, and is written by Ctrl+S in the same save as they are. A locked or protected PDF is
       read, not marked.
     - **How a mark is written** (`src/pdfmarks.cpp`, the QML singleton `PdfMarks`): as the annotation it is
       (Highlight, Underline, StrikeOut, Text, Ink, Stamp), with what other editors read to change it (the words'
       quadrilaterals, ink's list of strokes, a note's words), its colour, the print flag, a name of its own and the
       time, and an appearance QOR Media draws (`FPDFAnnot_SetAP`), so every reader shows it as QOR Media does. A
       highlight is 40% of its colour, through the graphics state PDFium gives an appearance whose `/CA` is below 1, so
       the words read through it. A signature is a stamp: a drawn one's lines, or a typed name's outlines from Great
       Vibes. Everything the reader hands over is checked whole before the file is touched (a kind, page, colour or
       place it cannot use, more than 200,000 points, a link or a mark the page lacks); one bad entry and nothing is
       written.
     - **Where a mark lands:** the reader works in the page as Qt PDF lays it out (points from its top left, turned by
       the page's own `/Rotate`). `src/pdfium.h`'s `pageSpace` maps that into the PDF's own space the way Qt PDF maps
       out of it (`FPDF_DeviceToPage` at the page's size in whole points). Marks are written before the pages are
       turned, since a turn changes that space. Tested on a page turned by a save: the highlight covers the words'
       characters, read back through PDFium.
     - **Taken away at once:** Qt PDF draws a page's marks all or none, so a page with some of its marks being taken
       away is drawn through QOR Media's own PDFium with those hidden for that drawing only
       (`image://pdfpage/<key>/<page>/<hidden>`). QOR Media's PDFium is now behind one lock (`src/pdfium.h`), since
       pages are drawn on the image provider's threads while the reader saves on the application's. A PDF's own marks
       are read a page at a time, as each is shown: PDFium parses all a page draws to load it.
     - **The page's, not the look's:** the six colours are what a PDF holds, the same in every theme, so they are kept
       in `src/pdfmarks.cpp` and reach the QML from there, which `check-qml.mjs` allows, since no QML writes a colour.
       Great Vibes (SIL Open Font License 1.1, from Google Fonts' repository, with its licence beside it) is drawn as
       outlines from C++ and named by no QML; `tokens.mjs` does not manage it.
     - **Evidence, on the owner's PC:** eight suites. Of four full runs of `build.ps1`, two passed whole, the last on
       the final tree; the other two each stopped once on the reader's window-focus wait, in tests this work does not
       touch (`a_locked_pdf_opens_with_its_password_and_not_without`, `the_pages_and_contents_sit_beside_it`), as two runs
       of the reader suite had earlier; each passed on its own, and the suite then passed seven runs in a row. That is
       the known flake of a window that gets no focus (§5), but those failures did not say which wait failed: the
       reader's helper now does, so the next one names itself. `tst_pdfmarks` new, 34
       entries (22 of them lists it must refuse, each leaving the file's bytes as they were); every mark read back
       through PDFium and drawn as the reader draws it, checked by its pixels. `tst_reader` 23 tests, six new, through
       the keys, the bar's buttons, the pointer and the pad. The reader looked at in window grabs at each step: the
       marks before saving and after, drawn by PDFium from what was written, look the same. A copy of the 72-page Snagit
       help: every page's marks read in 31 ms, one page's in 1, a highlight, a note and a typed signature written in 21
       ms and shown on its third page. **Nineteen planted faults, all caught in the end:** a highlight drawn opaque, no
       quadrilaterals, the page's turn left out of the mapping, a note without its words, ink without its strokes, a
       note's pop-up left behind, a link taken away, no limit on points, marks written after the turns, a signature
       without its box, marks being taken away still drawn; undo forgetting the marks, saving without them, Delete
       taking the page though a mark is chosen, a removed mark still drawn by the reader, a locked PDF marked, a mark's
       drag left for the view to take, Esc keeping a note, a typed signature placed as nothing. Two of the first set
       missed, and each changed the code: a second limit on points doing the first's work (now one limit, whose fault
       is caught), and code taking a release as a drag's last move, which Qt Quick already delivers (removed; the real
       cause of a short drag, the view taking it, is the fault that replaced it).
     - **Found on the way:** a mark's drag stopped half way because the pages view took it (`preventStealing`); a
       `TapHandler` that keeps its grab within bounds starves a `DragHandler` beside it; and Windows' `near` and `max`
       macros arrive with PDFium's header. These and the rest are in §5.
     - **Not yet:** filling forms (the rest of L9.5); certified signatures (ADR-114 decision 2, proposed, not decided);
       changing the words of a PDF's own note; words in a box, shapes, squiggles; marks on a locked PDF; a signature kept
       in the person's profile to place again, which ADR-114 allows; QOR Media deployed with any of it (the copy in
       `%LOCALAPPDATA%\qor-media` is #84's).
     - **Next:** #85 merged; this branch rebased on `main` and opened as a pull request; then filling forms.
139. **23 October 2026: #85 merged by the owner ("#85 is merged"), at `a0c20b9`; `session/pdf-markup` rebased onto it and
     opened as #86.** The merge's tree is #85's head's (`09ffed8`), so the rebase replayed item 138's three commits
     unchanged: now `e956c81` (the code), `0de2e7a` (QOR Media's README) and `f32385a` (its records), whose tree is
     the old head's (`d3c3cce`), byte for byte.
     - **Evidence, on the owner's PC, after the rebase:** `build.ps1` built it and ran the eight suites, each passing on
       the first run, in passed entries (a test of several cases counts each): `tst_pdfmarks` 34, `tst_reader` 23,
       `tst_pdfpages` 18, `tst_medialink` 14, `tst_playback` 9, `tst_textfile` 8, `tst_files` 5 and `tst_look` 4,
       Qt Test's own start and end of each suite not counted.
     - **CI on the merge** (run 40055718096) was still running when #86 was opened: ARQADE, Social, QOR ID, Coverage
       and Security had passed; the chain and the launcher were running.
     - **The live services, read from outside the same evening:** the homepage, QOR ID's `/ready`, Social's `/health`
       and ARQADE answered 200; the devnet `specVersion` 10, two peers, finalised to #291,864.
     - **Next:** #86's review and merge; filling forms built on its head meanwhile, kept back until it is merged.
140. **24 October 2026: L9.5's last part built on `session/pdf-forms`, on #86's head (`1836988`), kept back until #86 is
     merged: a PDF's form drawn, and filled where it is.** An earlier session had begun it with a test PDF's form fields
     (`tests/testpdf.h`) and a look at how they were drawn, uncommitted; this one went on from there. #86 is still open,
     its CI green; nothing is pushed.
     - **Found first: no page in QOR Media had ever shown a form's fields**, filled or not, so a filled form looked empty.
       Item 137's "filled fields" drawn was wrong, and DIRECTION, the README and §5 said so after it. PDFium's
       `FPDF_RenderPageBitmap` draws no widget even with `FPDF_ANNOT`; only `FPDF_FFLDraw` does, through a form
       environment, which nothing in Qt PDF makes (§5). `src/pdfdrawing.cpp` now draws a PDF with a form through QOR
       Media's PDFium and its form; one without, through Qt PDF as before, since QOR Media's PDFium is behind one lock
       the reader also takes on the application's thread to read a page's marks and fields, and drawing every page
       through it would make scrolling wait on drawing.
     - **What a person can do** (`qml/FieldItem.qml`, `qml/Reader.qml`): a form's fields are tinted in the ember at a
       tenth, a required one outlined. A tap on a text field types in it where it is, in the look's face scaled with the
       page and the field's own ink on the paper's white; Return keeps the words (Ctrl+Return where it takes lines), Tab
       keeps them and goes on, Esc leaves them as they were, and Esc again lets go of the field. A tap or Space ticks a
       box or chooses a group's button; a list's options open under it, chosen by a tap or the arrows and Return, a list
       that takes several staying open, and a drop-down list that takes words of its own with a line for them. Tab from
       the pages goes into the first field on the page in view and on through the fields after it, on to later pages'
       (Shift+Tab back), past fields that cannot be filled. A tap on a signature field opens the pad and places a picture
       of the signature in the field's middle, as large as it takes, which certifies nothing (ADR-114 decision 1). The
       focus ring shows when the keys put the keyboard on a field, not after a tap, as the look has it. Whatever is filled
       shows at once in the page's image, drawn by PDFium from a copy with the fields filled (`PdfDrawing.fill`, and the
       image's address names each filling), is undone and redone with the pages and marks, and is saved with them; words
       typed and not yet kept count as unsaved, and Ctrl+S, going back and the window's question keep them first. A field
       filled back as the file has it leaves nothing to save. A locked or protected PDF's fields are drawn and read, never
       filled.
     - **How it is filled** (`src/pdfforms.cpp`, `fillFields`): through PDFium's own form, as a person filling it in any
       reader would (`FORM_SetFocusedAnnot`, then `FORM_ReplaceSelection`, `FORM_OnChar` with a space or
       `FORM_SetIndexSelected`, then `FORM_ForceToKillFocus`), so PDFium writes each field's value and draws its widgets
       again in the field's own font. Every entry is checked before any field changes (a page or place the file lacks,
       a widget that is no field, the wrong value for the field, a field read only, words past its limit or a line break
       on one line, options a list lacks or does not take); after each, the field is read back and must hold it. One bad
       entry and nothing is written. Fields are filled first in a save, before marks are taken away, since an entry names
       its widget by its place among the page's annotations as the file has them.
     - **Shown and not filled, each saying why on a tap and in the tooltip:** a form that keeps its answers in XFA, said
       as the form opens: the IRS's W-9 and Form 1040 do (`FORMTYPE_XFA_FOREGROUND`), and Acrobat would show the answers
       in the XFA, which this PDFium cannot write; a password field, which PDFium would write into the file, as PDF 1.7
       says a reader never does; a field read only; a form's buttons, whose actions are not run. A form's scripts do not
       run at all, PDFium here having no JavaScript, so a total a script works out stays as it was.
     - **Real forms, fetched for this and not kept:** USCIS's I-9 (130 widgets, every one fillable) filled by Tab and typing
       in the reader, saved and read back, its words drawn by PDFium in the form's own bold face, and its comb boxes the
       form's own; the IRS's W-9 (23 widgets) and Form 1040 (199) read as XFA forms and not filled, the reader saying so.
     - **Evidence, on the owner's PC:** `build.ps1` (with `-Build ...\qor-media-build-qq`) built QOR Media and ran the nine
       suites, each passing, in passed entries (Qt Test's own start and end of each suite not counted): `tst_pdfforms` 37
       (new), `tst_reader` 29 (six new), `tst_pdfmarks` 34, `tst_pdfpages` 18, `tst_medialink` 14, `tst_playback` 9,
       `tst_textfile` 8, `tst_files` 5 and `tst_look` 4; then the reader suite alone five times, passing each time. Earlier,
       before the last changes, one of five runs of it had stopped on the window-focus wait (§5), in
       `its_controls_are_reached_by_tab`, which this work does not touch. `check-qml.mjs` and CI's language rule pass.
       The reader looked at in window grabs: the tint, typing in place, the words drawn after Return, a box ticked, a
       list's options open over the field near the window's foot, and the I-9 before and after saving. **Twenty-five
       planted faults, each caught**, each applied alone, built and run against its suite: fields drawn without their
       form; a PDF with a form drawn by Qt PDF; fields filled after the marks are taken away; a field read only, a
       button or a signature field filled; a password filled; widgets read in the PDF's own space; hidden widgets read;
       a page drawn without what was filled; an XFA form filled; a box left unticked, and again with nothing checking it
       afterwards; a page's form not told the page is loaded; in the reader, undo forgetting what was filled, Esc
       keeping words typed, saving without the words being typed, Tab in a field not going on, Tab from the pages not
       going in, a locked PDF filled, the pages drawn without what was filled, a signature placed beside its field, a
       tap on a field doing nothing, an XFA form saying nothing, a field filled back as the file has it still to save,
       the focus ring after a tap, and words typed and not kept going without a question. Not planted: a field's limit
       not checked first, a radio button unchosen, two options given a drop-down list; the check after filling should
       refuse each anyway, which was reasoned from PDFium's behaviour, not tried.
     - **Changed on the way:** `PdfMarks` and `PdfForms` share one header for checking what QML hands over
       (`src/qmlvalues.h`) and `pdfium.h`'s handles for a form, a page told to its form, and an annotation;
       `PdfPages.save` takes what was filled. The reader's `rest()` comment, stranded above another property, is back
       on it.
     - **Not yet:** filling a form kept in XFA (taking its XFA away, as pdf-lib and iText do, needs a change to the PDF's
       catalog that PDFium's interface does not make); a form's scripts; a list of a form's fields beside the pages; a
       `MaxLen` kept on a field above its widget, which `PdfForms.read` cannot see (PDFium cuts the words to it, and the
       save then says the words are too long); certified signatures (ADR-114 decision 2, proposed); a PDF carrying certified signatures is saved whole, which
       breaks them; QOR Media deployed with any of L9.5's last three parts.
     - **Next:** #86's merge; this branch then rebased onto `main` and opened as a pull request.
141. **24 October 2026: #86 and #87 merged by the owner ("PRs merged."), at `6654d4f` and `fddb3ed`; CI's security job
     red on `main` from a new advisory; ADR-115, the owner's guarded ignore, built and pushed as #88.** #87 had been
     opened against `main` from #86's head, at the owner's "Commit and push all recommended", saying to merge #86
     first; the owner merged #86 and then #87, thirteen seconds apart, and both landed on `main`.
     - **Found:** CI's nightly run of 24 October (run 40076288019, on `a0c20b9`) failed its security job on
       **RUSTSEC-2026-0387**, `crypto-bigint`, "Multiple timing variability issues in operations intended to be
       constant-time", published 23 October and fixed only in 0.7.6. QOR ID's, the launcher host's and the launcher
       core's audits each found `crypto-bigint` 0.5.5; Social's and Qontrol's helper's lockfiles do not hold it, and
       `chain/`'s report lists it among its twenty. (The run's newest-Rust clippy job also failed; it is report-only,
       ADR-072.) The runs on #86's and #87's merges would fail the same way.
     - **Why no update fixes it:** `sp-core` 43.0.0, the newest release, which the three pin to the chain's SDK
       (ADR-022), depends on `k256` 0.13 unconditionally, and `k256` 0.13's `elliptic-curve` 0.13 requires
       `crypto-bigint` 0.5; `k256` 0.14 moves to 0.7, and no `sp-core` uses it yet. No 0.5 release fixes it.
     - **Why it does not reach us:** what it warns of is in ECDSA's secp256k1 key operations, and no Rust file in the
       three names `ecdsa`, `k256`, `secp256k1`, `elliptic_curve` or `sp_core::ecdsa`: they use `sp-core` for sr25519
       and SS58 alone. In each lockfile `crypto-bigint` 0.5 is depended on only by `elliptic-curve`, which only `ecdsa`
       and `k256` depend on, `ecdsa` only by `k256`, and `k256` only by `sp-core`.
     - **The owner's choice** of three options put to them (a guarded ignore, recommended; leave it red; make the three
       audits reports): "Guarded ignore (Recommended)". ADR-115 records it, in ADR-089's and ADR-107's shape. The three
       `.cargo/audit.toml` files ignore it (the launcher core's file is new), each saying why, when it was verified and
       on what conditions; a comment beside `sp-core` in each `Cargo.toml` says that using `sp_core::ecdsa` fails CI;
       GATES logs it as a `ci-exemption` and its three audits' entries say so; SECURITY.md lists it, and in `chain/`'s
       section as reported, not measured.
     - **The step** ("Condition of the crypto-bigint ignore", before QOR ID's audit) fails if code in the three uses
       ECDSA or secp256k1 (a `use` list over lines too), if any crate but the next link depends on `crypto-bigint` 0.5,
       `elliptic-curve` 0.13, `ecdsa` 0.16 or `k256` 0.13, if a `crypto-bigint` below 0.7.6 joins a lockfile beside 0.5,
       or once 0.5 leaves a lockfile. **Evidence, on the owner's PC:** taken from the workflow as YAML and run, it
       passes; seen to fail each of five ways before it was committed (ECDSA on one line, ECDSA in a use list over
       lines, a changed set of dependents, 0.5 gone, 0.7.2 beside it). `cargo audit` passes in all five audited
       projects with the ignores, and the launcher core's fails without its own.
     - **Next:** #88's merge and CI green on it; then P7.20, ARQADE's four solo games rebuilt in QQ (ADR-102's order
       after P7.19), starting with Orbital on `session/arqade-orbital`.
142. **24 October 2026: P7.20's first game, Orbital, rebuilt in QQ and played in ARQADE's Qt app, on
     `session/arqade-orbital`, built on #88's head (`472d0b0`) and kept with it until #88 is merged.** Done after the
     owner's "Commit and push all recommended. Then proceed with next most recommended tasks." and "PRs merged.
     Proceed." Before it, on the same day: QOR Media deployed to `%LOCALAPPDATA%\qor-media` from `main` (`fddb3ed`)
     with all nine of its suites passing; and #88's CI passed on every job (run 40113234345), the security job with it.
     - **The game** (`products/arqade/games/orbital/`, a QQ package, so it also opens in QQ Studio and plays in the QQ
       Player): a satellite circles a gilt core; a lock while it crosses the amber arc holds, ten complete the orbit and
       three misses lose it. In 3D: physically based gilt and emissive beads under bloom, embers, an HDR sky of dust and
       260 stars, stereo sound written by `make_assets.py` (the same bytes every run), and title, HUD, pause and result
       screens. Space, Enter, the pad's A or the Lock button lock; P or the pad's Start pause. With reduced motion the
       core stops turning and the words do not swell.
     - **Rules in one place** (ADR-102's consequences): `logic/rules.mjs`, a plain module with no QQ or Qt in it. The
       game's logic hands it each event as QQ's tick delivers it and draws what it says. Its `replay(run)` plays a run
       file of QQ's (`qqRun` 1) to its outcome. The rules use only arithmetic every machine computes alike, and a test
       fails if their source uses a sine, cosine, power, logarithm or `Math.random`.
     - **In ARQADE** (`products/arqade/qt/`): the app builds QQ's runtime from `products/qq/runtime` and carries the
       package at `qrc:/games/orbital/`. Orbital's card plays it in place of the home (`qml/GameView.qml`), under a bar
       with Back and the game's name; Esc or Back returns. The launcher's reduced motion reaches the game too. Each play
       is kept as a run (QQ's `RunRecord`); nothing is sent until P7.22. The other three cards still open the website.
     - **An engineering choice, not an ADR:** the games play in ARQADE's own QML engine, unconfined, as the QQ Player
       plays a person's own project. They are ARQADE's own, reviewed here and carried in the executable. ADR-087's
       lockdown is the second wall around a stranger's game, and a stranger's game plays only in the browser
       (ADR-087 decision 1). Written in `GameView.qml` and the Qt README. If ARQADE ever plays a game it did not ship,
       this is to be decided again.
     - **Tests.** Node: `tests/orbital.test.mjs`, 8 tests, in `npm test` (57 tests, all passing on the owner's PC). One
       replays `tests/fixtures/orbital-run.json`, a run QQ recorded while ARQADE's suite played the game, to the
       outcome QQ reached; another seed, or the first lock 40 ticks later, reaches another. Qt: `tst_games`, 9 tests,
       plays the game by mouse, keyboard and a pad driven through QQ's input (the PC's own pads not read): a miss, a
       pause by P and by Start, ten locks by Space and A for a score of 2,000. The run ARQADE kept replays in QQ to the
       same outcome, and so does the fixture. Esc and Back return home. All four ARQADE suites pass through
       `build.ps1`: 44 tests, 35 before. The design check reads 12 QML files.
     - **One run in about fifteen failed:** the click on Orbital's card never reached it, so the game did not open and
       the tests after it failed with it. It was not reproduced in eight runs since. The test now moves the pointer onto
       the card and waits for the card to report the hover before it clicks, so a repeat names where the pointer was.
     - **Planted faults, both caught:** the rules' speed changed by 0.01 failed the fixture's replay in Node and in QQ
       (7 locks and 3 misses in place of 10 and 1); the game not given the record failed the run test.
     - **Looked at, and fixed:** captures of the title, a miss, a pause and the result, from QQ's Player and from the
       suite (`ARQADE_SHOT`). A dot on the core was the lamp's highlight, so the lamp went. In play, bloom washed the
       amber arc to the satellite's white, which made the game unreadable; it is now gold. Every sound warned of mono
       upmixing, so they are now written in stereo. A QQ Shape has a static physics body unless it says otherwise, so the
       380 or so beads and stars Orbital spawns each had one; they now have none.
     - **Deploy:** `build.ps1 -Deploy` also scans QQ's runtime and the games, which import Qt Quick 3D's physics,
       particles and spatial audio and Qt Multimedia. A copy deployed to the scratchpad was 187 MiB, against 155 MiB for
       the launcher's copy of 18 October. `tst_games` passed when run from inside it with Qt's own folders off the
       path. The launcher's copy in `%LOCALAPPDATA%\arqade` was not replaced.
     - **CI:** the language scan also reads `products/arqade/games/` (a tightening, logged in GATES).
     - **Not yet:** Void Runner, Synapse and Rift Survivor; a person playing Orbital in the running launcher; the
       launcher's copy and installer carrying the new ARQADE; runs sent and confirmed (P7.21, P7.22).
     - **Next:** the owner merges #88, then #89, Orbital's pull request (opened against `main`, carrying #88's two
       commits until #88 merges); then Void Runner in the same pattern.
143. **24 October 2026: Void Runner rebuilt in QQ and played in ARQADE (P7.20's second game), on
     `session/arqade-void-runner`, built on #89's head and kept with it until #88 and #89 are merged.** The owner, mid-turn:
     "Proceed with all tasks moving forward. I authorize full automation and decisions permissable. You are approved to
     continue development ops with best recommendations and highest quality when possible." Merging is blocked by
     Claude Code's own safety check ("Merge Without Review", tried on #88), so #88 and #89, both green on every job, wait
     for the owner to merge them.
     - **#89's fix first:** a QQ Shape has a static physics body unless it says otherwise, so the 380 or so beads and stars
       Orbital spawns each had one. They now have none (`aa5c1d8`, pushed to #89; its CI passed on every job).
     - **The game** (`products/arqade/games/void-runner/`): a ship runs three lanes into the collapse on the horizon. Red
       hazards and gold gems rush down the lanes; a hazard met breaches the hull, a gem is caught for 100 points. Forty-five
       seconds survived completes the run, three breaches end it, and after a breach the hull holds 0.8 s. The rush
       quickens. In 3D: a dark track between ember rails whose bars run with what is thrown, the collapse half-risen under
       bloom, the ship with a wake of embers, an HDR sky and 180 stars, and stereo sound written by `make_assets.py`.
       Lanes change by A and D, the arrows, the pad's d-pad or shoulders, or Left and Right buttons on the screen. Things
       thrown are drawn from two pools made as play starts, so nothing is made or removed during a run.
     - **Rules** (`logic/rules.mjs`), as Orbital's: one plain module, arithmetic every machine computes alike, its own
       xoshiro128** (a package reaches nothing outside itself). Rebuilt from the website's version, in steps: a thing
       thrown every 38 steps (the website's 0.62 s), speed `0.29 + seconds * 0.004` of the track a second, a hold of 48
       steps after a breach, and ten points a second counted each sixth of a second.
     - **Colour under bloom:** in play, QQ's tonemapping turned the red hazards white and the collapse pure yellow, which
       made red mean nothing. Each meaningful glow is kept near or under 1, which keeps its hue (written in the games
       README).
     - **Tests.** Node: `tests/void-runner.test.mjs`, 9 tests; `npm test` 67 on the owner's PC. Each game's test now also
       pins the SHA-256 of the rules' whole state at the end of its kept run. Qt: `tst_games` has 16 tests (Orbital's 9,
       Void Runner's 6, and one for which cards play here). A pilot steers Void Runner in real time, changing lane each way
       in turn until each is used and a gem caught with the hull whole, then steers into three hazards. Its kept run and
       `tests/fixtures/void-runner-run.json` replay in QQ to their outcomes. All four ARQADE suites pass through
       `build.ps1` (51 tests), and `tst_games` passed three runs in a row.
     - **Planted faults:** Void Runner's logic not handing presses to the rules failed the pilot's first lane change. Its
       acceleration changed from 0.004 to 0.0041 was **not** caught at first: the kept run's outcome did not move. The
       pinned state was added for that, and then failed for it and for Orbital's speed changed by 0.0001.
     - **What the test found in itself:** a separate lanes test made blind moves and once put the ship in a hazard's path
       (seen by replaying the kept run in Node); it was folded into the pilot. A seed threw sixteen hazards and no gem in
       thirteen seconds, so the pilot steers until it has caught one. And the click on a card was what failed one run in
       fifteen: on Windows the real pointer can take a synthetic one's hover away, so cards are now opened from the
       keyboard (focus and Space), and the pointer check that showed it is gone.
     - **Not yet:** Synapse and Rift Survivor; the launcher's copy and installer carrying the new ARQADE; runs sent and
       confirmed (P7.21, P7.22).
     - **Next:** the owner merges #88, #89 and this branch's pull request in that order; then Synapse in the same pattern.
144. **24 October 2026: Synapse rebuilt in QQ and played in ARQADE (P7.20's third game), on `session/arqade-synapse`,
     built on #90's head.** #90 (Void Runner) was opened against `main` from `session/arqade-void-runner`; #88, #89 and
     #90 wait on the owner's merge, in that order.
     - **The game** (`products/arqade/games/synapse/`): four nodes, Origin, Echo, Pulse and Aether, light in a signal that
       the player repeats; each signal repeated grows by one, eight master the game for 1,500, and a node out of turn
       loses it (150 for each whole signal, 20 for each node of the last, as the website scored it). Keys 1 to 4, the pad's
       X, Y, B and A or its d-pad, or four buttons along the foot of the screen. In 3D: glass nodes in an arc on rings of
       their own colours, dendrites and gilt filaments, sparks from a lit node, an HDR sky of teal and violet cloud (512 by
       256, since its smooth gradients ran to 1.2 MB at twice that), and a struck-glass note for each node.
     - **Rules** (`logic/rules.mjs`): whole numbers only, the signal counted in steps (a lead of 30, each node lit 28, 42
       apart: the website's 500, 460 and 700 ms). A press is judged as it is pressed, so what it says is read before the
       step that follows.
     - **Tests.** Node: `tests/synapse.test.mjs`, 10 tests, its kept run's state pinned as the others' are; `npm test` 77.
       Qt: `tst_games` 22 tests; Synapse's are pressed while the signal shows (no answer), then three signals repeated
       by the keys, the pad and the buttons in turn, then a node out of turn for 450, and its kept run and fixture replay
       in QQ. All four suites pass through `build.ps1` (57 tests).
     - **Planted faults, all caught:** the gap between nodes changed from 42 to 41 (by the pinned state); the time lit
       changed from 28 to 27, which only the drawing reads (by the timing test); the logic not handing presses to the
       rules (the second signal never came).
     - **Looked at:** the lit node washed to cream at a glow near 1; it is capped at 0.77, which keeps its hue.
     - **Next:** Rift Survivor, the last of the four.
145. **24 October 2026: Rift Survivor rebuilt in QQ and played in ARQADE, the last of P7.20's four games, on
     `session/arqade-rift-survivor`, built on #91's head.** #91 (Synapse) was opened against `main`; #90's CI passed on
     every job. #88, #89, #90 and #91 wait on the owner's merge, in that order.
     - **The game** (`products/arqade/games/rift-survivor/`): one ship in an arena for ninety seconds; drones and armoured
       wardens close from the edges, the ship fires on the nearest by itself, kills score more in a chain and leave
       fragments, rift pulses open under the ship and breach it once armed, and every so many kills the world holds
       while a perk is chosen (Overclock, Fracture, Second wind). Four breaches end it. Moved by W, A, S and D, the
       arrows, the d-pad or the screen's four buttons; perks by 1 to 3, X, Y and B, or their buttons. In 3D: a dark board
       tilted toward the eye under a teal grid (QQ's eye looks down only 16 degrees, so the board is tilted 40 more),
       pools for everything the rules hold, and an HDR sky of teal spiral arms round an ember heart.
     - **Rules** (`logic/rules.mjs`): the website's, step by step, in its units and order. Movement is four held actions,
       pressed and released, so a replay moves exactly; the replay takes releases as well as presses. They take square
       roots, which ECMAScript specifies exactly (IEEE 754), and never `hypot` or trigonometry: the spread's three turns
       are written in as cosines and sines.
     - **Tests.** Node: `tests/rift-survivor.test.mjs`, 11 tests with a pilot that keeps clear of enemies and armed rifts
       and goes for fragments; `npm test` 88. Qt: `tst_games` 29 tests. Rift Survivor is moved each way there is, steered
       clear by the keys until the signal evolves, a perk is chosen by its button while the world holds, then it is left
       standing until four breaches end it, a perk chosen by key each time it evolves. Its kept run and
       `tests/fixtures/rift-survivor-run.json` replay in QQ. All four suites pass through `build.ps1` (64 tests), and
       `tst_games` passed twice in a row.
     - **Planted faults, both caught:** the hold after a breach changed from 96 steps to 95 (by the pinned state); releases
       not handed to the rules (the left arrow did not move a ship still moving right).
     - **Found on the way:** the logic first failed to build (`vector3d` is no type without importing Qt Quick; a `var`
       now), which the Player's capture had hidden by showing an empty board; the suite now checks the logic before it
       waits on the game. The armed rift was a cream disc that hid the ship; it is amber now. And a Void Runner run lost a
       hull point: the pilot went two lanes over and its two presses reached consecutive ticks, so the ship sat for one
       tick in the middle lane as a hazard arrived there. It now waits, as a person would, for the middle lane to clear.
     - **The home** says the four games are rebuilt in 3D and all play here; the arenas' cards still open the website.
     - **Next:** the owner merges #88 to #91 and #92 (this branch), in order. Then the arenas (P7.24); the pool and
       confirmation (P7.21, P7.22) need the owner's word first: the pool's policy numbers, and the confirmation's design
       (what one run is, how a rule version is named, what the service accepts), which builds on ADR-102 decision 5.
146. **24 October 2026: #88 to #92 merged by the owner ("All merged"); CI green on `main`; ARQADE deployed from `main`;
     the games' sounds placed in the world, and P7.20 ticked, on `session/after-92` (#93).**
     - **Merged:** #88 (ADR-115) at `e8afc09`, #89 (Orbital) at `096d1c8`, #90 (Void Runner) at `510f832`, #91 (Synapse) at
       `64cf862` and #92 (Rift Survivor) at `405e41b`. CI on the last merge passed every job (run 40125800923), so the
       security job is green on `main` again; the runs on the four merges before it were cancelled by the next.
     - **Deployed:** ARQADE built from `main` (the same source as #92's head) to `%LOCALAPPDATA%\arqade`, the copy the
       launcher opens, after all four suites passed; 191.3 MiB with Qt Quick 3D's physics and Qt Multimedia. The
       launcher's opt-in test (`arqade_started_by_the_launcher_docks_signs_in_through_it_and_quits`) started it, saw it
       dock and sign in through the launcher, and saw it quit: passed.
     - **Found:** checking P7.20's text before ticking it, "spatial sound" was not met. Every sound in the four games was
       heard everywhere, since none was given a place. QQ places a sound when logic passes `at` (Qt's SpatialSound,
       heard through the eye, panned and quieter with distance), so each sound that happens somewhere is now heard from
       there. Orbital's lock and miss come from the satellite, and Void Runner's from the ship's lane. Synapse's notes
       come from their nodes, left to right. Rift Survivor's kills come from the enemy, its rifts from where they open,
       and the rest from the ship. Music and screen-level cues are still heard everywhere.
     - **Channels:** Qt's spatial sounds warned of every stereo file ("downmixing stereo source to mono"), as its ambient
       sounds warn of mono. So each game's `make_assets.py` writes the sounds it places in one channel and the rest in two;
       only those files changed.
     - **Tests:** `tst_games` checks that a lock is heard within a unit of the satellite, a change of lane from a placed
       voice, each Synapse note at its node's own place, and a Rift Survivor kill from a place. The first run of those
       checks found the Synapse lookup taking a screen button that shares a node's name; nodes are now taken from the
       logic. All four suites pass (64 tests), `npm test` 88, the design check, and no channel warning is printed.
     - **P7.20 ticked** in DIRECTION, with its evidence; no gate counts it (`arqade.roadmap` stops at P7.15).
     - **Not yet:** the launcher's copy has the games without placed sounds until #93 merges and it is deployed again; a
       person playing them in the running launcher; a launcher release carrying them.
     - **Next:** #93's merge and a redeploy. Then the owner's word on P7.21 and P7.22: the pool's numbers, and the
       confirmation's design (what one run is, how a rule version is named, what the service accepts). The arenas in QQ
       (P7.24) can start without it.
147. **24 October 2026: ADR-116 proposed, confirming an ARQADE run by replay (P7.22), for the owner to decide;** on
     `session/run-confirmation`, on #93's head. It fills in ADR-102 decision 5:
     - one run is one game on a seed the service issued for it, so "Play again" starts a new run;
     - a rule version is the SHA-256 of the game's `rules.mjs`, with its line endings made `\n`;
     - the app sends QQ's run file at the game's end, refused past 1 MiB, an hour of ticks, an expired run (two hours)
       or anything the game does not have;
     - the service replays it with the version it issued and confirms only a run that ended, once, its run id the
       payout's outcome id;
     - a replay does not prove a person played, which the pool's caps bound.

     It decides no economic value; the pool's numbers stay P7.21's. Nothing is built on it until the owner decides.
148. **24 October 2026: #93 and #94 merged by the owner ("Merged. I have no PRs"), at `69a533a` and `be63c34`; CI passed
     every job on `main` (run 40135651010); ARQADE deployed again from `main`, its sounds placed.** No pull request is
     open.
     - ARQADE built from `be63c34` and deployed to `%LOCALAPPDATA%\arqade` after all four suites passed (64 tests),
       190.9 MiB. The launcher's opt-in test started it again, and saw it dock, sign in through the launcher and quit.
     - ADR-116 is merged **as Proposed**: merging a proposal does not decide it. P7.22 waits on the owner's decision and
       P7.21 on the pool's numbers.
     - **Next:** the two arenas in QQ (P7.24), which wait on neither.

149. **24 October 2026: P7.24 built: ARQADE's two arenas, Flux Four and Rift Reversi, rebuilt in QQ and played in
     ARQADE's Qt app against another person on ARQADE's server, through QQ's new host channel (ADR-117, accepted under the
     owner's delegation); on `session/qq-arenas`, on #95's head, opened against `main` (merge #95 first).** Not merged,
     not yet deployed to the launcher's copy.
     - **QQ, ADR-117:** a game asks its host with `askHost` and hears it in `onTold`; the host hears `asked` and answers
       with `tell`. A message is a JSON value of 16 KiB at most, at most 64 wait, heard at the next tick after its
       clicks and before its presses, and kept in the run (`told`), so a replay hears it at the same tick. A message
       reaches the logic as the JSON value it was, its lists arrays, in play and in a replay: a QML signal's own
       argument makes them Qt's lists, which `Array.isArray` does not know, and the arena's lobby never read as signed
       in until the runtime handed the value on itself (`SceneIO.tellLogic`). QQ's play suite tests it.
     - **ARQADE's server** (`src/arqadeserver.h`) plays the arenas as the website does: a match created, asked after,
       joined, moved in, conceded or closed, each post as JSON from the server's own origin (its arena routes require
       it), only an id shaped as the server's sent, a refusal in the server's words; the live arcade's matches read.
     - **ARQADE's window** (`qml/GameView.qml`) hosts an arena: it answers its asks from the server, reads the live
       arcade every 3 seconds while the lobby shows (the home's 15 stay), asks after the followed match every 1.2
       seconds, and tells only what changed. A match that could not be joined or opened is not followed, and the lobby
       is told again; a game begun afresh starts its host afresh. Both cards on the home play here now.
     - **The package** `products/arqade/games/arenas/` (its README section says how they play): a scene for each, one
       logic drawing either board in 3D, and `logic/arena.mjs`, the game's side of the conversation, which decides no
       move. The boards were looked at in the suite's screenshots, and changed for what they showed: facing the eye
       exactly they read flat, so they turn a little; the pieces were pastel under their own glow, so a lamp lights them;
       the result's veil hid the winning line, so the result stands above the board and its buttons below.
     - **Tests:** Node 105 (17 new, `tests/arenas.test.mjs`, in CI), against boards made by the server's own rules,
       with both runs QQ kept replayed and their end state pinned. ARQADE's Qt suites 78, `tst_games` 41 of them:
       the arenas played against a stand-in ARQADE that signs in as the website does, keeps matches as `arena-store.ts`
       keeps them and moves for the opponent. QQ's eight suites 177 cases. All on the owner's PC, 24 October 2026.
       In QQ's full run, one case of `tst_agent` (`qqMcpRelaysAClientToARunningStudio`) failed once, the Studio's MCP
       answer not arriving in time; it passed alone twice and in its suite again. A timing flake, recorded here, not
       yet a trap. And the arena tests failed in two runs back to back (a rematch's lobby never shown, Down not moving
       Rift Reversi's cursor), with nothing changed but an unused argument, then passed in four runs after; the real
       pointer had not moved. A failed test in `tst_games` now prints the game's screens and state. **Found in item
       152:** the window was not the active one, a full-screen game holding the foreground, so Esc and the keys the
       arenas read through the window did not arrive (§5's trap).
     - **Planted faults caught:** six in `arena.mjs` (an older board or another match's drawn, a Space that pressed a
       button also moving, the cursor not set, the clock at half speed, a refusal leaving the ask waiting, a call to
       `Date`); the host's message through the signal's own argument (QQ's play test); a post without its origin (the
       server's suite); the window never asking after the match (the Flux Four test).
     - **Found and fixed in the test itself:** `QTRY_VERIFY(click(...))` clicks again as it checks, so each button was
       pressed two to four times; the arenas ignored the extra asks, which hid it. Buttons are now clicked once, after
       they are drawn (`clickShown`).
     - **Not yet:** merged and deployed to the launcher's copy; a match played against another person on the live
       server, which needs two QOR IDs signed in through launchers (the stand-in follows `arena-store.ts`, and the app
       sends what the website sends).

150. **24 October 2026: P7.23's last part built: the website's own games point players to the QOR Launcher, where
     they are rebuilt in 3D; on `session/arqade-launcher-pointer`, on #96's head, opened against `main` (merge #95 and
     #96 first).** Not merged, so not live.
     - Each game's start and end screens on ARQADE's website (`app/games.tsx`) say the game is rebuilt in 3D in the
       launcher's ARQADE, where runs ARQADE confirms will be rewarded in Sparks, and that on the devnet a Spark is test
       CGT, with no value; then a link, *Get the QOR Launcher*, to the homepage, whose download is the current tester
       build. One place to change when a release comes, not two.
     - The launcher answers no link that opens a screen of its own (only `qor://pay` and `qor://agent`), so the website
       cannot open ARQADE in it; such a link is a launcher feature for a later release.
     - The phone check holds it (`scripts/check-phone.mjs`): the game points to the launcher by a link of 44 by 44
       pixels or more. 18 checks pass on the owner's PC, against a production build; the start screen looked at on a
       phone's screen. Lint, the type check, the design check, the language scan and 105 Node tests pass.

151. **25 October 2026: #95, #96 and #97 merged by the owner ("All three are merged."), at `66311b2`, `36f024c` and
     `beafda4`; CI passed every job on `main` (run 40172133732); ARQADE deployed from `main` with the arenas; P7.23 and
     P7.24 ticked.**
     - CI on `beafda4`: ARQADE's 105 Node tests and its 18 phone checks among every job passed.
     - ARQADE built from `beafda4` and deployed to `%LOCALAPPDATA%\arqade` after all four Qt suites passed there (78
       tests), 192.9 MiB; the executable carries both arenas' scenes and `arena.mjs`. The launcher's opt-in test
       (`arqade_started_by_the_launcher_docks_signs_in_through_it_and_quits`) started it and saw it dock, sign in through
       the launcher and quit.
     - The website on Vercel carries the pointer: its production scripts hold *Get the QOR Launcher* and its link to
       `https://qorsync.dev/` (read 25 October 2026, deployment `dpl_Cu6J7k3KZG6KCoiJMdrupVAud2Pc`). With Energy gone
       since #54, P7.23 is done.
     - P7.24 is ticked as P7.20 was: merged, deployed and played by its tests against a stand-in server. **Not yet:** a
       match between two people on the live server, and a launcher release carrying the arenas.
     - The five live services answered from outside the same day, and the devnet at `specVersion` 10 with two peers.

152. **25 October 2026: #98 merged by the owner ("#98 is now merged"), at `3da41d0`, CI green on it (run
     40197756336); then ADR-118, accepted under the owner's delegation, and built: a `qor://arqade` link opens the
     launcher at ARQADE, at a game if it names one; on `session/qor-arqade-link`, opened against `main`.** Not merged,
     and in no release yet.
     - **The link:** `qor://arqade` opens the launcher's ARQADE screen; `?play=<game>` opens one of ARQADE's six games
       too (`orbital`, `void`, `synapse`, `rift`, `flux`, `reversi`). Only one of those names goes further, the first
       `play` decides, and anything else opens only the screen. It pays, signs and signs in nothing.
     - **The launcher** (`src-tauri/src/arqade.rs`, `arqade/app.rs`, `lib.rs`): the link is read (`open_link`), the
       window comes forward, and ARQADE's app is sent a new `play` command, at once if it runs or as it starts. The
       interface asks whether a link came as it starts and on each `arqade://asked`, so a link that started the
       launcher is not lost; before the vault opens, ARQADE is the screen shown after. ARQADE's website window, which
       handed every `qor://` link to the pay path, routes this one here too. 278 host tests pass (15 ignored), clippy
       is clean, and the interface's checks pass: the build, all seventeen browser checks and the release script's
       self-test.
     - **ARQADE's app** (`qt/src/arqadelink.*`, `qt/qml/Main.qml`): `play` opens the game as its card would; a name no
       card plays opens nothing. Tested in `tst_arqadelink` (15) and in `tst_games`, whose new case passed with the
       first; the app opening any name it was sent was planted and caught.
     - **The flake of item 149, found.** `tst_games`' whole run on this branch failed 13 tests: every Esc in every
       game, then the arenas, which the games left open spoiled. A failed test now prints whether the window was
       active, and the PC's foreground window was a full-screen game (DOOM: The Dark Ages). Esc is a window shortcut,
       which Qt fires only in the active window, and a window the person is not using cannot take the foreground; the
       games' own keys, read by QQ's input from the application, still arrived. The whole run is not repeated on this
       branch's last code while the owner plays: the window would come and go over the game. It is repeated when the
       PC is free. (§5 has the trap.)
     - **Not yet:** a launcher release carrying it, and then the website offering "Play in the QOR Launcher" beside the
       download (ADR-118 decision 4: an older launcher refuses the link as a payment request).

153. **25 October 2026: #99 merged by the owner ("#99 is merged"), at `20d505f`; CI passed every job on `main` (run
     40205286214, the launcher host's 279 tests among them); ARQADE deployed from `main` with the launcher's `play`
     command, after its four suites passed whole with the PC to themselves.**
     - The first whole run on `20d505f` failed 5 of `tst_games`' tests while the owner worked at the PC: the window was
       not active when Esc and the rematch failed, and the Rift Reversi tests after them found the game still where the
       rematch had left it. The run was then held until the PC had had no input for two minutes and no game in front,
       and it passed whole: 79 tests in the four suites (`tst_games` 42). ARQADE was deployed to
       `%LOCALAPPDATA%\arqade`, and the launcher's opt-in test started it and saw it dock, sign in through the launcher
       and quit.
     - ADR-118's link is in `main` and in no release. **Not yet:** launcher 0.1.17 carrying it (the owner's go), then
       the website offering it.

154. **25 October 2026: #100 merged by the owner ("#100 is now merged"), at `13c43ad`; then P7.16's last piece built:
     the themes a level unlocks, Dusk at level 2 and Nocturne at level 5, in the launcher's Settings; on
     `session/level-themes`, opened against `main`.** Not merged, and in no release yet.
     - QOR ID has named them in its unlock list since ADR-078 (`services/qor-auth/src/handlers/progress.rs`); the
       launcher showed them as "Coming with levels" because their palettes did not exist. Now each is a theme in
       `src/styles/themes.ts`, with the unlock that opens it, offered in Settings to everyone, named with its level, and
       chosen only once the profile's progress lists it. The old "Coming with levels" list is gone.
     - Dusk: apricot light on plum. Nocturne: moonlit silver-blue on the deep blue of the small hours. Both were looked
       at on screen, and the contrast and readability checks cover them as they cover every theme.
     - `scripts/check-themes-view.mjs`, in `npm run check` and CI: 11 checks. Both locked at level 1, pressing one
       changes nothing, a theme everyone has still chosen, Dusk chosen and worn (its accent read back) once QOR ID lists
       it, Nocturne still locked. Every theme open whatever the level was planted and caught (3 checks failed). All
       eighteen browser checks and the release script's self-test pass on the owner's PC.
     - **Not yet:** merged, which ticks P7.16; the themes reach testers with launcher 0.1.17.

155. **25 October 2026: #101 merged by the owner ("#101 merged"), at `c6b5536`, CI green on it (run 40215742066), which
     ticks P7.16; launcher 0.1.17 published at the owner's "Proced.", from `5db8349` on `session/release-0.1.17`, opened
     against `main` as #102.** The owner asked for "the most current version download link on the updated web portal";
     that was 0.1.16, and 0.1.17 was offered to carry everything merged since, to start only on their word.
     - **Published** by `scripts/release.mjs --publish`: the pre-release `launcher-v0.1.17`, its installer
       `QOR-Launcher_0.1.17_x64-setup.exe`, 88,784,650 bytes, SHA-256
       `6e1081ce0883f5a82dd262af08924cee858d017671bcb840ef6507ab7dbe4c44`, downloaded back and verified against the
       channel's key; `launcher-channel`'s `latest.json` announces 0.1.17 (read back from outside). It carries P7.20
       and P7.24 (#89 to #92, #96), ADR-118's `qor://arqade` (#99), the level themes (#101), ADR-113's progress cards
       (#82) and L9.5's PDFs (#83 to #87). Unsigned (L6.1). Before it passed, QOR Media's nine Qt suites (176 tests) and
       ARQADE's four (87) ran whole in the release build.
     - **#102** carries the version in its five places, the homepage's four links, its note and its structured data, and
       the launcher README's status, whose download link still named 0.1.10. The homepage offers 0.1.16 until it merges.
     - **The release build found a real bug, fixed before the release** (`5db8349`): a rematch could be lost. The host
       asks after a finished match every 1.2 seconds; a poll answered after Rematch was pressed was taken for the new
       match (any match of the arena's was, while one was awaited), so the host followed the old one, ignored the new,
       and the screen stayed on the result. Each of `ArqadeServer`'s answers and refusals now names the match its
       request asked about, empty for a new one, and the host takes only its own request's answer; a refusal of an
       earlier request no longer cancels a create or a join. The stand-in server (`tests/fakes.h`) can hold answers back
       and send them in another order (`hold`, `release`), and the rematch test makes the crossing happen every run. The
       old logic planted back fails it at "waiting", twice; with the fix it passes. `tst_arqadeserver` checks each
       answer and refusal names its match. `18e5855` corrects one count in that test, which once heard a poll answered
       before it began listening (in the release build's second attempt; the app itself had shown the new match).
     - **Six release attempts stopped first on window suites whose window lost the keyboard**, nobody at the PC: a
       Linux window shown through WSL (`msrdc`, the owner's "QTOS · Prism", left alone) came to the front, and a window
       started in the background cannot take the keyboard back (§5). `tests/activewindow.h`, in ARQADE and QOR Media,
       lets a suite's window take it (Qt's `AlwaysActivateWindow`, through `Qt6::GuiPrivate`, whose CMake warning their
       builds now print), asks again for five seconds, and asks for it back at once when it is lost mid-test.
       `tst_games` starts each test active and holds the keyboard; QOR Media's reader, playback and look suites ask for
       it. No test was skipped or loosened. Since then `tst_games` passed 44 of 44 in seven of eight runs, four begun
       with the WSL window in front (the eighth failed on the count above). QOR Media's reader, a new window for each of
       its 31 tests, still failed a case in six of seven runs begun with the WSL window in front, and in one of five
       begun with VS Code in front; the release wrapper waits until that window is behind and retries a run stopped
       only by test cases.
     - **Checked from outside the same day:** the homepage (its links name 0.1.16 until #102 merges), QOR ID's
       `/health` and `/ready`, Social's `/health` and ARQADE (`https://qor-arqade-tau.vercel.app`) answer 200; the devnet
       answers `specVersion` 10 with two peers, finalised to #315,956.
     - **Not yet:** #102 merged, which puts 0.1.17 on the homepage; a tester taking 0.1.17 through *Update and
       restart* (L6.2); an arena match between two people on the live server; the website's "Play in the QOR Launcher"
       link (ADR-118 decision 4), which 0.1.17 now allows.

156. **25 October 2026: ADR-118 decision 4 built: ARQADE's website offers each game in the QOR Launcher, beside the
     download, now that launcher 0.1.17 answers `qor://arqade` (item 155); on `session/arqade-launcher-play`, on #102's
     head, opened against `main` as #103 (merge #102 first).** Not merged.
     - Each game's launcher note, on its start and end screens (`app/games.tsx`), links to the game in the launcher,
       `qor://arqade?play=<game>` (`orbital`, `void`, `synapse` or `rift`), beside **Get the QOR Launcher**, with a line
       saying playing there needs 0.1.17 or later, since an older launcher does not answer the link. The arenas page
       has no launcher note, so it has no link.
     - `scripts/check-phone.mjs` (in CI, `arqade.phone`): the check that read the note's first link becomes two, the
       play link (`qor://arqade?play=orbital`, 44 by 44 or more) and the download beside it (found by its address, held
       as before): 19 checks. Logged in `docs/GATES.toml` as a tightening. Hiding the play link was planted and caught
       (18 passed, 1 failed).
     - **Evidence, on the owner's PC:** type check; lint without errors (5 warnings, none in these files); 105 Node
       tests; the production build; the design check; the phone check 19 of 19. Looked at on screen at a phone's width,
       where the links stack, and a desktop's, where they sit side by side.
     - **Not yet:** merged, after which Vercel serves it; someone with 0.1.17 pressing it on the live site.

157. **25 October 2026: #102 and #103 merged by the owner ("All are merged successfully"), at `d360100` and `fd360be`;
     CI passed every job on `main` (run 40247707002). The homepage offers launcher 0.1.17, and ARQADE's website offers
     each game in the launcher.** Read from outside the same day: qorsync.dev's four download links name
     `launcher-v0.1.17`; ARQADE's live site (`https://qor-arqade-tau.vercel.app`) serves the `qor://arqade?play=` link
     in its scripts.
     - **The owner runs 0.1.17:** the launcher in `C:\Program Files\QOR Launcher` reports 0.1.17, and the ARQADE app
       running beside it is the copy bundled with it, which carries the rematch fix. Whether it arrived through *Update
       and restart* (L6.2) or by hand is asked, not known; nothing the launcher writes on this PC records an update.
     - **The owner looked for the arenas, with nobody to play.** One person can play both seats with two QOR IDs: the
       launcher's ARQADE and ARQADE's website play the same matches on the same server, and the server refuses one QOR
       ID in both seats. The website signs in with a QOR ID's name and password, and the launcher makes only QOR IDs
       bound to a key, so the second is made through QOR ID's `POST /api/v1/auth/register` (a name of 3 to 20 letters,
       digits or underscores, a password of 12 or more; three new QOR IDs per network in 30 days), then a match opened
       in one and joined in the other. Neither the launcher nor QOR ID's pages offer a form for a QOR ID with a
       password; a person without the API has no way to make the second.
     - **Not yet:** the arena match played, by two people or by one with two QOR IDs; L6.2.

158. **26 October 2026: #104 merged by the owner ("merged."), at `bf41489`, CI green on it (run 40294205647); L6.2
     ticked: the owner's launcher took 0.1.17 through the update channel.** Asked how 0.1.17 reached their PC, the owner
     answered "Update and restart": offered inside 0.1.16 and installed from it, the installer checked against the
     project's key before it ran (ADR-111). The installed launcher reports 0.1.17 (item 157). L6.2 asks for an update
     channel that verifies signatures before installing and is ticked when a release reaches a tester's launcher
     through it; this is the first known to have. `beta.launcher` counts L6.2 among its twelve items.
     - **Asked at the same time what to build next**, the owner chose M4.3's rental ("Asset rental (M4.3)") over a
       practice opponent in the arenas, a web sign-up for QOR ID and the backdrops' quality tiers. Fractional ownership
       was offered as left out until the legal review; physics was not part of the choice.

159. **26 October 2026: M4.3's rental built, the owner's choice of next work (item 158): ADR-119, accepted under the
     owner's delegation, and `spec_version` 11 on `session/asset-rental`, on #105's head, opened against `main` as #106
     (merge #105 first).** Not merged, and not on the devnet, which runs 10 until the owner's upgrade.
     - **ADR-119** amends ADR-061 decision 1: `pallet-drc369-royalties` holds the terms, the sale and the rental. A
       holder offers an asset at a price per period of blocks (zero lends it), for at most a number of periods, to
       anyone or to one account; a renter rents for some periods in one transaction, naming the most they will pay and
       the content they saw, and pays up front through `split`, so the creator's royalty and a remix source's share are
       paid on rent as on a sale (creators paid when their work is used). Until block `until` the renter is the asset's
       user (`user_of`); the rental ends by itself, with nothing sent and no hook. While it lasts the asset is held in
       place: `pallet-drc369`'s `Locker` answers that it is locked, so `pallet-nfts` refuses every transfer, and
       `revise`, `nest`, `unnest` and `list` are refused; progress records stay writable. A listed asset is not rented,
       nor a rented one listed. The renter may end it early with no refund; the holder may not; anyone clears one that
       has ended. No fee, no deposit, no platform share, no CGT created. Four decisions are product-shaped and the
       owner's to reverse before the freeze: royalties on rent, no early end by the holder, no extension, progress
       while rented.
     - **The chain:** `offer_rental`, `withdraw_rental_offer`, `rent` and `end_rental` (indices 5 to 8), the storage
       `RentalOffers`, `Rentals` and `RentalsOf`, and the runtime API `Drc369RentalApi` (`user_of`, `rental_of`,
       `rental_offer_of`, `rentals_of`, `rent_preview`, the last run and rolled back as `sale_preview` is);
       `pallet-drc369`'s `Config::Rented`, answered by the runtime from the royalties pallet as `ListedForSale` is.
       The rent is a checked multiplication of an amount by a count, refused rather than wrapped. `spec_version` 11,
       `transaction_version` 2.
     - **Evidence, on the owner's PC:** 18 new tests in `pallet-drc369-royalties` (44 in all), 2 in `pallet-drc369`
       (38), 3 in the runtime's asset tests (20); the royalties mock now mounts `pallet-drc369` as its `Locker`, as the
       runtime does. Fifteen faults planted one at a time, each failing at least one test: the lock ignoring a
       rental, a rental that never ends, rent paid to the holder whole, the renter's limit unchecked, an offer to one
       account rented by anyone, a rented asset listed, a rented asset revised, the holder ending a rental early, a
       rent wrapped, a listed asset rented, an ended rental left in the index, a part taken out of a rented whole, a
       rented asset's records held still, a revised asset rented anyway, and a rental lasting one period whatever was
       paid for. **213 workspace tests pass with the wasm built** (190 before), without `SKIP_WASM_BUILD`; format and
       clippy (`-D warnings`) clean. The ARQ Wallet's mock gained `Rented = Nothing`.
     - **Gates tightened and logged:** `alpha.pallets` requires the 20 pallet tests and `alpha.asset-core-tests` the 3
       runtime ones; the comment saying the royalties mock has no `Locker` is corrected.
     - **Records:** ADR-119 indexed (119 ADRs), ADR-061's status notes the amendment, `PROTOCOL.md` §12 and its header,
       `chain/README.md`, `CGT.md`'s access gating, DIRECTION's M4.3 (unticked).
     - **Rehearsed against the devnet's own live runtime, and passed, 43 checks** (`chain/scripts/rehearse-upgrade.mjs`,
       which gains rental checks). The devnet, read first over `https://rpc.qorsync.dev`: `demiurge` `spec_version` 10,
       `transaction_version` 2, its `:code` 554,535 bytes with SHA-256 `6192023b…5dd5`, the file set on 23 October
       (item 136). **The file:** `cargo build -p demiurge-node --release --features sudo --locked` at this branch's head,
       `chain/Cargo.lock` untouched: `demiurge_runtime.compact.compressed.wasm`, **566,765 bytes, SHA-256
       `7189a5b5747267271d984b0c8c7af4352bf18be8a252d6194d86dd178ce275f0`**, copied to the owner's Downloads as
       `demiurge-runtime-v11.compact.compressed.wasm`. **The rehearsal:** a raw development specification whose genesis
       `:code` is the devnet's live runtime 10, without the Mesh's genesis period and answer window (as the devnet has
       none); one local validator; a transfer and a mint left behind on 10; then
       `sudo.sudoUncheckedWeight(system.setCode(wasm))`. Every earlier check held (Sudo with the same key, the transfer
       and the asset kept, an ARQ Wallet payout and a round, the Mesh at 600 and 100 blocks with an escrow opened and
       closed, progress records), and the rental's: `Drc369RentalApi` at version 1, the asset minted on 10 offered and
       rented for two periods of 3 CGT paid to its holder, the renter its user until a block, a transfer refused
       (`nfts.ItemLocked`), a listing refused (`Rented`), the holder refused an early end (`NotRenter`), and the renter
       handing it back. The client logged its two decode warnings for the upgrade block's events, as at every rehearsal
       (item 44). **If #106 changes before it merges, the file is rebuilt and rehearsed again.**
     - **Not yet:** merged; the devnet upgrade to 11 with the owner's sudo key and the file above, as for 10 (item 136):
       in polkadot.js Apps on the devnet, `Developer → Sudo`, `sudo.sudoUncheckedWeight` with `system.setCode` (the field
       reading `code`, item 88's trap), then `spec_version` 11 read back and `:code` matched to the hash; the launcher's
       renting surface (offering, renting, returning, and a rented asset
       shown in the Inventory) and ARQADE's SDK reading `user_of`; fractional ownership, which waits for the legal
       review, and physics, so M4.3 stays unticked.

160. **26 October 2026: #105 merged by the owner, at `2a5ff2c`; Demiurge Devnet upgraded to `spec_version` 11 by the
     owner's sudo key ("Done."), rental live on the devnet; #106 still open.** Read back from `https://rpc.qorsync.dev`:
     `demiurge` `spec_version` 11, `transaction_version` 2; `:code` 566,765 bytes with SHA-256
     `7189a5b5747267271d984b0c8c7af4352bf18be8a252d6194d86dd178ce275f0`, the file rehearsed in item 159;
     `Drc369RentalApi` at version 1; two peers, finalised to #331,443.
     - **The devnet runs code that is not yet on `main`:** the upgrade came before #106's merge. #106's head `e61d7ef`
       passed every CI job (run 40319577516) and built that file; merging it brings `main` level with the devnet.
     - **The owner restarted the PC after this** (the owner's word, before starting the next task). The work tree was
       clean and pushed: `session/asset-rental` at this commit.
     - **Next:** #106 merged; then, at the owner's word, the launcher's renting surface (offer, rent, return, and a
       rented asset shown in the Inventory, reading `Drc369RentalApi`), or another item.

## 5. Traps, so nobody re-learns them

**Releases**
- **A CMake build folder belongs to one source tree.** The staging scripts build QOR Media and ARQADE in a folder
  named for the checkout; run by hand, `build.ps1`'s default folder fails from a second checkout.
- **`release.mjs` refuses to publish from a working tree with changes,** and checks only after building: set work
  aside before `--publish`, or lose the build.
- **GitHub renames an asset's space to a hyphen** (`QOR-Launcher_…`); the signature still names the file as built
  (`QOR Launcher_…`), which is what the launcher checks.

**CI's browser checks**
- **Chrome chooses its own DevTools port** (`--remote-debugging-port=0`) and names it on stderr, which
  `scripts/chrome.mjs` reads (ARQADE's phone check has its own copy). Never go back to a drawn port, and never set
  Chrome's stderr to `ignore`: it is what says why Chrome opened no page.
- **When a check fails with "Chrome named no DevTools port" or "Chrome exited",** the error carries Chrome's own last
  lines. Read them before re-running: until 22 October 2026 nothing said why, and a re-run passed every time.

**The update channel (ADR-111)**
- **The bundler signs with `TAURI_SIGNING_PRIVATE_KEY` (the key's contents) and ignores `TAURI_SIGNING_PRIVATE_KEY_PATH`,**
  which only `tauri signer sign` reads. A build with only the path fails at the very end, after the whole build.
- **Tauri CLI 2.11.4 records no version in a signature,** only `file:<name>`; the updater's `requireSignedVersion` would
  then refuse every update. `update.rs` checks the name instead.
- **GitHub's `releases/latest` skips pre-releases,** and every launcher release is one: the channel is a fixed tag.
- **The updater plugin's default features bring zip 4** and its advisories; Windows updates need none of it.
- **WebView2 opens a debugging port when `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=<n>` is in the
  app's environment,** and the installer it starts passes that on to the relaunched app: how `update-e2e.mjs` drives a
  real install.

**QOR Media and the Android shell (ADR-108's last slice)**
- **A line break inside a Markdown table row ends the row.** A script adding to a table cell writes one line.
- **A window copied from the screen can show as transparent pixels a Qt window never drew so.** QOR Media's music glow
  over `#06070a` came out of `CopyFromScreen` with a transparent chord and edge, drawn white by an image viewer, where
  its own frame (`QQuickWindow::grabWindow`) was opaque throughout. Read the frame itself before chasing a "white line".
- **Focusing a FocusScope hands the keyboard back to the child that last had it.** To take it from a control, clear
  that control's `focus` first (`Player.release()`).
- **A relative icon path in a component resolves against the file that places it,** not the component's own. Use
  `Qt.resolvedUrl` inside the component.
- **Barlow Semi Condensed names its Regular "Semi Condensed",** not "Regular". Ask whether the drawn style names a
  heavier weight, not for an exact name.
- **Items declared inside a Window are the window's QObject children, not its `contentItem`'s:** `findChild` from the
  window.
- **Planted faults and a build of the same tree at once can package the planted file.** Rebuild after, and check
  what was packaged.
- **On Android a Qt app's status bar icons follow the phone's light or dark setting,** not the app's grounds. The shell
  sets `Qt::ColorScheme::Dark`.

**QOR ID's pages (ADR-108's fifth slice)**
- **A field's `input[type=text]:focus` outranks `input:focus-visible`,** so an `outline: none` there takes the focus
  ring away from every field, however the shared rule reads. The page test refuses `outline: none`.
- **`cargo test` builds no binary.** To run QOR ID locally after it, `cargo build` first.
- **A database migrated with `psql`, as CI's compile database is, refuses QOR ID's own migrations** ("type
  `user_role` already exists"). Run the service on a fresh database of its own.
- **This PC has Barlow Condensed and Arial Narrow installed,** so an email looked at here draws its reading text
  condensed and its headings in Arial Narrow. A reader without them sees Segoe UI; that is the stack, not a fault.

**The launcher's faces (ADR-108's fourth slice)**
- **A face the stylesheet asks for at a weight it does not carry is thickened by the browser,** and smears. Carry
  each weight the interface uses (Barlow at 400, 500 and 600 since 21 October 2026); `check-look.mjs` asks for the
  SemiBold.
- **Tailwind 4 puts every `hover:` style under `@media (hover: hover)`,** which a browser without a pointer never matches
  and DevTools cannot emulate (`Emulation.setEmulatedMedia` leaves it as it was). CI's runner draws no hover at all, so
  a forced `:hover` passes on a PC and fails there (#72's first run, 39715298276). `check-look.mjs` reads the button's
  own `hover:` rules from the stylesheet and resolves their colours on a probe.

**ARQADE's Qt application (ADR-108's third slice)**
- **A QML property named `on` and a capital is a signal handler.** `property color onAccent` reads black, silently;
  name it otherwise (`inkOnAccent`).
- **Barlow's Medium answers for its Regular** when the Regular is not loaded, and draws wide, so a width check alone
  passes. Ask `QFontInfo` for the style drawn.
- **`build.ps1`'s default build folder belongs to whichever checkout configured it first.** Another checkout fails
  CMake's source check; pass `-Build` with a folder of its own.

**ARQADE's arenas and QQ's host channel (P7.24, ADR-117)**
- **`QTRY_VERIFY(click(...))` clicks again.** The macro evaluates its expression once more after it succeeds, so a
  button was pressed two to four times, and the arenas' ignoring of a second ask hid it. Wait for the button to be
  drawn, then click once (`tst_games`' `clickShown`).
- **A QML signal's `var` argument turns a JavaScript array into a Qt list,** which `Array.isArray` does not know as an
  array, at any depth; a function called on another QML object keeps it whole. A list read back from C++ (a
  `QVariantList` in a run record) comes back the same. QQ hands a told message on through `SceneIO.tellLogic` (a
  `QJSValue` inside a `QVariant` reaches the handler as itself), and a replay's through a JSON round trip.
- **A logic file imports only QQ,** so `QtObject`, `point` and `vector3d` are not types there: declare `var`.
- **An enum read through a `var` property is not `===` to the enum's value.** `server.state === ArqadeServer.SignedIn`
  was false while signed in; read it through an `int` property first, as `Rankings.qml` does.
- **QQ's `Sun` and `Lamp` take `color`; its shapes and screen parts take `colour`.** A scene with the wrong one does
  not load, and the test reported only that the game never settled until it was made to print the game's problem.
- **A screen button draws its words in capitals;** its `text` stays as written. Compare without case.
- **`tst_games` needs its window to be the active one.** Esc, and every other QML `Shortcut`, fires only in the active
  window, and Windows gives the foreground to what the person is using: with a full-screen game in front, every Esc
  failed and the arena tests after them (25 October 2026). The games' own keys still arrive, which hides it. A failed
  test prints `Window active:`; run the suite when the PC is free, and never over someone's game.
- **Something on the owner's PC takes the keyboard from a test's window even when nobody is there** (25 October
  2026): a Linux program's window shown through WSL (`msrdc`, WSLg's host), the owner's "QTOS · Prism", open since 24
  October, comes to the front now and then, and a window started by a background process cannot take the keyboard
  back on its own. Six release attempts stopped on it in one morning, ARQADE's arena tests and QOR Media's reader alike.
  Since then the window suites ask for it (`tests/activewindow.h` in each project, through `Qt6::GuiPrivate`): Qt may
  bring the window to the front (`AlwaysActivateWindow`), each `tst_games` test and each QOR Media window starts
  active, asked again for five seconds, and a window that loses the keyboard mid-test asks for it back at once. That
  holds ARQADE's one window (44 of 44 in seven of eight runs, four begun with the WSL window in front; the eighth failed
  on a count in the test, item 155). QOR Media's reader, a new window for each of its 31 tests, still failed a case in
  six of seven runs begun with the WSL window in front, and in one of five begun with VS Code in front: a key sent
  while another window has the keyboard is lost. Run window suites with that window
  behind, and rerun a failure before believing it.
- **A host's answers can cross** (item 155): a poll of a finished match was answered after Rematch had asked for a new
  one, and the host took the old match for the new, so the rematch never showed. Each of `ArqadeServer`'s answers now
  names the match its request asked about, and the host takes only its own request's answer. The stand-in server
  (`tests/fakes.h`) can hold answers back and send them in another order (`hold`, `release`), which is how the test
  makes the crossing happen every run instead of on a slow PC.

**ARQADE's stylesheet (ADR-108's third slice)**
- **The minifier folds `translate` into `transform` when one rule sets both,** so `transform: none` does not undo
  Tailwind 4's `translate`, which centres every dialog; the welcome sat a quarter off. Undo the offset through the
  variables it reads (`--tw-translate-x`, `--tw-translate-y`).
- **A `color-mix()` gets a fallback the minifier writes:** the bare token at full strength, before an `@supports`
  block with the mix. A browser without `color-mix` draws that fallback. Prefer a solid token where one fits.
- **Bash heredocs on this PC halve backslashes,** and Python's `'\b'` is then a backspace: a regular expression
  written through one matched nothing until a planted fault showed it. Write scripts with a file, not a heredoc.

**QQ's animation (P3.12)**
- **A Qt Quick Timeline switched off does not put back what it moved.** `Clips` keeps each animated property's value
  from before any clip and restores it.
- **A vector or a colour read from a QML property is a live reference, not a copy.** Copy it (`Qt.vector3d(v.x, v.y,
  v.z)`) before writing the property, or the copy moves too.
- **A binding that reassigns a list it also reads runs again** (a binding loop). Append in place (`push`) when a helper
  is called from a property's binding.
- **Qt's runtime loader times keyframes in milliseconds** (`assimpimporter_rt.cpp`: 1000 / ticks a second).

**QQ's clock (P3.11)**
- **Presses reach logic at the next tick, not at once.** A check that presses a key and reads logic straight away sees
  nothing yet. Let ticks pass first (`tst_play`'s `heard()`), and let a few more pass before saying something did *not*
  arrive.
- **Gameplay goes in `onStep`.** `onFrame` comes once a drawn frame, and never runs in a replay, which has no window.
- **`throws` (and QML's other reserved words) cannot name a property in a logic file.** The file does not build; the
  Behaviour now says so in the log (`QQ logic: <name>: …`), and a replay of such a game is refused.
- **What logic removes during a tick goes when the tick ends** (`Scene._doom`), not at `Qt.callLater`: several ticks
  can run in one frame, and a replay must remove it at the same moment.

**QQ's 2D (P3.10)**
- **Bloom brightens every colour, not only the brightest.** `ExtendedSceneEnvironment`'s glow drew a painted `#28c8c8`
  as `#65ffff`, though nothing in the frame was above its HDR threshold. Pixel art is drawn with no bloom, tonemapping or
  dithering (`World.asPainted`).
- **PhysX keeps a character controller 0.1 unit from what it touches** (its contact offset, which Qt Quick 3D Physics
  leaves at PhysX's default). `Player2D` builds its capsule that much smaller all round, or its feet float 0.1 unit (1.6
  art pixels at 16 to a unit) above the floor. Qt sets the step offset to a quarter of the capsule's straight part
  (`qphysxcharactercontroller.cpp`).
- **A second View3D's first change after it switches camera may not be drawn.** The Studio's overlay went
  orthographic with a 2D scene, and the handles shown next were neither drawn nor picked until something asked for a
  frame later. Its handles now ask for one, just after they change (`Qt.callLater`); asking synchronously, inside the
  binding update, did not help.
- **`QVariantList{QVariantList{…}}` with one element is a copy, not a list of one list** (C++ list-initialisation of a
  class from one element of its own type). Wrap it: `QVariantList{QVariant(QVariantList{…})}`.

**Frame counts in CI's browser checks**
- CI's Chrome draws WebGL in software. A heavy backdrop gets 16 to 47 frames a second there, and below about 41 the
  launcher's budget stops it after thirty frames, by design. A count of frames measures the runner: test that a loop
  runs (one frame or more), and keep frame rates for the positive control, where the lightest backdrop runs.

**Qt on Android 15 and later**
- An app draws behind the on-screen keyboard, which no longer shrinks the window. `Qt.inputMethod.keyboardRectangle`
  arrives in the screen's physical pixels (1080 wide over a 411-wide window on Android 16), so it must be scaled by
  `Screen.devicePixelRatio`. The safe area takes the keyboard in a moment later. Scroll the field into view when either
  changes, and after layout has run, or the scroll is clamped short.
- In the emulator, `-gpu swiftshader_indirect` drew diagonal smears across Qt Quick's rounded rectangles. `-gpu host`,
  this PC's RTX 4060, drew them cleanly. It was the renderer, not the app.
- Qt's release APK is unsigned, and Android installs nothing unsigned: `shell/build.ps1` signs it.

**Next.js in development, at 127.0.0.1**
- `next dev` refuses its development scripts to any host but `localhost`, so a page opened at `127.0.0.1` renders and
  never comes alive: no click does anything. Open `http://localhost:<port>/`.
- After editing `app/globals.css` with the dev server running, the served stylesheet stayed the old one, even across a
  restart, until ARQADE's `.next` folder (ignored) was cleared. Check the served CSS before trusting a screenshot.
- The CSS minifier folds `transform: none; translate: none` into a single `transform` and loses `translate`, which is
  the property Tailwind 4 centres dialogs with. Zero `--tw-translate-x` and `--tw-translate-y` instead.

**Qt Quick 3D's own sky, many times over**
- Six `View3D`s in one window, each with a `ProceduralSkyTextureData` light probe, crashed Qt 6.12 with heap corruption
  or an access violation, three runs in three, and once lost the graphics device. Each view alone was fine, and so was
  each with no probe. Light every view from one image file (`products/arqade/qt/assets/sky.hdr`): Qt loads it once.
- `TorusGeometry` lies flat, around the up axis: stand it up (`eulerRotation.x: 90`) to face the camera. A
  `PrincipledMaterial` with `lighting: NoLighting` shows its base colour and drops its emission: for light, keep lighting
  on with a black base, no reflection and an emissive factor. ExtendedSceneEnvironment adds bloom after tonemapping, so
  strong bloom turns an orange to yellow.
- Qt takes the first graphics adapter, which on a laptop is the integrated one. ARQADE asks DXGI for the
  high-performance adapter and sets `QT_D3D_ADAPTER_INDEX` (`products/arqade/qt/src/main.cpp`).

**QOR Media's Qt Test output**
- Run from Git Bash or from Python, a Qt Test executable printed nothing and its exit code was all there was. Pass
  `-o <file>,txt` and read the file, as `build.ps1` does; a planted fault's "caught" means nothing until its FAIL line is
  read.

**A QML singleton that is not one**
- `set_source_files_properties(... QT_QML_SINGLETON_TYPE TRUE)` must come before `qt_add_qml_module`. After it, Theme
  was an ordinary type and every `Theme.x` read `undefined`, with a warning per binding.

**A test that hangs on its own failure**
- `assert!(m.lock().is_none(), "{:?}", m.lock())` with parking_lot locks twice once the assertion fails, and deadlocks
  instead of failing. Take the value first.

**Social's tests on a many-core PC**
- sqlx gives every `#[sqlx::test]` in a run one parent pool of 20 connections, each test up to 5. Cargo runs as many
  tests as there are cores, so twenty at once on a twenty-core PC waited on one another until some failed with
  `failed to connect test pool: PoolTimedOut`; CI's four cores never did. `services/social/.cargo/config.toml` holds the
  run to four. A `PoolTimedOut` there is this, not the change.

**The launcher's window that would not open**
- On 16 October 2026, for two hours, every debug build failed with `failed to create webview: WebView2 error …
  HRESULT(0x80070057)`, while the installed launcher opened. A fresh WebView2 profile, the older runtime, an older
  build and another path all failed the same; it passed by itself. Compare with the installed launcher before suspecting
  the code; it was the PC.

**A browser check that finds no page**
- The interface checks drive a headless Edge or Chrome over its debugging port. Once on CI (#34's run on 16 October)
  one failed with `The browser did not expose a page to inspect` before checking anything; the next run on `main`
  passed. That line alone is the browser starting slowly, not a failed check.

**CI's database images**
- The QOR ID and Social jobs pulled `postgres` and `redis` from Amazon ECR Public anonymously. It refused twice on 15
  October 2026 (#30, #32), on #35, and five times on #37's run 39005954469 on 17 October (`toomanyrequests: Data limit
  exceeded`), failing a job at "Initialize containers" before any step of the change ran; re-runs passed only some of
  the time. **Since #37 the images come from `mirror.gcr.io/library/`**, Google's mirror of Docker Hub's library, which
  needs no credential; both tags had the same digests there as on ECR. A job that fails at "Initialize containers" with
  a Docker pull error is a registry, not the change; if this mirror refuses too, the next step is an authenticated pull,
  which needs a credential and so is the owner's to choose.

**Launcher checks**
- `npm run check` runs sixteen checks (the one look since 21 October 2026, the ARQADE view since 18 October): design, the one look
  (`check-look.mjs`), accessibility, the gates view, the Projects view, the
  Inventory view, the Market view (`check-market-view.mjs`), QQ (`check-qq-view.mjs`, since 2026-10-08), the Social view, the Library view, the
  Mesh view, the profiles view, the ARQADE view (`check-arqade-view.mjs`), the Gate (`check-vault-gate.mjs`), QFX contrast and readability. It
  needs `npm run build` first, because most of them serve `dist`, and a
  Chromium-family browser (set `BROWSER_PATH` for one that is not Edge or Chrome in the usual places).
  Readability takes about four minutes; `READABILITY_THEMES=numen` runs one theme while iterating.
- **The intro covers the whole window for its first seconds, and takes the pointer.** A check that reads pixels
  or presses on a surface straight after boot reads and presses the splash (2 s; the first-run intro 4.2 s).
  `check-qq-view.mjs` sampled the splash's glow as the orb's colour and its drag selected nothing, on
  2026-10-08. Wait until `document.elementFromPoint` at the target returns the target.
- **Headless Chrome can stop producing frames for a page after a WebGL program is rebuilt**, with the main thread idle
  and the page "visible": every animation loop on the page, not only the backdrop's, drops to one to three frames a
  second. It made `check-accessibility.mjs` fail about two runs in five until 8 October 2026. Bring the page to the
  front again after such a switch (`toFront`), and prove with a planted fault that the check can still fail.
- **Docker Hub limits anonymous pulls, and GitHub's shared runners pull anonymously.** Run 37991419843 (#74, on
  pull request #19, 9 October 2026) failed before any step: QOR ID's job could not pull `postgres:16-alpine`
  ("toomanyrequests"); a re-run of the failed job passed. Pull request #20 moves the two CI service images to the same
  official images on `public.ecr.aws/docker/library/` (and #37 to `mirror.gcr.io/library/`, below). The Dockerfiles (`services/qor-auth`, `chain`) still start from
  Docker Hub images: Railway builds QOR ID's with its own pulls, and `devnet-image.yml`, started by hand, could meet the
  same limit.
- **Qt Quick 3D has a type called `Material`, and the import that comes later wins.** Imported after
  `QtQuick.Controls.Material`, it shadows the style's attached `Material.background` and the window fails to load
  ("Non-existent attached object"). Import `QtQuick3D` first (`products/qq/studio/Main.qml`).
- **Qt 6 keeps a URL property as written.** A scene's `source: "../assets/orb.gltf"` stays relative in the property;
  resolve it against the entity's own context (`SceneIO.resolvedUrl`) before copying or comparing it.
- **Across drives there is no relative path.** `QDir::relativeFilePath` then returns `X:/...`, which QML reads back as a
  URL with scheme `x`; write the whole `file:///` URL instead (`sceneio.cpp`, `urlText`).
- **A still Qt Quick scene draws once and then waits.** A test that waits for frames to be swapped times out; ask for
  frames (`window->update()`) and count them as they come (`products/qq/tests/tst_world.cpp`).
- **A lambda counting `frameSwapped` must not capture a local with the window as its context.** The signal comes from
  the render thread, so each call is queued; one still queued when the function returns arrives later and writes into
  a stack frame that has gone. In `tst_play` it overwrote the slot where `qWait`'s event loop had saved `r13`, and a
  later case crashed destroying a string (10 October 2026). Count through a local `QObject` as the context: destroying
  it drops the calls still queued (`products/qq/tests/tst_play.cpp`, `Stage::open`).
- **A crash in a `QString` destructor inside a test can be another frame's write.** The tools that found it: a
  `RelWithDebInfo` build in its own folder, `cdb` from the Windows SDK with Qt's `.pdb` files, a conditional breakpoint
  after each call to see which returns with a register changed, then a hardware watchpoint (`ba w8`) on the stack slot.
- **Qt for WebAssembly: use the multithreaded kit for QQ.** The single-threaded kit has no Qt Multimedia, so Qt Spatial
  Audio is not there and every QQ scene fails to load ("module QtQuick3D.SpatialAudio is not installed"). The threaded
  page must be cross-origin isolated (ADR-084).
- **Qt's spatial audio cannot sound in a browser.** It makes its sound on a thread of its own, a Web Worker, where the
  web has no audio; the thread throws in `QWasmAudioDevices::createAudioSink` and dies, and after that **no Qt log line
  reaches the console**, so the Player's own report vanished too. A threaded page's errors are on its workers'
  consoles: `measure-web.mjs` attaches to every worker. A function-named build (`--profiling-funcs` at link) gives the
  stack names.
- **`emscripten_get_now()` in a threaded build counts from 1970**, so that every thread shares one clock; a time
  rounded into an `int` from it overflows. For the page's clock, read `performance.now()` on the page's thread
  (`player/main.cpp`, `sinceStart`).
- **Headless Chrome on a laptop with two GPUs draws on the slower one** unless given `--force_high_performance_gpu`:
  12 frames a second on the Intel GPU, 220 on the RTX 4060, for the same build.
- **A macro's arguments are split at commas outside parentheses**, braces and brackets included: in `EM_ASM`, an
  object or array literal with commas must be wrapped in parentheses.
- **Qt 6 hands a `url` property to a URL interceptor as written, relative.** An interceptor that refuses what has no
  scheme refuses every relative path; let relative URLs through and check them whole when loaded. A relative URL given
  to a type inside a module (`RuntimeLoader` in `Prop.qml`) is resolved against the module, not the scene: resolve it
  against the entity's file first (`SceneIO.resolvedUrl`).
- **`Qt.openUrlExternally` takes its URL through the engine's interceptors, then `QDesktopServices`.** A URL an
  interceptor rewrites goes on to the operating system unless its new scheme has a handler: on Windows an unknown
  scheme opens "Pick an app" dialogs, which block a test (10 October 2026: four were left on the desktop). `QDesktopServices`
  has no catch-all handler, only one per scheme.
- **A module whose types are registered is imported without its `qmldir`.** Qt 6 registers a QML module's types when
  its library loads: `Qt6Quick3DSpatialAudio.dll` links `Qt6MultimediaQuick.dll`, so wherever spatial audio is loaded,
  `import QtMultimedia` works even with its `qmldir` refused.
- **`cargo test` does not build an example's executable.** It only checks that the example compiles; the
  executable in `target/debug/examples/` is whatever `cargo build --example` last made. A test that starts an example
  builds it first (`host::tests::fake_product`), or it runs stale code (10 October 2026).
- **A thread's own `tracing` logger misses lines another test first decided nobody wanted.** `tracing` caches each
  log line's interest across threads; a test that captures the log installs one process-wide logger
  (`host::tests::captured_log`), or a planted secret in the log can pass (10 October 2026).
- **"Inability to pay some fees" on a chain with no fees means the account does not exist there.** The node refuses
  a transaction from an account it holds nothing for before any block, and subxt reports it as a transaction "not
  accepted". On a fresh `--dev --tmp` node only Alice's well-known accounts hold anything; fund the account that
  signs, and note that publishing signs with the vault's first account (15 October 2026).
- **A model that fails to load still lets a game settle, and sooner.** `Game.settled` waits for every model to load
  *or fail*, so a missing model made the bundled game start faster and looked like a pass until item 71; `Game.unloaded`
  now names it and the Player's checks fail on it. Look at a first frame, not only its time.
- **Performance measured on battery is not comparable.** At 18% on battery the native Player drew 6 frames a second
  against 155 to 345 on mains (10 October 2026): check `Win32_Battery` before recording a measurement.
- **A pull request based on another's branch merges into that branch, not `main`, if it is merged first or at once.**
  GitHub retargets a stacked pull request only after its base is merged and deleted; #22, merged 16 seconds after #21,
  landed in `session/qq-p3.2-play` (10 October 2026) and needed #23 to reach `main`. Base every pull request on `main`.
- **`QJsonObject` keeps its keys sorted, so it is taken in name order, not the order written.** A check that a refused
  value undoes the values before it must put a good field earlier in name order than the bad one, or it passes with the
  undo removed (`tst_agent`, 10 October 2026).
- **A file held open on Windows cannot be replaced.** `QSaveFile` writes beside it and renames over it, which is
  refused ("Access is denied") while anyone, a test included, still has it open. Close a `QFile` before the next write;
  look in the process before blaming a virus scanner.
- **A Windows GUI program's Qt messages go to the debugger, not to standard error.** A test reading a started Studio's
  output sets `QT_FORCE_STDERR_LOGGING=1` for it.
- **MCP tool names are letters, digits, `_` and `-` only** (Claude's clients enforce `^[a-zA-Z0-9_-]{1,64}$`):
  `scene_read`, not `scene.read`.
- **Qt 6.12's `Qt6Mcp` module has no public API**, only private headers; QQ speaks MCP's JSON-RPC itself
  (`products/qq/agent/mcpserver.cpp`).
- **`Copy-Item` keeps a file's old time, so Ninja does not rebuild it.** Restoring a source after a planted fault that
  way left the faulty build in place, and the next run tested the fault (9 October 2026). Touch restored files.
- **`check-design.mjs` matches `rgb(` anywhere outside the token files, a function name included.** A helper
  called `rgb()` fails "Colours come from tokens". Name it something else; put fixed colours (QQ's starter
  palette, `QQ_PALETTE`) in `src/styles/themes.ts`, which defines tokens. Never exempt a file.
- **`?.` in a check's click turns a missing element into a silent pass.**
  `document.querySelector(sel)?.click()` does nothing when `sel` matches nothing, and the checks after it then
  read an absent dialog and fail with no hint of why — or, worse, read nothing and pass. A Sell block shipped
  that way on 2026-09-23: it opened the menu of an asset the trade block before it had already traded away.
  **In a check, an element that must be there is addressed without `?.`**, so the run throws at the line that
  is wrong. Keep `?.` for reading something that is legitimately optional.
- **A fixed sleep in a view check is a flake waiting for a loaded machine.** `check-vault-gate.mjs` failed one
  run in three under `npm run check` (2026-09-28) and never alone: a click or a field was addressed while the
  previous screen was still drawn. Wait for the target (`until`), and for a headless page, send
  `Page.bringToFront` and `Emulation.setFocusEmulationEnabled` after every reload, or it may draw no frames
  and a framer-motion cross-fade never finishes.
- **A later block runs against the state the earlier ones left.** `check-inventory-view.mjs` mints, makes
  permanent, trades and then sells, all in one page. After the trade only one asset is held, so a block added
  at the end cannot use the fixtures the earlier blocks used. Read what the previous block left before
  choosing an asset to act on.
- **A check file that does not parse reports nothing, and looks like nothing ran.** A duplicate `const` made
  Node refuse `check-inventory-view.mjs` entirely on 2026-09-23; the `GATES.toml` entry beside it had already
  been written with a check count worked out by arithmetic. **A count in `GATES.toml` is evidence and comes
  from a run**, never from adding up the checks you think you wrote.
- **A pipeline's exit code is the last command's, not npm's.** `npm run check | grep RESULT` exited 0 while
  `check-readability.mjs` had **crashed**, because grep succeeded. The crash was invisible: its per-check
  lines were filtered out and its `RESULT` line never came. Read the whole output, or redirect to a file and
  check the exit code of the command you care about.
- **A runtime upgrade beats purging a development chain.** `sudo.sudoUncheckedWeight(system.setCode(wasm))`
  against a `--dev` node keeps every account, asset and block; the node binary need not match, because the
  runtime executes as wasm from state. Check the chain type is `Development` first, and read `spec_version`
  back afterwards. Restarting the node with the same `--base-path` and no upgrade serves the OLD runtime,
  which is also how the previously recorded metadata size was confirmed to the byte.
- **A running node holds its own binary open on Windows.** `cargo build --release` fails with "Access is
  denied" while the development node is running; stop it first. To measure both runtime shapes, copy the
  first binary aside before building the second, because they are the same path.
- **A token check cannot see a painting mistake.** `check-contrast.mjs` does the colour arithmetic and assumes
  the layers are canvas, scrim, interface. On 2026-09-22 they were not — the scrim was painted over most of
  the interface — and every check passed while the launcher was unreadable. `check-readability.mjs` measures
  the painted result. Keep the interface in `.qfx-interface`; anything rendered outside it, beside the canvas
  and the scrim, paints beneath them.
- **A headless screenshot can wait for a frame that never comes.** `Page.captureScreenshot` hung for 60 s
  and killed two full runs of `check-readability.mjs` on the first theme, after the check grew three more
  screens. It is the test browser, not the app: `shoot()` now brings the page to front, restores focus
  emulation and asks once more, and the run says how often it had to. A second stall still fails the run.
- **Headless Edge needs `--disable-backgrounding-occluded-windows`, `--disable-renderer-backgrounding` and
  `--disable-background-timer-throttling` for any long run.** Without them the page can be marked hidden,
  stop producing frames, and a screenshot waits for ever; `check-readability.mjs` stalled that way once.
- **A class name that names nothing fails silently.** The Projects surface shipped with `button`,
  `button-quiet` and `input`, none of which exists in `qor.css`, and rendered browser defaults while every
  check passed. The stylesheet's classes are `btn btn-primary`, `btn btn-ghost` and `field`. The Projects
  and Inventory checks now fail on any class the stylesheet does not define (lucide's icon markers
  excepted).
- **`check-accessibility.mjs`'s case "stored off, then live: the backdrop starts" used to fail about half
  the time**, with 200 to 800 ms frames after that boot. Since the interface was lifted above the canvas and
  the scrim (`1f5f4ed`) it has passed 7 runs of 7. The likely cause was the interface being repainted beneath
  every backdrop frame; that is inferred, not measured. If it returns, **do not lower the threshold**; that
  is a loosening.
- **The two live tests share Alice.** They run in parallel, and two fundings that read her next nonce at
  once collide ("priority is too low"). `fund_from_alice` takes a lock; a new live test that funds must use
  it.
- **The repository ignores every `package-lock.json` except the launcher's**, which is tracked since
  2026-09-29 because CI installs it with `npm ci` and all of its dependencies are ranges. Woodpecker's
  second run failed at `npm ci` for want of it.
  `chain/scripts/package.json` pins its two direct dependencies exactly and is installed with `npm install`,
  not `npm ci`.

**Chain**
- **A `sp_api::decl_runtime_apis!` trait's runtime-side module is snake-cased with digits split:**
  `Drc369Api`'s is `runtime_decl_for_drc_369_api`, and its trait is `Drc369ApiV1`.
- **A mint on a development chain needs about 1,100 CGT free** at the placeholder deposits: 500 for the
  singles collection, 400 for the asset, 100 plus 0.625 a byte for the name, and the existential deposit.
  `chain/scripts/dev-fund.mjs` sends 10,000 by default.
- **Only `pallet-drc369` can create an asset.** Of `pallet-nfts`'s calls the base call filter lets through
  only `transfer`, `approve_transfer`, `cancel_approval` and `clear_all_transfer_approvals`, and its
  `CreateOrigin` refuses everyone. A test that dispatches any other `Nfts` call gets `CallFiltered`.
- **CI runs `npm run check` itself, not a list of scripts** (since `7a3c8ea`). It used to name three, and
  two checks added later were never in the workflow. Add a check to `package.json` and CI has it.
- **The QFX cases in `check-accessibility.mjs` count frame callbacks.** "Still" means no callback is
  scheduled, which a screenshot cannot show. A positive control proves the backdrop is drawing first;
  without it the still cases would pass on a dead canvas. Headless Edge gets a WebGL2 context even under
  `--disable-gpu`.

**Products (P1 to P6)**
- **Build Qontrol's helper before its tests:** `cargo build --manifest-path tools/qor-launcher/qontrol-git/Cargo.toml`.
  Without it the tests that commit skip and say so; with `--features qontrol-no-skips`, as CI and the
  `qontrol` gate's suite run them, they fail. A skipped test reports ok, which is why the feature exists.
- **The helper is found through `QONTROL_GIT_BIN`, beside the launcher executable, or in
  `qontrol-git/target/`.** Since P1.2 the installer bundles it (`src-tauri/tauri.bundle.conf.json`,
  `externalBin` `binaries/qontrol-git`).
- **Write "Accepted" plainly at the start of an ADR index's Status cell.** The gates parser does not strip
  Markdown, so a cell reading `**Accepted**` counts as not accepted. ADR-045's row is written that way; no gate
  counts it today. Stripping emphasis in the parser would widen what counts as met, which is loosening.
- **A product item is `P<n>.<m>` under `#### P<n>:`** in `DIRECTION.md`. The launcher's parser has read
  `P` headings only since `d4be885`; before it, every product item would have read as unmeasurable.
- **Product gates may count chain items; chain gates never count product items.** They are appended after
  Public Release, because a `gate` criterion resolves against earlier gates only.
- **`check-design.mjs` has no effect rules and no exemptions since ADR-080** (6 October 2026). Glow, canvas,
  pointer-reactive light and looping animation are allowed on every surface; the check enforces theme colour
  tokens, the type scale and the tracking scale only. Reduce motion and readability as painted still bind, through
  `check-accessibility.mjs` and `check-readability.mjs`.
- **Ambience Off hides the canvas and the scrim in one rule.** Hiding only the scrim leaves a frame on
  screen with nothing over it, which was layer one's first defect.

**The chain (`chain/`, M3)**
- `chain/` is the only chain (ADR-013, ADR-032). `framework/` was retired and deleted at M3.5.
- **The pin is `polkadot-sdk = "=2606.1.0"`**, exact, for `polkadot-stable2606-1` (ADR-022). Resolving the
  workspace on 2026-09-17 locked 1,559 packages and reported that **2606.2.0, 2606.3.0 and 2606.4.0 now
  exist**. **The owner's decision on 2026-09-17: stay on 2606.1.0**, noted and deferred rather than
  stale. Moving is its own task with its own verification pass under ADR-033 rule 3, and mid-M3 is the
  wrong moment; ADR-022 already declined 2606-2 once. **A move is proposed once the node and
  `pallet-validator-set` are in and the chain produces blocks under a governance-chosen validator set,
  not before.** Also recorded in `chain/README.md`, where someone building the chain will see it.
- **`SKIP_WASM_BUILD=1` gives a fast compile of the Rust**, without building the runtime's wasm. Use it
  while iterating; never use it to claim the runtime builds. A full release build takes about four minutes.
- **The wasm target is `wasm32v1-none`**, installed on 2026-09-17. The SDK builds with
  `wasm32-unknown-unknown` if it is the only one present, and warns that the newer target is preferred;
  switching targets changes the runtime blob, so it was settled before anything depended on the blob.
  Changing it again means `cargo clean` first, as the SDK's own warning says.
- **No transaction payment, no issuance, no genesis allocation.** Fee classes and burn shares are OPEN-4,
  the issuance rate OPEN-1, the genesis split OPEN-2. A `WeightToFee` is exactly the kind of value
  AGENTS.md §5 forbids inventing, so the runtime has no fee pallet yet. The custom chain charges no fee
  either, so the devnet loses nothing.
- **`pallet-sudo` is behind the `sudo` feature** (ADR-037), on for development and test networks and off
  for mainnet, so removal is a build configuration and not a code edit under pressure.

**R-1 on the new chain: closed as met by standard behaviour (ADR-038)**
- Writing the acceptance tests measured it. The forgery the custom chain accepted (R = the identity
  point, s = 0) is **refused by Sr25519 on 64 of 64 messages** for both small-order keys, and **accepted
  by Ed25519 on 64 of 64**.
- **It is not the custom chain's defect repeated.** `sp-core` verifies Ed25519 with `ed25519-zebra`,
  which implements ZIP-215: a precisely specified rule chosen so that every node agrees on validity.
  Consensus needs that agreement more than strictness. Changing it means departing from the SDK's
  standard verification, which ADR-013 does not allow without a written reason.
- **What it costs:** an account whose address is a small-order Ed25519 point can be spent from by anyone.
  Nobody holds a secret for such an address, so nothing is taken from a person; funds sent there are lost
  to whoever claims them, as with any unowned address.
- **ADR-023 chose Sr25519 for account keys**, so the scheme this chain actually uses is not exposed.
- **Decided on 2026-09-18 (ADR-038): the runtime is not restricted to Sr25519 signatures.** Doing so
  would own a consensus-critical divergence permanently, in exchange for closing a hole that harms
  nobody, and ADR-001 names consensus and key handling as where not to be clever. R-1 is closed as met by
  standard behaviour, not left half-met.
- **ADR-023 therefore stays a convention, not an enforced rule.** Account keys are Sr25519 by decision and
  by what the launcher generates, not because the runtime refuses anything else.
- Three tests pin all of this, including one that **fails if the SDK ever becomes stricter**, so the day
  that changes is the day someone revisits R-1 rather than never noticing.

**Two validators, by script rather than by cargo test**
- `chain/scripts/check-two-validators.mjs` starts two real nodes on the `local` specification and checks
  13 things: they find each other, both author, they hold the same block at a common height, GRANDPA
  finalises, they agree on what is finalised, and a killed validator restarts, catches up and agrees
  about the blocks made while it was away. Run on 2026-09-18, 13 of 13.
- **It needs a release build first** (`cargo build -p demiurge-node --release --features sudo`) and takes
  about two minutes, which is why it is not in `cargo test`. Criterion `alpha.multi-validator` reads it.
- **The owner accepted that run as evidence on 2026-09-18**, so `alpha.multi-validator` is met, and it was
  re-run on 2026-09-20 after ADR-041 changed the address type. **It is a CI job of its own since
  2026-09-20** (`two-validators`, `[ci].quality_gates`), which fails the run, so nothing sits unwatched
  between two manual runs again. It is a separate job because it needs the release build and the quality
  gates do not. It runs on the nightly schedule and by hand (`ci.yml`), and carries `timeout-minutes: 90`
  rather than a guessed number.
- **A validator will not generate its own network key**, so both nodes are given `--node-key`. Without
  one a validator exits with `NetworkKeyNotFound`.
- **The first validator runs with `--force-authoring`, deliberately.** A node with no peers stops
  authoring, which is Substrate refusing to build a one-sided chain while it might be partitioned. That
  is correct behaviour and it made an earlier version of this check fail when the second validator was
  killed. Forcing it is what lets the chain advance while the other node is away, which is the only way
  catch-up can be tested.
- **Finality stops while one of two validators is down**, because GRANDPA needs more than two thirds of
  the voters. The chain keeps producing blocks and resumes finalising when the other returns. That is
  safety, not a fault.

**pallet-validator-set (ADR-020)**
- **A governance action must not be able to stop the chain.** The validator set is what authors blocks, and
  a halted chain needs a new genesis rather than a governance fix. Three redundant guards: the
  configuration (`MinValidators >= 1`, refused at startup), every call (nothing may take the set below the
  minimum), and the session boundary (`new_session` answers `None` rather than an empty set, keeping the
  previous authors and emitting `PreviousSetKept`).
- **If `PreviousSetKept` is ever emitted, treat it as an incident.** It means a guard above the session
  boundary failed and only the failsafe stood between the chain and a halt.
- **Each guard was verified by deleting it**, not by reading it: removing the session-boundary check fails
  three tests, removing the per-call bound fails four. Doing that found a real gap — the sequence test
  never reached the removal of the *last* validator — which is now covered and commented so it is not
  reintroduced.
- **An empty genesis is allowed**, following `pallet-collator-selection`, which ADR-020 says to mirror. It
  yields a chain with no authorities that never authors block one, which is immediate and unmissable; the
  danger guarded here is a *running* chain governed into a halt.
- **On the mainnet shape there is no path to the governance origin yet.** It is `EnsureRoot`, and root has
  no caller without `pallet-sudo`. That is correct rather than an oversight: ADR-021's collective is not
  built. A mainnet runtime therefore cannot change its validator set, and that is recorded rather than
  papered over with a weaker origin.

**A `cargo tree -i` query without an exact version answers about one copy and reads like both**

- **This is how a wrong claim survived in four files.** `SECURITY.md`, `HANDOFF.md`, `GATES.toml` and
  `ci.yml` all said none of `chain/`'s ten advisories was reachable through a normal dependency edge.
  Four of them are. The query behind the claim was `cargo tree -i -e normal <crate>`, run **per crate**
  and not **per crate version**.
- **A lockfile resolves several versions of the same crate at once.** `chain/Cargo.lock` holds two
  `hickory-proto` (0.24.4 and 0.25.2) and two `tracing-subscriber` (0.2.25 and 0.3.19). An advisory names
  one of them. `cargo tree -i` given a bare name answers about whichever it picks, and prints no warning
  that the other exists, so an answer about the unreachable copy reads as though it covered the compiled
  one too.
- **Query by exact version, always:** `cargo tree --workspace -i <crate>@<version> --target all`.
  `--target all` covers platform-gated edges, and the default edge kinds already include build and dev,
  so a dev-dependency counts as compiled — which is what you want when asking whether something is built.
- **`-e normal` was the second half of the mistake.** It excludes build and dev edges, so a crate that is
  genuinely compiled can answer "not reachable". Use it to ask a narrower question deliberately, never to
  ask "is this compiled".
- **Confirm a negative before trusting it.** A crate the query cannot reach is in the lockfile because a
  lockfile resolves the union of every optional dependency in the graph. Repeat the query with
  `--all-features`; if it is still unreachable, no feature this workspace can enable selects it.
- **`cargo audit` lists the version.** Read it from there rather than from the crate name in the advisory
  title, which is where the two versions blur together.

**Whenever scope moves between milestones, check gate coverage**
- Twice now, work moved out of an item has landed in a milestone whose gate listed only the items that
  existed when that gate was written, so the moved work would have been counted by **no gate at all**:
  M6.4 and M6.5 when M3.1 was narrowed, and M4.5, M5.6 and M6.6 when M3.3 was. A scope decision must not
  become a gating decision by omission.
- The check, run from the repository root:

  ```bash
  python - <<'PY'
  import tomllib, re, io
  d = tomllib.load(open('docs/GATES.toml','rb'))
  counted = set()
  for g in d['gates']:
      for c in g.get('criteria', []):
          if c.get('kind') == 'roadmap':
              counted.update(c.get('items', []))
  text = io.open('docs/DIRECTION.md', encoding='utf-8').read()
  items, cur = set(), None
  for line in text.splitlines():
      # Reset on EVERY heading. Without this, a heading that is not a milestone
      # -- '### 7.1 Security track' -- leaves `cur` pointing at the last
      # milestone, and that section's items are reported as phantom M1.1..M1.7.
      if line.startswith('#'):
          m = re.match(r'^#{3,4} (M\d+|L\d+|P\d+)', line)
          cur = m.group(1) if m else None
          continue
      m2 = re.match(r'^(\d+)\. \[[ x]\]', line)
      if m2 and cur:
          items.add(f'{cur}.{m2.group(1)}')
  print(sorted(i for i in items if i not in counted))
  PY
  ```

- **M1.1 to M1.7 do not exist, and a version of this note used to say they were expected.** M1 is
  prose with no checkbox items. The phantom entries came from the bug fixed in the script above: it
  never reset `cur` on a heading that is not a milestone, so `### 7.1 Security track` left it pointing
  at `M1` and that section's seven items were reported as `M1.1`…`M1.7`. With the reset in place the
  only uncounted item is **`L2.1`**, which is deliberate and explained on the criterion. Anything else
  appearing is a real gap.

**Keys and addresses (ADR-023, ADR-024, ADR-039)**
- **One implementation, not two.** Both the launcher and QOR ID use `sp-core` itself, pinned to
  `=43.0.0`, the version inside `polkadot-stable2606-1`. Do not reimplement the derivation, the SS58
  encoding or the verification anywhere; ADR-033 rule 1 is what puts them on the SDK's version.
- **`full_crypto` is what makes a key able to sign.** The launcher enables it. QOR ID does not, and its
  tests do, through a dev-dependency, so the service binary cannot sign anything.
- **A junction built from a value is not the junction a person types.** `DeriveJunction::hard("0")`
  encodes the *string* `"0"` and is a different account from `//0`. Build junctions from the path string;
  a test in `vault/derive.rs` asserts the two differ, so this cannot be reintroduced quietly.
- **Account `n` is `//(n-1)`, not `//n`.** The first account is the bare phrase. That offset is Talisman's
  enumeration and is deliberate (ADR-039).
- **`pubkey` is gone from every QOR ID request.** An account is named by `address` (SS58 at the chain's
  prefix) or by `account_id` (`0x` hex), exactly one of the two. Hex in `address` is refused on purpose.
- **A prefix check is not chain identification.** Many chains use 42. Do not describe it, or rely on it,
  as evidence of which chain an address came from (ADR-024, F-Q9).
- **The challenge still names no network** (ADR-039 point 7). The reason in the code, beside the domain tag in
  `src/handlers/auth.rs`, is that no network was deployed. Demiurge Devnet has been live since October (ADR-068),
  so binding the challenge to a genesis hash is now an open item, not a wait.
- **A node and a launcher from opposite sides of 2026-09-20 refuse each other.** ADR-041 changed the
  extrinsic's address from a bare `AccountId32` to a `MultiAddress`, so a node built before that commit
  refuses what a launcher built after it signs, and the reverse. Rebuild both. Only the devnet is deployed,
  and it holds only test CGT. The failure is loud: the client cannot even encode the call.
- **The launcher talks to `chain/` and nothing else** (ADR-040). Its default is Demiurge Devnet,
  `wss://rpc.qorsync.dev`; for a local node, point it at `ws://127.0.0.1:9944` (`LOCAL_RPC`) with
  `chain/target/release/demiurge-node --dev`.
- **The launcher's chain endpoint is a WebSocket address now.** `ws://` or `wss://`, because the client's
  transport is WebSocket and nothing else. A stored `http://` endpoint is upgraded on load rather than
  refused, since it names the same node on the same port. QOR ID's endpoint is unchanged and is still
  `http(s)://`: the two are not the same kind of address.
- **A transfer is reported only once GRANDPA has finalised it,** and a timeout waiting for that is
  reported as a timeout naming the transaction hash, never as a failure. A transaction that has been
  submitted may still finalise, and must not be sent twice.
- **Take the nonce from `system_accountNextIndex`, not from `subxt`'s default.** `at_current_block()`
  pins the latest *finalised* block, so a nonce read there is stale the moment a transaction is in a
  block but not yet finalised, and the second of two transfers is refused as "Transaction is outdated".
  This was observed, then fixed; the live test sends two in a row so it cannot come back.
- **A database that ran the old schema needs migration 018**, which drops `users.on_chain_address`,
  `users.primary_pubkey` and `auth_challenges.pubkey`. A build database from before it will not compile
  the crate.

**Working rules**
- Do not extend or patch the custom chain (ADR-013). Record defects as migration requirements.
- Do not invent open values (`docs/economics/OPEN_QUESTIONS.md`).
- "Session keys" means validator keys in Substrate. What QOR ID issues to agents is an **agent
  authorisation** (ADR-014).
- In `GATES.toml`, never write a URL for a service that has not actually been deployed.

**qor-auth**
- **Compiling needs a migrated Postgres,** because `sqlx` checks its `query!` macros at compile time. Create a
  throwaway database, apply `migrations/*.sql` to it with `psql` for the compile schema, and point
  `DATABASE_URL` at it. Run the service itself against a separate empty database, and let it apply its
  own migrations.
- **The flat `DATABASE_URL` wins over `QOR_AUTH__DATABASE__URL`** at runtime (`src/config.rs`, about line 275).
  Unset it before starting the service, or it silently uses the build database.
- **The `vyb-redis` container on 6379 requires a password.** Use a separate Redis for `qor-auth` tests.
- **Registration shapes changed.** `POST /agents/register` and `POST /profile/link-wallet` take `pubkey`,
  `challenge` (from `GET /auth/challenge?pubkey=`) and `signature`. `link-keypair` needs a bearer token.
- **`POST /auth/keypair-register` signs the new account in** (2026-09-14). It returns `access_token`,
  `refresh_token`, `expires_in` and `token_type` alongside the account fields, and accepts an optional
  `device_id`, as `keypair-login` does. Before this it returned no tokens, so claiming a QOR ID from the
  launcher created the account and then failed with "did not return a usable token pair".
- **Challenge signatures carry a domain tag.** Every client signs `demiurge:qor-id:challenge:v1:` followed
  by the challenge. A signature over the bare challenge gets 401. The tag is defined twice, in
  `services/qor-auth/src/handlers/auth.rs` and `tools/qor-launcher/src-tauri/src/identity/mod.rs`, and
  each side's test pins it.
- **Migration 008 changed.** A database that ran the old version needs its sqlx checksum updated once
  (see the 008 header).
- Production configuration takes secrets only from the nested `QOR_AUTH__*` environment variables.
- **The service will not start on the flat variables alone.** `QOR_AUTH__JWT__ACCESS_SECRET`,
  `QOR_AUTH__JWT__REFRESH_SECRET`, `QOR_AUTH__DATABASE__URL` and `QOR_AUTH__REDIS__URL` must be set, or
  it exits with "missing field". The flat forms then override them.
- **Every access token needs its Redis session** (2026-09-14). Logout ends the token at once; with Redis
  down, authenticated routes answer 500.
- **Nested root routes have no trailing slash.** `/api/v1/profile` works; `/api/v1/profile/` is a 404
  that never reaches the auth middleware, so it proves nothing about authentication.
- **`cargo clippy` and `cargo test` do not rebuild `target/debug/qor-auth.exe`.** Run `cargo build`
  before an end-to-end run, or the checks exercise the previous build. It happened on 2026-09-14.
- **End-to-end scripts** are in `services/qor-auth/scripts/e2e/`. Each takes the base URL and needs a
  running service against Postgres and Redis:
  - `account-and-admin.mjs`, `sign-in-enumeration.mjs`, `sessions.mjs` and `registration-mints-nothing.mjs`
    expect email not configured;
  - `logout.mjs` stops Redis for one check only when `REDIS_CONTAINER` is set;
  - `email-via-resend.mjs` runs a stand-in for Resend on port 59925, so start the service with
    `RESEND_API_KEY`, `EMAIL_FROM` and `RESEND_API_URL=http://127.0.0.1:59925`.
- **Registration calls no chain** (R-3, 2026-09-14). `BLOCKCHAIN_RPC_URL` is no longer read, and the
  response has no `starter_cgt_minted` field.
- **The tests need a Postgres they can create databases on.** Handler tests use `#[sqlx::test]`, which
  creates a database per test from `DATABASE_URL`; in CI that user is the container superuser. Migrations
  012 to 014 are applied to the compile database like the others. Tests that need Redis are `#[ignore]`
  locally: run everything with `QOR_AUTH_TEST_REDIS_URL=redis://... cargo test -- --include-ignored`, as CI
  now does.
- **A password reset needs Redis.** Sessions are revoked before the reset commits, so with Redis down
  nothing is reset.
- **`account-and-admin.mjs`** promotes an account to god and plants a reset token with `docker exec psql`.
  It needs `PG_CONTAINER` and `PG_DATABASE`, and reports SKIP without them.
- **Admin tokens issued before the role fix carry `God`** and are refused. Sign in again.
- **Email is Resend, configured only by environment:** `RESEND_API_KEY` and `EMAIL_FROM`, with `BASE_URL`
  for the link targets. Without both, `forgot-password` answers 503 and registration sends no verification
  email. `RESEND_API_URL` may name another API base, but only HTTPS or local HTTP. No email content is logged.
  `config/production/*` and `scripts/setup-email.*` still describe SMTP and do not work with the service.
- **No address at `demiurge.cloud` receives mail.**
  - The domain has no MX record, and receiving is disabled in Resend.
  - `noreply@demiurge.cloud` and `Godmode@Demiurge.Cloud` are sender identities only. A reply to either
    goes nowhere, and mail sent to either bounces.
  - Never use either address as a test inbox, a contact address or a `Reply-To`.
  - Whether the domain receives mail is a separate decision for the owner. It is not to be set up as a
    side effect of other work.
- **Opening a link page changes nothing.**
  - `GET /verify-email` and `GET /reset-password` only read. The token is spent by the form their button
    submits, `POST /verify-email` or `POST /reset-password`, because mail scanners open links on their own.
  - A test or tool that expects a GET to verify is wrong.
  - A used or expired link answers 410, with a form that posts to `/resend-verification` or
    `/forgot-password`.
  - These page routes sit at the root, beside the JSON API under `/api/v1/auth`, and share its handlers.
- **Request log spans record the path, never the query**, because the link pages carry tokens in the query.
  Do not put `uri` back into the span.
- **`POST /api/v1/webhooks/resend` needs `RESEND_WEBHOOK_SECRET`** (`whsec_…`). Without it, 503.
  - A missing, wrong or more-than-five-minutes-old signature is 401.
  - Only a permanent bounce, `email.suppressed` or a complaint marks an address.
  - Marked addresses are kept as SHA-256 hashes in `email_suppressions` (migration 017). Every send checks
    them, through `AppState::new`, and a marked address is refused with 422 `UNDELIVERABLE`.
- **Unmarking an address is admin-only:** `POST /api/v1/admin/email-suppressions/unmark` with
  `{"email", "reason"}`.
  - The address goes in the body, never the path, because paths are logged.
  - A god account cannot unmark an address that is its own, or pending on its own account. No account
    holder can clear their own mark.
  - It is audited under the acting administrator, by hash, never by address.
  - Resend's own suppression list is separate. An address Resend suppressed must also be removed there.
  - Do not delete rows by hand.
- **Nothing secret may reach a log, at any level** (GATES `alpha.no-secrets-in-logs`).
  - `log_hygiene::nothing_secret_reaches_a_log_at_any_level` drives every such flow through
    `crate::router` with trace-level capture. It needs Redis, so it runs with `--include-ignored`. A new
    flow that handles a token, key, password or address belongs in it.
  - CI's security job also fails when a log call interpolates such a value by name, or a request span
    records the URI. Name such values plainly (`token`, `email`, …) rather than hiding them in a
    differently named variable; the scan reads names.
- **The service is built by `router()` in `main.rs`**, not inside `main`, so tests drive exactly what is
  served.
- **A query built from values fails the build** (`sql_hygiene`, GATES `alpha.parameterised-sql`).
  - sqlx logs full statement text at debug level, so every value is bound. `sqlx::query*` and `raw_sql`
    take SQL fixed at compile time: a string literal, `include_str!`, an upper-case constant, or a loop
    variable over one.
  - Import them only through the `sqlx::` prefix, since an unprefixed import fails the check.
  - Use a `QueryBuilder` with `push_bind`, never `push(format!(…))`.
  - Test SQL with literal values still shows up in the log check's process-wide capture. That is why the
    log check matches only markers unique to itself (`logcheck.invalid`).
- **Review requirement (AGENTS.md §9).** A new flow handling a token, key, secret, password, backup code,
  link or address joins `log_hygiene` in the same change, whatever its values are named.
- **Registration returns `backup_codes`**, ten of them, for an account without an email, instead of one
  `backup_code`. `reset-password-backup` answers with `backup_codes_remaining`.
- **`POST /api/v1/auth/resend-verification`** takes `{"identifier"}` and issues a new verification token,
  replacing the old one. It sends at most once every 5 minutes and 5 times in 24 hours per account
  (migration 015), and registration's message counts as the first.
- **`POST /api/v1/profile/backup-codes`** needs a bearer token and `{"password"}`. It works for an account
  without an email address, or one that added an address while it held codes. It replaces every old code.
- **`POST /api/v1/profile/email`** needs a bearer token and `{"email", "password"}`.
  - It stores the address as pending (migration 016) and emails a link to `/verify-email`. The existing
    `verify-email` route confirms it.
  - The account's current address, if any, gets a notice without a link.
  - It shares the verification-link limits, so a second request within 5 minutes is 429.
  - A password reset clears the pending change.
  - A keypair account cannot use it: its password is random and unknown.
- **Every sign-in refusal is 401 "Invalid credentials"**, including locked and banned accounts. Clients
  cannot tell them apart, by design.
- **The password minimum is now enforced** from `security.password_min_length`: 12 by default, 8 in
  `services/qor-auth/config/production.toml`.
- **`server.host` must be an IP address**, and is now honoured.
- **The launcher's lockfile held a vulnerable `rustls` for five days** (0.23.44, RUSTSEC-2026-0285),
  although `SECURITY.md` said both lockfiles had been upgraded. Only QOR ID's had been. Found on
  2026-09-19 by running `cargo audit` rather than by reading the claim, upgraded to 0.23.45 the same day,
  and corrected in `SECURITY.md`. **Run the audit in both projects when either lockfile changes**; it
  takes a minute, and a written claim is not a check.
- **`cargo audit` configuration lives in each project's `.cargo/audit.toml`, and there is now exactly one:**
  `services/qor-auth`, whose ignore holds only while `rsa` stays out of the compiled graph, which CI
  enforces. The `framework/` exemption expired with the directory at M3.5. The launcher has none, and
  `chain/` has none, deliberately — an exemption file is a place for things to go quiet.
- **`cargo audit` in `chain/` reports 10 vulnerabilities. Measured one by one on 2026-09-20: four are
  compiled, six are not.** Every one is transitive through the pinned SDK, and every one of the four is
  held by a version the SDK dictates, so ADR-033 rule 1 does not override it. Each advisory is named,
  with what blocks it and what would close it, in `SECURITY.md`, "Dependency advisories in `chain/`".
  **Correction:** this file previously said none of the ten was reachable through a normal dependency
  edge. Four of them are: `tracing-subscriber` 0.3.19, through `sc-tracing` and `sp-tracing`, and
  `hickory-proto` 0.24.4 and 0.25.2, through libp2p's DNS transport, mDNS and litep2p. The old claim came
  from a `cargo tree -i -e normal` query not made **per crate version**, and two versions each of
  `hickory-proto` and `tracing-subscriber` are in the lockfile, so an answer about the unreachable copy
  read as though it covered both.
  - **Query per exact version**, or the answer is wrong the same way:
    `cargo tree --workspace -i <crate>@<version> --target all`. Default edge kinds already include build
    and dev, so a dev-dependency counts as compiled.
  - **`tracing-subscriber` does not move by moving the pin.** `sc-tracing` 47.0.0 is the newest published
    and every release from 45.0.0 requires `=0.3.19`. It closes upstream or not at all.
  - **The two `hickory` advisories might.** `libp2p-mdns` admits the fixed `hickory-proto` at 0.49.0.
    Whether a later `polkadot-stable2606` patch carries a new enough libp2p or litep2p is **unmeasured**:
    the exact pin means even a dry run needs the manifest edited, and editing it is the move. That
    measurement is the first step of the pin-move task (ADR-033 rule 3), which is a reason to propose one.
  - **RUSTSEC-2026-0118 has no fixed release anywhere**, so it is carried regardless.
  - **The audit now runs in CI as a report that does not fail the run**, registered in `GATES.toml` as
    `[ci].reports`. No criterion reads it and nothing is exempted: `chain/` still has no `audit.toml`, and
    the answer is still not one. Making it a gate is a decision for when those four can close.

**Chain and launcher**
- ~~**The launcher still mirrors a 13,000,000,000 supply.**~~ **Gone on 2026-09-20**, which is the
  removal ADR-003 asks for by name. `cgt.rs` now declares `MAX_AMOUNT_SPARKS`, a ceiling on what a person
  can type, and **it is not a supply**: ADR-004's perpetual issuance means no fixed total exists, and
  `chain/` declares no such constant for the launcher to mirror. The value is ADR-003's base supply
  because a guard needs a number and that one is decided; a test pins it to that record and pins the
  refusal a person reads to not calling it a total supply. The "Total supply" row left the Settings
  surface with it. **The trap it leaves:** do not put a supply figure back into `TokenInfo`. A panel of
  facts about the connected node should hold only what can be pinned to that node.
- **An account has one nonce**, read from `system_accountNextIndex` (ADR-040). The custom devnet's
  separate request nonce and transaction nonce (D-004) went with it.
- **There is no faucet and no mint.** The chain has no issuance mechanism, and will not until OPEN-1 and
  OPEN-2 are decided. Fund a development account by transferring from one endowed in the chain
  specification.
- After moving the repository, run `cargo clean -p tauri` in `tools/qor-launcher/src-tauri`.
- Zustand 5 selectors must return cached values.
- A `cd` inside a shell command persists between commands here. Use absolute paths.

**Operational files**
- **Four Substrate-fork scripts were deleted on 2026-09-18**, by the owner's decision:
  `apply-substrate-fix.ps1`, `setup-substrate-from-zip.ps1`, `test-blockchain-build.ps1` and
  `complete-substrate-fix.ps1`. The first inserted a `[patch.crates-io]` section pointing `sc-cli` at a
  personal Substrate fork (`ALaustrup/substrate.git`), which contradicts ADR-022's pinned release and
  ADR-033 directly: "The SDK's transitive pins are not overridden, with `[patch]` or otherwise." They also
  targeted `blockchain\Cargo.toml`, a path from the pre-realignment layout that no longer exists. The
  owner's rule: nothing that repoints the dependency graph at a personal fork stays in the repository.
- **Four more were deleted the same day**, found by scanning for the hazard rather than the filename:
  `fork-substrate.ps1` and `verify-and-setup-fork.ps1` cloned the fork, `apply-substrate-fix-in-fork.ps1`
  edited `sc-cli` inside it, and `test-node-build-versions.sh` recommended a `[patch.crates-io]`
  override. Seven in total.
- **A standing check now enforces it**, because deleting by hand is a fix and a check is a control. CI
  fails if any tracked manifest holds a `[patch]` section, or if anything tracked references the fork.
  Criterion `alpha.no-dependency-patching`, recorded in `[ci].quality_gates`. It was proven to fail
  before it was trusted: a planted patch section and a planted fork clone were both caught.
  - **Two files are excluded, deliberately:** `.github/workflows/ci.yml`, which has to contain the
    strings in order to search for them, and this file, which records the rule. Widening that list shows
    up in a diff.
  - The `[patch]` half scans manifests only, because a `[patch]` section is a hazard in a `Cargo.toml`
    and prose in an ADR. The fork half scans everything, so no document can quietly reintroduce it.
- About a hundred other pre-realignment files under `scripts/` do not touch dependencies. They are a
  separate, unurgent question.
- `config/`, `testnet/`, `scripts/*.sh` and the root deploy scripts do not work against the current node.
- **Five of them reference a directory that does not exist**, since M3.5 deleted `framework/`:
  `Dockerfile`, `docker/Dockerfile.node`, `config/production/demiurge-node.service`,
  `testnet/systemd/*.service` and `fly.toml` (a sixth, `docker/docker-compose.testnet.yml`, is gone from the
  tree). They were already non-working, so nothing regressed; they were deliberately left out of the deletion
  commit, which was scoped to `framework/` alone. The plan to rewrite `fly.toml` for `chain/` under ADR-015 is
  moot: the devnet runs on Railway (ADR-068, `chain/DEPLOY-RAILWAY.md`). They are a cleanup of their own.

- **A QQ logic file imports only `QQ`** (19 October 2026), so `Component.onCompleted` does not exist in it ("Non-existent
  attached object"): a game sets itself up in `onStarted`, which `Logic` sends once before its first step. In
  `Logic.qml`, which imports only QtQml, a signal parameter typed `vector3d` makes every logic file fail to build
  ("Invalid signal parameter type"), so `contact`'s point and normal are `var`.
- **A View3D is not its scene root's object parent** in Qt 6.12, so walking up from a scene never finds the view a ray
  must be cast through. `World.qml` names it on the node scenes go under (`qqView`), and `SceneIO::viewOf` reads that.
- **Qt Quick 3D Physics reports a contact as two bodies meet**, not while they rest together (a crate landing and
  settling was heard at most three times), and only when the other body sends reports: Ground's slab and the Player's
  controller send them, and a Trigger hears only bodies that send trigger reports.
- **QQ's scenes are compared byte for byte, and SceneIO writes LF** (19 October 2026). With `core.autocrlf=true`, this
  PC's system setting, a checkout gave the fixtures CRLF, and `tst_scene`'s round trip failed on a fresh clone of `main`.
  `.gitattributes` pins `products/qq/tests/fixtures/**/*.qml` and `products/qq/samples/**/*.qml` to LF (#55).
- **A screen part is made before its scene is placed in its world** (19 October 2026), so a binding that finds the
  World through the scene must also depend on the scene's own parent, or it never finds it. And a part's frame is
  set up before its fade is turned on, or a hidden screen's parts fade out of view at the start.
- **A frame grabbed from a window is in device pixels**, and this PC's display is scaled: scale a point from an
  item's place by the frame's `devicePixelRatio()` before reading the pixel there.
- **The core's `Cargo.lock` is what the Android build resolves** (Corrosion runs it `LOCKED`). When the core gains
  dependencies, copy the Windows launcher's lockfile into it and let cargo prune it, so both build the same versions;
  CI audits both.
- **Qt's spatial sounds do not say when they have finished** (19 October 2026): `SpatialSound` and `AmbientSound` have
  no `playing` or `finished`. A one-shot is put away after its file's length, read from a WAV header (`Mixer.lengthOf`).
- **A QML `Timer` has no `parent`** to name the object it serves; give it a property of its own (`Voices.qml`).
- **A `Node` made inside a scene is one of its entities** to everything that walks the scene, so a scene's helper that
  is not an entity (its one-shots) is a `QtObject`, and a placed sound is in the scene only while it sounds.
- **`Q_ARG` with a brace-initialised map** (`QVariantMap{{a, b}}`) is split at its comma by the macro and does not
  compile ("'{': no matching token found"): build the map in a variable first.
- **A browser's Web Audio is on the page's main thread only**, and Qt's spatial audio makes its sound on a worker, so in
  a browser QQ sounds through `WebVoice`'s own Web Audio (`EM_JS`). A page makes no sound until a person touches it or
  presses a key; headless Chrome needs `--autoplay-policy=no-user-gesture-required` to play without one.
- **`build.ps1 -Web` run from Git Bash** did not take the Emscripten environment (`emsdk_env.bat` set nothing that
  reached CMake, which then looked for `/opt/emsdk`). From PowerShell it does. A web build folder configured from one
  checkout cannot be reused from another: give `-WebBuild` a folder of its own.
- **A CustomMaterial's colour properties arrive in linear light, its textures do not** (19 October 2026): Qt converts a
  `color` property for the shader, but a picture sampled in it is still sRGB and must be converted there
  (`surface.frag`'s `qqLinear`, the polynomial Qt's own materials use).
- **`NORMAL_MATRIX` undoes a model's scale.** A normal worked out in the shape's own units (already scaled) must only be
  turned into the world (the model matrix's normalised columns), or a stretched shape's relief is flattened.
- **Qt lays an equirectangular sky with its middle column toward +x and a quarter of the way across toward -z** (u =
  atan2(z, x) / 2π + ½), its top row up; `probeOrientation` turns it. Measured with a fixed camera, not guessed.
- **Qt misreads an uncompressed Radiance `.hdr`:** it shows as a dark seam where the image's edges meet. Write HDR
  files run-length encoded, as tools do (`make_looks.py` in the fixtures and in Orb Run).
- **`glowBloom` bleeds a blur of the whole frame back over it**, not only what passes `glowHDRMinimumValue`: a made sky
  takes it, a bright sky image is washed out to white. It is 0 under a sky image.
- **Qt Quick 3D reads a picture (`Texture.source`) and a model itself, past the QML engine's URL interceptor.** A
  confined game's files are kept to its package where every file a scene names is found: `SceneIO::resolvedUrl`.
- **A Qt Quick `Shortcut` whose sequences overlap the platform's own is ambiguous, and does nothing** (20 October
  2026): `StandardKey.Redo` on Windows is already Ctrl+Y and Ctrl+Shift+Z, so naming Ctrl+Shift+Z beside it silenced
  it. Name the standard key alone.
- **A kinematic body moved in jumps is shot at what rests on it** (20 October 2026): Qt Quick 3D Physics turns a
  kinematic body's move into a speed over a single step, so a lift raised 0.05 every few frames threw its crate into the
  air. Move one a little each step or frame, as logic does.
- **A check that something reached a height can pass as it falls by.** Wait, and look again, before calling it at rest.
- **Qt Quick 3D's node class (`QQuick3DNode`) is private API in Qt 6.12**: SceneIO uses a node's properties
  (`position`, `scenePosition`) and invokables (`mapPositionFromScene`) through `QObject`.
- **A Markdown table row is one line.** A line break inside a cell ends the table there.
- **GitHub's runner can fail `apt-get update` on a third-party repository** (403 from `packages.microsoft.com`, 20
  October 2026), before any of the repository's steps run: rerun the failed job.
- **A QML function is invoked from C++ by its exact parameter count**: `QMetaObject::invokeMethod(window, "refresh")`
  finds no `refresh(key)`. The Studio keeps `refresh()` without parameters and takes an edit's key from a property.
- **The QQ Player is a windowed program:** started from PowerShell with `&` it is not waited for (no exit code, no
  capture yet). `Start-Process -Wait -PassThru` waits.
- **`keyring` has no Android store and falls back to one in memory**, which would lose a wallet's key when the app
  closes. The core's OS keychain is compiled only off Android; the phone's store is Keystore (L13.3).
- **In Qt 6.12 a `TableView`'s delegates cannot see the file's ids** (`TypeError: Cannot read property ... of
  undefined` on every binding) unless the file declares `pragma ComponentBehavior: Bound`; a `ListView`'s can. Found
  building QOR Media's reader (§4 item 134). Under `Bound`, delegates take model data by `required property` only.
- **`PdfSearchModel.count` and `countChanged` are not reachable from QML in Qt 6.12** ("not available in QtQuick.Pdf
  6.12"): they sit on the base class at revision 6.8, which the QML type does not export. Count the model's rows
  (`rowCount()`, `rowsInserted`, `modelReset`) instead. Setting its `currentResult` to -1 is ignored.
- **`forceActiveFocus()` on a `FocusScope` gives the keyboard back to the child that had it last**, a text field
  included, and `hasActiveFocus()` on the scope is then still true. To take the keyboard off a field, focus another
  item inside the scope.
- **`QTest::keyClicks` types only into widgets**, not into a `QQuickWindow`: send `QTest::keyClick` a character at a
  time. And `findChild` misses what a `Repeater` or a view made, which has no `QObject` parent: walk `childItems()`.
- **A `--tmp` validator needs `--node-key`** (a fixed one does for a rehearsal), or it stops at `NetworkKeyNotFound`
  (items 44 and 133).
- **Qt PDF keeps open the file it shows**, even after `PdfDocument.source` is cleared (which also warns "Cannot open"),
  and Windows will not replace or rename an open file. To write the file, show another one first (item 135).
- **A `PdfPageImage` a view has let go of lingers until the next turn of the event loop**, and still follows its
  document: a change of the document's file then warns that "document and source properties [are] in conflict", and
  the lingering image may open the new file itself. Let the view go first (`forceLayout()` applies a model change at
  once), change the file on the next turn. A `PdfPageImage` with no document falls back to opening its file itself.
- **`QFile::copy` on Windows copies the read-only mark**, and a read-only file cannot be deleted: clear it first.
- **`Qt.callLater(f)` runs `f` even if its object has gone** ("attempted to evaluate a function in an invalid
  context"). Deferred work on an item that may be destroyed belongs on a `Timer` of its own, which goes with it.
- **`TableView.positionViewAtRow` on a row not laid out yet moves the view at its next frame**, so a later jump made
  before then is undone when the stale one lands. Call `forceLayout()` after it (item 136). A test that passes may be
  passing on timing: one that jumps twice at once shows it.
- **QOR Media's reader suite crashed once in Qt's image thread** (`QQuickPixmap` → `QImageReader` → Qt PDF's image
  plugin, `0xc0000005`, 23 October 2026), and not in fourteen runs after. If it recurs, keep the stack QTest prints
  and look at page images being made and dropped quickly while Qt PDF renders them (item 136).
- **Qt PDF's `PdfPageImage` draws no annotations**, and has no setting for it: highlights, notes and stamps are missing.
  `QPdfDocument::render` with `RenderFlag::Annotations` draws those (item 137), but **nothing in Qt PDF draws a form's
  fields**, filled or not: item 137 said it did, and it was wrong until item 140 (see the next trap).
- **PDFium's `FPDF_RenderPageBitmap` draws no form field (widget), even with `FPDF_ANNOT`.** Only `FPDF_FFLDraw` does,
  through a form environment (`FPDFDOC_InitFormFillEnvironment`) told of each page (`FORM_OnAfterLoadPage`), drawn
  over the page; Qt PDF never makes one (its private `QPdfDocumentPrivate` holds no form handle). Since item 140 QOR
  Media draws a PDF with a form that way, through its own PDFium, and one without through Qt PDF, whose PDFium has a
  lock of its own (`src/pdfdrawing.cpp`).
- **QOR Media's PDFium is behind one lock (`PdfiumLock`)**, which the reader's own reads of a page's marks and fields
  take on the application's thread as each page shows. Drawing every page through it would make scrolling wait on
  drawing; drawing a PDF without a form through Qt PDF keeps them apart (item 140).
- **PDFium fills a field only through its form, as a person would**: `FORM_SetFocusedAnnot` (false unless the page was
  told to the form), then `FORM_SelectAllText` and `FORM_ReplaceSelection`, `FORM_OnChar` with a space for a box or a
  radio button, or `FORM_SetIndexSelected`; the value is written and the widgets drawn again only at
  `FORM_ForceToKillFocus`. A radio group cannot be turned off that way. `FPDFAnnot_GetLinkedAnnot` returns only
  dictionaries whose `/Type` is `/Annot`, so a field's own dictionary above its widget (`/Parent`) cannot be read with
  it: a `MaxLen` kept there is unseen, and PDFium cuts words to it when filling (item 140).
- **PDFium writes a password field's words into the file**, which PDF 1.7 (table 228) says a reader never does: QOR
  Media does not fill password fields (item 140).
- **The IRS's forms (W-9, Form 1040) keep their answers in XFA beside their fields** (`FPDF_GetFormType` is
  `FORMTYPE_XFA_FOREGROUND`; field names like `topmostSubform[0].Page1[0].f1_01[0]`); USCIS's I-9 is a plain form.
  `grep /XFA` on the file finds nothing, its catalog being compressed: ask PDFium. Acrobat shows such a form's answers
  from its XFA, so QOR Media shows these and does not fill them (item 140).
- **A `Loader` makes itself the context object of what it creates**: inside a `Component` it loads, a bare `item`
  means the Loader's `item` (null while loading), not an `id: item` in the same file. Name no id `item` in a file whose
  Loaders load its components (`FieldItem.qml`, item 140).
- **A function a binding calls must not write a property it reads**, or the binding loops ("Binding loop detected for
  property model"); work that writes, such as a notice, goes to `Qt.callLater` (item 140).
- **`QTest::keyClick(window, Qt::Key_Tab, Qt::ShiftModifier)` arrives as `Key_Tab` with Shift**, not `Key_Backtab`, as
  a real Shift+Tab may: a handler takes both (item 140).
- **`QPdfDocument::close()` does not let go of a file the document opened by name**; destroying the document does. Give
  it a `QFile` of your own (`load(QIODevice *)`) and close that, to free the file at a time you choose (item 137).
- **QOR Media's reader suite stopped now and then on its window-focus wait**: `read()` waits for the window to be
  active, and Windows may keep the keyboard on another window, more often for a run started in the background. Four
  times on 23 October 2026, each in a test that then passed on its own; the helper warns which wait failed ("did not
  become active" is this flake; anything else is not). Since 25 October 2026 the window asks for the keyboard and holds
  it (`tests/activewindow.h`, and the trap above on what takes it); rerun the suite alone before suspecting the code
  (items 138 and 155).
- **A `MouseArea` inside a `Flickable` (a `TableView` too) loses its drag to the view** once the pointer passes the drag
  distance, and is cancelled part way: a mark dragged on QOR Media's pages stopped half way. `preventStealing: true` on
  a `MouseArea` that drags (item 138).
- **A `TapHandler` with `gesturePolicy: TapHandler.WithinBounds` keeps its grab until the release**, so a `DragHandler`
  beside it in the same item becomes active only at the release and never sees the drag. Something both tapped and
  dragged takes a `MouseArea`, which also keeps its press from the items under it (item 138).
- **PDFium's `fpdfview.h` brings in `windows.h`**, whose `near`, `far`, `min` and `max` macros break what follows: no
  function named `near`, and `qMax` (or `(std::max)`) for `std::max` (item 138).
- **A `QVariantList` braced around one list copies that list** (`QVariantList{points}` is `points`), where two or more
  make a list of lists: wrap a single one in `QVariant(...)` (item 138).
- **`QPdfDocument::render` draws onto transparent pixels**, which `QColor(image.pixel(...))` reads as black: a test that
  counts dark pixels lays the page on white first, as the reader's paper does (item 138).
- **A QML `var` property holds a `QVariantList` when C++ gives it a list**, and a `QJSValue` when QML made the array: a
  test reading one handles both (item 138).
- **`FPDF_LoadPage` parses all a page draws**: reading every page's annotations as a PDF opens would stall a long one.
  QOR Media reads a page's marks as the page is shown (item 138).
- **`FPDFAnnot_SetAP` gives an appearance an `/ExtGState` named `GS` only when the annotation's `/CA` is below 1**, which
  `FPDFAnnot_SetColor`'s alpha sets, and which it refuses once an appearance exists: set the colour first, then draw
  with `/GS gs` for a translucent highlight (item 138).

## 6. Local environment

*Recorded between 14 and 21 September 2026 and not re-checked since. Read `docker ps -a` before relying on any
container named here.*

- **Cleaned up on 2026-09-14.**
  - The test containers `qor-db-1` and `qor-redis-e2e` are removed.
  - Their throwaway databases (`qor_auth_build`, `qor_auth_old008`, `qor_auth_e2e`) are dropped.
  - The volume `qor_qor_pgdata` was kept. It still holds the `qor` and `qor_test` databases that existed
    before these sessions.
- **Docker Desktop was running on 2026-09-15,** with the `vyb` stack it auto-started. Nothing in `vyb` was changed.
  It had stopped by the logout round (2026-09-14) and was started again for it, and again for the live email
  run (2026-09-15). It is installed at `%LOCALAPPDATA%\Programs\DockerDesktop`, not under Program Files.
- **For the rounds of 2026-09-14 and 2026-09-15:** the temporary containers `qor-verify-pg` (port 55432) and
  `qor-verify-redis` (port 56379) were created for each round and removed after it, most recently at the end
  of 2026-09-15.
- **Correction:** `qor-ci-check` (5434) and `qor-redis-ci` (6381) were stopped, not removed. They still exist,
  exited, alongside `qor-launcher-test-pg` (5433) and `qor-launcher-test-redis` (6380). None was touched in
  these rounds, by the owner's instruction; remove them with `docker rm` when no longer wanted.
- **For the key-scheme round of 2026-09-19:** two throwaway containers were created and **left running**,
  `qor-sr25519-pg` (port 55434, database `qor_auth_build`, user and password `qor`) and
  `qor-sr25519-redis` (port 56380). They are what `DATABASE_URL` and `QOR_AUTH_TEST_REDIS_URL` pointed at
  for the compile and the tests. Remove them with `docker rm -f` when they are no longer wanted; nothing
  in them matters.
- **For M3, on 2026-09-17:** the `wasm32v1-none` Rust target was installed with
  `rustup target add wasm32v1-none`, because the Polkadot SDK prefers it for runtime wasm. Nothing else was
  installed; `protoc` 36.0 was already present.
- **For the L1.7 round:**
  - the temporary containers `qor-ci-check` and `qor-redis-ci` were created and then removed;
  - `cargo-audit` was installed into `~/.cargo/bin`;
  - PyYAML was installed only into the session scratchpad, to parse `ci.yml`.
- **Measured on 2026-09-21, against `docker ps -a`:** `qor-verify-pg` and `qor-verify-redis` still exist,
  exited, although the entry above says they were removed after 2026-09-15; so do `qor-ci-check`,
  `qor-redis-ci`, `qor-launcher-test-pg` and `qor-launcher-test-redis`. `qor-sr25519-pg` and
  `qor-sr25519-redis` are exited, no longer running. All are throwaway; remove them with `docker rm` when
  no longer wanted. For QOR ID's run of 2026-09-21, `qa6231-pg` (port 55433) and `qa6231-redis` (56380)
  were created with `--rm` and stopped afterwards, so they are gone.

## 7. Repository state

- **Now (26 October 2026): `main` on `QOR-MATRIX/demiurge-chain` is at `bf41489`**, the merge of #104; pull requests
  #1 to #104 are merged (§4 item 158), and CI passed every job on it (run 40294205647). The L6.2 records are #105, and
  M4.3's rental #106 on #105's head (§4 item 159).
- **Earlier (25 October 2026): `main` was at `fd360be`**, the merge of #103 (§4 item 157).
- **Earlier (25 October 2026): `main` was at `c6b5536`**, the merge of #101 (§4 item 155); launcher 0.1.17 was published
  from `5db8349` on `session/release-0.1.17` (#102), and the website's play link was #103 (item 156).
- **Earlier (25 October 2026): `main` was at `13c43ad`**, the merge of #100 (§4 item 154).
- **Earlier (25 October 2026): `main` was at `20d505f`**, the merge of #99 (§4 item 153). Each pull request targets `main`; one
  built on an unmerged one's head says so and is opened against `main` all the same, its diff shrinking once the first
  merges.
- **Earlier (25 October 2026): `main` was at `3da41d0`**, the merge of #98, with ADR-118's link as #99 (§4 item 152).
- **Earlier (25 October 2026): `main` was at `beafda4`**, the merge of #97 (§4 item 151).
- **Earlier (24 October 2026): `main` was at `be63c34`**, the merge of #94, with #95 (records), #96 (the arenas) and #97
  (the website's pointer) to follow, each on the one before (§4 items 148 to 150).
- **Earlier (24 October 2026): `main` was at `405e41b`**, the merge of #92, with #93 and #94 to follow (§4 items 146 and
  147).
- **Earlier (24 October 2026): `main` was at `fddb3ed`**, the merge of #87, with #88 to #92 stacked on one another
  (§4 items 141 to 145).
- **Earlier (22 October 2026): `main` was at `a401c0f`**, the merge of #71, with ADR-108's fourth slice, the launcher,
  as pull request #72 (§4 items 117 and 118).
- **Earlier (10 October 2026): the public repository `QOR-MATRIX/demiurge-chain`** (ADR-064, remote `matrix`), whose
  `main` is at `0583835` (PR #19, P3.1; PRs #1 to #20 are merged). Work goes on a `session/*` branch and reaches `main`
  by pull request. `origin` is the private archive and is not the current tree, and the local branch `main` follows
  it, so it is not the public `main`. §4 item 65 (P3.2) is on `session/qq-p3.2-play`, branched from `0583835`, in
  pull request #21 (merged); §4 items 66 and 67 (P3.3) were merged as pull request #22 into #21's branch, not `main`,
  and pull request #23 brought them to `main`; §4 item 68 was merged as pull request #24. `main` is at `b99c689`.
  Stacked pull requests are not used again: each targets `main`.
- **Earlier (29 September 2026): the public repository's start.** Local branch `public-main` then tracked
  `public/main` on `ALaustrup/demiurge-chain`: `e611c99` (import) and `2681a51` (CI and Railway records). The
  repository moved to `QOR-MATRIX` on 1 October 2026 (ADR-064).
- **Before that (29 September 2026): branch `session/trading-cards-2026-09-28`**, pushed to `origin` up to `3d63b43`, 64
  commits ahead of `main`. §4 items 18 to 24, listed below as not committed, were committed on 28 September
  (`a3109e2`, `928cab5`). **§4 item 31 is committed, on the owner's word, in three concerns:** the chain and the
  launcher's one-line mint change (`f24f6dd`); the `DMRG` prose sweep (`57232da`); and the records —
  ADR-061, ADR-062, both indexes, `PROTOCOL.md`, `DIRECTION.md`, `GATES.toml`, `OPEN_QUESTIONS.md`, `SYSTEMS.md`,
  `OWNER.md` and this file — in the commit this entry is part of. Pushed to `origin` the same day. `archive-7gUg78/`
  at the root is a GitKraken CLI download (`gk_3.1.76_windows_amd64.zip`) left by a plugin install; it is not
  project work and is not committed.
- **Earlier: branch `session/realignment-2026-09-21`**, created from `main` at `7bfcec6` after
  `session/cgt-foundation-and-launcher` was fast-forwarded into `main`; remote `ALaustrup/demiurge-cloud`.
- **Committed on 2026-09-21 on this branch, one commit per concern:** QOR ID's deployment configuration
  (`119ca53`); Projects, Qontrol's first surface (`5a44207`); QFX layer one (`75b2ead`) and its scope-change
  entry (`42d00c3`); then, after an API failure stopped the session and the work was resumed: Ambience Off
  fixed with checks that can fail (`6e5bb28`); ADR-046 to ADR-051 (`df3ccce`); the seven blueprints
  (`e998faf`); the gates parser reading `P` tracks (`d4be885`); Qontrol's no-skips feature and the CI that
  runs what `npm run check` runs (`7a3c8ea`); the product tracks, gates and systems table (`53d1b7c`);
  `OWNER.md` and this file (`4233c10`); and P1.1 unticked, because its diffs were never written (`f1d499b`).
- **Committed on 2026-09-22, one commit per concern:** `--ink-faint` fixed and the contrast check extended to
  every theme (`320be99`); ADR-047 accepted and M2.3 ticked (`d5133d7`); the check narrowing approved and
  ADR-051 amended and accepted (`9a7eeb9`); the owner's product decisions (`a2537e1`); and this file with
  `OWNER.md` and `SYSTEMS.md` (`643c54e`).
- **Committed on 2026-09-22, the M4.1 session, one commit per concern:** M2.1 ticked and the royalty bound
  carried forward (`99bb0be`); `alpha.chain-tests`' four test names corrected (`5a0378f`); M4.1's chain half,
  `pallet-nfts` and `pallet-drc369` with ADR-052 and U-14 (`d11862e`); `dev-fund.mjs` (`6f934c7`); the
  Projects surface's undefined classes (`3f67870`); the accessibility check's visibility precondition
  (`f596a2b`); M4.1's launcher half (`021d1c4`); M4.1 ticked with the roadmap, blueprints and ADR-047's second
  open item (`3d27c69`); and this file with `OWNER.md` and `SYSTEMS.md` (`5648e91`).
- **Committed on 2026-09-22, from the owner's smoke test:** the launcher matching QOR ID's refusals as they are
  worded now (`26f2521`); the interface lifted above the QFX backdrop and its scrim, with
  `check-readability.mjs` (`1f5f4ed`); three colours raised to read AA over any backdrop (`67a083d`); those
  status documents (`53d7ea2`); the DRC-369 card shell with the carve-out the owner widened (`5d8ad74`); the
  owner's asks of that day written into the roadmap, with the correction that the chain holds no files
  (`ebb93f3`); trade — `pallet-utility` with only `batch_all` reachable, an asset's menu, the warning and the
  live proof, plus the widened media-type table and Q-20 (`d5e59f4`); and this file with `OWNER.md` and
  `SYSTEMS.md`.
- **Committed on 2026-09-23, on the owner's word, one commit per concern — four, not five.** The trade half
  and the Sell half **could not be split**: `lib.rs` declares both new modules and `ipc.ts`'s `trade` carries a
  `label`, which binds `TradeDialog` and `SellDialog` to one signature, so splitting them would mean inventing
  an intermediate state that never existed — the same reason the vault and chain-client halves were one commit
  on 2026-09-20. An earlier draft of this entry promised five; that was wrong and is corrected here rather
  than quietly dropped.
  - `9484f23` — the code: the trade window's two sides and lane, `partners.rs`, `Sigil.tsx` lifted out of the
    card, and the listing draft (`listings.rs`, `SellDialog.tsx`, five Tauri commands, `ipc.ts`).
  - `35b82aa` — the Inventory check: the duplicate `sent` that stopped it parsing, the Sell block that read a
    dialog which never opened, and the twenty checks.
  - `a8c22eb` — `GATES.toml`: L4.6 and P2.8 counted, and the correction of the 110 that was never measured.
  - the documents, in the commit this entry is part of: `DIRECTION.md`, the launcher `README.md`, `OWNER.md`,
    `docs/SYSTEMS.md` and this file.
  - **Not pushed, and no pull request, on 2026-09-23.** (Superseded: work now reaches `main` by pull request.)
- **What was already in the tree when the session of 2026-09-23 began, written and never once run:** the
  trade window's two sides and lane, `partners.rs`, `Sigil.tsx`, the listing draft and its five Tauri
  commands, the Sell wiring, and the Inventory check's twenty new checks. Running it is what that session
  did, and it is how all three faults in §4 item 17 were found — none of them was visible by reading. The
  documents and the gate-coverage fix were written that day, after the evidence existed.
- **Not committed when written; committed on 28 September 2026 in `928cab5`, in the private archive:
  §4 item 18**, Qontrol's diffs — `qontrol/diff.rs`, the change to `qontrol/mod.rs` and
  `commands.rs`, ten host tests, `ipc.ts`, `Projects.tsx`, the Projects and readability checks, `GATES.toml`,
  `DIRECTION.md`, the Qontrol blueprint, the launcher `README.md`, `OWNER.md`, `docs/SYSTEMS.md` and this
  file. The owner commits.
- **Not committed when written; committed on 28 September 2026 with item 18: §4 item 19**, Qontrol's git layer — `qontrol/guard.rs`, the changes to `qontrol/mod.rs`,
  `commands.rs`, `sidecar.rs`, `tests.rs` and `lib.rs`, the helper's `main.rs`, `scripts/build-helper.mjs`,
  `src-tauri/tauri.bundle.conf.json`, `package.json`, `.gitignore` (the built helper and licence are not
  tracked), `ipc.ts`, `Projects.tsx`, the Projects and readability checks, and the same documents as item 18.
  Also the trade fix found while re-running it: `src/views/TradeDialog.tsx`, the new check in
  `scripts/check-inventory-view.mjs`, and its `GATES.toml` entry — a separate concern, and a clean commit
  of its own. And §4 item 20, the vault restore: `vault/mod.rs`, `error.rs`, two commands in `lib.rs`,
  `ipc.ts`, `components/gate/Gate.tsx`, and the launcher `README.md` — a commit of its own except for `lib.rs`,
  which also carries item 19's Qontrol command registrations, so that file needs staging by hunk. §4 item 21,
  Windows Hello, touches the same vault, Gate and `lib.rs` files as item 20, plus `vault/hello.rs`,
  `SettingsView.tsx`, `Cargo.toml`, `Cargo.lock`, and ADR-054 with its two index rows. **§4 item 23**, Hello
  alone (ADR-055), rewrites those same files again — `vault/mod.rs`, `vault/hello.rs`, `error.rs`, `lib.rs`,
  `chain/mod.rs`'s three live tests, `ipc.ts`, `Gate.tsx`, `SettingsView.tsx`, `ascent.ts`, both Gate checks —
  and adds ADR-055 with its index rows, the notes on ADR-054 and ADR-016, and edits to `SECURITY.md`,
  `DIRECTION.md`, `MIGRATION_INVENTORY.md`, `GATES.toml` and the launcher `README.md`. §4 item 24 (ADR-056) rewrites
  them once more and adds `vault/keychain.rs`, `App.tsx`, `store.ts`, `TitleBar.tsx`, `VaultView.tsx` and
  `chain/assets.rs`'s tests. Items 20, 21, 23 and 24 cannot be separated cleanly: one commit for the vault is
  the honest shape. Items 18 and 19 share files (`mod.rs`, `tests.rs`, `Projects.tsx`, the checks), so they may not split into
  two commits cleanly. The owner commits.
- Earlier history, kept as it was written:
- The owner confirmed credential rotation on 2026-09-14, which cleared the branch for pushing.
- The QOR ID fixes are committed one commit per fix: those of 2026-09-14 end at `67de317`, and those of
  2026-09-15 at `517fc83`. That covers:
  - resend-verification, backup-code regeneration and ADR-017;
  - adding or changing an email address and the rewritten emails;
  - the log span fix, the link pages and Resend webhooks;
  - `BASE_URL` origin-only, the admin unmark route and the standing log check;
  - ADR-015's clarification, and the records of all of these.
- Committed on 2026-09-15 after `517fc83`, one commit per concern:
  - the SQL parameterisation check;
  - AGENTS.md §9, the review requirement;
  - the base-layer decisions (ADR-018 to ADR-032), with the release re-check and the naming amendment;
  - U-10, U-13 and the other open-question updates;
  - the sponsorship proposal;
  - the dependency-version policy (ADR-033).
- Committed on 2026-09-17, one commit per concern:
  - the record of the ticker research, and three corrections to what U-13 had claimed;
  - a correction of occurrences counted as lines;
  - **the ticker change itself** (ADR-034): the declared symbol, the launcher's hard-coded fallbacks and its
    copy, the migration requirement R-4, the `public-release.name-clearance` gate criterion, the rename of the
    economic model to `docs/economics/CGT.md`, and two defects found while doing it;
  - this documentation alignment pass.
- Committed on 2026-09-18, one commit per concern: the acceptance tests and ADR-038, the block-import
  harness, the two-validator script and its accepted evidence, the narrowing of M3.1 and M3.3, the deletion
  of the seven fork scripts with the check that enforces it, and the rewrite of `PROTOCOL.md`.
- **Committed on 2026-09-20, one commit per concern.** This covers both the key-scheme round of
  2026-09-19 and the chain-client round of 2026-09-20, which were both still uncommitted:
  - **QOR ID's accounts become Sr25519 and SS58** (ADR-039, migration 018), with `SECURITY.md`, the CI
    log scan's new names and `GATES.toml`'s two tightenings for it.
  - **The launcher: an Sr25519 vault and a `subxt` chain client** (L3.2 and L3.1, ADR-040). The vault's
    half and the client's half are in one commit because the client rewrote the files the vault's half
    had changed, and the vault's half was never committed on its own; splitting them now would mean
    inventing an intermediate state that never existed.
  - **The documentation alignment** for both rounds: `DIRECTION.md` (L3.1, L3.2 and M3.4 ticked),
    `DECISIONS.md`, the ADR index, the migration inventory, `docs/audit/RECONCILIATION.md`,
    `chain/README.md` and this file.
  - **Q-17, the runtime's address type, written up for the owner**: `docs/architecture/ADDRESS_TYPE.md`,
    its row in `docs/README.md`, and `GATES.toml`'s `beta.address-type` tightening.
  - **`AccountIdLookup` and `MultiAddress`** (ADR-041), the owner's decision on Q-17, carried out the same
    day: the runtime, the two chain test files, the launcher's `subxt::Config` and its transfer, in one
    commit because a node and a client on opposite sides of it refuse each other.
  - **M3.5's reference work**: AGENTS.md §4, §7 and §8, `.cursorrules`, CI (the `framework/` job replaced
    by a `chain/` job that is a quality gate), `HANDOFF.md` §2.0, `scripts/run-local-stack.md`, `GATES.toml`'s
    CI scope, and DIRECTION's M3.5. **Verified with the tree still present:** `framework/` 285,
    `chain/` 51, the launcher 91, all green.
  - **The deletion of `framework/`**, on its own, after the owner confirmed it: 150 files, 49,058 lines,
    plus the CI guard against its return and M3.5 ticked. Scoped to `framework/` alone, by the owner's
    choice; the six deploy files that referred to it are a separate cleanup (§5).
- **No pull request was opened then.** Since October 2026 work reaches `main` by pull request (the first entry
  above).
- The confirmation that gated M3 arrived on 2026-09-17 (§4 item 3), so nothing in the chain work waits on it any longer.
