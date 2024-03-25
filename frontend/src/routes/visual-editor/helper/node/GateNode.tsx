import { Position, type NodeProps, Handle } from "reactflow";
import type { GateNodeData } from "../types";
import { useContext } from "react";
import { VisualEditorContext } from "../VisualEditorContext";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";
import AlertHelper from "@/components/AlertHelper";
import { TbTrash } from "react-icons/tb";

export default function EventTypeNode({ data, id }: NodeProps<GateNodeData>) {
  const { violationsPerNode, showViolationsFor, onNodeDataChange, ocelInfo } =
    useContext(VisualEditorContext);

  const hideViolations: boolean | undefined = false;
  const violations = hideViolations
    ? undefined
    : violationsPerNode?.find((v) => v.nodeID === id);

  return (
    <div
      title={data.type}
      className={`border shadow z-10 backdrop-blur flex flex-col items-center justify-center pt-1.5 py-0.5 px-0.5 rounded-md relative min-w-[8rem] min-h-[5rem] font-mono text-4xl font-bold
      ${
        violations?.violations !== undefined && violations.violations.length > 0
          ? "bg-red-200 border-red-300"
          : violations?.violations === undefined
          ? "bg-blue-50 border-blue-100"
          : "bg-green-200 border-green-300"
      }`}
    >
      {violations?.violations !== undefined && (
        <button
          onClick={() => {
            if (showViolationsFor !== undefined) {
              showViolationsFor(violations);
            }
          }}
          className={`absolute right-0 top-0 text-xs flex flex-col items-center gap-x-1 border border-transparent px-1 py-0.5 rounded-sm hover:bg-amber-100/70 hover:border-gray-400/50`}
          title={`Found ${violations.violations.length} Violations of ${violations.numBindings} Bindings`}
        >
          {violations.violations.length > 0 && (
            <ExclamationTriangleIcon className="text-red-400 h-3 mt-1" />
          )}
          {violations.violations.length === 0 && (
            <CheckCircledIcon className="text-green-400 h-3" />
          )}
          <div className="flex flex-col items-center justify-center">
            {violations.violations.length}
            <div className="text-[0.6rem] leading-none text-muted-foreground">
              {Math.round(
                100 *
                  100 *
                  (violations.violations.length / violations.numBindings),
              ) / 100.0}
              %
            </div>
          </div>
        </button>
      )}
      <div className="">
        {data.type === "not" && "¬"}
        {data.type === "or" && "∨"}
        {data.type === "and" && "∧"}
        <Handle
          className="!w-3 !h-3"
          position={Position.Top}
          type="target"
          id={id + "-target"}
        />
        {data.type === "not" && (
          <>
            <Handle
              className="!w-3 !h-3"
              position={Position.Bottom}
              type="source"
              id={id + "-source"}
            />
          </>
        )}

        {data.type !== "not" && (
          <>
            <Handle
              className="!w-3 !h-3 mt-9"
              position={Position.Left}
              type="source"
              id={id + "-left-source"}
            />
            <Handle
              className="!w-3 !h-3 mt-9"
              position={Position.Right}
              type="source"
              id={id + "-right-source"}
            />
          </>
        )}
      </div>
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
    </div>
  );
}
