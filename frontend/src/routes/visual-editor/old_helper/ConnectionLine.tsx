import { getBezierPath, type ConnectionLineComponentProps } from "reactflow";
import { extractFromHandleID } from "./visual-editor-utils";

export default function ConnectionLine(
  props: ConnectionLineComponentProps & {
    objectTypeToColor: Record<string, string>;
  },
) {
  const color =
    props.fromHandle?.id != null
      ? props.objectTypeToColor[
          extractFromHandleID(props.fromHandle.id).objectType
        ]
      : undefined;

  const [path] = getBezierPath({
    sourceX: props.fromX,
    sourceY: props.fromY,
    sourcePosition: props.fromPosition,
    targetPosition: props.toPosition,
    targetX: props.toX,
    targetY: props.toY,
  });

  return (
    <g className="react-flow__connection">
      <path
        className="animated !stroke-2"
        strokeWidth={10}
        fill={"none"}
        stroke={color}
        d={path}
      />
    </g>
  );
}
