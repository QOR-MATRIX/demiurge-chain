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

/** A C++ header of the values, for a Qt application's own style: `namespace` holds them, as constexpr. */
function cpp(namespace) {
  const pascal = (name) => name[0].toUpperCase() + name.slice(1);
  const ease = tokens.motion.ease.match(/cubic-bezier\(([^)]+)\)/)[1].split(',').map((n) => Number(n.trim()));
  const { ease: _, ...durations } = tokens.motion;
  const lines = [];
  lines.push('// Colours, as QRgb (0xrrggbb).');
  for (const [name, value] of Object.entries(tokens.color)) lines.push(`inline constexpr QRgb color${pascal(name)} = 0x${value.slice(1)};`);
  lines.push('', '// Faces: family names, as each is registered once its file is loaded.');
  for (const [role, f] of Object.entries(tokens.font)) lines.push(`inline constexpr const char *font${pascal(role)} = "${f.family}";`);
  lines.push('', '// The type scale, in pixels; tracking, in ems.');
  for (const [step, px] of Object.entries(tokens.size)) lines.push(`inline constexpr int size${pascal(step)} = ${px};`);
  for (const [name, value] of Object.entries(tokens.tracking)) lines.push(`inline constexpr double tracking${pascal(name)} = ${parseFloat(value)};`);
  lines.push('', '// Corners, in pixels.');
  for (const [name, px] of Object.entries(tokens.radius)) lines.push(`inline constexpr int radius${pascal(name)} = ${px};`);
  lines.push('', '// Motion: durations in milliseconds, and the one easing as a cubic Bézier curve.');
  for (const [name, ms] of Object.entries(durations)) lines.push(`inline constexpr int motion${pascal(name)} = ${ms};`);
  lines.push(`inline constexpr double easeBezier[4] = {${ease.join(', ')}};`);
  lines.push('', '// The focus ring.');
  lines.push(`inline constexpr int focusWidth = ${tokens.focus.width};`);
  lines.push(`inline constexpr int focusHalo = ${tokens.focus.halo};`);
  lines.push(`inline constexpr QRgb focusColor = color${pascal(tokens.focus.color)};`);
  const comment = GENERATED.split('\n').map((l) => `// ${l}`).join('\n');
  return `${comment}\n\n#pragma once\n\n#include <QtGui/qrgb.h>\n\nnamespace ${namespace} {\n\n${lines.join('\n')}\n\n}  // namespace ${namespace}\n`;
}

/** A QML singleton of the values, for a Qt Quick application's own theme to bind to. */
function qml() {
  const pascal = (name) => name[0].toUpperCase() + name.slice(1);
  const ease = tokens.motion.ease.match(/cubic-bezier\(([^)]+)\)/)[1].split(',').map((n) => Number(n.trim()));
  const { ease: _, ...durations } = tokens.motion;
  const lines = [];
  lines.push('    // Colours.');
  for (const [name, value] of Object.entries(tokens.color)) lines.push(`    readonly property color color${pascal(name)}: "${value}"`);
  lines.push('', '    // Faces: family names, as each is registered once its file is loaded.');
  for (const [role, f] of Object.entries(tokens.font)) lines.push(`    readonly property string font${pascal(role)}: "${f.family}"`);
  lines.push('', '    // The type scale, in pixels; tracking, in ems (multiply by the size for letterSpacing).');
  for (const [step, px] of Object.entries(tokens.size)) lines.push(`    readonly property int size${pascal(step)}: ${px}`);
  for (const [name, value] of Object.entries(tokens.tracking)) lines.push(`    readonly property real tracking${pascal(name)}: ${parseFloat(value)}`);
  lines.push('', '    // Corners, in pixels.');
  for (const [name, px] of Object.entries(tokens.radius)) lines.push(`    readonly property int radius${pascal(name)}: ${px}`);
  lines.push('', '    // Motion: durations in milliseconds, and the one easing as an Easing.BezierSpline curve.');
  for (const [name, ms] of Object.entries(durations)) lines.push(`    readonly property int motion${pascal(name)}: ${ms}`);
  lines.push(`    readonly property var easeCurve: [${ease.join(', ')}, 1, 1]`);
  lines.push('', '    // The focus ring.');
  lines.push(`    readonly property int focusWidth: ${tokens.focus.width}`);
  lines.push(`    readonly property int focusHalo: ${tokens.focus.halo}`);
  lines.push(`    readonly property color focusColor: color${pascal(tokens.focus.color)}`);
  const comment = GENERATED.split('\n').map((l) => `// ${l}`).join('\n');
  return `${comment}\n\npragma Singleton\nimport QtQuick\n\nQtObject {\n${lines.join('\n')}\n}\n`;
}

/** A TypeScript module of the values, for a web application's own tables (the launcher's themes) to read. */
function ts() {
  const fonts = Object.fromEntries(Object.entries(tokens.font).map(([role, f]) => [role, f.family]));
  const values = { color: tokens.color, font: fonts, size: tokens.size, tracking: tokens.tracking, radius: tokens.radius,
                   motion: tokens.motion, focus: tokens.focus };
  const comment = GENERATED.split('\n').map((l) => ` * ${l}`).join('\n');
  return `/**\n${comment}\n */\n\nexport const TOKENS = ${JSON.stringify(values, null, 2).replace(/"([A-Za-z]+)":/g, '$1:').replaceAll('"', "'")} as const;\n`;
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
  // QQ (P2.9's second slice): Look's values, for the Studio and a game's screens alike, and the Studio's reading and
  // numeric faces, which only the Studio carries, so the Player and its browser build do not grow.
  { file: 'products/qq/runtime/tokens.h', text: () => cpp('qq::tokens') },
  ...['Barlow-Regular.ttf', 'Barlow-Medium.ttf', 'IBMPlexMono-Regular.ttf', 'IBMPlexMono-Medium.ttf', 'Barlow-OFL.txt',
      'IBMPlexMono-OFL.txt'].map((name) => ({ file: `products/qq/studio/fonts/${name}`, copy: `docs/design/fonts/${name}` })),
  // ARQADE on the web (P2.9's third slice): its globals.css imports these, and Next serves the faces from public/.
  { file: 'products/arqade/app/tokens.css', text: () => css('/fonts') },
  ...web.faces.map((f) => ({ file: `products/arqade/public/fonts/${f.file}`, copy: `docs/design/fonts/web/${f.file}` })),
  // ARQADE's Qt application (P2.9's fourth slice): its Theme binds to Tokens.qml, and it carries Barlow for reading
  // beside the faces above, with each family's licence.
  { file: 'products/arqade/qt/qml/Tokens.qml', text: () => qml() },
  ...['Barlow-Regular.ttf', 'Barlow-Medium.ttf', 'Barlow-OFL.txt', 'BarlowSemiCondensed-OFL.txt', 'Cinzel-OFL.txt']
    .map((name) => ({ file: `products/arqade/qt/fonts/${name}`, copy: `docs/design/fonts/${name}` })),
  // The QOR Launcher (P2.9's fourth slice): qor.css imports tokens.css, whose faces Vite bundles from src/styles/fonts/,
  // and its theme table makes Obsidian, its default, from tokens.ts (ADR-109).
  { file: 'tools/qor-launcher/src/styles/tokens.css', text: () => css('./fonts') },
  { file: 'tools/qor-launcher/src/styles/tokens.ts', text: () => ts() },
  ...web.faces.map((f) => ({ file: `tools/qor-launcher/src/styles/fonts/${f.file}`, copy: `docs/design/fonts/web/${f.file}` })),
];

let written = 0;
for (const t of targets) {
  const path = join(repo, t.file);
  const want = t.copy ? readFileSync(join(repo, t.copy)) : Buffer.from(t.text(), 'utf8');
  const have = existsSync(path) ? readFileSync(path) : null;
  // Text is compared whatever its line endings, since Git on Windows writes CRLF where it stores LF. A face is compared
  // byte for byte.
  const isText = !t.copy || /\.(txt|css|h|qml|json)$/.test(t.file);
  const same = isText
    ? have !== null && have.toString('utf8').replaceAll('\r\n', '\n') === want.toString('utf8').replaceAll('\r\n', '\n')
    : have?.equals(want);
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
