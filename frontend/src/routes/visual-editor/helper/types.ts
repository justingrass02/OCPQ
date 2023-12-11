export interface ObjectVariable {
  name: string;
  type: string;
}

export type SelectedVariables = {
  variable: ObjectVariable;
  qualifier: string;
  bound: boolean;
}[];
export type ViolationReason = "TooFewMatchingEvents" | "TooManyMatchingEvents";
export type Binding = unknown;
export type Violation = [Binding, ViolationReason];
export type ViolationsPerNode = {
  violations: Violation[];
  nodeID: string;
};
export type ViolationsPerNodes = ViolationsPerNode[];
export type CountConstraint = { min: number; max: number };
