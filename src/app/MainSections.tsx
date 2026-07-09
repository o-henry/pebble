import { docReferences, principles } from "./appContent";
import { PERFORMANCE_LIMITS } from "../features/performance/performanceLimits";
import type { TileWindowState } from "../features/window-shell/tileWindowState";

export function Principles() {
  return (
    <section className="principles-grid" aria-label="Product principles">
      {principles.map((principle) => (
        <article className="principle-card" key={principle.title}>
          <h2>{principle.title}</h2>
          <p>{principle.body}</p>
        </article>
      ))}
    </section>
  );
}

export function PerformanceLimits() {
  return (
    <section className="limits-section" aria-labelledby="limits-title">
      <div>
        <p className="section-label">Performance contract</p>
        <h2 id="limits-title">Low FPS by default</h2>
      </div>
      <dl className="limit-list">
        <div>
          <dt>Default refresh</dt>
          <dd>{PERFORMANCE_LIMITS.defaultFps} FPS</dd>
        </div>
        <div>
          <dt>Maximum refresh</dt>
          <dd>{PERFORMANCE_LIMITS.maxFps} FPS</dd>
        </div>
        <div>
          <dt>Active tiles</dt>
          <dd>{PERFORMANCE_LIMITS.maxActiveTiles}</dd>
        </div>
        <div>
          <dt>Hard max region</dt>
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
    <section className="window-section" aria-labelledby="window-shell-title">
      <div>
        <p className="section-label">Window shell</p>
        <h2 id="window-shell-title">Test tile window</h2>
      </div>
      <div className="window-shell-panel">
        <div>
          <p className="tile-name">{tile.title}</p>
          <p className="tile-description">{tile.placeholder}</p>
        </div>
        <dl className="tile-state-list">
          <div>
            <dt>Mode</dt>
            <dd>{tile.mode}</dd>
          </div>
          <div>
            <dt>Always on top</dt>
            <dd>{tile.alwaysOnTop ? "enabled" : "disabled"}</dd>
          </div>
          <div>
            <dt>Capture</dt>
            <dd>{tile.captureActive ? "shell live" : "off"}</dd>
          </div>
        </dl>
        <div className="window-actions">
          <button type="button" onClick={onOpen}>
            Open test tile
          </button>
          <button type="button" className="secondary-action" onClick={onRefresh}>
            Refresh state
          </button>
        </div>
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
