import { createContext } from "react";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  ViolationsPerNode,
  ViolationsPerNodes,
} from "./types";
import type { OCELInfo } from "@/types/ocel";
import type { EventVariable } from "@/types/generated/EventVariable";
import type { ObjectVariable } from "@/types/generated/ObjectVariable";

export type VisualEditorContextValue = {
  violationsPerNode?: ViolationsPerNodes;
  showViolationsFor?: (data: ViolationsPerNode) => unknown;
  onNodeDataChange: (
    id: string,
    newData: Partial<EventTypeNodeData> | undefined,
  ) => unknown;
  onEdgeDataChange: (
    id: string,
    newData: Partial<EventTypeLinkData> | undefined,
  ) => unknown;
  ocelInfo?: OCELInfo;
  getAvailableVars: (
    nodeID: string,
    type: "object" | "event",
  ) => (ObjectVariable | EventVariable)[];
};

export const VisualEditorContext = createContext<VisualEditorContextValue>({
  onNodeDataChange: () => {},
  onEdgeDataChange: () => {},
  getAvailableVars: () => [],
});
