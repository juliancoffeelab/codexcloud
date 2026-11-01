/** Native Sites orchestration. Preparation and deployment are separate operations. */
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

/** Save the checked implementation as a remotely stored, unpublished release. */
export async function prepareSite(tools, options, progress = () => {}) {
  const cwd = options.cwd;
  const quoted = (value) => "'" + value.replaceAll("'", "'\\''") + "'";
  const read = await tools.exec_command({ cmd: "cat .openai/hosting.json", workdir: cwd });
  if (read.exit_code !== 0) throw new Error(read.output);
  const { project_id } = JSON.parse(read.output);
  const site = decode(await tools.mcp__codex_apps__sites_get_site({ project_id }));
  const credential = decode(
    await tools.mcp__codex_apps__sites_create_source_repository_write_credential({ project_id }),
  );
  if (credential.publish_on_push_accepted) {
    throw new Error("An automatic preparation window is active; reconcile its deployment first");
  }
  progress("Building and synchronizing the preparation source.");
  let cell = await tools.exec_command({
    cmd:
      "npm run release:prepare -- --message " +
      quoted(options.message ?? "Prepare Hex Command release"),
    workdir: cwd,
    tty: true,
    yield_time_ms: 1000,
    max_output_tokens: 1000,
  });
  let output = cell.output;
  while (cell.session_id && !output.includes("Ready for Site credential JSON")) {
    cell = await tools.write_stdin({ session_id: cell.session_id, chars: "", yield_time_ms: 1000 });
    output += cell.output;
  }
  if (!cell.session_id)
    throw new Error("Preparation command failed before credential input: " + output);
  cell = await tools.write_stdin({
    session_id: cell.session_id,
    chars: JSON.stringify(credential) + "\n",
    yield_time_ms: 1000,
    max_output_tokens: 1000,
  });
  output += cell.output;
  while (cell.session_id) {
    cell = await tools.write_stdin({
      session_id: cell.session_id,
      chars: "",
      yield_time_ms: 20000,
      max_output_tokens: 1000,
    });
    output += cell.output;
  }
  if (cell.exit_code !== 0) throw new Error(output);
  const line = output.split(/\r?\n/).findLast((line) => line.startsWith("SITE_PACKAGE: "));
  if (!line) throw new Error("Preparation command returned no package");
  const prepared = JSON.parse(line.slice("SITE_PACKAGE: ".length));
  if (prepared.project_id !== project_id)
    throw new Error("Preparation package Site identity changed");
  const version = decode(
    await tools.mcp__codex_apps__sites_save_site_version({
      project_id,
      commit_sha: prepared.commit_sha,
      archive: prepared.archive,
    }),
  );
  return {
    project_id,
    version_id: version.id,
    version_number: version.version_number,
    source_commit: prepared.source_commit,
    site_commit: prepared.commit_sha,
    owner_private: ownerPrivate(site),
  };
}

/** Deployment does no shell commands, Git, builds, checks, uploads or source preparation. */
export default async function publishSite(tools, options) {
  const { project_id, version_id } = options ?? {};
  if (!project_id || !version_id)
    throw new Error("An exact preparation receipt with project_id and version_id is required");
  const privateSite =
    options.owner_private ??
    ownerPrivate(decode(await tools.mcp__codex_apps__sites_get_site({ project_id })));
  let deployment = decode(
    await (privateSite
      ? tools.mcp__codex_apps__sites_deploy_private_site_version({ project_id, version_id })
      : tools.mcp__codex_apps__sites_deploy_site_version({ project_id, version_id })),
  );
  while (["pending", "building", "publishing"].includes(deployment.status)) {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    deployment = decode(
      await tools.mcp__codex_apps__sites_get_deployment_status({
        project_id,
        deployment_id: deployment.id,
      }),
    );
  }
  if (deployment.status !== "succeeded" || !deployment.url)
    throw new Error(
      `Deployment ${deployment.id} failed: ${deployment.failure_message ?? "no production URL"}`,
    );
  return { url: deployment.url, version_id, deployment_id: deployment.id };
}
