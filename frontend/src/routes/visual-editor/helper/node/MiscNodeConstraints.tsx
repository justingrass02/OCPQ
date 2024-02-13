import AlertHelper from "@/components/AlertHelper";
import TimeDurationInput, {
  formatSeconds,
} from "@/components/TimeDurationInput";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import toast from "react-hot-toast";
import { AiOutlineNumber } from "react-icons/ai";
import { CgRowLast, CgRowFirst, CgStopwatch } from "react-icons/cg";
import { LuDelete } from "react-icons/lu";
import { parseIntAllowInfinity } from "../infinity-input";
import type { EventTypeNodeData } from "../types";

interface MiscNodeConstraintsProps {
  id: string;
  data: EventTypeNodeData;
  onNodeDataChange: (
    id: string,
    newData: Partial<EventTypeNodeData> | undefined,
  ) => unknown;
}
export default function MiscNodeConstraints({
  data,
  onNodeDataChange,
  id,
}: MiscNodeConstraintsProps) {
  return (
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
            onNodeDataChange(id, {
              ...data,
              firstOrLastEventOfType: "first",
            });
          } else if (data.firstOrLastEventOfType === "first") {
            onNodeDataChange(id, {
              ...data,
              firstOrLastEventOfType: "last",
            });
          } else {
            onNodeDataChange(id, {
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
          onNodeDataChange(id, {
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
                disabled={data.minSeconds === 0 && data.maxSeconds === Infinity}
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
          onNodeDataChange(id, {
            ...data,
            ...newDataFields,
          });
        }}
        content={({ data: d, setData: setD }) => {
          return (
            <>
              <span className="mb-2 block">
                Please select the qualifier and the minimal and maximal number
                of objects associated throught that qualifier below.
              </span>
              <ul className="list-disc pl-4 text-base my-4">
                {Object.entries(d.numQualifiedObjectsConstraint).map(
                  ([qualifier, numObjectsConstraint], i) => (
                    <li key={i} className="font-medium">
                      <span className="inline-flex items-center gap-x-2 w-[calc(100%-2rem)]">
                        {qualifier}:
                        <Input
                          className="max-w-[5ch]"
                          type="text"
                          key={id + +"_" + i + "min" + numObjectsConstraint.min}
                          defaultValue={numObjectsConstraint.min}
                          onBlur={(ev) => {
                            const val = parseIntAllowInfinity(
                              ev.currentTarget.value,
                            );
                            if (val === undefined) {
                              return;
                            }
                            setD({
                              ...d,
                              numQualifiedObjectsConstraint: {
                                ...d.numQualifiedObjectsConstraint,
                                [qualifier]: {
                                  min: val,
                                  max: numObjectsConstraint.max,
                                },
                              },
                            });
                          }}
                        />
                        -
                        <Input
                          className="max-w-[5ch]"
                          type="text"
                          key={id + +"_" + i + "max" + numObjectsConstraint.max}
                          defaultValue={
                            numObjectsConstraint.max === Infinity
                              ? "∞"
                              : numObjectsConstraint.max.toString()
                          }
                          onBlur={(ev) => {
                            const val = parseIntAllowInfinity(
                              ev.currentTarget.value,
                            );
                            if (val === undefined) {
                              return;
                            }
                            setD({
                              ...d,
                              numQualifiedObjectsConstraint: {
                                ...d.numQualifiedObjectsConstraint,
                                [qualifier]: {
                                  min: numObjectsConstraint.min,
                                  max: val,
                                },
                              },
                            });
                          }}
                        />
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
              <div className="flex gap-x-1 items-end mt-4 mb-2">
                <Label className="w-[16rem] flex flex-col gap-y-1">
                  Qualifier
                  <Combobox
                    value={d.currentlyEditing.qualifier}
                    options={Object.keys(data.eventTypeQualifier)
                      .filter((q) => !(q in d.numQualifiedObjectsConstraint))
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
                  variant="outline"
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
  );
}
