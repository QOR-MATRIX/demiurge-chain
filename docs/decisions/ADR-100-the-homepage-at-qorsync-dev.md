# ADR-100: The project's homepage at qorsync.dev, written for search engines and AI crawlers

**Status:** **Accepted**, 18 October 2026, by the owner. The owner asked for the project to be found and described
accurately in search ("Currently a forked version of the project is being displayed when searched, id like to make sure
that information is consistant with this project as closely as possible"), chose `qorsync.dev` as the one official
homepage with the other domains redirecting to it, chose that the assistant build and publish it, and asked for "a high
quality, ancient futuristic style with animations and effects mouse-over hover and reactive background. Premium only
look and feel." The design and the names below are the assistant's, under that instruction. It decides **no economic
value**. **Decision 5, the look, is amended by [ADR-108](ADR-108-one-look-obsidian-and-gilt-in-two-registers.md)**
(21 October 2026): obsidian and gilt become every surface's look, from one source, and the page's faces are served
from its own folder rather than a font service. **Decisions 3, 4, 5 and 7 are amended by [ADR-123](ADR-123-the-homepage-leads-with-qor-os.md)** (29 October 2026): the page leads with QOR OS, under QOR OS's mark. **Brings into scope:** `site/` (AGENTS.md §4).

## Context

Searching for the project found an older copy of it, `AuthorPrime/Demiurge-Blockchain`: the owner's brother's public
repository, describing a design from before the realignment and linking to `demiurge.cloud`. The owner does not want it
removed; he wants the current project portrayed accurately beside it.

On 18 October 2026 the project had no page of its own for a search engine to find. `qorsync.dev`, `demiurge.cloud` and
`demiurge.guru`, all the owner's and all pointed at Vercel, answered 404. The repository `QOR-MATRIX/demiurge-chain` had no
description or topics, and its website field named a Vercel address that answered 404. The owner's other copies
(`Astra-Matrix/DEMIURGE-PROTOCOL`, `ALaustrup/demiurge-chain-old`, `ALaustrup/qor_qloud_demiurge`) are private and not
indexed.

## Decision

1. **`https://qorsync.dev/` is the canonical homepage.** `www.qorsync.dev`, `demiurge.cloud`, `www.demiurge.cloud`,
   `demiurge.guru` and `www.demiurge.guru` redirect to it permanently, so links to the older copy's homepage arrive at
   the current project.
2. **It is a static page in `site/`**, with no framework and no build: `index.html`, `styles.css`, `main.js` and the
   files crawlers read. It is deployed by a Vercel project in the owner's team (`astramatrix`), connected to this
   repository with `site/` as its root, so a merge to `main` publishes it.
3. **Everything a search engine or AI crawler reads is there and true:** a title and description, a canonical address,
   Open Graph and Twitter cards with a share image, structured data (schema.org `Organization`, `WebSite`,
   `SoftwareSourceCode`, `SoftwareApplication` for the QOR Launcher and `WebApplication` for ARQADE) naming the current
   repository, `robots.txt`, `sitemap.xml`, `llms.txt`, a 404 page, a web manifest, `security.txt`, and an IndexNow key
   so Bing and others can be told of changes at once.
4. **Its words come from current documents only** (the README, `DIRECTION.md`, `CGT.md`, `SYSTEMS.md`) and from services
   checked when it was written, follow ADR-008's language, and say plainly that the network is a test network and that
   nothing holds real value. It states that `QOR-MATRIX/demiurge-chain` is the only current repository and that earlier
   copies describe a design it no longer follows, without naming or linking any copy.
5. **The look:** obsidian and gilt with the launcher's ember as the one accent, carved Roman capitals (Cinzel) over
   condensed modern type (Barlow), engraved cards that tilt and light under the pointer, and a canvas behind the page: a
   hexagonal lattice that lights around the pointer, an astrolabe of rings, ticks, a triangle and a square turning
   slowly, and rising embers. Every word is HTML; the canvas is decoration. Reduced motion draws one still frame; a
   hidden tab draws nothing; the first screen is complete at rest.
6. **The repository's own page carries the same facts:** its description, topics and website field are set, and the
   README is brought up to date, since GitHub's page is what a search engine shows most often.
7. **`site/scripts/check-site.mjs`** holds the page to points 3 and 4 (26 checks); `site/scripts/render-images.mjs` renders
   the share image and icons.

**What it does not do:** no analytics, cookies or trackers. Verifying the domain in Google Search Console and Bing
Webmaster Tools, and asking them to re-crawl, need the owner's accounts.

## Consequences

- A new release of the launcher changes three places in `site/index.html` (the download link, the button's note and
  `softwareVersion`) and `site/sitemap.xml`'s date.
- Search engines take days to weeks to re-crawl. The older copy stays in results; the current project should rank beside
  it for its own names once the homepage and the repository page are indexed.
