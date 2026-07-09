import {
  ISSUE_MESSAGES,
  type PhysicalRegion,
  type RegionSelectionIssue,
  type RegionSelectionIssueCode
} from "./regionSelectionTypes";

export function issue(
  code: RegionSelectionIssueCode,
  limit: number,
  actual: number
): RegionSelectionIssue {
  return {
    code,
    limit: finiteOrZero(limit),
    actual: finiteOrZero(actual),
    message: ISSUE_MESSAGES[code]
  };
}

export function isRegionSelectionIssue(
  value: number | [number, number] | PhysicalRegion | RegionSelectionIssue
): value is RegionSelectionIssue {
  return typeof value === "object" && value !== null && "code" in value;
}

function finiteOrZero(value: number): number {
  return Number.isFinite(value) ? value : 0;
}
