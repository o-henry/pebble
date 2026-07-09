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
    <section className="selector-section" aria-labelledby="selector-title">
      <div>
        <p className="section-label">Region selector</p>
        <h2 id="selector-title">Transparent overlay shell</h2>
      </div>
      <div className="selector-panel">
        <dl className="selector-state-list">
          <div>
            <dt>Overlay</dt>
            <dd>{shell.visualOverlay ? "transparent" : "plain"}</dd>
          </div>
          <div>
            <dt>Always on top</dt>
            <dd>{shell.alwaysOnTop ? "enabled" : "disabled"}</dd>
          </div>
          <div>
            <dt>Capture</dt>
            <dd>{shell.captureActive ? "on" : "off"}</dd>
          </div>
        </dl>
        {error ? <p className="selector-error">{error}</p> : null}
        <div className="window-actions">
          <button type="button" onClick={onOpen}>
            Open selector overlay
          </button>
        </div>
      </div>
    </section>
  );
}
