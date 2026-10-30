declare module "virtual:terrain-prebuilt" {
  const maps: Readonly<Record<string, import("./presentation/terrain-map").TerrainMapSprite>>;
  export const presentations: Readonly<
    Record<string, import("./presentation/battlefield-lighting").BattlefieldLighting>
  >;
  export default maps;
}
