import assert from "node:assert/strict";
import * as Json from "@frontend/Fixture_json";
export const ok = Json.ok;
export const isOk = Json.is_ok;
export const resultError = Json.result_error;
export function requireResult(result, message) {
  if (isOk(result)) return Json.value(result);
  const cause = resultError(result);
  throw new Error(message(cause), { cause });
}
export function assertResultError(result, message, pattern) {
  assert(!isOk(result), "operation must report a typed failure");
  if (pattern) assert.match(message(resultError(result)), pattern);
}
