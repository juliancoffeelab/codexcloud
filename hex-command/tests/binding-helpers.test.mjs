import assert from "node:assert/strict";
import { test } from "vitest";
import * as Enum from "@frontend/Enum";
import * as Equal from "@frontend/Snapshot_equal";
import * as Json from "@frontend/Fixture_json";
import schema from "@wasm/schema.json" with { type: "json" };

const snake = (name) => name.replace(/([a-z])([A-Z])/g, "$1_$2").toLowerCase();
test("Frontend enum conversions follow Rust spellings and reject unknown closed values", () => {
  for (const entry of Object.values(schema).filter(
    (e) => e.kind === "enum" && e.variants.every((v) => !v.body.fields.length),
  )) {
    const name = snake(entry.name),
      stringify = Enum[`string_of_${name}`],
      parse = Enum[`${name}_of_string`];
    if (!stringify && !parse) continue;
    for (const variant of entry.variants) {
      const spelling = variant.serde_name ?? variant.name;
      const native = Json[`of_${name}`](spelling);
      if (stringify) assert.equal(stringify(native), spelling);
      if (parse) assert.deepEqual(Json.put_option(Json[`to_${name}`], parse(spelling)), spelling);
    }
    if (parse) assert.equal(Json.put_option(Json[`to_${name}`], parse("unknown-value")), null);
  }
});
test("Snapshot comparisons preserve NaN, signed zero, nested content and sequence lengths", () => {
  const light = (power) =>
    Json.of_battlefield_lighting({
      unlit_tiles: [{ col: 0, row: 0 }],
      lights: [{ col: 0, row: 0, radius: 10, power }],
    });
  const nan = light(NaN);
  assert.equal(Equal.battlefield_lighting(nan, nan), false);
  assert.equal(Equal.battlefield_lighting(light(0), light(-0)), true);
  assert.equal(Equal.battlefield_lighting(light(1), light(2)), false);
  assert.equal(
    Equal.battlefield_lighting(
      light(1),
      Json.of_battlefield_lighting({ unlit_tiles: [{ col: 0, row: 0 }], lights: [] }),
    ),
    false,
  );
  const unit = (name, items = []) =>
    Json.of_public_unit({
      experience: 0,
      id: "unit",
      type: "kind",
      side: "player",
      tile: { col: 0, row: 0 },
      hp: 1,
      moved: false,
      attackSpent: false,
      faction: "player",
      revealedTo: [],
      perceptionEffects: [],
      character: name,
      items,
    });
  const one = unit("one");
  assert(Equal.public_unit(one, one));
  assert(Equal.public_unit(one, unit("one")));
  assert.equal(Equal.public_unit(one, unit("two")), false);
  assert.equal(Equal.public_unit(one, unit("one", ["item"])), false);
});
