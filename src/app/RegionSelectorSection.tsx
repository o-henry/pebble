import type { RegionSelectorWindowShell } from "../features/region-selector/regionSelectorShell";

export function RegionSelectorSection({
  shell,
  error,
  onOpen
}: {
  shell: RegionSelectorWindowShell;
  error: string | null;
  onOpen: () => void;
}) {
  return (
    <section className="selector-section compact-panel" aria-labelledby="selector-title">
      <div className="panel-heading">
        <div>
          <p className="section-label">New observer</p>
          <h2 id="selector-title">Choose a region</h2>
        </div>
        <span className="panel-index" aria-hidden="true">01</span>
      </div>
      <div className="selector-geometry" aria-hidden="true">
        <span className="selector-geometry__frame" />
        <span className="selector-geometry__cursor" />
      </div>
      <dl className="selector-state-list">
        <div>
          <dt>Overlay</dt>
          <dd>{shell.visualOverlay ? "Clear" : "Plain"}</dd>
        </div>
        <div>
          <dt>Topmost</dt>
          <dd>{shell.alwaysOnTop ? "On" : "Off"}</dd>
        </div>
        <div>
          <dt>Capture</dt>
          <dd>{shell.captureActive ? "On" : "Off"}</dd>
        </div>
      </dl>
      {error ? <p className="selector-error">{error}</p> : null}
      <div className="window-actions">
        <button type="button" className="primary-action" onClick={onOpen}>
          Select region
        </button>
      </div>
    </section>
  );
}
