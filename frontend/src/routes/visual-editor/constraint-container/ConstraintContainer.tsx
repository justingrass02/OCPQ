import AlertHelper from "@/components/AlertHelper";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import type {
  EventTypeQualifiers,
  ObjectTypeQualifiers,
  OCELInfo,
} from "@/types/ocel";
import { useContext, useState } from "react";
import toast from "react-hot-toast";
import { LuUnlink, LuX } from "react-icons/lu";
import { TbVariablePlus } from "react-icons/tb";
import { ReactFlowProvider } from "reactflow";
import { ConstraintInfoContext } from "../helper/ConstraintInfoContext";
import { FlowContext } from "../helper/FlowContext";
import type { ObjectVariable } from "../helper/types";
import VisualEditor from "../VisualEditor";
import { Input } from "@/components/ui/input";

interface ConstraintContainerProps {
  qualifiers: EventTypeQualifiers;
  objectQualifiers: ObjectTypeQualifiers;
  ocelInfo: OCELInfo;
}

export default function ConstraintContainer({
  qualifiers,
  objectQualifiers,
  ocelInfo,
}: ConstraintContainerProps) {
  const { otherData } = useContext(FlowContext);
  const [info, setInfo] = useState<{
    objectVariables: ObjectVariable[];
  }>({ objectVariables: otherData?.objectVariables ?? [] });
  const [editMetaInfoData, setEditMetaInfoData] = useState<ObjectVariable>({
    name: "",
    type: "",
    initiallyBound: true,
    o2o: undefined,
  });

  function getPossibleO2O(forObjectType: string) {
    const possibleO2O: { qualifier: string; parentVariableName: string }[] = [];
    for (const v of info.objectVariables) {
      const quals = objectQualifiers[v.type];
      if (quals != null) {
        for (const [q, t] of quals) {
          if (t === forObjectType) {
            possibleO2O.push({ qualifier: q, parentVariableName: v.name });
          }
        }
      }
    }
    return possibleO2O;
  }

  return (
    <div className="relative">
      <div>
        <div className="flex flex-wrap gap-x-2 absolute ml-1 mt-1 z-10">
          {info.objectVariables.map((m, i) => (
            <div
              className="text-center flex items-center px-1 bg-slate-100 rounded-md border cursor-help"
              key={i}
              title={
                "Type: " +
                m.type +
                (m.o2o != null
                  ? "\nO2O: " + m.o2o.qualifier + "@" + m.o2o.parentVariableName
                  : "\nO2O: -") +
                (m.initiallyBound ? "\nInitially bound" : "\nInitially unbound")
              }
            >
              <button
                title="Remove"
                className="cursor-pointer text-xs mr-1 my-0 rounded-full transition-colors hover:bg-red-50 hover:outline hover:outline-1 hover:outline-red-400 hover:text-red-400 focus:text-red-500"
                onClick={() => {
                  const newObjectVariables = [...info.objectVariables];
                  newObjectVariables.splice(i, 1);
                  setInfo({ ...info, objectVariables: newObjectVariables });
                }}
              >
                <LuX />
              </button>
              {m.name}
              {m.o2o != null && (
                <span className="pl-2 text-gray-600">
                  {m.o2o.qualifier}@{m.o2o.parentVariableName}
                </span>
              )}
              {!m.initiallyBound && <LuUnlink className="ml-2" />}
            </div>
          ))}
        </div>
      </div>
      <div className="w-[50rem] xl:w-[70rem] h-[50rem] border p-2">
        <ReactFlowProvider>
          <ConstraintInfoContext.Provider value={info}>
            {qualifiers !== undefined && ocelInfo !== undefined && (
              <>
                <VisualEditor
                  eventTypeQualifiers={qualifiers}
                  ocelInfo={ocelInfo}
                >
                  <AlertHelper
                    trigger={
                      <Button
                        variant="outline"
                        title="Add Object Variable"
                        className="bg-white"
                      >
                        <TbVariablePlus size={20} />
                      </Button>
                    }
                    title="Add Object Variable"
                    initialData={undefined}
                    content={() => (
                      <>
                        <p className="mb-4">
                          Select the object type and variable name below.
                        </p>
                        <div className="flex flex-wrap items-start gap-2">
                          <Combobox
                            value={editMetaInfoData.type}
                            options={ocelInfo.object_types.map((ot) => ({
                              value: ot.name,
                              label: ot.name,
                            }))}
                            name="Object type"
                            onChange={(valWithCorrectCaps) => {
                              // TODO: Seems like this is scheduled to also be fixed upstream?!
                              // https://github.com/pacocoursey/cmdk/commit/3dae25da8ca8448ea5b101a50f5d5987fe27679c
                              // https://github.com/pacocoursey/cmdk/issues/150
                              // const valWithCorrectCaps = ocelInfo.object_types.find(
                              //   (o) => o.name.toLowerCase() === val,
                              // )?.name;
                              console.log({ valWithCorrectCaps });
                              if (
                                valWithCorrectCaps == null ||
                                valWithCorrectCaps === ""
                              ) {
                                return;
                              }
                              console.log(
                                { valWithCorrectCaps },
                                ocelInfo.object_types,
                              );
                              if (
                                editMetaInfoData.name === "" ||
                                editMetaInfoData.name.match(
                                  new RegExp(
                                    editMetaInfoData.type
                                      .toLowerCase()
                                      .substring(0, 2) + "_[0-9]$",
                                  ),
                                ) != null
                              ) {
                                let name =
                                  valWithCorrectCaps
                                    .toLowerCase()
                                    .substring(0, 2) + "_";
                                for (let i = 0; i < 10; i++) {
                                  if (
                                    info.objectVariables.find(
                                      (v) => v.name === name + i,
                                    ) === undefined
                                  ) {
                                    name = name + i;
                                    break;
                                  }
                                }
                                const newEditMetaInfoData = JSON.parse(
                                  JSON.stringify(editMetaInfoData),
                                );
                                if (
                                  newEditMetaInfoData.o2o != null &&
                                  getPossibleO2O(valWithCorrectCaps).find(
                                    (va) =>
                                      va.parentVariableName ===
                                        newEditMetaInfoData.o2o!
                                          .parentVariableName &&
                                      va.qualifier ===
                                        newEditMetaInfoData.o2o!.qualifier,
                                  ) === undefined
                                ) {
                                  newEditMetaInfoData.o2o = undefined;
                                }
                                setEditMetaInfoData({
                                  ...newEditMetaInfoData,
                                  type: valWithCorrectCaps,
                                  name,
                                });
                              } else {
                                setEditMetaInfoData({
                                  ...editMetaInfoData,
                                  type: valWithCorrectCaps,
                                });
                              }
                            }}
                          />
                          <Input
                            value={editMetaInfoData.name}
                            onChange={(ev) => {
                              setEditMetaInfoData({
                                ...editMetaInfoData,
                                name: ev.currentTarget.value,
                              });
                            }}
                            className="max-w-[20ch]"
                            placeholder="Variable name"
                          />
                          <Combobox
                            value={
                              editMetaInfoData.initiallyBound
                                ? "Initially bound"
                                : "Initially unbound"
                            }
                            options={[
                              {
                                value: "Initially bound",
                                label: "Initially bound",
                              },
                              {
                                value: "Initially unbound",
                                label: "Initially unbound",
                              },
                            ]}
                            name="Initial Binding"
                            onChange={(val) => {
                              if (val != null && val !== "") {
                                const initiallyBound =
                                  val === "Initially bound";
                                setEditMetaInfoData({
                                  ...editMetaInfoData,
                                  initiallyBound,
                                  o2o: initiallyBound
                                    ? editMetaInfoData.o2o
                                    : undefined,
                                });
                              }
                            }}
                          />
                          <Combobox
                            disabled={
                              info.objectVariables.length === 0 ||
                              !editMetaInfoData.initiallyBound
                            }
                            value={
                              editMetaInfoData.o2o != null
                                ? JSON.stringify(editMetaInfoData.o2o)
                                : ""
                            }
                            options={[
                              ...getPossibleO2O(editMetaInfoData.type).map(
                                (p) => ({
                                  value: JSON.stringify(p),
                                  label:
                                    p.qualifier + "@" + p.parentVariableName,
                                }),
                              ),
                            ]}
                            name="O2O Binding"
                            onChange={(val) => {
                              if (val != null && val !== "" && val !== "-") {
                                const valJson = JSON.parse(val);
                                console.log({ valJson });
                                // const [qualifier, parentVariableName] = val.split("@");
                                setEditMetaInfoData({
                                  ...editMetaInfoData,
                                  o2o: valJson,
                                });
                              } else {
                                setEditMetaInfoData({
                                  ...editMetaInfoData,
                                  o2o: undefined,
                                });
                              }
                            }}
                          />
                        </div>
                      </>
                    )}
                    submitAction={"Add"}
                    onSubmit={(data, ev) => {
                      console.log({ editMetaInfoData });
                      if (
                        editMetaInfoData.name === "" ||
                        editMetaInfoData.type === ""
                      ) {
                        toast("Please input a name and object type");
                        ev.preventDefault();
                        return;
                      }
                      setInfo((cs) => {
                        return {
                          ...cs,
                          objectVariables: [
                            ...cs.objectVariables,
                            editMetaInfoData,
                          ],
                        };
                      });
                    }}
                  />
                </VisualEditor>
              </>
            )}
          </ConstraintInfoContext.Provider>
        </ReactFlowProvider>
      </div>
    </div>
  );
}
