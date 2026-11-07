import type { BattleView } from "../controller/view";
import { artFrameKey, itemFrame } from "./art";
import { TYPES, TERRAIN } from "../game/catalog";
import { CAMPAIGNS } from "../game/campaign-data";
import { MAPS } from "../game/maps";
import { AUTHORED_ITEMS } from "../game/authored-content";
import type { UnitType, TerrainType } from "../game/types";
import { terrainPreviewFrame } from "./terrain-preview";
import { terrainPortraits } from "./terrain-portraits";
import { terrainTexture } from "./terrain-texture";
import { terrainTileView } from "./terrain-map";
import type { TileView } from "../game/view";
import { audioSamples, battlefieldActionFamilies, soundForInteraction } from "./sound";
import { missionSounds } from "./campaign-sound";

export interface Resource {
  key: string;
  url: string;
  kind: "image" | "audio";
  priority: number;
  hud?: boolean;
  width?: number;
  height?: number;
}

/** Sounds for interactions with the currently published map, excluding future content. */
export function currentMapAudio(view: BattleView) {
  const families = new Set<string>(battlefieldActionFamilies);
  for (const unit of view.game.units) {
    families.add(soundForInteraction({ unitType: unit.type }));
    families.add(soundForInteraction({ attackType: unit.type }));
  }
  for (const tile of view.game.tiles)
    families.add(soundForInteraction({ terrain: TERRAIN[tile.terrain].role }));
  return audioSamples.filter((sample) => families.has(sample.family));
}

/** Presentation dependencies only; Rust still chooses recruits, promotions and content. */
export function resourcePlan(view: BattleView) {
  const { game, artwork } = view;
  const resources = new Map<string, Resource>();
  const add = (resource: Resource) => {
    const old = resources.get(resource.key);
    resources.set(resource.key, {
      ...resource,
      priority: Math.min(old?.priority ?? 5, resource.priority),
      hud: old?.hud || resource.hud,
    });
  };
  const image = (key: string, priority: number) => {
    const frame = artwork.frames[key];
    if (!frame) throw new Error(`Missing artwork: ${key}`);
    add({
      key: frame.url,
      url: frame.url,
      kind: "image",
      priority,
      hud: true,
      width: frame.width,
      height: frame.height,
    });
  };
  const audio = (families: Iterable<string>, priority: number) => {
    const wanted = new Set(families);
    for (const sample of audioSamples)
      if (wanted.has(sample.family)) add({ ...sample, kind: "audio", priority });
  };
  const palettes = new Set(["player", ...game.factions.map((faction) => faction.palette)]);
  const units = (
    types: ReadonlySet<string>,
    priority: number,
    sides: ReadonlySet<string> = palettes,
  ) => {
    // Optional prefetch variants come from the artwork catalogue. A palette
    // present on this map does not imply every archetype has that palette.
    // Actual battlefield unit frames are separately required by the Scene.
    for (const type of types)
      for (const palette of sides) {
        const key = artFrameKey("units", type, palette);
        if (artwork.frames[key]) image(key, priority);
      }
    audio(
      [...types].map((type) => soundForInteraction({ unitType: type as UnitType })),
      priority,
    );
    audio(["attack-bow", "attack-melee"], priority);
  };
  const contentTypes = (content: unknown, includeRecruitment = false) => {
    const found = new Set<string>();
    const visit = (value: unknown): void => {
      if (typeof value === "string" && Object.hasOwn(TYPES, value)) found.add(value);
      else if (Array.isArray(value)) value.forEach(visit);
      else if (value && typeof value === "object")
        for (const [key, child] of Object.entries(value))
          if (
            includeRecruitment ||
            !["recruitment", "recruits", "unlockRecruits", "advancesTo", "recall"].includes(key)
          )
            visit(child);
    };
    visit(content);
    return found;
  };
  const campaign = game.campaign && CAMPAIGNS[game.campaign.id];
  const mission = campaign?.missions[game.campaign!.mission];

  const prepareTerrain = (
    tiles: readonly TileView[],
    mapId: string,
    content: unknown,
    priority: number,
  ) => {
    const frame = (tile: TileView) => {
      const image = terrainTileView(tile, mapId, artwork).terrainFrame;
      if (!image) throw new Error(`Missing terrain artwork: ${mapId}/${tile.id}`);
      add({
        key: image.url,
        url: image.url,
        kind: "image",
        priority,
        width: image.width,
        height: image.height,
      });
    };
    const changes: { tile: string; terrain: TerrainType }[] = [];
    const visit = (value: unknown): void => {
      if (Array.isArray(value)) return value.forEach(visit);
      if (!value || typeof value !== "object") return;
      for (const [key, child] of Object.entries(value)) {
        if (key === "setTerrain") {
          changes.push(child as { tile: string; terrain: TerrainType });
        } else visit(child);
      }
    };
    visit(content);
    if (!changes.length) return;
    tiles.forEach(frame);
    for (const change of changes) {
      const literal = tiles.find((t) => t.id === change.tile);
      for (const tile of literal ? [literal] : tiles)
        frame({
          ...tile,
          terrain: change.terrain,
          connections: undefined,
          walls: undefined,
          deadEnds: undefined,
        });
    }
  };

  // Put actual current-map audio ahead of speculative sounds at the same priority.
  for (const sample of currentMapAudio(view)) add({ ...sample, kind: "audio", priority: 1 });
  // 1: Current world and immediately relevant presentation/audio.
  const baked = terrainTexture(game.mapId, game.mapRevision);
  for (const page of baked.pages)
    add({
      key: page.url,
      url: page.url,
      kind: "image",
      priority: 1,
      width: page.width,
      height: page.height,
    });
  for (const unit of game.units)
    for (const item of unit.items ?? []) image(itemFrame(artwork, item.id).key!, 1);
  image("portrait_worker", 1);
  const currentTypes = contentTypes(MAPS[game.mapId]);
  game.units.forEach((unit) => currentTypes.add(unit.type));
  units(currentTypes, 1);
  const terrains = new Set(game.tiles.map((tile) => tile.terrain));
  for (const terrain of terrains) image(terrainPreviewFrame({ terrain }, artwork).key!, 1);
  audio(
    [
      ...battlefieldActionFamilies,
      ...[...terrains].map((terrain) => soundForInteraction({ terrain: TERRAIN[terrain].role })),
    ],
    1,
  );
  for (const id of Object.keys(AUTHORED_ITEMS)) image(itemFrame(artwork, id).key!, 1);
  if (mission) {
    const scripted = contentTypes(mission);
    for (const type of scripted) currentTypes.add(type);
    units(scripted, 1);
    audio(
      missionSounds(
        mission,
        game.campaign!.mission === campaign!.missions.length - 1 ? campaign!.ending : [],
      ),
      1,
    );
  }
  units(contentTypes(game.campaign?.veterans), 1);
  units(contentTypes(game.campaign?.storyVeterans), 1);

  // 2: Rust-published recruitment; 3: their and current units' promotion closure.
  const recruits = new Set(
    game.factions.flatMap((faction) => [...game.recruitmentOptions(faction.id)]),
  );
  const collectRecruits = (value: unknown): void => {
    if (Array.isArray(value)) value.forEach(collectRecruits);
    else if (value && typeof value === "object")
      for (const [key, child] of Object.entries(value)) {
        if (["recruitment", "recruits", "unlockRecruits"].includes(key))
          for (const type of contentTypes(child)) recruits.add(type as UnitType);
        else collectRecruits(child);
      }
  };
  collectRecruits(mission);
  units(recruits, 2);
  const promotions = new Set([...currentTypes, ...recruits]);
  for (const type of promotions)
    for (const next of TYPES[type as UnitType].advancesTo ?? []) promotions.add(next);
  units(promotions, 3);

  // 4: Prepare the next chapter while the current one remains playable.
  const next = campaign?.missions[game.campaign!.mission + 1];
  if (next) {
    const frame = terrainTexture(next.map.id, next.map.revision ?? 1);
    for (const page of frame.pages)
      add({
        key: page.url,
        url: page.url,
        kind: "image",
        priority: 4,
        width: page.width,
        height: page.height,
      });
    prepareTerrain(next.map.terrain as readonly TileView[], next.map.id, next, 4);
    const nextPalettes = new Set([
      ...palettes,
      ...Object.keys(artwork.frames)
        .filter((key) => /^(player|enemy|passive|green)_/.test(key))
        .map((key) => key.split("_")[0]!),
    ]);
    units(contentTypes(next, true), 4, nextPalettes);
    audio(missionSounds(next), 4);
    audio(
      next.map.terrain.map((tile) => soundForInteraction({ terrain: TERRAIN[tile.terrain].role })),
      4,
    );
  }

  // Complete tile frames prepare background runtime edits, never initial-map readiness.
  prepareTerrain(game.tiles, game.mapId, mission ?? MAPS[game.mapId], 1);

  // 5: Small shared catalogues, not every terrain permutation in the game.
  for (const key of Object.keys(artwork.frames))
    if (/^(player|enemy|passive|green|portrait)_/.test(key)) image(key, 5);
  for (const terrain of Object.keys(terrainPortraits) as TerrainType[])
    image(terrainPreviewFrame({ terrain }, artwork).key!, 5);
  for (const sample of audioSamples) add({ ...sample, kind: "audio", priority: 5 });
  // Interleave audio/images within a priority instead of serializing whole categories.
  const ordered: Resource[] = [];
  for (let priority = 1; priority <= 5; priority++) {
    const images = [...resources.values()].filter(
      (r) => r.priority === priority && r.kind === "image",
    );
    const sounds = [...resources.values()].filter(
      (r) => r.priority === priority && r.kind === "audio",
    );
    for (let i = 0; i < Math.max(images.length, sounds.length); i++) {
      if (sounds[i]) ordered.push(sounds[i]!);
      if (images[i]) ordered.push(images[i]!);
    }
  }
  return new Map(ordered.map((resource) => [resource.key, resource]));
}

/** Position/HP/selection publications do not rebuild the prefetch plan. */
export function resourcePlanKey(view: BattleView) {
  return JSON.stringify([
    view.sessionVersion,
    view.game.mapId,
    view.game.factions,
    [...new Set(view.game.units.map((u) => u.type))],
    [...new Set(view.game.tiles.map((t) => t.terrain))],
    view.game.factions.map((f) => view.game.recruitmentOptions(f.id)),
    view.game.campaign?.mission,
  ]);
}
