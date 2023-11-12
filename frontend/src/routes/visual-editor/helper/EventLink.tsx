import { FiX } from "react-icons/fi";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from "reactflow";

export default function EventLink({
  id,
  label,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  markerEnd,
  data,
  style = {},
}: EdgeProps<{
  label: string;
  color: string;
  onDelete: (id: string) => unknown;
}>) {
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  return (
    <>
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={style} />
      {data !== undefined && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              fontSize: 12,
              pointerEvents: "all",
            }}
            className="nodrag nopan flex flex-col items-center -mt-1"
          >
            <span
              className="bg-slate-50/80 rounded-sm px-1 border border-slate-100 mb-1"
              style={{ color: data.color }}
            >
              {label}
            </span>
            <button
              className="hover:text-red-400 text-red-700 text-gray-700/50 rounded-lg text-base"
              title="Delete edge"
              onClick={() => data.onDelete(id)}
            >
              <FiX />
            </button>
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
