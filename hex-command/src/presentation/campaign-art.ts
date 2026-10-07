import { pixelFrames, pixelSources } from "./pixel-art";
import { styledTerrainSheets } from "./styled-art";

/* One complete terrain atlas for each style; unit frames live in styled-art. */
export const campaignSheets = {
  normal: {
    ...styledTerrainSheets.normal.source,
    sources: { terrain: styledTerrainSheets.normal.source },
    frames: styledTerrainSheets.normal.frames,
  },
  "low-poly": {
    ...styledTerrainSheets["low-poly"].source,
    sources: { terrain: styledTerrainSheets["low-poly"].source },
    frames: styledTerrainSheets["low-poly"].frames,
  },
  pixel: {
    ...pixelSources.units,
    sources: {
      player: pixelSources.units,
      enemy: pixelSources.units,
      terrain: pixelSources.terrain,
    },
    frames: pixelFrames,
  },
} as const;
