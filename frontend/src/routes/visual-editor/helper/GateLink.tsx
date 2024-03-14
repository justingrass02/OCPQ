import TimeDurationInput, {
  formatSeconds,
} from "@/components/TimeDurationInput";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogPortal,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useContext, useState } from "react";
import { LuClock } from "react-icons/lu";

import { MdRemoveCircleOutline } from "react-icons/md";
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
  getStraightPath,
  getSimpleBezierPath,
  useReactFlow,
} from "reactflow";
import type { EventTypeLinkData, GateLinkData, TimeConstraint } from "./types";
import { VisualEditorContext } from "./VisualEditorContext";
import { GATE_NODE_TYPE } from "./const";

export default function GateLink({
  id,
  source,
  sourceHandleId,
  target,
  targetHandleId,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  markerEnd,
  data,
  style = {},
}: EdgeProps<GateLinkData>) {
  const {getNode} = useReactFlow();
  const isSourceNodeGate = getNode(source)?.type === GATE_NODE_TYPE;
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY: sourceY + (isSourceNodeGate ? 0 : 0),
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });
  const { onEdgeDataChange } = useContext(VisualEditorContext);
  return (
    <>
      <BaseEdge path={edgePath} markerEnd={markerEnd} style={style} />
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
          <button
            className="hover:text-red-500 text-red-400/30  rounded-lg text-sm"
            title="Delete edge"
            // onClick={() => { console.log({source, sourceHandleId, target, targetHandleId}); }}
            onClick={() => onEdgeDataChange(id, undefined)}
          >
            <MdRemoveCircleOutline />
          </button>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}
