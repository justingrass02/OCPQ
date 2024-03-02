import {
  memo,
  useCallback,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import ReactFlow, {
  Background,
  Controls,
  MarkerType,
  Panel,
  addEdge,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  useReactFlow,
} from "reactflow";

import AlertHelper from "@/components/AlertHelper";
import { Button } from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
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
import { PiPlayFill } from "react-icons/pi";
import { TbLogicAnd, TbRestore, TbSquarePlus } from "react-icons/tb";
import "reactflow/dist/style.css";
import type { EventTypeQualifiers, OCELInfo } from "../../types/ocel";
import { evaluateConstraints } from "./evaluation/evaluate-constraints";
import { ConstraintInfoContext } from "./helper/ConstraintInfoContext";
import { FlowContext } from "./helper/FlowContext";
import { useLayoutedElements } from "./helper/LayoutFlow";
import { VisualEditorContext } from "./helper/VisualEditorContext";
import {
  EVENT_TYPE_LINK_TYPE,
  EVENT_TYPE_NODE_TYPE,
  GATE_LINK_TYPE,
  GATE_NODE_TYPE,
  edgeTypes,
  nodeTypes,
} from "./helper/const";
import {
  ALL_GATE_TYPES,
  type GateLinkData,
  type EventTypeLinkData,
  type EventTypeNodeData,
  type GateNodeData,
  type ViolationsPerNode,
  type ViolationsPerNodes,
} from "./helper/types";

interface VisualEditorProps {
  ocelInfo: OCELInfo;
  eventTypeQualifiers: EventTypeQualifiers;
  children?: ReactNode;
}

export default function VisualEditor(props: VisualEditorProps) {
  const [mode, setMode] = useState<"normal" | "view-tree" | "readonly">(
    "normal",
  );

  const { setInstance, registerOtherDataGetter, otherData, flushData } =
    useContext(FlowContext);

  const [nodes, setNodes, onNodesChange] = useNodesState<
    EventTypeNodeData | GateNodeData
  >(otherData?.nodes ?? []);

  const [edges, setEdges, onEdgesChange] = useEdgesState<
    EventTypeLinkData | GateLinkData
  >(otherData?.edges ?? []);
  const instance = useReactFlow();

  useEffect(() => {
    instance.setNodes(otherData?.nodes ?? nodes);
  }, [otherData?.nodes, otherData?.edges, instance]);

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
          const sourceNode = instance.getNode(source);
          const targetNode = instance.getNode(target);
          if (sourceNode?.type === "gate" || targetNode?.type === "gate") {
            // TODO: Implement link between gates and event nodes
            const color = "#969696";
            const newEdge: Edge<EventTypeLinkData> = {
              id: sourceHandle + "|||" + targetHandle,
              type: GATE_LINK_TYPE,
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
              },
            };
            return addEdge(newEdge, eds);
          }
          console.log(sourceHandle, targetHandle);
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
            },
          };
          return addEdge(newEdge, eds);
        }
      });
    },
    [setEdges],
  );

  const { getLayoutedElements } = useLayoutedElements();

  const [violationDetails, setViolationDetails] = useState<ViolationsPerNode>();

  const [violationInfo, setViolationInfo] = useState<{
    violationsPerNode?: ViolationsPerNodes;
    showViolationsFor?: (data: ViolationsPerNode) => unknown;
  }>({
    showViolationsFor: (d) => {
      console.log({ d });
      setViolationDetails(d);
    },
    violationsPerNode: otherData?.violations,
  });

  useEffect(() => {
    registerOtherDataGetter(() => ({
      violations: violationInfo.violationsPerNode,
      objectVariables,
    }));
  }, [violationInfo, objectVariables]);
  const initialized = useRef<boolean>(false);
  return (
    <VisualEditorContext.Provider
      value={{
        ocelInfo: props.ocelInfo,
        violationsPerNode: violationInfo.violationsPerNode,
        showViolationsFor: violationInfo.showViolationsFor,
        onNodeDataChange: (id, newData) => {
          setNodes((ns) => {
            const newNodes = [...ns];
            const changedNodeIndex = newNodes.findIndex((n) => n.id === id);
            if (newData === undefined) {
              newNodes.splice(changedNodeIndex, 1);
              return newNodes;
            }
            const changedNode = newNodes[changedNodeIndex];
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
        onEdgeDataChange: (id, newData) => {
          if (newData !== undefined) {
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
          } else {
            setEdges((edges) => {
              const newEdges = edges.filter((e) => e.id !== id);
              return newEdges;
            });
          }
        },
      }}
    >
      <ReactFlow
        onInit={(flow) => {
          initialized.current = true;
          if (initialized.current) {
            setInstance(flow);
          }
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
            disabled={mode !== "normal"}
            variant="outline"
            size="icon"
            title="Apply automatic layout"
            className="bg-white"
            onClick={() => {
              instance?.fitView();
              getLayoutedElements(
                {
                  "elk.algorithm": "layered",
                  "elk.direction": "DOWN",
                },
                true,
              );
              setTimeout(() => {
                instance?.fitView();
              }, 200);
            }}
          >
            <LuLayoutDashboard />
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
          {props.children}
          <AlertHelper
            // eslint-disable-next-line @typescript-eslint/consistent-type-assertions
            initialData={{ type: "not" } as GateNodeData}
            trigger={
              <Button
                disabled={mode !== "normal"}
                variant="outline"
                title="Add Gate"
                className="bg-white"
                onClick={() => {}}
              >
                <TbLogicAnd size={20} />
              </Button>
            }
            title={"Add Gate"}
            submitAction={"Submit"}
            onSubmit={(data, ev) => {
              setNodes((nodes) => {
                const center =
                  instance != null
                    ? instance.screenToFlowPosition({
                        x: window.innerWidth / 2,
                        y: window.innerHeight / 2,
                      })
                    : { x: 0, y: 0 };
                return [
                  ...nodes,
                  {
                    id: "gate" + Date.now(),
                    type: GATE_NODE_TYPE,
                    position: center,
                    data: {
                      type: data.type,
                    },
                  },
                ];
              });
            }}
            content={({ data, setData }) => {
              const sortedOcelEventTypes = [...props.ocelInfo.event_types];
              sortedOcelEventTypes.sort((a, b) => a.name.localeCompare(b.name));
              return (
                <>
                  <p className="mb-2">
                    Please select the type of gate to add below.
                  </p>
                  <Combobox
                    value={data.type}
                    onChange={(v) => {
                      setData({ ...data, type: v as GateNodeData["type"] });
                    }}
                    name="Gate Type"
                    options={ALL_GATE_TYPES.map((t) => ({
                      label: t,
                      value: t,
                    }))}
                  ></Combobox>
                </>
              );
            }}
          />
          <AlertHelper
            initialData={{ eventType: "" }}
            trigger={
              <Button
                disabled={mode !== "normal"}
                variant="outline"
                title="Add Event Node"
                className="bg-white"
                onClick={() => {}}
              >
                <TbSquarePlus size={20} />
              </Button>
            }
            title={"Add Event Node"}
            submitAction={"Submit"}
            onSubmit={(data, ev) => {
              if (data.eventType === "") {
                toast("Please select an event type to add a node.");
                ev.preventDefault();
                return;
              }
              setNodes((nodes) => {
                const center =
                  instance != null
                    ? instance.screenToFlowPosition({
                        x: window.innerWidth / 2,
                        y: window.innerHeight / 2,
                      })
                    : { x: 0, y: 0 };
                return [
                  ...nodes,
                  {
                    id: data.eventType + Date.now(),
                    type: EVENT_TYPE_NODE_TYPE,
                    position: center,
                    data: {
                      eventType: { type: "exactly", value: data.eventType },
                      eventTypeQualifier:
                        props.eventTypeQualifiers[data.eventType],
                      selectedVariables: [],
                      countConstraint: { min: 0, max: Infinity },
                    },
                  },
                ];
              });
              console.log({ data });
            }}
            content={({ data, setData }) => {
              const sortedOcelEventTypes = [...props.ocelInfo.event_types];
              sortedOcelEventTypes.sort((a, b) => a.name.localeCompare(b.name));
              return (
                <>
                  <p className="mb-2">
                    Please select the node event type to add below.
                  </p>
                  <Combobox
                    value={data.eventType}
                    onChange={(v) => {
                      setData({ ...data, eventType: v });
                    }}
                    name="Event Type"
                    options={sortedOcelEventTypes.map((o) => ({
                      value: o.name,
                      label: o.name,
                    }))}
                  ></Combobox>
                </>
              );
            }}
          />
          <Button
            variant="outline"
            title={mode !== "view-tree" ? "Evaluate" : "Edit"}
            className="bg-fuchsia-100 border-fuchsia-300 hover:bg-fuchsia-200 hover:border-fuchsia-300"
            onClick={async () => {
              const res = await evaluateConstraints(
                objectVariables,
                nodes,
                edges,
              );
              setViolationInfo((vi) => ({ ...vi, violationsPerNode: res }));
              flushData({ violations: res, objectVariables });
            }}
          >
            {mode !== "view-tree" && (
              <PiPlayFill size={20} className="text-fuchsia-700" />
            )}
            {mode === "view-tree" && <TbRestore />}
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
    </VisualEditorContext.Provider>
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
