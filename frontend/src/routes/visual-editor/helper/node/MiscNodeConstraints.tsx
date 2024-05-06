import type { EventTypeNodeData } from "../types";

interface MiscNodeConstraintsProps {
  id: string;
  data: EventTypeNodeData;
  onNodeDataChange: (
    id: string,
    newData: Partial<EventTypeNodeData> | undefined,
  ) => unknown;
}
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export default function MiscNodeConstraints(props: MiscNodeConstraintsProps) {
  return <div className="flex gap-x-2">{}</div>;
}
