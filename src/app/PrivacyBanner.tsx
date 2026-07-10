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
      className={"privacy-banner " + (state.blankActive ? "is-blanked" : "")}
      aria-live="polite"
      aria-label="Privacy blank"
    >
      <div className="privacy-banner__intro">
        <span className="privacy-indicator" aria-hidden="true" />
        <div>
          <p className="section-label">Privacy control</p>
          <h2>{view.title}</h2>
        </div>
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
          <dt>Retention</dt>
          <dd>memory only</dd>
        </div>
      </dl>
      <button
        type="button"
        className={
          "privacy-action " + (state.blankActive ? "is-restoring" : "")
        }
        onClick={onAction}
      >
        {view.actionLabel}
      </button>
    </section>
  );
}
