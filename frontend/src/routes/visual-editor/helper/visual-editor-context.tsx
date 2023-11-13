import { createContext } from "react";

export const VisualEditorContext = createContext<{
  mode: "normal" | "view-tree" | "readonly";
}>({
  mode: "normal",
});
