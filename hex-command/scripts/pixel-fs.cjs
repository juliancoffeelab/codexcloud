// Saved construction recipes may still request paired sources. Keep their inputs
// virtual; render-pixel consolidates recipe outputs before baking the atlas.
const fs = require("node:fs");
const path = require("node:path");
const { execFileSync } = require("node:child_process");
const project = path.resolve(__dirname, "..");
function shared(file) {
  return typeof file === "string" && fs.existsSync(path.join(path.dirname(file), "sprite.txt"));
}
function readFileSync(file, options) {
  if (!shared(file)) return fs.readFileSync(file, options);
  const name = path.basename(file);
  let text;
  if ((name === "player.txt" || name === "enemy.txt") && !fs.existsSync(file)) {
    const [base, shadow] = name === "player.txt" ? ["T", "t"] : ["R", "r"];
    text = fs
      .readFileSync(path.join(path.dirname(file), "sprite.txt"), "utf8")
      .replaceAll("^", base)
      .replaceAll("*", shadow);
  } else if (name === "palette.json") {
    const palette = JSON.parse(fs.readFileSync(file, "utf8"));
    const factions = JSON.parse(
      fs.readFileSync(path.join(project, "art/pixel/factions.json"), "utf8"),
    );
    for (const [symbol, colour] of Object.entries({
      T: factions.player["^"],
      t: factions.player["*"],
      R: factions.enemy["^"],
      r: factions.enemy["*"],
    }))
      palette[symbol] ??= colour;
    text = JSON.stringify(palette);
  } else if (
    (name === "player.png" || name === "enemy.png") &&
    (!fs.existsSync(file) ||
      ["sprite.txt", "palette.json"].some(
        (source) =>
          fs.statSync(path.join(path.dirname(file), source)).mtimeMs > fs.statSync(file).mtimeMs,
      ) ||
      fs.statSync(path.join(project, "art/pixel/factions.json")).mtimeMs >
        fs.statSync(file).mtimeMs)
  ) {
    execFileSync(
      "python3",
      [
        path.join(project, "scripts/render-pixel.py"),
        path.join(path.dirname(file), "sprite.txt"),
        "--previews",
      ],
      { cwd: project, stdio: "pipe" },
    );
  }
  if (text === undefined) return fs.readFileSync(file, options);
  const encoding = typeof options === "string" ? options : options?.encoding;
  return encoding ? text : Buffer.from(text);
}
module.exports = {
  ...fs,
  readFileSync,
  existsSync(file) {
    return (
      fs.existsSync(file) ||
      (shared(file) && ["player.txt", "enemy.txt"].includes(path.basename(file)))
    );
  },
  copyFileSync(source, target, flags) {
    if (
      shared(source) &&
      (!fs.existsSync(source) ||
        ["palette.json", "player.png", "enemy.png"].includes(path.basename(source)))
    )
      return fs.writeFileSync(target, readFileSync(source), {
        flag: flags & fs.constants.COPYFILE_EXCL ? "wx" : "w",
      });
    return fs.copyFileSync(source, target, flags);
  },
};
