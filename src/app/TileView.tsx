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
      <div className="tile-topbar">
        <p className="status-line">test tile · capture off</p>
        <span className="tile-mode">{TILE_VIEW_STATE.mode}</span>
      </div>
      <section className="tile-placeholder">
        <h1 id="tile-title">{TILE_VIEW_STATE.title}</h1>
        <p>{TILE_VIEW_STATE.placeholder}</p>
        <div className="fake-frame" aria-hidden="true">
          <span />
          <span />
          <span />
        </div>
      </section>
    </main>
  );
}
