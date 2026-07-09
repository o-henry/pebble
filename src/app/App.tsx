import { appStatus, docReferences, principles } from "./appContent";
import { PERFORMANCE_LIMITS } from "../features/performance/performanceLimits";

export function App() {
  return (
    <main className="app-shell">
      <section className="hero-section" aria-labelledby="screenpebble-title">
        <p className="status-line">
          {appStatus.phase} · scaffold ready · capture off · AI off
        </p>
        <h1 id="screenpebble-title">ScreenPebble</h1>
        <p className="hero-copy">
          Pin a tiny part of your screen. Let local watchers notice what changed.
        </p>
        <p className="trust-copy">
          This build includes the desktop scaffold and hard performance limits.
          There is no screen capture, OCR, AI connector, telemetry, or network
          feature in this build.
        </p>
      </section>

      <section className="principles-grid" aria-label="Product principles">
        {principles.map((principle) => (
          <article className="principle-card" key={principle.title}>
            <h2>{principle.title}</h2>
            <p>{principle.body}</p>
          </article>
        ))}
      </section>

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
    </main>
  );
}
