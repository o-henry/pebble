import { MainView } from "./MainView";
import { TileView } from "./TileView";

export function App() {
  return getAppView() === "tile" ? <TileView /> : <MainView />;
}

function getAppView(): "main" | "tile" {
  return globalThis.location?.hash === "#tile" ? "tile" : "main";
}
