import { docReferences, principles } from "./appContent";
import { PERFORMANCE_LIMITS } from "../features/performance/performanceLimits";
import type { TileWindowState } from "../features/window-shell/tileWindowState";

export function Principles() {
  return (
    <section className="principles-section" aria-labelledby="principles-title">
      <div className="section-heading">
        <div>
          <p className="section-label">Trust model</p>
          <h2 id="principles-title">Guardrails remain on</h2>
        </div>
        <span className="section-heading__note">Local by default</span>
      </div>
      <div className="principles-grid">
        {principles.map((principle, index) => (
          <article className="principle-card" key={principle.title}>
            <span className="principle-index">0{index + 1}</span>
            <h3>{principle.title}</h3>
            <p>{principle.body}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

export function PerformanceLimits() {
  return (
    <section
      className="limits-section compact-panel"
      aria-labelledby="limits-title"
    >
      <div className="panel-heading">
        <div>
          <p className="section-label">Performance</p>
          <h2 id="limits-title">Hard limits</h2>
        </div>
        <span className="panel-index" aria-hidden="true">03</span>
      </div>
      <dl className="limit-list">
        <div>
          <dt>Default</dt>
          <dd>{PERFORMANCE_LIMITS.defaultFps} FPS</dd>
        </div>
        <div>
          <dt>Ceiling</dt>
          <dd>{PERFORMANCE_LIMITS.maxFps} FPS</dd>
        </div>
        <div>
          <dt>Tiles</dt>
          <dd>{PERFORMANCE_LIMITS.maxActiveTiles}</dd>
        </div>
        <div>
          <dt>Region</dt>
          <dd>
            {PERFORMANCE_LIMITS.maxRegion.width}x
            {PERFORMANCE_LIMITS.maxRegion.height}
          </dd>
        </div>
      </dl>
    </section>
  );
}

export function WindowShellControls({
  tile,
  onOpen,
  onRefresh
}: {
  tile: TileWindowState;
  onOpen: () => void;
  onRefresh: () => void;
}) {
  return (
    <section
      className="window-section compact-panel"
      aria-labelledby="window-shell-title"
    >
      <div className="panel-heading">
        <div>
          <p className="section-label">Tile shell</p>
          <h2 id="window-shell-title">Test tile</h2>
        </div>
        <span className={"mode-badge is-" + tile.mode}>{tile.mode}</span>
      </div>
      <p className="tile-description">{tile.title}</p>
      <dl className="tile-state-list">
        <div>
          <dt>Topmost</dt>
          <dd>{tile.alwaysOnTop ? "On" : "Off"}</dd>
        </div>
        <div>
          <dt>Capture</dt>
          <dd>{tile.captureActive ? "Live" : "Off"}</dd>
        </div>
      </dl>
      <div className="window-actions">
        <button type="button" className="secondary-action" onClick={onOpen}>
          Open tile
        </button>
        <button type="button" className="text-action" onClick={onRefresh}>
          Refresh
        </button>
      </div>
    </section>
  );
}

export function Docs() {
  return (
    <section className="docs-section" aria-labelledby="docs-title">
      <div>
        <p className="section-label">Implementation start point</p>
        <h2 id="docs-title">Read before changing code</h2>
      </div>
      <ul className="doc-list">
        {docReferences.map((doc) => (
          <li key={doc.path}>
            <strong className="doc-title">{doc.label}</strong>
            <code className="doc-path">{doc.path}</code>
            <span>{doc.description}</span>
          </li>
        ))}
      </ul>
    </section>
  );
}
