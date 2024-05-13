import { OcelInfoContext } from "@/App";
import { BackendProviderContext } from "@/BackendProviderContext";
import AlertHelper from "@/components/AlertHelper";

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
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import type { EventTypeQualifiers, ObjectTypeQualifiers } from "@/types/ocel";
import clsx from "clsx";
import { Fragment, useContext, useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import { CgTrash } from "react-icons/cg";
import { LuSave } from "react-icons/lu";
import { RxPlusCircled } from "react-icons/rx";
import { ReactFlowProvider, type ReactFlowInstance } from "reactflow";
import { FlowContext } from "../helper/FlowContext";

import type { FlowAndViolationData } from "@/types/misc";
import json5 from "json5";
import { PiPlayFill } from "react-icons/pi";
import { FixedSizeList, type ListChildComponentProps } from "react-window";
import VisualEditor from "../VisualEditor";
import type { ConstraintInfo, EvaluationResPerNodes } from "../helper/types";
import {
  getViolationStyles,
  getViolationTextColor,
} from "../helper/violation-styles";
import AutoDiscoveryButton from "./AutoDiscovery";

const LOCALSTORAGE_SAVE_KEY_DATA = "oced-declare-data";
const LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META = "oced-declare-meta";

function parse(s: string) {
  try {
    return JSON.parse(s);
  } catch (e) {
    console.warn("trying to use json5 instead");
    return json5.parse(s);
  }
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
  const [deletePromptForIndex, setDeletePromptForIndex] = useState<number>();

  const Row = ({ index, style }: ListChildComponentProps) => {
    const c = constraints[index];
    if (c === undefined) {
      return null;
    }
    return (
      <div style={style} className="pb-1">
        <button
          onClick={() => setActiveIndex(index)}
          className={clsx(
            "w-full h-full block whitespace-nowrap overflow-hidden text-ellipsis border rounded px-2",
            index !== activeIndex && "bg-gray-50 border-gray-300",
            index === activeIndex && "bg-blue-200 border-blue-300",
          )}
        >
          {c.name !== "" ? c.name : `Constraint ${index + 1}`}
        </button>
      </div>
    );
  };

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
      console.log(JSON.stringify(prevDataRef.current));
      localStorage.setItem(
        LOCALSTORAGE_SAVE_KEY_DATA,
        JSON.stringify(
          prevDataRef.current.map((x) => ({ ...x, violations: undefined })),
        ),
      );
    }
    localStorage.setItem(
      LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META,
      JSON.stringify(constraints),
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
                </div>
                <AlertDialog
                  open={deletePromptForIndex !== undefined}
                  onOpenChange={(o) => {
                    if (!o) {
                      setDeletePromptForIndex(undefined);
                    }
                  }}
                >
                  <AlertDialogContent className="flex flex-col max-h-full justify-between">
                    <AlertDialogHeader>
                      <AlertDialogTitle>Delete Constraint</AlertDialogTitle>
                    </AlertDialogHeader>
                    <div className="text-sm text-gray-700 max-h-full overflow-auto px-2">
                      Deleting this constraint will delete all contained nodes.
                    </div>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={(ev) => {
                          if (deletePromptForIndex === undefined) return;
                          prevDataRef.current.splice(deletePromptForIndex, 1);
                          if (
                            activeIndex !== undefined &&
                            activeIndex >= constraints.length - 1
                          ) {
                            changeIndex(activeIndex - 1);
                          }
                          setConstraints((constraints) => {
                            const newConstraints = [...constraints];
                            newConstraints.splice(deletePromptForIndex, 1);
                            return newConstraints;
                          });
                        }}
                      >
                        Delete
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>

                <ReactFlowProvider>
                  <Fragment>
                    <div
                      className={clsx(
                        "grid",
                        activeIndex !== undefined &&
                          constraints[activeIndex] !== undefined &&
                          "grid-cols-3",
                        (activeIndex === undefined ||
                          constraints[activeIndex] === undefined) &&
                          "grid-cols-1",
                      )}
                    >
                      <div>
                        <p className="h-[2rem]">
                          {constraints.length} Constraints
                        </p>
                        <FixedSizeList
                          height={125}
                          itemCount={constraints.length}
                          itemSize={40}
                          width={300}
                        >
                          {Row}
                        </FixedSizeList>
                      </div>
                      {activeIndex !== undefined &&
                        constraints[activeIndex] !== undefined && (
                          <div>
                            <p className="h-[2rem]">Selected Constraint</p>
                            <div
                              className="max-w-xs w-full flex flex-col gap-y-1 px-2"
                              key={activeIndex}
                            >
                              <>
                                <Input
                                  className="text-lg font-medium"
                                  placeholder="Name"
                                  type="text"
                                  defaultValue={
                                    constraints[activeIndex].name !== ""
                                      ? constraints[activeIndex].name
                                      : `Constraint ${activeIndex + 1}`
                                  }
                                  onBlur={(ev) => {
                                    setConstraints((cs) => {
                                      if (ev.target != null) {
                                        const newCs = [...cs];
                                        newCs[activeIndex].name =
                                          ev.target.value;
                                        return newCs;
                                      } else {
                                        return cs;
                                      }
                                    });
                                  }}
                                />
                                <div className="px-2">
                                  <Textarea
                                    defaultValue={
                                      constraints[activeIndex].description
                                    }
                                    placeholder="Description"
                                    onBlur={(ev) => {
                                      setConstraints((cs) => {
                                        if (ev.target != null) {
                                          const newCs = [...cs];
                                          newCs[activeIndex].description =
                                            ev.target.value;
                                          return newCs;
                                        } else {
                                          return cs;
                                        }
                                      });
                                    }}
                                  />
                                </div>
                              </>
                            </div>
                          </div>
                        )}
                      {activeIndex !== undefined &&
                        constraints[activeIndex] !== undefined && (
                          <div>
                            <p className="h-[2rem]">Constraint Info</p>
                            <div className="px-2 border rounded flex flex-col items-center justify-around w-full">
                              {
                                prevDataRef.current[activeIndex].flowJson.nodes
                                  .length
                              }{" "}
                              Nodes
                              <br />
                              {
                                prevDataRef.current[activeIndex].flowJson.edges
                                  .length
                              }{" "}
                              Edges
                              <TotalViolationInfo
                                violations={
                                  prevDataRef.current[activeIndex].violations
                                }
                              />
                            </div>
                          </div>
                        )}
                    </div>
                    <div className="relative w-full h-full px-12">
                      {activeIndex !== undefined &&
                        constraints[activeIndex] !== undefined && (
                          <div className="xl:w-full min-h-[50rem] h-full border p-2">
                            {qualifiers !== undefined &&
                              ocelInfo !== undefined && (
                                <>
                                  <VisualEditor
                                    constraintInfo={constraints[activeIndex]}
                                    eventTypeQualifiers={qualifiers}
                                    ocelInfo={ocelInfo}
                                  ></VisualEditor>
                                </>
                              )}
                          </div>
                        )}
                    </div>
                  </Fragment>
                </ReactFlowProvider>
              </>
            )}
        </div>
      </FlowContext.Provider>
    </div>
  );
}

function TotalViolationInfo({
  violations,
}: {
  violations: EvaluationResPerNodes | undefined;
}) {
  const [situationViolatedCount, situationCount] = Object.values(
    violations?.evalRes ?? {},
  ).reduce(
    ([violationCount, situationCount], val) => [
      violationCount + val.situationViolatedCount,
      situationCount + val.situationCount,
    ],
    [0, 0],
  );
  const percentage = (100 * situationViolatedCount) / situationCount;

  return (
    <div
      className={clsx(
        "rounded w-full",
        isNaN(percentage) && "text-gray-700",
        !isNaN(percentage) && "font-bold border-2",
        !isNaN(percentage) &&
          getViolationStyles({ situationViolatedCount, situationCount }),
        !isNaN(percentage) &&
          getViolationTextColor({ situationViolatedCount, situationCount }),
      )}
    >
      {!isNaN(percentage) && (
        <>{Math.round(100 * percentage) / 100}% Violations</>
      )}
      {isNaN(percentage) && <>No evaluation result available</>}
      <br />
      {!isNaN(percentage) && (
        <>
          ({situationViolatedCount} of {situationCount})
        </>
      )}
      {isNaN(percentage) && (
        <span className="inline-flex items-center gap-x-1">
          Evaluate using the <PiPlayFill className="text-purple-600" /> button
          below
        </span>
      )}
    </div>
  );
}
