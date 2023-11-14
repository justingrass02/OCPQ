import type { EventTypeQualifiers } from "@/types/ocel";
import toast from "react-hot-toast";
import type { Edge } from "reactflow";
import type { EventTypeLinkData } from "../helper/EventTypeLink";
import { extractFromHandleID } from "../helper/visual-editor-utils";

export type DependencyType =
  | "simple"
  | "all"
  | "existsInTarget"
  | "existsInSource";

type NodeDependency = {
  sourceQualifier: string;
  targetQualifier: string;
  objectType: string;
  dependencyType: DependencyType;
  variableName: string;
};

type TreeNodeDependency = { dependency: NodeDependency; eventType: string };

type TreeNode = {
  eventType: string;
  parents: TreeNodeDependency[];
  children: TreeNodeDependency[];
};

export function getDependencyType(
  isSourceMultiple: boolean,
  isTargetMultiple: boolean,
): DependencyType {
  return isSourceMultiple
    ? isTargetMultiple
      ? "all"
      : "existsInSource"
    : isTargetMultiple
    ? "existsInTarget"
    : "simple";
}

export function constructTree(
  eventTypes: EventTypeQualifiers,
  edges: Edge<EventTypeLinkData>[],
) {
  const treeNodes: Record<string, TreeNode> = Object.fromEntries(
    Object.keys(eventTypes).map((evt) => [
      evt,
      {
        eventType: evt,
        parents: [],
        children: [],
      },
    ]),
  );
  for (const e of edges) {
    if (e.sourceHandle == null || e.targetHandle == null) {
      console.warn("No source or target handle", e);
      continue;
    }
    const sourceHandleInfo = extractFromHandleID(e.sourceHandle);
    const targetHandleInfo = extractFromHandleID(e.targetHandle);
    const isSourceMultiple =
      eventTypes[e.source][sourceHandleInfo.qualifier].multiple;
    const isTargetMultiple =
      eventTypes[e.target][targetHandleInfo.qualifier].multiple;
    const dependency: NodeDependency = {
      sourceQualifier: sourceHandleInfo.qualifier,
      targetQualifier: targetHandleInfo.qualifier,
      objectType: sourceHandleInfo.objectType,
      dependencyType: getDependencyType(isSourceMultiple, isTargetMultiple),
      variableName:
        (isSourceMultiple && isTargetMultiple
          ? sourceHandleInfo.objectType.substring(0, 1).toUpperCase()
          : sourceHandleInfo.objectType.substring(0, 1)) + "1",
    };
    treeNodes[e.source].children.push({ dependency, eventType: e.target });
    treeNodes[e.target].parents.push({ dependency, eventType: e.source });
  }

  const disconnectedTreeNodes: Record<string, TreeNode> = {};
  const connectedTreeNodes: Record<string, TreeNode> = {};
  const rootTreeNodes: TreeNode[] = [];

  for (const eventType of Object.keys(treeNodes)) {
    if (
      treeNodes[eventType].children.length === 0 &&
      treeNodes[eventType].parents.length === 0
    ) {
      disconnectedTreeNodes[eventType] = treeNodes[eventType];
    } else {
      if (treeNodes[eventType].parents.length === 0) {
        rootTreeNodes.push(treeNodes[eventType]);
      }
      connectedTreeNodes[eventType] = treeNodes[eventType];
    }
  }

  function getFirstNodeIndexSatisfyingDependencies(
    queue: TreeNode[],
    reachableFromRootIDs: string[],
  ): number | undefined {
    for (let i = 0; i < queue.length; i++) {
      const node = queue[i];
      if (
        node.parents.every((p) => reachableFromRootIDs.includes(p.eventType))
      ) {
        return i;
      }
    }
    return undefined;
  }

  // List of reachable IDS;
  // conincidely also a possible execution error that satisfies all dependency
  const reachableFromRootIDs: string[] = [];
  let queue: TreeNode[] = [...rootTreeNodes];

  let invalid = false;
  while (queue.length > 0) {
    const indexInqueue = getFirstNodeIndexSatisfyingDependencies(
      queue,
      reachableFromRootIDs,
    );
    if (indexInqueue !== undefined) {
      const [node] = queue.splice(indexInqueue, 1);
      reachableFromRootIDs.push(node.eventType);
      queue = queue.concat(node.children.map((c) => treeNodes[c.eventType]));
    } else {
      toast.error("Invalid requirements: Cycle detected!");
      invalid = true;
      break;
    }
  }
  const unreachableNodeIDs: string[] = [];

  for (const nodeID of Object.keys(connectedTreeNodes)) {
    if (!reachableFromRootIDs.includes(nodeID)) {
      unreachableNodeIDs.push(nodeID);
    }
  }
  if (unreachableNodeIDs.length > 0) {
    if (!invalid) {
      // If invalid state was not detected before notify the user now (otherwise skip)
      toast.error("Invalid requirements detected");
    }
    invalid = true;
    toast(
      <span>
        <b>Nodes not reachable from root:</b>
        <br />
        {unreachableNodeIDs.join(", ")}
      </span>,
    );
  } else {
    toast.success(
      <span>Constructed tree with {rootTreeNodes.length} root nodes</span>,
    );
  }
  console.log({ connectedTreeNodes, rootTreeNodes, reachableFromRootIDs });

  fetch("http://localhost:3000/ocel/check-constraints", {
    method: "post",
    body: JSON.stringify(
      reachableFromRootIDs.map((id) => connectedTreeNodes[id]),
    ),
    headers: { "Content-Type": "application/json" },
  })
    .then(async (res) => {
      const violations: string[] = await res.json();
      console.log({ violations });
      toast(violations.length + " violations found");
    })
    .catch((err) => {
      toast.error(String(err));
    });
}
