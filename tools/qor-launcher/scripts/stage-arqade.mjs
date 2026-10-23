// Build ARQADE's Qt app and put a self-contained copy where the bundler takes it from, so the installer carries it.
//
// The launcher's ARQADE screen docks ARQADE's app (ADR-102, DIRECTION P7.19), found beside the launcher by
// `arqade::app::find`. It is built with the owner's Qt 6, MSVC 2022 and Ninja by `products/arqade/qt/build.ps1`, which
// runs its Qt Test suites first and refuses to deploy if one fails. The copy lands in
// `src-tauri/arqade-bundle/arqade/` (not tracked), which `src-tauri/tauri.bundle.conf.json` maps to `arqade/` in the
// install folder. Declared there, not in `tauri.conf.json`, so a clean checkout's `cargo test` does not need Qt.
//
//   node scripts/stage-arqade.mjs
//
// Windows only: the other bundles do not carry ARQADE yet, and this says so rather than failing. Where it is not
// carried, the ARQADE screen offers the website's window, as before.

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const target = join(root, "src-tauri", "arqade-bundle", "arqade");

if (process.platform !== "win32") {
  console.log("ARQADE's app is bundled into the Windows installer only; this build carries none.");
  process.exit(0);
}

const script = resolve(root, "..", "..", "products", "arqade", "qt", "build.ps1");
// A build folder of this checkout's own. CMake refuses a folder configured from another source tree, so two
// checkouts sharing ARQADE's default folder would stop each other's release build.
const build = join(process.env.LOCALAPPDATA ?? root, "arqade-stage-" + createHash("sha256").update(root).digest("hex").slice(0, 8));
execFileSync("pwsh", ["-NoProfile", "-File", script, "-Build", build, "-Deploy", "-DeployTo", target], { stdio: "inherit" });

const program = join(target, "arqade.exe");
if (!existsSync(program)) {
  console.error(`ARQADE was not deployed to ${target}`);
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
console.log(`ARQADE staged for the installer: ${files} files, ${(bytes / 1024 / 1024).toFixed(1)} MiB, in ${target}`);
