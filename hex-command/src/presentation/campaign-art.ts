import { pixelFrames, pixelSources } from "./pixel-art";

/* Original Normal/Low-poly terrain sheets; complete unit atlases are in styled-art. */
export const campaignSheets = {
  normal: {
    width: 1254,
    height: 1254,
    revision: "37f5ababb12d",
    sources: {
      terrain: {
        filename: "campaign-normal-terrain.png",
        width: 1254,
        height: 1254,
        revision: "6bfd2de31991",
      },
    },
    frames: {
      terrain_marsh: [52, 30, 534, 594],
      terrain_ruins: [666, 29, 536, 595],
      terrain_fort: [54, 632, 533, 594],
      terrain_sacred_grove: [666, 631, 536, 597],
    },
  },
  "low-poly": {
    width: 1254,
    height: 1254,
    revision: "60b26106daaa",
    sources: {
      terrain: {
        filename: "campaign-low-poly-terrain.png",
        width: 1276,
        height: 1233,
        revision: "76d96d68f4c5",
      },
    },
    frames: {
      terrain_marsh: [58, 24, 539, 586],
      terrain_ruins: [664, 23, 538, 590],
      terrain_fort: [55, 617, 541, 593],
      terrain_sacred_grove: [660, 619, 542, 591],
    },
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
