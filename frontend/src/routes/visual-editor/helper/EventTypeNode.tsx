import AlertHelper from "@/components/AlertHelper";
import TimeDurationInput, {
  formatSeconds,
} from "@/components/TimeDurationInput";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Toggle } from "@/components/ui/toggle";
import type { EventTypeQualifierInfo } from "@/types/ocel";
import {
  CheckCircledIcon,
  ExclamationTriangleIcon,
} from "@radix-ui/react-icons";
import { useContext, useEffect } from "react";
import toast from "react-hot-toast";
import { AiOutlineNumber } from "react-icons/ai";
import {
  CgRowFirst,
  CgRowLast,
  CgStopwatch
} from "react-icons/cg";
import { LuDelete, LuLink, LuUnlink, LuX } from "react-icons/lu";
import {
  PiSirenDuotone,
  PiSirenThin
} from "react-icons/pi";
import { TbTrash } from "react-icons/tb";
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
      className={`border shadow z-10 backdrop-blur flex flex-col py-2 px-0.5 rounded-md relative ${
        violations !== undefined ? (violations.violations.length > 0
          ? "bg-red-100/70  border-red-200"
          : "bg-emerald-50/70  border-emerald-200 ") : "bg-blue-50/70 border-blue-200"
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
          data.onDataChange(id, { ...data, hideViolations: pressed });
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
          data.onDataChange(id, undefined);
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
        <span>{data.eventType}</span>
        <div className="flex gap-x-2">
          <button
            className="flex items-center gap-x-2 px-1 py-0.5 rounded border border-blue-300/30 my-1 hover:bg-blue-300/20 text-xs font-light"
            title={
              data.firstOrLastEventOfType === "first"
                ? "First matching event"
                : data.firstOrLastEventOfType === "last"
                ? "Last matching event"
                : "Any matching event"
            }
            onClick={() => {
              if (data.firstOrLastEventOfType === undefined) {
                data.onDataChange(id, {
                  ...data,
                  firstOrLastEventOfType: "first",
                });
              } else if (data.firstOrLastEventOfType === "first") {
                data.onDataChange(id, {
                  ...data,
                  firstOrLastEventOfType: "last",
                });
              } else {
                data.onDataChange(id, {
                  ...data,
                  firstOrLastEventOfType: undefined,
                });
              }
            }}
          >
            {data.firstOrLastEventOfType === undefined && (
              <div className="relative text-gray-200">
                <CgRowLast className="absolute brightness-75" />
                <CgRowFirst className=" brightness-75" />
              </div>
            )}
            {data.firstOrLastEventOfType === "first" && (
              <CgRowFirst className="text-blue-500" />
            )}
            {data.firstOrLastEventOfType === "last" && (
              <CgRowLast className="text-blue-500" />
            )}
          </button>
          <AlertHelper
            initialData={
              data.waitingTimeConstraint != null
                ? { ...data.waitingTimeConstraint }
                : { minSeconds: 0, maxSeconds: Infinity }
            }
            trigger={
              <button
                title="Edit Waiting Time Constraint"
                className="flex items-center gap-x-2 px-1 py-0.5 rounded border border-blue-300/30 my-1 hover:bg-blue-300/20 text-xs font-light"
                onClick={() => {}}
              >
                <CgStopwatch
                  className={
                    data.waitingTimeConstraint === undefined
                      ? "text-gray-500/50"
                      : "text-blue-500"
                  }
                />
                {data.waitingTimeConstraint !== undefined && (
                  <>
                    {formatSeconds(data.waitingTimeConstraint?.minSeconds ?? 0)}
                    <span>-</span>
                    {formatSeconds(
                      data.waitingTimeConstraint?.maxSeconds ?? Infinity,
                    )}
                  </>
                )}
              </button>
            }
            title={"Waiting Time Constraint"}
            submitAction={"Submit"}
            onSubmit={(waitingTimeConstraintData, ev) => {
              if (
                waitingTimeConstraintData.minSeconds >
                waitingTimeConstraintData.maxSeconds
              ) {
                toast(
                  "Maximal waiting time must not be smaller than minimal waiting time.",
                );
                ev.preventDefault();
                return;
              }
              let newWaitingTimeConstraintData:
                | { minSeconds: number; maxSeconds: number }
                | undefined = waitingTimeConstraintData;
              if (
                waitingTimeConstraintData.minSeconds === 0 &&
                waitingTimeConstraintData.maxSeconds === Infinity
              ) {
                newWaitingTimeConstraintData = undefined;
              }
              data.onDataChange(id, {
                ...data,
                waitingTimeConstraint: newWaitingTimeConstraintData,
              });
            }}
            content={({ data, setData }) => {
              return (
                <>
                  <span className="mb-2 block">
                    Please select the minimal and maximal waiting time below.
                  </span>

                  <h3>Minimum</h3>
                  <TimeDurationInput
                    durationSeconds={data.minSeconds ?? 0}
                    onChange={(v) => {
                      setData({ ...data, minSeconds: v });
                    }}
                  />
                  <h3>Maximum</h3>
                  <TimeDurationInput
                    durationSeconds={data.maxSeconds ?? Infinity}
                    onChange={(v) => {
                      setData({ ...data, maxSeconds: v });
                    }}
                  />
                  <div className="mt-2"></div>
                  <Button
                    size="sm"
                    variant="destructive"
                    disabled={
                      data.minSeconds === 0 && data.maxSeconds === Infinity
                    }
                    onClick={() => {
                      setData({ minSeconds: 0, maxSeconds: Infinity });
                    }}
                  >
                    Reset
                  </Button>
                </>
              );
            }}
          />
          <AlertHelper
            initialData={{
              numQualifiedObjectsConstraint:
                data.numQualifiedObjectsConstraint != null
                  ? { ...data.numQualifiedObjectsConstraint }
                  : {},
              currentlyEditing: { qualifier: "", min: "0", max: "∞" },
            }}
            trigger={
              <button
                title="Edit Number of Related Qualified Objects Constraints"
                className="flex items-center gap-x-2 px-1 py-0.5 rounded border border-blue-300/30 my-1 hover:bg-blue-300/20 text-xs font-light"
                onClick={() => {}}
              >
                <AiOutlineNumber
                  className={
                    data.numQualifiedObjectsConstraint === undefined
                      ? "text-gray-500/50"
                      : "text-blue-500"
                  }
                />
              </button>
            }
            title={"Edit Related Qualified Object Constraints"}
            submitAction={"Submit"}
            onSubmit={(qualifiedObjConstraintData, ev) => {
              const newDataFields =
                Object.keys(
                  qualifiedObjConstraintData.numQualifiedObjectsConstraint,
                ).length === 0
                  ? { numQualifiedObjectsConstraint: undefined }
                  : {
                      numQualifiedObjectsConstraint: {
                        ...qualifiedObjConstraintData.numQualifiedObjectsConstraint,
                      },
                    };
              data.onDataChange(id, {
                ...data,
                ...newDataFields,
              });
            }}
            content={({ data: d, setData: setD }) => {
              return (
                <>
                  <span className="mb-2 block">
                    Please select the qualifier and the minimal and maximal
                    number of objects associated throught that qualifier below.
                  </span>
                  <ul className="list-disc pl-4 text-base my-4">
                    {Object.entries(d.numQualifiedObjectsConstraint).map(
                      ([qualifier, numObjectsConstraint], i) => (
                        <li key={i} className="font-medium">
                          <span className="inline-block w-[calc(100%-2rem)]">
                            {qualifier}: {numObjectsConstraint.min} -{" "}
                            {numObjectsConstraint.max === Infinity
                              ? "∞"
                              : numObjectsConstraint.max}
                          </span>
                          <Button
                            size="icon"
                            title="Remove"
                            className="h-5 w-5 ml-2 text-red-500"
                            variant="outline"
                            onClick={() => {
                              const newD = {
                                ...d,
                                numObjectsConstraint: {
                                  ...d.numQualifiedObjectsConstraint,
                                  [qualifier]: undefined,
                                },
                              };
                              delete (
                                // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                                newD.numQualifiedObjectsConstraint[qualifier]
                              );
                              setD(newD);
                            }}
                          >
                            <LuDelete />
                          </Button>
                        </li>
                      ),
                    )}
                  </ul>
                  <div className="flex gap-x-1 items-end mt-4">
                    <Label className="w-[16rem] flex flex-col gap-y-1">
                      Qualifier
                      <Combobox
                        value={d.currentlyEditing.qualifier}
                        options={Object.keys(data.eventTypeQualifier)
                          .filter(
                            (q) => !(q in d.numQualifiedObjectsConstraint),
                          )
                          .map((q) => ({
                            value: q,
                            label: q,
                          }))}
                        name="Qualifier"
                        onChange={(val) => {
                          setD({
                            ...d,
                            currentlyEditing: {
                              ...d.currentlyEditing,
                              qualifier: val,
                            },
                          });
                        }}
                      />
                    </Label>
                    <Label className="flex flex-col gap-y-1">
                      Minimum
                      <Input
                        type="text"
                        value={d.currentlyEditing.min}
                        onChange={(ev) => {
                          setD({
                            ...d,
                            currentlyEditing: {
                              ...d.currentlyEditing,
                              min: ev.currentTarget.value,
                            },
                          });
                        }}
                      />
                    </Label>
                    <Label className="flex flex-col gap-y-1">
                      Maximum
                      <Input
                        type="text"
                        value={d.currentlyEditing.max}
                        onChange={(ev) => {
                          setD({
                            ...d,
                            currentlyEditing: {
                              ...d.currentlyEditing,
                              max:
                                ev.currentTarget.value === "∞" ||
                                ev.currentTarget.value === "infinity" ||
                                ev.currentTarget.value === "inf"
                                  ? "∞"
                                  : ev.currentTarget.value,
                            },
                          });
                        }}
                      />
                    </Label>
                    <Button
                      onClick={() => {
                        if (d.currentlyEditing.qualifier === "") {
                          toast("Please select a qualifier");
                          return;
                        }
                        const parsedMin = parseInt(d.currentlyEditing.min);
                        const parsedMax =
                          d.currentlyEditing.max === "∞" ||
                          d.currentlyEditing.max === "inf" ||
                          d.currentlyEditing.max === "infinity"
                            ? Infinity
                            : parseInt(d.currentlyEditing.max);
                        if (isNaN(parsedMin) || isNaN(parsedMax)) {
                          toast("Invalid number");
                          return;
                        }

                        setD({
                          ...d,
                          currentlyEditing: {
                            qualifier: "",
                            min: "0",
                            max: "∞",
                          },
                          numQualifiedObjectsConstraint: {
                            ...d.numQualifiedObjectsConstraint,
                            [d.currentlyEditing.qualifier]: {
                              min: parsedMin,
                              max: parsedMax,
                            },
                          },
                        });
                      }}
                    >
                      Add
                    </Button>
                  </div>
                </>
              );
            }}
          />
        </div>
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
