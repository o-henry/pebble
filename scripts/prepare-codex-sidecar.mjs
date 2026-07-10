import { chmodSync, copyFileSync, mkdirSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const targetByPlatform = {
  "darwin:arm64": {
    packageName: "@openai/codex-darwin-arm64",
    triple: "aarch64-apple-darwin"
  },
  "darwin:x64": {
    packageName: "@openai/codex-darwin-x64",
    triple: "x86_64-apple-darwin"
  }
};

const target = targetByPlatform[`${process.platform}:${process.arch}`];
if (!target) {
  throw new Error(
    `ScreenPebble currently supports macOS only; received ${process.platform}/${process.arch}.`
  );
}

const codexPackageJson = require.resolve("@openai/codex/package.json");
const codexRequire = createRequire(codexPackageJson);
const packageJson = codexRequire.resolve(`${target.packageName}/package.json`);
const binaryName = process.platform === "win32" ? "codex.exe" : "codex";
const source = path.join(
  path.dirname(packageJson),
  "vendor",
  target.triple,
  "codex",
  binaryName
);
const destinationDir = path.join(root, "src-tauri", "binaries");
const destination = path.join(
  destinationDir,
  `codex-${target.triple}${process.platform === "win32" ? ".exe" : ""}`
);

mkdirSync(destinationDir, { recursive: true });

const sourceSize = statSync(source).size;
let destinationSize = -1;
try {
  destinationSize = statSync(destination).size;
} catch {
  // The first preparation creates the sidecar.
}

if (destinationSize !== sourceSize) {
  copyFileSync(source, destination);
}
chmodSync(destination, 0o755);

console.log(`Prepared Codex app-server sidecar for ${target.triple}.`);
