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
  ObjectTypeQualifiers,
} from "./types/ocel";
import { EvaluationResultWithCount } from "./types/generated/EvaluationResultWithCount";
import { TableExportOptions } from "./types/generated/TableExportOptions";
import { z } from "zod";
import { ConnectionConfig, JobStatus } from "./types/hpc-backend";
import { OCPQJobOptions } from "./types/generated/OCPQJobOptions";
export type BackendProvider = {
  "ocel/info": () => Promise<OCELInfo|undefined>;
  "ocel/upload"?: (file: File) => Promise<OCELInfo>;
  "ocel/available"?: () => Promise<string[]>;
  "ocel/load"?: (name: string) => Promise<OCELInfo>;
  "ocel/picker"?: () => Promise<OCELInfo>;
  "ocel/check-constraints-box": (
    tree: BindingBoxTree,
    measurePerformance?: boolean,
  ) => Promise<EvaluateBoxTreeResult>;
  "ocel/export-filter-box": (
    tree: BindingBoxTree,
    format: "XML" | "JSON" | "SQLITE",
  ) => Promise<Blob | void>;
  "ocel/event-qualifiers": () => Promise<EventTypeQualifiers>;
  "ocel/object-qualifiers": () => Promise<ObjectTypeQualifiers>;
  "ocel/discover-constraints": (
    autoDiscoveryOptions: DiscoverConstraintsRequest,
  ) => Promise<DiscoverConstraintsResponse>;
  "ocel/export-bindings": (
    nodeIndex: number,
    options: TableExportOptions,
  ) => Promise<Blob | undefined>;
  "ocel/graph": (options: OCELGraphOptions) => Promise<{
    nodes: (OCELEvent | OCELObject)[];
    links: { source: string; target: string; qualifier: string }[];
  }>;
  "ocel/get-object": (
    specifier: { id: string } | { index: number },
  ) => Promise<{ index: number; object: OCELObject }>;
  "ocel/get-event": (
    specifier: { id: string } | { index: number },
  ) => Promise<{ index: number; event: OCELEvent }>;
  "hpc/login": (connectionConfig: ConnectionConfig) => Promise<void>,
  "hpc/start": (jobOptions: OCPQJobOptions) => Promise<string>,
  "hpc/job-status": (jobID: string) => Promise<JobStatus>,
  "download-blob": (blob: Blob, fileName: string) => unknown,
  "translate-to-sql": (
    tree: BindingBoxTree)
    => Promise<string>,
};

export async function warnForNoBackendProvider<T>(): Promise<T> {
  console.warn("No BackendProviderContext!");
  return await new Promise<T>((_resolve, reject) => {
    reject(Error("No BackendProviderContext"));
  });
}

export const ErrorBackendContext: BackendProvider = {
  "ocel/info": warnForNoBackendProvider,
  "ocel/check-constraints-box": warnForNoBackendProvider,
  "ocel/export-filter-box": warnForNoBackendProvider,
  "ocel/event-qualifiers": warnForNoBackendProvider,
  "ocel/object-qualifiers": warnForNoBackendProvider,
  "ocel/discover-constraints": warnForNoBackendProvider,
  "ocel/export-bindings": warnForNoBackendProvider,
  "ocel/graph": warnForNoBackendProvider,
  "ocel/get-event": warnForNoBackendProvider,
  "ocel/get-object": warnForNoBackendProvider,
  "hpc/login": warnForNoBackendProvider,
  "hpc/start": warnForNoBackendProvider,
  "hpc/job-status": warnForNoBackendProvider,
  "download-blob": warnForNoBackendProvider,
  "translate-to-sql": warnForNoBackendProvider,
};

export const BackendProviderContext = createContext<BackendProvider>(ErrorBackendContext);

export const DEFAULT_BACKEND_URL = "http://localhost:3000";

export function getAPIServerBackendProvider(localBackendURL: string):  BackendProvider {
  return {
  "ocel/info": async () => {
    const res = await fetch(localBackendURL + "/ocel/info", {
      method: "get",
      headers: {},
    });
    return await res.json();
  },
  "ocel/available": async () => {
    return await (
      await fetch(localBackendURL + "/ocel/available", {
        method: "get",
        headers: {},
      })
    ).json();
  },
  "ocel/upload": async (ocelFile) => {
    const type = ocelFile.name.endsWith(".json")
      ? "json"
      : ocelFile.name.endsWith(".xml")
      ? "xml"
      : "sqlite";
    return await (
      await fetch(localBackendURL + `/ocel/upload-${type}`, {
        method: "post",
        body: ocelFile,
        headers: {},
      })
    ).json();
  },
  "ocel/load": async (name) => {
    return await (
      await fetch(localBackendURL + "/ocel/load", {
        method: "post",
        body: JSON.stringify({ name }),
        headers: { "Content-Type": "application/json" },
      })
    ).json();
  },
  "ocel/check-constraints-box": async (tree, measurePerformance) => {
    return await (
      await fetch(localBackendURL + "/ocel/check-constraints-box", {
        method: "post",
        body: JSON.stringify({ tree, measurePerformance }),
        headers: { "Content-Type": "application/json" },
      })
    ).json();
  },
  "ocel/export-filter-box": async (tree, exportFormat) => {
    return await (
      await fetch(localBackendURL + "/ocel/export-filter-box", {
        method: "post",
        body: JSON.stringify({ tree, exportFormat }),
        headers: { "Content-Type": "application/json" },
      })
    ).blob();
  },
  "ocel/event-qualifiers": async () => {
    return await (
      await fetch(localBackendURL + "/ocel/event-qualifiers", {
        method: "get",
        headers: {},
      })
    ).json();
  },
  "ocel/object-qualifiers": async () => {
    return await (
      await fetch(localBackendURL + "/ocel/object-qualifiers", {
        method: "get",
        headers: {},
      })
    ).json();
  },
  "ocel/export-bindings": async (nodeId, options) => {
    const res = await fetch(localBackendURL + "/ocel/export-bindings", {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify([nodeId, options]),
    });
    if (res.ok) {
      return await res.blob();
    } else {
      throw new Error(res.status + " " + res.statusText);
    }
  },
  "ocel/discover-constraints": async (autoDiscoveryOptions) => {
    return await (
      await fetch(localBackendURL + "/ocel/discover-constraints", {
        method: "post",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(autoDiscoveryOptions),
      })
    ).json();
  },
  "ocel/graph": async (options) => {
    const res = await fetch(localBackendURL + "/ocel/graph", {
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
  "ocel/get-event": async (specifier) => {
    const res = await fetch(localBackendURL + "/ocel/get-event", {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(specifier),
    });
    console.log({ res });
    if (res.ok) {
      return await res.json();
    } else {
      throw new Error(res.statusText);
    }
  },
  "ocel/get-object": async (specifier) => {
    const res = await fetch(localBackendURL + "/ocel/get-object", {
      method: "post",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(specifier),
    });
    console.log({ res });
    if (res.ok) {
      return await res.json();
    } else {
      throw new Error(res.statusText);
    }
  },
  "hpc/login": async (connectionConfig) => {
    const res = await fetch(localBackendURL + "/hpc/login", {
        method: "post",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(connectionConfig),
      });
      if(res.ok){
        return await res.json()
      }else{
        throw Error(await res.text())
      }
  },
  "hpc/start": async (jobConfig) => {
    const res = await fetch(localBackendURL + "/hpc/start", {
        method: "post",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(jobConfig),
      });
      if(res.ok){
        return await res.json()
      }else{
        throw Error(await res.text())
      }
  },
  "hpc/job-status": async (jobID) => {
    const res = await fetch(localBackendURL + `/hpc/job-status/${jobID}`, {
        method: "get",
      });
      if(res.ok){
        return await res.json()
      }else{
        throw Error(await res.text())
      }
  },
"download-blob": async (blob,fileName) => {
    const dataURL = URL.createObjectURL(blob)
    const a = document.createElement("a");
    a.setAttribute("download", fileName);
    // a.setAttribute("target", "_blank");
    a.setAttribute("href", dataURL);
    document.body.appendChild(a);
    a.click();
    // console.log(a);
    document.body.removeChild(a);
    setTimeout(() => {
      URL.revokeObjectURL(dataURL);
    },2000);

},
"translate-to-sql": async (tree) => {
  const res = await fetch(localBackendURL + "/translate-to-sql",{
    method: "post",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(tree),
  });
  if(res.ok){
    return await res.json()
  }else{
    throw Error(await res.text())
  }
  }
}
};
