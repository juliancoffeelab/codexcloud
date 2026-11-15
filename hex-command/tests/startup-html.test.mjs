// @vitest-environment jsdom
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "vitest";
import { JSDOM } from "jsdom";
import React, { act } from "react";
import { createRoot } from "react-dom/client";
import * as Shell from "@frontend/Shell";

const html = readFileSync("index.html", "utf8");
function structure(node) {
  if (node.nodeType === 3) return node.textContent.trim() || null;
  if (node.nodeType !== 1) return null;
  return [
    node.localName,
    [...node.attributes]
      .filter((attr) => !(attr.name === "id" && attr.value.startsWith("startup")))
      .filter((attr) => !(attr.name === "style" && node.classList.contains("loading-fill")))
      .filter((attr) => !(attr.name === "class" && attr.value === ""))
      .map((attr) => [
        attr.name,
        node.localName === "progress" && attr.name === "value"
          ? String(Number(attr.value))
          : attr.value,
      ])
      .sort(([a], [b]) => a.localeCompare(b)),
    [...node.childNodes].map(structure).filter((value) => value !== null),
  ];
}

test("Pre-baked HTML matches the application's initial shell without running JavaScript", async () => {
  const page = new DOMParser().parseFromString(html, "text/html");
  const startup = page.getElementById("startup");
  assert.equal(startup.querySelector("progress").getAttribute("value"), "0");
  assert.equal(startup.querySelector("progress").getAttribute("max"), "5");
  assert([...startup.querySelectorAll("button")].every((button) => button.disabled));
  assert.equal(page.getElementById("startup-failure").hidden, true);
  startup.querySelectorAll("#startup-failure, noscript").forEach((node) => node.remove());
  const host = document.createElement("div");
  const root = createRoot(host);
  try {
    await act(async () => root.render(React.createElement(Shell.make, { failure: undefined })));
    assert.deepEqual(structure(startup), structure(host.firstElementChild));
  } finally {
    await act(async () => root.unmount());
  }
});

function bootPage(source = html) {
  return new JSDOM(source, { runScripts: "dangerously", url: "https://example.test/" });
}

test("An error before the HTML shell is parsed replaces its bar with diagnostics and reload", () => {
  const page = bootPage(
    html.replace(
      "</head>",
      `<script>window.dispatchEvent(new ErrorEvent("error", { error: new Error("Engine failed") }));</script></head>`,
    ),
  );
  try {
    const { document, Event } = page.window;
    document.dispatchEvent(new Event("DOMContentLoaded"));
    assert.equal(document.getElementById("startup").getAttribute("aria-busy"), "false");
    assert.equal(document.getElementById("startup-loading").hidden, true);
    assert.equal(page.window.BOOT_PROCESS.status, "failed");
    assert.match(page.window.BOOT_PROCESS.error, /Engine failed/);
    assert.equal(page.window.BOOT_PROCESS.events.at(-1).status, "failed");
    assert.equal(document.getElementById("startup-failure").hidden, false);
    assert.equal(document.getElementById("startup-error").open, true);
    assert.match(
      document.getElementById("startup-detail").textContent,
      /Error: Engine failed\n\s+at/,
    );
    assert.match(document.getElementById("startup-reload").href, /reload=1/);
  } finally {
    page.window.close();
  }
});

test("The mounted application replaces the HTML shell at the existing readiness handoff", () => {
  const page = bootPage();
  try {
    const { document, Event } = page.window;
    document.getElementById("app").textContent = "Mounted application";
    page.window.dispatchEvent(new Event("hex-command-shell-ready"));
    assert.equal(document.getElementById("startup"), null);
    assert.equal(document.getElementById("app").textContent, "Mounted application");
  } finally {
    page.window.close();
  }
});
