import type { EventTypeQualifiers } from "@/types/ocel";
import toast from "react-hot-toast";
import type { Edge } from "reactflow";
import type {
  CONSTRAINT_TYPES,
  EventTypeLinkData,
} from "../helper/EventTypeLink";
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
  constraintType: (typeof CONSTRAINT_TYPES)[number];
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
    ? "all"
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
    if (e.sourceHandle == null || e.targetHandle == null || e.data == null) {
      console.warn("No source/target handle or no data on edge", e);
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
      // TODO: Update NodeDependency to reflect changed bindings (with in and out variables)
      variableName: e.data?.inVariable,
      constraintType: e.data.constraintType,
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
      if (!reachableFromRootIDs.includes(node.eventType)) {
        reachableFromRootIDs.push(node.eventType);
      }
      queue = queue.concat(node.children.map((c) => treeNodes[c.eventType]));
    } else {
      toast.error("Invalid requirements: Cycle detected!");
      invalid = true;
      break;
    }
    queue.sort((a, b) => b.parents.length - a.parents.length);
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
    console.log(`Constructed tree with ${rootTreeNodes.length} root nodes`);
  }
  console.log({ connectedTreeNodes, rootTreeNodes, reachableFromRootIDs });

  void toast.promise(
    callCheckConstraintsEndpoint(
      reachableFromRootIDs.map((id) => connectedTreeNodes[id]),
    ),
    {
      loading: "Evaluating...",
      success: ([sizes, violations]) => (
        <span>
          <b>Evaluation finished</b>
          <br />
          <span>
            Bindings per step:
            <br />
            <span className="font-mono">{sizes.join(", ")}</span>
            <br />
            Violations per step:
            <br />
            <span className="font-mono">{violations.join(", ")}</span>
          </span>
        </span>
      ),
      error: "Evaluation failed",
    },
  );
}

async function callCheckConstraintsEndpoint(nodesOrder: TreeNode[]) {
  const res = await fetch("http://localhost:3000/ocel/check-constraints", {
    method: "post",
    body: JSON.stringify(nodesOrder),
    headers: { "Content-Type": "application/json" },
  });
  const matchingSizesAndViolations: [number[], number[]] = await res.json();
  return matchingSizesAndViolations;
}
