import {
  MdDoNotDisturb,
  MdKeyboardArrowRight,
  MdKeyboardDoubleArrowRight,
  MdRemoveCircleOutline,
} from "react-icons/md";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from "reactflow";

export const EVENT_TYPE_LINK_TYPE = "eventTypeLink";

export const CONSTRAINT_TYPES = [
  "response",
  "unary-response",
  "non-response",
] as const;

export type EventTypeLinkData = {
  color: string;
  constraintType: (typeof CONSTRAINT_TYPES)[number];
  onDataChange: (id: string, newData: Partial<EventTypeLinkData>) => unknown;
  onDelete: (id: string) => unknown;
};

export default function EventTypeLink({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  markerEnd,
  data,
  style = {},
}: EdgeProps<EventTypeLinkData>) {
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
              transform: `translate(0.66rem,-0.5rem) translate(-50%, -50%) translate(${sourceX}px,${sourceY}px)`,
              fontSize: 12,
              pointerEvents: "all",
            }}
            className="nodrag nopan flex flex-col items-center -mt-1"
          ></div>
          <div
            style={{
              position: "absolute",
              transform: `translate(-0.75rem,-0.5rem) translate(-50%, -50%) translate(${targetX}px,${targetY}px)`,
              fontSize: 12,
              pointerEvents: "all",
            }}
            className="nodrag nopan flex flex-col items-center -mt-1"
          ></div>
          <div
            style={{
              position: "absolute",
              transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
              fontSize: 12,
              pointerEvents: "all",
            }}
            className="nodrag nopan flex flex-col items-center -mt-1"
          >
            <button
              onClick={() => {
                const newIndex =
                  (CONSTRAINT_TYPES.indexOf(data.constraintType) + 1) %
                  CONSTRAINT_TYPES.length;
                data.onDataChange(id, {
                  constraintType: CONSTRAINT_TYPES[newIndex],
                });
              }}
            >
              <span className="text-xl text-black" title={data.constraintType}>
                {data.constraintType === "response" && (
                  <MdKeyboardDoubleArrowRight />
                )}
                {data.constraintType === "unary-response" && (
                  <MdKeyboardArrowRight />
                )}
                {data.constraintType === "non-response" && (
                  <div className="relative">
                    <MdDoNotDisturb className="absolute -rotate-12 text-orange-800/50" />
                    <MdKeyboardArrowRight />
                  </div>
                )}
              </span>
            </button>
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
