# Sites operation transport

Use this recipe when invoking preparation or deployment. It is the host adapter
for `scripts/site-operation.mjs`, not a preliminary inspection step. Work from the
game root. This document owns the calling convention; the skill selects the mode.

- `prepare` returns an unpublished release receipt.
- `deploy` unconditionally runs preparation, passes the exact returned receipt to
  deployment, waits for its terminal result and returns `url`.

The module API is `runSiteOperation(tools, {mode, cwd, message?})`.
`tools` supplies the enabled execution and native Sites functions by name. Internally,
`prepareSite(tools, {cwd, message?})` returns a receipt with `project_id`,
`version_id`, `version_number`, `source_commit`, `site_commit` and `owner_private`;
`publishSite(tools, receipt)` returns `{url, version_id, deployment_id}`.
The transport forwards operation-owned tool calls unchanged. It never chooses
source, versions, audience, builds or retries on the agent’s behalf.

## Code-mode invocation

Run the following in `functions.exec`. Set `mode` to the skill’s operation and
`cwd` to the game checkout. The snippet relays JSON-lines messages until the
operation finishes; it does not read helper implementations. Transport polling
is distinct from deciding or polling deployment status, which the operation owns.

```js
const mode = "deploy"; // "prepare" for an unpublished release
const cwd = "/workspace/codexcloud/hex-command";
let cell = await tools.exec_command({
  cmd: `node scripts/site-operation.mjs ${mode}`,
  workdir: cwd,
  tty: true,
  yield_time_ms: 1000,
  max_output_tokens: 6000,
});
const session = cell.session_id;
let buffer = "";
while (true) {
  buffer += cell.output;
  let response;
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
    if (message.type !== "tool" || typeof tools[message.name] !== "function")
      throw new Error("Unsupported operation tool request");
    try {
      response = { id: message.id, result: await tools[message.name](message.args) };
    } catch (error) {
      response = { id: message.id, error: String(error) };
    }
  }
  if (!session || cell.exit_code !== undefined)
    throw new Error("Site operation ended without a result");
  cell = await tools.write_stdin({
    session_id: session,
    chars: response ? JSON.stringify(response) + "\n" : "",
    yield_time_ms: 1000,
    max_output_tokens: 6000,
  });
}
```

Treat protocol traffic as confidential: relay it in memory, without printing tool
requests/responses or saving them to files. Only the final receipt, URL or returned
failure is user-facing. Do not invoke the real operations to test this adapter;
use injected operation doubles for transport/sequence checks.
