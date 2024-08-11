import { createContext } from "react";
import type {
  EvaluationResPerNodes,
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "./types";
import type { OCELInfo, OCELType } from "@/types/ocel";
import type { EventVariable } from "@/types/generated/EventVariable";
import type { ObjectVariable } from "@/types/generated/ObjectVariable";

export type VisualEditorContextValue = {
  violationsPerNode?: EvaluationResPerNodes;
  showViolationsFor?: (nodeID: string, mode?: "violations" | "situations" | "satisfied-situations") => unknown;
  onNodeDataChange: (
    id: string,
    newData: Partial<EventTypeNodeData | GateNodeData> | undefined,
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
  getNodeIDByName: (name: string) => string | undefined;
  getAvailableChildNames: (nodeID: string) => string[];
  getVarName: (
    variable: EventVariable | ObjectVariable,
    type: "object" | "event",
  ) => { name: string; color: string };
  getTypesForVariable: (
    nodeID: string,
    variable: number,
    type: "object" | "event",
  ) => OCELType[];
};

export const VisualEditorContext = createContext<VisualEditorContextValue>({
  onNodeDataChange: () => {},
  onEdgeDataChange: () => {},
  getAvailableVars: () => [],
  getNodeIDByName: () => undefined,
  getTypesForVariable: () => [],
  getAvailableChildNames: () => [],
  getVarName: (variable, type) => ({
    name: type.substring(0, 1) + variable,
    // name: type.substring(0, 2) + "_" + variable,
    color: "black",
  }),
});
