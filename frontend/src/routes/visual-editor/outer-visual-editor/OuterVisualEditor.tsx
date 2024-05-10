import { OcelInfoContext } from "@/App";
import { BackendProviderContext } from "@/BackendProviderContext";
import AlertHelper from "@/components/AlertHelper";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { EventTypeQualifiers, ObjectTypeQualifiers } from "@/types/ocel";
import clsx from "clsx";
import json5 from "json5";
import { Fragment, useContext, useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import { CgTrash } from "react-icons/cg";
import { LuSave } from "react-icons/lu";
import { RxCrossCircled, RxPlusCircled } from "react-icons/rx";
import { type ReactFlowInstance } from "reactflow";
import ConstraintContainer from "../constraint-container/ConstraintContainer";
import { FlowContext } from "../helper/FlowContext";

import type { FlowAndViolationData } from "@/types/misc";
import type { ConstraintInfo, EvaluationResPerNodes } from "../helper/types";
import AutoDiscoveryButton from "./AutoDiscovery";
const LOCALSTORAGE_SAVE_KEY_DATA = "oced-declare-data";
const LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META = "oced-declare-meta";

function parse(s: string) {
  return json5.parse(s);
}

export default function VisualEditorOuter() {
  const [qualifiers, setQualifiers] = useState<EventTypeQualifiers>({});
  const [objectQualifiers, setObjectQualifiers] =
    useState<ObjectTypeQualifiers>({});
  const ocelInfo = useContext(OcelInfoContext);
  const [constraints, setConstraints] = useState<ConstraintInfo[]>([]);
  const [currentInstanceAndData, setCurrentInstanceAndData] = useState<{
    instance?: ReactFlowInstance | undefined;
    getter?: () =>
      | {
          violations?: EvaluationResPerNodes;
        }
      | undefined;
  }>({});
  const backend = useContext(BackendProviderContext);
  useEffect(() => {
    toast
      .promise(
        backend["ocel/event-qualifiers"](),
        {
          loading: "Fetching qualifier info...",
          success: "Loaded qualifier info",
          error: "Failed to fetch qualifier info",
        },
        { id: "fetch-event-qualifiers" },
      )
      .then(async (res) => {
        setQualifiers(res);
      })
      .catch((e) => {
        console.error(e);
      });
  }, []);

  useEffect(() => {
    toast
      .promise(
        backend["ocel/object-qualifiers"](),
        {
          loading: "Fetching object qualifier info...",
          success: "Loaded object qualifier info",
          error: "Failed to fetch object qualifier info",
        },
        { id: "fetch-object-qualifiers" },
      )
      .then((res) => {
        setObjectQualifiers(res);
      })
      .catch((e) => {
        console.error(e);
      });
  }, []);

  const prevDataRef = useRef<FlowAndViolationData[]>([]);
  const [activeIndex, setActiveIndex] = useState<number>();
  useEffect(() => {
    const meta = parse(
      localStorage.getItem(LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META) ?? "[]",
    );
    const data = parse(
      localStorage.getItem(LOCALSTORAGE_SAVE_KEY_DATA) ?? "[]",
    );

    prevDataRef.current = data;
    setConstraints(meta);
  }, []);
  function saveData() {
    if (
      currentInstanceAndData.instance !== undefined &&
      activeIndex !== undefined &&
      currentInstanceAndData.getter !== undefined
    ) {
      // First, save current data
      const prevOtherData = currentInstanceAndData.getter();
      prevDataRef.current[activeIndex] = {
        flowJson: currentInstanceAndData.instance.toObject(),
        violations: prevOtherData?.violations,
      };
    }

    if (prevDataRef.current !== undefined) {
      console.log(json5.stringify(prevDataRef.current));
      localStorage.setItem(
        LOCALSTORAGE_SAVE_KEY_DATA,
        json5.stringify(
          prevDataRef.current.map((x) => ({ ...x, violations: undefined })),
        ),
      );
    }
    localStorage.setItem(
      LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META,
      json5.stringify(constraints),
    );
  }

  function changeIndex(newIndex: number, length = constraints.length) {
    if (!isNaN(newIndex) && newIndex >= 0 && newIndex < length) {
      if (
        currentInstanceAndData.instance !== undefined &&
        activeIndex !== undefined &&
        currentInstanceAndData.getter !== undefined
      ) {
        const dataFromPrevIndex = currentInstanceAndData.instance.toObject();
        const prevOtherData = currentInstanceAndData.getter();
        prevDataRef.current[activeIndex] = {
          flowJson: dataFromPrevIndex,
          violations: prevOtherData?.violations,
        };
      }
      setActiveIndex(newIndex);
    }
  }
  return (
    <div className="w-full h-full">
      <FlowContext.Provider
        value={{
          flushData: (data) => {
            if (
              data !== undefined &&
              activeIndex !== undefined &&
              currentInstanceAndData.instance !== undefined
            ) {
              prevDataRef.current[activeIndex] = {
                flowJson: currentInstanceAndData.instance.toObject(),
                violations: data?.violations,
              };
              setConstraints([...constraints]);
            }
          },
          instance: currentInstanceAndData?.instance,
          setInstance: (i) => {
            setCurrentInstanceAndData((ci) => ({
              ...ci,
              instance: i,
            }));
          },
          registerOtherDataGetter: (getter) => {
            setCurrentInstanceAndData((ci) => ({ ...ci, getter }));
          },
          otherData:
            activeIndex !== undefined
              ? {
                  violations: prevDataRef.current[activeIndex]?.violations,
                  nodes: prevDataRef.current[activeIndex]?.flowJson?.nodes,
                  edges: prevDataRef.current[activeIndex]?.flowJson?.edges,
                  viewport:
                    prevDataRef.current[activeIndex]?.flowJson?.viewport,
                }
              : undefined,
        }}
      >
        <div className="flex flex-col justify-start items-center mb-2 gap-y-2 h-full">
          {ocelInfo !== undefined &&
            qualifiers !== undefined &&
            objectQualifiers !== undefined && (
              <>
                <div
                  className={`w-full max-w-4xl gap-y-2 ${
                    constraints.length > 0
                      ? "justify-between"
                      : "justify-center"
                  }`}
                >
                  <div className="flex justify-center gap-x-2 items-start mb-2">
                    <AlertHelper
                      trigger={
                        <Button
                          title={"Delete All"}
                          variant="destructive"
                          size="icon"
                          className="h-12 w-12"
                          disabled={constraints.length === 0}
                        >
                          <CgTrash size={"24"} />
                        </Button>
                      }
                      title={"Delete All Constraints"}
                      initialData={undefined}
                      content={() => (
                        <p>Are you sure? This will delete all constraints.</p>
                      )}
                      submitAction={"Delete All"}
                      onSubmit={() => {
                        prevDataRef.current = [];
                        setConstraints([]);
                      }}
                    />
                    <div className="flex flex-col justify-center gap-y-2">
                      <Button
                        className="text-xl py-6 px-4"
                        onClick={() => {
                          prevDataRef.current.splice(constraints.length, 1);
                          changeIndex(
                            constraints.length,
                            constraints.length + 1,
                          );
                          setConstraints((cs) => [
                            ...cs,
                            { name: "", description: "" },
                          ]);
                        }}
                      >
                        <RxPlusCircled className="mr-2" />
                        Add Constraint
                      </Button>
                    </div>
                    <AutoDiscoveryButton
                      ocelInfo={ocelInfo}
                      constraints={constraints}
                      setConstraints={setConstraints}
                      prevDataRef={prevDataRef}
                    />
                    <Button
                      variant="outline"
                      size="icon"
                      className="h-12 w-12"
                      onClick={() => {
                        saveData();
                        toast.success("Saved Data");
                      }}
                    >
                      <LuSave size={"24"} />
                    </Button>
                  </div>
                  <p className="mt-2 mb-1">{constraints.length} Constraints</p>
                  {constraints.length > 0 && (
                    <ToggleGroup
                      type="single"
                      className="flex flex-wrap max-h-[8rem] overflow-y-auto w-full border rounded py-2"
                      value={activeIndex?.toString()}
                      onValueChange={(newVal) => {
                        const newIndex = parseInt(newVal);
                        changeIndex(newIndex);
                      }}
                    >
                      {constraints.map((c, i) => (
                        <div key={i} className="relative">
                          <ToggleGroupItem
                            value={i.toString()}
                            variant="outline"
                            className={clsx(
                              "data-[state=on]:bg-blue-200 data-[state=on]:border-blue-300",
                              prevDataRef.current[i]?.violations == null && "",
                              prevDataRef.current[i]?.violations != null &&
                                "bg-green-200/30 data-[state=on]:bg-green-300/80",
                              prevDataRef.current[i] !== undefined &&
                                prevDataRef.current[i]?.violations?.evalRes !==
                                  null &&
                                [
                                  ...Object.values(
                                    prevDataRef.current[i]?.violations
                                      ?.evalRes ?? {},
                                  ),
                                ]?.find(
                                  (vs) => vs.situationViolatedCount > 0,
                                ) != null &&
                                "bg-red-200/30 data-[state=on]:bg-red-300/80",
                            )}
                            title={c.name}
                          >
                            <span className="w-[21ch] whitespace-nowrap overflow-hidden text-ellipsis">
                              {c.name !== "" ? c.name : `Constraint ${i + 1}`}
                            </span>
                          </ToggleGroupItem>
                          <AlertHelper
                            trigger={
                              <button
                                className="absolute -right-1 -top-1 opacity-25 hover:opacity-100 hover:text-red-600"
                                title="Remove constraint"
                              >
                                <RxCrossCircled />
                              </button>
                            }
                            title="Are you sure?"
                            initialData={undefined}
                            content={() => (
                              <>
                                Deleting this constraint will delete all
                                contained nodes.
                              </>
                            )}
                            submitAction="Delete"
                            onSubmit={() => {
                              prevDataRef.current.splice(i, 1);
                              if (
                                activeIndex !== undefined &&
                                activeIndex >= constraints.length - 1
                              ) {
                                changeIndex(activeIndex - 1);
                              }
                              setConstraints((constraints) => {
                                const newConstraints = [...constraints];
                                newConstraints.splice(i, 1);
                                return newConstraints;
                              });
                            }}
                          />
                        </div>
                      ))}
                    </ToggleGroup>
                  )}
                </div>

                {constraints.map(
                  (c, i) =>
                    activeIndex === i && (
                      <Fragment key={i}>
                        <div className="max-w-xs w-full flex flex-col gap-y-1">
                          <Input
                            className="text-lg font-medium"
                            placeholder="Name"
                            type="text"
                            defaultValue={
                              c.name !== "" ? c.name : `Constraint ${i + 1}`
                            }
                            onBlur={(ev) => {
                              setConstraints((cs) => {
                                if (ev.target != null) {
                                  const newCs = [...cs];
                                  newCs[i].name = ev.target.value;
                                  return newCs;
                                } else {
                                  return cs;
                                }
                              });
                            }}
                          />
                          <div className="px-2">
                            <Textarea
                              defaultValue={c.description}
                              placeholder="Description"
                              onBlur={(ev) => {
                                setConstraints((cs) => {
                                  if (ev.target != null) {
                                    const newCs = [...cs];
                                    newCs[i].description = ev.target.value;
                                    return newCs;
                                  } else {
                                    return cs;
                                  }
                                });
                              }}
                            />
                          </div>
                        </div>
                        <ConstraintContainer
                          constraintInfo={c}
                          ocelInfo={ocelInfo}
                          objectQualifiers={objectQualifiers}
                          qualifiers={qualifiers}
                        />
                      </Fragment>
                    ),
                )}
              </>
            )}
        </div>
      </FlowContext.Provider>
    </div>
  );
}
