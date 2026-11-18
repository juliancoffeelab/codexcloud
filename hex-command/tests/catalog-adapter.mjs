import * as Catalog from "@frontend/Catalog";
import { fromJSON as from, toJSON as to, listToArray as toArray } from "./json-fixture.mjs";
export * from "@frontend/Catalog";
export const of_view = (v) => Catalog.of_view(from("CatalogView", v));
export const raw = (v) => to("CatalogView", Catalog.raw(v));
export const units = (c) => toArray(Catalog.units(c)).map((v) => to("WikiUnit", v));
export const terrain = (c) => toArray(Catalog.terrain(c)).map((v) => to("WikiTerrain", v));
export const races = (c) => toArray(Catalog.races(c)).map((v) => to("WikiRace", v));
export const maps = (c) => toArray(Catalog.maps(c)).map((v) => to("BrowserMap", v));
export const campaigns = (c) => toArray(Catalog.campaigns(c)).map((v) => to("BrowserCampaign", v));
export const unit = (c, id) => to("WikiUnit", Catalog.unit(c, id));
export const terrain_type = (c, id) => to("WikiTerrain", Catalog.terrain_type(c, id));
export const race = (c, id) => to("WikiRace", Catalog.race(c, id));
export const map = (c, id) => to("BrowserMap", Catalog.map(c, id));
export const campaign = (c, id) => to("BrowserCampaign", Catalog.campaign(c, id));
export const scenario_maps = (c) =>
  toArray(Catalog.scenario_maps(c)).map((v) => to("BrowserMap", v));
export const mission_map = (c, m) =>
  to("BrowserMap", Catalog.mission_map(c, from("BrowserChapter", m)));
