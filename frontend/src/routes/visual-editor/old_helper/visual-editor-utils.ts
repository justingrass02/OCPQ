export const HANDLE_ID_INFO_SEPERATOR = "@@@";
export function buildHandleID(
  type: "source" | "destination",
  eventType: string,
  qualifier: string,
  objectType: string,
) {
  return (
    type +
    HANDLE_ID_INFO_SEPERATOR +
    eventType +
    HANDLE_ID_INFO_SEPERATOR +
    qualifier +
    HANDLE_ID_INFO_SEPERATOR +
    objectType
  );
}
export function extractFromHandleID(handleID: string): {
  type: "source" | "destination";
  eventType: string;
  qualifier: string;
  objectType: string;
} {
  const info = handleID.split(HANDLE_ID_INFO_SEPERATOR);
  if (info.length !== 4) {
    throw new Error(
      "Handle ID does not contain required information: " + handleID,
    );
  }
  return {
    type: info[0] as "source" | "destination",
    eventType: info[1],
    qualifier: info[2],
    objectType: info[3],
  };
}
