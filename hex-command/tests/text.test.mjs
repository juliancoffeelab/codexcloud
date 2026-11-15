import assert from "node:assert/strict";
import { test } from "vitest";
import * as BindingText from "@frontend/Binding_text";
import * as Ui from "@frontend/Ui";
import * as Fixtures from "@frontend/Fixture_json";
test("React and JSON boundaries retain JavaScript text without byte conversion", () => {
  for (const host of ["ASCII", "Nera’s λ🦀 − × …", "\ufeffé中𝄞e\u0301", "\ud800"]) {
    assert.equal(Ui.text(host), host);
    assert.equal(Fixtures.get_string(JSON.parse(JSON.stringify(host))), host);
    assert.equal(Fixtures.put_string(host), host);
  }
});

test("The SDK loader adapter preserves Unicode and rejects malformed native bytes", () => {
  for (const host of [
    "",
    "ASCII\0",
    "\ufeff",
    "\ufeffNera’s λ🦀",
    "é中𝄞e\u0301",
    "a".repeat(4095) + "🦀",
  ]) {
    const native = BindingText.of_js(host);
    assert.equal(native, Buffer.from(host, "utf8").toString("latin1"));
    assert.equal(BindingText.to_js(native), host);
  }
  for (const bytes of [
    [0x80],
    [0xc0, 0xaf],
    [0xc2],
    [0xed, 0xa0, 0x80],
    [0xf4, 0x90, 0x80, 0x80],
  ]) {
    assert.throws(() => BindingText.to_js(Buffer.from(bytes).toString("latin1")), TypeError);
  }
  assert.equal(BindingText.to_js(BindingText.of_js("\ud800")), "\ufffd");
});
