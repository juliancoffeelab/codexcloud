import assert from "node:assert/strict";
import { test } from "vitest";
import publish from "../scripts/publish-sites-call.js";

for (const succeeded of [true, false]) {
  test(`Saved-release deployment ${succeeded ? "succeeds" : "fails"} without preparation or Git`, async () => {
    const calls = [];
    const tools = new Proxy(
      {},
      {
        get(_, name) {
          assert.equal(name, "mcp__codex_apps__sites_deploy_private_site_version");
          return async (input) => {
            calls.push(input);
            return {
              structuredContent: {
                id: "deployment",
                version_id: "prepared",
                status: succeeded ? "succeeded" : "failed",
                url: "https://example.test",
                failure_message: "hosting failed",
              },
            };
          };
        },
      },
    );
    const result = publish(tools, {
      project_id: "site",
      version_id: "prepared",
      owner_private: true,
    });
    if (succeeded) assert.equal((await result).url, "https://example.test");
    else await assert.rejects(result, /Deployment deployment failed: hosting failed/);
    assert.deepEqual(calls, [{ project_id: "site", version_id: "prepared" }]);
  });
}
