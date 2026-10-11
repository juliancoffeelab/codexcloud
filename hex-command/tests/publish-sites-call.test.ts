import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "vitest";

const source = readFileSync(new URL("../scripts/publish-sites-call.js", import.meta.url), "utf8");
const publish = new Function("return (" + source.replace("export default ", "") + ")")() as (
  tools: Record<string, (input: Record<string, unknown>) => Promise<unknown>>,
  options: { cwd: string },
) => Promise<{ source_commit: string }>;
const sourceCommit = "1234567890123456789012345678901234567890";
const url = "https://example.test";
const structured = (value: unknown) => ({ structuredContent: value });

function fixture(deployed = true, pushed = true) {
  const events: string[] = [];
  const tools = {
    async exec_command(input: Record<string, unknown>) {
      const cmd = String(input.cmd);
      if (cmd === "cat .openai/hosting.json")
        return { exit_code: 0, output: '{"project_id":"site"}' };
      if (cmd.startsWith("npm run publish"))
        return { session_id: 1, output: "Ready for Site credential JSON\n" };
      events.push(cmd);
      return { exit_code: pushed ? 0 : 1, output: "push result" };
    },
    async write_stdin() {
      events.push("package");
      return {
        exit_code: 0,
        output:
          "SITE_PACKAGE: " +
          JSON.stringify({
            project_id: "site",
            commit_sha: "site-source",
            archive: {},
            source_commit: sourceCommit,
            source_branch: "main",
          }),
      };
    },
    async mcp__codex_apps__sites_get_site() {
      return structured({
        current_user_role: "owner",
        access_policy: {
          access_mode: "custom",
          allowed_users: [{ role: "owner" }],
        },
      });
    },
    async mcp__codex_apps__sites_create_source_repository_write_credential() {
      return structured({ token: "test-only" });
    },
    async mcp__codex_apps__sites_save_version_and_deploy_private() {
      events.push(deployed ? "deployed" : "failed");
      return structured({
        id: "deployment",
        status: deployed ? "succeeded" : "failed",
        url,
        failure_message: "build failed",
      });
    },
  };
  return { tools, events };
}

test("Publication pushes exactly the deployed GitHub commit after Sites succeeds", async () => {
  const { tools, events } = fixture();
  const result = await publish(tools, { cwd: "/project" });
  assert.equal(result.source_commit, sourceCommit);
  assert.deepEqual(events, [
    "package",
    "deployed",
    `git push origin '${sourceCommit}:refs/heads/main'`,
  ]);
});

test("A failed deployment cannot push the unpublished commit to GitHub", async () => {
  const { tools, events } = fixture(false);
  await assert.rejects(publish(tools, { cwd: "/project" }), /Deployment deployment failed/);
  assert.deepEqual(events, ["package", "failed"]);
});

test("A post-deployment push failure reports the live URL and deployment for recovery", async () => {
  const { tools } = fixture(true, false);
  await assert.rejects(
    publish(tools, { cwd: "/project" }),
    (error: Error) =>
      error.message.includes(url) &&
      error.message.includes("deployment") &&
      error.message.includes("GitHub push failed"),
  );
});
