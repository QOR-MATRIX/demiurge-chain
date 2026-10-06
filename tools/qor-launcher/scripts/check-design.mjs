// Does the launcher frontend stay inside its design system? (roadmap L1.1)
//
// Scans src/ for what keeps the launcher consistent and themeable, so it cannot
// drift one class at a time: colour literals outside the token files, and sizes
// and tracking off the scale. Effects (glow, gradients, canvas, pointer-reactive
// light, looping animation) are allowed since ADR-080, the owner's decision of
// 6 October 2026; what they owe people is measured elsewhere: contrast and
// readability as painted (check-contrast.mjs, check-readability.mjs), and reduce
// motion stilling them.
//
//   node scripts/check-design.mjs
//
// Exits non-zero, listing every violation with its file and line.

import { readdirSync, readFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const src = join(root, 'src');

/** Files that define tokens and so may contain colour literals. */
const TOKEN_FILES = new Set(['src/styles/themes.ts', 'src/styles/qor.css']);

const RULES = [
  {
    name: 'Colours come from tokens, not literals',
    test: (file) => !TOKEN_FILES.has(file),
    pattern: /#[0-9a-fA-F]{3,8}\b|rgba?\(/,
  },
  {
    name: 'Font sizes come from the type scale (text-micro to text-figure)',
    test: (file) => file.endsWith('.tsx') || file.endsWith('.ts'),
    pattern: /\btext-\[\d+(\.\d+)?(px|rem|em)\]/,
  },
  {
    name: 'Letter-spacing comes from the tracking scale',
    test: (file) => file.endsWith('.tsx') || file.endsWith('.ts'),
    pattern: /\btracking-\[/,
  },
];

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return walk(path);
    return /\.(tsx?|css)$/.test(entry.name) ? [path] : [];
  });
}

/** Strip comments so prose that names a colour or a size is not a violation. */
function code(text, file) {
  const blank = (m) => m.replace(/[^\n]/g, ' ');
  let out = text.replace(/\/\*[\s\S]*?\*\//g, blank);
  if (!file.endsWith('.css')) out = out.replace(/(^|[^:'"`])\/\/.*$/gm, (m, lead) => lead + blank(m.slice(lead.length)));
  return out;
}

const violations = [];

for (const path of walk(src)) {
  const file = relative(root, path).replaceAll('\\', '/');
  const lines = code(readFileSync(path, 'utf8'), file).split('\n');
  for (const rule of RULES) {
    if (!rule.test(file)) continue;
    lines.forEach((line, i) => {
      if (rule.pattern.test(line)) violations.push({ rule: rule.name, where: `${file}:${i + 1}`, line: line.trim() });
    });
  }
}

for (const rule of RULES) {
  const hits = violations.filter((v) => v.rule === rule.name);
  console.log(`${hits.length ? 'FAIL' : 'PASS'}  ${rule.name}`);
  for (const hit of hits) console.log(`        ${hit.where}  ${hit.line}`);
}
const failed = RULES.filter((rule) => violations.some((v) => v.rule === rule.name)).length;
console.log(`RESULT: ${RULES.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
