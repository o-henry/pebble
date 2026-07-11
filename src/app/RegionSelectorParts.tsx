import type {
  DragRect,
  RegionSelectorState
} from "../features/region-selector/regionSelectorInteraction";

export function SelectorHud({
  status,
  onCancel
}: {
  status: string;
  onCancel: () => void;
}) {
  return (
    <aside
      className="selector-hud"
      onPointerDown={(event) => event.stopPropagation()}
    >
      <div className="selector-hud__header">
        <div>
          <p className="status-line">pebble</p>
          <h1>Select a region</h1>
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
      <p className="selector-hud__state">{selectorInstruction(status)}</p>
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

export function SelectorResult({
  state,
  committing,
  error
}: {
  state: RegionSelectorState;
  committing: boolean;
  error: string | null;
}) {
  if (error) {
    return (
      <aside className="selector-result error" role="alert">
        <span>Could not start</span>
        <strong>{error}</strong>
        <span>Drag another region to try again.</span>
      </aside>
    );
  }

  if (state.status === "cancelled") {
    return null;
  }

  if (!state.result) {
    return null;
  }

  if (!state.result.ok) {
    return (
      <aside className="selector-result error" role="alert">
        <span>Choose another region</span>
        <strong>{state.result.error.message}</strong>
      </aside>
    );
  }

  const { warnings } = state.result.selection;

  return (
    <aside
      className={
        warnings.length > 0 ? "selector-result warning" : "selector-result"
      }
      aria-live="polite"
    >
      <strong>{committing ? "Starting Pebble" : "Region selected"}</strong>
      {warnings.length > 0 ? <span>{warnings[0].message}</span> : null}
    </aside>
  );
}

function selectorInstruction(status: string) {
  switch (status) {
    case "dragging":
      return "Release to start watching";
    case "ready":
      return "Opening your floating pebble";
    default:
      return "Drag over the part you keep checking";
  }
}
