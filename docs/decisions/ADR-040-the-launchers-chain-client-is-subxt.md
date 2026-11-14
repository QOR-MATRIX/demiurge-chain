# ADR-040: The launcher's chain client is `subxt`, driven by the chain's own metadata

> Editorial note, 13 November 2026: retired host/contact literals are redacted. Historical claims retain their original dates and are not current deployment evidence. ADR-130 establishes QORSYNC.DEV as the only public domain.

**Status:** Accepted, 20 September 2026.
**Accepted under the owner's standing delegation for engineering choices.** Nothing here is a value
[`OPEN_QUESTIONS.md`](../economics/OPEN_QUESTIONS.md) leaves open, and nothing here is reserved to the owner. No
pallet is named, no economic number is invented, and no gate or check is loosened.
**Carries out:** roadmap item **L3.1**, the second half of **M3.4**, whose first half was
[ADR-039](ADR-039-account-derivation-and-the-scheme-change.md).
**Governed by:** [ADR-033](ADR-033-dependency-versions.md) rule 2 — `subxt` is not published as part of the pinned
SDK release, so it is a dependency that stays current rather than one the pin dictates.
**Depends on:** [ADR-013](ADR-013-substrate-polkadot-sdk.md) (the chain is Substrate),
[ADR-022](ADR-022-pin-polkadot-stable2606-1.md) (the pinned release),
[ADR-023](ADR-023-sr25519-with-ecosystem-derivation.md) (Sr25519 account keys),
[ADR-024](ADR-024-ss58-addresses-prefix-42-until-mainnet.md) (SS58 addresses),
[ADR-028](ADR-028-provenance-from-events-archive-node-and-indexer.md) (history comes from an indexer).

## Context

The launcher's chain client speaks the custom devnet's private RPC vocabulary — `chain_getBlockNumber`,
`balances_getBalance`, `account_getTransactionNonce`, `chain_getTransactionHistory` — and builds transactions in a
shape only that devnet accepts. The Substrate chain in `chain/` answers none of those methods.

ADR-039 moved the vault to Sr25519 and SS58 on 19 September 2026, which is the half of M3.4 that concerns keys. It
left the two state-changing calls refusing outright, because a vault that signs Sr25519 cannot produce anything the
Ed25519-only devnet will accept. This record is the other half: the client.

The roadmap states the requirement as "transactions built and signed from runtime metadata". Three ways to meet it
were considered.

1. **`subxt`**, the ecosystem's Rust client for Substrate chains. It reads the runtime's metadata and builds calls,
   storage keys and the signer payload from it.
2. **A hand-written client** on `parity-scale-codec`, `frame-metadata` and `sp-core`: decode metadata, find the call
   index, encode the call, assemble the transaction extensions, produce the signer payload, SCALE-encode the
   extrinsic.
3. **Depending on the runtime crate itself** (`demiurge-runtime`) for compile-time call types.

Option 3 was dismissed first. It couples a desktop application's build to the runtime's, including its wasm build,
and it makes the launcher unable to talk to a chain whose runtime has been upgraded past the version it was compiled
against — the precise thing metadata exists to avoid.

Option 2 is a re-implementation of what option 1 already does, in the one part of the system where a mistake is
quiet: a signer payload assembled slightly wrong produces a signature that verifies against nothing, and a payload
assembled wrong in a way the node *accepts* is worse. AGENTS.md §7 asks for a written reason before departing from a
standard component; there is no such reason here.

## What was verified, on 20 September 2026, before deciding

None of this is taken from documentation. Each was run in this repository, against
`chain/target/release/demiurge-node.exe --dev` on port 9955.

1. **Versions resolve with no duplicates and no conflict.** `sp-core =43.0.0`, the version ADR-033 rule 1 pins the
   launcher to, and `subxt 0.51.0` resolve together to a single copy of every crate they share:
   `parity-scale-codec 3.7.5`, `scale-info 2.11.6`, `primitive-types 0.13.1`, `schnorrkel 0.11.5`, and
   **`frame-metadata 23.0.1`, the same version `chain/` builds against.** `subxt` depends on no Polkadot SDK crate,
   so it cannot drag the pin forward, which is what ADR-033 rule 1 exists to prevent.
2. **The runtime's transaction extensions are accepted.** `chain/`'s `TxExtension` is `CheckNonZeroSender`,
   `CheckSpecVersion`, `CheckTxVersion`, `CheckGenesis`, `CheckEra`, `CheckNonce`, `CheckWeight`. `subxt` resolves
   them from metadata and builds a valid payload; the two it does not name explicitly contribute nothing to the
   signed payload.
3. **A transfer built from metadata and signed by an `sp-core` Sr25519 pair was accepted and included.** Alice's
   nonce went from 0 to 1, read back with `system_accountNextIndex`.
4. **The nonce must come from `system_accountNextIndex`, not from `subxt`'s default.** `subxt`'s
   `at_current_block()` pins the **latest finalized** block and reads the nonce there, so a second transaction
   submitted before the first finalises is signed with a nonce already spent, and the node refuses it as
   "Transaction is outdated". Observed, then fixed by supplying the nonce explicitly: two transfers back to back
   were accepted, at nonces 1 and 2.
5. **`signer_payload()` already applies Substrate's "hash it if it is longer than 256 bytes" rule.** A
   `System.remark` of 600 bytes produced a 32-byte signer payload
   (`frame-decode-0.18.1/src/methods/extrinsic_encoder.rs:515`). The vault therefore signs those bytes verbatim, and
   must not hash them again.
6. **The transport is WebSocket only.** `subxt-rpcs`'s jsonrpsee client builds a WebSocket transport and nothing
   else, so the launcher's chain endpoint changes scheme.

## Decision

1. **The launcher's chain client is `subxt`**, at the latest release that works with the pinned runtime — `0.51.0`
   on 20 September 2026 — under ADR-033 rule 2, upgraded like the launcher's other dependencies rather than pinned
   to the SDK.

2. **Calls, storage and constants are addressed dynamically, from the metadata the connected node serves.** No
   generated interface is checked in and no metadata file is committed. A runtime upgrade that adds a call does not
   require a launcher release, and a launcher pointed at a chain that lacks a call it asks for fails with the node's
   own answer rather than with a stale generated type.

3. **The launcher declares its own `subxt::Config` rather than using `SubstrateConfig` or `PolkadotConfig`, because
   the runtime uses `IdentityLookup`.** `chain/`'s extrinsic address is a plain `AccountId32`, not the
   `MultiAddress` the Polkadot relay chain uses, and its calls take a bare account for `dest`. The launcher's
   `Config` therefore sets `Address = AccountId32`. Everything else follows the Substrate defaults:
   `MultiSignature`, `BlakeTwo256`, and the default transaction extensions resolved from metadata.
   - The runtime's choice of `IdentityLookup` is not changed here. It is the minimal template's default and works
     with Polkadot.js and Talisman, which both read the address type from metadata. Whether the chain should use
     `MultiAddress` before mainnet is a chain-side question, recorded in
     [`MIGRATION_INVENTORY.md`](../architecture/MIGRATION_INVENTORY.md), not a launcher one.

4. **The vault signs; the key does not leave it, and nothing is signed before the person has approved it.** The
   client uses `create_signable`, reads `signer_payload()`, and passes those bytes to `Vault::sign`, which draws the
   host dialog (L1.4) and signs inside the vault's lock. The signature is attached with
   `sign_with_account_and_signature`. `subxt`'s own `Signer` trait is deliberately **not** implemented: it is
   synchronous and infallible, so a vault that has locked, or a person who declines, would have no way to say so,
   and a meaningless signature would be submitted.

5. **The nonce comes from `system_accountNextIndex`**, which counts the pool as well as the chain, and is supplied
   explicitly for every transaction. Finding 4 above is the reason, and a test pins it.

6. **Transfers use `Balances.transfer_keep_alive`.** The existential deposit is 100 DMRG (ADR-036); a transfer that
   silently reaps the sender's account is not something a wallet should be able to do by accident. An interface for
   deliberately emptying an account is a later, separate decision.

7. **A transfer is reported only once it is finalised.** The client submits and watches, and waits for the
   transaction to be finalised and to have succeeded, under a timeout. The devnet reported every transaction as
   finalized regardless (`HANDOFF.md` §2.0); this chain has GRANDPA (ADR-018), so the launcher says finalised only
   when it is. A timeout is reported as a timeout, naming the transaction hash, and never as a failure, because a
   transaction that has been submitted may still finalise and must not be sent twice.

8. **The chain endpoint is a WebSocket address**, `ws://` or `wss://`. The compiled-in defaults become
   `retired-public-host` and `ws://127.0.0.1:9944`. A stored `http://` or `https://` endpoint from an earlier
   version is upgraded in place to `ws://` or `wss://` when settings are read, rather than failing, since it names
   the same node on the same port.

9. **Two calls keep refusing, with accurate reasons.**
   - **Transaction history.** The Substrate chain has no history RPC, by design: provenance comes from events, an
     archive node and an indexer (ADR-028), and that indexer does not exist. The client says so. It does not
     synthesise a history by walking blocks, which would be slow, incomplete and quietly wrong.
   - **The starter claim.** There is no issuance mechanism, the issuance rate is OPEN-1 and the genesis split is
     OPEN-2, and AGENTS.md §5 forbids adding any path that creates DMRG outside `--dev`. The claim waits on the
     economics, not on the client.

10. **The custom devnet's RPC vocabulary leaves the launcher entirely.** `chain_getBlockNumber`,
    `balances_getBalance`, `account_getNonce`, `account_getTransactionNonce` and `chain_getTransactionHistory` are
    gone, along with the separate "request nonce" and "transaction nonce" that devnet distinguished (D-004). A
    Substrate account has one nonce.

11. **The launcher reports which chain answered.** `system_chain` is read on connection and shown in the chain
    status. `HANDOFF.md` §2.0 records that the two chains in this repository share port 9944 and that `system_chain`
    is what tells them apart; the launcher now does that automatically, instead of leaving it to a person with
    `curl`.

## Consequences

- **The launcher's minimum Rust version rises to 1.88**, which is `subxt 0.51.0`'s. The manifest's `rust-version` is
  updated to match. The toolchain in use is 1.98.1 and CI installs stable, so nothing else moves.
- **`framework/` is not touched.** It keeps its own untrusted devnet and its own tests (AGENTS.md §7). The launcher
  simply no longer talks to it, which is part of what retiring it at M3.5 means.
- **The Chain view in the launcher shows a chain name and a finalised height** it could not show before.
- **A person can now be shown, and refuse, a real transfer.** L1.4's dialog was implemented and unit-tested but had
  nothing to guard once signing was refused outright; the transfer path restores its subject.
- **What still needs a running launcher and a person** is unchanged in kind: approving and declining a real transfer
  against a local node. The host tests cover construction, refusal, endpoint handling and the nonce rule; they
  cannot cover a native dialog.
- **This record does not open the chain to anyone.** Nothing is deployed, `retired-public-host` resolves to
  nothing, and the devnet in `framework/` is still never exposed.
