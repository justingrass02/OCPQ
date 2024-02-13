import type { EventTypeQualifier } from "@/types/ocel";
import { type CONSTRAINT_TYPES } from "./const";

export interface ObjectVariable {
  name: string;
  type: string;
  initiallyBound: boolean;
  o2o?: undefined | { parentVariableName: string; qualifier: string };
}

export type SelectedVariables = {
  variable: ObjectVariable;
  qualifier: string|undefined;
  bound: boolean;
}[];
export type ViolationReason = "TooFewMatchingEvents" | "TooManyMatchingEvents";
export type Binding = [
  { past_events: { event_id: string; node_id: string }[] },
  Record<string, { Single: string } | { Multiple: unknown }>,
];
export type Violation = [Binding, ViolationReason];
export type ViolationsPerNode = {
  violations: Violation[];
  numBindings: number;
  nodeID: string;
};
export type ViolationsPerNodes = ViolationsPerNode[];
export type CountConstraint = { min: number; max: number };

export type EventTypeNodeData = {
  eventType:
    | { type: "any" }
    | { type: "exactly"; value: string }
    | { type: "anyOf"; values: string[] }
    | { type: "anyExcept"; values: string[] };
  eventTypeQualifier: EventTypeQualifier;
  objectTypeToColor: Record<string, string>;
  countConstraint: CountConstraint;
  firstOrLastEventOfType?: "first" | "last" | undefined;
  selectedVariables: SelectedVariables;
  waitingTimeConstraint?: { minSeconds: number; maxSeconds: number };
  // Record of string (qualifier) and min/max number of associated objects wrt. that qualifier
  numQualifiedObjectsConstraint?: Record<string, { min: number; max: number }>;
  hideViolations?: boolean;
};

export type TimeConstraint = { minSeconds: number; maxSeconds: number };
export type EventTypeLinkData = {
  color: string;
  constraintType: (typeof CONSTRAINT_TYPES)[number];
  timeConstraint: TimeConstraint;
  onDataChange: (id: string, newData: Partial<EventTypeLinkData>) => unknown;
  onDelete: (id: string) => unknown;
};
