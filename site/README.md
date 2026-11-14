# qorsync.dev

The project's homepage (ADR-100): one static page, written so that search engines and AI crawlers describe the project as
it is now. Since ADR-123 it leads with QOR OS, the owner's operating system, with Demiurge's network beneath it, and since
ADR-127 it offers QOR Mobile and the QOR Launcher for download and is designed first for the phone. It remains a
static site without a framework. ADR-131 adds Vercel Web Analytics and a small asset-copy build.

| File | What it is |
| --- | --- |
| `index.html` | The page: every word is HTML, with the title, description, canonical address, share cards and structured data in its head |
| `styles.css`, `main.js` | The look (obsidian, gilt and ember, with QOR OS's mark), the phone's menu sheet, and the living background (ADR-127): one WebGL shader, the mark as a lens that bends a drifting nebula and its stars, a tilted burning disc whose far side arcs over the ring, the mark's tail orbiting, and a small mass under the finger or pointer, with a ring of light at each tap. Reduced motion draws one still frame, a hidden tab nothing, an untouched phone half as often; the resolution drops when frames come late; without WebGL the painted ground stays |
| `robots.txt`, `sitemap.xml`, `llms.txt` | What crawlers read: crawl everything, the one address, and a plain summary for AI search |
| `og.png`, `icon-*.png`, `favicon.svg` | The share image and icons, rendered by `scripts/render-images.mjs` from `scripts/og.html` and `favicon.svg` |
| `<32 hex digits>.txt` | The IndexNow key, public by design: it proves to Bing and others that a change notice comes from this site |
| `404.html`, `site.webmanifest`, `.well-known/security.txt`, `vercel.json` | The not-found page, the manifest, where to report a vulnerability, and the headers |
| `analytics.mjs`, `package.json`, `package-lock.json`, `scripts/build.mjs` | Homepage-only analytics, its exact dependency and integrity lock, and the public asset build; generated `dist/` and `node_modules/` stay untracked |

## Checking it

```sh
npm ci --prefix site --ignore-scripts  # Node 24; do not install the frozen root workspace
npm run build --prefix site           # 41 public files, including the SDK and its MIT licence
npm run check --prefix site           # 33 site checks and 6 analytics/build tests
npm audit --prefix site --audit-level=low
node site/scripts/render-images.mjs     # after changing og.html or favicon.svg (needs Edge or Chrome)
```

To look at it, serve `site/dist/` with a static server and open it at a phone's width
as well as a desktop's. The build copies an explicit public asset list and the
package's unmodified, self-contained browser module with its licence. It does not
publish scripts, manifests or dependencies. No lifecycle scripts run during install.

## Analytics

The owner requested `@vercel/analytics`; version 2.0.1 is pinned with its integrity
in the lockfile. `analytics.mjs` calls its plain JavaScript `inject` function once
on `https://qorsync.dev/`. Local and preview origins, the redirect alias and other
paths do not initialise it. Browser Do Not Track and Global Privacy Control
opt-outs prevent initialisation. Only homepage pageviews are accepted: the
`beforeSend` callback removes query strings, fragments and extra fields, and
refuses custom events and other paths/origins. There is no account integration.
The script and collection endpoints are same-origin; the content security policy
still permits only same-origin scripts and connections. Referrer policy is
`no-referrer`, and development event logging is disabled.

This is source integration, not proof of live collection. Enable Web Analytics
on the verified Vercel homepage project and deploy only with the owner's publication
instruction, then verify the script endpoint and the dashboard. Follow Vercel's
[quickstart](https://vercel.com/docs/analytics/quickstart) and
[data-redaction guide](https://vercel.com/docs/analytics/redacting-sensitive-data).
The hosted homepage job checks installation, build, privacy behaviour and npm audit;
Dependabot proposes reviewed package updates weekly.

## Publishing

The canonical address is `https://qorsync.dev`. On 13 November Vercel returned
verified apex and `www` assignments, with `www` redirecting to the apex, and
Cloudflare nameservers. The connected account could not open the assigned project
(404), including after the owner approved renewed team access. The supplied
`astramatrix/qor` project URL remains inaccessible; the domain API binds apex and
`www` to the same project. The old `demiurge-site` automatic-deployment claim is
not current production evidence.
Verify the project, source branch, domain bindings and DNS before any publication.
No retired public domain should be reattached. See ADR-130 and
[`HOSTING.md`](../docs/architecture/HOSTING.md).
The project root must be `site/`. Its checked-in `vercel.json` uses `npm ci
--ignore-scripts --no-fund`, `npm run build`, and output directory `dist`; remove
stale dashboard overrides when project access is restored.

After a change worth re-crawling, tell IndexNow (Bing, Yandex and others):

```
node site/scripts/indexnow.mjs
```

## When the launcher or QOR Mobile has a new release

Each download is offered in five places, and all of them change with the release: the hero's button and the phone menu's
(`index.html`), its card in the Download section (version, size, SHA-256 and the file's link), its node in the structured
data (`softwareVersion`, `downloadUrl`, `fileSize`), and `llms.txt`. Then `lastmod` in `sitemap.xml`. A link names the
release's file itself, `releases/download/<tag>/<file>`; take the size and the SHA-256 from the release (`gh api
repos/QOR-MATRIX/demiurge-chain/releases/tags/<tag>`, whose assets carry their `digest`). `check-site.mjs` fails a link
whose tag and file name disagree on the version, or a card without its hash.

- **The launcher**: tags `launcher-v<version>`, file `QOR-Launcher_<version>_x64-setup.exe`.
- **QOR Mobile**: tags `android-v<version>`, file `qor-mobile-<version>-arm64-v8a.apk`, built and signed with
  `tools/qor-launcher/shell/build.ps1 -Android -TestKey` (its README).

Every claim must still be one a current document makes (ADR-100 decision 4).

## When QOR OS's facts change

QOR OS's code is in a private repository, so the page may say about it only what ADR-128 decision 1 lists. A change to
those facts (a new stage reached, a download) is a later record that restates them; then the page, `llms.txt`, the
structured data and `scripts/og.html` follow it.
