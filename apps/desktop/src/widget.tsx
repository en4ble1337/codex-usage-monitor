import React from "react";
import { createRoot } from "react-dom/client";
import "./styles.css";
import { WidgetWindow } from "./windows/WidgetWindow";

createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <WidgetWindow />
  </React.StrictMode>,
);
