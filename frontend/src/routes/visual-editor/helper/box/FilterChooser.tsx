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
import type { BindingBox } from "@/types/generated/BindingBox";
import type { Constraint } from "@/types/generated/Constraint";
import type { EventVariable } from "@/types/generated/EventVariable";
import type { Filter } from "@/types/generated/Filter";
import type { ObjectVariable } from "@/types/generated/ObjectVariable";
import type { SizeFilter } from "@/types/generated/SizeFilter";
import { useContext, useState } from "react";
import { LuPlus } from "react-icons/lu";
import { VisualEditorContext } from "../VisualEditorContext";
import FilterOrConstraintEditor, {
  FilterOrConstraintDisplay,
} from "./FilterOrConstraintEditor";
import { getEvVarName, getObVarName } from "./variable-names";

export default function FilterChooser({
  id,
  box,
  updateBox,
  type,
}: {
  id: string;
  box: BindingBox;
  updateBox: (box: BindingBox) => unknown;
  type: "filter" | "constraint";
}) {
  const { getAvailableVars, getAvailableChildNames } =
    useContext(VisualEditorContext);
  const availableObjectVars = getAvailableVars(id, "object");
  const availableEventVars = getAvailableVars(id, "event");
  const availableChildSets = getAvailableChildNames(id);
  const [alertState, setAlertState] = useState<
    (
      | {
          type: "filter";
          value?: Filter | SizeFilter | Constraint;
        }
      | { type: "sizeFilter"; value?: Filter | SizeFilter | Constraint }
      | { type: "constraint"; value?: Filter | SizeFilter | Constraint }
    ) &
      (
        | { mode: "add" }
        | { mode: "edit"; editIndex: number; wasSizeFilter: boolean }
      )
  >();

  return (
    <div className="w-full text-left border-t border-t-slate-700 mt-2 pt-2">
      <div className="flex items-center gap-x-1">
        <Label>{type === "filter" ? "Filters" : "Constraints"}</Label>
        <Button
          size="icon"
          variant="ghost"
          className="h-4 w-4 hover:bg-blue-400/50 hover:border-blue-500/50 mt-1"
          onClick={() => {
            setAlertState({ mode: "add", type });
          }}
        >
          <LuPlus size={10} />
        </Button>
      </div>
      <ul className="w-full">
        {type === "filter" &&
          box.filters.map((fc, i) => (
            <li key={i}>
              <button
                className="hover:bg-blue-200/50 px-0.5 rounded-sm text-left w-fit max-w-full"
                onClick={() => {
                  setAlertState({
                    editIndex: i,
                    mode: "edit",
                    type: "filter",
                    value: JSON.parse(JSON.stringify(fc)),
                    wasSizeFilter: false,
                  });
                }}
              >
                <FilterOrConstraintDisplay value={fc} />
              </button>
            </li>
          ))}
        {type === "filter" &&
          box.sizeFilters.map((sf, i) => (
            <li key={"sizeFilters" + i}>
              <button
                className="hover:bg-blue-200/50 px-0.5 rounded-sm text-left w-fit max-w-full"
                onClick={() => {
                  setAlertState({
                    editIndex: i,
                    mode: "edit",
                    type: "filter",
                    value: JSON.parse(JSON.stringify(sf)),
                    wasSizeFilter: true,
                  });
                }}
              >
                <FilterOrConstraintDisplay value={sf} />
              </button>
            </li>
          ))}
        {type === "constraint" &&
          box.constraints.map((c, i) => (
            <li key={"constraints" + i}>
              <button
                className="hover:bg-blue-200/50 px-0.5 rounded-sm text-left  w-fit max-w-full"
                onClick={() => {
                  setAlertState({
                    editIndex: i,
                    mode: "edit",
                    type: "constraint",
                    value: JSON.parse(JSON.stringify(c)),
                    wasSizeFilter: false,
                  });
                }}
              >
                <FilterOrConstraintDisplay value={c} />
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
          <AlertDialogContent className="max-w-3xl">
            <AlertDialogHeader>
              <AlertDialogTitle>
                {alertState?.mode === "add" ? "Add " : "Edit "}{" "}
                {alertState.type !== "constraint" ? "Filter" : "Constraint"}
              </AlertDialogTitle>
              <div className="text-sm text-gray-700 grid grid-cols-1 gap-y-1.5">
                <Label>Type</Label>
                <Combobox
                  name="Type"
                  value={
                    alertState.value !== undefined
                      ? alertState.value.type === "Filter"
                        ? alertState.value.filter.type
                        : alertState.value.type === "SizeFilter"
                        ? alertState.value.filter.type
                        : alertState.value.type
                      : ""
                  }
                  options={[
                    {
                      label: "Object associated with Event",
                      value: "O2E",
                    },
                    {
                      label: "Object associated with Object",
                      value: "O2O",
                    },
                    {
                      label: "Time between Events",
                      value: "TimeBetweenEvents",
                    },
                    {
                      label: "Event Attribute Value",
                      value: "EventAttributeValueFilter",
                    },
                    {
                      label: "Object Attribute Value",
                      value: "ObjectAttributeValueFilter",
                    },
                    ...(alertState.type !== "filter" ||
                    alertState.mode !== "edit" ||
                    alertState.wasSizeFilter
                      ? [
                          {
                            label: "Number of Child Bindings",
                            value: "NumChilds",
                          },
                          {
                            label: "Child Binding Sets Equal",
                            value: "BindingSetEqual",
                          },
                          {
                            label: "Child Binding Set Projects Equal",
                            value: "BindingSetProjectionEqual",
                          },
                        ]
                      : []),

                    ...(alertState.type === "constraint"
                      ? [
                          { label: "Child Satisfied (SAT)", value: "SAT" },
                          { label: "Child not Satisfied (NOT)", value: "NOT" },
                          { label: "Logic AND", value: "AND" },
                          { label: "Logic OR", value: "OR" },
                        ]
                      : []),
                  ]}
                  onChange={(val) => {
                    if (val === "O2E") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "O2E",
                          object: 0,
                          event: 0,
                          qualifier: null,
                        },
                      });
                    } else if (val === "O2O") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "O2O",
                          object: 0,
                          other_object: 1,
                          qualifier: null,
                        },
                      });
                    } else if (val === "TimeBetweenEvents") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "TimeBetweenEvents",
                          from_event: 0,
                          to_event: 1,
                          min_seconds: null,
                          max_seconds: null,
                        },
                      });
                    } else if (val === "NumChilds") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "NumChilds",
                          child_name: "A",
                          min: null,
                          max: null,
                        },
                      });
                    } else if (val === "BindingSetEqual") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "BindingSetEqual",
                          child_names: ["A"],
                        },
                      });
                    } else if (val === "BindingSetProjectionEqual") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "BindingSetProjectionEqual",
                          child_name_with_var_name: [["A", { Object: 0 }]],
                        },
                      });
                    } else if (
                      alertState.type === "constraint" &&
                      val === "SAT"
                    ) {
                      setAlertState({
                        ...alertState,
                        value: { type: "SAT", child_names: ["A"] },
                      });
                    } else if (
                      alertState.type === "constraint" &&
                      val === "NOT"
                    ) {
                      setAlertState({
                        ...alertState,
                        value: { type: "NOT", child_names: ["A"] },
                      });
                    } else if (
                      alertState.type === "constraint" &&
                      val === "AND"
                    ) {
                      setAlertState({
                        ...alertState,
                        value: { type: "AND", child_names: ["A", "B"] },
                      });
                    } else if (
                      alertState.type === "constraint" &&
                      val === "OR"
                    ) {
                      setAlertState({
                        ...alertState,
                        value: { type: "OR", child_names: ["A", "B"] },
                      });
                    } else if (val === "EventAttributeValueFilter") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "EventAttributeValueFilter",
                          event: 0,
                          attribute_name: "",
                          value_filter: { type: "String", is_in: [""] },
                        },
                      });
                    } else if (val === "ObjectAttributeValueFilter") {
                      setAlertState({
                        ...alertState,
                        value: {
                          type: "ObjectAttributeValueFilter",
                          object: 0,
                          attribute_name: "",
                          value_filter: { type: "String", is_in: [""] },
                          at_time: { type: "Sometime" },
                        },
                      });
                    }
                  }}
                />
              </div>
            </AlertDialogHeader>
            {alertState.value !== undefined && (
              <div className="flex gap-x-2">
                <FilterOrConstraintEditor
                  value={alertState.value}
                  updateValue={(val) => {
                    setAlertState({ ...alertState, value: val as any });
                  }}
                  availableEventVars={availableEventVars}
                  availableObjectVars={availableObjectVars}
                  availableChildSets={availableChildSets}
                  nodeID={id}
                />
              </div>
            )}
            <AlertDialogFooter>
              {" "}
              {alertState.mode === "edit" && (
                <Button
                  className="mr-auto"
                  variant="destructive"
                  onClick={() => {
                    const newBox = { ...box };
                    if (alertState.type === "filter") {
                      if (alertState.wasSizeFilter) {
                        newBox.sizeFilters.splice(alertState.editIndex, 1);
                      } else {
                        newBox.filters.splice(alertState.editIndex, 1);
                      }
                    } else {
                      newBox.constraints.splice(alertState.editIndex, 1);
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
                  if (alertState.value !== undefined) {
                    if (
                      alertState.type !== "constraint" &&
                      [
                        "NumChilds",
                        "BindingSetEqual",
                        "BindingSetProjectionEqual",
                      ].includes(alertState.value.type)
                    ) {
                      alertState.type = "sizeFilter";
                    }
                    const index =
                      alertState.mode === "edit"
                        ? alertState.editIndex
                        : (alertState.type === "filter"
                            ? newBox.filters
                            : alertState.type === "sizeFilter"
                            ? newBox.sizeFilters
                            : newBox.constraints
                          ).length;
                    console.log({ newBox, index, alertState }, alertState.type);
                    if (alertState.type === "filter") {
                      newBox.filters[index] = alertState.value as Filter;
                    } else if (alertState.type === "sizeFilter") {
                      newBox.sizeFilters[index] =
                        alertState.value as SizeFilter;
                    } else if (alertState.type === "constraint") {
                      if (
                        [
                          "NumChilds",
                          "BindingSetEqual",
                          "BindingSetProjectionEqual",
                        ].includes(alertState.value.type)
                      ) {
                        newBox.constraints[index] = {
                          type: "SizeFilter",
                          filter: alertState.value as SizeFilter,
                        };
                      } else if (
                        [
                          "SAT",
                          "NOT",
                          "AND",
                          "OR",
                          "Filter",
                          "SizeFilter",
                        ].includes(alertState.value.type)
                      ) {
                        newBox.constraints[index] =
                          alertState.value as Constraint;
                      } else if (
                        ["O2E", "O2O", "TimeBetweenEvents","ObjectAttributeValueFilter","EventAttributeValueFilter"].includes(
                          alertState.value.type,
                        )
                      ) {
                        newBox.constraints[index] = {
                          type: "Filter",
                          filter: alertState.value as Filter,
                        };
                      } else {
                        newBox.constraints[index] =
                          alertState.value as Constraint;
                      }
                    }
                  }
                  updateBox(newBox);
                  setAlertState(undefined);

                  // if (alertState.mode === "edit") {
                  //   if (alertState.value !== undefined) {
                  //     if (alertState.type === "filter") {
                  //       if ("NumChilds" in alertState.f) {
                  //         newBox.sizeFilters[alertState.editIndex] =
                  //           alertState.f;
                  //       } else {
                  //         newBox.filters[alertState.editIndex] = alertState.f;
                  //       }
                  //     } else {
                  //       if ("NumChilds" in alertState.f) {
                  //         newBox.constraints[alertState.editIndex] = {
                  //           SizeFilter: alertState.f,
                  //         };
                  //       } else {
                  //         newBox.constraints[alertState.editIndex] = {
                  //           Filter: alertState.f,
                  //         };
                  //       }
                  //     }
                  //   } else if (
                  //     alertState.type === "constraint" &&
                  //     alertState.x !== undefined
                  //   ) {
                  //     newBox.constraints[alertState.editIndex] = alertState.x;
                  //   }
                  // } else {
                  //   if (alertState.f !== undefined) {
                  //     if (alertState.type === "filter") {
                  //       if ("NumChilds" in alertState.f) {
                  //         newBox.sizeFilters.push(alertState.f);
                  //       } else {
                  //         newBox.filters.push(alertState.f);
                  //       }
                  //     } else {
                  //       if ("NumChilds" in alertState.f) {
                  //         newBox.constraints.push({ SizeFilter: alertState.f });
                  //       } else {
                  //         newBox.constraints.push({ Filter: alertState.f });
                  //       }
                  //     }
                  //   } else if (
                  //     alertState.type === "constraint" &&
                  //     alertState.x !== undefined
                  //   ) {
                  //     newBox.constraints.push(alertState.x);
                  //   }
                  // };
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

export function ObjectOrEventVarSelector({
  objectVars,
  eventVars,
  value,
  onChange,
}: {
  objectVars: ObjectVariable[];
  eventVars: EventVariable[];
  value:
    | { type: "object"; value: ObjectVariable }
    | { type: "event"; value: EventVariable }
    | undefined;
  onChange: (
    value:
      | { type: "object"; value: ObjectVariable }
      | { type: "event"; value: EventVariable }
      | undefined,
  ) => unknown;
}) {
  const { getVarName } = useContext(VisualEditorContext);
  return (
    <Combobox
      options={[
        ...objectVars.map((v) => ({
          label: getObVarName(v),
          value: `${v} --- object --- ${getVarName(v, "object").name}`,
        })),
        ...eventVars.map((v) => ({
          label: getEvVarName(v),
          value: `${v} --- event --- ${getVarName(v, "event").name}`,
        })),
      ]}
      onChange={(val) => {
        const [newVarString, type] = val.split(" --- ");
        const newVar = parseInt(newVarString);
        if (!isNaN(newVar)) {
          onChange({ type: type as "object" | "event", value: newVar });
        } else {
          onChange(undefined);
        }
      }}
      name={"Object/Event Variable"}
      value={
        value !== undefined
          ? `${value.value} --- ${value.type} --- ${
              getVarName(value.value, value.type).name
            }`
          : ""
      }
    />
  );
}

export function ObjectVarSelector({
  objectVars,
  value,
  onChange,
}: {
  objectVars: ObjectVariable[];
  value: ObjectVariable | undefined;
  onChange: (value: ObjectVariable | undefined) => unknown;
}) {
  const { getVarName } = useContext(VisualEditorContext);
  return (
    <Combobox
      options={objectVars.map((v) => ({
        label: getObVarName(v),
        value: `${v} --- ${getVarName(v, "object").name}`,
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
      value={`${value} --- ${
        value !== undefined ? getVarName(value, "object").name : ""
      }`}
    />
  );
}

export function EventVarSelector({
  eventVars,
  value,
  onChange,
}: {
  eventVars: EventVariable[];
  value: EventVariable | undefined;
  onChange: (value: EventVariable | undefined) => unknown;
}) {
  const { getVarName } = useContext(VisualEditorContext);
  return (
    <Combobox
      options={eventVars.map((v) => ({
        label: getEvVarName(v),
        value: `${v} --- ${getVarName(v, "event").name}`,
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
      value={`${value} --- ${
        value !== undefined ? getVarName(value, "event").name : ""
      }`}
    />
  );
}
