// SPDX-License-Identifier: Apache-2.0
// Copy only public assets and the pinned, self-contained browser entry point.
// No bundler, framework, network access or dependency lifecycle script is needed.
import { copyFileSync, existsSync, lstatSync, mkdirSync, readdirSync, realpathSync, rmSync } from 'node:fs';
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = realpathSync(resolve(dirname(fileURLToPath(import.meta.url)), '..'));
const output = join(site, 'dist');
// This is the only directory removed. Refuse links and verify the absolute
// target remains the site's build output before removing an earlier build.
if (dirname(output) !== site || basename(output) !== 'dist'
    || (existsSync(output) && (lstatSync(output).isSymbolicLink() || realpathSync(output) !== output))) {
  throw new Error('Refusing an unexpected build output directory');
}

const publicFiles = [
  'index.html', '404.html', 'styles.css', 'tokens.css', 'main.js', 'analytics.mjs',
  'robots.txt', 'sitemap.xml', 'llms.txt', 'site.webmanifest', '.well-known/security.txt',
  'favicon.svg', 'og.png', 'icon-32.png', 'icon-180.png', 'icon-512.png',
  ...readdirSync(site).filter((name) => /^[0-9a-f]{32}\.txt$/.test(name)),
  ...readdirSync(join(site, 'fonts')).filter((name) => /^[a-z0-9-]+\.woff2$/.test(name))
    .map((name) => `fonts/${name}`),
];
const sources = publicFiles.map((file) => [join(site, file), file]);
sources.push(
  [fileURLToPath(import.meta.resolve('@vercel/analytics')), 'vendor/vercel-analytics.mjs'],
  [join(dirname(fileURLToPath(import.meta.resolve('@vercel/analytics/package.json'))), 'LICENSE'),
    'vendor/vercel-analytics.LICENSE.txt'],
);
// Validate the full copy list before clearing old output. Never follow an asset
// link outside the site, and never publish the repository or node_modules tree.
for (const [source] of sources) {
  const within = relative(site, realpathSync(source));
  if (!lstatSync(source).isFile() || lstatSync(source).isSymbolicLink()
      || within.startsWith(`..${sep}`) || isAbsolute(within)) {
    throw new Error('Build input must be a regular file within the site');
  }
}
rmSync(output, { recursive: true, force: true });
for (const [source, file] of sources) {
  const destination = join(output, file);
  mkdirSync(dirname(destination), { recursive: true });
  copyFileSync(source, destination);
}
console.log(`Built ${sources.length} public files in site/dist`);
