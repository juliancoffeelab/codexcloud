import type { TerrainType } from "../game/types";
import { artFrame, type Artwork, type FrameImage } from "./art";

type PreviewTile = { readonly terrain: TerrainType };

// UI portraits are explicit presentation assets, never map geometry defaults.
// Every orientation, floor and junction configuration of a terrain shares this image.
const portraits = {
  plains: ["plains_layout_0", "green-plain"],
  road: ["road_9", "green-plain"],
  forest: ["forest_layout_0", "green-forest"],
  hill: ["hill_layout_0", "green-plain"],
  water: ["water", "dark-water"],
  bridge: ["bridge_9", "dark-water"],
  ford: ["ford_9_layout_0", "shallow-water"],
  village: ["village_0", "green-plain"],
  keep: ["keep", "green-plain"],
  marsh: ["marsh", "marsh"],
  ruins: ["ruins", "green-plain"],
  fort: ["fort", "green-plain"],
  grove: ["grove", "green-forest"],
  landing: ["landing_1_layout_0", "shallow-water"],
  ferry: ["ferry_1_layout_0", "dark-water"],
  house: ["house_layout_0", "green-plain"],
  mountain: ["mountain_layout_0", "mountain-rock"],
  cave: ["cave_layout_0", "mountain-rock"],
  "burned-village": ["burned-village_layout_0", "green-plain"],
  signpost: ["signpost_9_layout_0", "green-plain"],
  "castle-wall": ["castle-wall_63", "castle-paving"],
  "castle-bell": ["castle-bell_63", "castle-paving"],
  monastery: ["monastery_layout_0", "green-plain"],
  church: ["church_layout_0", "green-plain"],
  graveyard: ["graveyard_layout_0", "green-plain"],
  "ritual-star": ["ritual-star_layout_0", "mountain-rock"],
  "ritual-spent": ["ritual-spent_layout_0", "mountain-rock"],
} as const satisfies Record<TerrainType, readonly [string, string]>;

export function terrainPreviewFrame(tile: PreviewTile, artwork: Artwork): Readonly<FrameImage> {
  const [name, floor] = portraits[tile.terrain];
  const frame = artFrame(artwork, "terrain", name, "player", floor);
  if (!frame) throw new Error(`Missing terrain preview: ${name} (${floor})`);
  return frame;
}

interface PreviewEntry {
  image: HTMLImageElement;
  ready: boolean;
  loaded: Promise<HTMLImageElement>;
}

/** Retain decoded shared terrain portraits; panels mount these same image nodes. */
export class TerrainPreviewCache {
  private entries = new Map<string, PreviewEntry>();

  ready(frame: Readonly<FrameImage>): HTMLImageElement | undefined {
    const entry = this.entries.get(frame.url);
    return entry?.ready ? entry.image : undefined;
  }

  load(frame: Readonly<FrameImage>): Promise<HTMLImageElement> {
    const existing = this.entries.get(frame.url);
    if (existing) return existing.loaded;
    const image = new Image();
    image.decoding = "async";
    image.src = frame.url;
    const entry: PreviewEntry = {
      image,
      ready: false,
      loaded: image
        .decode()
        .then(() => {
          entry.ready = true;
          return image;
        })
        .catch((cause) => {
          throw new Error(`Terrain preview could not load: ${frame.url}`, { cause });
        }),
    };
    this.entries.set(frame.url, entry);
    return entry.loaded;
  }

  /** Warm the complete shared portrait set independently of the active map. */
  prepare(artwork: Artwork): void {
    for (const terrain of Object.keys(portraits) as TerrainType[]) {
      const frame = terrainPreviewFrame({ terrain }, artwork);
      // Keep the rejected entry so opening it shows an explicit retry state.
      void this.load(frame).catch(() => {});
    }
  }

  retry(frame: Readonly<FrameImage>) {
    this.entries.delete(frame.url);
    return this.load(frame);
  }

  dispose() {
    this.entries.clear();
  }
}
