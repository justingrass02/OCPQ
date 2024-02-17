import { OcelInfoContext } from "@/App";
import AlertHelper from "@/components/AlertHelper";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { EventTypeQualifiers, ObjectTypeQualifiers } from "@/types/ocel";
import { Fragment, useContext, useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import { RiRobot2Line } from "react-icons/ri";
import { RxCrossCircled } from "react-icons/rx";
import type { ReactFlowInstance, ReactFlowJsonObject } from "reactflow";
import ConstraintContainer from "../constraint-container/ConstraintContainer";
import { FlowContext } from "../helper/FlowContext";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  ObjectVariable,
  ViolationsPerNodes,
} from "../helper/types";
import clsx from "clsx";

export default function VisualEditorOuter() {
  const [qualifiers, setQualifiers] = useState<EventTypeQualifiers>();
  const [objectQualifiers, setObjectQualifiers] =
    useState<ObjectTypeQualifiers>();
  const ocelInfo = useContext(OcelInfoContext);
  const [constraints, setConstraints] = useState<
    { name: string; description: string }[]
  >([]);
  const [currentInstanceAndData, setCurrentInstanceAndData] = useState<{
    instance?: ReactFlowInstance | undefined;
    getter?: () =>
      | {
          violations?: ViolationsPerNodes;
          objectVariables?: ObjectVariable[];
        }
      | undefined;
  }>({});
  useEffect(() => {
    toast
      .promise(
        fetch("http://localhost:3000/ocel/event-qualifiers", { method: "get" }),
        {
          loading: "Fetching qualifier info...",
          success: "Loaded qualifier info",
          error: "Failed to fetch qualifier info",
        },
        { id: "fetch-event-qualifiers" },
      )
      .then(async (res) => {
        const json: EventTypeQualifiers = await res.json();
        setQualifiers(json);
      })
      .catch((e) => {
        console.error(e);
      });
  }, []);

  useEffect(() => {
    toast
      .promise(
        fetch("http://localhost:3000/ocel/object-qualifiers", {
          method: "get",
        }),
        {
          loading: "Fetching object qualifier info...",
          success: "Loaded object qualifier info",
          error: "Failed to fetch object qualifier info",
        },
        { id: "fetch-object-qualifiers" },
      )
      .then(async (res) => {
        const json: ObjectTypeQualifiers = await res.json();
        setObjectQualifiers(json);
      })
      .catch((e) => {
        console.error(e);
      });
  }, []);

  const [activeIndex, setActiveIndex] = useState<number>();
  const prevDataRef = useRef<
    {
      flowJson: ReactFlowJsonObject<EventTypeNodeData, EventTypeLinkData>;
      violations?: ViolationsPerNodes;
      objectVariables?: ObjectVariable[];
    }[]
  >([]);

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
          objectVariables: prevOtherData?.objectVariables,
        };
      }
      setActiveIndex(newIndex);
    }
  }
  return (
    <div>
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
                objectVariables: data?.objectVariables,
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
            // setReactFlowInstance(i);
            if (activeIndex !== undefined && i !== undefined) {
              const prevData = prevDataRef.current[activeIndex];
              i.setNodes(prevData?.flowJson?.nodes ?? []);
              i.setEdges(prevData?.flowJson?.edges ?? []);
              i.setViewport(prevData?.flowJson?.viewport ?? {});
            }
          },
          registerOtherDataGetter: (getter) => {
            setCurrentInstanceAndData((ci) => ({ ...ci, getter }));
          },
          otherData:
            activeIndex !== undefined
              ? {
                  objectVariables:
                    prevDataRef.current[activeIndex]?.objectVariables,
                  violations: prevDataRef.current[activeIndex]?.violations,
                }
              : undefined,
        }}
      >
        <div className="flex flex-col justify-around items-center mb-2 gap-y-2">
          {ocelInfo !== undefined &&
            qualifiers !== undefined &&
            objectQualifiers !== undefined && (
              <>
                <div
                  className={`w-full max-w-lg gap-y-2 ${
                    constraints.length > 0
                      ? "justify-between"
                      : "justify-center"
                  }`}
                >
                  <div className="flex justify-center gap-x-1 mb-2">
                    <Button
                      className=""
                      onClick={() => {
                        changeIndex(constraints.length, constraints.length + 1);
                        setConstraints((cs) => [
                          ...cs,
                          { name: "", description: "" },
                        ]);
                      }}
                    >
                      Add {constraints.length === 0 && " Constraints"}
                    </Button>
                    <Button
                      size="icon"
                      variant="outline"
                      onClick={() => {
                        type DiscoverConstraintsRequest = {
                          countConstraints: { coverFraction: number };
                        };
                        type DiscoverConstraintsResponse = {
                          countConstraints: {
                            countConstraint: { min: number; max: number };
                            objectType: string;
                            eventType: EventTypeNodeData["eventType"];
                          }[];
                        };
                        void toast.promise(
                          fetch(
                            "http://localhost:3000/ocel/discover-constraints",
                            {
                              method: "post",
                              headers: { "Content-Type": "application/json" },
                              body: JSON.stringify({
                                countConstraints: { coverFraction: 0.9 },
                              } satisfies DiscoverConstraintsRequest),
                            },
                          )
                            .then(async (res) => {
                              const json =
                                (await res.json()) as DiscoverConstraintsResponse;
                              console.log({ json });
                              const updatedConstraints = [...constraints];
                              let index = constraints.length;
                              for (const c of json.countConstraints) {
                                const countStr =
                                  c.countConstraint.min ===
                                  c.countConstraint.max
                                    ? `=${c.countConstraint.min}`
                                    : c.countConstraint.min === 0
                                    ? `<=${c.countConstraint.max}`
                                    : `${c.countConstraint.min} - ${c.countConstraint.max}`;
                                const name = `${countStr} "${
                                  c.eventType.type === "exactly"
                                    ? c.eventType.value
                                    : "..."
                                }" per ${c.objectType}`;
                                if (
                                  updatedConstraints.find(
                                    (c) => c.name === name,
                                  ) !== undefined
                                ) {
                                  console.log(
                                    "Skipping new constraint " +
                                      name +
                                      " bc. constraint with same name already exists",
                                  );
                                  continue;
                                }
                                updatedConstraints.push({
                                  name,
                                  description:
                                    "Automatically Discovered Constraint",
                                });
                                const varName =
                                  c.objectType.substring(0, 2) + "_0";
                                const variable = {
                                  name: varName,
                                  type: c.objectType,
                                  initiallyBound: true,
                                };
                                prevDataRef.current[index] = {
                                  violations: undefined,
                                  // TODO: Add object variable
                                  objectVariables: [variable],
                                  flowJson: {
                                    nodes: [
                                      {
                                        id: Date.now() + "auto" + index,
                                        type: "eventType",
                                        position: { x: 100, y: 100 },
                                        data: {
                                          eventType: c.eventType,
                                          eventTypeQualifier:
                                            qualifiers[
                                              c.eventType.type === "exactly"
                                                ? c.eventType.value
                                                : ""
                                            ],
                                          countConstraint: c.countConstraint,
                                          selectedVariables: [
                                            {
                                              qualifier: undefined,
                                              variable,
                                              bound: false,
                                            },
                                          ],
                                        },
                                      },
                                    ],
                                    edges: [],
                                    viewport: { x: 0, y: 0, zoom: 1.0 },
                                  },
                                };
                                index++;
                              }
                              setConstraints(updatedConstraints);
                              return json;
                            })
                            .catch((err) => {
                              console.error(err);
                              return undefined;
                            }),
                          {
                            loading: "Executing Auto-Discovery...",
                            success: (s) =>
                              `Discovered ${s?.countConstraints.length} Constraints`,
                            error: "Failed to Discover Constraints",
                          },
                        );
                      }}
                    >
                      <RiRobot2Line />
                    </Button>
                  </div>
                  {constraints.length > 0 && (
                    <ToggleGroup
                      type="single"
                      className="flex flex-wrap"
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
                              prevDataRef.current[i]?.violations?.find(
                                (vs) => vs.violations.length > 0,
                              ) != null &&
                                "bg-red-200/30 data-[state=on]:bg-red-300/80",
                            )}
                          >
                            {c.name !== "" ? c.name : `Constraint ${i + 1}`}
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
