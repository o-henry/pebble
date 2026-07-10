import { useEffect, useState } from "react";
import { MainView } from "./MainView";
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

  if (view === "tile") {
    return <TileView />;
  }

  if (view === "selector") {
    return <RegionSelectorView />;
  }

  return <MainView />;
}

function getAppView(): "main" | "selector" | "tile" {
  if (globalThis.location?.hash === "#tile") {
    return "tile";
  }

  if (globalThis.location?.hash === "#selector") {
    return "selector";
  }

  return "main";
}
