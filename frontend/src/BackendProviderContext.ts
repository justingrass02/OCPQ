import { createContext } from "react";
import type {
  DiscoverConstraintsRequest,
  DiscoverConstraintsResponse,
} from "./routes/visual-editor/helper/types";
import type { BindingBoxTree } from "./types/generated/BindingBoxTree";
import type { EvaluateBoxTreeResult } from "./types/generated/EvaluateBoxTreeResult";
import type { OCELGraphOptions } from "./types/generated/OCELGraphOptions";
import type {
  EventTypeQualifiers,
  OCELEvent,
  OCELInfo,
  OCELObject,
  ObjectTypeQualifiers
} from "./types/ocel";
export type BackendProvider = {
  "ocel/info": () => Promise<OCELInfo>;
  "ocel/upload"?: (file: File) => Promise<OCELInfo>;
  "ocel/available"?: () => Promise<string[]>;
  "ocel/load"?: (name: string) => Promise<OCELInfo>;
  "ocel/picker"?: () => Promise<OCELInfo>;
  "ocel/check-constraints-box": (
    tree: BindingBoxTree,
  ) => Promise<EvaluateBoxTreeResult>;
  "ocel/event-qualifiers": () => Promise<EventTypeQualifiers>;
  "ocel/object-qualifiers": () => Promise<ObjectTypeQualifiers>;
  "ocel/discover-constraints": (
    autoDiscoveryOptions: DiscoverConstraintsRequest,
  ) => Promise<DiscoverConstraintsResponse>;
  "ocel/graph": (options: OCELGraphOptions) => Promise<{
    nodes: (OCELEvent | OCELObject)[];
    links: { source: string; target: string; qualifier: string }[];
  }>;
};

export async function warnForNoBackendProvider<T>(): Promise<T> {
  console.warn("No BackendProviderContext!");
  return await new Promise<T>((_resolve, reject) => {
    reject(Error("No BackendProviderContext"));
  });
}

export const BackendProviderContext = createContext<BackendProvider>({
  "ocel/info": warnForNoBackendProvider,
  "ocel/check-constraints-box": warnForNoBackendProvider,
  "ocel/event-qualifiers": warnForNoBackendProvider,
  "ocel/object-qualifiers": warnForNoBackendProvider,
  "ocel/discover-constraints": warnForNoBackendProvider,
  "ocel/graph": warnForNoBackendProvider,
});

export const API_WEB_SERVER_BACKEND_PROVIDER: BackendProvider = {
  "ocel/info": async () => {
    const res = await fetch("http://localhost:3000/ocel/info", {
      method: "get",
    });
    return await res.json();
  },
  "ocel/available": async () => {
    return await (
      await fetch("http://localhost:3000/ocel/available", { method: "get" })
    ).json();
  },
  "ocel/upload": async (ocelFile) => {
    const type = ocelFile.name.endsWith(".json") ? "json" : "xml";
    return await (
      await fetch(`http://localhost:3000/ocel/upload-${type}`, {
        method: "post",
        body: ocelFile,
      })
    ).json();
  },
  "ocel/load": async (name) => {
    return await (
      await fetch("http://localhost:3000/ocel/load", {
        method: "post",
        body: JSON.stringify({ name }),
        headers: { "Content-Type": "application/json" },
      })
    ).json();
  },
  "ocel/check-constraints-box": async (tree) => {
    return await (
      await fetch("http://localhost:3000/ocel/check-constraints-box", {
        method: "post",
        body: JSON.stringify({ tree }),
        headers: { "Content-Type": "application/json" },
      })
    ).json();
  },
  "ocel/event-qualifiers": async () => {
    return await (
      await fetch("http://localhost:3000/ocel/event-qualifiers", {
        method: "get",
      })
    ).json();
  },
  "ocel/object-qualifiers": async () => {
    return await (
      await fetch("http://localhost:3000/ocel/object-qualifiers", {
        method: "get",
      })
    ).json();
  },
  "ocel/discover-constraints": async (autoDiscoveryOptions) => {
    return await (
      await fetch("http://localhost:3000/ocel/discover-constraints", {
        method: "post",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(autoDiscoveryOptions),
      })
    ).json();
  },
  "ocel/graph": async (options) => {
    const res = await fetch("http://localhost:3000/ocel/graph", {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(options),
    });
    console.log({ res });
    if (res.ok) {
      return await res.json();
    } else {
      throw new Error(res.statusText);
    }
  },
};
