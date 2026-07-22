import assert from "node:assert/strict";
import test from "node:test";
import {
  parseCodeSignDetails,
  validateInstallCandidate
} from "./macos-install-policy.mjs";

const signedDetails = `Executable=/tmp/Pebble.app/Contents/MacOS/pebble
Identifier=com.ohenry.screenpebble
CodeDirectory v=20500 size=100 flags=0x10000(runtime) hashes=1+3 location=embedded
Authority=Developer ID Application: Pebble (TEAM123)
Authority=Developer ID Certification Authority
Authority=Apple Root CA
TeamIdentifier=TEAM123`;
const signedRequirement = `designated => identifier "com.ohenry.screenpebble" and anchor apple generic and certificate leaf[subject.OU] = TEAM123`;

function verifiedCandidate(overrides = {}) {
  return {
    ...parseCodeSignDetails(signedDetails, signedRequirement),
    codeValid: true,
    gatekeeperAccepted: true,
    notarizationStapled: true,
    ...overrides
  };
}

test("accepts a notarized Developer ID candidate", () => {
  assert.deepEqual(validateInstallCandidate({
    candidate: verifiedCandidate()
  }), []);
});

test("rejects ad-hoc, cdhash-bound, unnotarized candidates", () => {
  const candidate = verifiedCandidate({
    teamIdentifier: "not set",
    flags: ["adhoc", "runtime"],
    authorities: [],
    designatedRequirement: 'cdhash H"012345"',
    gatekeeperAccepted: false,
    notarizationStapled: false
  });
  const errors = validateInstallCandidate({ candidate });

  assert.ok(errors.some((error) => error.includes("TeamIdentifier")));
  assert.ok(errors.some((error) => error.includes("Ad-hoc")));
  assert.ok(errors.some((error) => error.includes("Developer ID")));
  assert.ok(errors.some((error) => error.includes("cdhash")));
  assert.ok(errors.some((error) => error.includes("Gatekeeper")));
  assert.ok(errors.some((error) => error.includes("notarization")));
});

test("rejects an update signed by a different team", () => {
  const errors = validateInstallCandidate({
    candidate: verifiedCandidate(),
    installed: verifiedCandidate({ teamIdentifier: "OTHERTEAM" })
  });
  assert.ok(errors.some((error) => error.includes("same TeamIdentifier")));
});

test("requires an explicit one-time transition from an ad-hoc install", () => {
  const installed = verifiedCandidate({
    teamIdentifier: "not set",
    flags: ["adhoc", "runtime"],
    authorities: [],
    designatedRequirement: 'cdhash H"012345"'
  });
  assert.ok(validateInstallCandidate({
    candidate: verifiedCandidate(),
    installed
  }).some((error) => error.includes("explicit acknowledgement")));
  assert.deepEqual(validateInstallCandidate({
    candidate: verifiedCandidate(),
    installed,
    allowAdHocReplacement: true
  }), []);
});

test("parses the identity fields used by the installer", () => {
  assert.deepEqual(parseCodeSignDetails(signedDetails, signedRequirement), {
    identifier: "com.ohenry.screenpebble",
    teamIdentifier: "TEAM123",
    flags: ["runtime"],
    authorities: [
      "Developer ID Application: Pebble (TEAM123)",
      "Developer ID Certification Authority",
      "Apple Root CA"
    ],
    designatedRequirement: signedRequirement.replace("designated => ", "")
  });
});
