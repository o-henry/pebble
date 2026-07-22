import { spawnSync } from "node:child_process";
import { randomUUID } from "node:crypto";
import {
  cpSync,
  existsSync,
  renameSync,
  rmSync,
  statSync
} from "node:fs";
import { basename, resolve } from "node:path";
import {
  parseCodeSignDetails,
  validateInstallCandidate
} from "./macos-install-policy.mjs";

const args = process.argv.slice(2);
const allowAdHocReplacement = args.includes("--replace-ad-hoc");
const paths = args.filter((arg) => !arg.startsWith("--"));
const source = resolve(paths[0] ?? "");
const destination = resolve(paths[1] ?? "/Applications/Pebble.app");

function run(command, commandArgs) {
  return spawnSync(command, commandArgs, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"]
  });
}

function combinedOutput(command, commandArgs) {
  const result = run(command, commandArgs);
  return `${result.stdout ?? ""}${result.stderr ?? ""}`;
}

function succeeds(command, commandArgs) {
  return run(command, commandArgs).status === 0;
}

function inspectApp(path, requireDistributionChecks) {
  const details = combinedOutput("codesign", ["-dv", "--verbose=4", path]);
  const requirement = combinedOutput("codesign", ["-dr", "-", path]);
  return {
    ...parseCodeSignDetails(details, requirement),
    codeValid: succeeds("codesign", ["--verify", "--deep", "--strict", path]),
    gatekeeperAccepted: requireDistributionChecks
      ? succeeds("spctl", ["--assess", "--type", "execute", path])
      : true,
    notarizationStapled: requireDistributionChecks
      ? succeeds("xcrun", ["stapler", "validate", path])
      : true
  };
}

function fail(errors) {
  for (const error of errors) console.error(`- ${error}`);
  process.exit(1);
}

const unknownOptions = args.filter((arg) =>
  arg.startsWith("--") && arg !== "--replace-ad-hoc"
);
if (unknownOptions.length > 0) {
  fail([`Unknown option: ${unknownOptions.join(", ")}`]);
}
if (!paths[0] || !existsSync(source) || !statSync(source).isDirectory()) {
  console.error(
    "Usage: npm run install:macos -- /path/to/Pebble.app [/Applications/Pebble.app] [--replace-ad-hoc]"
  );
  process.exit(1);
}
if (basename(source) !== "Pebble.app" || basename(destination) !== "Pebble.app") {
  fail(["Both source and destination must be named Pebble.app."]);
}
if (source === destination) {
  fail(["Source and destination must be different app bundles."]);
}

const candidate = inspectApp(source, true);
const hadDestination = existsSync(destination);
if (
  hadDestination &&
  succeeds("pgrep", ["-f", `${destination}/Contents/MacOS/pebble`])
) {
  fail(["Quit Pebble before installing an update."]);
}
const installed = hadDestination
  ? inspectApp(destination, false)
  : null;
const errors = validateInstallCandidate({
  candidate,
  installed,
  allowAdHocReplacement
});
if (errors.length > 0) fail(errors);

const operationId = randomUUID();
const staging = `${destination}.installing-${operationId}`;
const backup = `${destination}.backup-${operationId}`;
rmSync(staging, { recursive: true, force: true });
rmSync(backup, { recursive: true, force: true });

try {
  cpSync(source, staging, { recursive: true, preserveTimestamps: true });
  const stagedErrors = validateInstallCandidate({
    candidate: inspectApp(staging, true),
    installed,
    allowAdHocReplacement
  });
  if (stagedErrors.length > 0) throw new Error(stagedErrors.join("\n"));

  if (existsSync(destination)) renameSync(destination, backup);
  renameSync(staging, destination);

  const finalErrors = validateInstallCandidate({
    candidate: inspectApp(destination, true),
    allowAdHocReplacement: true
  });
  if (finalErrors.length > 0) throw new Error(finalErrors.join("\n"));
  rmSync(backup, { recursive: true, force: true });
  console.log(`Installed verified Pebble at ${destination}.`);
} catch (error) {
  rmSync(staging, { recursive: true, force: true });
  if (existsSync(backup)) {
    rmSync(destination, { recursive: true, force: true });
    renameSync(backup, destination);
  } else if (!hadDestination) {
    rmSync(destination, { recursive: true, force: true });
  }
  console.error(`Pebble installation failed safely: ${error.message}`);
  process.exit(1);
}
