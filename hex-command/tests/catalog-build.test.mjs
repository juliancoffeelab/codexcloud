import "./native-runtime.mjs";
import assert from "node:assert/strict";
import { test } from "vitest";
import * as Domain from "@frontend/Domain";
import { toJSON } from "./json-fixture.mjs";

test("The baked browser build requires an explicit runtime-catalog feature for custom documents", () => {
  const result = Domain.NativeEngineFactory.with_documents(Domain.String_map.empty);
  assert.equal(result.TAG, 1);
  assert.deepEqual(toJSON("EngineError", result._0), {
    code: "invalidRequest",
    message: "Custom catalogs require a runtime-catalog build",
  });
});
