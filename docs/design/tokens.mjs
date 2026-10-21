// Demiurge's look, from one source (ADR-108): docs/design/tokens.json, and the fonts in docs/design/fonts/.
//
//   node docs/design/tokens.mjs            write every generated file and copy every font where a surface needs it
//   node docs/design/tokens.mjs --check    write nothing; fail if any of that is stale, or if a text colour reads
//                                          under the contrast floor on any background
//
// No surface writes a colour, a face, a radius or a duration of its own: each takes them from what this script
// generates for it. A surface joins by adding its target below, in the same change that makes it read the target.
//
// The contrast floor (WCAG 2.2 AA, 4.5:1) holds for every text colour on every background, and for the text on every
// filled control, so a palette that would leave some text unreadable cannot be committed.

import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '..', '..');
const tokens = JSON.parse(readFileSync(join(here, 'tokens.json'), 'utf8'));
const web = JSON.parse(readFileSync(join(here, 'fonts', 'web.json'), 'utf8'));
const checking = process.argv.includes('--check');
const problems = [];

// ── contrast ──────────────────────────────────────────────────────────────────

const luminance = (hex) => {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const lin = (c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
};
const ratio = (a, b) => {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
};
const colour = (name) => {
  const value = tokens.color[name];
  if (!/^#[0-9a-f]{6}$/.test(value ?? '')) problems.push(`colour "${name}" is not a lower-case #rrggbb value: ${value}`);
  return value;
};
const { minimum, backgrounds, text, onFill } = tokens.contrast;
let weakest = { ratio: Infinity };
for (const fg of text) {
  for (const bg of backgrounds) {
    const r = ratio(colour(fg), colour(bg));
    if (r < weakest.ratio) weakest = { ratio: r, pair: `${fg} on ${bg}` };
    if (r < minimum) problems.push(`${fg} on ${bg} reads ${r.toFixed(2)}:1, under ${minimum}:1`);
  }
}
for (const [fill, ink] of Object.entries(onFill)) {
  const r = ratio(colour(ink), colour(fill));
  if (r < minimum) problems.push(`${ink} on a ${fill} fill reads ${r.toFixed(2)}:1, under ${minimum}:1`);
}

// ── what each surface is given ────────────────────────────────────────────────

const kebab = (name) => name.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
const GENERATED =
  'Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,\n' +
  'run `node docs/design/tokens.mjs`, and commit what it writes.';

/** CSS custom properties, and @font-face rules for the faces served from `fontDir` (relative to the stylesheet). */
function css(fontDir) {
  const faces = web.faces
    .map(
      (f) =>
        `@font-face {\n  font-family: '${f.family}';\n  font-style: normal;\n  font-display: swap;\n  font-weight: ${f.weight};\n` +
        `  src: url(${fontDir}/${f.file}) format('woff2');\n  unicode-range: ${f.unicodeRange};\n}\n`,
    )
    .join('\n');
  const vars = [];
  for (const [name, value] of Object.entries(tokens.color)) vars.push(`  --qor-${kebab(name)}: ${value};`);
  for (const [role, f] of Object.entries(tokens.font)) vars.push(`  --qor-font-${role}: "${f.family}", ${f.fallback};`);
  for (const [step, px] of Object.entries(tokens.size)) vars.push(`  --qor-size-${step}: ${px}px;`);
  for (const [name, value] of Object.entries(tokens.tracking)) vars.push(`  --qor-tracking-${name}: ${value};`);
  for (const [name, px] of Object.entries(tokens.radius)) vars.push(`  --qor-radius-${name}: ${px}px;`);
  const { ease, ...durations } = tokens.motion;
  vars.push(`  --qor-ease: ${ease};`);
  for (const [name, ms] of Object.entries(durations)) vars.push(`  --qor-duration-${name}: ${ms}ms;`);
  vars.push(`  --qor-focus-width: ${tokens.focus.width}px;`);
  vars.push(`  --qor-focus-halo: ${tokens.focus.halo}px;`);
  vars.push(`  --qor-focus: var(--qor-${kebab(tokens.focus.color)});`);
  return `/*\n  ${GENERATED.replace('\n', '\n  ')}\n*/\n\n${faces}\n:root {\n${vars.join('\n')}\n}\n`;
}

// Each target: a file this script writes, or a font it copies. A surface joins by adding its own.
const targets = [
  // The homepage (site/, ADR-100): its stylesheet maps its own names onto these.
  { file: 'site/tokens.css', text: () => css('fonts') },
  ...web.faces.map((f) => ({ file: `site/fonts/${f.file}`, copy: `docs/design/fonts/web/${f.file}` })),
  // The Qt applications' faces, bundled in each: the same files, so they cannot drift apart.
  ...['Cinzel.ttf', 'BarlowSemiCondensed-Regular.ttf', 'BarlowSemiCondensed-Medium.ttf', 'BarlowSemiCondensed-SemiBold.ttf']
    .map((name) => ({ file: `products/arqade/qt/fonts/${name}`, copy: `docs/design/fonts/${name}` })),
  ...['Cinzel.ttf', 'BarlowSemiCondensed-Regular.ttf', 'BarlowSemiCondensed-SemiBold.ttf']
    .map((name) => ({ file: `products/qq/runtime/fonts/${name}`, copy: `docs/design/fonts/${name}` })),
];

let written = 0;
for (const t of targets) {
  const path = join(repo, t.file);
  const want = t.copy ? readFileSync(join(repo, t.copy)) : Buffer.from(t.text(), 'utf8');
  const have = existsSync(path) ? readFileSync(path) : null;
  // A generated text file is compared whatever its line endings: Git on Windows writes CRLF where it stores LF. A face
  // is compared byte for byte.
  const same = t.copy
    ? have?.equals(want)
    : have !== null && have.toString('utf8').replaceAll('\r\n', '\n') === want.toString('utf8');
  if (same) continue;
  if (checking) {
    const what = !have ? 'is missing' : t.copy ? `differs from ${t.copy}` : 'is stale';
    problems.push(`${t.file} ${what}: run node docs/design/tokens.mjs`);
    continue;
  }
  mkdirSync(dirname(path), { recursive: true });
  if (t.copy) copyFileSync(join(repo, t.copy), path);
  else writeFileSync(path, want);
  written += 1;
  console.log(`wrote ${relative(repo, path).replaceAll('\\', '/')}`);
}

console.log(
  `${targets.length} targets${checking ? ' checked' : `, ${written} written`}; ` +
    `${text.length} text colours on ${backgrounds.length} backgrounds, the weakest ${weakest.pair} at ${weakest.ratio.toFixed(2)}:1`,
);
if (problems.length) {
  for (const p of problems) console.error(`  ${p}`);
  console.error(`${problems.length} problem${problems.length === 1 ? '' : 's'}`);
  process.exit(1);
}
