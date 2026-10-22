# ADR-110: Emulated play in ARQADE: a host of our own, licensed and homebrew games first, and the economy only on games their makers put there

**Status:** **Deferred**, 22 October 2026, by the owner, the same day it was proposed: "Lets go ahead and log this emulation concept somewhere for future reference. For now lets proceed with our current roadmap objectives". It is kept here as
the reference for the day the owner returns to it. It is not a decision and not on the roadmap, and nothing is built or
deployed on its basis until the owner accepts it, amends it or rejects it (`docs/decisions/README.md`). Proposed at the
owner's request for "your absolute best recommendation for developmental support for a complete emulation framework
within our Arqade system". It decides **no economic value**: a retro game's price, the creator's share and any reward
are set as every other Cartridge's are.

**Relates to:** [ADR-069](ADR-069-arqade-the-gaming-platform.md) (ARQADE),
[ADR-071](ADR-071-arqade-self-publishing.md) and [ADR-088](ADR-088-a-cartridge-holds-a-commit.md) (publishing, and a
Cartridge holding a commit), [ADR-046](ADR-046-the-launcher-is-an-app-host.md) (the product host),
[ADR-087](ADR-087-strangers-games-are-confined.md) (strangers' code confined), [ADR-095](ADR-095-the-mesh-on-iroh-paid-from-hosting-escrow.md)
to [ADR-097](ADR-097-the-mesh-signs-its-own-work.md) (the Mesh), [ADR-099](ADR-099-each-qor-id-is-its-own-profile.md) (profiles),
[ADR-102](ADR-102-arqade-in-the-launcher-on-qt.md) (ARQADE's Qt application), [ADR-108](ADR-108-one-look-obsidian-and-gilt-in-two-registers.md)
(the one look), ADR-002 (the spend test) and ADR-006 (the demand sinks).

## Context

The owner asked whether ARQADE should support emulated play: running games made for other machines, consoles and
computers among them, inside ARQADE, building custom components where that is what it takes, with the legal side of
emulation understood. No current record had decided it; the question is new.

Three things shape the answer.

**Emulators are lawful; what they are used for often is not.** In the United States an emulator written from its own
code is lawful (*Sony Computer Entertainment v. Connectix*, 2000). The projects that rights holders have ended crossed
lines beside the emulator itself: Yuzu settled with Nintendo in 2024 (2.4 million dollars, and shut down) over games
decrypted with keys and over how it was presented; Dolphin's release on Steam was stopped in 2023 after Nintendo cited
the Wii's encryption key; Ryujinx stopped in October 2024. The lines are: distributing games or system software
(BIOS and firmware) without their rights holder; circumventing encryption; presenting a product as playing someone
else's games. These facts are from public reporting and are for the legal opinion to confirm (§4.0's Legal review).

**This system raises the stakes of each line.** ARQADE has an economy (CGT, Sparks, prizes awaiting the legal opinion,
U-15 to U-18), and the Mesh pays seeders in CGT for storing and serving files (ADR-095). Paying people to distribute a
copyrighted game, or rewarding play of it, would turn tolerated private use into commercial use of someone else's work,
in a product that already draws scrutiny for its currency.

**Emulators themselves come under licences of their own.** Several of the best known forbid commercial use, and most
are under the GPL, while this repository is Apache-2.0. Which emulator ARQADE may carry is a licence question before it
is a technical one.

And there is a real supply of games that are someone's to put there: new games made today for old machines (an active
homebrew scene sells new cartridges for the NES, Game Boy, Game Boy Advance and Mega Drive), "fantasy consoles" built
for creators (TIC-80), and classic catalogues licensed by their rights holders, as other platforms have done.

## Options

**A. No emulation.** ARQADE stays with QQ games and its own. *Costs:* gives up a large, loved category and a natural
home for retro and homebrew creators, whom ADR-008's "creators earn when their work is used" suits exactly.

**B. A player for anyone's game files.** A front end in the way of RetroArch: the person brings files, ARQADE plays them.
*Costs:* ARQADE becomes an emulator front end in rights holders' eyes, beside a currency and a paid file network; the
economy cannot touch any of it; and most of the value it shows belongs to others.

**C. A host of our own, the games their makers put there first (recommended).** Build the emulation layer complete
enough to play anything, and put distribution, promotion and the economy only on games whose rights holder publishes
them: homebrew, fantasy-console games and licensed classics. A person's own game files, played on their own PC with
nothing else attached, is a separate mode, off until the legal opinion approves it.

## Decision (proposed)

**Option C.**

1. **A libretro host of our own, not RetroArch and not new emulators.** libretro is the plug-in interface around a
   hundred emulators already implement; its header is MIT-licensed. ARQADE gets its own host for it in C++ and Qt,
   shown in ARQADE's Qt application (ADR-102) in the one look (ADR-108): integer scaling, optional screen filters, the
   gamepad and rebindable controls QQ already has, rewind and fast-forward, and the person's Motion setting. Emulators
   are not written here: decades of accuracy work exist, and improvements go upstream.
2. **One emulator per process, locked down.** An emulator parses a file it did not write, in native code. Each runs in a
   process of its own under the product host (ADR-046) with no network, only the one game file and its save folder,
   and nothing else of the person's; frames reach ARQADE through shared memory, input and sound through the host. This
   is ADR-087's reasoning for strangers' code, applied to emulators, and it also keeps each GPL emulator a separate
   program beside Apache-2.0 code.
3. **Only emulators whose licence allows it, and only machines that need no one's keys.** Every emulator is checked at
   its source, release by release, before it is carried, and its source offered as its licence asks. Machines that need
   decryption keys (Switch, 3DS, Wii U, PS3, Vita) are out. Machines that need a copyrighted BIOS wait until an emulator
   carries its own replacement or the legal opinion says otherwise.
4. **A retro game is a Cartridge.** Published through ARQADE's store (ADR-071) as a Cartridge holding its commit
   (ADR-088): the game file, the machine, the emulator it needs, and a statement of the publisher's right to publish
   it. A homebrew creator publishes and earns when it is played or bought, as any ARQADE creator does.
5. **The economy only where the maker put the game.** CGT and Sparks, prizes, leaderboards with prizes, paid
   achievements, cloud saves and Mesh distribution apply to retro Cartridges only. **Nothing of the economy ever touches
   a game file a person brought.**
6. **The Mesh carries a retro game only as its rights holder published it,** and only once the takedown list (L10.7)
   exists and ARQADE has a way to receive and act on a rights holder's notice.
7. **Bring your own is separate, and off until the legal opinion approves it.** If approved: the Library indexes a
   person's own game files, played on their own PC only; never uploaded, never in the Mesh, nothing of the economy, and
   no game file, BIOS or key ever offered, linked or suggested by ARQADE.
8. **Words.** ARQADE names machines only to describe compatibility, never with their makers' logos, and never presents
   emulated play as anyone's endorsement. ADR-008's language rule applies as everywhere.
9. **Replays.** A save state and a log of inputs reproduce a run, as QQ's replay does (P3.11), so a score on a retro
   Cartridge can be checked before any reward. Rewards wait on the legal opinion, as all of ARQADE's do.

**Spend test and sink.** A person spends CGT to buy or play a retro Cartridge because they want the game, and its maker
earns for the work: it passes the spend test (ADR-002) and belongs to **access gating** (ADR-006). Bring-your-own play
involves no CGT and needs neither.

## The roadmap it would add, on acceptance

Track P7, after P7.25, each step its own pull request, each with its tests and checks:

- **P7.26 The host.** The libretro host and its locked-down emulator process; ARQADE's Qt application playing three
  machines (NES, Game Boy and Game Boy Color, Game Boy Advance) from homebrew bundled with its makers' written
  permission; saves and save states per profile (ADR-099); every emulator's licence read at its source and recorded. No
  economy.
- **P7.27 Retro Cartridges.** Publishing for homebrew creators through the store; the Cartridge's machine, emulator and
  rights statement; a rights holder's notice received and acted on. The Mesh only after L10.7.
- **P7.28 More machines.** SNES, Mega Drive, PC Engine, Atari 2600 and TIC-80, each on an emulator that passes decision 3.
- **P7.29 In the browser.** The same emulators as WebAssembly, on a site of their own (ADR-087), for retro Cartridges.
- **P7.30 Replays, verified scores and netplay** for retro Cartridges; rewards only after the legal opinion.
- **P7.31 Bring your own,** only if the legal opinion approves it, under decision 7.

Licensed classics come from agreements with rights holders, which are the owner's to make; each arrives as a retro
Cartridge.

## Facts it relies on, and how sure they are

| Fact | Source | Checked |
| --- | --- | --- |
| An independently written emulator is lawful in the US | *Sony v. Connectix* (9th Cir. 2000) | Public record; for the legal opinion |
| Yuzu settled with Nintendo (2024), Dolphin's Steam release was stopped (2023), Ryujinx ended (2024) | Public reporting | Not re-read here; for the legal opinion |
| `libretro.h` is MIT-licensed | The libretro project | To be read at source in P7.26 |
| Mesen, bsnes and BlastEm are GPL-3.0; Gambatte, Stella and Beetle PCE are GPL-2.0; mGBA is MPL-2.0; SameBoy and TIC-80 are MIT; MAME is GPL-2.0 or later | Each project's licence | To be read at source in P7.26 and P7.28 |
| Snes9x, Genesis Plus GX, PicoDrive and FinalBurn Neo forbid commercial use; DuckStation moved to a non-commercial licence in 2024 | Each project's licence | To be read at source; none would be carried meanwhile |
| This repository is Apache-2.0 | `LICENSE` | Read, 22 October 2026 |

## Questions for the owner

1. **Accept, amend or reject** option C.
2. **A name** for the host and its directory (AGENTS.md §8). A placeholder until then: "the retro host", under
   `products/arqade/`.
3. **The first machines.** Recommended: NES, Game Boy and Game Boy Color, Game Boy Advance.
4. **Bring your own.** Recommended: built only if the legal opinion approves it (decision 7). Or: never; or: from the
   first release.
5. **The legal opinion's scope** gains emulated play, bring-your-own play, the emulators' licences, and receiving
   rights holders' notices.
6. **The first games.** Bundled homebrew needs each maker's written permission; approaching them is the owner's.

## Consequences

- If accepted: P7.26 to P7.31 join DIRECTION's track P7; `docs/blueprints/arqade.md` gains a section; the legal
  review row in HANDOFF §4.0 gains emulation; GATES gains criteria with the first step (the host's sandbox and the
  emulators' licence record).
- ARQADE's ADR-102 scope grows by one surface; QQ is untouched.
- If rejected, the record stays as the reasoning, with its status saying so.
