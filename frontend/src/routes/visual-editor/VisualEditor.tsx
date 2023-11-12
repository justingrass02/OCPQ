import { useCallback, useContext, useEffect, useState } from "react";
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
import { LuLayoutDashboard } from "react-icons/lu";
import "reactflow/dist/style.css";
import type { EventTypeQualifiers, OCELInfo } from "../../types/ocel";
import ConnectionLine from "./helper/ConnectionLine";
import EventLink from "./helper/EventLink";
import EventTypeNode from "./helper/EventTypeNode";
import { useLayoutedElements } from "./helper/LayoutFlow";

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
const edgeTypes = { eventLink: EventLink };

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
    }),
  );
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  const onConnect = useCallback(
    (params: Edge | Connection) => {
      setEdges((eds) => {
        const color = objectTypeToColor[params.sourceHandle!.split("===")[1]];
        return addEdge(
          {
            ...params,
            type: "eventLink",
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
            data: {
              color,
              onDelete: (id: string) => {
                setEdges((edges) => {
                  const newEdges = edges.filter((e) => e.id !== id);
                  return newEdges;
                });
              },
            },
          },
          eds,
        );
      });
    },
    [setEdges],
  );
  const { getLayoutedElements } = useLayoutedElements();

  return (
    <ReactFlow
      edgeTypes={edgeTypes}
      nodeTypes={nodeTypes}
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      proOptions={{ hideAttribution: true }}
      connectionLineComponent={(props) => (
        <ConnectionLine {...props} objectTypeToColor={objectTypeToColor} />
      )}
      fitView
    >
      <Controls />
      <Panel position="top-right">
        <Button
          variant="outline"
          size="icon"
          title="Apply automatic layout"
          onClick={() => {
            getLayoutedElements({
              "elk.algorithm": "layered",
              "elk.direction": "RIGHT",
            });
          }}
        >
          <LuLayoutDashboard />
        </Button>
      </Panel>
      <Background />
    </ReactFlow>
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
      {qualifiers !== undefined && ocelInfo !== undefined && (
        <>
          <VisualEditor eventTypeQualifiers={qualifiers} ocelInfo={ocelInfo} />
        </>
      )}{" "}
    </ReactFlowProvider>
  );
}
