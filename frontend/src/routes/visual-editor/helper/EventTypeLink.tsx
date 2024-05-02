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
import { EdgeLabelRenderer, getBezierPath, type EdgeProps } from "reactflow";
import QuantifiedObjectEdge from "./QuantifiedObjectEdge";
import { VisualEditorContext } from "./VisualEditorContext";
import type { EventTypeLinkData, TimeConstraint } from "./types";

export default function EventTypeLink(props: EdgeProps<EventTypeLinkData>) {
  const {
    id,
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
    data,
  } = props;
  // TODO: Fix, currently needs to be calculated twice
  const [_edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  });

  const { onEdgeDataChange } = useContext(VisualEditorContext);

  return (
    <>
      <QuantifiedObjectEdge {...props} />
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
            <button
              className="hover:text-red-500 text-red-400/30  rounded-lg text-sm"
              title="Delete edge"
              onClick={() => onEdgeDataChange(id, undefined)}
            >
              <MdRemoveCircleOutline />
            </button>
            <TimeDurationChangeDialog
              data={data}
              onChange={(newTimeConstraint) => {
                onEdgeDataChange(id, { timeConstraint: newTimeConstraint });
              }}
            />
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}

function TimeDurationChangeDialog({
  data,
  onChange,
}: {
  data: EventTypeLinkData;
  onChange: (newTimeConstraint: TimeConstraint) => unknown;
}) {
  const [timeConstraint, setTimeConstraint] = useState<TimeConstraint>(
    data.timeConstraint,
  );
  return (
    <Dialog>
      <DialogTrigger asChild>
        <button
          className="flex flex-col items-center px-1 my-1 py-0.5 rounded-md bg-blue-50/60 hover:bg-blue-200/70"
          title="Update time constraint..."
        >
          <LuClock />
          <div className="grid gap-x-1 grid-cols-[1fr_auto_1fr]">
            <span className="text-right">
              {formatSeconds(data.timeConstraint.minSeconds)}
            </span>
            <span className="mx-0.5 text-gray-500">-</span>
            <span className="text-left">
              {formatSeconds(data.timeConstraint.maxSeconds)}
            </span>
          </div>
        </button>
      </DialogTrigger>
      <DialogPortal>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Update time constraints</DialogTitle>
            <DialogDescription>
              Update the minimum and maximum time duration between the events
              below.
              <br />
              Negative values are allowed, as well as ∞ (inf) and -∞ (-inf).
            </DialogDescription>
          </DialogHeader>
          <h3>From</h3>
          <TimeDurationInput
            durationSeconds={timeConstraint.minSeconds}
            onChange={(v) => {
              setTimeConstraint({ ...timeConstraint, minSeconds: v });
            }}
          />
          <h3>To</h3>
          <TimeDurationInput
            durationSeconds={data.timeConstraint.maxSeconds}
            onChange={(v) => {
              setTimeConstraint({ ...timeConstraint, maxSeconds: v });
            }}
          />
          <DialogClose asChild>
            <Button
              disabled={timeConstraint.minSeconds > timeConstraint.maxSeconds}
              type="button"
              variant="secondary"
              onClick={() => {
                onChange(timeConstraint);
              }}
            >
              Save
            </Button>
          </DialogClose>
        </DialogContent>
      </DialogPortal>
    </Dialog>
  );
}
