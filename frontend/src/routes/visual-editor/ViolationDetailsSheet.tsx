import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import { memo, useRef, useState, useEffect, useContext } from "react";
import { Link } from "react-router-dom";
import AutoSizer from "react-virtualized-auto-sizer";
import { FilterOrConstraintDisplay } from "./helper/box/FilterOrConstraintEditor";
import { EvVarName, ObVarName } from "./helper/box/variable-names";
import type { Binding } from "@/types/generated/Binding";
import { VariableSizeList, type ListChildComponentProps } from "react-window";
import type { BindingBoxTreeNode } from "@/types/generated/BindingBoxTreeNode";
import type { ViolationReason } from "@/types/generated/ViolationReason";
import type { EvaluationRes, EvaluationResPerNodes } from "./helper/types";
import { VisualEditorContext } from "./helper/VisualEditorContext";

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
  const varRef = useRef<VariableSizeList>(null);
  function getItemHeight([binding, reason]: [Binding, ViolationReason | null]) {
    return (
      8 +
      (3 +
        (reason === null ? 0 : 1) +
        Object.keys(binding.eventMap).length +
        Object.keys(binding.objectMap).length) *
        24
    );
  }
  const [mode, setMode] = useState<
    "violations" | "situations" | "satisfied-situations"
  >(initialMode ?? "violations");
  useEffect(() => {
    if (initialMode !== undefined) {
      setMode(initialMode);
    }
  }, [initialMode]);
  const { showElementInfo } = useContext(VisualEditorContext);
  const items =
    mode === "violations"
      ? violationDetails.situations.filter(
          ([_binding, reason]) => reason !== null,
        )
      : mode === "satisfied-situations"
      ? violationDetails.situations.filter(
          ([_binding, reason]) => reason === null,
        )
      : violationDetails.situations;
  useEffect(() => {
    varRef.current?.resetAfterIndex(0);
  }, [mode, items]);
  const Row = ({ index, style }: ListChildComponentProps) => {
    const [binding, reason] = items[index];
    return (
      <div className="pb-2 h-full" style={style} key={index}>
        <div className="h-full border px-1 py-1 rounded-sm bg-blue-50 text-lg">
          <div>
            {reason !== null && (
              <div className="text-red-500 h-6 block">
                {typeof reason === "string" && reason}
                {typeof reason === "object" &&
                  "TooFewMatchingEvents" in reason &&
                  `TooFewMatchingEvents (#${reason.TooFewMatchingEvents})`}
                {typeof reason === "object" &&
                  "TooManyMatchingEvents" in reason &&
                  `TooManyMatchingEvents (#${reason.TooManyMatchingEvents})`}
                {/* {typeof reason === "object" &&
                  "ConstraintNotSatisfied" in reason &&
                  `ConstraintNotSatisfied (at index ${reason.ConstraintNotSatisfied})`} */}
                {
                  typeof reason === "object" &&
                    "Box" in node &&
                    "ConstraintNotSatisfied" in reason && (
                      <div className="flex items-center gap-x-2 justify-between mx-0 font-medium tracking-tighter flex-nowrap whitespace-nowrap pr-2">
                        Violated Constraint
                        <div className="max-w-[7.66rem]">
                          <FilterOrConstraintDisplay
                            compact={true}
                            value={
                              node.Box[0].constraints[
                                reason.ConstraintNotSatisfied
                              ]
                            }
                          />
                        </div>
                        <div className="text-xs">
                          @{reason.ConstraintNotSatisfied}
                        </div>
                      </div>
                    )
                  // `ConstraintNotSatisfied (at index ${})`
                }
              </div>
            )}
            <span className="text-emerald-700 font-bold h-6 block">
              Events:
            </span>{" "}
            <ul className="flex flex-col ml-6 list-disc">
              {Object.entries(binding.eventMap).map(([evVarName, evIndex]) => (
                <li key={evVarName} className="h-6">
                  <EvVarName eventVar={parseInt(evVarName)} />:{" "}
                  <Link
                    to={{
                      pathname: "/ocel-element",
                      search: `?id=${encodeURIComponent(
                        violationResPerNodes.eventIds[evIndex],
                      )}&type=event`,
                    }}
                    target="_blank"
                    onClick={(ev) => {
                      ev.preventDefault();
                      showElementInfo({
                        type: "event",
                        req: { id: violationResPerNodes.eventIds[evIndex] },
                      });
                    }}
                    rel="noreferrer"
                    className="max-w-[16ch] align-top whitespace-nowrap inline-block text-ellipsis overflow-hidden underline decoration decoration-blue-500/60 hover:decoration-blue-500"
                    title={violationResPerNodes.eventIds[evIndex]}
                    onDoubleClick={(ev) => {
                      const range = document.createRange();
                      range.selectNodeContents(ev.currentTarget);
                      const selection = window.getSelection();
                      if (selection != null) {
                        selection.removeAllRanges();
                        selection.addRange(range);
                      }
                    }}
                  >
                    {violationResPerNodes.eventIds[evIndex]}
                  </Link>
                </li>
              ))}
            </ul>
            <h3 className="text-blue-700 font-bold h-6 block">Objects:</h3>
            <ul className="flex flex-col ml-6 list-disc">
              {Object.entries(binding.objectMap).map(([obVarName, obIndex]) => (
                <li key={obVarName} className="h-6">
                  <ObVarName obVar={parseInt(obVarName)} />:{" "}
                  <Link
                    to={{
                      pathname: "/ocel-element",
                      search: `?id=${encodeURIComponent(
                        violationResPerNodes.objectIds[obIndex],
                      )}&type=object`,
                    }}
                    target="_blank"
                    onClick={(ev) => {
                      ev.preventDefault();
                      showElementInfo({
                        type: "object",
                        req: { id: violationResPerNodes.objectIds[obIndex] },
                      });
                    }}
                    rel="noreferrer"
                    className="max-w-[16ch] align-top whitespace-nowrap inline-block text-ellipsis overflow-hidden underline decoration decoration-blue-500/60 hover:decoration-blue-500"
                    title={violationResPerNodes.objectIds[obIndex]}
                    onDoubleClick={(ev) => {
                      const range = document.createRange();
                      range.selectNodeContents(ev.currentTarget);
                      const selection = window.getSelection();
                      if (selection != null) {
                        selection.removeAllRanges();
                        selection.addRange(range);
                      }
                    }}
                  >
                    {violationResPerNodes.objectIds[obIndex]}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    );
  };
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
          className="h-screen flex flex-col"
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
              <Button
                variant="outline"
                onClick={() => {
                  if (mode === "violations") {
                    setMode("situations");
                  } else if (mode === "situations") {
                    setMode("satisfied-situations");
                  } else {
                    setMode("violations");
                  }
                }}
              >
                Show{" "}
                {mode === "violations"
                  ? "All Situations"
                  : mode === "situations"
                  ? "Satisfied Situations"
                  : "Only Violations"}
              </Button>
            </SheetTitle>
            <SheetDescription>
              {mode === "violations"
                ? violationDetails.situationViolatedCount
                : violationDetails.situationCount}{" "}
              {mode === "situations" ? "Situations" : "Violations"}
            </SheetDescription>
          </SheetHeader>
          <ul className="overflow-auto h-full bg-slate-50 border rounded-sm mt-2 px-2 py-0.5 text-xs">
            <AutoSizer>
              {({ height, width }) => (
                <VariableSizeList
                  key={mode + items.length}
                  itemCount={items.length}
                  itemSize={(i) => getItemHeight(items[i])}
                  // estimatedItemSize={getItemHeight(items[0])}
                  width={width}
                  height={height}
                  ref={varRef}
                >
                  {Row}
                </VariableSizeList>
              )}
            </AutoSizer>
          </ul>
        </SheetContent>
      )}
    </Sheet>
  );
});
export default ViolationDetailsSheet;
