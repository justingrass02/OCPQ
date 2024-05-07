import type { BindingBoxTree } from "@/types/generated/BindingBoxTree";
import type { Edge, Node } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "../types";
import toast from "react-hot-toast";

export function getParentNodeID(
  nodeID: string,
  edges: Edge<EventTypeLinkData>[],
) {
  for (const edge of edges) {
    if (edge.target === nodeID) {
      return edge.source;
    }
  }
  return undefined;
}

export function getParentsNodeIDs(
  nodeID: string,
  edges: Edge<EventTypeLinkData>[],
): string[] {
  for (const edge of edges) {
    if (edge.target === nodeID) {
      return [...getParentsNodeIDs(edge.source, edges), edge.source];
    }
  }
  return [];
}

function getChildrenNodeIDs(nodeID: string, edges: Edge<EventTypeLinkData>[]) {
  const children = [];
  for (const edge of edges) {
    if (edge.source === nodeID) {
      children.push(edge.target);
    }
  }
  return children;
}

export function evaluateConstraints(
  nodes: Node<EventTypeNodeData | GateNodeData>[],
  edges: Edge<EventTypeLinkData>[],
): {
  tree: BindingBoxTree;
  nodesOrder: Node<EventTypeNodeData | GateNodeData>[];
} {
  const tree: BindingBoxTree = { nodes: [], sizeConstraints: [] };
  const newNodes = [...nodes];
  if (nodes.length === 0) {
    return { tree, nodesOrder: newNodes };
  }
  let root: Node<EventTypeNodeData | GateNodeData> | undefined;
  let numRootsFound = 0;
  for (const node of nodes) {
    const parentID = getParentNodeID(node.id, edges);
    if (parentID === undefined) {
      root = node;
      numRootsFound++;
    }
  }

  console.log("Found root: " + root?.id + " of " + numRootsFound);
  const rootIndex = nodes.indexOf(root!);
  if (numRootsFound > 1 || rootIndex < 0) {
    console.warn("Found multiple roots or no valid root! Invalid tree.");
    toast.error("Invalid Tree! Found multiple root nodes.");
    return { tree, nodesOrder: newNodes };
  }
  newNodes.splice(rootIndex, 1);
  newNodes.unshift(root!);
  const nodesIndexMap = new Map(newNodes.map((node, i) => [node.id, i]));
  const edgeMap = new Map(
    edges.map((edge) => [edge.source + "---" + edge.target, edge]),
  );

  tree.nodes = newNodes.map((node) => {
    const children = getChildrenNodeIDs(node.id, edges);
    for (const c of children) {
      const e = edgeMap.get(node.id + "---" + c)!;
      tree.sizeConstraints.push([
        [nodesIndexMap.get(node.id)!, nodesIndexMap.get(c)!],
        [e.data?.minCount ?? null, e.data?.maxCount ?? null],
      ]);
    }
    if ("box" in node.data) {
      return {
        Box: [node.data.box, children.map((c) => nodesIndexMap.get(c)!)],
      };
    } else {
      if (node.data.type === "and" && children.length === 2) {
        const [c1, c2] = children;
        return { AND: [nodesIndexMap.get(c1)!, nodesIndexMap.get(c2)!] };
      } else if (node.data.type === "or" && children.length === 2) {
        const [c1, c2] = children;
        return { OR: [nodesIndexMap.get(c1)!, nodesIndexMap.get(c2)!] };
      } else if (node.data.type === "not" && children.length === 1) {
        return { NOT: nodesIndexMap.get(children[0])! };
      } else {
        console.warn("Invalid GATE ", node);
      }
    }
    console.warn("Returning default box");
    return {
      Box: [{ newEventVars: {}, newObjectVars: {}, filterConstraint: [] }, []],
    };
  });

  return { tree, nodesOrder: newNodes };
}
