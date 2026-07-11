import { useEffect, useState } from "react";
import { RegionSelectorView } from "./RegionSelectorView";
import { TileView } from "./TileView";

export function App() {
  const [view, setView] = useState(getAppView);

  useEffect(() => {
    function handleHashChange() {
      setView(getAppView());
    }

    globalThis.addEventListener("hashchange", handleHashChange);
    return () => globalThis.removeEventListener("hashchange", handleHashChange);
  }, []);

  if (view === "selector") {
    return <RegionSelectorView />;
  }

  return <TileView />;
}

function getAppView(): "selector" | "tile" {
  if (globalThis.location?.hash === "#selector") {
    return "selector";
  }

  return "tile";
}
