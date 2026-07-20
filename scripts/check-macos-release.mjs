import { execFileSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { validateMacosRelease } from "./macos-release-policy.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const requireSigning = process.argv.includes("--require-signing");

function readJson(relativePath) {
  return JSON.parse(readFileSync(path.join(root, relativePath), "utf8"));
}

function cargoPackageVersion() {
  const output = execFileSync(
    "cargo",
    ["metadata", "--locked", "--no-deps", "--format-version", "1"],
    { cwd: path.join(root, "src-tauri"), encoding: "utf8" }
  );
  const metadata = JSON.parse(output);
  const pebblePackage = metadata.packages.find((candidate) => candidate.name === "pebble");
  if (!pebblePackage) {
    throw new Error("Cargo metadata does not contain the Pebble package.");
  }
  return pebblePackage.version;
}

function privateKeyFile() {
  if (!requireSigning || !process.env.APPLE_API_KEY_PATH) return null;
  try {
    const metadata = statSync(process.env.APPLE_API_KEY_PATH);
    return { isFile: metadata.isFile(), mode: metadata.mode };
  } catch {
    return { isFile: false, mode: 0 };
  }
}

const packageJson = readJson("package.json");
const packageLock = readJson("package-lock.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const errors = validateMacosRelease({
  packageVersion: packageJson.version,
  packageLockVersion: packageLock.packages?.[""]?.version,
  cargoVersion: cargoPackageVersion(),
  tauriConfig,
  environment: process.env,
  requireSigning,
  apiKeyFile: privateKeyFile()
});

if (errors.length > 0) {
  for (const error of errors) console.error(`Release policy: ${error}`);
  process.exitCode = 1;
} else {
  console.log(
    requireSigning
      ? "Signed macOS release policy passed."
      : "macOS release configuration passed."
  );
}
