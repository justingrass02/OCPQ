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
import { LuHash } from "react-icons/lu";

import { Input } from "@/components/ui/input";
import { MdRemoveCircleOutline } from "react-icons/md";
import { EdgeLabelRenderer, getBezierPath, type EdgeProps } from "reactflow";
import QuantifiedObjectEdge from "./QuantifiedObjectEdge";
import { VisualEditorContext } from "./VisualEditorContext";
import type { EventTypeLinkData } from "./types";

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
            <CountChangeDialog
              data={data}
              onChange={(newCountConstraint) => {
                onEdgeDataChange(id, { ...newCountConstraint });
              }}
            />
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  );
}

function CountChangeDialog({
  data,
  onChange,
}: {
  data: EventTypeLinkData;
  onChange: (newCountConstraint: {
    minCount: number | null;
    maxCount: number | null;
  }) => unknown;
}) {
  const [countConstraint, setCountConstraint] = useState({
    minCount: data.minCount,
    maxCount: data.maxCount,
  });
  return (
    <Dialog>
      <DialogTrigger asChild>
        <button
          className="flex flex-col items-center px-1 my-1 py-0.5 rounded-md bg-blue-50/60 hover:bg-blue-200/70"
          title="Update Count Constraint..."
        >
          <LuHash />
          <div className="grid gap-x-1 grid-cols-[1fr_auto_1fr]">
            <span className="text-right">
              {countConstraint.minCount ?? "0"}
            </span>
            <span className="mx-0.5 text-gray-500">-</span>
            <span className="text-left">{countConstraint.maxCount ?? "∞"}</span>
          </div>
        </button>
      </DialogTrigger>
      <DialogPortal>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Update Count Constraint</DialogTitle>
            <DialogDescription>
              Update the allowed minimum and maximum counts of bindings per
              single input binding.
              <br />
              The minimum and maximum counts can also be left unset to not
              enforce a lower or upper bound.
            </DialogDescription>
          </DialogHeader>
          <h3>Min. Count</h3>
          <Input
            type="number"
            className="w-full"
            placeholder="Min. Count"
            value={countConstraint.minCount ?? ""}
            onChange={(ev) => {
              const newVal = ev.currentTarget.valueAsNumber;
              setCountConstraint({ ...countConstraint, minCount: newVal });
            }}
          />
          <h3>Max. Count</h3>
          <Input
            type="number"
            className="w-full"
            placeholder="Max. Count"
            value={countConstraint.maxCount ?? ""}
            onChange={(ev) => {
              const newVal = ev.currentTarget.valueAsNumber;
              setCountConstraint({ ...countConstraint, maxCount: newVal });
            }}
          />
          <DialogClose asChild>
            <Button
              type="button"
              variant="secondary"
              onClick={() => {
                onChange(countConstraint);
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
