import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

// Exercise failures before application resources are allocated, including in a
// separate worker realm. Every wait is bounded by the page timeout.
export async function testStartup(browser, origin, root, backend) {
  const url = (project) => `${origin}/${project}/dist/${backend}/index.html`;
  const ready = (page) =>
    page.waitForFunction(
      () => document.documentElement.dataset.ready === "true",
    );
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    page.setDefaultTimeout(10000);
    await context.route("**/backend.wasm", (route) => route.abort("failed"));
    await page.goto(url("calculator"));
    const reload = page.getByRole("button", { name: "Reload", exact: true });
    await reload.waitFor();
    assert((await page.locator("pre").textContent()).length > 0);
    assert.notEqual(
      await page.locator("html").getAttribute("data-ready"),
      "true",
    );
    await context.unroute("**/backend.wasm");
    await reload.click();
    await ready(page);
    await page.keyboard.type("8+4=");
    assert.equal(await page.locator("#result").textContent(), "12");
  } finally {
    await context.close();
  }

  for (const mode of ["load", "start", "rejection"]) {
    const context = await browser.newContext();
    try {
      const page = await context.newPage();
      page.setDefaultTimeout(10000);
      let fetches = 0;
      const marker = `worker-${mode}-probe`;
      if (mode === "load") {
        await context.route("**/backend.wasm", (route) =>
          ++fetches === 2 ? route.abort("failed") : route.continue(),
        );
      } else {
        await context.route("**/worker.mjs", async (route) => {
          const source = await readFile(
            resolve(root, "async-search/dist", backend, "worker.mjs"),
            "utf8",
          );
          const prefix =
            mode === "start"
              ? `const original=globalThis.addEventListener;
               globalThis.addEventListener=function(kind,...args) {
                 if(kind==='message') throw Error('${marker}');
                 return original.call(this,kind,...args);
               };\n`
              : `Promise.reject(Error('${marker}'));\n`;
          await route.fulfill({
            contentType: "text/javascript",
            body: prefix + source,
          });
        });
      }
      await page.goto(url("async-search"));
      await ready(page);
      await page.waitForFunction(
        (expected) => {
          const message = document.querySelector("#message").textContent;
          return expected
            ? message.includes(expected)
            : message !== "Worker is loading" && message !== "Worker ready";
        },
        mode === "load" ? null : marker,
      );
      if (mode === "load") assert.equal(fetches, 2);
      const diagnostic = await page.locator("#message").textContent();
      assert(diagnostic.length > 0);
      await page.click("#start");
      assert.equal(
        await page.locator("#message").textContent(),
        diagnostic,
        "failed worker must remain closed",
      );
    } finally {
      await context.close();
    }
  }
  console.log(
    `${backend}: startup failure, reload and worker failure checks passed`,
  );
}
