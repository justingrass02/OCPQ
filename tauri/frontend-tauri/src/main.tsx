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
  "ocel/available": async () => {
    return ["Select file manually..."];
  },
  "ocel/load": async (fileName: string) => {
    console.log(fileName);
    const path = await dialog.open({
      title: "Select an OCEL2 file",
      filters: [{ name: "OCEL2", extensions: ["json", "xml"] }],
    });
    if (typeof path === "string") {
      const ocelInfo: OCELInfo = await invoke("import_ocel", { path });
      return ocelInfo;
    }
    throw new Error("No valid OCEL path");
  },
  "ocel/check-constraints": async (variables, nodes) => {
    return await invoke("check_constraint_with_tree", { variables, nodes });
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
};

// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Toaster position="bottom-left" />
    <BackendProviderContext.Provider value={tauriBackend}>
      <MainRouterProvider />
    </BackendProviderContext.Provider>
  </React.StrictMode>,
);
