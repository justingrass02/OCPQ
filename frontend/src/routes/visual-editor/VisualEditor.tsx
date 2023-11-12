import { useCallback, useContext, useEffect, useState } from "react";
import ReactFlow, {
  Background,
  type Connection,
  Controls,
  type Edge,
  Handle,
  MarkerType,
  MiniMap,
  type NodeProps,
  Panel,
  Position,
  ReactFlowProvider,
  addEdge,
  useEdgesState,
  useNodesState,
} from "reactflow";

import "reactflow/dist/style.css";
import { type OCELInfo } from "../../types/ocel";
import ConnectionLine from "./helper/ConnectionLine";
import { useLayoutedElements } from "./helper/LayoutFlow";
import { Button } from "@/components/ui/button";
import { OcelInfoContext } from "@/App";

export type EventTypeQualifier = Record<
  string,
  { qualifier: string; counts: number[]; multiple: boolean; object_types: string[] }
>;

export type EventTypeQualifiers = Record<string, EventTypeQualifier>;

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

function VisualEditor(props: VisualEditorProps) {
  const objectTypeToColor: Record<string, string> = {};

  props.ocelInfo.object_types.forEach((type, i) => {
    objectTypeToColor[type.name] = COLORS[i % COLORS.length];
  });

  const [nodes, setNodes, onNodesChange] = useNodesState(
    Object.keys(props.eventTypeQualifiers).map((eventType) => {
      return {
        id: eventType,
        type: "eventType",
        position: { x: 0, y: 0 },
        data: {
          label: eventType,
          eventTypeQualifier: props.eventTypeQualifiers[eventType],
          objectTypeToColor,
        },
      };
    })
  );
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const onConnect = useCallback(
    (params: Edge | Connection) => {
      setEdges((eds) => {
        const color = objectTypeToColor[params.sourceHandle!.split("===")[1]];
        return addEdge(
          {
            ...params,
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
            label: params.sourceHandle!.split("===")[1].substring(0, 1) + "1",
          },
          eds
        );
      });
    },
    [setEdges]
  );
  const { getLayoutedElements } = useLayoutedElements();

  return (
    <ReactFlow
      nodeTypes={nodeTypes}
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      proOptions={{ hideAttribution: true }}
      connectionLineComponent={(props) => <ConnectionLine {...props} objectTypeToColor={objectTypeToColor} />}
      fitView
    >
      <MiniMap />
      <Controls />
      <Panel position="top-right">
        <Button
          onClick={() => {
            getLayoutedElements({ "elk.algorithm": "layered", "elk.direction": "RIGHT" });
          }}
        >
          horizontal layout
        </Button>
      </Panel>
      <Background />
    </ReactFlow>
  );
}

function EventTypeNode({
  data,
  id,
}: NodeProps<{ label: string; eventTypeQualifier: EventTypeQualifier; objectTypeToColor: Record<string, string> }>) {
  const qualifiers = Object.keys(data.eventTypeQualifier);
  return (
    <>
      <div
        className="border border-blue-500 shadow flex flex-col pb-6 bg-blue-50 py-0.5 rounded-md"
        style={{ height: 4 + qualifiers.length * 2 + "rem" }}
      >
        <div className="h-[2rem] text-large font-semibold mx-4 flex items-center">
          <span>{id}</span>
        </div>
        <div className="flex flex-col relative h-full w-full border-t border-t-blue-500">
          {qualifiers.map((q, i) => (
            <Handle
              type="target"
              key={i}
              id={`${q}===${data.eventTypeQualifier[q].object_types[0]}`}
              isValidConnection={(connection) =>
                connection.sourceHandle?.split("===")[1] === data.eventTypeQualifier[q].object_types[0]
              }
              onConnect={(params) => {
                if (params.sourceHandle?.split("===")[1] === data.eventTypeQualifier[q].object_types[0]) {
                  return params;
                }
              }}
              position={Position.Left}
              className="!w-2 !h-2 !-left-1 !border-none"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                background: data.objectTypeToColor[data.eventTypeQualifier[q].object_types[0]],
              }}
            />
          ))}
          {qualifiers.map((q, i) => (
            <div
              className="absolute text-sm mx-auto flex flex-col text-center w-full"
              key={q}
              style={{ top: `${((100 - 10) * i) / (qualifiers.length - 1)}%` }}
            >
              {q}
              <span className="text-gray-500 text-xs -mt-1">
                {data.eventTypeQualifier[q].object_types[0]}
                {data.eventTypeQualifier[q].multiple ? "*" : ""}
              </span>
            </div>
          ))}
          {qualifiers.map((q, i) => (
            <Handle
              key={i}
              type="source"
              isValidConnection={(connection) =>
                connection.targetHandle?.split("===")[1] === data.eventTypeQualifier[q].object_types[0]
              }
              onConnect={(params) => {
                if (params.targetHandle?.split("===")[1] === data.eventTypeQualifier[q].object_types[0]) {
                  return params;
                }
              }}
              id={`${q}===${data.eventTypeQualifier[q].object_types[0]}`}
              position={Position.Right}
              className="!w-2 !h-2 !-right-1 !border-none"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                background: data.objectTypeToColor[data.eventTypeQualifier[q].object_types[0]],
              }}
            />
          ))}
        </div>
      </div>
    </>
  );
}

export default function VisualEditorOuter() {
  const [qualifiers, setQualifiers] = useState<EventTypeQualifiers>();
  const ocelInfo = useContext(OcelInfoContext);
  useEffect(() => {
    fetch("http://localhost:3000/ocel/qualifiers", { method: "get" })
    .then(async (res) => {
      const json: EventTypeQualifiers = await res.json();
      setQualifiers(json);
      console.log(json);
    })
    .catch((e) => {
      console.error(e);
    });
  }, []);

  return (
    <ReactFlowProvider>
      {qualifiers && ocelInfo && (
        <><p>Test</p><VisualEditor
          eventTypeQualifiers={qualifiers}
          ocelInfo={ocelInfo} /></>
      )}{" "}
    </ReactFlowProvider>
  );
}
