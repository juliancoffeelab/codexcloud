import "./native-runtime.mjs";
import assert from "node:assert/strict";
import { test } from "vitest";
import * as Domain from "@frontend/Domain";
import { toJSON } from "./json-fixture.mjs";

test("Custom catalogs are parsed only in builds with the runtime-catalog capability", () => {
  const result = Domain.NativeEngineFactory.with_documents(Domain.Text_map.empty);
  assert.equal(result.TAG, 1);
  const error = toJSON("EngineError", result._0);
  if (process.env.HEX_COMMAND_PROFILE === "dev") {
    assert.equal(error.code, "content");
    assert.equal(error.path, "manifest.json");
    assert.match(error.message, /missing authored document/);
  } else {
    assert.deepEqual(error, {
      code: "invalidRequest",
      message: "Custom catalogs require a runtime-catalog build",
    });
  }
});
