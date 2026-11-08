import assert from "node:assert/strict";
import { test } from "vitest";
import * as Utf8 from "@frontend/Utf8";
import * as Ui from "@frontend/Ui";
import * as Fixtures from "@frontend/Fixture_json";
test("Native UTF-8 crosses React and JSON as Unicode, while ASCII stays unchanged", () => {
  const host = "Nera’s λ🦀 − × …";
  const native = Utf8.of_js(host);
  assert.equal(native, Buffer.from(host, "utf8").toString("latin1"));
  assert.equal(Ui.text(native), host);
  assert.equal(Fixtures.get_string(JSON.parse(JSON.stringify(host))), native);
  assert.equal(Fixtures.put_string(native), host);
  assert.equal(Utf8.to_js("ASCII"), "ASCII");
  assert.equal(Utf8.of_js("ASCII"), "ASCII");
});

test("The byte adapter preserves BOM and Unicode spelling and rejects malformed UTF-8", () => {
  for (const host of [
    "",
    "ASCII\0",
    "\ufeff",
    "\ufeffNera’s λ🦀",
    "é中𝄞e\u0301",
    "a".repeat(4095) + "🦀",
  ]) {
    const native = Utf8.of_js(host);
    assert.equal(native, Buffer.from(host, "utf8").toString("latin1"));
    assert.equal(Utf8.to_js(native), host);
    assert.equal(Ui.text(native), host);
    assert.equal(Fixtures.put_string(native), host);
  }
  for (const bytes of [
    [0x80],
    [0xc0, 0xaf],
    [0xc2],
    [0xed, 0xa0, 0x80],
    [0xf4, 0x90, 0x80, 0x80],
  ]) {
    assert.throws(() => Utf8.to_js(Buffer.from(bytes).toString("latin1")), TypeError);
  }
  assert.equal(Utf8.to_js(Utf8.of_js("\ud800")), "\ufffd");
});
