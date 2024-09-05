import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import {
  memo,
  useState,
  useEffect,
  useContext,
  useMemo,
  Suspense,
  lazy,
} from "react";
import type { BindingBoxTreeNode } from "@/types/generated/BindingBoxTreeNode";
import type { EvaluationRes, EvaluationResPerNodes } from "./helper/types";
import { VisualEditorContext } from "./helper/VisualEditorContext";
import { columnsForBinding } from "@/components/binding-table/columns";
import Spinner from "@/components/Spinner";
import { Button } from "@/components/ui/button";
import  type PaginatedBindingTable from "@/components/binding-table/PaginatedBindingTable";
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

  const [mode, setMode] = useState<
    "violations" | "situations" | "satisfied-situations"
  >(initialMode ?? "violations");
  useEffect(() => {
    if (initialMode !== undefined) {
      setMode(initialMode);
    }
  }, [initialMode]);
  const { showElementInfo } = useContext(VisualEditorContext);
  const [appliedCutoff, setAppliedCutoff] = useState<number|undefined>(DEFAULT_CUTOFF);
  const items = useMemo(() => {
    return violationDetails.situations.slice(0,appliedCutoff)
  },[appliedCutoff]);

  const numBindings = violationDetails.situationCount;
  const numViolations = violationDetails.situationViolatedCount;


  const columns = useMemo(() => {
    return columnsForBinding(
      items[0][0],
      violationResPerNodes.objectIds,
      violationResPerNodes.eventIds,
      showElementInfo,
      node,
    );
  }, []);

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
              {mode === "situations"
                ? "Situations"
                : mode === "violations"
                ? "Violations"
                : "Satisfied Situations"}{" "}
            </SheetTitle>
            <SheetDescription asChild>
              <div>
              <p className="text-primary text-base">
                {numBindings} Bindings
                <br />
                {numViolations} Violations
              </p>
              {numBindings > DEFAULT_CUTOFF && (
                <div className="flex items-center gap-x-2">
                  {appliedCutoff !== undefined ? `For performance reasons, only the first ${DEFAULT_CUTOFF} output bindings are shown.` : "All output bindings are shown."}
                  <Button size="sm" variant="ghost" onClick={() => {
                    if(appliedCutoff !== undefined){
                      setAppliedCutoff(undefined);
                    }else{
                      setAppliedCutoff(DEFAULT_CUTOFF)
                    }
                  }}>{appliedCutoff !== undefined ? "Show All" : "Undo"}</Button>
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
