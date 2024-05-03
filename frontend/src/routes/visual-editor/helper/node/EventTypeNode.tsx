import AlertHelper from "@/components/AlertHelper";
import { Toggle } from "@/components/ui/toggle";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";
import { useContext } from "react";
import { PiSirenDuotone, PiSirenThin } from "react-icons/pi";
import { TbTrash } from "react-icons/tb";
import { Handle, Position, type NodeProps } from "reactflow";
import { VisualEditorContext } from "../VisualEditorContext";
import type { EventTypeNodeData, ViolationsPerNode } from "../types";
import MiscNodeConstraints from "./MiscNodeConstraints";
import NewVariableChooser from "../box/NewVariablesChooser";
import FilterConstraintChooser from "../box/FilterConstraintChooser";

export default function EventTypeNode({
  data,
  id,
}: NodeProps<EventTypeNodeData>) {
  const { violationsPerNode, onNodeDataChange, ocelInfo } =
    useContext(VisualEditorContext);

  const violations =
    data.hideViolations === true
      ? undefined
      : violationsPerNode?.find((v) => v.nodeID === id);

  return (
    <div
      className={`border shadow z-10 backdrop-blur flex flex-col py-1 pb-2 px-0.5 rounded-md relative min-h-[5rem] w-[15rem] ${
        violations !== undefined
          ? violations.violations.length > 0
            ? "bg-red-50  border-red-200"
            : "bg-emerald-50  border-emerald-200 "
          : "bg-blue-50 border-blue-200"
      }`}
    >
      {/* <Toggle
        className="flex w-6 h-6 p-0 absolute left-1"
        variant="outline"
        title={
          data.hideViolations === true
            ? "Hide violations (just filter)"
            : "Show violations"
        }
        pressed={data.hideViolations === true}
        onPressedChange={(pressed) => {
          onNodeDataChange(id, { ...data, hideViolations: pressed });
        }}
      >
        {data.hideViolations !== true && (
          <PiSirenDuotone className="text-blue-500" />
        )}
        {data.hideViolations === true && (
          <PiSirenThin className="text-gray-400" />
        )}
      </Toggle> */}
      <ViolationIndicator violationsPerNode={violations} />

      <AlertHelper
        trigger={
          <button
            className="absolute -top-3.5 right-1 opacity-10 hover:opacity-100 hover:text-red-500"
            title="Delete node"
          >
            <TbTrash size={12} />
          </button>
        }
        title="Are you sure?"
        initialData={undefined}
        content={() => (
          <>This node and all contained constraints will be deleted.</>
        )}
        submitAction="Delete"
        onSubmit={() => {
          onNodeDataChange(id, undefined);
        }}
      />
      <div className="text-large font-semibold mx-4 flex flex-col justify-center items-center">
        <MiscNodeConstraints
          id={id}
          data={data}
          onNodeDataChange={onNodeDataChange}
        />
        <NewVariableChooser
          box={data.box}
          updateBox={(box) => onNodeDataChange(id, { box })}
        />
        <FilterConstraintChooser
          box={data.box}
          updateBox={(box) => onNodeDataChange(id, { box })}
        />
      </div>
      <div>
        <Handle
          className="!w-3 !h-3"
          position={Position.Top}
          type="target"
          id={id + "-target"}
        />

        <Handle
          className="!w-3 !h-3"
          position={Position.Bottom}
          type="source"
          id={id + "-source"}
        />
      </div>
    </div>
  );
}

function ViolationIndicator({
  violationsPerNode,
}: {
  violationsPerNode: ViolationsPerNode | undefined;
}) {
  const { showViolationsFor } = useContext(VisualEditorContext);
  if (violationsPerNode?.violations === undefined) {
    return null;
  }

  return (
    <button
      onClick={() => {
        if (showViolationsFor !== undefined) {
          showViolationsFor(violationsPerNode);
        }
      }}
      className={`absolute right-1 top-1 text-xs flex flex-col items-center gap-x-1 border border-transparent px-1 py-0.5 rounded-sm hover:bg-amber-100/70 hover:border-gray-400/50`}
      title={`Found ${violationsPerNode.violations.length} Violations of ${violationsPerNode.numBindings} Bindings`}
    >
      {violationsPerNode.violations.length > 0 && (
        <ExclamationTriangleIcon className="text-red-400 h-3 mt-1" />
      )}
      {violationsPerNode.violations.length === 0 && (
        <CheckCircledIcon className="text-green-400 h-3" />
      )}
      <div className="flex flex-col items-center justify-center">
        {violationsPerNode.violations.length}
        <div className="text-[0.6rem] leading-none text-muted-foreground">
          {Math.round(
            100 *
              100 *
              (violationsPerNode.violations.length /
                violationsPerNode.numBindings),
          ) / 100.0}
          %
        </div>
      </div>
    </button>
  );
}
