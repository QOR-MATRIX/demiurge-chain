# Where the network and Launcher stand

For Andrew, 13 November 2026. Source work is on `session/qorsync-domains`, based
on upstream `74a1014`; it is not a release or deployed update.

## Domain and developer access

QORSYNC.DEV is the only public domain, with service subdomains retained on your
instruction. ADR-130, the numbered decision, replaces the old split. Retired host
literals were removed from current tracked text, and historical redactions are
labelled. Stable application and credential-store identifiers remain unchanged.

Launcher Developers now points to QOR OS's guide, with repository access required.
Six frozen web tiles no longer offer dead destinations. Old clients remain
unsupported; their obsolete network examples use loopback, this machine only.
Old deployment helpers refuse before changing a host. No new framework was added.

Vercel returned verified domain assignments for the homepage and its `www`
redirect. DNS, the domain routing records, uses Cloudflare. The connected Vercel
account cannot open the assigned project, so this is not a live-site acceptance.
Social and ARQADE still need their service hosts migrated and tested; installed
settings, mail and released clients need separate verification. None was changed.

## Security and next work

QOR OS's core dependency audit found four vulnerability advisories and 13 warnings
in versions still locked here. The affected libraries are libsqlite3-sys,
quinn-proto and quinn-udp. The UDP issue excludes Linux and Windows; other
reachability still needs review. No advisory was ignored and no exploit is claimed.

The core bridge, the shell's command interface, still has only four version/amount
commands. Remediate dependencies before extending Launcher/DRC-369 asset and agent
capabilities. QOR OS owns native product development; this repository owns the core,
network and Launcher. No key, payment, runtime or consensus behavior changed.

Source-policy, Launcher-link and homepage validation are recorded in the handoff.
Full native builds, service acceptance and a replacement release are separate gates.
See `docs/DIRECTION.md` for the plan and `docs/SYSTEMS.md` for verified source routes.
