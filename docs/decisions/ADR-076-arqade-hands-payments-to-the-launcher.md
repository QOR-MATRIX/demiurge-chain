# ADR-076: ARQADE hands a payment to the QOR Launcher to sign (devnet, P7.4)

**Status:** **Accepted**, 5 October 2026, by the project owner: "I accept ADR-076. 100,000 CGT", the per-request cap
of decision 6. Proposed the same day. Not yet built. It is the "own
record" [ADR-069](ADR-069-arqade-the-gaming-platform.md) decision 5 requires before option A (a request handed to the
launcher) is built, for devnet play before delegated keys (M5.2) exist. It decides no economic value: the one number it
needs, decision 6's cap, is the owner's, set on acceptance for the devnet.

## Context

P7.4: a player approves a CGT payment from their own Vault, previewed and finalised, never with a key held by ARQADE.
ARQADE is a website; the Vault is in the QOR Launcher, on the player's PC (ADR-046, ADR-055). ADR-069 decision 5 weighed
three ways a website gets a Vault signature:

| | How | For | Against |
|---|---|---|---|
| **A** | ARQADE hands a request to the launcher, which shows it and signs | Works now; the key never leaves the Vault; the launcher's host dialog already exists | A browser-to-launcher hand-off is new security-critical code; only on a PC with the launcher |
| **B** | Delegated session keys with caps enforced by the chain (ADR-010, ADR-026, M5.2) | What ADR-011 says web surfaces use; no hand-off per payment | Needs `pallet-proxy`, `pallet-agent-caps` and M5.2: unbuilt |
| **C** | ARQADE runs inside the launcher (ADR-046) | Signs over the existing host protocol | Not a website any more |

ADR-069 recommended **B for production and A for devnet play before M5.2**. This record is A.

Two hand-offs were considered: a local web server in the launcher, which ADR-069 already refuses because any web page
can reach one; and a **custom URL scheme**, which opens the launcher only when the person clicks a link and lets the
launcher, not the page, decide what happens.

## Decision

1. **A `qor://pay` link.** ARQADE shows **Pay with the QOR Launcher**; the link opens the launcher with one payment
   request. The launcher registers the `qor` scheme when installed. Nothing else in the browser can reach the launcher.
2. **The request says exactly one thing:** pay *amount* CGT (integer Sparks) to *recipient* on *network*, for *what*
   (a short label), with a request id and an expiry of a few minutes. Only a plain transfer (or a round entry, once the
   ARQ Wallet's entry call exists) is accepted; any other call is refused.
3. **ARQADE's server signs the request** with a key registered for ARQADE, the way QOR ID already registers ARQADE as an
   app (ADR-073). The launcher shows **"ARQADE asks you to pay"** only when that signature checks against its list of
   known apps; any other link is shown as **"An unknown website asks you to pay"**, in a warning style, or refused. A link
   cannot claim to be ARQADE.
4. **The launcher shows it in its host dialog** (L1.4): who asks, the amount in CGT, the recipient, the network, the fee
   the chain computes, and the balance after. The player approves with Windows Hello, as for any Vault signature
   (ADR-055). The launcher signs and submits; the key never leaves the Vault, and ARQADE never sees it.
5. **ARQADE learns the result from the chain, not from the launcher.** The payment carries the request id (in a
   `system.remark` batched with the transfer); ARQADE's server watches finalised blocks for it. No reply channel from the
   launcher to the browser is needed, and a payment that did not finalise is not counted.
6. **Devnet only, with a cap per request.** The launcher refuses a `qor://pay` request for any network but Demiurge
   Devnet, and above a per-request cap of **100,000 CGT** (100,000 x 10^18 Sparks), set by the owner on acceptance.
   ARQADE never asks above it. The cap is for the devnet's test CGT; a cap for real CGT is decided with production.
7. **Expiry and single use.** A request id is paid at most once (the launcher remembers the ids it has signed; the
   chain-side check of decision 5 counts one payment per id) and not after its expiry.

## What it needs

- The launcher: register the scheme (Tauri's deep-link plugin), parse and verify a request, the dialog, sign and
  submit, remember paid ids. ARQADE: the request signer, the link, the chain watcher, and a "waiting for payment" state.
- A list of known apps in the launcher, with ARQADE's request-signing public key.
- Tests: a forged link, an expired or reused request, another network, an amount above the cap, and an unknown
  website, each refused or warned; the whole flow on the devnet.

## Consequences

- A player without the launcher (a phone, another computer) cannot pay until B exists. They can still play free games.
- Every payment needs a click in the browser and an approval in the launcher. B removes that for small amounts later.
- When B exists, `qor://pay` stays for amounts above a delegated key's caps.

## Not decided here

Real CGT, paid entry with prizes (U-16) and the legal review.
