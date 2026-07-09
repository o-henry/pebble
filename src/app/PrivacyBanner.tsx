import type { PrivacyBlankState } from "../features/privacy/privacyBlank";
import { privacyBannerView } from "../features/privacy/privacyBlank";

export function PrivacyBanner({
  state,
  onBlank,
  onRestore
}: {
  state: PrivacyBlankState;
  onBlank: () => void;
  onRestore: () => void;
}) {
  const view = privacyBannerView(state);
  const onAction = state.blankActive ? onRestore : onBlank;

  return (
    <section
      className={`privacy-banner ${state.blankActive ? "is-blanked" : ""}`}
      aria-live="polite"
      aria-label="Privacy blank"
    >
      <div>
        <p className="section-label">Privacy</p>
        <h2>{view.title}</h2>
      </div>
      <dl className="privacy-state-list">
        <div>
          <dt>Status</dt>
          <dd>{view.status}</dd>
        </div>
        <div>
          <dt>Capture</dt>
          <dd>{view.captureLabel}</dd>
        </div>
        <div>
          <dt>Hotkey</dt>
          <dd>{state.hotkeyPermission}</dd>
        </div>
      </dl>
      <button type="button" onClick={onAction}>
        {view.actionLabel}
      </button>
    </section>
  );
}
