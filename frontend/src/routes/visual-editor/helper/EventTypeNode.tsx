import { type EventTypeQualifier } from "@/types/ocel";
import { Handle, Position, type NodeProps } from "reactflow";

export default function EventTypeNode({
  data,
  id,
}: NodeProps<{
  label: string;
  eventTypeQualifier: EventTypeQualifier;
  objectTypeToColor: Record<string, string>;
}>) {
  const qualifiers = Object.keys(data.eventTypeQualifier).sort((a, b) =>
    a.localeCompare(b),
  );
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
                connection.sourceHandle?.split("===")[1] ===
                data.eventTypeQualifier[q].object_types[0]
              }
              onConnect={(params) => {
                if (
                  params.sourceHandle?.split("===")[1] ===
                  data.eventTypeQualifier[q].object_types[0]
                ) {
                  return params;
                }
              }}
              position={Position.Left}
              className="!w-2 !h-2 !-left-1 !border-none"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                background:
                  data.objectTypeToColor[
                    data.eventTypeQualifier[q].object_types[0]
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
                connection.targetHandle?.split("===")[1] ===
                data.eventTypeQualifier[q].object_types[0]
              }
              onConnect={(params) => {
                if (
                  params.targetHandle?.split("===")[1] ===
                  data.eventTypeQualifier[q].object_types[0]
                ) {
                  return params;
                }
              }}
              id={`${q}===${data.eventTypeQualifier[q].object_types[0]}`}
              position={Position.Right}
              className="!w-2 !h-2 !-right-1 !border-none"
              style={{
                top: `${5 + ((100 - 10) * i) / (qualifiers.length - 1)}%`,
                background:
                  data.objectTypeToColor[
                    data.eventTypeQualifier[q].object_types[0]
                  ],
              }}
            />
          ))}
        </div>
      </div>
    </>
  );
}
