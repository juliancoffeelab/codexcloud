# Sites operation transport

Use this recipe when invoking preparation or deployment. It is the host adapter
for `scripts/site-operation.mjs`, not a preliminary inspection step. Work from the
game root. This document owns the calling convention; the skill selects the mode.

- `prepare` returns an unpublished release receipt.
- `deploy` resolves the exact `HEAD` release, reusing an exact saved version or
  preparing it when necessary, then deploys that version and returns the result.

The module API is `runSiteOperation(tools, {mode, cwd, message?, preview?})`.
`preview: true` (CLI `--preview`) selects the committed `.openai/hosting.preview.json`
manifest and a separate release cache. The preview Site must differ from the main
Site; missing or identical preview configuration fails without falling back to main.
Plain `prepare`/`deploy` continues to use `.openai/hosting.json`.

`tools` supplies the enabled execution and native Sites functions by name. Internally,
`prepareSite(tools, {cwd, message?})` returns a receipt with `project_id`,
`version_id`, `source_commit`, `site_commit` and `owner_private` (plus
`version_number` when returned by the saving API);
`publishSite(tools, receipt)` returns `{url, deployed: true, source_commit,
version_id, deployment_id}`. The deployment operation returns
`{url, deployed: false, error, source_commit}` on failure when a known site address
is available. That address does not indicate a successful update. It throws a
failure when no address is available; callers must never fabricate one.
An operation failure is the end of that attempt, not an instruction to abandon
the deployment request. Follow the deploy skill's announced, focused repair and
retry workflow. A repaired source commit begins a new pinned attempt.
The transport forwards operation-owned tool calls unchanged. It never chooses
source, versions, audience, builds or retries on the agent’s behalf.

The command serializes concurrent runs, pins the implementation commit once,
and owns local subprocesses and native Sites calls. An exact cached release
needs only the current audience lookup and deployment call; no credential,
source synchronization, build, packaging or save is repeated. A cache miss
fetches audience and credentials concurrently, installs locked dependencies only
when needed, builds and packages once. Owner-private deployments use the native
combined save-and-deploy operation when available, avoiding a separate save/deploy
round trip; save-only requests never use it. Cache failures
do not block publication. A deleted saved version is rebuilt, never replaced
with an older release. Transient native failures retry the same operation once.
No tests, formatting, quality gates or post-deployment browser checks run here.

## Code-mode invocation

Run the following in `functions.exec`. Set `mode` to the skill’s operation and
`cwd` to the game checkout. The snippet relays JSON-lines messages until the
operation finishes; it does not read helper implementations. Transport polling
is distinct from deciding or polling deployment status, which the operation owns.

```js
const mode = "deploy"; // "prepare" for an unpublished release
const cwd = "/workspace/codexcloud/hex-command";
const preview = false; // true only for an explicitly requested preview deployment
let cell = await tools.exec_command({
  cmd: `node scripts/site-operation.mjs ${mode}${preview ? " --preview" : ""}`,
  workdir: cwd,
  tty: true,
  yield_time_ms: 1000,
  max_output_tokens: 6000,
});
const session = cell.session_id;
let buffer = "";
while (true) {
  buffer += cell.output;
  const requests = [];
  while (buffer.includes("\n")) {
    const end = buffer.indexOf("\n");
    const line = buffer.slice(0, end).trim();
    buffer = buffer.slice(end + 1);
    if (!line) continue;
    const message = JSON.parse(line);
    if (message.type === "result") {
      store("site-operation-result", message.result);
      text(message.result);
      exit();
    }
    if (message.type === "error") throw new Error(message.error);
    if (message.type !== "tool" || typeof message.name !== "string")
      throw new Error("Unsupported operation tool request");
    requests.push(message);
  }
  if (!session || cell.exit_code !== undefined)
    throw new Error("Site operation ended without a result");
  const responses = await Promise.all(
    requests.map(async (message) => {
      try {
        if (typeof tools[message.name] !== "function")
          throw new Error("Unavailable tool: " + message.name);
        return { id: message.id, result: await tools[message.name](message.args) };
      } catch (error) {
        return { id: message.id, error: String(error) };
      }
    }),
  );
  cell = await tools.write_stdin({
    session_id: session,
    chars: responses.length
      ? responses.map((response) => JSON.stringify(response)).join("\n") + "\n"
      : "",
    yield_time_ms: 1000,
    max_output_tokens: 6000,
  });
}
```

Treat protocol traffic as confidential: relay it in memory, without printing tool
requests/responses or saving them to files. Only the final receipt, URL or returned
failure is user-facing. Do not invoke the real operations to test this adapter;
use injected operation doubles for transport/sequence checks.
