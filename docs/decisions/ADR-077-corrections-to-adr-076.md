# ADR-077: Three corrections to ADR-076, from the launcher as it is

**Status:** Accepted, 5 October 2026. Decision 1 is the owner's choice, given when asked ("Approval dialog"); decisions
2 and 3 correct facts. It amends [ADR-076](ADR-076-arqade-hands-payments-to-the-launcher.md), which was accepted the
same day and is not built; ADR-076 says so. It decides no economic value.

## Context

ADR-076 was written before the launcher's signing code was read for it. Reading it (`src-tauri/src/lib.rs` `Prompt`,
`Confirm`, `HostDialog`; `src-tauri/src/vault/mod.rs` `Vault::sign`; `src-tauri/src/chain/`) showed three statements in
ADR-076 that do not match the launcher:

- ADR-076 decision 4 says the player approves with Windows Hello "as for any Vault signature (ADR-055)". Since
  [ADR-056](ADR-056-no-lock-screen.md) no signature uses Windows Hello: every send, buy and trade is approved in the
  host dialog, a native window the webview cannot draw. Hello is read only to move an old vault to the keychain.
- Decision 4 also says the dialog shows "the fee the chain computes". The runtime has no transaction-payment pallet
  (OPEN-4); there is no fee to compute.
- Decision 6 limits requests to Demiurge Devnet, but the launcher identifies a network only by `system_chain`, its name,
  which any node can report.

## Decision

1. **A `qor://pay` payment is approved in the launcher's host dialog**, exactly as every other signature is (ADR-056),
   not with Windows Hello. The owner chose this over adding a Windows Hello prompt for these payments alone.
2. **The dialog says there is no fee** while the chain charges none, and shows the balance after the payment; when a fee
   exists (OPEN-4), it shows the fee the chain reports.
3. **"Devnet only" is checked by genesis hash**: the launcher refuses a request unless the connected chain's genesis is
   Demiurge Devnet's, `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a` (ADR-068). The chain's name
   is not trusted for this.

ADR-076 is otherwise unchanged.
