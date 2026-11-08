import * as Catalog from "@frontend/Catalog";
import { catalog } from "./native-fixture.mjs";
import * as Plan from "@frontend/Terrain_plan";
import { fromJSON as from, fromArray } from "./json-fixture.mjs";
export * from "@frontend/Terrain_plan";
export const key = (id, cols, rows, tiles) =>
  Plan.key(id, cols, rows, fromArray(tiles.map((t) => from("Tile", t))));
export const same_tile = (a, b) => Plan.same_tile(from("Tile", a), from("Tile", b));
export const layout_name = (t, id) => Plan.layout_name(from("Tile", t), id);
export const prepare = (id, cols, rows, tiles, art) =>
  Plan.prepare(
    Catalog.terrain_roles(catalog),
    id,
    cols,
    rows,
    fromArray(tiles.map((t) => from("Tile", t))),
    art,
  );
