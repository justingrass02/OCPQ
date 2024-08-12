import "$/index.css";
import React from "react";
import ReactDOM from "react-dom/client";
import { Toaster } from "react-hot-toast";
import { MainRouterProvider } from "$/router";
import {
  type BackendProvider,
  BackendProviderContext,
} from "$/BackendProviderContext";
import { dialog, invoke } from "@tauri-apps/api";
import type {
  EventTypeQualifiers,
  OCELInfo,
  ObjectTypeQualifiers,
} from "$/types/ocel";
import type { DiscoverConstraintsResponse } from "$/routes/visual-editor/helper/types";

const tauriBackend: BackendProvider = {
  "ocel/info": async () => {
    const ocelInfo: OCELInfo = await invoke("get_current_ocel_info");
    return ocelInfo;
  },
  "ocel/picker": async () => {
    const path = await dialog.open({
      title: "Select an OCEL2 file",
      filters: [{ name: "OCEL2", extensions: ["json", "xml"] }],
    });
    if (typeof path === "string") {
      const ocelInfo: OCELInfo = await invoke("import_ocel", { path });
      return ocelInfo;
    }
    throw new Error("No file selected");
  },

  "ocel/check-constraints-box": async (tree,measurePerformance) => {
    return await invoke("check_with_box_tree", { req: { tree, measurePerformance } });
  },
  "ocel/event-qualifiers": async () => {
    return await invoke<EventTypeQualifiers>("get_event_qualifiers");
  },
  "ocel/object-qualifiers": async () => {
    return await invoke<ObjectTypeQualifiers>("get_object_qualifiers");
  },
  "ocel/discover-constraints": async (options) => {
    return await invoke<DiscoverConstraintsResponse>(
      "auto_discover_constraints",
      { options },
    );
  },
  "ocel/graph": async (options) => {
    return await invoke("ocel_graph", { options });
  },
};

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <BackendProviderContext.Provider value={tauriBackend}>
      <Toaster position="bottom-left" />
      <MainRouterProvider />
    </BackendProviderContext.Provider>
  </React.StrictMode>,
);
