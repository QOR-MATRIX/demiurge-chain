// Are the Qt applications in the tool register drawn in Demiurge's one look? (ADR-108, DIRECTION P2.9)
//
//   node docs/design/check-qml.mjs
//
// It reads the QML of QOR Media (tools/qor-media/qml/) and the QOR Launcher for Android (tools/qor-launcher/shell/qml/),
// whose Themes bind to the Tokens.qml that tokens.mjs generates, and fails when:
//   - Tokens.qml is missing, or Theme writes a colour of its own or does not bind to Tokens;
//   - a face is not one of Theme's (ui, text and mono; display too where the surface has a wordmark);
//   - a text size is not one of Theme's steps, or tracking is not a step times one of Theme's (letterSpacing in pixels
//     is the size times the ems);
//   - a colour is written, as a string or as a function of numbers (Qt.rgba() and the like);
//   - a corner is not Theme's, another item's, a circle, or a hairline of 2 pixels or less;
//   - a duration is not Theme's, but for the art listed below, or an easing is not the look's (Theme.ease);
//   - a control is kept from the keyboard (focusPolicy: Qt.NoFocus).
// CI has no licensed Qt; on the owner's PC each application's own tests check the same as painted.
//
// Exits non-zero, naming each problem.

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');

const surfaces = [
  {
    name: 'QOR Media',
    dir: 'tools/qor-media/qml',
    faces: ['ui', 'text', 'mono'],
    // The music's slow glow, which stays still for less motion.
    art: { 'Player.qml': { durations: [1800] } },
  },
  {
    name: 'the QOR Launcher for Android',
    dir: 'tools/qor-launcher/shell/qml',
    faces: ['display', 'ui', 'text', 'mono'],
    art: {},
  },
];

// A value's branches: both sides of a choice (`code ? Theme.mono : Theme.text`), or the value itself.
const branches = (value) =>
  value.includes('?') ? value.slice(value.indexOf('?') + 1).split(':').map((b) => b.trim()) : [value.trim()];

const problems = [];
let files = 0;
for (const surface of surfaces) {
  const dir = join(repo, surface.dir);
  if (!existsSync(join(dir, 'Tokens.qml'))) problems.push(`${surface.dir}/Tokens.qml is missing: run node docs/design/tokens.mjs`);
  const theme = readFileSync(join(dir, 'Theme.qml'), 'utf8');
  if (/"#[0-9a-fA-F]{3,8}"/.test(theme)) problems.push(`${surface.dir}/Theme.qml writes a colour of its own: its values are Tokens.qml's`);
  if (!theme.includes('Tokens.')) problems.push(`${surface.dir}/Theme.qml does not bind to Tokens`);
  const faces = new RegExp(`^Theme\\.(${surface.faces.join('|')})$`);
  for (const file of readdirSync(dir).filter((f) => f.endsWith('.qml') && f !== 'Tokens.qml' && f !== 'Theme.qml')) {
    files += 1;
    const text = readFileSync(join(dir, file), 'utf8');
    const where = `${surface.dir}/${file}`;
    const art = surface.art[file] ?? { durations: [] };
    for (const [, family] of text.matchAll(/font\.family:\s*([^\n;}]+)/g))
      if (!branches(family).every((b) => faces.test(b)))
        problems.push(`${where}: font.family: ${family.trim()} is not one of the look's faces (${surface.faces.map((f) => `Theme.${f}`).join(', ')})`);
    for (const [, size] of text.matchAll(/font\.pixelSize:\s*([^\n;}]+)/g))
      if (!branches(size).every((b) => /^Theme\.size[A-Z]\w*$/.test(b)))
        problems.push(`${where}: font.pixelSize: ${size.trim()} is off the scale: a size is one of Theme's`);
    for (const [, spacing] of text.matchAll(/font\.letterSpacing:\s*([^\n;}]+)/g))
      if (!/^Theme\.size\w+ \* Theme\.tracking\w+$/.test(spacing.trim()))
        problems.push(`${where}: font.letterSpacing: ${spacing.trim()} is tracking of its own: a size times one of Theme's`);
    for (const [, hex] of text.matchAll(/"(#[0-9a-fA-F]{3,8})"/g))
      problems.push(`${where}: ${hex} is written as a colour: the interface's colours are Theme's`);
    for (const [, name] of text.matchAll(/[cC]olor:\s*"([a-z]+)"/g))
      if (name !== 'transparent') problems.push(`${where}: "${name}" is written as a colour: the interface's colours are Theme's`);
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
      if (/^Theme\.(quick|standard|enter)$/.test(time)) continue;
      if (/^\d+$/.test(time) && art.durations.includes(Number(time))) continue;
      problems.push(`${where}: duration: ${time} is timed by its own numbers: the durations are Theme's`);
    }
    for (const [, type] of text.matchAll(/easing\.type:\s*Easing\.(\w+)/g))
      if (type !== 'BezierSpline' && !(art.durations.length && type === 'InOutSine'))
        problems.push(`${where}: Easing.${type} is not the look's easing (Easing.BezierSpline on Theme.ease)`);
    for (const [, curve] of text.matchAll(/easing\.bezierCurve:\s*([^\n;}]+)/g))
      if (curve.trim() !== 'Theme.ease') problems.push(`${where}: easing.bezierCurve: ${curve.trim()} is not Theme.ease`);
    if (/focusPolicy:\s*Qt\.NoFocus/.test(text))
      problems.push(`${where}: focusPolicy: Qt.NoFocus keeps a control from the keyboard: Tab reaches every control`);
  }
}

if (problems.length) {
  for (const p of problems) console.log(`FAIL  ${p}`);
  console.log(`RESULT: ${problems.length} problem${problems.length === 1 ? '' : 's'}`);
  process.exit(1);
}
console.log(
  `RESULT: every face, size, colour, corner and timing the look's, and every control within the keyboard's reach, in ` +
    `${files} QML files of ${surfaces.map((s) => s.name).join(' and ')}`,
);
