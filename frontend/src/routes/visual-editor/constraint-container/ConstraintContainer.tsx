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
    <div className="relative">
      <div className="w-[50rem] xl:w-[70rem] h-[50rem] border p-2">
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
