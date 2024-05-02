import type { BindingBoxTree } from "@/types/generated/BindingBoxTree";
import type { Edge, Node } from "reactflow";
import { type CONSTRAINT_TYPES } from "../helper/const";
import type {
  CountConstraint,
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ObjectVariable,
  SelectedVariables,
  TimeConstraint,
} from "../helper/types";

type Connection = {
  type: (typeof CONSTRAINT_TYPES)[number];
  timeConstraint: TimeConstraint;
};

type TreeNodeConnection = {
  connection: Connection | null;
  id: string;
  // eventType: EventTypeNodeData["eventType"];
};

export type NewTreeNode = {
  id: string;
  parents: TreeNodeConnection[];
  children: TreeNodeConnection[];
  data: TreeNodeType;
};
type TreeNodeType =
  | { Event: EventTreeNode }
  | { OR: [string, string] }
  | { AND: [string, string] }
  | { NOT: string };
type EventTreeNode = {
  eventType: EventTypeNodeData["eventType"];
  variables: SelectedVariables;
  countConstraint: CountConstraint;
  firstOrLastEventOfType?: "first" | "last" | undefined;
  waitingTimeConstraint?:
    | { minSeconds: number; maxSeconds: number }
    | undefined;
  numQualifiedObjectsConstraint?:
    | Record<string, { max: number; min: number }>
    | undefined;
};

function replaceInfinity(x: number) {
  if (x === Infinity) {
    return Number.MAX_SAFE_INTEGER;
  } else if (x === -Infinity) {
    return Number.MIN_SAFE_INTEGER;
  }
  return x;
}

function getParentNodeID(
  nodeID: string,
  edges: Edge<EventTypeLinkData | GateLinkData>[],
) {
  for (const edge of edges) {
    if (edge.target === nodeID) {
      return edge.source;
    }
  }
  return undefined;
}

export function evaluateConstraints(
  variables: ObjectVariable[],
  nodes: Node<EventTypeNodeData | GateNodeData>[],
  edges: Edge<EventTypeLinkData | GateLinkData>[],
): BindingBoxTree {
  const tree: BindingBoxTree = { nodes: [], sizeConstraints: [] };
  if (nodes.length === 0) {
    return tree;
  }

  const nodesMap = new Map(nodes.map((node) => [node.id, node]));
  const edgesMap = new Map(edges.map((edge) => [edge.id, edge]));

  let root = nodes[0];
  let numRootsFound = 0;
  for (const node of nodes) {
    const parentID = getParentNodeID(node.id, edges);
    if (parentID === undefined) {
      root = node;
      numRootsFound++;
    }
  }
  console.log("Found root: " + root.id);
  if (numRootsFound > 1) {
    console.warn("Found multiple roots! Invalid tree.");
    return tree;
  }

  tree.nodes.push(
    {
      Box: [
        {
          newEventVars: {},
          newObjectVars: Object.fromEntries(
            variables.map((v, i) => [i, [v.type]]),
          ),
          filterConstraint: [],
        },
        [1],
      ],
    },
    {
      Box: [{ newEventVars: {}, newObjectVars: {}, filterConstraint: [] }, []],
    },
  );

  return tree;
}
