# qorsync.dev

The project's homepage (ADR-100): one static page, written so that search engines and AI crawlers describe the project as
it is now. Since ADR-123 it leads with QOR OS, the owner's operating system, with Demiurge's network beneath it, and since
ADR-127 it offers QOR Mobile and the QOR Launcher for download and is designed first for the phone. No framework, no
build step and no trackers.

| File | What it is |
| --- | --- |
| `index.html` | The page: every word is HTML, with the title, description, canonical address, share cards and structured data in its head |
| `styles.css`, `main.js` | The look (obsidian, gilt and ember, with QOR OS's mark), the phone's menu sheet, and the living background (ADR-127): one WebGL shader, the mark as a lens that bends a drifting nebula and its stars, a tilted burning disc whose far side arcs over the ring, the mark's tail orbiting, and a small mass under the finger or pointer, with a ring of light at each tap. Reduced motion draws one still frame, a hidden tab nothing, an untouched phone half as often; the resolution drops when frames come late; without WebGL the painted ground stays |
| `robots.txt`, `sitemap.xml`, `llms.txt` | What crawlers read: crawl everything, the one address, and a plain summary for AI search |
| `og.png`, `icon-*.png`, `favicon.svg` | The share image and icons, rendered by `scripts/render-images.mjs` from `scripts/og.html` and `favicon.svg` |
| `<32 hex digits>.txt` | The IndexNow key, public by design: it proves to Bing and others that a change notice comes from this site |
| `404.html`, `site.webmanifest`, `.well-known/security.txt`, `vercel.json` | The not-found page, the manifest, where to report a vulnerability, and the headers |

## Checking it

```
node site/scripts/check-site.mjs        # 33 checks: the head, the share image, structured data, crawler files, links, ADR-008's words, QOR OS, the downloads
node site/scripts/render-images.mjs     # after changing og.html or favicon.svg (needs Edge or Chrome)
```

To look at it, serve the folder (`npx serve site`, or any static server) and open it, at a phone's width as well as a
desktop's.

## Publishing

The Vercel project `demiurge-site` in the `astramatrix` team is connected to this repository with `site/` as its root, so a
merge to `main` publishes it. `qorsync.dev` is the address; `www.qorsync.dev`, `demiurge.cloud`, `www.demiurge.cloud`,
`demiurge.guru` and `www.demiurge.guru` redirect to it.

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
