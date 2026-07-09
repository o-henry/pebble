import type {
  DragRect,
  RegionSelectorState
} from "../features/region-selector/regionSelectorInteraction";

export function SelectorHud({
  status,
  dimensions
}: {
  status: string;
  dimensions: string;
}) {
  return (
    <aside className="selector-hud">
      <p className="status-line">selector · capture off</p>
      <h1>Select Region</h1>
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
    </aside>
  );
}

export function SelectionBox({ rect }: { rect: DragRect }) {
  return (
    <div
      className="selector-box"
      style={{
        transform: `translate(${rect.x}px, ${rect.y}px)`,
        width: `${rect.width}px`,
        height: `${rect.height}px`
      }}
    />
  );
}

export function SelectorResult({ state }: { state: RegionSelectorState }) {
  if (state.status === "cancelled") {
    return <output className="selector-result">Cancelled</output>;
  }

  if (!state.result) {
    return <output className="selector-result">Physical region pending</output>;
  }

  if (!state.result.ok) {
    return (
      <output className="selector-result error">
        {state.result.error.message}
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
      {region.width} x {region.height} · x {region.x} · y {region.y}
      {warnings.length > 0 ? <span>{warnings[0].message}</span> : null}
    </output>
  );
}
