# ARQADE SDK

**For developers building games on ARQADE**, the gaming platform on Demiurge. It lets a game use QOR ID players, pay
and charge in CGT, and award DRC-369 assets, without the game ever holding a player's key.

> **Status: started 4 October 2026, unpublished.** It is published only after the DRC-369 wire format is frozen
> (ADR-009, `beta.wire-format-frozen`), and its package and command names are placeholders until the project owner
> approves them (AGENTS.md §8). The design is ADR-069 decision 9 and P7.8 in `docs/DIRECTION.md`. Its chain-facing half
> moves to `platform/` when the ecosystem SDK (M5.1) exists; this directory keeps the game layer.

## What exists today

| Module | What it does | Tests |
| --- | --- | --- |
| [`src/amount.ts`](src/amount.ts) | CGT as integer Sparks (`bigint`, `1 CGT = 10^18 Sparks`): `parseCgt`, `formatCgt`, `sparksFromJson`, the existential deposit (100 CGT, ADR-036). Refuses excess precision, signs, exponents and anything past `u128`; never rounds, never uses floating point | `tests/sdk.test.mjs` |
| [`src/arq-wallet-policy.ts`](src/arq-wallet-policy.ts) | An ARQ Wallet's payout policy: `validatePolicy` names every problem before the governor signs it, against the bounds the runtime reports; `loosens` says which changes must wait the delay | `tests/sdk.test.mjs` |
| [`src/profile.ts`](src/profile.ts) and [`templates/arqade-project.json`](templates/arqade-project.json) | The project profile that becomes a store page; `checkReadiness` and `readyStage` for the four stages; `priceProblem` with the owner's 10,000 CGT ceiling | `tests/profile.test.mjs` |
| [`src/arq-wallet.ts`](src/arq-wallet.ts) | A Cartridge's ARQ Wallet address (`arqWalletAccountId`, `arqWalletAddress`, `ss58`), the same bytes the chain derives and pins; `outcomeId` and `roundId`, deterministic and domain-separated, so a retried payout names the same outcome; `toChainPolicy` and `payoutArgs`, the chain calls' arguments, checked | `tests/arq-wallet.test.mjs` |
| `../lib/chain.ts` (the app's, for now) | Reads Demiurge Devnet read-only, refusing any chain whose genesis is not the devnet's | `../tests/chain.test.mjs` |

Run them from `products/arqade`: `npm test`.

## Guides

- [**ARQ Wallets**](docs/arq-wallet.md): paying players CGT from a keyless account that belongs to your game alone,
  and how to unlock one ("ignition"). **Decided (ADR-070); the chain module is built, tested and live on Demiurge Devnet; the SDK calls are not built.**
- [**Publishing**](docs/publishing.md): the four stages, the profile, prices up to 10,000 CGT, in-game items.
  **The checks are built; the rest is decided (ADR-071, accepted 4 October 2026) and not built.**
- [**Backing**](docs/backing.md): campaigns, memberships and tips in CGT, rewards and never proceeds. **Designed.**
- [**Building with your own LLM**](docs/building-with-agents.md): the MCP connection and what a model may do alone.
  **Designed.**

## What is proposed, and waits on what

| Surface | Waits on |
| --- | --- |
| Player sign-in (browser) | ADR-043's redirect flow in QOR ID (accepted, not built) and a way for a game server to verify a token |
| Signing a player's payment | The signing path of ADR-069 decision 5, each with its own record |
| ARQ Wallets: `vault ignite`, `payouts.award`, rounds, claims | Sending and watching the calls (the chain client, M5.1); the CLI's `vault ignite`. Rounds are on chain at `spec_version` 8, reaching the devnet at its next upgrade |
| Trophies | An issuer account minting and transferring today; account-bound ones wait on G-16 |
| Cards, sets, packs | G-15 (editions) and G-14 (randomness) |
| Evolving assets | M4.2's state and XP |
| Game manifest, sessions, outcomes, leaderboards | P7.8, with the three reference games |

## Rules this SDK keeps

- **No key in a browser, and no player key anywhere.** A browser entry point can read and request; only a server entry
  point holds a game's payout authority, and only the player's own Vault signs for the player.
- **Integer Sparks end to end.** Decimal strings in JSON, `bigint` in code.
- **Finality, not inclusion.** An operation resolves at finality with its event, or reports itself unresolved; it is
  never blindly resent.
- **No value talk.** CGT pays for play, work and licensing; nothing here describes it as appreciating (ADR-008). On the
  devnet it is test CGT, and the SDK says which network every time value moves.
