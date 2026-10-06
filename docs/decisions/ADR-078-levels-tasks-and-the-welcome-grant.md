# ADR-078: Levels, tasks and a welcome grant across the ecosystem, and ARQADE inside the launcher

**Status:** **Accepted**, 6 October 2026, by the project owner: "I accept ADR-078 with 100 CGT", setting decision 6's
N to 100 CGT. Proposed the same day. Decisions 1, 6, 7 and 9 are the owner's direction of 5 and 6 October 2026; the rest
were recommendations, accepted as written. Glowing ring styles (decision 4) were not decided: until the owner decides,
ring styles are colour and pattern only. It opens U-18 for the rest. Not yet built.

## Context

The owner asked for "a truly gamified experience that rewards users in CGT for completing tasks within the ecosystem",
embedded in the QOR Launcher and ARQADE and in what comes after them: a level shown in a small bubble beside the avatar,
an experience bar towards the next level with a hint of what it unlocks, levels that unlock simple things (avatar ring
styles, themes, a ten-minute boost to game rewards), level 0 for a new account and level 1 reachable within the tutorial.
"Leveling should be somewhat simple, and not require users to do anything difficult or costly nor scammy." New users are
to receive a small amount of CGT on finishing the tutorial, "to invoke a sense of value to the platform".

What this must respect:

- **No CGT is created** until issuance is designed (AGENTS.md §5): every CGT a user receives is moved from an account
  someone funded.
- **An account holds at least 100 CGT or does not exist** (the existential deposit, ADR-036). Its last 100 CGT cannot be
  spent while it stays open, so an account holding exactly 100 CGT exists but has nothing to spend.
- **CGT is something to use, not to hold for its price** (ADR-008, ADR-002): copy never speaks of earnings, yield or
  income.
- **The design system rules out glow and neon** on every surface ([`DESIGN_SYSTEM.md`](../design/DESIGN_SYSTEM.md), from
  the owner's original direction), with one exception already made: the launcher's ceremonies (`src/qfx/ceremony/`)
  may glow. The owner asked for glowing ring effects; extending the exception to them is the owner's call, and decision 4
  leaves it to them.
- **QOR ID reads no chain** today, and the launcher runs on the user's own machine, so neither can, on its own, prove
  that something happened on the chain or that a tutorial was finished.

## Decision

1. **One level per QOR ID, across every app.** Level and experience (XP) belong to the QOR ID account, so the launcher,
   ARQADE and any later product show the same progress and add to it. **A new account is at level 0.**
2. **XP is granted only by QOR ID or by a registered app's server, for something it can check itself, and each task
   once.** A task has a fixed key; an account receives its XP once per key (a unique row in QOR ID). The user's own
   machine cannot grant XP: the launcher reports a finished tutorial, and QOR ID grants the tutorial's XP only together
   with what it can confirm itself (decision 6).
3. **Tasks are free, easy and honest**: no purchase, no invitation quota, no streak that punishes a missed day. The first
   set, and the XP each grants (game design, not money; the owner may change any of them):

   | Task | Checked by | XP |
   |---|---|---|
   | Finish the launcher's tutorial | the launcher reports, QOR ID records once | 50 |
   | Verify an email address | QOR ID | 25 |
   | Link a key (a chain account for the QOR ID) | QOR ID | 25 |
   | Sign in to ARQADE | QOR ID (an app session) | 10 |
   | Finish a first multiplayer match | ARQADE's server | 25 |
   | Make a first tip or payment through the launcher | ARQADE's server, from finalised blocks | 25 |

   **Level L needs 50 x L x (L + 1) XP in all**: 100 for level 1, 300 for level 2, 600 for level 3, each level a little
   further than the last. The tutorial, an email and a key make 100: level 1 within setup.
4. **What is shown, and what levels unlock.** A small level bubble on the avatar, in the launcher and in ARQADE; an XP bar
   to the next level, with the next unlock named. Unlocks are cosmetic first: avatar ring styles, themes, badges. **Whether
   a ring may glow** is the owner's call against the design system; until they make it, ring styles are colour and
   pattern only. A **ten-minute boost to game rewards** is an unlock in name only until U-18 sets what it multiplies.
5. **ARQADE inside the launcher.** A **Play** section in the launcher opens ARQADE in a window of its own, signed in with
   the launcher's QOR ID, and a payment there is approved in the launcher's host dialog directly (ADR-046's host
   protocol), with no `qor://` link. The website stays for anyone without the launcher.
6. **A welcome grant**: once per new QOR ID, **N CGT** is sent to its chain account when QOR ID confirms that the tutorial
   is finished, the email address is verified and a key is linked. **N is the owner's.** The owner named 100 CGT; at 100
   CGT exactly the account opens with nothing to spend (the existential deposit above), so the grant does what it is for
   only if **N is above 100**: N - 100 is what the user can spend. One grant per email address and per key, as well as per
   account.

   **Set on acceptance: N = 100 CGT**, by the owner, told the above. The grant opens every new player's chain account, so
   a first reward, tip or prize can land in it; it leaves nothing to spend until one does. Raising N is a change of this
   value only.
7. **The grant is paid from a Welcome account, with no daily cap** (the owner's choice), by a server that holds that
   account's key and nothing else. The Welcome account's balance is then the only cap: when it is empty, grants stop and
   are owed until it is refilled. It never creates CGT. **On Demiurge Devnet, with test CGT, from the faucet, now;** with
   real CGT only once U-18 says who funds it and the owner's legal review has covered it.
8. **Task rewards in CGT** beyond the welcome grant, and the boost's size, are U-18's. Until it is answered, tasks grant
   XP only.
9. **At most three QOR ID accounts per network address** (the owner's choice). QOR ID keeps a keyed hash of the address
   a sign-up came from, not the address, for 30 days, and refuses a fourth sign-up from it in that time.
10. **An abuse watcher**: a scheduled agent reads sign-ups, task completions and grants for farming patterns and **flags**
    accounts for the owner's review. It suspends nothing on its own until the owner decides it may. Its design is a
    record of its own.
11. **Words.** "Level", "XP", "unlock", "reward", "boost", "welcome": never "earn", "earnings", "yield", "income" or a
    statement about CGT's price (ADR-008). The grant reads: "Welcome: CGT to start playing, tipping and creating."

## Consequences

- **First build, if accepted:** XP, levels and the task ledger in QOR ID with its log check; the level bubble and XP bar
  in the launcher and ARQADE; ring styles and themes as unlocks; the welcome grant on the devnet; the per-address limit.
  Then ARQADE in the launcher (decision 5), then CGT rewards for checked multiplayer wins from ARQADE's ARQ Wallet
  (P7.5).
- **A limit by network address is weak both ways**: a VPN or a mobile network gives a new address at once, while people
  who share one (families, students, internet cafes, carrier networks) meet the limit together. It deters the casual,
  not the determined; decisions 6's per-email and per-key rules and decision 10's watcher carry the rest.
- **Production CGT** for the grant, task rewards and game prizes waits on U-16, U-18, the legal review, and a production
  chain: today there is only the devnet.

## Not decided here

Whether ring styles may glow (decision 4), and U-18.
