import { createContext } from "react";
import { type ObjectVariable } from "./types";

export const ConstraintInfoContext = createContext<{
  objectVariables: ObjectVariable[];
}>({ objectVariables: [] });