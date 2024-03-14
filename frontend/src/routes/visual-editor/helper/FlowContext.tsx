import { createContext } from "react";
import type { Edge, Node, ReactFlowInstance } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ObjectVariable,
  ViolationsPerNodes,
} from "./types";

export const FlowContext = createContext<{
  instance: ReactFlowInstance | undefined;
  registerOtherDataGetter: (
    getter: () =>
      | {
          violations?: ViolationsPerNodes;
          objectVariables?: ObjectVariable[];
        }
      | undefined,
  ) => unknown;
  setInstance: (i: ReactFlowInstance | undefined) => unknown;
  otherData:
    | {
        violations?: ViolationsPerNodes;
        objectVariables?: ObjectVariable[];
        nodes?: Node<EventTypeNodeData|GateNodeData>[];
        edges?: Edge<EventTypeLinkData|GateLinkData>[];
      }
    | undefined;
  flushData: (
    data:
      | {
          violations?: ViolationsPerNodes;
          objectVariables?: ObjectVariable[];
        }
      | undefined,
  ) => unknown;
}>({
  instance: undefined,
  registerOtherDataGetter: () => () => undefined,
  setInstance: () => {},
  otherData: undefined,
  flushData: () => {},
});
