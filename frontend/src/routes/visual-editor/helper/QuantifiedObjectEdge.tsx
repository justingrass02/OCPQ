import {
  BaseEdge,
  getBezierPath,
  useReactFlow,
  type EdgeProps,
  type Node,
} from "reactflow";
import type { EventTypeNodeData, GateNodeData } from "./types";
import { useContext } from "react";
import { ConstraintInfoContext } from "./ConstraintInfoContext";
import { COLORS } from "./colors";

const STROKE_WIDTH = 4;

export default function QuantifiedObjectEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  markerEnd,
  data,
  source,
  target,
  style = {},
}: EdgeProps<unknown>) {
  const flow = useReactFlow();
  const sourceNode: Node<EventTypeNodeData | GateNodeData> | undefined =
    flow.getNode(source);
  const targetNode: Node<EventTypeNodeData | GateNodeData> | undefined =
    flow.getNode(target);

  const { objectVariables } = useContext(ConstraintInfoContext);
  const connectedObjects: { color: string }[] = [];
  if (
    sourceNode?.data !== undefined &&
    targetNode?.data !== undefined &&
    "selectedVariables" in sourceNode?.data &&
    "selectedVariables" in targetNode?.data
  ) {
    const sourceVariables = sourceNode.data.selectedVariables;
    const targetVariables = targetNode.data.selectedVariables;
    for (const v of sourceVariables) {
      if (
        targetVariables.find((v2) => v2.variable.name === v.variable.name) !==
        undefined
      ) {
        const index = objectVariables.findIndex(
          (v3) => v3.name === v.variable.name,
        );
        connectedObjects.push({ color: COLORS[index] });
      }
    }
  }
  const edges: { path: string; style: React.CSSProperties }[] =
    connectedObjects.map((conObj, i) => ({
      path: getBezierPath({
        sourceX:
          sourceX +
          (i % 2 === 0 ? -1 : 1) * Math.ceil(i / 2) * (STROKE_WIDTH + 0.4),
        sourceY: sourceY - 5,
        sourcePosition,
        targetX:
          targetX +
          (i % 2 === 0 ? -1 : 1) * Math.ceil(i / 2) * (STROKE_WIDTH + 0.4),
        targetY: targetY - 7,
        targetPosition,
      })[0],
      style: {
        stroke: conObj.color,
        strokeWidth: STROKE_WIDTH,
      },
    }));

  const pathStyle: React.CSSProperties = {
    stroke: edges.length > 0 ? "#96969600" : "#969696",
    strokeWidth: STROKE_WIDTH,
    marginLeft: "1rem",
  };

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY: sourceY - (targetNode?.type === "gate" ? 0 : 5),
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      {edges.map((edge, i) => (
        <BaseEdge key={i} path={edge.path} style={edge.style} />
      ))}
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={pathStyle} />
    </>
  );
}
