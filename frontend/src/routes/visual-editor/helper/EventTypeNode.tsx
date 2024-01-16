import { Combobox } from "@/components/ui/combobox";
import type { EventTypeQualifierInfo } from "@/types/ocel";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";
import { useContext, useEffect } from "react";
import { LuLink, LuUnlink, LuX } from "react-icons/lu";
import { Handle, Position, type NodeProps } from "reactflow";
import { ConstraintInfoContext } from "./ConstraintInfoContext";
import { ViolationsContext } from "./ViolationsContext";
import type {
  CountConstraint,
  EventTypeNodeData,
  ObjectVariable,
} from "./types";

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

  const { violationsPerNode, showViolationsFor } =
    useContext(ViolationsContext);
  const violations = violationsPerNode?.find((v) => v.nodeID === id);

  const { objectVariables } = useContext(ConstraintInfoContext);
  const hasAssociatedObjects = data.selectedVariables.length > 0;

  function getCountConstraint(): CountConstraint {
    return data.countConstraint;
  }

  useEffect(() => {
    const newSelectedVariables = data.selectedVariables.filter((v) =>
      objectVariables.includes(v.variable),
    );
    if (newSelectedVariables.length !== data.selectedVariables.length) {
      data.onDataChange(id, {
        selectedVariables: newSelectedVariables,
      });
    }
  }, [objectVariables]);

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

  const canAddObjects =
    objectVariables.filter((ot) => qualifierPerObjectType[ot.type].length > 0)
      .length > 0;
  return (
    <div
      className={`border shadow backdrop-blur flex flex-col py-2 px-0.5 rounded-md relative ${
        violations !== undefined && violations.violations.length > 0
          ? "bg-red-100/70  border-red-200"
          : "bg-blue-50/70  border-blue-200"
      }`}
    >
      {violations?.violations !== undefined && (
        <button
          onClick={() => {
            console.log({ showViolationsFor });
            if (showViolationsFor !== undefined) {
              showViolationsFor(violations);
            }
          }}
          className={`absolute right-1 top-1 text-xs flex flex-col items-center gap-x-1 border border-transparent px-1 py-0.5 rounded-sm hover:bg-amber-100/70 hover:border-gray-400/50`}
          title={`${violations.violations.length} violations found`}
        >
          {violations.violations.length > 0 && (
            <ExclamationTriangleIcon className="text-red-400 h-3 mt-1" />
          )}
          {violations.violations.length === 0 && (
            <CheckCircledIcon className="text-green-400 h-3" />
          )}
          <span>{violations.violations.length}</span>
        </button>
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
          className="bg-transparent disabled:cursor-not-allowed disabled:hover:bg-transparent hover:bg-blue-100 w-[4ch] text-center"
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
          className="bg-transparent disabled:cursor-not-allowed disabled:hover:bg-transparent hover:bg-blue-100 w-[4ch] text-center"
          type="text"
          pattern="([0-9]|&#8734;)+"
          defaultValue={
            countConstraint.max === Infinity ? "∞" : countConstraint.max
          }
        />
      </div>
      <div className="text-large font-semibold mt-1 mx-4 flex justify-center items-center">
        <span>{data.eventType}</span>
      </div>
      <div className="mb-1">
        {data.selectedVariables.map((selectedVar, i) => (
          <div
            key={i}
            className="grid grid-cols-[auto,6rem,auto] gap-x-2 items-center w-fit mx-auto"
          >
            <button
              title="Remove"
              className="text-xs my-0 rounded-full transition-colors hover:bg-red-50 hover:outline hover:outline-1 hover:outline-red-400 hover:text-red-400 focus:text-red-500"
              onClick={() => {
                const newSelectedVariables = [...data.selectedVariables];
                newSelectedVariables.splice(i, 1);
                data.onDataChange(id, {
                  selectedVariables: newSelectedVariables,
                });
              }}
            >
              <LuX />
            </button>
            <span className="text-left mb-1">
              <span title={"Object type: " + selectedVar.variable.type}>
                {selectedVar.variable.name}
              </span>
              <span
                className="text-gray-500"
                title={"Qualifier: " + selectedVar.qualifier}
              >
                {" "}
                @{selectedVar.qualifier}
              </span>
            </span>
            <button
              title="Toggle bound"
              className="text-xs py-1 px-1 rounded-sm transition-colors hover:bg-cyan-50 hover:outline hover:outline-1 hover:outline-cyan-400 hover:text-cyan-400 focus:text-cyan-500"
              onClick={() => {
                selectedVar.bound = !selectedVar.bound;
                data.onDataChange(id, {
                  selectedVariables: [...data.selectedVariables],
                });
              }}
            >
              {selectedVar.bound ? <LuLink /> : <LuUnlink />}
            </button>
          </div>
        ))}
      </div>
      <div>
        <Combobox
          title={
            canAddObjects
              ? "Link object variables..."
              : "No options available. Please first add object variables above!"
          }
          options={objectVariables.flatMap((ot) => {
            return qualifierPerObjectType[ot.type].map((qualifier) => ({
              value: JSON.stringify({ objectvariable: ot, qualifier }),
              label: `${ot.name} @${qualifier} (${ot.type})`,
            }));
          })}
          onChange={(jsonValue: string) => {
            if (jsonValue !== undefined && jsonValue !== "") {
              const {
                objectvariable,
                qualifier,
              }: { objectvariable: ObjectVariable; qualifier: string } =
                JSON.parse(jsonValue);
              console.log({ objectvariable, qualifier, jsonValue });
              data.onDataChange(id, {
                selectedVariables: [
                  ...data.selectedVariables,
                  {
                    variable: objectvariable,
                    qualifier,
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
