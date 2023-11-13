import { MdRemoveCircleOutline } from "react-icons/md";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from "reactflow";

export const EVENT_LINK_TYPE = "eventLink";

export type EventLinkData = {
  color: string;
  multiple: boolean;
  onDelete: (id: string) => unknown;
};

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
}: EdgeProps<EventLinkData>) {
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
              className="bg-slate-50/90 rounded-sm px-1 border font-mono font-semibold border-slate-100 mb-1"
              style={{ color: data.color }}
            >
              {label}
            </span>
            <button
              className="hover:text-red-500 text-red-400/30  rounded-lg text-sm"
              title="Delete edge"
              onClick={() => data.onDelete(id)}
            >
              <MdRemoveCircleOutline />
            </button>
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
