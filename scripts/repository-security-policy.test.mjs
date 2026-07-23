import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import test from "node:test";

const workflowsDirectory = new URL("../.github/workflows/", import.meta.url);

function read(relativePath) {
  return readFileSync(new URL(`../${relativePath}`, import.meta.url), "utf8");
}

test("every third-party GitHub Action is pinned to a full commit SHA", () => {
  for (const fileName of readdirSync(workflowsDirectory)) {
    if (!fileName.endsWith(".yml") && !fileName.endsWith(".yaml")) continue;
    const workflow = read(`.github/workflows/${fileName}`);
    for (const [, action, revision] of workflow.matchAll(
      /^\s*uses:\s*([^@\s]+)@([^\s#]+)/gm
    )) {
      assert.match(
        revision,
        /^[0-9a-f]{40}$/,
        `${fileName}: ${action} must be pinned to a full commit SHA`
      );
    }
  }
});

test("pull requests run application, CodeQL, and full-history secret checks", () => {
  const ci = read(".github/workflows/ci.yml");
  const codeql = read(".github/workflows/codeql.yml");
  const secrets = read(".github/workflows/secret-scan.yml");

  assert.match(ci, /^\s{2}pull_request:\s*$/m);
  assert.match(ci, /^\s{2}frontend:\s*$/m);
  assert.match(ci, /^\s{2}rust:\s*$/m);
  assert.match(codeql, /^\s{2}pull_request:\s*$/m);
  assert.match(codeql, /language:\s+javascript-typescript/);
  assert.match(codeql, /language:\s+rust/);
  assert.match(secrets, /^\s{2}pull_request:\s*$/m);
  assert.match(secrets, /fetch-depth:\s+0/);
  assert.match(secrets, /gitleaks\/gitleaks-action@[0-9a-f]{40}/);
});

test("release secrets are gated by the release environment", () => {
  const release = read(".github/workflows/release-macos.yml");

  assert.match(release, /^\s{4}environment:\s+release\s*$/m);
  assert.doesNotMatch(release, /permissions:\s+write-all/);
});

test("the repository publishes private vulnerability reporting guidance", () => {
  const policy = read("SECURITY.md");

  assert.match(policy, /security\/advisories\/new/);
  assert.match(policy, /Do not open a public issue/);
});
