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
} from "reactflow";

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
import { LuLayoutDashboard } from "react-icons/lu";
import { RxPlus, RxReset } from "react-icons/rx";
import { TbBinaryTree, TbRestore } from "react-icons/tb";
import "reactflow/dist/style.css";
import type { EventTypeQualifiers, OCELInfo } from "../../types/ocel";
import { evaluateConstraints } from "./evaluation/evaluate-constraints";
import { ConstraintInfoContext } from "./helper/ConstraintInfoContext";
import EventTypeLink, {
  EVENT_TYPE_LINK_TYPE,
  type EventTypeLinkData,
} from "./helper/EventTypeLink";
import EventTypeNode, { type EventTypeNodeData } from "./helper/EventTypeNode";
import { useLayoutedElements } from "./helper/LayoutFlow";
import {
  ViolationsContext,
  type ViolationsContextValue,
} from "./helper/ViolationsContext";
import type { ObjectVariable, ViolationsPerNode } from "./helper/types";

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
const nodeTypes = { eventType: EventTypeNode };
const edgeTypes = {
  [EVENT_TYPE_LINK_TYPE]: EventTypeLink,
};

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

  const [nodes, setNodes, onNodesChange] = useNodesState<EventTypeNodeData>(
    Object.keys(props.eventTypeQualifiers).map((eventType) => {
      return {
        id: eventType,
        type: "eventType",
        position: { x: 0, y: 0 },
        data: {
          label: eventType,
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
  );

  const [edges, setEdges, onEdgesChange] = useEdgesState<EventTypeLinkData>([]);

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
          <Button
            disabled={edges.length === 0 || mode !== "normal"}
            variant="outline"
            size="icon"
            title="Reset edges"
            className="text-red-600 bg-white hover:bg-red-400"
            onClick={() => {
              setEdges([]);
            }}
          >
            <RxReset />
          </Button>

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
              const res = await evaluateConstraints(nodes, edges);
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
            </SheetDescription>
          </SheetHeader>
          <ul className="overflow-auto h-[80vh] bg-slate-50 border rounded-sm mt-2 px-2 py-0.5 text-xs">
            {violationDetails.violations.map(([[info, binding], reason], i) => (
              <li
                key={i}
                className="border mx-1 my-2 px-1 py-1 rounded-sm bg-blue-50"
              >
                <div>
                  <span className="text-emerald-700">Past events:</span>{" "}
                  <span className="font-mono">
                    {info.past_events.join(",")}
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
  ocelInfo: OCELInfo;
}

function ConstraintContainer({
  qualifiers,
  ocelInfo,
}: ConstraintContainerProps) {
  const [info, setInfo] = useState<{
    objectVariables: ObjectVariable[];
  }>({ objectVariables: [] });
  const [editMetaInfoData, setEditMetaInfoData] = useState<ObjectVariable>({
    name: "",
    type: "",
  });

  return (
    <div className="relative">
      <div>
        <div className="flex items-center gap-x-2 w-fit mx-auto mb-1">
          <Combobox
            value={editMetaInfoData.type}
            options={ocelInfo.object_types.map((ot) => ({
              value: ot.name,
              label: ot.name,
            }))}
            name="Object type"
            onChange={(val) => {
              if (
                editMetaInfoData.name === "" ||
                editMetaInfoData.name.match(
                  new RegExp(editMetaInfoData.type.substring(0, 2) + "_[0-9]$"),
                ) != null
              ) {
                let name = val.substring(0, 2) + "_";
                for (let i = 0; i < 10; i++) {
                  if (
                    info.objectVariables.find((v) => v.name === name + i) ===
                    undefined
                  ) {
                    name = name + i;
                    break;
                  }
                }
                setEditMetaInfoData({ ...editMetaInfoData, type: val, name });
              } else {
                setEditMetaInfoData({ ...editMetaInfoData, type: val });
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
        <div className="flex flex-wrap divide-x-2 absolute mt-1 z-10">
          {info.objectVariables.map((m, i) => (
            <div className="text-center px-2" key={i} title={m.type}>
              {m.name}
            </div>
          ))}
        </div>
      </div>
      <div className="w-[50rem] h-[50rem] border p-2">
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
  const ocelInfo = useContext(OcelInfoContext);
  const [constraints, setConstraints] = useState<string[]>([]);
  useEffect(() => {
    toast
      .promise(
        fetch("http://localhost:3000/ocel/qualifiers", { method: "get" }),
        {
          loading: "Fetching qualifier info...",
          success: "Loaded qualifier info",
          error: "Failed to fetch qualifier info",
        },
        { id: "fetch-qualifiers" },
      )
      .then(async (res) => {
        const json: EventTypeQualifiers = await res.json();
        setQualifiers(json);
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
        Add...
      </Button>
      <div className="flex flex-wrap justify-between">
        {ocelInfo !== undefined &&
          qualifiers !== undefined &&
          constraints.map((_, i) => (
            <ConstraintContainer
              key={i}
              ocelInfo={ocelInfo}
              qualifiers={qualifiers}
            />
          ))}
      </div>
    </div>
  );
}
