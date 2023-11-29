import { MdRemoveCircleOutline } from "react-icons/md";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from "reactflow";
import { useContext } from "react";
import { VisualEditorContext } from "./visual-editor-context";
import type { DependencyType } from "../evaluation/construct-tree";

export const EVENT_TYPE_LINK_TYPE = "eventTypeLink";
export type VariableChangeOptions = {
  linkID: string;
  type: "in" | "out";
  newValue: string;
};
export type EventTypeLinkData = {
  color: string;
  dependencyType: DependencyType;
  inVariable: string;
  outVariable: string;
  onVariableChange: (change: VariableChangeOptions) => unknown;
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
  // Slightly modify sourceX and targetX to force a little overlap (of start/end of arrow)
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX: sourceX - 1,
    sourceY,
    sourcePosition,
    targetX: targetX + 1,
    targetY,
    targetPosition,
  });

  const { mode } = useContext(VisualEditorContext);
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
          >
            <button
              onClick={() => {
                const newVar = prompt("New variable name");
                if (newVar !== null) {
                  
                  data.onVariableChange({
                    linkID: id,
                    newValue: newVar,
                    type: "in",
                  });
                }
              }}
              className="bg-slate-50/90 rounded-sm px-1 border border-slate-100 mb-1 flex flex-col"
            >
              <span
                className=" font-mono font-semibold"
                style={{ color: data.color }}
              >
                {data.inVariable}
              </span>
            </button>
          </div>
          <div
            style={{
              position: "absolute",
              transform: `translate(${
                data.dependencyType === "all" ||
                data.dependencyType === "existsInTarget"
                  ? "-1.4rem"
                  : "-0.75rem"
              },-0.5rem) translate(-50%, -50%) translate(${targetX}px,${targetY}px)`,
              fontSize: 12,
              pointerEvents: "all",
            }}
            className="nodrag nopan flex flex-col items-center -mt-1"
          >
            <button
              onClick={() => {
                const newVar = prompt("New variable name");
                if (newVar !== null) {
                  data.onVariableChange({
                    linkID: id,
                    newValue: newVar,
                    type: "out",
                  });
                }
              }}
              className="bg-slate-50/90 rounded-sm px-1 border border-slate-100 mb-1 flex flex-col"
            >
              <span
                className=" font-mono font-semibold"
                style={{ color: data.color }}
              >
                {data.outVariable}
              </span>
            </button>
          </div>
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
              className={`text-gray-700 ${
                mode === "view-tree" ? "rotate-90 -mr-1" : "-mt-1"
              }`}
              title={(() => {
                if (data.dependencyType === "simple") {
                  return "Equal (single object)";
                }
                if (data.dependencyType === "all") {
                  return "Equal (all objects)";
                }
                if (data.dependencyType === "existsInSource") {
                  return "There is an object in source set that is equal to object target";
                }
                if (data.dependencyType === "existsInTarget") {
                  return "There is an object in target set that is equal to object source";
                }
                return "Unknown dependency type";
              })()}
            >
              {data.dependencyType === "simple" && "="}
              {data.dependencyType === "all" && "≛"}
              {data.dependencyType === "existsInTarget" && "∈"}
              {data.dependencyType === "existsInSource" && "∋"}
            </span>
            {mode === "normal" && (
              <button
                className="hover:text-red-500 text-red-400/30  rounded-lg text-sm"
                title="Delete edge"
                onClick={() => data.onDelete(id)}
              >
                <MdRemoveCircleOutline />
              </button>
            )}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}
