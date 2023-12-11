import type { EventTypeQualifierInfo, EventTypeQualifier } from "@/types/ocel";
import { useContext } from "react";
import { Handle, Position, type NodeProps } from "reactflow";
import { ConstraintInfoContext } from "./ConstraintInfoContext";
import { Combobox } from "@/components/ui/combobox";
import { Button } from "@/components/ui/button";
import { LuLink, LuUnlink } from "react-icons/lu";
import type { CountConstraint, SelectedVariables } from "./types";
import { ViolationsContext } from "./ViolationsContext";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";

export type EventTypeNodeData = {
  label: string;
  eventTypeQualifier: EventTypeQualifier;
  objectTypeToColor: Record<string, string>;
  countConstraint: CountConstraint;
  selectedVariables: SelectedVariables;
  onDataChange: (id: string, newData: Partial<EventTypeNodeData>) => unknown;
};

function getObjectType(qualInfo: EventTypeQualifierInfo) {
  if (qualInfo.object_types.length > 1) {
    console.warn(
      "Warning: Encountered multiple object types. This is currently not supported",
    );
  }
  return qualInfo.object_types[0];
}

export default function EventTypeNode({
  data,
  id,
}: NodeProps<EventTypeNodeData>) {
  // Sort by object type
  const qualifiers = Object.keys(data.eventTypeQualifier).sort((a, b) =>
    getObjectType(data.eventTypeQualifier[a]).localeCompare(
      getObjectType(data.eventTypeQualifier[b]),
    ),
  );
  const qualifierPerObjectType: Record<string, string[]> = {};
  for (const ot of Object.keys(data.objectTypeToColor)) {
    qualifierPerObjectType[ot] = qualifiers.filter((q) =>
      data.eventTypeQualifier[q].object_types.includes(ot),
    );
  }

  const violationsContext = useContext(ViolationsContext);
  const violations = violationsContext?.violationsPerNode.find(
    (v) => v.nodeID === id,
  );

  const { objectVariables } = useContext(ConstraintInfoContext);
  const hasAssociatedObjects = data.selectedVariables.length > 0;

  function getCountConstraint(): CountConstraint {
    return data.countConstraint;
  }

  function handleCountInput(
    type: "min" | "max",
    ev:
      | React.FocusEvent<HTMLInputElement>
      | React.KeyboardEvent<HTMLInputElement>,
  ) {
    let value = 0;
    try {
      value = parseInt(ev.currentTarget.value);
    } catch (e) {}
    if (isNaN(value)) {
      if (
        ev.currentTarget.value === "∞" ||
        ev.currentTarget.value === "inf" ||
        ev.currentTarget.value === "infinity" ||
        ev.currentTarget.value === "infty"
      ) {
        value = Infinity;
      }
    }
    value = Math.max(0, value);
    if (!isNaN(value)) {
      ev.currentTarget.value = value === Infinity ? "∞" : value.toString();
    }
    const newCountConstraint = getCountConstraint();
    if (type === "min") {
      newCountConstraint.min = value;
    } else {
      newCountConstraint.max = value;
    }
    data.onDataChange(id, {
      countConstraint: newCountConstraint,
    });
  }
  const countConstraint = getCountConstraint();
  return (
    <div
      className={`border shadow backdrop-blur flex flex-col py-2 px-0.5 rounded-md relative ${
        violations !== undefined && violations.violations.length > 0
          ? "bg-red-100/70  border-red-200"
          : "bg-blue-50/70  border-blue-200"
      }`}
    >
      {violations?.violations !== undefined && (
        <div
          className={`absolute right-1 top-1 text-xs flex flex-col items-center gap-x-1`}
          title={`${violations.violations.length} violations found`}
        >
          {violations.violations.length > 0 && (
            <ExclamationTriangleIcon className="text-red-400 h-3 mt-1" />
          )}
          {violations.violations.length === 0 && (
            <CheckCircledIcon className="text-green-400 h-3" />
          )}
          <span>{violations.violations.length}</span>
        </div>
      )}
      <div className="absolute left-2 -top-[1.1rem] px-1 border border-b-0 border-inherit bg-inherit text-xs">
        <input
          disabled={!hasAssociatedObjects}
          onBlur={(ev) => {
            handleCountInput("min", ev);
          }}
          onKeyDown={(ev) => {
            if (ev.key === "Enter") {
              handleCountInput("min", ev);
              ev.currentTarget.blur();
            }
          }}
          className="bg-transparent disabled:cursor-not-allowed disabled:bg-gray-100 disabled:hover:bg-gray-100 hover:bg-blue-100 w-[4ch] text-center"
          type="text"
          pattern="([0-9]|&#8734;)+"
          defaultValue={
            countConstraint.min === Infinity ? "∞" : countConstraint.min
          }
        />
        -
        <input
          disabled={!hasAssociatedObjects}
          onBlur={(ev) => {
            handleCountInput("max", ev);
          }}
          onKeyDown={(ev) => {
            if (ev.key === "Enter") {
              handleCountInput("max", ev);
              ev.currentTarget.blur();
            }
          }}
          className="bg-transparent disabled:cursor-not-allowed disabled:bg-gray-100 disabled:hover:bg-gray-100 hover:bg-blue-100 w-[4ch] text-center"
          type="text"
          pattern="([0-9]|&#8734;)+"
          defaultValue={
            countConstraint.max === Infinity ? "∞" : countConstraint.max
          }
        />
      </div>
      <div className="text-large font-semibold mt-1 mx-4 flex justify-center items-center">
        <span>{id}</span>
      </div>
      <div className="mb-1">
        {data.selectedVariables.map((selectedVar) => (
          <div key={selectedVar.variable.name}>
            {selectedVar.variable.name} - {selectedVar.qualifier}{" "}
            <Button
              className="text-xs px-2 my-0 py-0"
              variant="ghost"
              onClick={() => {
                selectedVar.bound = !selectedVar.bound;
                data.onDataChange(id, {
                  selectedVariables: [...data.selectedVariables],
                });
              }}
            >
              {selectedVar.bound ? <LuLink /> : <LuUnlink />}
            </Button>
          </div>
        ))}
      </div>
      <div>
        <Combobox
          options={objectVariables
            .filter((ot) => qualifierPerObjectType[ot.type].length > 0)
            .map((ot) => ({
              value: ot.name,
              label: `${ot.name} (${ot.type})`,
            }))}
          onChange={(value: string) => {
            const type = objectVariables.find((ov) => ov.name === value)?.type;
            if (type !== undefined) {
              data.onDataChange(id, {
                selectedVariables: [
                  ...data.selectedVariables,
                  {
                    variable: { name: value, type },
                    qualifier: qualifierPerObjectType[type][0],
                    bound: false,
                  },
                ],
              });
            }
          }}
          name="Variable"
          value={""}
        />
      </div>
      {hasAssociatedObjects && (
        <Handle
          className="!w-3 !h-3"
          position={Position.Top}
          type="target"
          id={id + "-target"}
        />
      )}
      {hasAssociatedObjects && (
        <Handle
          className="!w-3 !h-3"
          position={Position.Bottom}
          type="source"
          id={id + "-source"}
        />
      )}
    </div>
  );
}
