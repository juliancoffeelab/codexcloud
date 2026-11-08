import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import { chromium } from "playwright";

const folder = import.meta.dirname;
assert.deepEqual(
  await readFile(resolve(folder, "Bindings.ml")),
  await readFile(resolve(folder, "generated/Bindings.ml")),
);
assert.deepEqual(
  await readFile(resolve(folder, "bindings.runtime.js")),
  await readFile(resolve(folder, "generated/bindings.runtime.mjs")),
);
let catResponse;
const server = createServer(async (request, response) => {
  if (request.url === "/fixture/cat") {
    response.setHeader("Content-Type", "application/json");
    response.flushHeaders();
    catResponse = response;
    return;
  }
  if (request.url === "/fixture/invalid") {
    response.setHeader("Content-Type", "application/json");
    response.end("not JSON");
    return;
  }
  if (request.url === "/fixture/error") {
    response.writeHead(503);
    response.end();
    return;
  }
  const name = request.url === "/" ? "index.html" : request.url.slice(1);
  const asset = resolve(folder, "dist", name);
  if (!asset.startsWith(resolve(folder, "dist") + "/")) {
    response.writeHead(404);
    response.end();
    return;
  }
  try {
    response.setHeader(
      "Content-Type",
      name.endsWith(".wasm")
        ? "application/wasm"
        : name.endsWith(".js") || name.endsWith(".mjs")
          ? "text/javascript"
          : "text/html",
    );
    response.end(await readFile(asset));
  } catch (error) {
    response.writeHead(500);
    response.end(String(error));
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM ?? "/usr/bin/chromium",
  args: ["--no-sandbox"],
});
try {
  const page = await browser.newPage();
  page.setDefaultTimeout(10000);
  const errors = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  await page.route("https://images.example/cat.svg", (route) =>
    route.fulfill({
      contentType: "image/svg+xml",
      body: '<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>',
    }),
  );
  await page.goto(origin);
  await page.waitForFunction(
    () => document.documentElement.dataset.ready === "true",
  );
  assert.equal(await page.locator("#distance").textContent(), "Distance: 5");
  assert.equal(
    await page.locator("#token-result").textContent(),
    "Token value: 13",
  );
  await page.locator("#token-input").fill("invalid");
  await page.locator("#read-token").click();
  assert.equal(
    await page.locator("#token-result").textContent(),
    "invalid token",
  );
  await page.locator("#token-input").fill("4294967295");
  await page.locator("#read-token").click();
  assert.equal(
    await page.locator("#token-result").textContent(),
    "Token value: 4294967295",
  );

  await page.locator("#cat-api").fill(`${origin}/fixture/cat`);
  const response = page.waitForResponse(`${origin}/fixture/cat`);
  await page.locator("#fetch-cat").click();
  await response;
  assert.equal(await page.locator("#cat-status").textContent(), "Loading…");
  catResponse.end(
    JSON.stringify([{ id: "λ-demo", url: "https://images.example/cat.svg" }]),
  );
  await page.waitForFunction(
    () => document.getElementById("cat-status").textContent === "Cat λ-demo",
  );
  assert.equal(
    await page.locator("#cat").getAttribute("src"),
    "https://images.example/cat.svg",
  );
  for (const [path, status] of [
    ["invalid", "Invalid cat response"],
    ["error", "Network error"],
  ]) {
    await page.locator("#cat-api").fill(`${origin}/fixture/${path}`);
    await page.locator("#fetch-cat").click();
    await page.waitForFunction(
      (status) => document.getElementById("cat-status").textContent === status,
      status,
    );
  }
  assert.equal(
    await page.evaluate(async () => {
      const { initialize } = await import("./bindings.runtime.mjs");
      const instance = await initialize();
      if (instance.e.__ocamljs_resource_diagnostics)
        return instance.e.__ocamljs_live() === 0;
      return !("__ocamljs_live" in instance.e);
    }),
    true,
  );
  assert.deepEqual(errors, []);
  console.log(
    "Rust → OCaml example: native record, Result, u32 range, opaque disposal, pending async fetch, success/error variants and optional resource probes passed in Chromium.",
  );
} finally {
  await browser.close();
  server.closeAllConnections();
  await new Promise((resolve) => server.close(resolve));
}
