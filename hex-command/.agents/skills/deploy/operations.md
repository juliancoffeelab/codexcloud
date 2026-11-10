# Sites operation transport

Use this recipe to invoke `node scripts/site-operation.mjs deploy` from the game
root. It is the host adapter, not a preliminary inspection step. The operation
resolves the exact `HEAD` release, reusing its saved version or preparing it when
necessary, then deploys that version and returns the result.

The module API is `runSiteOperation(tools, {mode, cwd, message?, preview?})`.
`preview` is the retained first-branch tag (`--preview fix/first-branch`). Keep it
for the chat, including after moving to another branch or PR. Numeric keys still
address existing PR previews. Invalid or missing tags fail without falling back
to main. The destination is independent of the current committed source.

First use creates an owner-private Site and persists its opaque identity locally.
The deterministic slug derives from the retained tag and source Site identity,
allowing rediscovery from another checkout. Existing readable slugs stay stable;
when the complete slug would exceed 63 characters, the operation uses
`hex-<12-character-preview-hash>-<8-character-source-site-hash>`. Keep the full
original tag in `--preview`; the operation owns hashing, and the title and
description retain the branch name. Creation races/timeouts reconcile through
discovery. Later deployments with that tag reuse the same Site and URL.
Plain `deploy` uses the committed `.openai/hosting.json` only for an explicitly
requested live/main deployment.

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

The command serializes each tag's save/deploy sequence. A separate build lock
protects one shared, pinned-source build checkout across all destinations. Site
Git histories remain separate. Committed input fingerprints and output digests
reuse the binding tool, WASM, frontend and artwork independently; missing or
changed outputs rebuild their stage. A new revision gets a bundle containing its
source commit. An exact saved release skips source preparation entirely.

The operation relays safe stage progress and elapsed build times. Forward those
messages through the transport below; they distinguish compilation from hosting
publication. Source credentials remain in memory. Owner-private Sites use the
combined save-and-deploy operation when available. Transient native failures retry
the same operation once; deleted versions rebuild the requested release. Tests,
formatting and quality gates are outside this operation.

## Code-mode invocation

Run the following in `functions.exec`, with `cwd` set to the game checkout. The snippet relays JSON-lines messages until the
operation finishes; it does not read helper implementations. Transport polling
is distinct from deciding or polling deployment status, which the operation owns.

```js
const mode = "deploy";
const cwd = "/workspace/codexcloud/hex-command";
const preview = "fix/first-branch"; // retain the first branch tag for this chat; undefined only for explicit live/main deployment
const previewArg =
  preview === undefined ? "" : " --preview '" + String(preview).replaceAll("'", "'\\''") + "'";
let cell = await tools.exec_command({
  cmd: `node scripts/site-operation.mjs ${mode}${previewArg}`,
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
    if (message.type === "progress") {
      notify(message.message);
      continue;
    }
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
