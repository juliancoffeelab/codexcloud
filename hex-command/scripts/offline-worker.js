/* Build placeholders are replaced with the complete, content-versioned release. */
const release = __RELEASE__;
const assets = __ASSETS__;
const prefix = `hex-command-offline:${self.registration.scope}:`;
const cacheName = `${prefix}${release}`;
const address = (path) => new URL(path, self.registration.scope).href;
const marker = address("offline-ready");

let preparing;
function prepare() {
  return (preparing ??= (async () => {
    try {
      const cache = await caches.open(cacheName);
      let next = 0;
      await Promise.all(
        Array.from({ length: 8 }, async () => {
          while (next < assets.length) {
            const path = assets[next++];
            const response = await fetch(address(path), { cache: "reload" });
            if (!response.ok)
              throw new Error(`Offline download failed: ${path} (${response.status})`);
            await cache.put(address(path), response);
          }
        }),
      );
      await cache.put(marker, new Response(release));
    } catch (error) {
      await caches.delete(cacheName);
      for (const client of await self.clients.matchAll({ includeUncontrolled: true }))
        client.postMessage({
          type: "hex-command-offline-error",
          error: error?.stack ?? String(error),
        });
      throw error;
    } finally {
      preparing = undefined;
    }
  })());
}
// Updates wait for existing game tabs to close, keeping each release coherent.
self.addEventListener("install", (event) => event.waitUntil(prepare()));

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      for (const key of await caches.keys())
        if (key.startsWith(prefix) && key !== cacheName) await caches.delete(key);
      await self.clients.claim();
    })(),
  );
});

self.addEventListener("message", (event) => {
  if (!["hex-command-offline-status", "hex-command-offline-prepare"].includes(event.data?.type))
    return;
  event.waitUntil(
    (async () => {
      try {
        if (event.data.type === "hex-command-offline-prepare") await prepare();
        const cache = await caches.open(cacheName);
        const complete =
          !!(await cache.match(marker)) && (await cache.keys()).length === assets.length + 1;
        event.ports[0]?.postMessage({ ready: complete });
      } catch (error) {
        event.ports[0]?.postMessage({ error: error?.stack ?? String(error) });
      }
    })(),
  );
});

self.addEventListener("fetch", (event) => {
  if (event.request.method !== "GET") return;
  const url = new URL(event.request.url);
  if (!url.href.startsWith(self.registration.scope)) return;
  event.respondWith(
    (async () => {
      const cache = await caches.open(cacheName);
      // Release files are static; Origin negotiation and cache-busting queries do not change their content.
      const cached = await cache.match(
        event.request.mode === "navigate" ? address("index.html") : event.request,
        { ignoreVary: true, ignoreSearch: true },
      );
      return cached ?? fetch(event.request);
    })(),
  );
});
