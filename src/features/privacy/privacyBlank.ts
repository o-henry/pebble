export type PrivacyBlankAction = { type: "blank" } | { type: "restore" };

export interface PrivacyBlankState {
  blankActive: boolean;
  lastAction: "none" | "blanked" | "restored";
  hotkeyPermission: "notRequested";
}

export interface PrivacyBannerView {
  status: "blanked" | "ready";
  title: string;
  actionLabel: string;
  captureLabel: string;
}

export interface PrivacyHotkeyEvent {
  key: string;
  shiftKey: boolean;
  metaKey?: boolean;
  ctrlKey?: boolean;
}

export const PRIVACY_BLANK_INITIAL_STATE: PrivacyBlankState = {
  blankActive: false,
  lastAction: "none",
  hotkeyPermission: "notRequested"
};

export function privacyBlankReducer(
  state: PrivacyBlankState,
  action: PrivacyBlankAction
): PrivacyBlankState {
  switch (action.type) {
    case "blank":
      return {
        ...state,
        blankActive: true,
        lastAction: "blanked"
      };
    case "restore":
      return {
        ...state,
        blankActive: false,
        lastAction: "restored"
      };
  }
}

export function privacyBannerView(
  state: PrivacyBlankState
): PrivacyBannerView {
  if (state.blankActive) {
    return {
      status: "blanked",
      title: "Privacy blank active",
      actionLabel: "Restore tiles",
      captureLabel: "capture stopped"
    };
  }

  return {
    status: "ready",
    title: "Privacy blank ready",
    actionLabel: "Blank all tiles",
    captureLabel: "capture allowed for live tiles"
  };
}

export function privacyHotkeyAction(
  event: PrivacyHotkeyEvent,
  state: PrivacyBlankState
): PrivacyBlankAction | null {
  const modifierPressed = event.metaKey === true || event.ctrlKey === true;

  if (!modifierPressed || !event.shiftKey || event.key.toLowerCase() !== "b") {
    return null;
  }

  return state.blankActive ? { type: "restore" } : { type: "blank" };
}
