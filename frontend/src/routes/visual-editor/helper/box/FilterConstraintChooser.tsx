import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import type { BindingBox } from "@/types/generated/BindingBox";
import type { EventVariable } from "@/types/generated/EventVariable";
import type { FilterConstraint } from "@/types/generated/FilterConstraint";
import type { ObjectVariable } from "@/types/generated/ObjectVariable";
import { useState } from "react";
import { LuArrowRight, LuLink, LuPlus } from "react-icons/lu";
import { getEvVarName, getObVarName } from "./variable-names";

export default function FilterConstraintChooser({
  box,
  updateBox,
}: {
  box: BindingBox;
  updateBox: (box: BindingBox) => unknown;
}) {
  const [alertState, setAlertState] = useState<
    {
      fc?: FilterConstraint;
    } & ({ mode: "add" } | { mode: "edit"; editIndex: number })
  >();

  return (
    <div className="w-full text-left border-t border-t-blue-400 mt-2 pt-2">
      <div className="flex items-center gap-x-1">
        <Label>Filter Constraints</Label>
        <Button
          size="icon"
          variant="ghost"
          className="h-4 w-4 hover:bg-blue-400/50 hover:border-blue-500/50 mt-1"
          onClick={() => {
            setAlertState({ mode: "add", fc: undefined });
          }}
        >
          <LuPlus size={10} />
        </Button>
      </div>
      <ul>
        {box.filterConstraint.map((fc, i) => (
          <li key={i}>
            <button
              className="hover:bg-blue-200/50 px-0.5 rounded-sm"
              onClick={() => {
                setAlertState({
                  editIndex: i,
                  mode: "edit",
                  fc: JSON.parse(JSON.stringify(fc)),
                });
              }}
            >
              <FilterConstraintDisplay fc={fc} />
            </button>
          </li>
        ))}
      </ul>
      <AlertDialog
        open={alertState !== undefined}
        onOpenChange={(o) => {
          if (!o) {
            setAlertState(undefined);
          }
        }}
      >
        {alertState !== undefined && (
          <AlertDialogContent className="max-w-xl">
            <AlertDialogHeader>
              <AlertDialogTitle>
                {alertState?.mode === "add" ? "Add " : "Edit "} Filter
                Constraint
              </AlertDialogTitle>
              <div className="text-sm text-gray-700 grid grid-cols-1 gap-y-1.5">
                <Label>Type</Label>
                <Combobox
                  name="Type"
                  value={
                    alertState.fc === undefined
                      ? ""
                      : "ObjectAssociatedWithEvent" in alertState.fc
                      ? "ObjectAssociatedWithEvent"
                      : "ObjectAssociatedWithObject" in alertState.fc
                      ? "ObjectAssociatedWithObject"
                      : "TimeBetweenEvents" in alertState.fc
                      ? "TimeBetweenEvents"
                      : ""
                  }
                  options={[
                    {
                      label: "Object associated with Event",
                      value: "ObjectAssociatedWithEvent",
                    },
                    {
                      label: "Object associated with Object",
                      value: "ObjectAssociatedWithObject",
                    },
                    {
                      label: "Time between Events",
                      value: "TimeBetweenEvents",
                    },
                  ]}
                  onChange={(val) => {
                    if (val === "ObjectAssociatedWithEvent") {
                      setAlertState({
                        ...alertState,
                        fc: { ObjectAssociatedWithEvent: [0, 0, null] },
                      });
                    } else if (val === "ObjectAssociatedWithObject") {
                      setAlertState({
                        ...alertState,
                        fc: { ObjectAssociatedWithObject: [0, 1, null] },
                      });
                    } else if (val === "TimeBetweenEvents") {
                      setAlertState({
                        ...alertState,
                        fc: { TimeBetweenEvents: [0, 1, [null, null]] },
                      });
                    }
                  }}
                />
                {alertState.fc !== undefined && (
                  <>
                    {"ObjectAssociatedWithEvent" in alertState.fc && (
                      <div className="flex gap-x-2">
                        <ObjectVarSelector
                          objectVars={Object.keys(box.newObjectVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.ObjectAssociatedWithEvent[0]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "ObjectAssociatedWithEvent" in alertState.fc!
                            ) {
                              alertState.fc.ObjectAssociatedWithEvent[0] = newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <EventVarSelector
                          eventVars={Object.keys(box.newEventVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.ObjectAssociatedWithEvent[1]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "ObjectAssociatedWithEvent" in alertState.fc!
                            ) {
                              alertState.fc.ObjectAssociatedWithEvent[1] = newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <Input
                          className="w-full"
                          placeholder="Qualifier"
                          value={
                            alertState.fc.ObjectAssociatedWithEvent[2] ?? ""
                          }
                          onChange={(ev) => {
                            const newVal = ev.currentTarget.value;
                            if ("ObjectAssociatedWithEvent" in alertState.fc!) {
                              if (newVal !== null && newVal !== "") {
                                alertState.fc.ObjectAssociatedWithEvent[2] =
                                  newVal;
                                setAlertState({ ...alertState });
                              } else {
                                alertState.fc.ObjectAssociatedWithEvent[2] =
                                  null;
                                setAlertState({ ...alertState });
                              }
                            }
                          }}
                        />
                      </div>
                    )}

                    {"ObjectAssociatedWithObject" in alertState.fc && (
                      <div className="flex gap-x-2">
                        <ObjectVarSelector
                          objectVars={Object.keys(box.newObjectVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.ObjectAssociatedWithObject[0]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "ObjectAssociatedWithObject" in alertState.fc!
                            ) {
                              alertState.fc.ObjectAssociatedWithObject[0] =
                                newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <ObjectVarSelector
                          objectVars={Object.keys(box.newObjectVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.ObjectAssociatedWithObject[1]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "ObjectAssociatedWithObject" in alertState.fc!
                            ) {
                              alertState.fc.ObjectAssociatedWithObject[1] =
                                newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <Input
                          className="w-full"
                          placeholder="Qualifier"
                          value={
                            alertState.fc.ObjectAssociatedWithObject[2] ?? ""
                          }
                          onChange={(ev) => {
                            const newVal = ev.currentTarget.value;
                            if (
                              "ObjectAssociatedWithObject" in alertState.fc!
                            ) {
                              if (newVal !== null && newVal !== "") {
                                alertState.fc.ObjectAssociatedWithObject[2] =
                                  newVal;
                                setAlertState({ ...alertState });
                              } else {
                                alertState.fc.ObjectAssociatedWithObject[2] =
                                  null;
                                setAlertState({ ...alertState });
                              }
                            }
                          }}
                        />
                      </div>
                    )}

                    {"TimeBetweenEvents" in alertState.fc && (
                      <div className="flex gap-x-2">
                        <EventVarSelector
                          eventVars={Object.keys(box.newEventVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.TimeBetweenEvents[0]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "TimeBetweenEvents" in alertState.fc!
                            ) {
                              alertState.fc.TimeBetweenEvents[0] = newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <EventVarSelector
                          eventVars={Object.keys(box.newEventVars).map((v) =>
                            parseInt(v),
                          )}
                          value={alertState.fc.TimeBetweenEvents[1]}
                          onChange={(newV) => {
                            if (
                              newV !== undefined &&
                              "TimeBetweenEvents" in alertState.fc!
                            ) {
                              alertState.fc.TimeBetweenEvents[1] = newV;
                              setAlertState({ ...alertState });
                            }
                          }}
                        />
                        <Input
                          type="number"
                          className="w-full"
                          placeholder="Min. Delay (s)"
                          value={alertState.fc.TimeBetweenEvents[2][0] ?? ""}
                          onChange={(ev) => {
                            const newVal = ev.currentTarget.valueAsNumber;
                            if ("TimeBetweenEvents" in alertState.fc!) {
                              if (newVal !== null && !isNaN(newVal)) {
                                alertState.fc.TimeBetweenEvents[2][0] = newVal;
                                setAlertState({ ...alertState });
                              } else {
                                alertState.fc.TimeBetweenEvents[2][0] = null;
                                setAlertState({ ...alertState });
                              }
                            }
                          }}
                        />

                        <Input
                          type="number"
                          className="w-full"
                          placeholder="Max. Delay (s)"
                          value={alertState.fc.TimeBetweenEvents[2][1] ?? ""}
                          onChange={(ev) => {
                            const newVal = ev.currentTarget.valueAsNumber;
                            if ("TimeBetweenEvents" in alertState.fc!) {
                              if (newVal !== null && !isNaN(newVal)) {
                                alertState.fc.TimeBetweenEvents[2][1] = newVal;
                                setAlertState({ ...alertState });
                              } else {
                                alertState.fc.TimeBetweenEvents[2][1] = null;
                                setAlertState({ ...alertState });
                              }
                            }
                          }}
                        />
                      </div>
                    )}
                  </>
                )}
              </div>
            </AlertDialogHeader>
            <AlertDialogFooter>
              {" "}
              {alertState.mode === "edit" && (
                <Button
                  className="mr-auto"
                  variant="destructive"
                  onClick={() => {
                    const newBox = { ...box };
                    newBox.filterConstraint.splice(alertState.editIndex, 1);
                    updateBox(newBox);
                    setAlertState(undefined);
                  }}
                >
                  Delete
                </Button>
              )}
              <AlertDialogCancel>Cancel</AlertDialogCancel>
              <AlertDialogAction
                onClick={() => {
                  if (alertState.fc !== undefined) {
                    const newBox = { ...box };
                    if (alertState.mode === "edit") {
                      newBox.filterConstraint[alertState.editIndex] =
                        alertState.fc;
                    } else {
                      newBox.filterConstraint.push(alertState.fc);
                    }
                    updateBox(newBox);
                  }
                  setAlertState(undefined);
                }}
              >
                {alertState.mode === "add" ? "Add" : "Save"}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        )}
      </AlertDialog>
    </div>
  );
}

function FilterConstraintDisplay({ fc }: { fc: FilterConstraint }) {
  if ("ObjectAssociatedWithEvent" in fc) {
    const [obVar, evVar, qualifier] = fc.ObjectAssociatedWithEvent;
    return (
      <div className="flex items-center gap-x-1 font-normal text-sm">
        {getObVarName(obVar)} <LuLink /> {getEvVarName(evVar)}{" "}
        {qualifier != null ? `@${qualifier}` : ""}
      </div>
    );
  }

  if ("ObjectAssociatedWithObject" in fc) {
    const [obVar1, obVar2, qualifier] = fc.ObjectAssociatedWithObject;
    return (
      <div className="flex items-center gap-x-1 font-normal text-sm">
        {getObVarName(obVar1)} <LuLink /> {getObVarName(obVar2)}{" "}
        {qualifier != null ? `@${qualifier}` : ""}
      </div>
    );
  }

  if ("TimeBetweenEvents" in fc) {
    const [evVar1, evVar2, [minTime, maxTime]] = fc.TimeBetweenEvents;
    return (
      <div className="flex items-center gap-x-1 font-normal text-sm">
        {getEvVarName(evVar1)} <LuArrowRight /> {getEvVarName(evVar2)}{" "}
        <span className="ml-2 inline-flex items-center gap-x-1 text-xs">
          {minTime ?? "-∞"} <span className="mx-1">-</span> {maxTime ?? "∞"} (s)
        </span>
      </div>
    );
  }

  return <div>Unknown Filter Constraint</div>;
}

function ObjectVarSelector({
  objectVars,
  value,
  onChange,
}: {
  objectVars: ObjectVariable[];
  value: ObjectVariable | undefined;
  onChange: (value: ObjectVariable | undefined) => unknown;
}) {
  return (
    <Combobox
      options={objectVars.map((v) => ({
        label: getObVarName(v),
        value: `${v} --- ${getObVarName(v)}`,
      }))}
      onChange={(val) => {
        const newVar = parseInt(val.split(" --- ")[0]);
        if (!isNaN(newVar)) {
          onChange(newVar);
        } else {
          onChange(undefined);
        }
      }}
      name={"Object Variable"}
      value={`${value} --- ${value !== undefined ? getObVarName(value) : ""}`}
    />
  );
}

function EventVarSelector({
  eventVars,
  value,
  onChange,
}: {
  eventVars: EventVariable[];
  value: EventVariable | undefined;
  onChange: (value: EventVariable | undefined) => unknown;
}) {
  return (
    <Combobox
      options={eventVars.map((v) => ({
        label: getEvVarName(v),
        value: `${v} --- ${getEvVarName(v)}`,
      }))}
      onChange={(val) => {
        const newVar = parseInt(val.split(" --- ")[0]);
        if (!isNaN(newVar)) {
          onChange(newVar);
        } else {
          onChange(undefined);
        }
      }}
      name={"Event Variable"}
      value={`${value} --- ${value !== undefined ? getEvVarName(value) : ""}`}
    />
  );
}
