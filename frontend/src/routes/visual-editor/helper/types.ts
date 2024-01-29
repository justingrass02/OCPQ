import type { EventTypeQualifier } from "@/types/ocel";
import { type CONSTRAINT_TYPES } from "./const";

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
export type Binding = [
  { past_events: {event_id: string, node_id: string}[] },
  Record<string, { Single: string } | { Multiple: unknown }>,
];
export type Violation = [Binding, ViolationReason];
export type ViolationsPerNode = {
  violations: Violation[];
  nodeID: string;
};
export type ViolationsPerNodes = ViolationsPerNode[];
export type CountConstraint = { min: number; max: number };

export type EventTypeNodeData = {
  eventType: string;
  eventTypeQualifier: EventTypeQualifier;
  objectTypeToColor: Record<string, string>;
  countConstraint: CountConstraint;
  selectedVariables: SelectedVariables;
  onDataChange: (id: string, newData: Partial<EventTypeNodeData>) => unknown;
};

export type TimeConstraint = { minSeconds: number; maxSeconds: number };
export type EventTypeLinkData = {
  color: string;
  constraintType: (typeof CONSTRAINT_TYPES)[number];
  timeConstraint: TimeConstraint;
  onDataChange: (id: string, newData: Partial<EventTypeLinkData>) => unknown;
  onDelete: (id: string) => unknown;
};
