import assert from "node:assert/strict";
import test from "node:test";

import { validateMacosRelease } from "./macos-release-policy.mjs";

function releaseInput(overrides = {}) {
  return {
    packageVersion: "0.2.0",
    packageLockVersion: "0.2.0",
    cargoVersion: "0.2.0",
    tauriConfig: {
      version: "0.2.0",
      identifier: "com.ohenry.screenpebble",
      bundle: {
        active: false,
        externalBin: ["binaries/codex"],
        targets: ["app", "dmg"],
        macOS: {
          hardenedRuntime: true,
          minimumSystemVersion: "14.0"
        }
      }
    },
    ...overrides
  };
}

test("accepts the checked-in unsigned development configuration", () => {
  assert.deepEqual(validateMacosRelease(releaseInput()), []);
});

test("rejects version drift and ad-hoc public signing", () => {
  const input = releaseInput({ cargoVersion: "0.1.0", packageLockVersion: "0.1.0" });
  input.tauriConfig.bundle.macOS.signingIdentity = "-";

  const errors = validateMacosRelease(input);

  assert.ok(errors.some((error) => error.includes("versions must match")));
  assert.ok(errors.some((error) => error.includes("package-lock.json")));
  assert.ok(errors.some((error) => error.includes("ad-hoc")));
});

test("rejects changes to every protected bundle setting", () => {
  const cases = [
    [
      "bundle identifier",
      (input) => {
        input.tauriConfig.identifier = "com.example.pebble";
      }
    ],
    [
      "app target",
      (input) => {
        input.tauriConfig.bundle.targets = ["dmg"];
      }
    ],
    [
      "dmg target",
      (input) => {
        input.tauriConfig.bundle.targets = ["app"];
      }
    ],
    [
      "Codex sidecar",
      (input) => {
        input.tauriConfig.bundle.externalBin = [];
      }
    ],
    [
      "minimum macOS version",
      (input) => {
        input.tauriConfig.bundle.macOS.minimumSystemVersion = "13.0";
      }
    ],
    [
      "hardened runtime",
      (input) => {
        input.tauriConfig.bundle.macOS.hardenedRuntime = false;
      }
    ]
  ];

  for (const [name, mutate] of cases) {
    const input = releaseInput();
    mutate(input);
    assert.notDeepEqual(validateMacosRelease(input), [], `${name} must be protected`);
  }
});

test("requires every signing and notarization value in release mode", () => {
  const errors = validateMacosRelease({
    ...releaseInput(),
    requireSigning: true,
    environment: {},
    apiKeyFile: null
  });

  assert.ok(errors.some((error) => error.includes("APPLE_API_ISSUER")));
  assert.ok(errors.some((error) => error.includes("APPLE_API_KEY_PATH")));
});

test("accepts a Developer ID identity and private App Store Connect key", () => {
  const value = "configured";
  const errors = validateMacosRelease({
    ...releaseInput(),
    requireSigning: true,
    environment: {
      APPLE_API_ISSUER: value,
      APPLE_API_KEY: value,
      APPLE_API_KEY_PATH: "/private/tmp/AuthKey_TEST.p8",
      APPLE_SIGNING_IDENTITY: "Developer ID Application: Pebble (TEAMID)"
    },
    apiKeyFile: { isFile: true, mode: 0o100600 }
  });

  assert.deepEqual(errors, []);
});

test("rejects a private key readable by other users", () => {
  const value = "configured";
  const errors = validateMacosRelease({
    ...releaseInput(),
    requireSigning: true,
    environment: {
      APPLE_API_ISSUER: value,
      APPLE_API_KEY: value,
      APPLE_API_KEY_PATH: "/private/tmp/AuthKey_TEST.p8",
      APPLE_SIGNING_IDENTITY: "Developer ID Application: Pebble (TEAMID)"
    },
    apiKeyFile: { isFile: true, mode: 0o100644 }
  });

  assert.ok(errors.some((error) => error.includes("group/world")));
});
