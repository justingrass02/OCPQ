import type {
  EvaluationResPerNodes,
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "@/routes/visual-editor/helper/types";
import type { ReactFlowJsonObject } from "reactflow";

export type FlowAndViolationData = {
  flowJson: ReactFlowJsonObject<
    EventTypeNodeData | GateNodeData,
    EventTypeLinkData
  >;
  violations?: EvaluationResPerNodes;
};
