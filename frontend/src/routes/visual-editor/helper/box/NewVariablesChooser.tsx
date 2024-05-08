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
import { Label } from "@/components/ui/label";
import MultiSelect from "@/components/ui/multi-select";
import type { BindingBox } from "@/types/generated/BindingBox";
import type { EventVariable } from "@/types/generated/EventVariable";
import type { ObjectVariable } from "@/types/generated/ObjectVariable";
import { useContext, useState } from "react";
import { LuPlus } from "react-icons/lu";
import { VisualEditorContext } from "../VisualEditorContext";
import { getEvVarName, getObVarName } from "./variable-names";

export default function NewVariableChooser({
  id,
  box,
  updateBox,
}: {
  id: string;
  box: BindingBox;
  updateBox: (box: BindingBox) => unknown;
}) {
  const { ocelInfo, getAvailableVars } = useContext(VisualEditorContext);
  const availableObjectVars = getAvailableVars(id, "object");
  const availableEventVars = getAvailableVars(id, "event");
  const [alertState, setAlertState] = useState<
    {
      variant: "event" | "object";
      key: ObjectVariable | EventVariable;
      value: string[];
    } & (
      | { mode: "add" }
      | { mode: "edit"; editKey: ObjectVariable | EventVariable }
    )
  >();

  function getAvailableObjVars(allowObjectVar?: ObjectVariable | undefined) {
    return Array(100)
      .fill(0)
      .map((_, i) => i)
      .filter((i) => i === allowObjectVar || !availableObjectVars.includes(i))
      .filter((i, index) => index < 10);
  }

  function getAvailableEvVars(allowedEventVar?: EventVariable | undefined) {
    return Array(100)
      .fill(0)
      .map((_, i) => i)
      .filter((i) => i === allowedEventVar || !availableEventVars.includes(i))
      .filter((i, index) => index < 10);
  }

  if (ocelInfo === undefined) {
    return <div>No OCEL Info available. Check backend and reload.</div>;
  }
  return (
    <div className="font-normal text-left w-full">
      <div className="flex items-center gap-x-1 -mb-0.5">
        <Label>Object Variables</Label>
        <Button
          size="icon"
          variant="ghost"
          className="h-4 w-4 hover:bg-blue-400/50 hover:border-blue-500/50 mt-1"
          onClick={() =>
            setAlertState({
              mode: "add",
              variant: "object",
              key: getAvailableObjVars()[0],
              value: [ocelInfo.object_types[0].name],
            })
          }
        >
          <LuPlus size={10} />
        </Button>
      </div>
      <ul className="w-full text-left text-sm min-h-[0.5rem]">
        {Object.entries(box.newObjectVars).map(([obVar, obTypes]) => (
          <li key={obVar} className="flex items-center gap-x-0.5">
            <button
              className="hover:bg-blue-200/50 px-0.5 rounded-sm"
              onClick={() =>
                setAlertState({
                  mode: "edit",
                  variant: "object",
                  key: parseInt(obVar),
                  editKey: parseInt(obVar),
                  value: obTypes,
                })
              }
            >
              {getObVarName(parseInt(obVar))}: {obTypes.join(", ")}
            </button>
          </li>
        ))}
      </ul>
      <div className="flex items-center gap-x-1 mt-1">
        <Label>Event Variables</Label>
        <Button
          size="icon"
          variant="ghost"
          className="h-4 w-4 hover:bg-blue-400/50 hover:border-blue-500/50 mt-1"
          onClick={() =>
            setAlertState({
              mode: "add",
              variant: "event",
              key: getAvailableEvVars()[0],
              value: [ocelInfo.event_types[0].name],
            })
          }
        >
          <LuPlus size={10} />
        </Button>
      </div>
      <ul className="w-full text-left text-sm min-h-[0.5rem]">
        {Object.entries(box.newEventVars).map(([evVar, evTypes]) => (
          <li key={evVar}>
            <button
              className="hover:bg-blue-200/50 px-0.5 rounded-sm"
              onClick={() =>
                setAlertState({
                  mode: "edit",
                  variant: "event",
                  key: parseInt(evVar),
                  editKey: parseInt(evVar),
                  value: evTypes,
                })
              }
            >
              {getEvVarName(parseInt(evVar))}: {evTypes.join(", ")}
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
          <AlertDialogContent className="max-w-2xl">
            <AlertDialogHeader>
              <AlertDialogTitle>
                {alertState?.mode === "add" ? "Add " : "Edit "}
                {alertState?.variant === "event"
                  ? "Event Variable"
                  : "Object Variable"}
              </AlertDialogTitle>
              <div className="text-sm text-gray-700 pt-4 grid grid-cols-[1fr,2fr] gap-x-2 gap-y-1.5">
                <Label>Variable</Label>
                <Label>
                  {alertState?.variant === "event" ? "Event" : "Object"} Types
                </Label>
                <Combobox
                  options={
                    alertState?.variant === "object"
                      ? getAvailableObjVars(
                          alertState.mode === "edit"
                            ? alertState.key
                            : undefined,
                        ).map((i) => ({
                          value: i.toString() + " --- " + getObVarName(i),
                          label: getObVarName(i),
                        }))
                      : getAvailableEvVars(
                          alertState.mode === "edit"
                            ? alertState.key
                            : undefined,
                        ).map((i) => ({
                          value: i.toString() + " --- " + getEvVarName(i),
                          label: getEvVarName(i),
                        }))
                  }
                  onChange={(value: string) => {
                    const variableKey = parseInt(value.split(" --- ")[0]);
                    if (!isNaN(variableKey) && variableKey >= 0) {
                      setAlertState({ ...alertState, key: variableKey });
                    }
                  }}
                  name={""}
                  value={
                    alertState.key.toString() +
                    " --- " +
                    (alertState?.variant === "object"
                      ? getObVarName(alertState.key)
                      : getEvVarName(alertState.key))
                  }
                />
                <MultiSelect
                  options={(alertState?.variant === "object"
                    ? ocelInfo.object_types
                    : ocelInfo.event_types
                  ).map((t) => ({
                    label: t.name,
                    value: t.name,
                  }))}
                  placeholder={""}
                  defaultValue={alertState.value}
                  onValueChange={(value: string[]) => {
                    setAlertState({ ...alertState, value });
                  }}
                />
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
                    if (alertState.variant === "object") {
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete newBox.newObjectVars[alertState.editKey];
                    } else {
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete newBox.newEventVars[alertState.editKey];
                    }
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
                  const newBox = { ...box };
                  if (alertState.mode === "edit") {
                    if (alertState.variant === "object") {
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete newBox.newObjectVars[alertState.editKey];
                    } else {
                      // eslint-disable-next-line @typescript-eslint/no-dynamic-delete
                      delete newBox.newEventVars[alertState.editKey];
                    }
                  }
                  if (alertState.variant === "object") {
                    newBox.newObjectVars[alertState.key] = alertState.value;
                  } else {
                    newBox.newEventVars[alertState.key] = alertState.value;
                  }
                  updateBox(newBox);
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
