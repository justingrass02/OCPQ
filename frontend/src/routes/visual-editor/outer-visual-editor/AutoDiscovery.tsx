import AlertHelper from "@/components/AlertHelper";
import { Button } from "@/components/ui/button";
import { RiRobot2Line } from "react-icons/ri";
import type {
  DiscoverConstraintsRequest,
  DiscoverConstraintsRequestWrapper,
} from "../helper/types";
import type { EventTypeQualifiers, OCELInfo } from "@/types/ocel";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

// import {
//   constructDiscoveredCountConstraint,
//   constructDiscoveredEFConstraint,
//   constructDiscoveredORConstraint,
// } from "../helper/constructNodes";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import clsx from "clsx";
import { LuDelete } from "react-icons/lu";
import { Combobox } from "@/components/ui/combobox";
import toast from "react-hot-toast";
import { BackendProviderContext } from "@/BackendProviderContext";
import { useContext } from "react";
import type { FlowAndViolationData } from "@/types/misc";
import { bindingBoxTreeToNodes } from "../helper/constructNodes";

export default function AutoDiscoveryButton({
  ocelInfo,
  qualifiers,
  constraints,
  setConstraints,
  prevDataRef,
}: {
  ocelInfo: OCELInfo;
  qualifiers: EventTypeQualifiers;
  constraints: {
    name: string;
    description: string;
  }[];
  setConstraints: React.Dispatch<
    React.SetStateAction<
      {
        name: string;
        description: string;
      }[]
    >
  >;
  prevDataRef: { current: FlowAndViolationData[] };
}) {
  const backend = useContext(BackendProviderContext);

  return (
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
                        if (newData.countConstraints.enabled) {
                          const d =
                            ev.currentTarget.parentElement?.parentElement;
                          if (d !== null && d?.dataset.state === "closed") {
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
                      !data.countConstraints.enabled && "text-gray-400",
                    )}
                  >
                    <Label>Cover Fraction</Label>
                    <Input
                      disabled={!data.countConstraints.enabled}
                      type="number"
                      min={0.0}
                      step={0.05}
                      max={1.0}
                      value={data.countConstraints.coverFraction}
                      onChange={(ev) => {
                        setData({
                          ...data,
                          countConstraints: {
                            ...data.countConstraints,
                            coverFraction: ev.currentTarget.valueAsNumber,
                          },
                        });
                      }}
                    />
                    <Label className="mt-3 mb-1 block">Object Types</Label>
                    <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                      {data.countConstraints.objectTypes.map((ot, i) => (
                        <li key={i}>
                          <div className="flex gap-x-2 items-center">
                            {ot}
                            <button
                              disabled={!data.countConstraints.enabled}
                              className="enabled:hover:text-red-500"
                              onClick={() => {
                                const newData = { ...data };
                                data.countConstraints.objectTypes.splice(i, 1);
                                setData(newData);
                              }}
                            >
                              <LuDelete className="w-4 h-4" />
                            </button>
                          </div>
                        </li>
                      ))}
                    </ul>
                    <Combobox
                      disabled={!data.countConstraints.enabled}
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
                              ...data.countConstraints.objectTypes,
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
                      checked={data.eventuallyFollowsConstraints.enabled}
                      onClick={(ev) => {
                        ev.preventDefault();
                        const newData = { ...data };
                        newData.eventuallyFollowsConstraints.enabled =
                          !newData.eventuallyFollowsConstraints.enabled;
                        if (newData.eventuallyFollowsConstraints.enabled) {
                          const d =
                            ev.currentTarget.parentElement?.parentElement;
                          if (d !== null && d?.dataset.state === "closed") {
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
                      !data.eventuallyFollowsConstraints.enabled &&
                        "text-gray-400",
                    )}
                  >
                    <Label>Cover Fraction</Label>
                    <Input
                      disabled={!data.eventuallyFollowsConstraints.enabled}
                      type="number"
                      min={0.0}
                      step={0.05}
                      max={1.0}
                      value={data.eventuallyFollowsConstraints.coverFraction}
                      onChange={(ev) => {
                        setData({
                          ...data,
                          eventuallyFollowsConstraints: {
                            ...data.eventuallyFollowsConstraints,
                            coverFraction: ev.currentTarget.valueAsNumber,
                          },
                        });
                      }}
                    />
                    <Label className="mt-3 mb-1 block">Object Types</Label>
                    <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                      {data.eventuallyFollowsConstraints.objectTypes.map(
                        (ot, i) => (
                          <li key={i}>
                            <div className="flex gap-x-2 items-center">
                              {ot}
                              <button
                                disabled={
                                  !data.eventuallyFollowsConstraints.enabled
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
                      disabled={!data.eventuallyFollowsConstraints.enabled}
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
                              ...data.eventuallyFollowsConstraints.objectTypes,
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
                            ev.currentTarget.parentElement?.parentElement;
                          if (d !== null && d?.dataset.state === "closed") {
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
                      !data.orConstraints.enabled && "text-gray-400",
                    )}
                  >
                    <Label className="mt-3 mb-1 block">Object Types</Label>
                    <ul className="flex flex-col mb-1 list-disc ml-6 text-base">
                      {data.orConstraints.objectTypes.map((ot, i) => (
                        <li key={i}>
                          <div className="flex gap-x-2 items-center">
                            {ot}
                            <button
                              disabled={!data.orConstraints.enabled}
                              className="enabled:hover:text-red-500"
                              onClick={() => {
                                const newData = { ...data };
                                data.orConstraints.objectTypes.splice(i, 1);
                                setData(newData);
                              }}
                            >
                              <LuDelete className="w-4 h-4" />
                            </button>
                          </div>
                        </li>
                      ))}
                    </ul>
                    <Combobox
                      disabled={!data.orConstraints.enabled}
                      options={ocelInfo.object_types
                        .filter(
                          (ot) =>
                            !data.orConstraints.objectTypes.includes(ot.name),
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
                              ...data.orConstraints.objectTypes,
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
                for (const [name, newConstraint] of json.constraints) {
                  updatedConstraints.push({
                    name,
                    description: "Automatically Discovered",
                  });
                  const [ns, es] = bindingBoxTreeToNodes(
                    newConstraint,
                    0,
                    0,
                    0,
                    Date.now() + " - " + index,
                  );
                  prevDataRef.current[index] = {
                    flowJson: {
                      nodes: ns,
                      edges: es,
                      viewport: { x: 0, y: 0, zoom: 0.4 },
                    },
                  };
                  index++;
                }

                // for (const c of json.countConstraints) {
                //   const constructedCountConstraint =
                //     constructDiscoveredCountConstraint(c, qualifiers);
                //   if (
                //     updatedConstraints.find(
                //       (c) => c.name === constructedCountConstraint.name,
                //     ) !== undefined
                //   ) {
                //     console.log(
                //       "Skipping new constraint " +
                //         constructedCountConstraint.name +
                //         " bc. constraint with same name already exists",
                //     );
                //     continue;
                //   }
                //   updatedConstraints.push({
                //     name: constructedCountConstraint.name,
                //     description: constructedCountConstraint.description,
                //   });
                //   prevDataRef.current[index] =
                //     constructedCountConstraint.constraint;
                //   index++;
                // }

                // for (const c of json.eventuallyFollowsConstraints) {
                //   const constructedEFConstraint =
                //     constructDiscoveredEFConstraint(c, qualifiers);
                //   if (
                //     updatedConstraints.find(
                //       (c) => c.name === constructedEFConstraint.name,
                //     ) !== undefined
                //   ) {
                //     console.log(
                //       "Skipping new constraint " +
                //         constructedEFConstraint.name +
                //         " bc. constraint with same name already exists",
                //     );
                //     continue;
                //   }
                //   updatedConstraints.push({
                //     name: constructedEFConstraint.name,
                //     description: constructedEFConstraint.description,
                //   });
                //   prevDataRef.current[index] =
                //     constructedEFConstraint.constraint;
                //   index++;
                // }

                // for (const c of json.orConstraints) {
                //   const constructedORConstraint =
                //     constructDiscoveredORConstraint(c, qualifiers);
                //   if (constructedORConstraint === undefined) {
                //     continue;
                //   }
                //   if (
                //     updatedConstraints.find(
                //       (c) => c.name === constructedORConstraint.name,
                //     ) !== undefined
                //   ) {
                //     console.log(
                //       "Skipping new constraint " +
                //         constructedORConstraint.name +
                //         " bc. constraint with same name already exists",
                //     );
                //     continue;
                //   }
                //   updatedConstraints.push({
                //     name: constructedORConstraint.name,
                //     description: constructedORConstraint.description,
                //   });
                //   prevDataRef.current[index] =
                //     constructedORConstraint.constraint;
                //   index++;
                // }

                setConstraints(updatedConstraints);
                return json;
              })
              .catch((err) => {
                console.error(err);
                return undefined;
              }),
            {
              loading: "Executing Auto-Discovery...",
              success: (s) => `Discovered ${s?.constraints.length} Constraints`,
              error: "Failed to Discover Constraints",
            },
          )
          .finally(() => {});
      }}
    />
  );
}
