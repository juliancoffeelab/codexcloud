import * as Catalog from "@frontend/Catalog";
import { fromJSON as from, toJSON as to, listToArray as toArray } from "./json-fixture.mjs";
export * from "@frontend/Catalog";
export const of_view = (v) => Catalog.of_view(from("CatalogView", v));
export const raw = (v) => to("CatalogView", Catalog.raw(v));
export const units = (c) => toArray(Catalog.units(c)).map((v) => to("UnitRule", v));
export const terrain = (c) => toArray(Catalog.terrain(c)).map((v) => to("TerrainDefinition", v));
export const races = (c) => toArray(Catalog.races(c)).map((v) => to("RaceDefinition", v));
export const maps = (c) => toArray(Catalog.maps(c)).map((v) => to("MapDefinition", v));
export const campaigns = (c) => toArray(Catalog.campaigns(c)).map((v) => to("Campaign", v));
export const unit = (c, id) => to("UnitRule", Catalog.unit(c, id));
export const terrain_type = (c, id) => to("TerrainDefinition", Catalog.terrain_type(c, id));
export const race = (c, id) => to("RaceDefinition", Catalog.race(c, id));
export const map = (c, id) => to("MapDefinition", Catalog.map(c, id));
export const campaign = (c, id) => to("Campaign", Catalog.campaign(c, id));
export const scenario_maps = (c) =>
  toArray(Catalog.scenario_maps(c)).map((v) => to("MapDefinition", v));
export const mission_map = (c, m) =>
  to("MapDefinition", Catalog.mission_map(c, from("Mission", m)));
