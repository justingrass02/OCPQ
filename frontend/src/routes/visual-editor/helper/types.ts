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
  qualifier: string | undefined;
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
  countConstraint: CountConstraint;
  firstOrLastEventOfType?: "first" | "last" | undefined;
  selectedVariables: SelectedVariables;
  waitingTimeConstraint?: { minSeconds: number; maxSeconds: number };
  // Record of string (qualifier) and min/max number of associated objects wrt. that qualifier
  numQualifiedObjectsConstraint?: Record<string, { min: number; max: number }>;
  hideViolations?: boolean;
};

export const ALL_GATE_TYPES = ["not", "or", "and"];
export type GateNodeData = { type: "not" | "or" | "and" };

export type TimeConstraint = { minSeconds: number; maxSeconds: number };
export type EventTypeLinkData = {
  color: string;
  constraintType: (typeof CONSTRAINT_TYPES)[number];
  timeConstraint: TimeConstraint;
};
// eslint-disable-next-line @typescript-eslint/ban-types
export type GateLinkData = {};

export type DiscoverConstraintsRequest = {
  countConstraints?: {
    objectTypes: string[];
    coverFraction: number;
  };
  eventuallyFollowsConstraints?: {
    objectTypes: string[];
    coverFraction: number;
  };
  orConstraints?: {
    objectTypes: string[];
  };
};

export type DiscoverConstraintsRequestWrapper = DiscoverConstraintsRequest & {
  countConstraints: { enabled: boolean };
  eventuallyFollowsConstraints: { enabled: boolean };
  orConstraints: { enabled: boolean };
};

export type DiscoveredCountConstraint = {
  countConstraint: { min: number; max: number };
  objectType: string;
  eventType: EventTypeNodeData["eventType"];
};
export type DiscoveredEFConstraint = {
  secondsRange: {
    minSeconds: number;
    maxSeconds: number;
  };
  objectTypes: string[];
  fromEventType: string;
  toEventType: string;
};
export type DiscoveredORConstraint =
  | {
      EfOrCount: [DiscoveredEFConstraint, DiscoveredCountConstraint];
    }
  | { CountOrEf: [DiscoveredCountConstraint, DiscoveredEFConstraint] };

export type DiscoverConstraintsResponse = {
  countConstraints: DiscoveredCountConstraint[];
  eventuallyFollowsConstraints: DiscoveredEFConstraint[];
  orConstraints: DiscoveredORConstraint[];
};
