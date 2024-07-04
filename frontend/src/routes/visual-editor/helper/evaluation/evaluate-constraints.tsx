import type { BindingBoxTree } from "@/types/generated/BindingBoxTree";
import type { Edge, Node } from "reactflow";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  GateNodeData,
} from "../types";
import { Filter } from "@/types/generated/Filter";
import { SizeFilter } from "@/types/generated/SizeFilter";
import { Constraint } from "@/types/generated/Constraint";
import { formatSeconds } from "@/components/TimeDurationInput";
import { Variable } from "@/types/generated/Variable";

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
    return [{ tree: { nodes: [], edgeNames: [] }, nodesOrder: nodes }];
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
    const tree: BindingBoxTree = { nodes: [], edgeNames: [] };
    tree.nodes = nodesOrder.map((node) => {
      const children = getChildrenNodeIDs(node.id, edges);
      for (const c of children) {
        const e = edgeMap.get(node.id + "---" + c)!;
        const name = e.data?.name;
        if (name != null) {
          tree.edgeNames.push([
            [nodesIndexMap.get(node.id)!, nodesIndexMap.get(c)!],
            name,
          ]);
        }
        // tree.sizeConstraints.push([
        //   [nodesIndexMap.get(node.id)!, nodesIndexMap.get(c)!],
        //   [e.data?.minCount ?? null, e.data?.maxCount ?? null],
        // ]);
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
          {
            newEventVars: {},
            newObjectVars: {},
            filters: [],
            constraints: [],
            sizeFilters: [],
          },
          [],
        ],
      };
    });
    console.log(bindingTreeToLaTeXSchema(tree));
    console.log("===")
    console.log(bindingTreeToTikzTree(tree));
    ret.push({ tree, nodesOrder });
  }

  return ret;
}

function bindingTreeToLaTeXSchema(tree: BindingBoxTree) {
  let s = "";
  console.log({ tree });
  for (let i = 0; i < tree.nodes.length; i++) {
    const node = tree.nodes[i];
    if ("Box" in node) {
      const [box, children] = node.Box;
      s += String.raw`
\begin{schema}{\boxFunc(v_{${i}}) \text{ with } \mathrm{sc}_{v_{${i}}} \text{ and } constr_{v_{${i}}}}
      ${Object.entries(box.newObjectVars)
        .map(
          ([key, val]) =>
            `\\texttt{o${
              parseInt(key) + 1
            }}: \\textsc{Object} (\\texttt{${val.join(",")}})\\\\`,
        )
        .join("\n      ")} ${Object.entries(box.newEventVars)
        .map(
          ([key, val]) =>
            `\\texttt{e${
              parseInt(key) + 1
            }}: \\textsc{Event} (\\texttt{${val.join(",")}})\\\\`,
        )
        .join("\n      ")}
      \where ${[...box.filters, ...box.sizeFilters]
        .map((f) => predicateToLaTeX(f))
        .join("\\\\\n")}
      \where ${box.constraints.map((f) => predicateToLaTeX(f)).join("\\\\\n")}
\end{schema}
\vspace{-1.33cm}`;
    } else {
      console.warn("Non-Box node type not handled!");
    }
  }
  return s;
}

function predicateToLaTeX<T extends Filter | SizeFilter | Constraint>(
  value: T,
) {
  switch (value.type) {
    case "O2E":
      return String.raw`\mathrm{E2O}(\texttt{e${value.event + 1}},\texttt{o${
        value.object + 1
      }},\texttt{${value.qualifier ?? "$\\ast$"}})`;
    case "O2O":
      return String.raw`\mathrm{O2O}(\texttt{e${value.object + 1}},\texttt{o${
        value.other_object + 1
      }},\texttt{${value.qualifier ?? "$\\ast$"}})`;
    case "TimeBetweenEvents":
      return String.raw`\texttt{e${
        value.from_event + 1
      }} \xrightarrow{\text{${formatSeconds(
        value.min_seconds ?? -Infinity,
        "$\\infty$",
        "$-\\infty$",
      )} -- ${formatSeconds(
        value.max_seconds ?? Infinity,
        "$\\infty$",
        "$-\\infty$",
      )}}} \texttt{e${value.to_event + 1}}`;
    case "NumChilds":
      return String.raw`${value.min ?? 0} \leq \left|\texttt{${
        value.child_name
      }}\right| \leq ${value.max ?? "\\infty"}`;
      case "AND":
        return String.raw`\mathrm{AND}(${value.child_names.map(s => "\\texttt{"+s+"}").join(", ")})`
        case "SAT":
        return String.raw`\mathrm{SAT}(${value.child_names.map(s => "\\texttt{"+s+"}").join(", ")})`
        case "NOT":
        return String.raw`\mathrm{NOT}(${value.child_names.map(s => "\\texttt{"+s+"}").join(", ")})`
        case "OR":
        return String.raw`\mathrm{OR}(${value.child_names.map(s => "\\texttt{"+s+"}").join(", ")})`
      case "BindingSetEqual":
        return String.raw`${value.child_names.map(s => "\\texttt{"+s+"}").join(" = ")}`
      case "BindingSetProjectionEqual": {
        function varName(v: Variable){
          if("Event" in v){
            return "e" + (1 + v.Event)
          }else{
            return "o" + (1 + v.Object)
          }
        }
        return String.raw`${value.child_name_with_var_name.map(([s,v]) => "\\texttt{"+s+"}["+ varName(v) + "]").join(" = ")}`
      }
    case "Filter":
      return predicateToLaTeX(value.filter);
    case "SizeFilter":
      return predicateToLaTeX(value.filter);
    default:
      return "TODO";
  }
}

function bindingTreeToTikzTree(tree: BindingBoxTree) {
  let s = String.raw`
\begin{tikzpicture}
    \graph[tree layout, sibling distance=15mm, nodes={align=center},level 2/.style={level distance=15mm}]
    {
      root [as={}];
`;
  const edgeNames = [...tree.edgeNames];
  for (let i = 0; i < tree.nodes.length; i++) {
    s += String.raw`  v${i} [as={$v_{${i}}$}];`
    s += "\n"
    const node = tree.nodes[i];
    if("Box" in node){
      for(const child of node.Box[1]){
        if(edgeNames.find(([[from,to],_name]) => from === i && child === to) === undefined){
          edgeNames.push([[i,child],""]);
        }
      }
    }
  }
  let i = 0;
  s+= `
  root -> v0,
`;
  for(const [[from,to],name] of edgeNames){
    s += String.raw`  v${from} -> ["${name}"${i%2 === 0 ? ",swap" : ""}] v${to},`
    s += "\n"
    i++;
  }
  return s + String.raw`
  };
\end{tikzpicture}`;
}
