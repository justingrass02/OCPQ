import { getBezierPath, type ConnectionLineComponentProps } from "reactflow";

export default function ConnectionLine(
  props: ConnectionLineComponentProps & {
    objectTypeToColor: Record<string, string>;
  },
) {
  const color = props.objectTypeToColor[props.fromHandle!.id!.split("===")[1]];

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
