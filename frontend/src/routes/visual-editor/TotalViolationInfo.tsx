import clsx from "clsx";
import type { EvaluationResPerNodes } from "./helper/types";
import {
  getViolationStyles,
  getViolationTextColor,
} from "./helper/violation-styles";
import { PiPlayFill } from "react-icons/pi";

export default function TotalViolationInfo({
  violations,
}: {
  violations: EvaluationResPerNodes | undefined;
}) {
  const [situationViolatedCount, situationCount] = Object.values(
    violations?.evalRes ?? {},
  ).reduce(
    ([violationCount, situationCount], val) => [
      violationCount + val.situationViolatedCount,
      situationCount + val.situationCount,
    ],
    [0, 0],
  );
  const percentage = (100 * situationViolatedCount) / situationCount;

  return (
    <div
      className={clsx(
        "rounded w-full h-[3.5rem]",
        isNaN(percentage) && "text-gray-700",
        !isNaN(percentage) && "font-bold border-2",
        !isNaN(percentage) &&
          getViolationStyles({ situationViolatedCount, situationCount }),
        !isNaN(percentage) &&
          getViolationTextColor({ situationViolatedCount, situationCount }),
      )}
    >
      {!isNaN(percentage) && (
        <>{Math.round(100 * percentage) / 100}% ⌀ Violations </>
      )}
      {isNaN(percentage) && (
        <span className="text-sm">No evaluation result available</span>
      )}
      <br />
      {!isNaN(percentage) && (
        <>
          ({situationViolatedCount} of {situationCount})
        </>
      )}
      {isNaN(percentage) && (
        <span className="inline-flex items-center gap-x-1 text-xs">
          Evaluate using the <PiPlayFill className="text-purple-600" /> button
          below
        </span>
      )}
    </div>
  );
}
