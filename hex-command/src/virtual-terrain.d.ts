declare module "virtual:terrain-prebuilt" {
  const textures: Readonly<Record<string, import("./presentation/terrain-map").TerrainMapSprite>>;
  export default textures;
}
