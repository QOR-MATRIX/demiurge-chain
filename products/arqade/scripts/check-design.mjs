// Is ARQADE drawn in Demiurge's one look, on the web and in its Qt application? (ADR-108, DIRECTION P2.9)
//
//   node scripts/check-design.mjs
//
// It reads app/globals.css and fails when:
//   - it does not import tokens.css, which docs/design/tokens.mjs generates from docs/design/tokens.json;
//   - one of shadcn's variables (--background … --ring) is not a token;
//   - a font is set to a face of its own, not one of the look's (var(--qor-font-…));
//   - a colour is written that is not one the games paint with: every colour of the interface is a token, and what is
//     still written as a colour is the games' own art and its glows, listed below as they stood when the look arrived.
//     A colour of the interface added later is a token, never an entry here.
//   - a colour is written as a function of numbers (rgb(), hsl(), oklch() …): a token with transparency is
//     color-mix(in srgb, var(--qor-…) 12%, transparent), and a colour built on a variable, as the players' hues are, passes.
//   - a transition is timed by anything but the tokens (var(--qor-duration-…) and var(--qor-ease)), or an animation
//     is, other than the games' own: a disc's drop, and the intro's glitch and fades;
//   - a corner is not a token (var(--qor-radius-…)), a circle (50%), a pill (999px), square (0) or the 2 pixels of a
//     meter's track and a key's cap.
//
// And it reads the Qt application's QML (qt/qml/), whose Theme binds to the generated Tokens.qml, and fails when:
//   - Tokens.qml is missing, or Theme writes a colour of its own;
//   - a face is not one of Theme's (display, ui, body);
//   - a text under the showcase's Cinzel titles (18 pixels) is not a step of the scale (Theme.size…);
//   - a colour is written, as a string or as Qt.rgba() of numbers, but for the wordmark's sheen and its masks;
//   - a corner is not Theme's, another item's, a circle, or a hairline of 2 pixels or less;
//   - a duration is not Theme's, but for three slow loops of light, or an easing is not the look's (Theme.ease).
// The 3D scenes (Emblem, Portal) are the games' own art: their lights, metals and motion are theirs, and only the face
// rule reads them. CI has no licensed Qt; on the owner's PC tests/tst_interface.cpp checks the same as painted.
//
// Exits non-zero, naming each problem.

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const css = readFileSync(join(root, 'app', 'globals.css'), 'utf8');
const problems = [];

// The games' own art, and its glows: the colours that are not the interface's.
const ART = new Set([
  '#0001', '#0004', '#0005', '#0006', '#0008', '#0009', '#000a', '#07131de8', '#09121b', '#0a151d', '#0c1217',
  '#0f1013b8', '#0f171f', '#10101144', '#10101188', '#101011ba', '#101011e8', '#101012aa', '#101012ed', '#102229',
  '#111214f0', '#11171d', '#11171ef5', '#12191f', '#12272c', '#122d32', '#141e27', '#15222f', '#17232c', '#18232e',
  '#182f35', '#19232d88', '#1c242cf0', '#1c2733', '#1e211fef', '#1e3441', '#201a16', '#211b17', '#211c16',
  '#213c40', '#221b17', '#233445', '#253745', '#263d35', '#263d4b', '#293e49', '#2947570d', '#29475722', '#2b2219',
  '#30201b', '#30475520', '#31281e', '#34281d66', '#34494d', '#35545d', '#36505b', '#37302238', '#3a5661',
  '#3b535344', '#3b747f', '#3e211722', '#3e6267', '#40526844', '#418594', '#44332644', '#466267', '#598cac',
  '#5cabb6', '#5d9ca7', '#62c9d02b', '#69cbdf4a', '#6b9aa7', '#70899344', '#74472935', '#79c3b20a', '#79c6bb44',
  '#7a603c44', '#82bdcc', '#83b5bc', '#85c3cf', '#85c3cf25', '#86d0d8', '#8aa3b31a', '#8bc4cb', '#8cacae',
  '#8d684428', '#8dd4d9', '#8dd5c7', '#8eadaf', '#90c5c9', '#90cfc5', '#9b5a3c', '#9ddcde', '#a632183a', '#a9c4d2',
  '#aac4d0', '#b1c6d5', '#b3b598', '#b3c5d2', '#c4e5e5', '#caebe2', '#d39b4b0c', '#d7bd8e25', '#dbf5f2', '#def6f5',
  '#e4f9f4', '#ed794144', '#eead6b25', '#fb6c3425', '#ff683808', '#ff683821', '#ff6a3028', '#ff6b3500',
  '#ff6b351f', '#ff88512b', '#fff9', '#ffffff0a', '#ffffff0d', '#ffffff13', '#ffffff15', '#ffffff18'
]);

if (!/@import\s+"\.\/tokens\.css";/.test(css)) problems.push('app/globals.css does not import ./tokens.css');
if (!existsSync(join(root, 'app', 'tokens.css'))) problems.push('app/tokens.css is missing: run node docs/design/tokens.mjs');

const root_ = css.match(/:root\{([^}]*)\}/)?.[1] ?? '';
for (const name of ['background', 'foreground', 'card', 'popover', 'primary', 'primary-foreground', 'secondary', 'muted',
                    'muted-foreground', 'accent', 'accent-foreground', 'destructive', 'border', 'input', 'ring', 'radius']) {
  const value = root_.match(new RegExp(`--${name}:([^;]+)`))?.[1]?.trim();
  if (!value?.startsWith('var(--qor-')) problems.push(`--${name} is ${value ?? 'not set'}, not a token`);
}

for (const [, value] of css.matchAll(/font(?:-family)?:\s*([^;}]+)/g)) {
  if (/^inherit\b/.test(value.trim())) continue;
  const family = value.replace(/var\(--qor-font-[a-z]+\)/g, '');
  if (/\b(Arial|Helvetica|Segoe|system-ui|monospace|sans-serif|serif|Roboto|Inter)\b/i.test(family))
    problems.push(`a face of its own: font: ${value.trim().slice(0, 60)}`);
}

const stray = [...new Set([...css.matchAll(/#[0-9a-fA-F]{3,8}\b/g)].map((m) => m[0].toLowerCase()))].filter((c) => !ART.has(c));
for (const c of stray) problems.push(`${c} is written as a colour: the interface's colours are tokens (var(--qor-…))`);

for (const [fn] of css.matchAll(/\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(\s*[\d.]+[^)]*\)/gi))
  problems.push(`${fn} is written as a colour: the interface's colours are tokens (var(--qor-…))`);

// The games' own motion: what is timed by them and not by the interface.
const ART_MOTION = new Set(['disc-drop', 'glitch', 'fade-portal']);
// Each part, between commas: a property or a keyframes' name, a duration token, the easing, and for an animation
// anything after them that is not a time (a count, a direction, a fill).
const TIMED = /^[a-z-]+ var\(--qor-duration-[a-z]+\) var\(--qor-ease\)((?: (?![\d.]+m?s\b)[a-z0-9-]+)*)(?:!important)?$/;
const parts = (value) => value.trim().split(/,(?![^(]*\))/).map((p) => p.trim().replace(/\s+/g, ' '));
for (const [, value] of css.matchAll(/transition:([^;}]+)/g)) {
  if (value.trim().startsWith('none')) continue;
  if (!parts(value).every((p) => TIMED.test(p) && !TIMED.exec(p)[1]))
    problems.push(`transition:${value} is timed by its own numbers: its durations and easing are tokens`);
}
for (const [, value] of css.matchAll(/animation:([^;}]+)/g)) {
  const name = value.trim().split(/[\s!]+/)[0];
  if (name === 'none' || ART_MOTION.has(name)) continue;
  if (!parts(value).every((p) => TIMED.test(p)))
    problems.push(`animation:${value} is timed by its own numbers: its durations and easing are tokens`);
}

for (const [, value] of css.matchAll(/border-radius:([^;}]+)/g)) {
  const corner = value.replace(/!important/, '').trim();
  if (!/^(var\(--qor-radius-[a-z]+\)|50%|999px|0|2px)$/.test(corner))
    problems.push(`border-radius:${corner} is a corner of its own: a panel's and a control's are tokens`);
}

// ── The Qt application ──────────────────────────────────────────────────────────────────────────────────────────────

const qml = join(root, 'qt', 'qml');
// The games' own 3D scenes.
const SCENES = new Set(['Emblem.qml', 'Portal.qml']);
// What the 2D files still write themselves: the wordmark's sheen and the masks it draws through, and three slow loops of
// light (the sheen's pass and pause, a card's status breathing, the light along the signing-in rule).
const QML_ART = {
  'Wordmark.qml': { colours: ['#fff4d6', 'white'], durations: [4200, 3800] },
  'GameCard.qml': { durations: [1100] },
  'Rankings.qml': { durations: [1400] },
};
if (!existsSync(join(qml, 'Tokens.qml'))) problems.push('qt/qml/Tokens.qml is missing: run node docs/design/tokens.mjs');
if (/"#[0-9a-fA-F]{3,8}"/.test(readFileSync(join(qml, 'Theme.qml'), 'utf8')))
  problems.push("qt/qml/Theme.qml writes a colour of its own: its values are Tokens.qml's");
let qmlFiles = 0;
for (const file of readdirSync(qml).filter((f) => f.endsWith('.qml') && f !== 'Tokens.qml' && f !== 'Theme.qml')) {
  qmlFiles += 1;
  const text = readFileSync(join(qml, file), 'utf8');
  const where = `qt/qml/${file}`;
  const art = QML_ART[file] ?? { colours: [], durations: [] };
  for (const [, family] of text.matchAll(/font\.family:\s*([^\n;}]+)/g))
    if (!/^Theme\.(display|ui|body)$/.test(family.trim()))
      problems.push(`${where}: font.family: ${family.trim()} is not one of the look's faces (Theme.display, .ui, .body)`);
  for (const [, size] of text.matchAll(/font\.pixelSize:\s*([\d.]+)\s*$/gm))
    if (Number(size) < 18)
      problems.push(`${where}: font.pixelSize: ${size} is off the scale: under the showcase's titles a size is Theme's`);
  if (SCENES.has(file)) continue;
  for (const [, hex] of text.matchAll(/"(#[0-9a-fA-F]{3,8})"/g))
    if (!(art.colours ?? []).includes(hex.toLowerCase()))
      problems.push(`${where}: ${hex} is written as a colour: the interface's colours are Theme's`);
  for (const [, name] of text.matchAll(/[cC]olor:\s*"([a-z]+)"/g))
    if (name !== 'transparent' && !(art.colours ?? []).includes(name))
      problems.push(`${where}: "${name}" is written as a colour: the interface's colours are Theme's`);
  for (const [call] of text.matchAll(/Qt\.(?:rgba|hsla|hsva|hsl)\(\s*[\d.]+[^)]*\)/g))
    problems.push(`${where}: ${call} is written as a colour: a colour with transparency is built on Theme's`);
  for (const [, value] of text.matchAll(/\bradius:\s*([^\n;}]+)/g)) {
    const corner = value.trim();
    if (/^(Theme\.radius(Control|Panel)|\w+\.radius|width \/ 2|height \/ 2)\b/.test(corner)) continue;
    if (/^[\d.]+$/.test(corner) && Number(corner) <= 2) continue;
    problems.push(`${where}: radius: ${corner} is a corner of its own: a panel's and a control's are Theme's`);
  }
  for (const [, value] of text.matchAll(/\bduration:\s*([^\n;}]+)/g)) {
    const time = value.trim();
    if (/^Theme\.(quick|standard|enter|sheen)$/.test(time)) continue;
    if (/^\d+$/.test(time) && (art.durations ?? []).includes(Number(time))) continue;
    problems.push(`${where}: duration: ${time} is timed by its own numbers: the durations are Theme's`);
  }
  for (const [, type] of text.matchAll(/easing\.type:\s*Easing\.(\w+)/g))
    if (type !== 'BezierSpline' && !((art.durations ?? []).length && type === 'InOutSine'))
      problems.push(`${where}: Easing.${type} is not the look's easing (Easing.BezierSpline on Theme.ease)`);
  for (const [, curve] of text.matchAll(/easing\.bezierCurve:\s*([^\n;}]+)/g))
    if (curve.trim() !== 'Theme.ease') problems.push(`${where}: easing.bezierCurve: ${curve.trim()} is not Theme.ease`);
}

if (problems.length) {
  for (const p of problems) console.log(`FAIL  ${p}`);
  console.log(`RESULT: ${problems.length} problem${problems.length === 1 ? '' : 's'}`);
  process.exit(1);
}
console.log(
  `RESULT: every colour, corner and timing of the interface a token, every face the look's, on the web (${ART.size} ` +
    `colours of the games' art) and in ${qmlFiles} QML files of the Qt application`,
);
