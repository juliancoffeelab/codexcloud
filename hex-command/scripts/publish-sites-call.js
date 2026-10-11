/** One-call adapter: local publication command plus native Sites operations. */
export default async function publishSite(tools, options, progress = () => {}) {
  const cwd = options.cwd;
  const decode = (result) => {
    if (result.isError) {
      throw new Error(
        "Sites operation failed: " +
          (result.content ?? [])
            .filter((item) => item.type === "text")
            .map((item) => item.text)
            .join("\n"),
      );
    }
    if (result.structuredContent) return result.structuredContent;
    for (const item of result.content ?? []) {
      if (item.type === "text") {
        try {
          return JSON.parse(item.text);
        } catch {
          // A text notice may precede the structured result.
        }
      }
    }
    throw new Error("Sites returned no structured result");
  };
  const quoted = (value) => "'" + value.replaceAll("'", "'\\''") + "'";
  const read = await tools.exec_command({ cmd: "cat .openai/hosting.json", workdir: cwd });
  if (read.exit_code !== 0) throw new Error(read.output);
  const { project_id } = JSON.parse(read.output);
  const site = decode(await tools.mcp__codex_apps__sites_get_site({ project_id }));
  const policy = site.access_policy;
  const ownerPrivate =
    site.current_user_role === "owner" &&
    policy?.access_mode === "custom" &&
    policy.allowed_users?.length === 1 &&
    policy.allowed_users[0].role === "owner" &&
    !policy.external_visitor_count &&
    !policy.allowed_groups?.length &&
    !policy.allowed_editors?.length &&
    !policy.allowed_workspace_group_ids?.length &&
    !policy.allowed_tenant_group_ids?.length;
  const credential = decode(
    await tools.mcp__codex_apps__sites_create_source_repository_write_credential({ project_id }),
  );
  if (credential.publish_on_push_accepted) {
    throw new Error("An automatic publication window is active; reconcile its deployment first");
  }
  progress("Building and synchronizing the publication source.");
  let cell = await tools.exec_command({
    cmd: "npm run publish -- --message " + quoted(options.message ?? "Publish Hex Command"),
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
    throw new Error("Publication command failed before credential input: " + output);
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
      yield_time_ms: 1000,
      max_output_tokens: 1000,
    });
    output += cell.output;
  }
  if (cell.exit_code !== 0) throw new Error(output);
  const line = output.split(/\r?\n/).findLast((line) => line.startsWith("SITE_PACKAGE: "));
  if (!line) throw new Error("Publication command returned no package");
  const prepared = JSON.parse(line.slice("SITE_PACKAGE: ".length));
  if (prepared.project_id !== project_id)
    throw new Error("Publication package Site identity changed");
  const input = { project_id, commit_sha: prepared.commit_sha, archive: prepared.archive };
  progress("Sites is publishing the packaged build; GitHub will be pushed after it succeeds.");
  let deployment;
  if (ownerPrivate) {
    deployment = decode(await tools.mcp__codex_apps__sites_save_version_and_deploy_private(input));
  } else {
    const version = decode(await tools.mcp__codex_apps__sites_save_site_version(input));
    deployment = decode(
      await tools.mcp__codex_apps__sites_deploy_site_version({
        project_id,
        version_id: version.id,
      }),
    );
  }
  while (["pending", "building", "publishing"].includes(deployment.status)) {
    await new Promise((resolve) => setTimeout(resolve, 5000));
    deployment = decode(
      await tools.mcp__codex_apps__sites_get_deployment_status({
        project_id,
        deployment_id: deployment.id,
      }),
    );
  }
  if (deployment.status !== "succeeded") {
    throw new Error(`Deployment ${deployment.id} failed: ${deployment.failure_message}`);
  }
  if (!deployment.url) {
    deployment = decode(
      await tools.mcp__codex_apps__sites_get_deployment_status({
        project_id,
        deployment_id: deployment.id,
      }),
    );
  }
  if (!deployment.url) throw new Error("Deployment succeeded but Sites returned no URL");
  progress("Site published; pushing the deployed commit to GitHub.");
  cell = await tools.exec_command({
    cmd:
      "git push origin " + quoted(prepared.source_commit + ":refs/heads/" + prepared.source_branch),
    workdir: cwd,
    yield_time_ms: 1000,
    max_output_tokens: 1000,
  });
  let pushOutput = cell.output;
  while (cell.session_id) {
    cell = await tools.write_stdin({
      session_id: cell.session_id,
      chars: "",
      yield_time_ms: 1000,
      max_output_tokens: 1000,
    });
    pushOutput += cell.output;
  }
  if (cell.exit_code !== 0)
    throw new Error(
      `Site deployed at ${deployment.url} (${deployment.id}), but GitHub push failed: ${pushOutput}`,
    );
  return {
    url: deployment.url,
    source_commit: prepared.source_commit,
    deployment_id: deployment.id,
  };
}
