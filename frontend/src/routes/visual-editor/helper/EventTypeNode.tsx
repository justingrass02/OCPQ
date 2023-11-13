import type { EventTypeQualifierInfo, EventTypeQualifier } from "@/types/ocel";
import { Handle, Position, type NodeProps } from "reactflow";
import { buildHandleID, extractFromHandleID } from "./visual-editor-utils";

export type EventTypeNodeData = {
  label: string;
  eventTypeQualifier: EventTypeQualifier;
  objectTypeToColor: Record<string, string>;
};

function getObjectType(qualInfo: EventTypeQualifierInfo) {
  if (qualInfo.object_types.length > 1) {
    console.warn(
      "Warning: Encountered multiple object types. This is currently not supported",
    );
  }
  return qualInfo.object_types[0];
}
export default function EventTypeNode({
  data,
  id,
}: NodeProps<EventTypeNodeData>) {
  // Sort by object type
  const qualifiers = Object.keys(data.eventTypeQualifier).sort((a, b) =>
    getObjectType(data.eventTypeQualifier[a]).localeCompare(
      getObjectType(data.eventTypeQualifier[b]),
    ),
  );
  return (
    <>
      <div
        className="border border-blue-500 shadow flex flex-col pb-6 bg-blue-50 py-0.5 rounded-md min-w-[10rem]"
        style={{ height: 4 + qualifiers.length * 2 + "rem" }}
      >
        <div className="h-[2rem] text-large font-semibold mx-4 flex justify-center items-center">
          <span>{id}</span>
        </div>
        <div className="flex flex-col relative h-full w-full border-t border-t-blue-500">
          {qualifiers.map((q, i) => (
            <Handle
              type="target"
              key={i}
              id={buildHandleID(
                "destination",
                id,
                q,
                getObjectType(data.eventTypeQualifier[q]),
              )}
              isValidConnection={(connection) =>
                connection.sourceHandle != null &&
                connection.source !== id &&
                extractFromHandleID(connection.sourceHandle).objectType ===
                  getObjectType(data.eventTypeQualifier[q])
              }
              onConnect={(params) => {
                return params;
              }}
              position={Position.Left}
              className="!border-none hover:brightness-125"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                width: data.eventTypeQualifier[q].multiple
                  ? "0.66rem"
                  : "0.5rem",
                height: data.eventTypeQualifier[q].multiple
                  ? "0.66rem"
                  : "0.5rem",
                left: data.eventTypeQualifier[q].multiple
                  ? "-0.33rem"
                  : "-0.25rem",
                background:
                  data.objectTypeToColor[
                    getObjectType(data.eventTypeQualifier[q])
                  ],
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
                {getObjectType(data.eventTypeQualifier[q])}
                {data.eventTypeQualifier[q].multiple ? "*" : ""}
              </span>
            </div>
          ))}
          {qualifiers.map((q, i) => (
            <Handle
              key={i}
              type="source"
              isValidConnection={(connection) =>
                connection.targetHandle != null &&
                connection.target !== id &&
                extractFromHandleID(connection.targetHandle).objectType ===
                  getObjectType(data.eventTypeQualifier[q])
              }
              onConnect={(params) => {
                return params;
              }}
              id={buildHandleID(
                "source",
                id,
                q,
                getObjectType(data.eventTypeQualifier[q]),
              )}
              position={Position.Right}
              className="!border-none hover:brightness-125"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                width: data.eventTypeQualifier[q].multiple
                  ? "0.66rem"
                  : "0.5rem",
                height: data.eventTypeQualifier[q].multiple
                  ? "0.66rem"
                  : "0.5rem",
                right: data.eventTypeQualifier[q].multiple
                  ? "-0.33rem"
                  : "-0.25rem",
                background:
                  data.objectTypeToColor[
                    getObjectType(data.eventTypeQualifier[q])
                  ],
              }}
            />
          ))}
        </div>
      </div>
    </>
  );
}
