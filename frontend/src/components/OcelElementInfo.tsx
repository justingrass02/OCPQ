import { useContext, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { BackendProviderContext } from "@/BackendProviderContext";
import type {
  OCELEvent,
  OCELObject,
  OCELRelationship,
  OCELType,
} from "@/types/ocel";
import JSONEditor from "@/components/JsonEditor";
import { OcelInfoContext } from "@/App";
import { IconForDataType } from "@/routes/ocel-info/OcelTypeViewer";
import { VisualEditorContext } from "@/routes/visual-editor/helper/VisualEditorContext";
import { Button } from "./ui/button";
import OcelGraphViewer from "@/routes/OcelGraphViewer";

export default function OcelElementInfo({
  type,
  req,
}: {
  type: "event" | "object";
  req: { id: string; index?: undefined } | { index: number; id?: undefined };
}) {
  const backend = useContext(BackendProviderContext);
  const [info, setInfo] = useState<
    | {
        index: number;
        object: OCELObject;
        event?: undefined;
      }
    | { index: number; event: OCELEvent; object?: undefined }
    | null
    | undefined
  >(undefined);
  useEffect(() => {
    if (type === "object" && req != null) {
      void backend["ocel/get-object"](req)
        .then((res) => {
          setInfo(res);
        })
        .catch(() => setInfo(null));
    } else if (type === "event" && req != null) {
      void backend["ocel/get-event"](req)
        .then((res) => {
          setInfo(res);
        })
        .catch(() => setInfo(null));
    }
  }, [req, type]);

  const ocelInfo = useContext(OcelInfoContext);

  return (
    <div className="text-lg text-left h-full">
      <div className="h-full grid grid-cols-2">
        <div className="h-full overflow-auto">
          {info?.object != null && (
            <OcelObjectViewer
              object={info.object}
              type={ocelInfo?.object_types.find(
                (t) => t.name === info.object.type,
              )}
            />
          )}
          {info?.event != null && (
            <OcelEventViewer
              event={info.event}
              type={ocelInfo?.event_types.find(
                (t) => t.name === info.event.type,
              )}
            />
          )}

          {info === null && (
            <div className="text-4xl font-bold text-red-700">Not Found</div>
          )}
          <div className="h-[60rem] mx-8">
            <OcelGraphViewer
              initialGrapOptions={{
                type,
                id: (info?.event ?? info?.object)?.id ?? req.id,
              }}
            />
          </div>
        </div>
        <div
          className={`block mx-2 p-1 mt-2 bg-white border rounded-lg shadow-md max-w-4xl text-left h-full`}
        >
          <h2 className="text-xl font-semibold my-1">JSON Representation</h2>
          <JSONEditor
            value={JSON.stringify(info?.event ?? info?.object ?? null, null, 2)}
            onChange={() => {}}
          />
        </div>
      </div>
    </div>
  );
}

function RelationshipViewer({ rels }: { rels?: OCELRelationship[] }) {
  const { showElementInfo } = useContext(VisualEditorContext);
  return (
    <div className="mt-4">
      Relationships
      <ul className="list-disc ml-6">
        {rels?.map((rel, i) => (
          <li key={i} className="my-1">
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                showElementInfo({ type: "object", req: { id: rel.objectId } });
              }}
            >
              {rel.objectId} @ {rel.qualifier}
            </Button>
          </li>
        ))}
      </ul>
    </div>
  );
}

function OcelObjectViewer({
  object,
  type,
}: {
  object: OCELObject;
  type?: OCELType;
}) {
  return (
    <div
      className={`block p-1 bg-white border rounded-lg shadow-md max-w-4xl text-left`}
    >
      <h4 className="font-semibold text-2xl">{object.id}</h4>
      <span className="text-gray-600 text-xl block mb-2">
        Type: {object.type}
      </span>
      <ul className="text-left text-xl space-y-1 ">
        {type?.attributes.map((attr) => (
          <li key={attr.name}>
            <div className="flex gap-x-1 items-center">
              <span className="flex justify-center -mt-1 w-8">
                {/* {attr.name} */}
                <IconForDataType dtype={attr.type} />
              </span>
              <span className="font-mono">{attr.name}:</span>{" "}
              <span className="font-mono text-blue-700">
                {object.attributes.find((a) => a.name === attr.name)?.value}
              </span>
            </div>
          </li>
        ))}
      </ul>
      <RelationshipViewer rels={object.relationships} />
    </div>
  );
}

function OcelEventViewer({
  event,
  type,
}: {
  event: OCELEvent;
  type?: OCELType;
}) {
  return (
    <div
      className={`block p-2 bg-white m-2 border rounded-lg shadow-md max-w-4xl text-left`}
    >
      <h4 className="font-semibold text-2xl">{event.id}</h4>
      <span className="text-gray-600 text-xl block mb-2">
        Type: {event.type}
      </span>
      <ul className="text-left text-xl space-y-1 ">
        {type?.attributes.map((attr) => (
          <li key={attr.name}>
            <div className="flex gap-x-1 items-center">
              <span className="flex justify-center -mt-1 w-8">
                {/* {attr.name} */}
                <IconForDataType dtype={attr.type} />
              </span>
              <span className="font-mono">{attr.name}:</span>{" "}
              <span className="font-mono text-blue-700">
                {event.attributes.find((a) => a.name === attr.name)?.value}
              </span>
            </div>
          </li>
        ))}
      </ul>
      <RelationshipViewer rels={event.relationships} />
    </div>
  );
}
