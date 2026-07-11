import { useEffect, useState } from "react";

export function useDocumentVisibility() {
  const [visible, setVisible] = useState(() =>
    visibilityAllowsCapture(document.visibilityState)
  );

  useEffect(() => {
    const update = () => {
      setVisible(visibilityAllowsCapture(document.visibilityState));
    };
    document.addEventListener("visibilitychange", update);
    return () => document.removeEventListener("visibilitychange", update);
  }, []);

  return visible;
}

export function visibilityAllowsCapture(state: DocumentVisibilityState) {
  return state === "visible";
}
