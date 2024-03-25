import EventTypeLink from "./EventTypeLink";
import GateLink from "./GateLink";
import EventTypeNode from "./node/EventTypeNode";
import GateNode from "./node/GateNode";

export const EVENT_TYPE_LINK_TYPE = "eventTypeLink";
export const EVENT_TYPE_NODE_TYPE = "eventType";
export const GATE_NODE_TYPE = "gate";
export const GATE_LINK_TYPE = "gateLink";

export const CONSTRAINT_TYPES = [
  "response",
  "unary-response",
  "non-response",
] as const;

export const nodeTypes = {
  [EVENT_TYPE_NODE_TYPE]: EventTypeNode,
  [GATE_NODE_TYPE]: GateNode,
};
export const edgeTypes = {
  [EVENT_TYPE_LINK_TYPE]: EventTypeLink,
  [GATE_LINK_TYPE]: GateLink,
};
