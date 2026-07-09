import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./app/App";
import "./styles/app.css";
import "./styles/window-shell.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("ScreenPebble root element was not found.");
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>
);
