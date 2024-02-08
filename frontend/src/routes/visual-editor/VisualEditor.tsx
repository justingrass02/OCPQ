import {
  memo,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from "react";
import ReactFlow, {
  Background,
  Controls,
  MarkerType,
  Panel,
  ReactFlowProvider,
  addEdge,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
} from "reactflow";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

import { OcelInfoContext } from "@/App";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import { Input } from "@/components/ui/input";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { ImageIcon } from "@radix-ui/react-icons";
import { toPng } from "html-to-image";
import toast from "react-hot-toast";
import { LuLayoutDashboard, LuUnlink, LuX } from "react-icons/lu";
import { MdAdd } from "react-icons/md";
import { RxPlus, RxReset } from "react-icons/rx";
import { TbBinaryTree, TbRestore } from "react-icons/tb";
import "reactflow/dist/style.css";
import type {
  EventTypeQualifiers,
  OCELInfo,
  ObjectTypeQualifiers,
} from "../../types/ocel";
import { evaluateConstraints } from "./evaluation/evaluate-constraints";
import { ConstraintInfoContext } from "./helper/ConstraintInfoContext";
import { useLayoutedElements } from "./helper/LayoutFlow";
import {
  ViolationsContext,
  type ViolationsContextValue,
} from "./helper/ViolationsContext";
import { EVENT_TYPE_LINK_TYPE, edgeTypes, nodeTypes } from "./helper/const";
import type {
  EventTypeLinkData,
  EventTypeNodeData,
  ObjectVariable,
  ViolationsPerNode,
} from "./helper/types";
import AlertHelper from "@/components/AlertHelper";

interface VisualEditorProps {
  ocelInfo: OCELInfo;
  eventTypeQualifiers: EventTypeQualifiers;
}
const COLORS = [
  "#1f78b4", // Blue
  "#33a02c", // Green
  "#e31a1c", // Red
  "#ff7f00", // Orange
  "#6a3d9a", // Purple
  "#b2df8a", // Light Green
  "#fb9a99", // Light Red
  "#fdbf6f", // Light Orange
  "#cab2d6", // Light Purple
  "#ffff99", // Yellow
];

function VisualEditor(props: VisualEditorProps) {
  const [mode, setMode] = useState<"normal" | "view-tree" | "readonly">(
    "normal",
  );
  const [violationDetails, setViolationDetails] = useState<ViolationsPerNode>();

  const [violationInfo, setViolationInfo] = useState<ViolationsContextValue>({
    showViolationsFor: (d) => {
      console.log({ d });
      setViolationDetails(d);
    },
  });

  const objectTypeToColor: Record<string, string> = useMemo(() => {
    const ret: Record<string, string> = {};
    props.ocelInfo.object_types.forEach((type, i) => {
      ret[type.name] = COLORS[i % COLORS.length];
    });
    return ret;
  }, [props.eventTypeQualifiers]);
  const intitialNodes = useMemo<Node<EventTypeNodeData>[]>(
    () =>
      Object.keys(props.eventTypeQualifiers).map((eventType) => {
        return {
          id: eventType + "-01",
          type: "eventType",
          position: { x: 0, y: 0 },
          data: {
            eventType,
            eventTypeQualifier: props.eventTypeQualifiers[eventType],
            objectTypeToColor,
            selectedVariables: [],
            countConstraint: { min: 0, max: Infinity },
            onDataChange: (id, newData) => {
              setNodes((ns) => {
                const newNodes = [...ns];
                const changedNode = newNodes.find((n) => n.id === id);
                if (changedNode?.data !== undefined) {
                  changedNode.data = { ...changedNode.data, ...newData };
                } else {
                  console.warn("Did not find changed node data");
                }
                return newNodes;
              });
            },
          },
        };
      }),
    [props.eventTypeQualifiers],
  );
  const [nodes, setNodes, onNodesChange] =
    useNodesState<EventTypeNodeData>(intitialNodes);

  const [edges, setEdges, onEdgesChange] = useEdgesState<EventTypeLinkData>([]);

  const { objectVariables } = useContext(ConstraintInfoContext);

  const onConnect = useCallback(
    ({ source, sourceHandle, target, targetHandle }: Edge | Connection) => {
      setEdges((eds) => {
        if (
          source === null ||
          target == null ||
          sourceHandle == null ||
          targetHandle == null
        ) {
          return eds;
        } else {
          const color = "#969696";
          const newEdge: Edge<EventTypeLinkData> = {
            id: sourceHandle + "|||" + targetHandle,
            type: EVENT_TYPE_LINK_TYPE,
            source,
            sourceHandle,
            target,
            targetHandle,
            markerEnd: {
              type: MarkerType.ArrowClosed,
              width: 15,
              height: 12,
              color,
            },
            style: {
              strokeWidth: 2,
              stroke: color,
            },
            data: {
              color,
              constraintType: "response",
              timeConstraint: { minSeconds: 0, maxSeconds: Infinity },
              onDataChange: (id, newData) => {
                setEdges((es) => {
                  const newEdges = [...es];
                  const changedEdge = newEdges.find((e) => e.id === id);
                  if (changedEdge?.data !== undefined) {
                    changedEdge.data = { ...changedEdge.data, ...newData };
                  } else {
                    console.warn("Did not find changed edge data");
                  }
                  return newEdges;
                });
              },
              onDelete: (id: string) => {
                setEdges((edges) => {
                  const newEdges = edges.filter((e) => e.id !== id);
                  return newEdges;
                });
              },
            },
          };
          return addEdge(newEdge, eds);
        }
      });
    },
    [setEdges],
  );

  const { getLayoutedElements } = useLayoutedElements();
  return (
    <ViolationsContext.Provider value={violationInfo}>
      <ReactFlow
        onInit={(flow) => {
          getLayoutedElements(
            {
              "elk.algorithm": "layered",
              "elk.direction": "RIGHT",
            },
            false,
          );
          setTimeout(() => {
            flow.fitView({ duration: 300 });
          }, 200);
        }}
        edgeTypes={edgeTypes}
        nodeTypes={nodeTypes}
        nodes={nodes}
        edges={edges}
        nodesConnectable={mode === "normal"}
        nodesDraggable={mode === "normal" || mode === "view-tree"}
        elementsSelectable={mode === "normal"}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        proOptions={{ hideAttribution: true }}
        // connectionLineComponent={(props) => (
        //   <ConnectionLine {...props} objectTypeToColor={objectTypeToColor} />
        // )}
      >
        <Controls
          onInteractiveChange={(status) => {
            if (status) {
              setMode("normal");
            } else {
              setMode("readonly");
            }
          }}
        />
        <Panel position="top-right" className="flex gap-x-2">
          <AlertHelper
            initialData={{ eventType: props.ocelInfo.event_types[0].name }}
            trigger={
              <Button
                disabled={mode !== "normal"}
                variant="outline"
                size="icon"
                title="Add node"
                className="bg-white"
                onClick={() => {}}
              >
                <MdAdd />
              </Button>
            }
            title={"Add Node"}
            submitAction={"Submit"}
            onSubmit={(data) => {
              setNodes((nodes) => {
                return [
                  ...nodes,
                  {
                    id: data.eventType + Date.now(),
                    type: "eventType",
                    position: { x: 0, y: 0 },
                    data: {
                      eventType: data.eventType,
                      eventTypeQualifier:
                        props.eventTypeQualifiers[data.eventType],
                      objectTypeToColor,
                      selectedVariables: [],
                      countConstraint: { min: 0, max: Infinity },
                      onDataChange: (id, newData) => {
                        setNodes((ns) => {
                          const newNodes = [...ns];
                          const changedNode = newNodes.find((n) => n.id === id);
                          if (changedNode?.data !== undefined) {
                            changedNode.data = {
                              ...changedNode.data,
                              ...newData,
                            };
                          } else {
                            console.warn("Did not find changed node data");
                          }
                          return newNodes;
                        });
                      },
                    },
                  },
                ];
              });
              console.log({ data });
            }}
            content={({ data, setData }) => {
              return (
                <>
                  <span>Please select the node type to add below.</span>
                  <Select
                    value={data.eventType}
                    onValueChange={(v) => {
                      setData({ ...data, eventType: v });
                    }}
                  >
                    <SelectTrigger className={"my-2"}>
                      <SelectValue placeholder="Select an event type" />
                    </SelectTrigger>
                    <SelectContent>
                      {props.ocelInfo.event_types.map((t) => (
                        <SelectItem key={t.name} value={t.name}>
                          {t.name}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </>
              );
            }}
          />
          <AlertHelper
            trigger={
              <Button
                disabled={mode !== "normal"}
                variant="outline"
                size="icon"
                title="Reset edges"
                className="text-red-600 bg-white hover:bg-red-400"
              >
                <RxReset />
              </Button>
            }
            title={"Reset all nodes and edges"}
            initialData={undefined}
            content={() => (
              <span>
                This will delete all current nodes and edges. Are you sure?
              </span>
            )}
            submitAction={"Yes, I am sure"}
            onSubmit={() => {
              setEdges([]);
              setNodes([]);
            }}
          />

          <Button
            disabled={mode !== "normal"}
            variant="outline"
            size="icon"
            title="Save PNG"
            className="bg-white"
            onClick={(ev) => {
              const button = ev.currentTarget;
              button.disabled = true;
              const scaleFactor = 2.0;
              const viewPort = document.querySelector(
                ".react-flow__viewport",
              ) as HTMLElement;
              setTimeout(() => {
                void toPng(viewPort, {
                  canvasHeight: viewPort.clientHeight * scaleFactor,
                  canvasWidth: viewPort.clientWidth * scaleFactor,
                })
                  .then((dataURL) => {
                    const a = document.createElement("a");
                    a.setAttribute("download", "oced-declare-export.png");
                    a.setAttribute("href", dataURL);
                    a.click();
                  })
                  .finally(() => {
                    button.disabled = false;
                  });
              }, 50);
            }}
          >
            <ImageIcon />
          </Button>

          <Button
            variant="outline"
            size="icon"
            title={mode !== "view-tree" ? "Construct tree" : "Edit"}
            className="bg-white"
            onClick={async () => {
              const res = await evaluateConstraints(
                objectVariables,
                nodes,
                edges,
              );
              setViolationInfo((vi) => ({ ...vi, violationsPerNode: res }));
            }}
          >
            {mode !== "view-tree" && <TbBinaryTree />}
            {mode === "view-tree" && <TbRestore />}
          </Button>

          <Button
            disabled={mode !== "normal"}
            variant="outline"
            size="icon"
            title="Apply automatic layout"
            className="bg-white"
            onClick={() => {
              getLayoutedElements(
                {
                  "elk.algorithm": "layered",
                  "elk.direction": "DOWN",
                },
                true,
              );
            }}
          >
            <LuLayoutDashboard />
          </Button>
        </Panel>
        <Background />
      </ReactFlow>
      {violationDetails !== undefined && (
        <ViolationDetailsSheet
          violationDetails={violationDetails}
          setViolationDetails={setViolationDetails}
        />
      )}
    </ViolationsContext.Provider>
  );
}

const ViolationDetailsSheet = memo(function ViolationDetailsSheet({
  violationDetails,
  setViolationDetails,
}: {
  violationDetails: ViolationsPerNode;
  setViolationDetails: React.Dispatch<
    React.SetStateAction<ViolationsPerNode | undefined>
  >;
}) {
  const VIOLATIONS_TO_SHOW = 100;
  return (
    <Sheet
      modal={false}
      open={violationDetails !== undefined}
      onOpenChange={(o) => {
        if (!o) {
          setViolationDetails(undefined);
        }
      }}
    >
      {violationDetails !== undefined && (
        <SheetContent
          overlay={false}
          onInteractOutside={(ev) => {
            ev.preventDefault();
          }}
        >
          <SheetHeader>
            <SheetTitle>
              Violations for{" "}
              <span className="text-blue-900">{violationDetails?.nodeID}</span>
            </SheetTitle>
            <SheetDescription>
              {violationDetails?.violations.length} Violations
              {violationDetails?.violations.length > VIOLATIONS_TO_SHOW && (
                <>
                  <br />
                  <span className="text-xs">
                    Showing only the first {VIOLATIONS_TO_SHOW} Violations
                  </span>
                </>
              )}
            </SheetDescription>
          </SheetHeader>
          <ul className="overflow-auto h-[80vh] bg-slate-50 border rounded-sm mt-2 px-2 py-0.5 text-xs">
            {violationDetails.violations
              .slice(0, VIOLATIONS_TO_SHOW)
              .map(([[info, binding], reason], i) => (
                <li
                  key={i}
                  className="border mx-1 my-2 px-1 py-1 rounded-sm bg-blue-50"
                >
                  <div>
                    <p className="text-orange-600">{reason}</p>
                    <span className="text-emerald-700">Past events:</span>{" "}
                    <span className="font-mono">
                      {info.past_events.map((e) => e.event_id).join(",")}
                    </span>
                    <h3 className="text-blue-700">Objects:</h3>
                    <ul className="flex flex-col ml-6 list-disc">
                      {Object.entries(binding).map(([variable, value]) => (
                        <li key={variable}>
                          <span className="text-cyan-700">{variable}:</span>{" "}
                          {"Single" in value ? value.Single : "?"}
                        </li>
                      ))}
                    </ul>
                  </div>
                </li>
              ))}
          </ul>
        </SheetContent>
      )}
    </Sheet>
  );
});

interface ConstraintContainerProps {
  qualifiers: EventTypeQualifiers;
  objectQualifiers: ObjectTypeQualifiers;
  ocelInfo: OCELInfo;
}

function ConstraintContainer({
  qualifiers,
  objectQualifiers,
  ocelInfo,
}: ConstraintContainerProps) {
  const [info, setInfo] = useState<{
    objectVariables: ObjectVariable[];
  }>({ objectVariables: [] });
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
        <span className="text-lg text-gray-600">Add Object Variables</span>
        <div className="flex items-center gap-x-2 w-fit mx-auto mb-1">
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
              if (valWithCorrectCaps == null || valWithCorrectCaps === "") {
                return;
              }
              console.log({ valWithCorrectCaps }, ocelInfo.object_types);
              if (
                editMetaInfoData.name === "" ||
                editMetaInfoData.name.match(
                  new RegExp(
                    editMetaInfoData.type.toLowerCase().substring(0, 2) +
                      "_[0-9]$",
                  ),
                ) != null
              ) {
                let name =
                  valWithCorrectCaps.toLowerCase().substring(0, 2) + "_";
                for (let i = 0; i < 10; i++) {
                  if (
                    info.objectVariables.find((v) => v.name === name + i) ===
                    undefined
                  ) {
                    name = name + i;
                    break;
                  }
                }
                if (
                  editMetaInfoData.o2o != null &&
                  getPossibleO2O(valWithCorrectCaps).find(
                    (va) =>
                      va.parentVariableName ===
                        editMetaInfoData.o2o!.parentVariableName &&
                      va.qualifier === editMetaInfoData.o2o!.qualifier,
                  ) === undefined
                ) {
                  editMetaInfoData.o2o = undefined;
                }
                setEditMetaInfoData({
                  ...editMetaInfoData,
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
              { value: "Initially bound", label: "Initially bound" },
              { value: "Initially unbound", label: "Initially unbound" },
            ]}
            name="Initial Binding"
            onChange={(val) => {
              if (val != null && val !== "") {
                const initiallyBound = val === "Initially bound";
                setEditMetaInfoData({
                  ...editMetaInfoData,
                  initiallyBound,
                  o2o: initiallyBound ? editMetaInfoData.o2o : undefined,
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
              ...getPossibleO2O(editMetaInfoData.type).map((p) => ({
                value: JSON.stringify(p),
                label: p.qualifier + "@" + p.parentVariableName,
              })),
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
                setEditMetaInfoData({ ...editMetaInfoData, o2o: undefined });
              }
            }}
          />
          <Button
            disabled={
              editMetaInfoData.name === "" ||
              editMetaInfoData.type === "" ||
              info.objectVariables.findIndex(
                (c) => c.name === editMetaInfoData.name,
              ) > -1
            }
            variant="outline"
            size="icon"
            title="Add variable"
            onClick={() => {
              setInfo((cs) => {
                return {
                  ...cs,
                  objectVariables: [...cs.objectVariables, editMetaInfoData],
                };
              });
              // setEditMetaInfoData({ name: "", type: "" });
            }}
          >
            <RxPlus />
          </Button>
        </div>
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
                  : "\nO2O: -")
                  + (m.initiallyBound ? "\nInitially bound" : "\nInitially unbound")
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
              {!m.initiallyBound && <LuUnlink className="ml-2"/>}
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
                />
              </>
            )}
          </ConstraintInfoContext.Provider>
        </ReactFlowProvider>
      </div>
    </div>
  );
}

export default function VisualEditorOuter() {
  const [qualifiers, setQualifiers] = useState<EventTypeQualifiers>();
  const [objectQualifiers, setObjectQualifiers] =
    useState<ObjectTypeQualifiers>();
  const ocelInfo = useContext(OcelInfoContext);
  const [constraints, setConstraints] = useState<string[]>([]);
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

  return (
    <div>
      <Button
        className="mb-2"
        onClick={() => {
          setConstraints((cs) => [...cs, "new"]);
        }}
      >
        Add Constraints
      </Button>
      <div className="flex flex-wrap justify-around">
        {ocelInfo !== undefined &&
          qualifiers !== undefined &&
          objectQualifiers !== undefined &&
          constraints.map((_, i) => (
            <ConstraintContainer
              key={i}
              ocelInfo={ocelInfo}
              objectQualifiers={objectQualifiers}
              qualifiers={qualifiers}
            />
          ))}
      </div>
    </div>
  );
}
