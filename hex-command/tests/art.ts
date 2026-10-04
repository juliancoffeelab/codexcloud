import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { artSprite, loadArtStyle } from "../src/presentation/art";
import { campaignSheets } from "../src/presentation/campaign-art";

const undeadFrames = campaignSheets.pixel.frameSources;
for (const [frame, source] of Object.entries(undeadFrames)) {
  const png = readFileSync(new URL(`../public/assets/${source.filename}`, import.meta.url));
  assert.equal(png.subarray(1, 4).toString(), "PNG");
  assert.equal(png.readUInt32BE(16), source.width);
  assert.equal(png.readUInt32BE(20), source.height);
  assert.equal(png[25], 6, "Dedicated undead sheet retains RGBA transparency");
  assert.equal(createHash("sha256").update(png).digest("hex").slice(0, 12), source.revision);
  const [x, y, width, height] = campaignSheets.pixel.frames[frame as keyof typeof undeadFrames];
  assert(x >= 0 && y >= 0 && x + width <= source.width && y + height <= source.height);
}
for (const side of ["player", "enemy"] as const) {
  for (const name of ["skeleton", "bonearcher"]) {
    const sprite = artSprite("pixel", "units", name, 0, 0, 60, 70, "", side);
    assert(sprite.includes("campaign-pixel-undead.png?v=44d5e0513ea7"));
    assert(sprite.includes("width:1330px;height:1182px"));
    assert(sprite.includes('class="art-sprite pixel-art'));
    for (const style of ["normal", "low-poly"] as const) {
      const unchanged = artSprite(style, "units", name, 0, 0, 60, 70, "", side);
      assert(unchanged.includes(`campaign-${style}-${side}.png`));
      assert(!unchanged.includes("campaign-pixel-undead"));
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
let failUndead = true;
class TestImage {
  onload?: () => void;
  onerror?: () => void;
  set src(url: string) {
    requested.push(url);
    queueMicrotask(() => {
      if (failUndead && url.includes("campaign-pixel-undead")) this.onerror?.();
      else this.onload?.();
    });
  }
}
try {
  Object.defineProperty(globalThis, "Image", { configurable: true, value: TestImage });
  assert.equal(await loadArtStyle("pixel"), false, "Missing focused sheet rejects ready state");
  assert.equal(requested.length, 5, "Shared undead source is preloaded only once");
  assert(requested.some((url) => url.includes("campaign-pixel-undead.png?v=44d5e0513ea7")));
  failUndead = false;
  assert.equal(await loadArtStyle("pixel"), true, "A failed load can be retried");
  assert.equal(requested.length, 10);
  assert.equal(await loadArtStyle("pixel"), true);
  assert.equal(requested.length, 10, "Ready style keeps its cached load");
} finally {
  if (originalImage) Object.defineProperty(globalThis, "Image", originalImage);
  else Reflect.deleteProperty(globalThis, "Image");
}
console.log(
  "PASS: focused pixel undead source, crop bounds, unchanged fallbacks and preload retry",
);
