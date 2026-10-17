// Build QOR Media and put a self-contained copy where the bundler takes it from, so the installer carries it.
//
// The Library opens files in QOR Media (ADR-094). Until 0.1.10 a launcher found it only where its build script deploys
// it on the PC that built it, so a launcher installed anywhere else opened files in whatever Windows chose. Now the
// installer carries QOR Media beside the launcher, and `media::find` looks there.
//
// QOR Media is built with the owner's Qt 6, MSVC 2022 and Ninja by `tools/qor-media/build.ps1`, which runs its Qt Test
// suites first and refuses to deploy if one fails. The copy lands in `src-tauri/media-bundle/qor-media/` (not tracked),
// which `src-tauri/tauri.bundle.conf.json` maps to `qor-media/` in the install folder. Declared there, not in
// `tauri.conf.json`, so a clean checkout's `cargo test` does not need Qt.
//
//   node scripts/stage-qor-media.mjs
//
// Windows only: the other bundles do not carry QOR Media yet, and this says so rather than failing.

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const target = join(root, "src-tauri", "media-bundle", "qor-media");

if (process.platform !== "win32") {
  console.log("QOR Media is bundled into the Windows installer only; this build carries none.");
  process.exit(0);
}

const script = resolve(root, "..", "qor-media", "build.ps1");
execFileSync("pwsh", ["-NoProfile", "-File", script, "-Deploy", "-DeployTo", target], { stdio: "inherit" });

const program = join(target, "qor-media.exe");
if (!existsSync(program)) {
  console.error(`QOR Media was not deployed to ${target}`);
  process.exit(1);
}

let files = 0;
let bytes = 0;
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    const stat = statSync(path);
    if (stat.isDirectory()) walk(path);
    else {
      files += 1;
      bytes += stat.size;
    }
  }
};
walk(target);
console.log(`QOR Media staged for the installer: ${files} files, ${(bytes / 1024 / 1024).toFixed(1)} MiB, in ${target}`);
