declare module "virtual:terrain-prebuilt" {
  const maps: Readonly<Record<string, import("./presentation/terrain-map").TerrainMapSprite>>;
  export default maps;
}
