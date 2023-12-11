import { createContext } from "react";
import type { ViolationsPerNode, ViolationsPerNodes } from "./types";

export type ViolationsContextValue = {
  violationsPerNode?: ViolationsPerNodes;
  showViolationsFor?: (data: ViolationsPerNode) => unknown;
};

export const ViolationsContext = createContext<ViolationsContextValue>({});
