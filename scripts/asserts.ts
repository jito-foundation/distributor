// Minimal assertion helpers used by the smoke script.
export function assert(condition: any, msg = "assertion failed"): asserts condition {
  if (!condition) {
    throw new Error(msg);
  }
}

export function assertEqual<T>(a: T, b: T, msg = "values are not equal") {
  if (a !== b) {
    throw new Error(`${msg}: ${a} !== ${b}`);
  }
}
