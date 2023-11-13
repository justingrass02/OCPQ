import type { EventTypeQualifierInfo, EventTypeQualifier } from "@/types/ocel";
import {
  Handle,
  Position,
  type NodeProps,
  useUpdateNodeInternals,
} from "reactflow";
import { buildHandleID, extractFromHandleID } from "./visual-editor-utils";
import { Fragment, useContext, useEffect, useRef } from "react";
import { VisualEditorContext } from "./visual-editor-context";

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

  const divRef = useRef<HTMLDivElement>(null);

  const updateNodeInternals = useUpdateNodeInternals();
  const { mode } = useContext(VisualEditorContext);
  const topDownHandles = mode === "view-tree";

  useEffect(() => {
    updateNodeInternals(id);
  }, [topDownHandles]);

  return (
    <>
      <div
        ref={divRef}
        className="border border-blue-200 shadow backdrop-blur flex flex-col pb-6 bg-blue-50/70 bg-blue-50 py-0.5 rounded-md min-w-[10rem] min-h-[10rem]"
        style={{ height: 4 + qualifiers.length * 2 + "rem" }}
      >
        <div className="h-[2rem] text-large font-semibold mx-4 flex justify-center items-center">
          <span>{id}</span>
        </div>
        <div className="h-full [&_.react-flow\_\_handle]:!relative">
          <div
            className={`grid grid-cols-[1fr_auto_1fr] justify-between w-full ${
              topDownHandles
                ? "rotate-90 absolute top-1/2 -translate-y-1/2 h-full reset-handles"
                : " h-full border-t border-t-blue-100"
            }`}
            style={{
              width:
                divRef.current !== null && topDownHandles
                  ? `${Math.floor(divRef.current?.clientHeight * 10) / 10}px`
                  : undefined,
              height:
                divRef.current !== null && topDownHandles
                  ? `${Math.floor(divRef.current?.clientWidth * 10) / 10}px`
                  : undefined,
            }}
          >
            {qualifiers.map((q, i) => (
              <Fragment key={i}>
                <div className={"flex justify-start"}>
                  <Handle
                    type="target"
                    id={buildHandleID(
                      "destination",
                      id,
                      q,
                      getObjectType(data.eventTypeQualifier[q]),
                    )}
                    isValidConnection={(connection) =>
                      mode === "normal" &&
                      connection.sourceHandle != null &&
                      connection.source !== id &&
                      extractFromHandleID(connection.sourceHandle)
                        .objectType ===
                        getObjectType(data.eventTypeQualifier[q])
                    }
                    onConnect={(params) => {
                      return params;
                    }}
                    position={topDownHandles ? Position.Top : Position.Left}
                    className="!border-none hover:brightness-125 [&>.react-flow__handle]:relative"
                    style={{
                      width: data.eventTypeQualifier[q].multiple
                        ? "0.66rem"
                        : "0.5rem",
                      height: data.eventTypeQualifier[q].multiple
                        ? "0.66rem"
                        : "0.5rem",
                      marginLeft: topDownHandles
                        ? data.eventTypeQualifier[q].multiple
                          ? "-0.3rem"
                          : "-0.2rem"
                        : data.eventTypeQualifier[q].multiple
                        ? "-0.125rem"
                        : undefined,
                      background:
                        data.objectTypeToColor[
                          getObjectType(data.eventTypeQualifier[q])
                        ],
                    }}
                  />
                </div>
                <div
                  className={`text-sm mx-auto flex flex-col text-center w-full ${
                    topDownHandles ? "max-w-[12ch] text-ellipsis" : ""
                  }`}
                >
                  {q}
                  <span className="text-gray-500 text-xs -mt-1">
                    {getObjectType(data.eventTypeQualifier[q])}
                    {data.eventTypeQualifier[q].multiple ? "*" : ""}
                  </span>
                </div>
                <div className="flex justify-end">
                  <Handle
                    type="source"
                    isValidConnection={(connection) =>
                      mode === "normal" &&
                      connection.targetHandle != null &&
                      connection.target !== id &&
                      extractFromHandleID(connection.targetHandle)
                        .objectType ===
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
                    position={topDownHandles ? Position.Bottom : Position.Right}
                    className="!border-none hover:brightness-125"
                    style={{
                      width: data.eventTypeQualifier[q].multiple
                        ? "0.66rem"
                        : "0.5rem",
                      height: data.eventTypeQualifier[q].multiple
                        ? "0.66rem"
                        : "0.5rem",
                      marginRight: topDownHandles
                        ? data.eventTypeQualifier[q].multiple
                          ? "-0.33rem"
                          : "-0.25rem"
                        : data.eventTypeQualifier[q].multiple
                        ? "-0.125rem"
                        : undefined,
                      background:
                        data.objectTypeToColor[
                          getObjectType(data.eventTypeQualifier[q])
                        ],
                    }}
                  />
                </div>
              </Fragment>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}
