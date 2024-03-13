import AlertHelper from "@/components/AlertHelper";
import { Combobox } from "@/components/ui/combobox";
import { Toggle } from "@/components/ui/toggle";
import type { EventTypeQualifierInfo } from "@/types/ocel";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";
import { useContext, useEffect } from "react";
import { LuDelete, LuLink, LuUnlink, LuX } from "react-icons/lu";
import { PiSirenDuotone, PiSirenThin } from "react-icons/pi";
import { TbTrash } from "react-icons/tb";
import { Handle, Position, type NodeProps } from "reactflow";
import { ConstraintInfoContext } from "../ConstraintInfoContext";
import { VisualEditorContext } from "../VisualEditorContext";
import { parseIntAllowInfinity } from "../infinity-input";
import type {
  CountConstraint,
  EventTypeNodeData,
  ObjectVariable,
} from "../types";
import MiscNodeConstraints from "./MiscNodeConstraints";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import toast from "react-hot-toast";
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

  const { violationsPerNode, showViolationsFor, onNodeDataChange, ocelInfo } =
    useContext(VisualEditorContext);
  const qualifierPerObjectType: Record<string, string[]> = {};
  if (ocelInfo !== undefined) {
    for (const ot of ocelInfo.object_types) {
      qualifierPerObjectType[ot.name] = qualifiers.filter((q) =>
        data.eventTypeQualifier[q].object_types.includes(ot.name),
      );
    }
  }
  const violations =
    data.hideViolations === true
      ? undefined
      : violationsPerNode?.find((v) => v.nodeID === id);

  const { objectVariables } = useContext(ConstraintInfoContext);
  const hasAssociatedObjects = data.selectedVariables.length > 0;

  function getCountConstraint(): CountConstraint {
    return data.countConstraint;
  }

  useEffect(() => {
    const newSelectedVariables = data.selectedVariables.filter((v) =>
      objectVariables.find(
        (ov) => ov.name === v.variable.name && ov.type === v.variable.type,
      ),
    );
    if (newSelectedVariables.length !== data.selectedVariables.length) {
      onNodeDataChange(id, {
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
    let value = parseIntAllowInfinity(ev.currentTarget.value);
    if (value === undefined) {
      return;
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
    onNodeDataChange(id, {
      countConstraint: newCountConstraint,
    });
  }
  const countConstraint = getCountConstraint();
  const canAddObjects =
    objectVariables.filter((ot) => qualifierPerObjectType[ot.type].length > 0)
      .length > 0;
  return (
    <div
      className={`border shadow z-10 backdrop-blur flex flex-col py-2 px-0.5 rounded-md relative w-[15rem] ${
        violations !== undefined
          ? violations.violations.length > 0
            ? "bg-red-100/70  border-red-200"
            : "bg-emerald-100/70  border-emerald-200 "
          : "bg-blue-100/70 border-blue-200"
      }`}
    >
      <Toggle
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
      </Toggle>
      {violations?.violations !== undefined && (
        <button
          onClick={() => {
            if (showViolationsFor !== undefined) {
              showViolationsFor(violations);
            }
          }}
          className={`absolute right-1 top-1 text-xs flex flex-col items-center gap-x-1 border border-transparent px-1 py-0.5 rounded-sm hover:bg-amber-100/70 hover:border-gray-400/50`}
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
      <div className="absolute left-2 -top-[1.1rem] px-1 border border-b-0 border-inherit bg-inherit text-xs z-0">
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
      <div className="text-large font-semibold mt-1 mx-4 flex flex-col justify-center items-center">
        <AlertHelper
          trigger={
            <button
              className=" hover:bg-blue-200/80 px-1 block rounded text-lg max-w-[11rem]"
              title="Edit event type"
            >
              {data.eventType.type === "exactly" && (
                <span>{data.eventType.value}</span>
              )}
              {data.eventType.type === "any" && (
                <span className="underline">any</span>
              )}
              {(data.eventType.type === "anyOf" ||
                data.eventType.type === "anyExcept") && (
                <p className="text-xs leading-none py-1">
                  <span className="font-mono">
                    {data.eventType.type === "anyOf" ? "∈" : "∉"}
                  </span>{" "}
                  {"{"}
                  {data.eventType.values.map((et, i) => (
                    <span key={i}>
                      {et}
                      {(data.eventType.type === "anyOf" ||
                        data.eventType.type === "anyExcept") &&
                      i < data.eventType.values.length - 1
                        ? ","
                        : "}"}
                      <br />
                    </span>
                  ))}
                </p>
              )}
            </button>
          }
          title="Edit Event Type"
          initialData={{ ...data.eventType }}
          content={({ data: d, setData: setD }) => (
            <>
              <div className="flex flex-col gap-y-1">
                <Label>Mode</Label>
                <Select
                  value={d.type}
                  onValueChange={(v) => {
                    if (v === "exactly") {
                      setD({ type: v, value: "" });
                    } else if (v === "any") {
                      setD({ type: "any" });
                    } else if (v === "anyOf") {
                      setD({ type: "anyOf", values: [""] });
                    } else if (v === "anyExcept") {
                      setD({ type: "anyExcept", values: [""] });
                    }
                  }}
                >
                  <SelectTrigger className={"w-[200px] font-medium"}>
                    <SelectValue placeholder="Event Type Mode" />
                  </SelectTrigger>
                  <SelectContent>
                    {[
                      { value: "exactly", label: "exactly" },
                      { value: "any", label: "any" },
                      { value: "anyOf", label: "any of" },
                      { value: "anyExcept", label: "any except" },
                    ].map((e) => (
                      <SelectItem key={e.value} value={e.value}>
                        {e.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="flex flex-col gap-y-1 mt-2">
                <Label>
                  {d.type === "anyExcept" || d.type === "anyOf"
                    ? "Values"
                    : d.type === "exactly"
                    ? "Value"
                    : ""}
                </Label>
                {d.type === "exactly" && (
                  <>
                    {ocelInfo !== undefined && (
                      <Combobox
                        options={ocelInfo.event_types.map((et) => ({
                          value: et.name,
                          label: et.name,
                        }))}
                        name="Event Type"
                        value={d.value}
                        onChange={(newVal) => setD({ ...d, value: newVal })}
                      />
                    )}
                  </>
                )}
                {(d.type === "anyExcept" || d.type === "anyOf") && (
                  <>
                    <div className="flex flex-col gap-y-1 mb-2">
                      {ocelInfo !== undefined &&
                        d.values.map((v, i) => (
                          <div key={i} className="flex gap-x-1 items-center">
                            <Combobox
                              options={ocelInfo.event_types.map((et) => ({
                                value: et.name,
                                label: et.name,
                              }))}
                              name="Event Type"
                              value={v}
                              onChange={(newVal) => {
                                const values = [...d.values];
                                values[i] = newVal;
                                setD({ ...d, values });
                              }}
                            />
                            <Button
                              size="icon"
                              title="Remove"
                              className="h-5 w-5 ml-2 text-red-500"
                              variant="outline"
                              onClick={() => {
                                const newVals = [...d.values];
                                newVals.splice(i, 1);
                                const newD = {
                                  ...d,
                                  values: newVals,
                                };
                                setD(newD);
                              }}
                            >
                              <LuDelete />
                            </Button>
                          </div>
                        ))}
                    </div>
                    <Button
                      className="block w-fit -mt-1"
                      variant="outline"
                      onClick={() => {
                        setD({ ...d, values: [...d.values, ""] });
                      }}
                    >
                      Add
                    </Button>
                  </>
                )}
              </div>
            </>
          )}
          submitAction="Save"
          onSubmit={(d, ev) => {
            if (d.type === "anyOf" || d.type === "anyExcept") {
              const filteredValues = d.values.filter((v) => v !== "");
              if (filteredValues.length === 0) {
                ev.preventDefault();
                toast("Please select at least one event type.");
                return;
              }
              onNodeDataChange(id, {
                ...data,
                eventType: { ...d, values: filteredValues },
              });
            } else if (d.type === "exactly") {
              if (d.value === "") {
                ev.preventDefault();
                toast("Please select an event type.");
                return;
              }
              onNodeDataChange(id, { ...data, eventType: d });
            } else {
              onNodeDataChange(id, { ...data, eventType: d });
            }
          }}
        />
        <MiscNodeConstraints
          id={id}
          data={data}
          onNodeDataChange={onNodeDataChange}
        />
      </div>
      <div className="mb-1">
        {data.selectedVariables.map((selectedVar, i) => (
          <div
            key={i}
            className="grid grid-cols-[auto,8rem,1.2rem] gap-x-2 items-center w-fit mx-auto"
          >
            <button
              title="Remove"
              className="text-xs my-0 rounded-full transition-colors hover:bg-red-50 hover:outline hover:outline-1 hover:outline-red-400 hover:text-red-400 focus:text-red-500"
              onClick={() => {
                const newSelectedVariables = [...data.selectedVariables];
                newSelectedVariables.splice(i, 1);
                onNodeDataChange(id, {
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
                {selectedVar.qualifier !== undefined
                  ? `@${selectedVar.qualifier}`
                  : ""}
              </span>
            </span>
            {!selectedVar.variable.initiallyBound && (
              <button
                title="Toggle bound"
                className="text-xs py-1 px-1 rounded-sm transition-colors hover:bg-cyan-50 hover:outline hover:outline-1 hover:outline-cyan-400 hover:text-cyan-400 focus:text-cyan-500"
                onClick={() => {
                  selectedVar.bound = !selectedVar.bound;
                  onNodeDataChange(id, {
                    selectedVariables: [...data.selectedVariables],
                  });
                }}
              >
                {selectedVar.bound ? <LuLink /> : <LuUnlink />}
              </button>
            )}
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
          options={
            data.eventType.type === "anyExcept" ||
            data.eventType.type === "anyOf"
              ? objectVariables
                  .filter(
                    (ov) =>
                      data.selectedVariables.find(
                        (v) => v.variable.name === ov.name,
                      ) === undefined,
                  )
                  .map((ov) => ({
                    value: JSON.stringify({
                      objectvariable: ov,
                      qualifier: undefined,
                    }),
                    label: `${ov.name} (${ov.type})`,
                  }))
              : objectVariables.flatMap((ot) => {
                  return qualifierPerObjectType[ot.type]
                    .filter(
                      (qualifier) =>
                        data.selectedVariables.find(
                          (sv) =>
                            sv.variable.name === ot.name &&
                            sv.qualifier === qualifier,
                        ) === undefined,
                    )
                    .map((qualifier) => ({
                      value: JSON.stringify({
                        objectvariable: ot,
                        qualifier,
                      }),
                      label: `${ot.name} @${qualifier} (${ot.type})`,
                    }));
                })
          }
          onChange={(jsonValue: string) => {
            if (jsonValue !== undefined && jsonValue !== "") {
              const {
                objectvariable,
                qualifier,
              }: {
                objectvariable: ObjectVariable;
                qualifier: string | undefined;
              } = JSON.parse(jsonValue);
              onNodeDataChange(id, {
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
