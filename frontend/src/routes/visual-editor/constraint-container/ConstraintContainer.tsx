import type {
  EventTypeQualifiers,
  ObjectTypeQualifiers,
  OCELInfo,
} from "@/types/ocel";
import { ReactFlowProvider } from "reactflow";
import VisualEditor from "../VisualEditor";

interface ConstraintContainerProps {
  qualifiers: EventTypeQualifiers;
  objectQualifiers: ObjectTypeQualifiers;
  ocelInfo: OCELInfo;
}

export default function ConstraintContainer({
  qualifiers,
  ocelInfo,
}: ConstraintContainerProps) {
  return (
    <div className="relative w-full h-full px-12">
      <div className="xl:w-full min-h-[50rem] h-full border p-2">
        <ReactFlowProvider>
          {qualifiers !== undefined && ocelInfo !== undefined && (
            <>
              <VisualEditor
                eventTypeQualifiers={qualifiers}
                ocelInfo={ocelInfo}
              ></VisualEditor>
            </>
          )}
        </ReactFlowProvider>
      </div>
    </div>
  );
}
