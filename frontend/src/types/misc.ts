import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ViolationsPerNodes,
} from "@/routes/visual-editor/helper/types";
import type { ReactFlowJsonObject } from "reactflow";

export type FlowAndViolationData = {
  flowJson: ReactFlowJsonObject<
    EventTypeNodeData | GateNodeData,
    EventTypeLinkData | GateLinkData
  >;
  violations?: ViolationsPerNodes;
};
