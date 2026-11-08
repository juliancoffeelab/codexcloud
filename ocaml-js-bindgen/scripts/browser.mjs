import { testStartup } from "./browser-startup.mjs";
import { testPaint } from "./browser-paint.mjs";
import assert from "node:assert/strict";
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve, extname, sep } from "node:path";
import { chromium } from "playwright";

const root = resolve(import.meta.dirname, "..", "demos");
const server = createServer(async (request, response) => {
  try {
    if (request.url === "/__unicode") {
      response.setHeader("Content-Type", "application/x-ndjson");
      response.end(JSON.stringify({ text: "\ufeffλ 🦀 OCaml" }) + "\n");
      return;
    }
    if (request.url === "/__long-record") {
      response.setHeader("Content-Type", "application/x-ndjson");
      response.end(JSON.stringify({ text: "OCaml λ".repeat(6000) }) + "\n");
      return;
    }
    if (request.url.startsWith("/__gate/")) {
      response.setHeader("Content-Type", "application/x-ndjson");
      response.flushHeaders();
      response.write(
        Array.from(
          { length: 16 },
          (_, n) => JSON.stringify({ text: "OCaml gated record " + n }) + "\n",
        ).join(""),
      );
      return;
    }
    const path = resolve(
      root,
      "." +
        decodeURIComponent(new URL(request.url, "http://localhost").pathname),
    );
    if (!path.startsWith(root + sep)) throw new Error("invalid path");
    const body = await readFile(path);
    response.setHeader(
      "Content-Type",
      {
        ".html": "text/html",
        ".js": "text/javascript",
        ".mjs": "text/javascript",
        ".wasm": "application/wasm",
        ".css": "text/css",
      }[extname(path)] ?? "application/octet-stream",
    );
    response.end(body);
  } catch {
    response.writeHead(404);
    response.end();
  }
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const catGates = new Map();
const catApi = createServer((request, response) => {
  response.setHeader("Access-Control-Allow-Origin", "*");
  response.setHeader("Content-Type", "application/json");
  const path = new URL(request.url, "http://fixture");
  if (path.pathname === "/cat") {
    response.flushHeaders();
    catGates.set(path.searchParams.get("backend"), response);
  } else if (path.pathname === "/bad") response.end("not JSON");
  else if (path.pathname === "/large") response.end(" ".repeat(66000));
  else {
    response.writeHead(503);
    response.end();
  }
});
await new Promise((resolve) => catApi.listen(0, "127.0.0.1", resolve));
const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM ?? "/usr/bin/chromium",
  args: ["--no-sandbox"],
});
try {
  for (const backend of ["melange", "js_of_ocaml", "rescript"]) {
    for (const project of [
      "calculator",
      "hello-world",
      "todo",
      "game-of-life",
      "image-editor",
      "document-explorer",
      "async-search",
      "scripted-game",
      "cat-pics",
    ]) {
      const page = await browser.newPage();
      page.setDefaultTimeout(10000);
      const errors = [];
      page.on("pageerror", (error) => errors.push(String(error)));
      if (project === "async-search")
        await page.addInitScript(() => {
          const Original = globalThis.Worker;
          const probe = (globalThis.searchProbe = {
            commands: [],
            holdValidation: false,
          });
          globalThis.Worker = class extends Original {
            constructor(...args) {
              super(...args);
              probe.worker = this;
            }
            postMessage(data, ...rest) {
              probe.commands.push(data.kind);
              return super.postMessage(data, ...rest);
            }
            set onmessage(handler) {
              super.onmessage = (event) => {
                const deliver = () => handler(event);
                if (event.data.kind === "ready") probe.ready = deliver;
                else if (
                  event.data.kind === "validation" &&
                  probe.holdValidation
                )
                  probe.validation = deliver;
                else deliver();
              };
            }
          };
        });
      if (project === "cat-pics") {
        await page.addInitScript(() => {
          globalThis.Worker = class {
            constructor() {
              throw new Error("ordinary async must not create a Worker");
            }
          };
        });
        await page.route("https://cat.fixture/image.svg", (route) =>
          route.fulfill({
            contentType: "image/svg+xml",
            body: '<svg xmlns="http://www.w3.org/2000/svg" width="240" height="180"><path fill="orange" d="M40 150V30L90 70h60l50-40v120Z"/><circle cx="90" cy="110" r="8"/><circle cx="150" cy="110" r="8"/></svg>',
          }),
        );
      }
      if (project === "image-editor")
        await page.addInitScript(() => {
          const Native = Uint8ClampedArray;
          globalThis.pixelProbe = { copies: 0, bytes: 0 };
          globalThis.Uint8ClampedArray = new Proxy(Native, {
            construct(Target, args) {
              if (ArrayBuffer.isView(args[0])) {
                pixelProbe.copies++;
                pixelProbe.bytes += args[0].byteLength;
              }
              return Reflect.construct(Target, args);
            },
          });
        });
      await page.goto(
        `http://127.0.0.1:${server.address().port}/${project}/dist/${backend}/index.html`,
      );
      await page.waitForFunction(
        () => document.documentElement.dataset.ready === "true",
      );
      assert.deepEqual(
        await page.evaluate(() => [
          typeof globalThis.OcamljsDom,
          typeof globalThis.SearchChannel,
        ]),
        ["undefined", "undefined"],
      );
      if (project === "cat-pics") {
        const base = `http://127.0.0.1:${catApi.address().port}`;
        await page.evaluate(async () => {
          const { initialize } = await import("./bindings.runtime.mjs");
          (await initialize()).diagnostics = true;
        });
        await page.fill("#api-url", `${base}/cat?backend=${backend}`);
        await page.click("#fetch");
        await page.waitForFunction(
          () => document.querySelector("#status").textContent === "Loading…",
        );
        await page.waitForFunction(async () => {
          const { initialize } = await import("./bindings.runtime.mjs");
          const i = await initialize();
          return (
            i.e.__ocamljs_tasks() === 1 &&
            i.stats.polls >= 1 &&
            i.fetches.size === 1
          );
        });
        const before = await page.evaluate(async () => {
          const { initialize, api } = await import("./bindings.runtime.mjs");
          if (api.worker_load) throw new Error("unexpected worker runtime");
          return (await initialize()).stats.polls;
        });
        await page.evaluate(
          () =>
            new Promise((resolve) => {
              const c = new MessageChannel();
              c.port1.onmessage = () => {
                c.port1.close();
                c.port2.close();
                resolve();
              };
              c.port2.postMessage(0);
            }),
        );
        assert.equal(
          await page.evaluate(async () => {
            const { initialize } = await import("./bindings.runtime.mjs");
            return (await initialize()).stats.polls;
          }),
          before,
          "idle fetch was polled without wake",
        );
        assert(catGates.has(backend));
        catGates
          .get(backend)
          .end(
            JSON.stringify([
              { id: "λ🦀", url: "https://cat.fixture/image.svg" },
            ]),
          );
        catGates.delete(backend);
        await page.waitForFunction(
          () =>
            document.querySelector("#status").textContent === "Cat λ🦀" &&
            document.querySelector("#cat").naturalWidth === 240,
        );
        assert.equal(await page.locator("#cat").getAttribute("alt"), "Cat λ🦀");
        for (const [path, message] of [
          ["bad", "Invalid cat response"],
          ["large", "Response is too large"],
          ["failure", "Network request failed"],
        ]) {
          await page.evaluate(async () => {
            const { initialize } = await import("./bindings.runtime.mjs");
            (await initialize()).diagnostics = true;
          });
          await page.fill("#api-url", base + "/" + path);
          await page.click("#fetch");
          await page.waitForFunction(
            (text) => document.querySelector("#status").textContent === text,
            message,
          );
        }
        await page.waitForFunction(async () => {
          const { initialize } = await import("./bindings.runtime.mjs");
          const i = await initialize();
          return (
            i.e.__ocamljs_live() === 0 &&
            i.e.__ocamljs_tasks() === 0 &&
            i.fetches.size === 0
          );
        });
      }
      if (project === "calculator") {
        await page.setViewportSize({ width: 390, height: 844 });
        const display = () => page.locator("#result").innerText();
        const click = (key) => page.locator(`[id="key-${key}"]`).click();
        for (const key of ["1", "2", "/", "4", "Enter"]) await click(key);
        assert.equal(await display(), "3");
        await page.keyboard.press("Enter");
        assert.equal(await display(), "0.75");
        await page.keyboard.press("Escape");
        await page.keyboard.type("0.1+0.2=");
        assert.equal(await display(), "0.3");
        await page.keyboard.press("Escape");
        await page.keyboard.type("8+*2+1=");
        assert.equal(await display(), "17");
        await page.keyboard.press("Escape");
        await page.keyboard.type("9/0=");
        assert.equal(await display(), "Cannot divide by zero");
        await click("5");
        assert.equal(await display(), "5");
        await click("0");
        await click("%");
        assert.equal(await display(), "0.5");
        await page.keyboard.press("Escape");
        await click("n");
        await click("5");
        assert.equal(await display(), "-5");
        await page.keyboard.press("Backspace");
        assert.equal(await display(), "0");
        await page.keyboard.type("12345678901234567890");
        assert.equal((await display()).length, 16);
        assert(
          await page.evaluate(
            () =>
              document.documentElement.scrollWidth <= window.innerWidth &&
              [...document.querySelectorAll(".keypad button")].every(
                (button) => {
                  const rect = button.getBoundingClientRect();
                  return rect.width >= 44 && rect.height >= 44;
                },
              ),
          ),
        );
        const live = () =>
          page.evaluate(async () => {
            const { initialize } = await import("./bindings.runtime.mjs");
            return (await initialize("./backend.wasm")).e.__ocamljs_live();
          });
        assert.equal(await live(), 1);
        // A bfcache-preserved page must keep its calculator; an actual exit drops it.
        await page.evaluate(() =>
          window.dispatchEvent(
            new PageTransitionEvent("pagehide", { persisted: true }),
          ),
        );
        await page.keyboard.press("Escape");
        assert.equal(await display(), "0");
        assert.equal(await live(), 1);
        await page.evaluate(() =>
          window.dispatchEvent(
            new PageTransitionEvent("pagehide", { persisted: false }),
          ),
        );
        assert.equal(await live(), 0);
      } else if (project === "hello-world") {
        const result = () => page.locator("#result").innerText();
        assert.equal(await result(), "5");
        await page.fill("#a", "6");
        await page.fill("#b", "8");
        assert.equal(await result(), "10");
        await page.fill("#a", "1");
        await page.fill("#b", "1");
        assert.equal(await result(), String(Math.SQRT2));
        await page.fill("#a", "0");
        await page.fill("#b", "0");
        assert.equal(await result(), "0");
        await page.fill("#a", "");
        assert.equal(await result(), "Enter two nonnegative, finite numbers");
        await page.fill("#a", "-1");
        assert.equal(await result(), "Enter two nonnegative, finite numbers");
        await page.fill("#a", "1.7e308");
        await page.fill("#b", "1.7e308");
        assert.equal(await result(), "Result is too large");
        await page.fill("#a", "3");
        await page.fill("#b", "4");
        assert.equal(await result(), "5");
      } else if (project === "todo") {
        await page.waitForFunction(
          () =>
            document.querySelector("#remaining").textContent === "0 remaining",
        );
        await page.fill("#title", "\ufeffShip λ 🦀");
        await page.click("#add");
        await page.waitForFunction(
          () =>
            document.querySelector("#tasks span")?.textContent ===
            "\ufeffShip λ 🦀",
        );
        await page.getByRole("button", { name: "Done", exact: true }).click();
        await page.waitForFunction(
          () =>
            document.querySelector("#remaining").textContent === "0 remaining",
        );
        await page.getByRole("button", { name: "Delete", exact: true }).click();
        assert.equal(await page.locator("#tasks li").count(), 0);
        await page.click("#add");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "Enter a task title",
        );
      } else if (project === "game-of-life") {
        await page.waitForFunction(
          () => document.querySelectorAll(".alive").length === 3,
        );
        await page.click("#step");
        await page.waitForFunction(
          () =>
            document.querySelector("#generation").textContent ===
            "Generation 1",
        );
        for (const coordinate of ["9,6", "9,7", "9,8"])
          assert.match(
            await page
              .getByRole("button", { name: `Cell ${coordinate}`, exact: true })
              .getAttribute("class"),
            /alive/,
          );
        await page
          .getByRole("button", { name: "Cell 0,0", exact: true })
          .click();
        assert.equal(await page.locator(".alive").count(), 4);
      }
      if (project === "image-editor") await testPaint(page);
      if (project === "document-explorer") {
        await page.setInputFiles("#file", {
          name: "unicode.json",
          mimeType: "application/json",
          buffer: Buffer.from('{"title":"OCaml λ🦀"}'),
        });
        await page.click("#load-file");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "JSON file loaded",
        );
        await page
          .getByRole("button", { name: "title (string)", exact: true })
          .click();
        assert.equal(
          await page.locator("#selection").textContent(),
          '"OCaml λ🦀"',
        );
        await page.click("#release-view");
        await page.setInputFiles("#file", {
          name: "large.json",
          mimeType: "application/json",
          buffer: Buffer.alloc(65537, 32),
        });
        await page.click("#load-file");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "File is too large",
        );
        await page.evaluate(() => {
          const text = Blob.prototype.text;
          globalThis.documentProbe = {};
          Blob.prototype.text = function () {
            const value = text.call(this);
            return this.name === "delayed.json"
              ? new Promise((resolve, reject) =>
                  value.then((text) => {
                    documentProbe.release = () => resolve(text);
                  }, reject),
                )
              : value;
          };
        });
        await page.setInputFiles("#file", {
          name: "delayed.json",
          mimeType: "application/json",
          buffer: Buffer.from('{"stale":true}'),
        });
        await page.click("#load-file");
        await page.waitForFunction(
          () => typeof documentProbe.release === "function",
        );
        await page.fill("#source", '{"newer":true}');
        await page.click("#parse");
        await page.evaluate(() => documentProbe.release());
        await page.waitForFunction(async () => {
          const { initialize } = await import("./bindings.runtime.mjs");
          return (await initialize()).e.__ocamljs_host_live() === 0;
        });
        assert.equal(
          await page.locator("#message").textContent(),
          "Document parsed",
        );
        assert.equal(
          await page
            .getByRole("button", { name: "stale (bool)", exact: true })
            .count(),
          0,
        );

        await page.fill("#source", "not JSON");
        await page.click("#parse");
        assert.equal(await page.locator("#nodes button").count(), 0);
        assert.equal(
          await page.locator("#message").textContent(),
          "Invalid JSON",
        );
        await page.fill("#source", '{"items":[1,2]}');
        await page.click("#parse");
        await page
          .getByRole("button", { name: "root (object)", exact: true })
          .click();
        await page.fill("#source", '{"changed":true}');
        await page.click("#replace");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "Release live views before mutation",
        );
        await page.click("#release-parent");
        await page.waitForFunction(() =>
          document
            .querySelector("#message")
            .textContent.includes("selected view retains storage"),
        );
        assert.match(await page.locator("#selection").textContent(), /items/);
        await page.click("#release-view");
        await page.click("#parse");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "Document parsed",
        );
      }
      if (project === "async-search") {
        // Ready is held, so even already-compiled click handlers must not send.
        for (const selector of ["#start", "#query", "#validate", "#shutdown"])
          await page.click(selector);
        assert.deepEqual(await page.evaluate(() => searchProbe.commands), []);
        await page.waitForFunction(
          () => typeof searchProbe.ready === "function",
        );
        await page.evaluate(() => searchProbe.ready());
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent === "Worker ready",
        );
        assert(
          await page.evaluate(() => {
            const progress = document.querySelector("#progress").textContent;
            const validation =
              document.querySelector("#validation").textContent;
            for (const data of [
              { kind: "progress", value: "not-a-number" },
              { kind: "progress", value: "4097" },
              { kind: "validation", value: "maybe" },
              { kind: "closed", value: "unexpected payload" },
            ])
              searchProbe.worker.dispatchEvent(
                new MessageEvent("message", { data }),
              );
            searchProbe.worker.postMessage({
              kind: "shutdown",
              value: "unexpected payload",
            });
            return (
              document.querySelector("#progress").textContent === progress &&
              document.querySelector("#validation").textContent ===
                validation &&
              document.querySelector("#message").textContent === "Worker ready"
            );
          }),
        );
        await page.fill("#url", "x".repeat(32769));
        await page.click("#start");
        assert.equal(
          await page.locator("#message").textContent(),
          "Could not send search command",
        );
        await page.fill(
          "#url",
          `http://127.0.0.1:${server.address().port}/__gate/${backend}`,
        );
        await page.click("#start");
        await page.waitForFunction(
          () =>
            document.querySelector("#progress").textContent === "Indexed 16",
        );
        await page.click("#cancel");
        await page.waitForFunction(() =>
          document
            .querySelector("#message")
            .textContent.startsWith("Cancelled;"),
        );
        await page.fill("#url", "./corpus.ndjson");
        await page.click("#start");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent === "Index ready",
        );
        await page.fill("#needle", "x".repeat(32769));
        await page.click("#query");
        assert.equal(
          await page.locator("#message").textContent(),
          "Could not send search command",
        );
        await page.fill("#needle", "OCaml");
        await page.click("#query");
        await page.waitForFunction(
          () => document.querySelectorAll("#matches li").length === 64,
        );
        await page.evaluate(() => {
          searchProbe.holdValidation = true;
        });
        await page.click("#validate");
        await page.waitForFunction(
          () => typeof searchProbe.validation === "function",
        );
        const beforeBuild = await page.evaluate(
          () => searchProbe.commands.filter((x) => x === "build").length,
        );
        await page.click("#start");
        assert.equal(
          await page.evaluate(
            () => searchProbe.commands.filter((x) => x === "build").length,
          ),
          beforeBuild,
        );
        await page.evaluate(() => searchProbe.validation());
        assert.equal(await page.locator("#validation").textContent(), "PASS");
        await page.fill("#url", "/__long-record");
        await page.click("#start");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent === "Index ready",
        );
        await page.click("#query");
        await page.waitForFunction(
          () =>
            document.querySelector("#matches li")?.textContent.length === 42000,
        );
        assert.equal(
          await page.locator("#matches li").textContent(),
          "OCaml λ".repeat(6000),
        );
        await page.fill("#url", "/__unicode");
        await page.click("#start");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent === "Index ready",
        );
        await page.fill("#needle", "OCaml");
        await page.click("#query");
        await page.waitForFunction(
          () =>
            document.querySelector("#matches li")?.textContent ===
            "\ufeffλ 🦀 OCaml",
        );
        await page.click("#shutdown");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "Worker closed gracefully",
        );
      }
      if (project === "scripted-game") {
        for (let n = 0; n < 6; n++) await page.click("#step");
        assert.equal(await page.locator("#score").textContent(), "Score 10");
        assert.match(
          await page.locator("#message").textContent(),
          /reentry rejected/,
        );
        await page.click("#stop");
        await page.waitForFunction(
          () =>
            document.querySelector("#message").textContent ===
            "Handler and world released",
        );
      }
      if (project === "scripted-game")
        await page.evaluate(
          () =>
            new Promise((resolve, reject) => {
              const worker = new Worker("./worker.mjs", { type: "module" });
              worker.onmessage = ({ data }) => {
                worker.terminate();
                data?.kind === "ready"
                  ? resolve()
                  : reject(new Error("worker checks failed"));
              };
              worker.onerror = (error) => {
                worker.terminate();
                reject(new Error(error.message));
              };
            }),
        );
      assert.deepEqual(errors, [], `${project}/${backend} browser errors`);
      await page.close();
      console.log(`${project}/${backend}: Chromium interactions passed`);
    }
    await testStartup(
      browser,
      `http://127.0.0.1:${server.address().port}`,
      root,
      backend,
    );
  }
} finally {
  await browser.close();
  for (const response of catGates.values()) response.destroy();
  await new Promise((resolve) => catApi.close(resolve));
  await new Promise((resolve) => server.close(resolve));
}
