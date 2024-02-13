const INFINITY_STRINGS = ["infty", "infinity", "inf", "∞"];
export function parseIntAllowInfinity(s: string, allowMinusInf = false) {
  if (INFINITY_STRINGS.includes(s)) {
    return Infinity;
  } else if (
    allowMinusInf &&
    s.startsWith("-") &&
    INFINITY_STRINGS.includes(s.substring(1))
  ) {
    return -Infinity;
  } else {
    const num = parseInt(s);
    if (isNaN(num)) {
      return undefined;
    } else {
      return num;
    }
  }
}
