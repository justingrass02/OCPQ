import {
  BaseEdge,
  getBezierPath,
  useReactFlow,
  type EdgeProps,
  type Node,
} from "reactflow";
import type { EventTypeNodeData, GateNodeData } from "./types";

const STROKE_WIDTH = 4;

export default function QuantifiedObjectEdge({
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  markerEnd,
  // source,
  target,
  selected,
}: EdgeProps<unknown>) {
  const flow = useReactFlow();
  // const sourceNode: Node<EventTypeNodeData | GateNodeData> | undefined =
  //   flow.getNode(source);
  const targetNode: Node<EventTypeNodeData | GateNodeData> | undefined =
    flow.getNode(target);

  // const connectedObjects: { color: string }[] = [];

  // const edges: { path: string; style: React.CSSProperties }[] =
  //   connectedObjects.map((conObj, i) => ({
  //     path: getBezierPath({
  //       sourceX:
  //         sourceX +
  //         (i % 2 === 0 ? -1 : 1) * Math.ceil(i / 2) * (STROKE_WIDTH + 0.4),
  //       sourceY: sourceY - 5,
  //       sourcePosition,
  //       targetX:
  //         targetX +
  //         (i % 2 === 0 ? -1 : 1) * Math.ceil(i / 2) * (STROKE_WIDTH + 0.4),
  //       targetY: targetY - 7,
  //       targetPosition,
  //     })[0],
  //     style: {
  //       stroke: conObj.color,
  //       strokeWidth: STROKE_WIDTH,
  //     },
  //   }));

  const pathStyle: React.CSSProperties = {
    stroke: selected === true ? "#646464" : "#646464",
    strokeWidth: STROKE_WIDTH,
    marginLeft: "1rem",
    strokeDasharray: selected === true ? "7 3" : undefined,
  };

  const [edgePath] = getBezierPath({
    sourceX,
    sourceY: sourceY - (targetNode?.type === "gate" ? 0 : 5),
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      {/* {edges.map((edge, i) => (
        <BaseEdge key={i} path={edge.path} style={edge.style} />
      ))} */}
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={pathStyle} />
    </>
  );
}
