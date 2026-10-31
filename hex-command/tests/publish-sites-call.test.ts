import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "vitest";

const source = readFileSync(new URL("../scripts/publish-sites-call.js", import.meta.url), "utf8");
const publish = new Function(
  source
    .replace("export default async function", "async function")
    .replace("export async function", "async function") + "; return publishSite;",
)() as (
  tools: Record<string, (input: Record<string, unknown>) => Promise<unknown>>,
  options: { project_id: string; version_id: string; owner_private: boolean },
) => Promise<{ url: string }>;

for (const succeeded of [true, false]) {
  test(`Saved-release deployment ${succeeded ? "succeeds" : "fails"} without preparation or Git`, async () => {
    const calls: unknown[] = [];
    const tools = new Proxy(
      {},
      {
        get(_, name) {
          assert.equal(name, "mcp__codex_apps__sites_deploy_private_site_version");
          return async (input: unknown) => {
            calls.push(input);
            return {
              structuredContent: {
                id: "deployment",
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
