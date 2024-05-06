import type { BindingBox } from "@/types/generated/BindingBox";
import type { EvaluationResultWithCount } from "@/types/generated/EvaluationResultWithCount";

export type EvaluationResPerNodes = {
  evalRes: Map<string, EvaluationRes>;
  objectIds: string[];
  eventIds: string[];
};
export type EvaluationRes = EvaluationResultWithCount;

export type CountConstraint = { min: number; max: number };

export type EventTypeNodeData = {
  hideViolations?: boolean;
  box: BindingBox;
};

export const ALL_GATE_TYPES = ["not", "or", "and"];
export type GateNodeData = { type: "not" | "or" | "and" };

export type TimeConstraint = { minSeconds: number; maxSeconds: number };
export type EventTypeLinkData = {
  color: string;
  minCount: number | null;
  maxCount: number | null;
};

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
  eventType: unknown; // EventTypeNodeData["eventType"];
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
