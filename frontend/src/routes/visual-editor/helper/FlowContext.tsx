import { createContext } from "react";
import type { Edge, Node, ReactFlowInstance } from "reactflow";
import type {
  EvaluationResPerNodes,
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "./types";

export const FlowContext = createContext<{
  instance: ReactFlowInstance | undefined;
  registerOtherDataGetter: (
    getter: () =>
      | {
          violations?: EvaluationResPerNodes;
        }
      | undefined,
  ) => unknown;
  setInstance: (i: ReactFlowInstance | undefined) => unknown;
  otherData:
    | {
        violations?: EvaluationResPerNodes;
        nodes?: Node<EventTypeNodeData | GateNodeData>[];
        edges?: Edge<EventTypeLinkData>[];
      }
    | undefined;
  flushData: (
    data:
      | {
          violations?: EvaluationResPerNodes;
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
