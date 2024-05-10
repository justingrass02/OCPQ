import AlertHelper from "@/components/AlertHelper";
import clsx from "clsx";
import { memo, useContext } from "react";
import { TbTrash } from "react-icons/tb";
import { Handle, Position, type NodeProps } from "reactflow";
import { VisualEditorContext } from "../VisualEditorContext";
import FilterConstraintChooser from "../box/FilterConstraintChooser";
import NewVariableChooser from "../box/NewVariablesChooser";
import type { EventTypeNodeData } from "../types";
import MiscNodeConstraints from "./MiscNodeConstraints";
import ViolationIndicator from "./ViolationIndicator";
import { getViolationStyles } from "../violation-styles";
export default memo(EventTypeNode);
function EventTypeNode({ data, id, selected }: NodeProps<EventTypeNodeData>) {
  const { violationsPerNode, onNodeDataChange } =
    useContext(VisualEditorContext);

  const violations =
    violationsPerNode === undefined || data.hideViolations === true
      ? undefined
      : violationsPerNode.evalRes[id];
  return (
    <div
      className={clsx(
        "border-2 shadow-lg z-10 flex flex-col py-1 pb-2 px-0.5 rounded-md relative min-h-[5rem] w-[15rem]",
        getViolationStyles(violations),
        selected && "border-dashed",
      )}
    >
      {/* <Toggle
        className="flex w-6 h-6 p-0 absolute right-11"
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
      {violations !== undefined && (
        <ViolationIndicator violationsPerNode={violations} />
      )}

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
          id={id}
          box={data.box}
          updateBox={(box) => onNodeDataChange(id, { box })}
        />
        <FilterConstraintChooser
          id={id}
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
