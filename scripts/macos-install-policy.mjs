import { PEBBLE_BUNDLE_ID } from "./macos-release-policy.mjs";

function valueFor(text, key) {
  const match = text.match(new RegExp(`^${key}=(.+)$`, "m"));
  return match?.[1]?.trim() ?? null;
}

export function parseCodeSignDetails(details, designatedRequirement = "") {
  const flags = details.match(/^CodeDirectory .* flags=\S+\(([^)]*)\)/m)?.[1]
    ?.split(",")
    .map((flag) => flag.trim())
    .filter(Boolean) ?? [];
  const authorities = [...details.matchAll(/^Authority=(.+)$/gm)]
    .map((match) => match[1].trim());

  return {
    identifier: valueFor(details, "Identifier"),
    teamIdentifier: valueFor(details, "TeamIdentifier"),
    flags,
    authorities,
    designatedRequirement: designatedRequirement
      .replace(/^.*designated =>\s*/s, "")
      .trim()
  };
}

export function validateInstallCandidate({
  candidate,
  installed = null,
  allowAdHocReplacement = false
}) {
  const errors = [];

  if (!candidate.codeValid) {
    errors.push("The candidate app does not have a valid sealed code signature.");
  }
  if (candidate.identifier !== PEBBLE_BUNDLE_ID) {
    errors.push(`The candidate bundle identifier must be ${PEBBLE_BUNDLE_ID}.`);
  }
  if (!candidate.teamIdentifier || candidate.teamIdentifier === "not set") {
    errors.push("The candidate app must have a stable TeamIdentifier.");
  }
  if (candidate.flags.includes("adhoc")) {
    errors.push("Ad-hoc apps must never replace the installed Pebble app.");
  }
  if (!candidate.authorities.some((authority) =>
    authority.startsWith("Developer ID Application:")
  )) {
    errors.push("The candidate app must be signed with Developer ID Application.");
  }
  if (!candidate.designatedRequirement.includes("anchor apple generic")) {
    errors.push("The candidate app must use an Apple-anchored designated requirement.");
  }
  if (candidate.designatedRequirement.includes("cdhash")) {
    errors.push("The candidate designated requirement must not depend on a changing cdhash.");
  }
  if (!candidate.gatekeeperAccepted) {
    errors.push("Gatekeeper must accept the candidate app.");
  }
  if (!candidate.notarizationStapled) {
    errors.push("The candidate app must contain a valid notarization ticket.");
  }

  if (installed) {
    if (installed.identifier && installed.identifier !== candidate.identifier) {
      errors.push("The installed app and candidate must use the same bundle identifier.");
    }
    const installedIsAdHoc = installed.flags.includes("adhoc") ||
      !installed.teamIdentifier ||
      installed.teamIdentifier === "not set";
    if (installedIsAdHoc && !allowAdHocReplacement) {
      errors.push(
        "The installed app is ad-hoc signed. Replacing it requires explicit acknowledgement because macOS will request Screen Recording again."
      );
    }
    if (
      !installedIsAdHoc &&
      installed.teamIdentifier !== candidate.teamIdentifier
    ) {
      errors.push("The installed app and candidate must use the same TeamIdentifier.");
    }
  }

  return errors;
}
