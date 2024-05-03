import { createContext } from "react";
import type { Edge, Node, ReactFlowInstance } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ViolationsPerNodes,
} from "./types";

export const FlowContext = createContext<{
  instance: ReactFlowInstance | undefined;
  registerOtherDataGetter: (
    getter: () =>
      | {
          violations?: ViolationsPerNodes;
        }
      | undefined,
  ) => unknown;
  setInstance: (i: ReactFlowInstance | undefined) => unknown;
  otherData:
    | {
        violations?: ViolationsPerNodes;
        nodes?: Node<EventTypeNodeData | GateNodeData>[];
        edges?: Edge<EventTypeLinkData | GateLinkData>[];
      }
    | undefined;
  flushData: (
    data:
      | {
          violations?: ViolationsPerNodes;
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
