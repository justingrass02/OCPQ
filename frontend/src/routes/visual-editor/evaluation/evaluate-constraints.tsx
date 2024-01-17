import toast from "react-hot-toast";
import type { Edge, Node } from "reactflow";
import type { CONSTRAINT_TYPES } from "../helper/const";
import type {
  CountConstraint,
  EventTypeLinkData,
  EventTypeNodeData,
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
  connection: Connection;
  id: string;
  eventType: string;
};

type TreeNode = {
  id: string;
  eventType: string;
  parents: TreeNodeConnection[];
  children: TreeNodeConnection[];
  variables: SelectedVariables;
  countConstraint: CountConstraint;
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
  nodes: Node<EventTypeNodeData>[],
  edges: Edge<EventTypeLinkData>[],
): Promise<ViolationsPerNodes> {
  const treeNodes: Record<string, TreeNode> = Object.fromEntries(
    nodes.map((evtNode) => [
      evtNode.id,
      {
        id: evtNode.id,
        eventType: evtNode.data.eventType,
        parents: [],
        children: [],
        variables: evtNode.data.selectedVariables,
        countConstraint: {
          min: replaceInfinity(evtNode.data.countConstraint.min),
          max: replaceInfinity(evtNode.data.countConstraint.max),
        },
      } satisfies TreeNode,
    ]),
  );

  for (const e of edges) {
    if (e.sourceHandle == null || e.targetHandle == null || e.data == null) {
      console.warn("No source/target handle or no data on edge", e);
      continue;
    }
    const dependencyConnection: Connection = {
      type: e.data.constraintType,
      timeConstraint: {
        minSeconds: replaceInfinity(e.data.timeConstraint.minSeconds),
        maxSeconds: replaceInfinity(e.data.timeConstraint.maxSeconds),
      },
    };

    treeNodes[e.target].parents.push({
      connection: dependencyConnection,
      id: e.source,
      eventType: treeNodes[e.source].eventType,
    });
    treeNodes[e.source].children.push({
      connection: dependencyConnection,
      id: e.target,
      eventType: treeNodes[e.target].eventType,
    });
  }

  const disconnectedTreeNodes: Record<string, TreeNode> = {};
  const connectedTreeNodes: Record<string, TreeNode> = {};
  const rootTreeNodes: TreeNode[] = [];

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
    queue: TreeNode[],
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
  let queue: TreeNode[] = [...rootTreeNodes];

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
  ].filter((n) => n.variables.length > 0);
  // eslint-disable-next-line no-unused-vars, @typescript-eslint/no-unused-vars
  const [_sizes, violations] = await toast.promise(
    callCheckConstraintsEndpoint(inputNodes),
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
  }));
}

async function callCheckConstraintsEndpoint(nodesOrder: TreeNode[]) {
  const res = await fetch("http://localhost:3000/ocel/check-constraints", {
    method: "post",
    body: JSON.stringify(nodesOrder),
    headers: { "Content-Type": "application/json" },
  });
  const matchingSizesAndViolations: [number[], Violation[][]] =
    await res.json();
  console.log({ matchingSizesAndViolations });
  return matchingSizesAndViolations;
}
