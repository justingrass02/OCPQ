import { createContext } from "react";
import type { ViolationsPerNodes } from "./types";

export const ViolationsContext = createContext<
  | {
      violationsPerNode: ViolationsPerNodes;
    }
  | undefined
>(undefined);
