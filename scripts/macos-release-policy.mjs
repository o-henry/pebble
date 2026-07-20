export const PEBBLE_BUNDLE_ID = "com.ohenry.screenpebble";
export const MACOS_MINIMUM_VERSION = "14.0";

const REQUIRED_TARGETS = ["app", "dmg"];
const REQUIRED_SIGNING_ENV = [
  "APPLE_API_ISSUER",
  "APPLE_API_KEY",
  "APPLE_API_KEY_PATH",
  "APPLE_SIGNING_IDENTITY"
];

function hasText(value) {
  return typeof value === "string" && value.trim().length > 0;
}

export function validateMacosRelease({
  packageVersion,
  packageLockVersion,
  cargoVersion,
  tauriConfig,
  environment = {},
  requireSigning = false,
  apiKeyFile = null
}) {
  const errors = [];
  const bundle = tauriConfig?.bundle;
  const macos = bundle?.macOS;

  if (!hasText(packageVersion) || packageVersion !== cargoVersion) {
    errors.push("package.json and Cargo.toml versions must match.");
  }
  if (packageLockVersion !== packageVersion) {
    errors.push("package-lock.json version must match package.json.");
  }
  if (tauriConfig?.version !== packageVersion) {
    errors.push("tauri.conf.json version must match package.json.");
  }
  if (tauriConfig?.identifier !== PEBBLE_BUNDLE_ID) {
    errors.push(`The bundle identifier must remain ${PEBBLE_BUNDLE_ID}.`);
  }
  if (!Array.isArray(bundle?.targets)) {
    errors.push("bundle.targets must explicitly list app and dmg.");
  } else {
    for (const target of REQUIRED_TARGETS) {
      if (!bundle.targets.includes(target)) {
        errors.push(`bundle.targets must include ${target}.`);
      }
    }
  }
  if (!bundle?.externalBin?.includes("binaries/codex")) {
    errors.push("The fixed Codex sidecar must remain in bundle.externalBin.");
  }
  if (macos?.minimumSystemVersion !== MACOS_MINIMUM_VERSION) {
    errors.push(`macOS minimumSystemVersion must be ${MACOS_MINIMUM_VERSION}.`);
  }
  if (macos?.hardenedRuntime !== true) {
    errors.push("macOS hardenedRuntime must be enabled.");
  }
  if (macos?.signingIdentity === "-") {
    errors.push("Public releases must never use an ad-hoc signing identity.");
  }

  if (requireSigning) {
    for (const name of REQUIRED_SIGNING_ENV) {
      if (!hasText(environment[name])) {
        errors.push(`Missing required release secret or environment value: ${name}.`);
      }
    }
    if (
      hasText(environment.APPLE_SIGNING_IDENTITY) &&
      !environment.APPLE_SIGNING_IDENTITY.startsWith("Developer ID Application:")
    ) {
      errors.push("APPLE_SIGNING_IDENTITY must be a Developer ID Application identity.");
    }
    if (hasText(environment.APPLE_API_KEY_PATH)) {
      if (!apiKeyFile?.isFile) {
        errors.push("APPLE_API_KEY_PATH must point to a regular file.");
      } else if ((apiKeyFile.mode & 0o077) !== 0) {
        errors.push("The App Store Connect private key must not be group/world accessible.");
      }
    }
  }

  return errors;
}
