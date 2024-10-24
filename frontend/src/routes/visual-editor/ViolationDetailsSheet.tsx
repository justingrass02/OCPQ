import { BackendProviderContext } from "@/BackendProviderContext";
import { columnsForBinding } from "@/components/binding-table/columns";
import type PaginatedBindingTable from "@/components/binding-table/PaginatedBindingTable";
import Spinner from "@/components/Spinner";
import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { downloadURL } from "@/lib/download-url";
import type { BindingBoxTreeNode } from "@/types/generated/BindingBoxTreeNode";
import {
  Suspense,
  lazy,
  memo,
  useContext,
  useMemo,
  useState
} from "react";
import toast from "react-hot-toast";
import { TbTableExport } from "react-icons/tb";
import type { EvaluationRes, EvaluationResPerNodes } from "./helper/types";
import { VisualEditorContext } from "./helper/VisualEditorContext";
const DataTablePaginationLazy = lazy(
  async () => await import("@/components/binding-table/PaginatedBindingTable"),
) as typeof PaginatedBindingTable;

const DEFAULT_CUTOFF = 20_000;
const ViolationDetailsSheet = memo(function ViolationDetailsSheet({
  violationDetails,
  violationResPerNodes,
  reset,
  initialMode,
  node,
}: {
  violationDetails: EvaluationRes;
  violationResPerNodes: EvaluationResPerNodes;
  initialMode: "violations" | "situations" | "satisfied-situations" | undefined;
  node: BindingBoxTreeNode;
  reset: () => unknown;
}) {
  const backend = useContext(BackendProviderContext);
  const hasConstraints =
    "Box" in node ? node.Box[0].constraints.length > 0 : true;

  const { showElementInfo } = useContext(VisualEditorContext);
  const [appliedCutoff, setAppliedCutoff] = useState<number | undefined>(
    DEFAULT_CUTOFF,
  );
  const items = useMemo(() => {
    return violationDetails.situations.slice(0, appliedCutoff);
  }, [appliedCutoff, violationDetails, node]);

  const numBindings = violationDetails.situationCount;
  const numViolations = violationDetails.situationViolatedCount;

  const columns = useMemo(() => {
    return columnsForBinding(
      items[0][0],
      violationResPerNodes.objectIds,
      violationResPerNodes.eventIds,
      showElementInfo,
      node,
      hasConstraints,
    );
  }, [violationResPerNodes, node]);

  return (
    <Sheet
      modal={false}
      open={violationDetails !== undefined}
      onOpenChange={(o) => {
        if (!o) {
          reset();
          showElementInfo(undefined);
        }
      }}
    >
      {violationDetails !== undefined && (
        <SheetContent
          side="left"
          className="h-screen flex flex-col w-[50vw] min-w-fit"
          overlay={false}
          onInteractOutside={(ev) => {
            ev.preventDefault();
          }}
        >
          <SheetHeader>
            <SheetTitle className="flex items-center justify-between pr-4">
              Output Bindings
            </SheetTitle>
            <SheetDescription asChild>
              <div>
                <div className="flex justify-between">
                  <p className="text-primary text-base">
                    {numBindings} Bindings
                    <br />
                    {numViolations} Violations
                  </p>
                  <Button size="icon" variant="outline" onClick={() => {
                    toast.promise(backend["ocel/export-bindings-csv"](violationDetails), { loading: "Exporting to CSV...", error: (e) => <p>Failed to export to CSV!<br />{String(e)}</p>, success: "Finished CSV Export!" }).then((res) => {
                      console.log({ res });
                      if(res){
                        const url = URL.createObjectURL(res);
                        downloadURL(url,"situation-table.csv");
                        URL.revokeObjectURL(url);
                      }
                    }).catch(err => console.error(err))
                  }}>
                    <TbTableExport />
                  </Button>
                </div>
                {numBindings > DEFAULT_CUTOFF && (
                  <div className="flex items-center gap-x-2">
                    {appliedCutoff !== undefined
                      ? `For performance reasons, only the first ${DEFAULT_CUTOFF} output bindings are shown.`
                      : "All output bindings are shown."}
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        if (appliedCutoff !== undefined) {
                          setAppliedCutoff(undefined);
                        } else {
                          setAppliedCutoff(DEFAULT_CUTOFF);
                        }
                      }}
                    >
                      {appliedCutoff !== undefined ? "Show All" : "Undo"}
                    </Button>
                  </div>
                )}
              </div>
            </SheetDescription>
          </SheetHeader>

          {items.length > 0 && (
            <Suspense
              fallback={
                <div className="flex items-center gap-x-2">
                  Loading binding table... <Spinner />
                </div>
              }
            >
              <DataTablePaginationLazy
                key={JSON.stringify(node)}
                columns={columns}
                data={items}
                initialMode={initialMode}
              />
            </Suspense>
          )}
        </SheetContent>
      )}
    </Sheet>
  );
});
export default ViolationDetailsSheet;
