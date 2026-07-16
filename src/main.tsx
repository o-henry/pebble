import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import "./styles/app.css";
import "./styles/claude-credential.css";
import "./styles/smart-watch.css";
import "./styles/watch-recipes.css";
import "./styles/update-feed.css";
import "./styles/live-tile.css";
import "./styles/region-selector.css";
import "./styles/window-shell.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("Pebble root element was not found.");
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
