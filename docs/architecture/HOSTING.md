# Hosting and operations

Updated 13 November 2026 for [ADR-130](../decisions/ADR-130-qorsync-is-the-public-domain.md).
QORSYNC.DEV is the only public first-party domain; service subdomains remain approved.
This guide records source ownership and dated read-only checks, not an uptime guarantee.

## 1. Inventory: what will need hosting, and when

The homepage is maintained in QOR OS `site/` (ADR-132); this copy is frozen.
The active network sources here are `chain/` (network), `services/qor-auth/`
(identity), `services/social/` (Social), and `products/arqade/app/` (ARQADE's web
surface; `products/arqade/` is its application root). QOR OS is developed
in [QOR-MATRIX/QOR-OS](https://github.com/QOR-MATRIX/QOR-OS), a private repository.
Qt product development follows ADR-126. Frozen `apps/`, `sdk/`, `packages/`,
`cli/` and `client/` are not deployment candidates or current API guidance.

## 2. Domain map

| Purpose | Source address | Verification boundary |
| --- | --- | --- |
| Homepage | `https://qorsync.dev` | Serves QOR-MATRIX/QOR-OS main via the existing Astra Matrix / qor project; QOR OS hosting receipts record the deployed revision |
| Homepage alias | `https://www.qorsync.dev` | Vercel returned a verified redirect to the apex |
| QOR ID | `https://id.qorsync.dev/api/v1` | `/health` and `/ready` returned HTTP 200 at 22:33 UTC; the service reports healthy and its database/cache ready. Authentication flows were not exercised |
| Chain RPC | `wss://rpc.qorsync.dev` | At 22:34 UTC, read-only WebSocket RPC identifies Demiurge Devnet, runtime 11, and advancing finalized headers over 12 seconds |
| Social and ARQADE | Approved subdomains under QORSYNC.DEV required | Existing provider-host defaults/links require a coordinated migration; replacement hosts are not yet verified |

The owner supplied `https://vercel.com/astramatrix/qor`. Authenticated CLI access
reaches existing project `prj_pUNenZCSC3eh2tyJdHbHCt9rUlW0` in team `astramatrix`,
despite the connector's repeated 404. Its Git connection is now QOR-MATRIX/QOR-OS,
repository ID `1435244138`, branch `main`, root `site`. Node 24 installs from the
lock with scripts disabled, builds public assets and serves `dist`.

QOR OS PR #8 merged at `1dc617b`; the production alias was verified against that
exact Git source. HTTPS, canonical repository links, security headers, the `www`
308 redirect, private-file exclusion and Analytics script delivery pass. Public
DNS remains proxied through Cloudflare; its automatic beacon injection prompted
QOR OS's no-transform header correction. The canonical
[hosting guide](https://github.com/QOR-MATRIX/QOR-OS/blob/main/docs/HOSTING.md)
owns final deployment and delivery evidence, under session-08. Analytics dashboard
collection remains separate. No DNS transfer, replacement project, credential
copy or network-service deployment was needed for this homepage alignment.

The RPC probe used only `system_chain`, `state_getRuntimeVersion`,
`chain_getFinalizedHead` and `chain_getHeader`. It submitted no transaction.
No account was created and no sign-in or email flow was exercised. Receipts are
in QOR OS `build/development-2026-11-13/session-06/live-*.json`.


## 3. Cross-domain authentication

Retain service subdomains. QOR ID's request validation, origin allowlists, bearer
tokens and trusted client approvals remain separate security boundaries. A domain
rename does not authorise forwarding stored credentials or bypassing confirmation.
Stable keychain/application identifiers and cryptographic separators are unchanged.
Installed settings and released clients require a reviewed migration after service
hosts pass acceptance; do not silently reinterpret legacy protocol requests.

## 4. Validators and public RPC: the topology

Follow the current `chain/` configuration and the deployment owner's evidence.
Do not expose validator administration or rebuild the frozen custom protocol.
Changing hostnames does not prove consensus, finality, runtime compatibility or
public readiness. The roadmap and protocol document retain those requirements.

## 5. Email: where it should send from

An approved sender beneath QORSYNC.DEV must be verified in the mail provider before
deployment. No replacement mailbox or sender was created or tested in this session.
Never copy mail credentials into the repository or logs. Preserve unrelated mail,
verification and DNS records while planning any host change.

## 6. Cost

No hosting plan, purchase, account transfer or resource change was performed.
The earlier provider/cost observations are historical and must be rechecked before
being used in an operational decision.

## 7. `.dev` is HSTS-preloaded: every name needs real HTTPS

Every published host needs valid TLS. Verify the certificate, intended service,
origin policy and protocol response before updating a client's compiled defaults.
A successful DNS lookup or an HTTP response alone is insufficient acceptance.

## 8. What the owner has to decide

The domain choice is settled by ADR-130. Remaining work is implementation and
verification: retain verified homepage access, confirm the remaining DNS/service
projects, bind and test the
service subdomains, verify mail sending, update clients and validate their upgrade
path, then publish only with the owner's release/deployment instruction.

The legacy `scripts/deploy*`, `setup-ssl*`, `setup-rpc-ssl*`, `setup-email*` and
`wipe-and-redeploy*` helpers touched by domain cleanup now refuse before any host
side effects. The old `config/production/` and `config/nginx/n8n.conf` files are
marked retired templates, not current deployment configuration. Never use them
to recreate the old stack on the new domain.
