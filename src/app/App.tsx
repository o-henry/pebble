import { appStatus, docReferences, principles } from "./appContent";

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
          Phase 0 is only the desktop scaffold. There is no screen capture, OCR,
          AI connector, telemetry, or network feature in this build.
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
