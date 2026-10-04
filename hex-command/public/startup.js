(function () {
  const panel = document.getElementById("startup");
  if (!panel) return;
  const reload = document.getElementById("startup-reload");
  reload.addEventListener("click", function () {
    const url = new URL(window.location.href);
    url.searchParams.set("reload", Date.now().toString());
    reload.href = url.href;
  });
  function showFailure(error) {
    document.getElementById("startup-message").textContent =
      "The game could not start. Reloading keeps your saved battle.";
    document.getElementById("startup-error").hidden = false;
    document.getElementById("startup-error").open = true;
    document.getElementById("startup-detail").textContent = String(
      error?.stack || error?.message || error || "The game script could not be loaded.",
    );
  }
  function failed(event) {
    if (event.target?.tagName === "SCRIPT" || event.error || event.message) {
      showFailure(
        event.error ||
          (event.target?.tagName === "SCRIPT"
            ? `Could not load game script: ${event.target.src}`
            : `${event.message}\n${event.filename || ""}:${event.lineno || ""}:${event.colno || ""}`),
      );
    }
  }
  function rejected(event) {
    showFailure(event.reason);
  }
  window.addEventListener("error", failed, true);
  window.addEventListener("unhandledrejection", rejected);
  window.addEventListener("hex-command-ready", function ready() {
    panel.remove();
    window.removeEventListener("error", failed, true);
    window.removeEventListener("unhandledrejection", rejected);
    window.removeEventListener("hex-command-ready", ready);
  });
})();
