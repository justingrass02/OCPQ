import EventTypeLink from "./EventTypeLink";
import EventTypeNode from "./node/EventTypeNode";

export const EVENT_TYPE_LINK_TYPE = "eventTypeLink";

export const CONSTRAINT_TYPES = [
  "response",
  "unary-response",
  "non-response",
] as const;

export const nodeTypes = { eventType: EventTypeNode };
export const edgeTypes = {
  [EVENT_TYPE_LINK_TYPE]: EventTypeLink,
};
