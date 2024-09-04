import {
  EvVarName,
  ObVarName,
} from "@/routes/visual-editor/helper/box/variable-names";
import type { Binding } from "@/types/generated/Binding";
import type { ColumnDef } from "@tanstack/react-table";
import { Link } from "react-router-dom";

export function columnsForBinding(
  binding: Binding,
  objectIds: string[],
  eventIds: string[],
  showElementInfo: (
    elInfo:
      | { req: { id: string } | { index: number }; type: "object" | "event" }
      | undefined,
  ) => unknown,
): ColumnDef<Binding>[] {
  return [
    ...Object.entries(binding.objectMap).map(
      ([obVarName, _obIndex]) =>
        ({
          id: "o" + (parseInt(obVarName) + 1),
          cell: (c) => (
            <Link
              to={{
                pathname: "/ocel-element",
                search: `?id=${encodeURIComponent(
                  c.getValue<string>(),
                )}&type=object`,
              }}
              target="_blank"
              onClick={(ev) => {
                ev.preventDefault();
                showElementInfo({
                  type: "object",
                  req: { id: c.getValue<string>() },
                });
              }}
              rel="noreferrer"
              className="max-w-[16ch] align-top whitespace-nowrap inline-block text-ellipsis overflow-hidden underline decoration decoration-blue-500/60 hover:decoration-blue-500"
            >
              {c.getValue<string>()}
            </Link>
          ),
          header: () => <ObVarName obVar={parseInt(obVarName)} />,
          accessorFn: (b) => objectIds[b.objectMap[parseInt(obVarName)]],
        }) satisfies ColumnDef<Binding>,
    ),
    ...Object.entries(binding.eventMap).map(
      ([evVarName, _evIndex]) =>
        ({
          id: "e" + (parseInt(evVarName) + 1),
          cell: (c) => (
            <Link
              to={{
                pathname: "/ocel-element",
                search: `?id=${encodeURIComponent(
                  c.getValue<string>(),
                )}&type=object`,
              }}
              target="_blank"
              onClick={(ev) => {
                ev.preventDefault();
                showElementInfo({
                  type: "object",
                  req: { id: c.getValue<string>() },
                });
              }}
              rel="noreferrer"
              className="max-w-[16ch] align-top whitespace-nowrap inline-block text-ellipsis overflow-hidden underline decoration decoration-blue-500/60 hover:decoration-blue-500"
            >
              {c.getValue<string>()}
            </Link>
          ),
          header: () => <EvVarName eventVar={parseInt(evVarName)} />,
          accessorFn: (b) => eventIds[b.eventMap[parseInt(evVarName)]],
        }) satisfies ColumnDef<Binding>,
    ),
    // {
    //   accessorFn: (x) => {
    //     return x.
    //   },
    //   header: "Status",
    // },
    // {
    //   accessorKey: "email",
    //   header: "Email",
    // },
    // {
    //   accessorKey: "amount",
    //   header: "Amount",
    // },
  ];
}
