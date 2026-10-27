# Where Demiurge stands

For Andrew, 26 October 2026. One page, rewritten every session.

## What Demiurge is

A blockchain that pays makers when their work is used, and a desktop launcher to use it.
The currency, **CGT**, is for spending, not holding. Seven products are planned on one
sign-in, one asset format and one currency. Code: `github.com/QOR-MATRIX/demiurge-chain`.

## Live today (checked this session, from outside)

The homepage at **qorsync.dev**; **Devnet** — *the test chain on servers* — on runtime 11,
with renting, two peers; **QOR ID** — *the one sign-in*; **Social** — *the chat service*;
**ARQADE** — *the gaming website*. **Launcher 0.1.17** is the newest release and the
homepage's download. **#106** (renting) is merged, so `main` matches the devnet.

## This session

1. **QOR OS begins.** You chose to merge QTOS — *your Qt operating system* — with this
   project's device-side work into **QOR OS**, in a private repository. **ADR-120** — *its
   decision record* — says what that means here:
   - QQ, QOR Media and ARQADE's app are carried there with their history.
   - This repository stays their source.
   - QOR OS uses the launcher's **core** — *the code that holds keys* — at a fixed commit, never a copy.
   - Nothing in the network moves.
2. **The launcher's core is finished (L13.2).** Everything that holds a key or talks to a
   service moved into it with its 181 tests: QOR ID, Social, the chain, the Library, the Mesh
   and Qontrol. The Windows launcher behaves as before, and the phone app builds what it did
   (its APK built again today). On your PC 351 tests pass and none fail. It is pull request
   **#107**, waiting for you.
3. **One decision before #107 merges: ADR-121.** Moving the chain code put a known
   **advisory** — *a published flaw in a library* — into the core's dependency check. It sits
   in code nothing builds, and the launcher's check has ignored the same one since ADR-089. I
   recommend the same guarded ignore; CI proves every run that the code stays unbuilt. Ignoring
   it is your call, so it is a separate commit you can drop.
4. **Disk space:** X: had 9.6 GB free. Build leftovers and three finished agent copies are
   cleared, and it has 204.7 GB free.

## The release gates (measured 22 October)

**Alpha** — *the first test release* — lacks two things: M3.2, which needs the genesis split
(OPEN-2), and two launcher dialogs you have not yet tried (L1.4). **Beta** — *the worldwide
test* — has 8 of 45 checks met; most of the rest wait on the list below.

## What only you can do

1. **Decide ADR-121, then merge #107.** Then QOR OS can fix its copy of the core to that merge.
2. **Play an arena,** with a friend or alone with a second QOR ID: open a match in the
   launcher's ARQADE, join it from ARQADE's website in a private window (HANDOFF §4 item 157).
3. **Back up the update key now.** From any folder, run
   `pwsh -File "X:\REPOZ\qq-interface\tools\qor-launcher\scripts\updater-key.ps1" -Backup`
   It names the key file and shows its password once; keep both offline, together.
4. **For rewards in ARQADE:** decide **ADR-116** — *how a run is confirmed before it is
   paid* — and set the reward pool's numbers (largest payout, budget per epoch, most per player).
5. **Let a test agent ask you:** in `X:\REPOZ\qq-interface\tools\qor-launcher\src-tauri` run
   `cargo run --example agent_link` and press *Allow agent* (ticks L5.1).
6. **Terms of Service and a privacy policy, with a lawyer,** and **the legal opinion** on
   rewards, prizes and **KYC** — *identity checks* — before anything carries value.
7. **The economic numbers** — *issuance, the genesis split, the decay curve, fee and burn
   shares* (OPEN-1 to OPEN-4, U-4, U-11). Most of the Beta gate waits on them.
8. **Code-signing certificates**, an **external security review**, and **names** for the
   indexer, the SDK and the **MCP server** — *how AI tools reach Demiurge*.
9. **Still open, as HANDOFF §4.0 records:** godmode's 100,000,000 CGT (its address); Social's
   report rules; the Android app on your phone; whether #46's gate change reads to you as
   loosening; Search Console and the GitHub preview image; the welcome-grant payer; Qt for CI;
   the email webhook; the key pasted on 5 October; `social-secrets.txt` on your Desktop and
   `archive-bZLIQp/` in the old checkout, each to keep or delete; and which licence governs the
   launcher's crates (their manifests say MIT, the repository Apache-2.0).

**Next in engineering:** after #107 merges, QOR OS pins the core. Then, at your word, renting
in the launcher, or keys on Android (L13.3).

## Cost per month

The homepage and ARQADE: Vercel's free plan (not for commercial use). GitHub: free. The Mesh,
QOR Media, ARQADE's app and translation: $0, on each PC. Railway is not yet measured. Neon: free.
