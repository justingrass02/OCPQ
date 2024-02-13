import { createContext } from "react";
import type {
  EventTypeNodeData,
  ViolationsPerNode,
  ViolationsPerNodes,
} from "./types";

export type VisualEditorContextValue = {
  violationsPerNode?: ViolationsPerNodes;
  showViolationsFor?: (data: ViolationsPerNode) => unknown;
  onNodeDataChange: (
    id: string,
    newData: Partial<EventTypeNodeData> | undefined,
  ) => unknown;
};

export const VisualEditorContext = createContext<VisualEditorContextValue>({
  onNodeDataChange: () => {},
});
