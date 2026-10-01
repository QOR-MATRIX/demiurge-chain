# Architecture decision records

One record per decision, in context / decision / consequences form. The earlier decisions D-000 to
D-012 remain in [`../DECISIONS.md`](../DECISIONS.md), which also indexes these records and marks which
of the earlier decisions they supersede.

| ADR | Title | Status |
| --- | --- | --- |
| [ADR-001](ADR-001-innovation-budget.md) | The innovation budget: innovate where failure is loud, stay boring where it is silent | Accepted |
| [ADR-002](ADR-002-value-from-spending.md) | CGT's value comes from demand to spend it: the spend test | Accepted |
| [ADR-003](ADR-003-supply-model.md) | Base supply of one hundred trillion CGT, released on a decay curve | Accepted; numbers open |
| [ADR-004](ADR-004-issuance-and-burn.md) | Perpetual issuance for infrastructure, with fee burn as the counterweight | Accepted; numbers open |
| [ADR-005](ADR-005-useful-work-mining.md) | Useful-work mining on the Mesh, without artificial difficulty | Accepted |
| [ADR-006](ADR-006-demand-sinks.md) | Five structural demand sinks | Accepted |
| [ADR-007](ADR-007-display-layer.md) | A legible display unit over CGT, with no peg | Accepted |
| [ADR-008](ADR-008-language-discipline.md) | Language discipline: payment for work, never appreciation | Accepted |
| [ADR-009](ADR-009-universal-minting.md) | First release: universal minting, with game assets as the flagship | Accepted |
| [ADR-010](ADR-010-agent-rails.md) | Agent rails: delegated session keys and an MCP server | Accepted; amended by ADR-014 |
| [ADR-011](ADR-011-web-surface.md) | Two web surfaces: a public viewer and a thin remote console | Accepted |
| [ADR-012](ADR-012-substrate-base.md) | A purpose-built Substrate L1 as the base | Superseded by ADR-013 |
| [ADR-013](ADR-013-polkadot-sdk-migration.md) | Migrate the base layer to the Polkadot SDK | Accepted; amended by ADR-023 on the vault's key derivation |
| [ADR-014](ADR-014-agent-keys-authorised-not-created.md) | QOR ID authorises agent keys; it never creates them | Accepted. Amends ADR-010 |
| [ADR-015](ADR-015-infrastructure-ownership.md) | Which service owns which infrastructure concern | Accepted; clarified twice on 15 September 2026 |
| [ADR-016](ADR-016-sign-in-with-unlock.md) | Unlocking the vault signs in to QOR ID without a second approval | Accepted, under the owner's delegation. Narrows roadmap item L1.4 |
| [ADR-017](ADR-017-password-accounts-no-chain-identity-without-a-key.md) | A password-only account has no chain identity until it proves a key | Accepted. Resolves inventory Q-16; lands with the Substrate work |
| [ADR-018](ADR-018-standalone-chain-grandpa-finality.md) | A standalone chain, with its own validators and GRANDPA finality | Accepted. Resolves Q-1 |
| [ADR-019](ADR-019-aura-block-production.md) | Blocks are produced by Aura | Accepted. Resolves Q-2 |
| [ADR-020](ADR-020-permissioned-validators-now-npos-later.md) | A permissioned validator set chosen by governance now; nominated proof of stake later | Accepted. Resolves Q-3 |
| [ADR-021](ADR-021-collective-governance-now-opengov-later.md) | Governance by a collective now; a hybrid with OpenGov before mainnet | Accepted. Resolves Q-4 |
| [ADR-022](ADR-022-pin-polkadot-stable2606-1.md) | Pin the Polkadot SDK at polkadot-stable2606-1 | Accepted. Resolves Q-5 |
| [ADR-023](ADR-023-sr25519-with-ecosystem-derivation.md) | Sr25519 accounts, derived the way the ecosystem derives them | Accepted. Resolves Q-6; amends ADR-013 |
| [ADR-024](ADR-024-ss58-addresses-prefix-42-until-mainnet.md) | SS58 addresses, with prefix 42 until mainnet | Accepted. Resolves Q-7; prefix 42 on development and test networks |
| [ADR-025](ADR-025-drc369-on-pallet-nfts.md) | DRC-369 on pallet-nfts, with custom pallets for its semantics | Accepted. Resolves Q-8 |
| [ADR-026](ADR-026-agent-delegation-with-pallet-proxy.md) | Agent delegation through pallet-proxy, with custom spend caps | Accepted. Resolves Q-9 |
| [ADR-027](ADR-027-identity-stays-off-chain.md) | Identity stays off chain in QOR ID; only the account is on chain | Accepted. Resolves Q-10 |
| [ADR-028](ADR-028-provenance-from-events-archive-node-and-indexer.md) | Provenance comes from events, an archive node and an indexer | Accepted. Resolves Q-11 |
| [ADR-029](ADR-029-sponsor-pays-the-existential-deposit.md) | A sponsor pays the existential deposit, and the user then holds it | Accepted. Resolves Q-12 |
| [ADR-030](ADR-030-existential-deposit-against-a-stated-target.md) | The existential deposit is set against a stated target | Accepted as a method. Resolves Q-13; value awaits the owner |
| [ADR-031](ADR-031-no-fungible-game-items-in-the-first-release.md) | No fungible game items in the first release | Accepted. Resolves Q-14; `pallet-assets` for fractionalization only |
| [ADR-032](ADR-032-chain-location-and-names.md) | Where the new chain lives, and its crate and pallet names | Accepted. Resolves Q-15; `.cursorrules` amended; the fifth pallet is `pallet-validator-set` |
| [ADR-033](ADR-033-dependency-versions.md) | Dependency versions: the pinned SDK governs, everything else stays current | Accepted |
| [ADR-034](ADR-034-ticker-dmrg.md) | The ticker is DMRG; the name stays Creator-God Token | **Superseded** by ADR-045. Its clearance research on the project name stands, and `public-release.name-clearance` with it |
| [ADR-035](ADR-035-eighteen-decimals.md) | DMRG has eighteen decimal places, and scaled arithmetic uses the SDK's helpers | Accepted. Settles U-1 |
| [ADR-036](ADR-036-existential-deposit-100-dmrg.md) | The existential deposit is 100 DMRG | Accepted. Completes ADR-030 |
| [ADR-037](ADR-037-sudo-on-development-and-test-networks-only.md) | `pallet-sudo` on development and test networks only, absent from mainnet | Accepted |
| [ADR-038](ADR-038-ed25519-follows-the-sdk-zip-215-rule.md) | Ed25519 follows the SDK's ZIP-215 rule; R-1 met by standard behaviour | Accepted |
| [ADR-039](ADR-039-account-derivation-and-the-scheme-change.md) | Derivation paths, and what the key-scheme change does to accounts that already exist | Accepted under the owner's delegation. Completes ADR-017, ADR-023 and ADR-024; refines ADR-024's migration step |
| [ADR-040](ADR-040-the-launchers-chain-client-is-subxt.md) | The launcher's chain client is `subxt`, driven by the chain's own metadata | Accepted under the owner's delegation. Carries out L3.1, the second half of M3.4; governed by ADR-033 rule 2 |
| [ADR-041](ADR-041-multiaddress-and-accountidlookup.md) | The runtime looks accounts up with `AccountIdLookup`, and an address is a `MultiAddress` | Accepted. Resolves Q-17, closes F-Q10; amends ADR-040 decision 3, which set the launcher's address type from the runtime's old one |

| [ADR-042](ADR-042-two-domains.md) | Two domains: `qorsync.dev` for operations, `demiurge.cloud` for what people see | **Accepted**, 28 September 2026, by the project owner (proposed 20 September) |
| [ADR-043](ADR-043-qor-id-as-an-identity-provider.md) | A browser frontend signs in to QOR ID by redirect, not by holding a refresh token | **Proposed**, not accepted. Needed by ADR-011's web surfaces, M5.4 at the earliest |
| [ADR-044](ADR-044-validators-are-not-publicly-addressable.md) | Validators have no public hostname; public RPC is a separate node | **Proposed**, not accepted. Restates ADR-015's archive-node clarification |

| [ADR-045](ADR-045-the-ticker-returns-to-cgt.md) | The ticker returns to CGT | **Accepted**, by the owner's instruction. Supersedes ADR-034, and requirement R-4 in the migration inventory |

| [ADR-046](ADR-046-the-launcher-is-an-app-host.md) | The launcher hosts products as separate OS processes it spawns, supervises and signs for | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-047](ADR-047-the-object-model.md) | The object model: a BLAKE3 manifest root on chain, chunking beneath it, and a mint that pins a commit | Accepted, 22 September 2026, by the project owner, with its three owner questions answered as recommended. Carries out M2.3 |
| [ADR-048](ADR-048-interchange-formats.md) | Interchange formats, so a creator can leave with their work | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-049](ADR-049-data-ownership-and-the-user-directory.md) | A creator's work lives in their QOR ID directory, keeps working offline, and leaves in open formats | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-050](ADR-050-where-the-products-live.md) | Where each product lives, and a release gate of its own for each | **Accepted**, 28 September 2026, by the project owner (proposed 21 September) |
| [ADR-051](ADR-051-the-shells-rendering-layer-for-qfx.md) | QFX renders on one WebGL2 canvas behind the interface, under contracts the chrome owns | Accepted, 22 September 2026, by the project owner, as amended that day to fit layer one as built. Amends `DIRECTION.md` §5 and `DESIGN_SYSTEM.md` §1 and §6 |
| [ADR-052](ADR-052-assets-in-the-runtime.md) | Assets in the runtime: `pallet-nfts` configured from recorded sources, placeholder deposits, and one way in | Accepted, 22 September 2026, under the owner's delegation for engineering choices. Carries out M4.1's chain half; its deposits are placeholders under U-14 |
| [ADR-053](ADR-053-one-transaction-for-a-trade.md) | One transaction for a trade: `pallet-utility`'s atomic batch, and nothing else of it | Accepted, 22 September 2026, under the owner's delegation for engineering choices. Carries out M4.6 for L4.5; only `batch_all` is reachable, and it opens no second way to make an asset |
| [ADR-054](ADR-054-unlocking-with-windows-hello.md) | Unlocking the vault with Windows Hello: a TPM key's signature seals a second copy of the phrase, bound to the vault file | Accepted, 26 September 2026, by the project owner, who chose it over a DPAPI unlock and a plain key file. Extends ADR-016; opt-in, Windows only. **Superseded by ADR-055** |
| [ADR-055](ADR-055-the-vault-opens-with-windows-hello-alone.md) | The vault opens with Windows Hello alone: no passphrase, no password sign-in, a passphrase vault moved once | Accepted, 28 September 2026, by the project owner, who chose Hello only over a silent DPAPI unlock. Supersedes ADR-054; amends ADR-016; Windows only. **Superseded the same day by ADR-056** |
| [ADR-056](ADR-056-no-lock-screen.md) | No lock screen: the vault's key in the OS keychain, nothing asked, and QOR ID never blocks the launcher | Accepted, 28 September 2026, by the project owner. Supersedes ADR-055; amends ADR-016 |
| [ADR-057](ADR-057-eight-royalty-recipients.md) | Eight royalty recipients per asset: `MaxRoyaltyRecipients` stays 8 | Accepted, 28 September 2026, by the project owner. Answers Q-18 and ADR-047's first open item |
| [ADR-058](ADR-058-ci-on-woodpecker.md) | CI runs on Woodpecker, not GitHub Actions: a Fly.io server at `ci.qorsync.dev`, builds on the owner's computer | Accepted, 28 September 2026, by the project owner. Actions switched off; Codeberg the fallback Decision 1 superseded by ADR-059 **Superseded by ADR-063** |
| [ADR-059](ADR-059-operations-on-oracle-always-free.md) | The operations server runs on Oracle Cloud's Always Free tier: QOR ID, Postgres, Redis and the CI server behind Caddy on `qorsync.dev`, $0 | Accepted, 28 September 2026, by the project owner. Supersedes ADR-058 decision 1; departs from ADR-015 for these services; no public RPC until a private sudo key **Superseded by ADR-060** |
| [ADR-060](ADR-060-operations-on-the-owners-computer.md) | The operations stack runs on the owner's computer, published by Cloudflare Tunnel: QOR ID and Woodpecker at `id.`/`ci.qorsync.dev`, $0, no card | Accepted, 28 September 2026, under the owner's delegation after they rejected Oracle. Supersedes ADR-059; `qorsync.dev`'s DNS moves to Cloudflare **Superseded by ADR-063** |
| [ADR-061](ADR-061-royalties-and-the-settled-sale.md) | Royalties, remix royalties, and the sale settled in CGT they bind to: `pallet-drc369-royalties` | Accepted, 29 September 2026, under the owner's delegation for engineering choices. Carries out M4.2's royalty half; decision 2 superseded by ADR-062 |
| [ADR-062](ADR-062-royalty-terms-a-creator-can-correct.md) | Royalty terms a creator can correct while they hold the asset, and a remix share that never rises once the work is remixed | Accepted, 29 September 2026, by the project owner. Supersedes ADR-061 decision 2; confirms its one level of remix royalty |
| [ADR-063](ADR-063-public-repository-actions-and-railway.md) | A public repository without its history (`ALaustrup/demiurge-chain`), CI on GitHub Actions, and QOR ID with Postgres and Redis on Railway | Accepted, 29 September 2026, by the project owner. Supersedes ADR-058 and ADR-060, and ADR-015's Fly.io for these three services. Amended by ADR-064 |
| [ADR-064](ADR-064-the-repository-lives-in-the-qor-matrix-organisation.md) | The public repository, and CI with it, lives in the organisation `QOR-MATRIX` (`QOR-MATRIX/demiurge-chain`), where Actions jobs run; the personal account's are refused by a billing lock | Accepted, 1 October 2026, by the project owner. Amends ADR-063 |
| [ADR-065](ADR-065-nesting-and-the-cycle-rule.md) | Nesting in `pallet-drc369`: only the owner of both assets, depth 8 and 64 children, cycles refused by a bounded walk (R-2), and a nested asset and the asset holding it held in place | Accepted, 1 October 2026, under the delegation; choices 1, 2 and 5 are the owner's to confirm |

A record is never edited to change its decision. A later record supersedes it and both say so.

**Proposed records are not decisions.** ADR-043 and ADR-044 are written up for the owner and have not been
accepted; ADR-042 was accepted on 28 September 2026. ADR-047 and ADR-051 were accepted on 22 September 2026, and ADR-046 and ADR-048 to ADR-050 on 28 September 2026. Nothing may be built or deployed on their basis until their status says
Accepted. Two things in the tree already run ahead of ADR-046 and ADR-051, because the owner asked for them
to be built first: Qontrol's Projects surface with its helper, and QFX layer one. Each record says how the
tree differs from it.
