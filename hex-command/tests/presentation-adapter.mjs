import { cellKey } from "./hex-fixture.mjs";
import * as Utf8 from "@frontend/Utf8";
import * as P from "@frontend/Presentation";
import {
  fromJSON as from,
  toJSON as to,
  listToArray as toArray,
  fromArray,
} from "./json-fixture.mjs";
export * from "@frontend/Presentation";
// Presentation remains native UTF-8. Convert only presentation fields at this
// JavaScript test boundary; Domain DTOs already cross through Fixture_json.
const domainFields = new Set([
  "unit",
  "definition",
  "promotions",
  "weapon",
  "tile",
  "occupant",
  "moved_unit",
  "attacker",
]);
function host(value) {
  if (typeof value === "string") return Utf8.to_js(value);
  if (Array.isArray(value)) return value.map(host);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, v]) => [key, domainFields.has(key) ? v : host(v)]),
    );
  return value;
}
const optional = (type, v) => to(type, v);
const abilityView = (v) => host({ ...v, facts: toArray(v.facts) });
const details = (v) => ({
  ...host({
    ...v,
    ability: abilityView(v.ability),
    terrain_effects: toArray(v.terrain_effects).map((effect) => ({
      ...effect,
      facts: toArray(effect.facts),
    })),
  }),
  unit: to("PublicUnit", v.unit),
  definition: to("UnitRule", v.definition),
  promotions: toArray(v.promotions).map((x) => to("UnitRule", x)),
});
const forecast = (v) => host({ ...v, side: to("Side", v.side) });
export const unit_panel = (...args) => {
  const v = P.unit_panel(...args);
  if (v === 0) return v;
  if (v.TAG === 1) return { ...v, _0: details(v._0) };
  if (v.TAG === 2) return { ...v, _0: to("PublicUnit", v._0), _1: to("PublicUnit", v._1) };
  return v;
};
export const attacks = (c, v, u, t) =>
  toArray(P.attacks(c, v, from("PublicUnit", u), from("PublicUnit", t))).map((x) => ({
    ...host(x),
    weapon: to("Weapon", x.weapon),
    result: x.result && {
      ...host(x.result),
      player: forecast(x.result.player),
      enemy: forecast(x.result.enemy),
    },
  }));
export const ability = (c, u) => abilityView(P.ability(c, from("PublicUnit", u)));
export const armor = (c, u) => host(toArray(P.armor(c, from("PublicUnit", u))));
export const terrain_damage_facts = (values) =>
  host(toArray(P.terrain_damage_facts(fromArray(values.map(fromArray)))));
export const weapon_range = (w) => Utf8.to_js(P.weapon_range(from("Weapon", w)));
export const damage_name = (d) => Utf8.to_js(P.damage_name(from("DamageType", d.toLowerCase())));
export const damage_equation = (d) => Utf8.to_js(P.damage_equation(from("DamageDetail", d)));
export const terrain_panel = (...args) => {
  if (args[3] !== undefined) args[3] = cellKey(args[3]);
  const v = P.terrain_panel(...args);
  return (
    v && {
      ...host({ ...v, facts: toArray(v.facts) }),
      tile: to("Tile", v.tile),
      recruitment: v.recruitment && {
        ...v.recruitment,
        options: toArray(v.recruitment.options).map((x) => ({
          ...x,
          definition: to("UnitRule", x.definition),
        })),
      },
    }
  );
};
export const battlefield = (...args) =>
  toArray(P.battlefield(undefined, ...args)).map((x) => ({
    ...host(x),
    tile: to("Tile", x.tile),
    occupant: optional("PublicUnit", x.occupant),
  }));
export const changes = (a, b) => {
  const v = P.changes(from("State", a), from("State", b));
  return {
    ...v,
    tile_effects: toArray(v.tile_effects),
    captured_tiles: toArray(v.captured_tiles),
    new_winner: optional("Side", v.new_winner),
    moved_unit: optional("PublicUnit", v.moved_unit),
    attacker: optional("PublicUnit", v.attacker),
  };
};
