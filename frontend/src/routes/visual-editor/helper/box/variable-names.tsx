import { useContext } from "react";
import { VisualEditorContext } from "../VisualEditorContext";
import { MdEvent } from "react-icons/md";
import { LuBox } from "react-icons/lu";

export function getEvVarName(eventVar: number) {
  const { getVarName } = useContext(VisualEditorContext);
  const varInfo = getVarName(eventVar, "event");
  return (
    <span className="font-mono font-semibold" style={{ color: varInfo.color }}>
      <MdEvent className="inline-block -mr-1.5" /> {varInfo.name}
    </span>
  );
}

export function getObVarName(obVar: number) {
  const { getVarName } = useContext(VisualEditorContext);
  const varInfo = getVarName(obVar, "object");
  return (
    <span className="font-mono font-semibold" style={{ color: varInfo.color }}>
      <LuBox className="inline-block -mr-1.5" /> {varInfo.name}
    </span>
  );
}
