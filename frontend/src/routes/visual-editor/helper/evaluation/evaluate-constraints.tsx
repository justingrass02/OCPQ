import type { BindingBoxTree } from "@/types/generated/BindingBoxTree";
import type { Edge, Node } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "../types";

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

function getChildrenNodeIDsRec(
  nodeID: string,
  edges: Edge<EventTypeLinkData>[],
): string[] {
  const children = getChildrenNodeIDs(nodeID, edges);
  return [
    nodeID,
    ...children,
    ...children.map((c) => getChildrenNodeIDsRec(c, edges)).flat(),
  ];
}

export function evaluateConstraints(
  nodes: Node<EventTypeNodeData | GateNodeData>[],
  edges: Edge<EventTypeLinkData>[],
): {
  tree: BindingBoxTree;
  nodesOrder: Node<EventTypeNodeData | GateNodeData>[];
}[] {
  const nodeIDMap = new Map(nodes.map((node) => [node.id, node]));
  if (nodes.length === 0) {
    return [{ tree: { nodes: [], sizeConstraints: [] }, nodesOrder: nodes }];
  }
  const roots: Node<EventTypeNodeData | GateNodeData>[] = [];
  for (const node of nodes) {
    const parentID = getParentNodeID(node.id, edges);
    if (parentID === undefined) {
      roots.push(node);
    }
  }

  console.log(
    "Found roots: " +
      roots.map((r) => r.id).join(", ") +
      " (#" +
      roots.length +
      ")",
  );
  const ret: {
    tree: BindingBoxTree;
    nodesOrder: Node<EventTypeNodeData | GateNodeData>[];
  }[] = [];
  for (const root of roots) {
    const nodesOrder = getChildrenNodeIDsRec(root.id, edges).map(
      (nid) => nodeIDMap.get(nid)!,
    );

    const nodesIndexMap = new Map(nodesOrder.map((node, i) => [node.id, i]));
    const edgeMap = new Map(
      edges.map((edge) => [edge.source + "---" + edge.target, edge]),
    );
    const tree: BindingBoxTree = { nodes: [], sizeConstraints: [] };
    tree.nodes = nodesOrder.map((node) => {
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
        Box: [
          { newEventVars: {}, newObjectVars: {}, filterConstraint: [] },
          [],
        ],
      };
    });
    ret.push({ tree, nodesOrder });
  }

  return ret;
}
