import { useContext } from "react";

import { MdRemoveCircleOutline } from "react-icons/md";
import {
  EdgeLabelRenderer,
  getBezierPath,
  useReactFlow,
  type EdgeProps,
} from "reactflow";
import QuantifiedObjectEdge from "./QuantifiedObjectEdge";
import { VisualEditorContext } from "./VisualEditorContext";
import { GATE_NODE_TYPE } from "./const";
import type { GateLinkData } from "./types";

export default function GateLink(props: EdgeProps<GateLinkData>) {
  const {
    id,
    sourceX,
    sourceY,
    source,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
  } = props;
  const { getNode } = useReactFlow();
  const isSourceNodeGate = getNode(source)?.type === GATE_NODE_TYPE;
  const [_edgePath, labelX, labelY] = getBezierPath({
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
      <QuantifiedObjectEdge {...props} />
      {/* <BaseEdge path={edgePath} markerEnd={markerEnd} style={style} /> */}
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
