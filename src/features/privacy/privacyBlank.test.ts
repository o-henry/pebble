import { describe, expect, it } from "vitest";
import {
  PRIVACY_BLANK_INITIAL_STATE,
  privacyBannerView,
  privacyBlankReducer,
  privacyHotkeyAction
} from "./privacyBlank";

describe("privacy blank state", () => {
  it("shows blank state in the banner model", () => {
    const blanked = privacyBlankReducer(PRIVACY_BLANK_INITIAL_STATE, {
      type: "blank"
    });

    expect(privacyBannerView(blanked)).toMatchObject({
      status: "blanked",
      actionLabel: "Restore tiles",
      captureLabel: "capture stopped"
    });
  });

  it("restores from blank state without requesting hotkey permissions", () => {
    const blanked = privacyBlankReducer(PRIVACY_BLANK_INITIAL_STATE, {
      type: "blank"
    });
    const restored = privacyBlankReducer(blanked, { type: "restore" });

    expect(restored).toMatchObject({
      blankActive: false,
      lastAction: "restored",
      hotkeyPermission: "notRequested"
    });
  });

  it("maps focused-window hotkey to blank and restore actions only", () => {
    expect(
      privacyHotkeyAction(
        { key: "b", shiftKey: true, metaKey: true },
        PRIVACY_BLANK_INITIAL_STATE
      )
    ).toEqual({ type: "blank" });

    const blanked = privacyBlankReducer(PRIVACY_BLANK_INITIAL_STATE, {
      type: "blank"
    });

    expect(
      privacyHotkeyAction({ key: "b", shiftKey: true, ctrlKey: true }, blanked)
    ).toEqual({ type: "restore" });
    expect(
      privacyHotkeyAction({ key: "b", shiftKey: false, metaKey: true }, blanked)
    ).toBeNull();
  });
});
