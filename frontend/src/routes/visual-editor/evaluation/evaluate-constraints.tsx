import toast from "react-hot-toast";
import type { Edge, Node } from "reactflow";
import {
  EVENT_TYPE_NODE_TYPE,
  type CONSTRAINT_TYPES,
  GATE_NODE_TYPE,
} from "../helper/const";
import type {
  CountConstraint,
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ObjectVariable,
  SelectedVariables,
  TimeConstraint,
  Violation,
  ViolationsPerNodes,
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

type NewTreeNode = {
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

export async function evaluateConstraints(
  variables: ObjectVariable[],
  nodes: Node<EventTypeNodeData | GateNodeData>[],
  edges: Edge<EventTypeLinkData | GateLinkData>[],
): Promise<ViolationsPerNodes> {
  console.log({ variables, nodes });
  function getGateNodeType(node: Node<GateNodeData>): TreeNodeType {
    const outTargetNodeIDs = edges
      .filter(
        (e) =>
          e.source === node.id &&
          nodes.findIndex((n) => n.id === e.target) >= 0,
      )
      .map((e) => e.target);
    if (node.data.type === "not") {
      if (outTargetNodeIDs.length !== 1) {
        console.error("Expected a single child of NOT");
      }

      return { NOT: outTargetNodeIDs[0] };
    }
    if (outTargetNodeIDs.length !== 2) {
      console.error("Expected two children of OR/AND");
    }
    if (node.data.type === "and") {
      return { AND: outTargetNodeIDs as [string, string] };
    }

    return { OR: outTargetNodeIDs as [string, string] };
  }
  const treeNodes: Record<string, NewTreeNode> = Object.fromEntries(
    nodes.map((node) => [
      node.id,
      {
        id: node.id,
        parents: [],
        children: [],
        data:
          node.type === EVENT_TYPE_NODE_TYPE && "eventType" in node.data
            ? {
                Event: {
                  variables: node.data.selectedVariables,
                  eventType: node.data.eventType,
                  countConstraint: {
                    min: replaceInfinity(node.data.countConstraint.min),
                    max: replaceInfinity(node.data.countConstraint.max),
                  },
                  firstOrLastEventOfType: node.data.firstOrLastEventOfType,
                  waitingTimeConstraint:
                    node.data.waitingTimeConstraint !== undefined
                      ? {
                          minSeconds: replaceInfinity(
                            node.data.waitingTimeConstraint.minSeconds,
                          ),
                          maxSeconds: replaceInfinity(
                            node.data.waitingTimeConstraint.maxSeconds,
                          ),
                        }
                      : undefined,
                  numQualifiedObjectsConstraint:
                    node.data.numQualifiedObjectsConstraint !== undefined
                      ? Object.fromEntries(
                          Object.entries(
                            node.data.numQualifiedObjectsConstraint,
                          ).map(([key, val]) => [
                            key,
                            {
                              min: replaceInfinity(val.min),
                              max: replaceInfinity(val.max),
                            },
                          ]),
                        )
                      : undefined,
                } satisfies EventTreeNode,
              }
            : getGateNodeType(node as Node<GateNodeData>),
      } satisfies NewTreeNode,
    ]),
  );

  for (const e of edges) {
    if (e.sourceHandle == null || e.targetHandle == null) {
      console.warn("No source/target handle", e);
      continue;
    }
    if (e.data == null) {
      if (
        treeNodes[e.target] !== undefined &&
        treeNodes[e.source] !== undefined
      ) {
        // Gate!
        treeNodes[e.target].parents.push({
          connection: null,
          id: e.source,
        });
        treeNodes[e.source].children.push({
          connection: null,
          id: e.target,
        });
      }
      continue;
    }
    if (!("constraintType" in e.data)) {
      console.warn("GateLink edge not handled yet. TODO!", e);
      continue;
    }
    const dependencyConnection: Connection | null =
      "Event" in treeNodes[e.source].data
        ? {
            type: e.data.constraintType,
            timeConstraint: {
              minSeconds: replaceInfinity(e.data.timeConstraint.minSeconds),
              maxSeconds: replaceInfinity(e.data.timeConstraint.maxSeconds),
            },
          }
        : null;
    if (
      treeNodes[e.target] !== undefined &&
      treeNodes[e.source] !== undefined
    ) {
      treeNodes[e.target].parents.push({
        connection: dependencyConnection,
        id: e.source,
      });
      treeNodes[e.source].children.push({
        connection: dependencyConnection,
        id: e.target,
      });
    }
  }

  const disconnectedTreeNodes: Record<string, NewTreeNode> = {};
  const connectedTreeNodes: Record<string, NewTreeNode> = {};
  const rootTreeNodes: NewTreeNode[] = [];

  for (const eventType of Object.keys(treeNodes)) {
    if (
      treeNodes[eventType].parents.length === 0 &&
      treeNodes[eventType].children.length === 0
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
    queue: NewTreeNode[],
    reachableFromRootIDs: string[],
  ): number | undefined {
    for (let i = 0; i < queue.length; i++) {
      const node = queue[i];
      console.log(
        { node },
        node.parents.every((p) => reachableFromRootIDs.includes(p.id)),
      );
      if (node.parents.every((p) => reachableFromRootIDs.includes(p.id))) {
        return i;
      }
    }
    return undefined;
  }

  // List of reachable IDS;
  // conincidely also a possible execution error that satisfies all dependency
  const reachableFromRootIDs: string[] = [];
  let queue: NewTreeNode[] = [...rootTreeNodes];

  let invalid = false;
  while (queue.length > 0) {
    const indexInqueue = getFirstNodeIndexSatisfyingDependencies(
      queue,
      reachableFromRootIDs,
    );
    if (indexInqueue !== undefined) {
      const [node] = queue.splice(indexInqueue, 1);
      if (!reachableFromRootIDs.includes(node.id)) {
        reachableFromRootIDs.push(node.id);
      }
      queue = queue.concat(node.children.map((c) => treeNodes[c.id]));
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
  console.log({
    connectedTreeNodes,
    rootTreeNodes,
    reachableFromRootIDs,
    disconnectedTreeNodes,
  });
  const inputNodes = [
    ...Object.values(disconnectedTreeNodes),
    ...reachableFromRootIDs.map((id) => connectedTreeNodes[id]),
  ];
  // eslint-disable-next-line no-unused-vars, @typescript-eslint/no-unused-vars
  const [sizes, violations] = await toast.promise(
    callCheckConstraintsEndpoint(variables, inputNodes),
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
            <span className="font-mono">
              {violations.map((vs) => vs.length).join(", ")}
            </span>
          </span>
        </span>
      ),
      error: "Evaluation failed",
    },
  );

  return violations.map((vs, i) => ({
    nodeID: inputNodes[i].id,
    violations: vs,
    numBindings: sizes[i],
  }));
}

async function callCheckConstraintsEndpoint(
  variables: ObjectVariable[],
  nodesOrder: NewTreeNode[],
) {
  const res = await fetch("http://localhost:3000/ocel/check-constraints", {
    method: "post",
    body: JSON.stringify({ variables, nodesOrder }),
    headers: { "Content-Type": "application/json" },
  });
  const matchingSizesAndViolations: [number[], Violation[][]] =
    await res.json();
  console.log({ matchingSizesAndViolations });
  return matchingSizesAndViolations;
}
