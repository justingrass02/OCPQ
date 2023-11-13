import { type EventTypeQualifiers } from "@/types/ocel";
import { type Edge, type Node } from "reactflow";
import { type EventLinkData } from "../helper/EventLink";
import { type EventTypeNodeData } from "../helper/EventTypeNode";
import toast from "react-hot-toast";

export function constructTree(
  eventTypes: EventTypeQualifiers,
  nodes: Node<EventTypeNodeData>[],
  edges: Edge<EventLinkData>[],
) {
  toast("Constructing Tree");
  console.log({ eventTypes, nodes, edges });
}
