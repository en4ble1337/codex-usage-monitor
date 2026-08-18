import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { SettingsWindow } from "./windows/SettingsWindow";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <SettingsWindow />
  </React.StrictMode>,
);
