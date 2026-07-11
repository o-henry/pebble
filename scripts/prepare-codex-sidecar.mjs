import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  statSync,
  writeFileSync
} from "node:fs";
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
    `Pebble currently supports macOS only; received ${process.platform}/${process.arch}.`
  );
}

const codexPackageJson = require.resolve("@openai/codex/package.json");
const codexRequire = createRequire(codexPackageJson);
const packageJson = codexRequire.resolve(`${target.packageName}/package.json`);
const packageVersion = JSON.parse(readFileSync(packageJson, "utf8")).version;
const binaryName = process.platform === "win32" ? "codex.exe" : "codex";
const vendorTarget = path.join(
  path.dirname(packageJson),
  "vendor",
  target.triple
);
const source = [
  path.join(vendorTarget, "bin", binaryName),
  path.join(vendorTarget, "codex", binaryName)
].find(existsSync);
if (!source) {
  throw new Error(`Codex executable is missing for ${target.triple}.`);
}
const destinationDir = path.join(root, "src-tauri", "binaries");
const destination = path.join(
  destinationDir,
  `codex-${target.triple}${process.platform === "win32" ? ".exe" : ""}`
);
const versionMarker = `${destination}.version`;

mkdirSync(destinationDir, { recursive: true });

const sourceSize = statSync(source).size;
let destinationSize = -1;
let destinationMode = 0;
try {
  const destinationStat = statSync(destination);
  destinationSize = destinationStat.size;
  destinationMode = destinationStat.mode;
} catch {
  // The first preparation creates the sidecar.
}

let preparedVersion = "";
try {
  preparedVersion = readFileSync(versionMarker, "utf8").trim();
} catch {
  // A missing marker means the binary predates version-aware preparation.
}

if (destinationSize !== sourceSize || preparedVersion !== packageVersion) {
  copyFileSync(source, destination);
  chmodSync(destination, 0o755);
  writeFileSync(versionMarker, `${packageVersion}\n`, { mode: 0o644 });
} else if ((destinationMode & 0o111) === 0) {
  chmodSync(destination, 0o755);
}

console.log(
  `Prepared Codex app-server ${packageVersion} sidecar for ${target.triple}.`
);
