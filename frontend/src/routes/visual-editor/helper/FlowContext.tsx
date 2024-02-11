import { createContext } from "react";
import type { ReactFlowInstance } from "reactflow";
import type { ObjectVariable, ViolationsPerNodes } from "./types";

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
      }
    | undefined;
}>({
  instance: undefined,
  registerOtherDataGetter: () => () => undefined,
  setInstance: () => {},
  otherData: undefined,
});
