import { writable } from "svelte/store";

export interface OfflineStatus {
  phase: "preparing" | "ready" | "failed" | "unavailable";
  message: string;
  detail?: string;
}
export const offlineStatus = writable<OfflineStatus>({
  phase: "preparing",
  message: "Preparing offline play… Keep this tab online until ready.",
});
let pending = false;
const failure = (error: unknown) =>
  offlineStatus.set({
    phase: "failed",
    message: "Offline preparation failed. Stay online and retry before closing the game.",
    detail: error instanceof Error ? (error.stack ?? error.message) : String(error),
  });

function status(worker: ServiceWorker, prepare = false): Promise<boolean> {
  return new Promise((resolve, reject) => {
    const channel = new MessageChannel();
    const timeout = setTimeout(
      () => {
        channel.port1.close();
        reject(new Error("Offline readiness check timed out."));
      },
      prepare ? 180_000 : 10_000,
    );
    channel.port1.onmessage = (event) => {
      clearTimeout(timeout);
      channel.port1.close();
      if (event.data?.error) reject(new Error(event.data.error));
      else resolve(event.data?.ready === true);
    };
    worker.postMessage(
      { type: prepare ? "hex-command-offline-prepare" : "hex-command-offline-status" },
      [channel.port2],
    );
  });
}
function installed(worker: ServiceWorker): Promise<void> {
  return new Promise((resolve, reject) => {
    const changed = () => {
      if (worker.state === "activated") {
        worker.removeEventListener("statechange", changed);
        resolve();
      } else if (worker.state === "redundant") {
        worker.removeEventListener("statechange", changed);
        reject(
          new Error(
            "Offline download was interrupted or browser storage refused the files. Retry while online.",
          ),
        );
      }
    };
    worker.addEventListener("statechange", changed);
    changed();
  });
}

/** Called after mounting the shell. No release download sits on the gameplay loading path. */
export async function prepareOffline(): Promise<void> {
  if (pending) return;
  if (!import.meta.env.PROD || !("serviceWorker" in navigator) || !window.isSecureContext) {
    offlineStatus.set({
      phase: "unavailable",
      message: import.meta.env.PROD
        ? "Offline reopening is unavailable in this browser. Use HTTPS and enable browser storage."
        : "Offline reopening is available in production builds.",
    });
    return;
  }
  pending = true;
  offlineStatus.set({
    phase: "preparing",
    message: "Preparing offline play… Keep this tab online until ready.",
  });
  let downloadFailure: string | undefined;
  const onError = (event: MessageEvent) => {
    if (event.data?.type === "hex-command-offline-error") downloadFailure = event.data.error;
  };
  navigator.serviceWorker.addEventListener("message", onError);
  try {
    const url = new URL("./offline-worker.js", document.baseURI);
    const scope = new URL("./", document.baseURI).href;
    const existing = await navigator.serviceWorker.getRegistration(url.href);
    const registration =
      existing?.scope === scope
        ? existing
        : await navigator.serviceWorker.register(url, { scope, updateViaCache: "none" });
    if (registration.active && navigator.onLine) void registration.update().catch(() => {});
    // An already active release remains usable while a later release downloads.
    if (!registration.active) {
      const worker = registration.installing ?? registration.waiting;
      if (!worker) throw new Error("The offline worker did not start.");
      await installed(worker);
    }
    const worker = registration.active;
    if (!worker || (!(await status(worker)) && !(await status(worker, true))))
      throw new Error("Offline files are incomplete. Reconnect and retry preparation.");
    offlineStatus.set({ phase: "ready", message: "Ready for offline play" });
  } catch (error) {
    failure(downloadFailure ?? error);
  } finally {
    pending = false;
    navigator.serviceWorker.removeEventListener("message", onError);
  }
}
