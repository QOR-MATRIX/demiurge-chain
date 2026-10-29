# qorsync.dev

The project's homepage (ADR-100): one static page, written so that search engines and AI crawlers describe the project as
it is now. Since ADR-123 it leads with QOR OS, the owner's operating system, with Demiurge's network beneath it. No
framework, no build step and no trackers.

| File | What it is |
| --- | --- |
| `index.html` | The page: every word is HTML, with the title, description, canonical address, share cards and structured data in its head |
| `styles.css`, `main.js` | The look (obsidian, gilt and ember, with QOR OS's mark) and the living background: an aurora of ember and gilt across the top, a lattice that lights around the pointer, the lens (the mark's ring and burning tail, turning), rising embers. Reduced motion draws one still frame |
| `robots.txt`, `sitemap.xml`, `llms.txt` | What crawlers read: crawl everything, the one address, and a plain summary for AI search |
| `og.png`, `icon-*.png`, `favicon.svg` | The share image and icons, rendered by `scripts/render-images.mjs` from `scripts/og.html` and `favicon.svg` |
| `<32 hex digits>.txt` | The IndexNow key, public by design: it proves to Bing and others that a change notice comes from this site |
| `404.html`, `site.webmanifest`, `.well-known/security.txt`, `vercel.json` | The not-found page, the manifest, where to report a vulnerability, and the headers |

## Checking it

```
node site/scripts/check-site.mjs        # 28 checks: the head, the share image, structured data, crawler files, links, ADR-008's words, ADR-123's QOR OS
node site/scripts/render-images.mjs     # after changing og.html or favicon.svg (needs Edge or Chrome)
```

To look at it, serve the folder (`npx serve site`, or any static server) and open it.

## Publishing

The Vercel project `demiurge-site` in the `astramatrix` team is connected to this repository with `site/` as its root, so a
merge to `main` publishes it. `qorsync.dev` is the address; `www.qorsync.dev`, `demiurge.cloud`, `www.demiurge.cloud`,
`demiurge.guru` and `www.demiurge.guru` redirect to it.

After a change worth re-crawling, tell IndexNow (Bing, Yandex and others):

```
node site/scripts/indexnow.mjs
```

## When the launcher has a new release

Change, in `index.html`: the three download links, the button's note, and `softwareVersion` in the structured data; and
in `sitemap.xml`, `lastmod`. Every claim must still be one a current document makes (ADR-100 decision 4).

## When QOR OS's facts change

QOR OS's code is in a private repository, so the page may say about it only what ADR-123 decision 2 lists. A change to
those facts (a new stage reached, a download) is a later record that restates them; then the page, `llms.txt`, the
structured data and `scripts/og.html` follow it.
