# Sites operation transport

Use this recipe to invoke `node scripts/site-operation.mjs deploy` from the game
root. It is the host adapter, not a preliminary inspection step. The operation
resolves the exact `HEAD` release, reusing its saved version or preparing it when
necessary, then deploys that version and returns the result.

The module API is `runSiteOperation(tools, {mode, cwd, message?, preview?})`.
`preview` is a positive PR number (CLI `--preview 59`). Bare `--preview`, booleans,
zero and invalid numbers fail without falling back to main. The selected number
chooses a destination; the release still uses the current committed HEAD.

Each PR has its own Site, release cache and build checkout. First use provisions
an owner-private Site and immediately persists its opaque ID in an ignored local
hosting manifest. A stable slug scoped to the main Site and PR allows another
checkout to rediscover the same identity. Creation races/timeouts are reconciled
through discovery rather than blindly creating another Site. Known identities
are checked against the requested PR before any source push.

Default to a numbered PR preview; plain `deploy` uses the committed
`.openai/hosting.json` only for an explicitly requested live/main deployment.
A numbered preview operation leaves the main Site and other PR previews alone.

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
round trip. Cache failures do not block publication. A deleted saved version is rebuilt, never replaced
with an older release. Transient native failures retry the same operation once.
No tests, formatting, quality gates or post-deployment browser checks run here.

## Code-mode invocation

Run the following in `functions.exec`, with `cwd` set to the game checkout. The snippet relays JSON-lines messages until the
operation finishes; it does not read helper implementations. Transport polling
is distinct from deciding or polling deployment status, which the operation owns.

```js
const mode = "deploy";
const cwd = "/workspace/codexcloud/hex-command";
const preview = 59; // replace with the current PR number; undefined only for explicit live/main deployment
let cell = await tools.exec_command({
  cmd: `node scripts/site-operation.mjs ${mode}${preview === undefined ? "" : ` --preview ${preview}`}`,
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
