# ADR-112: The controller's own key signs every agent authorisation

**Status:** **Accepted**, 22 October 2026, under the owner's delegation of engineering choices (`HANDOFF.md`, the
owner's direction of 13 September 2026), when the owner asked to "proceed with all recommended steps moving forward"
and L5.1 was the next of them. The owner may amend it. It carries out **L5.1** in `docs/DIRECTION.md` and closes the
known gap [ADR-014](ADR-014-agent-keys-authorised-not-created.md) recorded. It decides **no economic value**.

**Relates to:** [ADR-014](ADR-014-agent-keys-authorised-not-created.md) (an agent's key is its own; QOR ID authorises
it), [ADR-016](ADR-016-sign-in-with-unlock.md) and
[ADR-017](ADR-017-password-accounts-no-chain-identity-without-a-key.md) (a QOR ID proves a key by signing a challenge,
and may have none), [ADR-076](ADR-076-arqade-hands-payments-to-the-launcher.md) and
[ADR-077](ADR-077-corrections-to-adr-076.md) (a `qor://` link checked by the host and approved in the host's own
dialog), [ADR-075](ADR-075-one-name-per-qor-id.md) (one name per QOR ID), security track item 7 (domain-separated
signatures).

## Context

ADR-014 made an agent prove its own key: `POST /api/v1/agents/register` takes the agent's account, a single-use
challenge issued for it, and the agent key's signature over that challenge. The controller, though, was whoever the
access token belonged to. Anyone holding a person's access token could make an agent act for that person, and widen any
of their agents' capabilities (`PUT /api/v1/agents/{did}/capabilities`) the same way. ADR-014 called this a known gap,
not a design choice; L5.1 closes it, and the Beta gate counts L5.1.

A person's key lives in the launcher's vault, which signs only after the person approves in a dialog the host draws
(L1.4). Nothing else holds it.

The first tests of agent registration, written for this record, found that its name rule did not match QOR IDs: a name
of up to 32 characters of any kind was taken, while an agent's QOR ID (`agent_` and the name) must be a QOR ID like any
other, 3 to 20 letters, digits and underscores. A longer name failed in the database; one with other characters was
stored.

## Decision

1. **Registering an agent needs the controller's own signature.** QOR ID verifies it, by Sr25519, against the chain
   account the controller's QOR ID has proved (`chain_account_id`, set at key sign-in or by linking a key). A QOR ID
   that has proved no key cannot authorise an agent until it proves one (ADR-017 allows it to exist without one).
2. **What the controller signs is a statement, one field per line, nothing left out:**

   ```
   demiurge:qor-id:agent-authorisation:v1
   controller:<the controller's account, 64 lowercase hex>
   agent:<the agent's account, 64 lowercase hex>
   challenge:<the challenge the agent answered>
   name:<the agent's name as asked>
   capabilities:<sorted, comma-separated>
   autonomy:<supervised | bounded | autonomous | sovereign>
   spending_limit:<whole CGT, or empty>
   model:<as asked, or empty>
   ```

   Every field QOR ID records is in it, so none can be changed after the controller saw it. The agent's single-use
   challenge is in it, so the signature authorises one registration and is spent with that challenge. Its domain is not
   the challenge's (`demiurge:qor-id:challenge:v1:`), so neither signature can be offered as the other, nor as a
   transaction. A field holding a line break or another control character is refused, and so is a capability named
   twice, so no field can pose as another.
3. **Changing an agent's capabilities needs the same.** The controller asks QOR ID for a challenge for its own account
   and signs

   ```
   demiurge:qor-id:agent-change:v1
   controller:<the controller's account, 64 lowercase hex>
   agent:<the agent's DID>
   challenge:<that challenge>
   capabilities:<sorted, comma-separated>
   ```

   and QOR ID spends the challenge. Switching an agent off still needs only the session: it takes authority away, and a
   stolen token can do no more with it than stop an agent, which then registers a new key.
4. **An agent's name makes a QOR ID like any other:** 3 to 14 letters, digits, spaces or underscores, so that `agent_`
   and the name, lowercased with spaces as underscores, is a valid QOR ID. Anything else is refused in words.
5. **The launcher signs it, after asking.** An agent hands its request to the launcher as a link,
   `qor://agent?r=<hex of a JSON object>` with `account_id`, `challenge`, `signature` (the agent key's, over the
   challenge under QOR ID's tag), `name`, `capabilities`, `autonomy`, and optionally `spending_limit` and `model`. The
   host checks all of it before anyone is asked: the agent's signature verifies; the challenge has QOR ID's shape and is
   no more than ten minutes old (QOR ID keeps one for five; the rest allows for the PC's clock); every capability is one
   QOR ID knows, named once; the autonomy is one of four; a bounded agent names a limit and no limit is negative; the
   name makes a QOR ID; the model is one short line; a profile is signed in, its QOR ID has proved a key, and that key
   is the open wallet's account. It then shows its own dialog, built from the checked values: the QOR ID it acts for,
   the agent's QOR ID and account, what it may do, its autonomy and limit, and that nothing enforces them yet. On
   approval the profile's vault signs the statement and the host registers the agent with QOR ID itself, with the
   profile's session. The interface can neither sign a statement nor choose its contents.
6. **Both flows join the log check** (`services/qor-auth/src/log_hygiene.rs`, AGENTS.md §9): the agent's account, its
   signature and every controller signature are among the values it looks for.
7. **Not this record:** listing, limiting and revoking agents from the launcher, and the launcher's side of changing
   capabilities (L5.2); enforcing capabilities and limits on chain (M5.2, Q-9).

## Consequences

- An access token alone can no longer make or widen an agent, once the QOR ID that carries this is deployed. SECURITY.md
  and HANDOFF §2 say so until then. L5.1 is ticked when a person has authorised an agent from an installed launcher
  against the deployed QOR ID.
- Registration changes shape again: clients send `controller_signature`. No active client registered agents before
  this record.
- A QOR ID with no proved key cannot have agents until it signs in with a key or links one.
- Capabilities cannot be changed from any client until L5.2 gives the launcher a surface for it; the API is ready.
- QOR ID and the launcher each build the statement; a test in each holds it to one vector, and a launcher test reads
  QOR ID's source for its capability list, domain and name rule, so the two cannot drift apart unseen.
