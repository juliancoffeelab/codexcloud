/** Native Sites orchestration. Preparation and deployment are separate operations. */
import { readFileSync, writeFileSync, mkdirSync, renameSync, unlinkSync } from "node:fs";
import { execFileSync, spawn } from "node:child_process";
import { join } from "node:path";

const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function native(tools, name, args) {
  try {
    return decode(await tools[name](args));
  } catch (error) {
    if (!/timeout|timed out|ECONNRESET|temporar|\b429\b|\b50[234]\b/i.test(error.message))
      throw error;
    // Retry the same operation/identity once, never a different or older release.
    await pause(250);
    return decode(await tools[name](args));
  }
}

function cachedReceipt(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    // This optional performance cache must never gate publication.
    return undefined;
  }
}

function buildSource(cwd, sourceCommit, credential, message) {
  return new Promise((resolve, reject) => {
    const child = spawn(
      "python3",
      ["scripts/prepare-site.py", "--source-commit", sourceCommit, "--message", message],
      { cwd, stdio: ["pipe", "pipe", "pipe"] },
    );
    let output = "";
    child.stdout.on("data", (data) => {
      output += data;
    });
    child.stderr.on("data", (data) => {
      output += data;
    });
    child.on("error", reject);
    child.stdin.on("error", reject);
    child.on("close", (code) => {
      if (code !== 0) return reject(new Error(output || `Preparation exited with ${code}`));
      const line = output.split(/\r?\n/).findLast((line) => line.startsWith("SITE_PACKAGE: "));
      if (!line) return reject(new Error("Preparation returned no package"));
      try {
        resolve(JSON.parse(line.slice("SITE_PACKAGE: ".length)));
      } catch (error) {
        reject(error);
      }
    });
    child.stdin.end(JSON.stringify(credential) + "\n");
  });
}
function decode(result) {
  if (result.isError)
    throw new Error(
      (result.content ?? [])
        .filter((i) => i.type === "text")
        .map((i) => i.text)
        .join("\n"),
    );
  if (result.structuredContent) return result.structuredContent;
  for (const item of result.content ?? []) {
    if (item.type === "text") {
      try {
        return JSON.parse(item.text);
      } catch {}
    }
  }
  throw new Error("Sites returned no structured result");
}
function ownerPrivate(site) {
  const policy = site.access_policy;
  return (
    site.current_user_role === "owner" &&
    policy?.access_mode === "custom" &&
    policy.allowed_users?.length === 1 &&
    policy.allowed_users[0].role === "owner" &&
    !policy.external_visitor_count &&
    !policy.allowed_groups?.length &&
    !policy.allowed_editors?.length &&
    !policy.allowed_workspace_group_ids?.length &&
    !policy.allowed_tenant_group_ids?.length
  );
}

/** Resolve the pinned commit to its exact saved release; build only on a cache miss. */
export async function prepareSite(tools, options, progress = () => {}) {
  const cwd = options.cwd;
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const source_commit = options.source_commit ?? git("rev-parse", "HEAD");
  const { project_id } = JSON.parse(git("show", `${source_commit}:./.openai/hosting.json`));
  const cachePath = join(cwd, ".cache", "site-release.json");
  const stored = cachedReceipt(cachePath);
  const cached = stored?.project_id === project_id ? stored : undefined;
  const exact =
    !options.refresh &&
    cached?.source_commit === source_commit &&
    cached.project_id === project_id &&
    cached.version_id &&
    cached.site_commit;
  let site;
  try {
    // Audience is current even when the already-saved build is reused.
    const siteRequest = native(tools, "mcp__codex_apps__sites_get_site", { project_id });
    const credentialRequest = exact
      ? undefined
      : native(tools, "mcp__codex_apps__sites_create_source_repository_write_credential", {
          project_id,
          publish_on_push: null,
        });
    const [siteResult, credentialResult] = await Promise.allSettled([
      siteRequest,
      credentialRequest,
    ]);
    if (siteResult.status === "fulfilled") site = siteResult.value;
    else throw siteResult.reason;
    if (credentialResult.status === "rejected") throw credentialResult.reason;
    const live_url = site.current_live_url ?? site.expected_url ?? cached?.live_url;
    if (exact) return { ...cached, owner_private: ownerPrivate(site), live_url, reused: true };
    const credential = await credentialRequest;
    if (
      options.mode !== "deploy" &&
      (credential.publish_on_push_accepted ||
        Date.parse(credential.publish_on_push_expires_at) > Date.now())
    )
      throw new Error(
        "An unpublished release cannot push during an active automatic publication window",
      );
    progress("Building and synchronizing the requested source.");
    const prepared = await buildSource(
      cwd,
      source_commit,
      credential,
      options.message ?? "Prepare Hex Command release",
    );
    if (prepared.project_id !== project_id || prepared.source_commit !== source_commit)
      throw new Error("Preparation returned a different source or Site");
    let version;
    let deployment;
    if (credential.publish_on_push_accepted) {
      // Reconcile the exact automatically published source, never an older/latest version.
      const deadline =
        Math.max(Date.now(), Date.parse(credential.publish_on_push_expires_at) || Date.now()) +
        30_000;
      do {
        const versions = await native(tools, "mcp__codex_apps__sites_list_site_versions", {
          project_id,
        });
        version = versions.items.find((entry) => entry.source.commit_sha === prepared.commit_sha);
        if (!version?.deployment_id) {
          if (Date.now() >= deadline)
            throw new Error("Automatic publication did not produce the requested release");
          await pause(1000);
        }
      } while (!version?.deployment_id);
    } else if (
      options.mode === "deploy" &&
      ownerPrivate(site) &&
      typeof tools.mcp__codex_apps__sites_save_version_and_deploy_private === "function"
    ) {
      try {
        deployment = await native(tools, "mcp__codex_apps__sites_save_version_and_deploy_private", {
          project_id,
          commit_sha: prepared.commit_sha,
          archive: prepared.archive,
        });
        version = { id: deployment.version_id, source: { commit_sha: prepared.commit_sha } };
      } catch (error) {
        if (!/unavailable tool|unknown tool|unsupported tool/i.test(error.message)) throw error;
      }
    }
    if (!version) {
      version = await native(tools, "mcp__codex_apps__sites_save_site_version", {
        project_id,
        commit_sha: prepared.commit_sha,
        archive: prepared.archive,
      });
    }
    if (version.source.commit_sha !== prepared.commit_sha)
      throw new Error("Saved version does not match the built source");
    try {
      unlinkSync(prepared.archive);
    } catch {
      // The native save has finished ingesting the archive; cleanup is optional.
    }
    const receipt = {
      project_id,
      version_id: version.id,
      version_number: version.version_number,
      source_commit,
      site_commit: prepared.commit_sha,
      owner_private: ownerPrivate(site),
      live_url,
      ...(deployment ? { deployment } : {}),
      ...(credential.publish_on_push_accepted && version.deployment_id
        ? { deployment_id: version.deployment_id }
        : {}),
    };
    try {
      mkdirSync(join(cwd, ".cache"), { recursive: true });
      const temporary = `${cachePath}.${process.pid}`;
      const { deployment_id, deployment, ...reusable } = receipt;
      writeFileSync(temporary, JSON.stringify(reusable));
      renameSync(temporary, cachePath);
    } catch {
      // The release is already saved; cache persistence is not a prerequisite.
    }
    return receipt;
  } catch (error) {
    error.url = site?.current_live_url ?? site?.expected_url ?? cached?.live_url;
    error.source_commit = source_commit;
    throw error;
  }
}

/** Deployment does no shell commands, Git, builds, checks, uploads or source preparation. */
export default async function publishSite(tools, options) {
  const { project_id, version_id } = options ?? {};
  if (!project_id || !version_id)
    throw new Error("An exact preparation receipt with project_id and version_id is required");
  const privateSite =
    options.owner_private ??
    ownerPrivate(await native(tools, "mcp__codex_apps__sites_get_site", { project_id }));
  let deployment =
    options.deployment ??
    (await (options.deployment_id
      ? native(tools, "mcp__codex_apps__sites_get_deployment_status", {
          project_id,
          deployment_id: options.deployment_id,
        })
      : privateSite
        ? native(tools, "mcp__codex_apps__sites_deploy_private_site_version", {
            project_id,
            version_id,
          })
        : native(tools, "mcp__codex_apps__sites_deploy_site_version", { project_id, version_id })));
  while (["pending", "building", "publishing"].includes(deployment.status)) {
    await pause(1000);
    deployment = await native(tools, "mcp__codex_apps__sites_get_deployment_status", {
      project_id,
      deployment_id: deployment.id,
    });
  }
  if (deployment.status === "succeeded" && deployment.version_id !== version_id)
    throw new Error(
      `Hosting reported version ${deployment.version_id} instead of requested ${version_id}`,
    );
  if (deployment.status !== "succeeded" || !deployment.url)
    throw new Error(
      `Deployment ${deployment.id} failed: ${deployment.failure_message ?? "no production URL"}`,
    );
  return {
    url: deployment.url,
    deployed: true,
    source_commit: options.source_commit,
    version_id,
    deployment_id: deployment.id,
  };
}
