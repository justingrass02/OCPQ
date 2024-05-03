import type { BindingBoxTree } from "@/types/generated/BindingBoxTree";
import type { Edge, Node } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
} from "../types";

export function getParentNodeID(
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

function getChildrenNodeIDs(
  nodeID: string,
  edges: Edge<EventTypeLinkData | GateLinkData>[],
) {
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
  edges: Edge<EventTypeLinkData | GateLinkData>[],
): BindingBoxTree {
  const tree: BindingBoxTree = { nodes: [], sizeConstraints: [] };
  if (nodes.length === 0) {
    return tree;
  }
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
  const rootIndex = nodes.indexOf(root);
  if (numRootsFound > 1 || rootIndex < 0) {
    console.warn("Found multiple roots or no valid root! Invalid tree.");
    return tree;
  }
  const newNodes = [...nodes];
  newNodes.splice(rootIndex, 1);
  newNodes.unshift(root);
  const nodesIndexMap = new Map(newNodes.map((node, i) => [node.id, i]));

  tree.nodes = newNodes.map((node) => {
    const children = getChildrenNodeIDs(node.id, edges);
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

  return tree;
}
