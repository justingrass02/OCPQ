import { OcelInfoContext } from "@/App";
import AlertHelper from "@/components/AlertHelper";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { EventTypeQualifiers, ObjectTypeQualifiers } from "@/types/ocel";
import clsx from "clsx";
import { Fragment, useContext, useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import { CgTrash } from "react-icons/cg";
import { RiRobot2Line } from "react-icons/ri";
import { RxCrossCircled } from "react-icons/rx";
import {
  MarkerType,
  type ReactFlowInstance,
  type ReactFlowJsonObject,
} from "reactflow";
import ConstraintContainer from "../constraint-container/ConstraintContainer";
import { FlowContext } from "../helper/FlowContext";
import type {
  DiscoverConstraintsRequest,
  DiscoverConstraintsResponse,
  EventTypeLinkData,
  EventTypeNodeData,
  ObjectVariable,
  ViolationsPerNodes,
} from "../helper/types";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { LuDelete, LuSave } from "react-icons/lu";
import { Combobox } from "@/components/ui/combobox";
import { EVENT_TYPE_LINK_TYPE } from "../helper/const";
import json5 from "json5";
const LOCALSTORAGE_SAVE_KEY_DATA = "oced-declare-data";
const LOCALSTORAGE_SAVE_KEY_CONSTRAINTS_META = "oced-declare-meta";

function parse(s: string) {
  console.log("Called parse");
  return json5.parse(s);
}
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
    if (prevDataRef.current !== undefined) {
      localStorage.setItem(
        LOCALSTORAGE_SAVE_KEY_DATA,
        json5.stringify(prevDataRef.current),
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
                  className={`w-full max-w-2xl gap-y-2 ${
                    constraints.length > 0
                      ? "justify-between"
                      : "justify-center"
                  }`}
                >
                  <div className="flex justify-center gap-x-1 items-center mb-2">
                    <AlertHelper
                      trigger={
                        <Button
                          title="Automatically Discover Constraints"
                          size="icon"
                          variant="outline"
                        >
                          <RiRobot2Line />
                        </Button>
                      }
                      title={"Automatic Constraint Discovery"}
                      initialData={
                        // eslint-disable-next-line @typescript-eslint/consistent-type-assertions
                        {
                          countConstraints: {
                            coverFraction: 0.9,
                            objectTypes: [ocelInfo.object_types[0].name],
                          },
                          eventuallyFollowsConstraints: {
                            objectTypes: [ocelInfo.object_types[0].name],
                            coverFraction: 0.9,
                          },
                        } satisfies DiscoverConstraintsRequest as DiscoverConstraintsRequest
                      }
                      content={({ data, setData }) => {
                        return (
                          <div>
                            <h3 className="text-lg text-gray-900 flex gap-x-2 items-center">
                              Count Constraints
                              <Switch
                                checked={data.countConstraints !== undefined}
                                onCheckedChange={(checked) => {
                                  if (checked) {
                                    setData({
                                      ...data,
                                      countConstraints: {
                                        coverFraction: 0.9,
                                        objectTypes: [
                                          ocelInfo.object_types[0].name,
                                        ],
                                      },
                                    });
                                  } else {
                                    setData({
                                      ...data,
                                      countConstraints: undefined,
                                    });
                                  }
                                }}
                              />
                            </h3>
                            {data.countConstraints !== undefined && (
                              <div className="pl-4">
                                <Label>Cover Fraction</Label>
                                <Input
                                  type="number"
                                  min={0.0}
                                  step={0.05}
                                  max={1.0}
                                  value={data.countConstraints.coverFraction}
                                  onChange={(ev) => {
                                    setData({
                                      ...data,
                                      countConstraints: {
                                        ...data.countConstraints!,
                                        coverFraction:
                                          ev.currentTarget.valueAsNumber,
                                      },
                                    });
                                  }}
                                />
                                <Label className="mt-3 mb-1 block">
                                  Object Types
                                </Label>
                                <ul className="flex flex-col mb-1 list-disc ml-6">
                                  {data.countConstraints.objectTypes.map(
                                    (ot, i) => (
                                      <li key={i} className="">
                                        <div className="flex gap-x-2 items-center">
                                          {ot}
                                          <button
                                            className="hover:text-red-500"
                                            onClick={() => {
                                              const newData = { ...data };
                                              data.countConstraints!.objectTypes.splice(
                                                i,
                                                1,
                                              );
                                              setData(newData);
                                            }}
                                          >
                                            <LuDelete className="w-4 h-4" />
                                          </button>
                                        </div>
                                      </li>
                                    ),
                                  )}
                                </ul>
                                <Combobox
                                  options={ocelInfo.object_types
                                    .filter(
                                      (ot) =>
                                        !data.countConstraints!.objectTypes.includes(
                                          ot.name,
                                        ),
                                    )
                                    .map((ot) => ({
                                      value: ot.name,
                                      label: ot.name,
                                    }))}
                                  onChange={(value) => {
                                    setData({
                                      ...data,
                                      countConstraints: {
                                        ...data.countConstraints!,
                                        objectTypes: [
                                          ...data.countConstraints!.objectTypes,
                                          value,
                                        ],
                                      },
                                    });
                                    console.log({ value });
                                  }}
                                  name={"Object Type"}
                                  value={""}
                                />
                              </div>
                            )}
                            <h3 className="text-lg text-gray-900 flex gap-x-2 items-center mt-4 block">
                              Eventually Follows Constraints
                              <Switch
                                checked={
                                  data.eventuallyFollowsConstraints !==
                                  undefined
                                }
                                onCheckedChange={(checked) => {
                                  if (checked) {
                                    setData({
                                      ...data,
                                      eventuallyFollowsConstraints: {
                                        coverFraction: 0.9,
                                        objectTypes: [],
                                      },
                                    });
                                  } else {
                                    setData({
                                      ...data,
                                      eventuallyFollowsConstraints: undefined,
                                    });
                                  }
                                }}
                              />
                            </h3>
                            {data.eventuallyFollowsConstraints !==
                              undefined && (
                              <div className="pl-4">
                                <Label>Cover Fraction</Label>
                                <Input
                                  type="number"
                                  min={0.0}
                                  max={1.0}
                                  step={0.05}
                                  value={
                                    data.eventuallyFollowsConstraints
                                      .coverFraction
                                  }
                                  onChange={(ev) => {
                                    setData({
                                      ...data,
                                      eventuallyFollowsConstraints: {
                                        ...data.eventuallyFollowsConstraints!,
                                        coverFraction:
                                          ev.currentTarget.valueAsNumber,
                                      },
                                    });
                                  }}
                                />
                                <Label className="mt-3 mb-1 block">
                                  Object Types
                                </Label>
                                <ul className="flex flex-col mb-1 list-disc ml-6">
                                  {data.eventuallyFollowsConstraints.objectTypes.map(
                                    (ot, i) => (
                                      <li key={i} className="">
                                        <div className="flex gap-x-2 items-center">
                                          {ot}
                                          <button
                                            className="hover:text-red-500"
                                            onClick={() => {
                                              const newData = { ...data };
                                              data.eventuallyFollowsConstraints!.objectTypes.splice(
                                                i,
                                                1,
                                              );
                                              setData(newData);
                                            }}
                                          >
                                            <LuDelete className="w-4 h-4" />
                                          </button>
                                        </div>
                                      </li>
                                    ),
                                  )}
                                </ul>
                                <Combobox
                                  options={ocelInfo.object_types
                                    .filter(
                                      (ot) =>
                                        !data.eventuallyFollowsConstraints!.objectTypes.includes(
                                          ot.name,
                                        ),
                                    )
                                    .map((ot) => ({
                                      value: ot.name,
                                      label: ot.name,
                                    }))}
                                  onChange={(value) => {
                                    setData({
                                      ...data,
                                      eventuallyFollowsConstraints: {
                                        ...data.eventuallyFollowsConstraints!,
                                        objectTypes: [
                                          ...data.eventuallyFollowsConstraints!
                                            .objectTypes,
                                          value,
                                        ],
                                      },
                                    });
                                    console.log({ value });
                                  }}
                                  name={"Object Type"}
                                  value={""}
                                />
                              </div>
                            )}
                          </div>
                        );
                      }}
                      submitAction={"Run Discovery"}
                      onSubmit={(data) => {
                        void toast
                          .promise(
                            fetch(
                              "http://localhost:3000/ocel/discover-constraints",
                              {
                                method: "post",
                                headers: { "Content-Type": "application/json" },
                                body: JSON.stringify(
                                  data satisfies DiscoverConstraintsRequest,
                                ),
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
                                          position: { x: 150, y: 150 },
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
                                      viewport: { x: 0, y: 0, zoom: 2.0 },
                                    },
                                  };
                                  index++;
                                }

                                for (const c of json.eventuallyFollowsConstraints) {
                                  const name = `${c.fromEventType} -> ${c.toEventType} for ${c.objectTypes[0]}`;
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
                                    c.objectTypes[0].substring(0, 2) + "_0";
                                  const variable = {
                                    name: varName,
                                    type: c.objectTypes[0],
                                    initiallyBound: true,
                                  };
                                  const ids = [
                                    Date.now() + "auto-ef-" + index,
                                    Date.now() + "auto-ef2-" + index,
                                  ] as const;
                                  prevDataRef.current[index] = {
                                    violations: undefined,
                                    // TODO: Add object variable
                                    objectVariables: [variable],
                                    flowJson: {
                                      nodes: [
                                        {
                                          id: ids[0],
                                          type: "eventType",
                                          position: { x: 200, y: 100 },
                                          data: {
                                            eventType: {
                                              type: "exactly",
                                              value: c.fromEventType,
                                            },
                                            eventTypeQualifier:
                                              qualifiers[c.fromEventType],
                                            countConstraint: {
                                              min: 0,
                                              max: Infinity,
                                            },
                                            selectedVariables: [
                                              {
                                                qualifier: undefined,
                                                variable,
                                                bound: false,
                                              },
                                            ],
                                            hideViolations: true,
                                          },
                                        },
                                        {
                                          id: ids[1],
                                          type: "eventType",
                                          position: { x: 200, y: 350 },
                                          data: {
                                            eventType: {
                                              type: "exactly",
                                              value: c.toEventType,
                                            },
                                            eventTypeQualifier:
                                              qualifiers[c.toEventType],
                                            countConstraint: {
                                              min: 1,
                                              max: Infinity,
                                            },
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
                                      edges: [
                                        {
                                          type: EVENT_TYPE_LINK_TYPE,
                                          source: ids[0],
                                          target: ids[1],
                                          sourceHandle: ids[0] + "-source",
                                          targetHandle: ids[1] + "-target",
                                          id: Date.now() + "link-ef-" + index,
                                          markerEnd: {
                                            type: MarkerType.ArrowClosed,
                                            width: 15,
                                            height: 12,
                                            color: "#969696",
                                          },
                                          style: {
                                            strokeWidth: 2,
                                            stroke: "#969696",
                                          },
                                          data: {
                                            color: "#969696",
                                            constraintType: "response",
                                            timeConstraint: c.secondsRange,
                                          },
                                        },
                                      ],
                                      viewport: { x: 0, y: 0, zoom: 1.5 },
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
                                `Discovered ${
                                  s!.countConstraints.length +
                                  s!.eventuallyFollowsConstraints.length
                                } Constraints`,
                              error: "Failed to Discover Constraints",
                            },
                          )
                          .finally(() => {});
                      }}
                    />
                    <Button
                      className=""
                      onClick={() => {
                        changeIndex(constraints.length, constraints.length + 1);
                        prevDataRef.current = [];
                        setConstraints((cs) => [
                          ...cs,
                          { name: "", description: "" },
                        ]);
                      }}
                    >
                      Add Constraint
                    </Button>
                    <AlertHelper
                      trigger={
                        <Button
                          variant="destructive"
                          size="icon"
                          className=""
                          disabled={constraints.length === 0}
                        >
                          <CgTrash />
                        </Button>
                      }
                      title={"Delete All Constraints"}
                      initialData={undefined}
                      content={() => (
                        <p>Are you sure? This will delete all constraints.</p>
                      )}
                      submitAction={"Delete All"}
                      onSubmit={() => {
                        setConstraints([]);
                      }}
                    />
                  </div>
                  <Button
                    variant="outline"
                    className="mb-4"
                    onClick={() => {
                      saveData();
                      toast.success("Saved Data");
                    }}
                  >
                    <LuSave />
                  </Button>

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
