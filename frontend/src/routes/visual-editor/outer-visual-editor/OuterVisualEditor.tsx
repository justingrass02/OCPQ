import { OcelInfoContext } from "@/App";
import { BackendProviderContext } from "@/BackendProviderContext";
import AlertHelper from "@/components/AlertHelper";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import type { EventTypeQualifiers, ObjectTypeQualifiers } from "@/types/ocel";
import clsx from "clsx";
import json5 from "json5";
import { Fragment, useContext, useEffect, useRef, useState } from "react";
import toast from "react-hot-toast";
import { CgTrash } from "react-icons/cg";
import { LuDelete, LuSave } from "react-icons/lu";
import { RiRobot2Line } from "react-icons/ri";
import { RxCrossCircled, RxPlusCircled } from "react-icons/rx";
import { type ReactFlowInstance, type ReactFlowJsonObject } from "reactflow";
import ConstraintContainer from "../constraint-container/ConstraintContainer";
import { FlowContext } from "../helper/FlowContext";
import {
  constructDiscoveredCountConstraint,
  constructDiscoveredEFConstraint,
  constructDiscoveredORConstraint,
} from "../helper/constructNodes";
import type {
  DiscoverConstraintsRequest,
  DiscoverConstraintsRequestWrapper,
  EventTypeLinkData,
  EventTypeNodeData,
  GateLinkData,
  GateNodeData,
  ObjectVariable,
  ViolationsPerNodes,
} from "../helper/types";
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

  const prevDataRef = useRef<
    {
      flowJson: ReactFlowJsonObject<
        EventTypeNodeData | GateNodeData,
        EventTypeLinkData | GateLinkData
      >;
      violations?: ViolationsPerNodes;
      objectVariables?: ObjectVariable[];
    }[]
  >([]);
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
    if (prevDataRef.current !== undefined) {
      console.log(json5.stringify(prevDataRef.current));
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
        console.log(
          "Prev data: for " + activeIndex,
          JSON.parse(JSON.stringify(prevDataRef.current)),
        );
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
                  nodes: prevDataRef.current[activeIndex]?.flowJson.nodes,
                  edges: prevDataRef.current[activeIndex]?.flowJson.edges,
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
                  <div className="flex justify-center gap-x-2 items-start mb-2">
                    <AlertHelper
                      trigger={
                        <Button
                          title={"Delete All"}
                          variant="destructive"
                          size="icon"
                          className="text-2xl h-14 w-14"
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
                        prevDataRef.current = [];
                        setConstraints([]);
                      }}
                    />
                    <div className="flex flex-col justify-center gap-y-2">
                      <Button
                        className="text-xl py-7 px-4"
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
                      <AlertHelper
                        mode="promise"
                        trigger={
                          <Button
                            className="text-xl py-6 px-4"
                            title="Automatically Discover Constraints"
                            variant="outline"
                          >
                            <RiRobot2Line className="mr-2" /> Auto-Discovery
                          </Button>
                        }
                        title={"Automatic Constraint Discovery"}
                        initialData={
                          {
                            countConstraints: {
                              coverFraction: 0.9,
                              objectTypes: [ocelInfo.object_types[0].name],
                              enabled: true,
                            },
                            eventuallyFollowsConstraints: {
                              objectTypes: [ocelInfo.object_types[0].name],
                              coverFraction: 0.9,
                              enabled: true,
                            },
                            orConstraints: {
                              objectTypes: [ocelInfo.object_types[0].name],
                              enabled: true,
                            },
                          } satisfies DiscoverConstraintsRequestWrapper as DiscoverConstraintsRequestWrapper
                        }
                        content={({ data, setData }) => {
                          return (
                            <div>
                              <Accordion
                                type="multiple"
                                defaultValue={Object.entries(data)
                                  .filter(([_k, v]) => v.enabled)
                                  .map(([k, _v]) => k)}
                              >
                                <AccordionItem value="countConstraints">
                                  <AccordionTrigger>
                                    <h3 className="text-lg text-gray-900 flex gap-x-2 items-center">
                                      Count Constraints
                                      <Switch
                                        checked={data.countConstraints.enabled}
                                        onClick={(ev) => {
                                          ev.preventDefault();
                                          const newData = { ...data };
                                          newData.countConstraints.enabled =
                                            !newData.countConstraints.enabled;
                                          if (
                                            newData.countConstraints.enabled
                                          ) {
                                            const d =
                                              ev.currentTarget.parentElement
                                                ?.parentElement;
                                            if (
                                              d !== null &&
                                              d?.dataset.state === "closed"
                                            ) {
                                              d.click();
                                            }
                                          }
                                          setData(newData);
                                        }}
                                      />
                                    </h3>
                                  </AccordionTrigger>
                                  <AccordionContent>
                                    <div
                                      className={clsx(
                                        "ml-2 pl-2 border-l-2",
                                        !data.countConstraints.enabled &&
                                          "text-gray-400",
                                      )}
                                    >
                                      <Label>Cover Fraction</Label>
                                      <Input
                                        disabled={
                                          !data.countConstraints.enabled
                                        }
                                        type="number"
                                        min={0.0}
                                        step={0.05}
                                        max={1.0}
                                        value={
                                          data.countConstraints.coverFraction
                                        }
                                        onChange={(ev) => {
                                          setData({
                                            ...data,
                                            countConstraints: {
                                              ...data.countConstraints,
                                              coverFraction:
                                                ev.currentTarget.valueAsNumber,
                                            },
                                          });
                                        }}
                                      />
                                      <Label className="mt-3 mb-1 block">
                                        Object Types
                                      </Label>
                                      <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                                        {data.countConstraints.objectTypes.map(
                                          (ot, i) => (
                                            <li key={i}>
                                              <div className="flex gap-x-2 items-center">
                                                {ot}
                                                <button
                                                  disabled={
                                                    !data.countConstraints
                                                      .enabled
                                                  }
                                                  className="enabled:hover:text-red-500"
                                                  onClick={() => {
                                                    const newData = { ...data };
                                                    data.countConstraints.objectTypes.splice(
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
                                        disabled={
                                          !data.countConstraints.enabled
                                        }
                                        options={ocelInfo.object_types
                                          .filter(
                                            (ot) =>
                                              !data.countConstraints.objectTypes.includes(
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
                                              ...data.countConstraints,
                                              objectTypes: [
                                                ...data.countConstraints
                                                  .objectTypes,
                                                value,
                                              ],
                                            },
                                          });
                                        }}
                                        name={"Add object type..."}
                                        value={""}
                                      />
                                    </div>
                                  </AccordionContent>
                                </AccordionItem>

                                <AccordionItem value="eventuallyFollowsConstraints">
                                  <AccordionTrigger>
                                    <h3 className="text-lg text-gray-900 flex gap-x-2 items-center">
                                      Eventually Follows Constraints
                                      <Switch
                                        checked={
                                          data.eventuallyFollowsConstraints
                                            .enabled
                                        }
                                        onClick={(ev) => {
                                          ev.preventDefault();
                                          const newData = { ...data };
                                          newData.eventuallyFollowsConstraints.enabled =
                                            !newData
                                              .eventuallyFollowsConstraints
                                              .enabled;
                                          if (
                                            newData.eventuallyFollowsConstraints
                                              .enabled
                                          ) {
                                            const d =
                                              ev.currentTarget.parentElement
                                                ?.parentElement;
                                            if (
                                              d !== null &&
                                              d?.dataset.state === "closed"
                                            ) {
                                              d.click();
                                            }
                                          }
                                          setData(newData);
                                        }}
                                      />
                                    </h3>
                                  </AccordionTrigger>
                                  <AccordionContent>
                                    <div
                                      className={clsx(
                                        "ml-2 pl-2 border-l-2",
                                        !data.eventuallyFollowsConstraints
                                          .enabled && "text-gray-400",
                                      )}
                                    >
                                      <Label>Cover Fraction</Label>
                                      <Input
                                        disabled={
                                          !data.eventuallyFollowsConstraints
                                            .enabled
                                        }
                                        type="number"
                                        min={0.0}
                                        step={0.05}
                                        max={1.0}
                                        value={
                                          data.eventuallyFollowsConstraints
                                            .coverFraction
                                        }
                                        onChange={(ev) => {
                                          setData({
                                            ...data,
                                            eventuallyFollowsConstraints: {
                                              ...data.eventuallyFollowsConstraints,
                                              coverFraction:
                                                ev.currentTarget.valueAsNumber,
                                            },
                                          });
                                        }}
                                      />
                                      <Label className="mt-3 mb-1 block">
                                        Object Types
                                      </Label>
                                      <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                                        {data.eventuallyFollowsConstraints.objectTypes.map(
                                          (ot, i) => (
                                            <li key={i}>
                                              <div className="flex gap-x-2 items-center">
                                                {ot}
                                                <button
                                                  disabled={
                                                    !data
                                                      .eventuallyFollowsConstraints
                                                      .enabled
                                                  }
                                                  className="enabled:hover:text-red-500"
                                                  onClick={() => {
                                                    const newData = { ...data };
                                                    data.eventuallyFollowsConstraints.objectTypes.splice(
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
                                        disabled={
                                          !data.eventuallyFollowsConstraints
                                            .enabled
                                        }
                                        options={ocelInfo.object_types
                                          .filter(
                                            (ot) =>
                                              !data.eventuallyFollowsConstraints.objectTypes.includes(
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
                                              ...data.eventuallyFollowsConstraints,
                                              objectTypes: [
                                                ...data
                                                  .eventuallyFollowsConstraints
                                                  .objectTypes,
                                                value,
                                              ],
                                            },
                                          });
                                        }}
                                        name={"Add object type..."}
                                        value={""}
                                      />
                                    </div>
                                  </AccordionContent>
                                </AccordionItem>

                                <AccordionItem value="orConstraints">
                                  <AccordionTrigger>
                                    <h3 className="text-lg text-gray-900 flex gap-x-2 items-center">
                                      OR-Gate Constraints
                                      <Switch
                                        checked={data.orConstraints.enabled}
                                        onClick={(ev) => {
                                          ev.preventDefault();
                                          const newData = { ...data };
                                          newData.orConstraints.enabled =
                                            !newData.orConstraints.enabled;

                                          if (newData.orConstraints.enabled) {
                                            const d =
                                              ev.currentTarget.parentElement
                                                ?.parentElement;
                                            if (
                                              d !== null &&
                                              d?.dataset.state === "closed"
                                            ) {
                                              d.click();
                                            }
                                          }
                                          setData(newData);
                                        }}
                                      />
                                    </h3>
                                  </AccordionTrigger>
                                  <AccordionContent>
                                    <div
                                      className={clsx(
                                        "ml-2 pl-2 border-l-2",
                                        !data.orConstraints.enabled &&
                                          "text-gray-400",
                                      )}
                                    >
                                      <Label className="mt-3 mb-1 block">
                                        Object Types
                                      </Label>
                                      <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                                        {data.orConstraints.objectTypes.map(
                                          (ot, i) => (
                                            <li key={i}>
                                              <div className="flex gap-x-2 items-center">
                                                {ot}
                                                <button
                                                  disabled={
                                                    !data.orConstraints.enabled
                                                  }
                                                  className="enabled:hover:text-red-500"
                                                  onClick={() => {
                                                    const newData = { ...data };
                                                    data.orConstraints.objectTypes.splice(
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
                                        disabled={!data.orConstraints.enabled}
                                        options={ocelInfo.object_types
                                          .filter(
                                            (ot) =>
                                              !data.orConstraints.objectTypes.includes(
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
                                            orConstraints: {
                                              ...data.orConstraints,
                                              objectTypes: [
                                                ...data.orConstraints
                                                  .objectTypes,
                                                value,
                                              ],
                                            },
                                          });
                                        }}
                                        name={"Add object type..."}
                                        value={""}
                                      />
                                    </div>
                                  </AccordionContent>
                                </AccordionItem>
                              </Accordion>
                            </div>
                          );
                        }}
                        submitAction={"Run Discovery"}
                        onSubmit={async (data, ev) => {
                          ev.preventDefault();
                          const reqData: DiscoverConstraintsRequest = {
                            ...data,
                          };
                          for (const k of [
                            "countConstraints",
                            "eventuallyFollowsConstraints",
                            "orConstraints",
                          ] as const) {
                            if (!data[k].enabled) {
                              reqData[k] = undefined;
                            }
                          }
                          await toast
                            .promise(
                              backend["ocel/discover-constraints"](reqData)
                                .then(async (json) => {
                                  console.log({ json });
                                  const updatedConstraints = [...constraints];
                                  let index = constraints.length;

                                  for (const c of json.countConstraints) {
                                    const constructedCountConstraint =
                                      constructDiscoveredCountConstraint(
                                        c,
                                        qualifiers,
                                      );
                                    if (
                                      updatedConstraints.find(
                                        (c) =>
                                          c.name ===
                                          constructedCountConstraint.name,
                                      ) !== undefined
                                    ) {
                                      console.log(
                                        "Skipping new constraint " +
                                          constructedCountConstraint.name +
                                          " bc. constraint with same name already exists",
                                      );
                                      continue;
                                    }
                                    updatedConstraints.push({
                                      name: constructedCountConstraint.name,
                                      description:
                                        constructedCountConstraint.description,
                                    });
                                    prevDataRef.current[index] =
                                      constructedCountConstraint.constraint;
                                    index++;
                                  }

                                  for (const c of json.eventuallyFollowsConstraints) {
                                    const constructedEFConstraint =
                                      constructDiscoveredEFConstraint(
                                        c,
                                        qualifiers,
                                      );
                                    if (
                                      updatedConstraints.find(
                                        (c) =>
                                          c.name ===
                                          constructedEFConstraint.name,
                                      ) !== undefined
                                    ) {
                                      console.log(
                                        "Skipping new constraint " +
                                          constructedEFConstraint.name +
                                          " bc. constraint with same name already exists",
                                      );
                                      continue;
                                    }
                                    updatedConstraints.push({
                                      name: constructedEFConstraint.name,
                                      description:
                                        constructedEFConstraint.description,
                                    });
                                    prevDataRef.current[index] =
                                      constructedEFConstraint.constraint;
                                    index++;
                                  }

                                  for (const c of json.orConstraints) {
                                    const constructedORConstraint =
                                      constructDiscoveredORConstraint(
                                        c,
                                        qualifiers,
                                      );
                                    if (constructedORConstraint === undefined) {
                                      continue;
                                    }
                                    if (
                                      updatedConstraints.find(
                                        (c) =>
                                          c.name ===
                                          constructedORConstraint.name,
                                      ) !== undefined
                                    ) {
                                      console.log(
                                        "Skipping new constraint " +
                                          constructedORConstraint.name +
                                          " bc. constraint with same name already exists",
                                      );
                                      continue;
                                    }
                                    updatedConstraints.push({
                                      name: constructedORConstraint.name,
                                      description:
                                        constructedORConstraint.description,
                                    });
                                    prevDataRef.current[index] =
                                      constructedORConstraint.constraint;
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
                    </div>
                    <Button
                      variant="outline"
                      className="text-2xl h-14 w-14"
                      onClick={() => {
                        saveData();
                        toast.success("Saved Data");
                      }}
                    >
                      <LuSave />
                    </Button>
                  </div>
                  <p className="mt-4 mb-2">{constraints.length} Constraints</p>
                  {constraints.length > 0 && (
                    <ToggleGroup
                      type="single"
                      className="flex flex-wrap max-h-[15rem] overflow-y-auto w-full border rounded py-2"
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
