# Hosting and operations

Updated 13 November 2026 for [ADR-130](../decisions/ADR-130-qorsync-is-the-public-domain.md).
QORSYNC.DEV is the only public first-party domain; service subdomains remain approved.
This guide describes source ownership and verified configuration, not a service uptime claim.

## 1. Inventory: what will need hosting, and when

The active sources are `site/` (homepage), `chain/` (network), `services/qor-auth/`
(identity), `services/social/` (Social), and `products/arqade/app/` (ARQADE's web
surface; `products/arqade/` is its application root). QOR OS is developed
in [QOR-MATRIX/QOR-OS](https://github.com/QOR-MATRIX/QOR-OS), a private repository.
Qt product development follows ADR-126. Frozen `apps/`, `sdk/`, `packages/`,
`cli/` and `client/` are not deployment candidates or current API guidance.

## 2. Domain map

| Purpose | Source address | Verification boundary |
| --- | --- | --- |
| Homepage | `https://qorsync.dev` | Vercel returned a verified project-domain assignment on 13 November; the project could not be opened through the connected account |
| Homepage alias | `https://www.qorsync.dev` | Vercel returned a verified redirect to the apex |
| QOR ID | `https://id.qorsync.dev/api/v1` | Core's `DEFAULT_AUTH`; an HTTP response at the host root does not prove API health |
| Chain RPC | `wss://rpc.qorsync.dev` | Core's `DEFAULT_RPC`; a WebSocket service requires a protocol check, not an HTTP root probe |
| Social and ARQADE | Approved subdomains under QORSYNC.DEV required | Existing provider-host defaults/links require a coordinated migration; replacement hosts are not yet verified |

Vercel reports Cloudflare nameservers, also confirmed through public DNS. The
connected Vercel account lists no accessible projects, although the domain API
retains the apex/`www` assignment. The project detail/domain endpoints return
404. Do not infer ownership transfer, deletion or a running deployment from that.
Confirm the correct project access and DNS account before changing routing.

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
verification: obtain access to the actual hosting/DNS projects, bind and test the
service subdomains, verify mail sending, update clients and validate their upgrade
path, then publish only with the owner's release/deployment instruction.

The legacy `scripts/deploy*`, `setup-ssl*`, `setup-rpc-ssl*`, `setup-email*` and
`wipe-and-redeploy*` helpers touched by domain cleanup now refuse before any host
side effects. The old `config/production/` and `config/nginx/n8n.conf` files are
marked retired templates, not current deployment configuration. Never use them
to recreate the old stack on the new domain.
