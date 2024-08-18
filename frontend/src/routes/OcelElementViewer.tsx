import { useContext, useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
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

export default function OcelElementViewer() {
  const backend = useContext(BackendProviderContext);
  const [s] = useSearchParams();
  const id: string | null = s.get("id");
  const type: "object" | "event" | string | null = s.get("type");
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
    if (type === "object" && id != null) {
      void backend["ocel/get-object"]({ id })
        .then((res) => {
          setInfo(res);
        })
        .catch(() => setInfo(null));
    } else if (type === "event" && id != null) {
      void backend["ocel/get-event"]({ id })
        .then((res) => {
          setInfo(res);
        })
        .catch(() => setInfo(null));
    }
  }, [id, type]);

  const ocelInfo = useContext(OcelInfoContext);

  return (
    <div className="my-4 text-lg text-left h-full">
      <div className="bg-white py-4 px-2 my-4 mt-2 rounded-lg shadow border text-lg h-full">
        <h3 className="text-2xl font-semibold">
          {type === "event"
            ? "Event"
            : type === "object"
            ? "Object"
            : "Unknown Type"}{" "}
          {id}
        </h3>
        <br />
        {info === null && <div>Not Found</div>}
        {info?.object != null && (
          <div>
            <OcelObjectViewer
              object={info.object}
              type={ocelInfo?.object_types.find(
                (t) => t.name === info.object.type,
              )}
            />
          </div>
        )}
        {info?.event != null && (
          <div>
            <OcelEventViewer
              event={info.event}
              type={ocelInfo?.event_types.find(
                (t) => t.name === info.event.type,
              )}
            />
          </div>
        )}
        <h2 className="text-xl font-semibold mt-4">JSON Representation</h2>
        <JSONEditor
          value={JSON.stringify(info?.event ?? info?.object ?? null, null, 2)}
          onChange={() => {}}
        />
      </div>
    </div>
  );
}

function RelationshipViewer({ rels }: { rels?: OCELRelationship[] }) {
  return (
    <div className="mt-4">
      Relationships
      <ul className="list-disc ml-6">
        {rels?.map((rel, i) => (
          <li key={i}>
            {rel.objectId} @ {rel.qualifier}
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
      className={`block p-2 bg-white m-2 border rounded-lg shadow-md max-w-4xl text-left`}
    >
      <h4 className="font-semibold text-2xl">{object.id}</h4>
      <span className="text-gray-600 text-xl block mb-2">
        Type:{" "}
        <Link
          className="text-blue-800 underline"
          to={`/object-type-details/${object.type}`}
        >
          {object.type}
        </Link>
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
        Type:{" "}
        <Link
          className="text-blue-800 underline"
          to={`/event-type-details/${event.type}`}
        >
          {event.type}
        </Link>
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
