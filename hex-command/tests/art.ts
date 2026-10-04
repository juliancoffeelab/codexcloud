import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { artSprite, loadArtStyle } from "../src/presentation/art";
import { campaignSheets } from "../src/presentation/campaign-art";
import { nativePixelSource } from "../src/presentation/native-pixel-art";

const frameSources = campaignSheets.pixel.frameSources;
for (const source of new Set(Object.values(frameSources))) {
  const png = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
  assert.equal(png.subarray(1, 4).toString(), "PNG");
  assert.equal(png.readUInt32BE(16), source.width);
  assert.equal(png.readUInt32BE(20), source.height);
  assert.equal(png[25], 6, "Focused pixel sources retain RGBA transparency");
  assert.equal(createHash("sha256").update(png).digest("hex").slice(0, 12), source.revision);
}
for (const [frame, source] of Object.entries(frameSources)) {
  const [x, y, width, height] = campaignSheets.pixel.frames[frame as keyof typeof frameSources];
  assert([x, y, width, height].every(Number.isInteger));
  assert(width > 0 && height > 0);
  assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
}
const nativeURL = `${nativePixelSource.filename}?v=${nativePixelSource.revision}`;
for (const side of ["player", "enemy"] as const) {
  for (const name of ["adept", "bonearcher", "boar", "necromancer", "skeleton", "wolf"] as const) {
    const key = `${side}_${name === "bonearcher" ? "bone_archer" : name}` as const;
    assert.equal(frameSources[key], nativePixelSource);
    const sprite = artSprite("pixel", "units", name, 0, 0, 60, 70, "", side);
    assert(sprite.includes(nativeURL));
    assert(
      sprite.includes(`width:${nativePixelSource.width}px;height:${nativePixelSource.height}px`),
    );
    assert(sprite.includes('class="art-sprite pixel-art'));
    assert(sprite.includes('viewBox="0 0 48 48"'));
  }
  for (const name of ["skeleton", "wolf", "adept", "bonearcher", "boar", "necromancer"]) {
    for (const style of ["normal", "low-poly"] as const) {
      const unchanged = artSprite(style, "units", name, 0, 0, 60, 70, "", side);
      assert(unchanged.includes(`campaign-${style}-${side}.png`));
      assert(!unchanged.includes("campaign-pixel-"));
    }
  }
  assert(
    artSprite("pixel", "units", "warden", 0, 0, 60, 70, "", side).includes(
      "campaign-pixel.png?v=f8e1face84a4",
    ),
  );
  assert(
    artSprite("pixel", "units", "soldier", 0, 0, 60, 70, "", side).includes(
      "units-pixel.png?v=cd2af5ff67fe",
    ),
  );
}
assert(
  artSprite("pixel", "terrain", "marsh", 0, 0, 70, 70).includes(
    "campaign-pixel.png?v=f8e1face84a4",
  ),
);

const originalImage = Object.getOwnPropertyDescriptor(globalThis, "Image");
const requested: string[] = [];
let failNative = true;
class TestImage {
  onload?: () => void;
  onerror?: () => void;
  set src(url: string) {
    requested.push(url);
    queueMicrotask(() => {
      if (failNative && url.includes(nativeURL)) this.onerror?.();
      else this.onload?.();
    });
  }
}
try {
  Object.defineProperty(globalThis, "Image", { configurable: true, value: TestImage });
  const firstLoad = loadArtStyle("pixel");
  assert.equal(loadArtStyle("pixel"), firstLoad, "Concurrent requests share the pending load");
  assert.equal(await firstLoad, false, "Missing native atlas rejects ready state");
  assert.equal(requested.length, 5, "Each shared campaign source is preloaded only once");
  assert.equal(requested.filter((url) => url.includes(nativeURL)).length, 1);
  assert.equal(requested.filter((url) => url.includes("campaign-pixel-undead.png")).length, 0);
  failNative = false;
  assert.equal(await loadArtStyle("pixel"), true, "A failed native atlas load can be retried");
  assert.equal(requested.length, 10);
  assert.equal(requested.filter((url) => url.includes(nativeURL)).length, 2);
  assert.equal(await loadArtStyle("pixel"), true);
  assert.equal(requested.length, 10, "Ready style keeps its cached load");
} finally {
  if (originalImage) Object.defineProperty(globalThis, "Image", originalImage);
  else Reflect.deleteProperty(globalThis, "Image");
}
console.log(
  "PASS: native pixel frames, source hashes/crops, preserved art and preload retry/cache",
);
