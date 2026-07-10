import type {
  DragRect,
  RegionSelectorState
} from "../features/region-selector/regionSelectorInteraction";

export function SelectorHud({
  status,
  dimensions,
  onCancel
}: {
  status: string;
  dimensions: string;
  onCancel: () => void;
}) {
  return (
    <aside
      className="selector-hud"
      onPointerDown={(event) => event.stopPropagation()}
    >
      <div className="selector-hud__header">
        <div>
          <p className="status-line">ScreenPebble</p>
          <h1>Region selector</h1>
        </div>
        <button
          type="button"
          className="selector-close"
          aria-label="Cancel selection"
          title="Cancel selection"
          onClick={onCancel}
        >
          ×
        </button>
      </div>
      <dl>
        <div>
          <dt>Status</dt>
          <dd>{status}</dd>
        </div>
        <div>
          <dt>Dimensions</dt>
          <dd>{dimensions}</dd>
        </div>
      </dl>
      <p className="selector-hud__state">{status}</p>
    </aside>
  );
}

export function SelectionBox({ rect }: { rect: DragRect }) {
  return (
    <div
      className="selector-box"
      style={{
        transform:
          "translate(" + String(rect.x) + "px, " + String(rect.y) + "px)",
        width: String(rect.width) + "px",
        height: String(rect.height) + "px"
      }}
    >
      <span className="selector-box__corner top-left" />
      <span className="selector-box__corner top-right" />
      <span className="selector-box__corner bottom-left" />
      <span className="selector-box__corner bottom-right" />
    </div>
  );
}

export function SelectorResult({ state }: { state: RegionSelectorState }) {
  if (state.status === "cancelled") {
    return (
      <output className="selector-result">
        <span>Selection</span>
        <strong>Cancelled</strong>
      </output>
    );
  }

  if (!state.result) {
    return (
      <output className="selector-result">
        <span>Physical region</span>
        <strong>Awaiting selection</strong>
      </output>
    );
  }

  if (!state.result.ok) {
    return (
      <output className="selector-result error">
        <span>Region limit</span>
        <strong>{state.result.error.message}</strong>
      </output>
    );
  }

  const { region, warnings } = state.result.selection;

  return (
    <output
      className={
        warnings.length > 0 ? "selector-result warning" : "selector-result"
      }
    >
      <span>Physical region</span>
      <strong>
        {region.width} x {region.height} · x {region.x} · y {region.y}
      </strong>
      {warnings.length > 0 ? <span>{warnings[0].message}</span> : null}
    </output>
  );
}
