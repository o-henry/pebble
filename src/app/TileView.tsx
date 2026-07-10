import {
  TEST_TILE_DEFAULT_STATE,
  tileWindowReducer
} from "../features/window-shell/tileWindowState";

const TILE_VIEW_STATE = tileWindowReducer(TEST_TILE_DEFAULT_STATE, {
  type: "opened"
});

export function TileView() {
  return (
    <main className="tile-shell" aria-labelledby="tile-title">
      <header className="tile-topbar">
        <div className="tile-brand">
          <span className="brand-mark" aria-hidden="true">
            <span />
          </span>
          <span>ScreenPebble</span>
        </div>
        <span className={"mode-badge is-" + TILE_VIEW_STATE.mode}>
          {TILE_VIEW_STATE.mode}
        </span>
      </header>
      <section className="tile-placeholder">
        <div className="tile-placeholder__copy">
          <p className="section-label">Test tile</p>
          <h1 id="tile-title">{TILE_VIEW_STATE.title}</h1>
          <p>Capture is off</p>
        </div>
        <dl className="tile-status-list">
          <div>
            <dt>Mode</dt>
            <dd>{TILE_VIEW_STATE.mode}</dd>
          </div>
          <div>
            <dt>Topmost</dt>
            <dd>{TILE_VIEW_STATE.alwaysOnTop ? "On" : "Off"}</dd>
          </div>
        </dl>
      </section>
    </main>
  );
}
