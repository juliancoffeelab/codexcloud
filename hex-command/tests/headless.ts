import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Game } from "../src/game/engine";
import { enemyActionIds } from "../src/game/enemy-turn";

const directory = mkdtempSync(join(tmpdir(), "hex-headless-"));
const path = join(directory, "trial.json");
function call(request: unknown) {
  const process = spawnSync(
    globalThis.process.execPath,
    ["--import", "tsx", "scripts/play-campaign.ts", path, JSON.stringify(request)],
    { encoding: "utf8", timeout: 3000 },
  );
  assert.equal(process.status, 0, process.stderr);
  return JSON.parse(process.stdout) as { error?: string; result: unknown };
}
const saved = () => JSON.parse(readFileSync(path, "utf8"));
try {
  call({ op: "start", campaign: "lantern-road" });
  const game = Game.startCampaign("lantern-road");
  game.advanceDialogue(true);
  assert.deepEqual(saved().state, game.captureState());
  const leader = game.units.find((unit) => unit.role === "leader" && unit.side === "player")!;
  const before = saved();
  call({ op: "options", unit: leader.id });
  call({ op: "threats", unit: leader.id, tiles: [leader.tile, "1,2"] });
  assert.deepEqual(saved(), before, "Mechanical inspections must not change the trial");

  const destination = game.recruitTiles(leader.id, "soldier")[0]!.id;
  const response = call({
    op: "act",
    actions: [
      { op: "recruit", unit: leader.id, type: "soldier", tile: destination },
      { op: "move", unit: leader.id, tile: "99,99" },
    ],
  });
  assert(response.error, "Invalid actions must return an error and current observation");
  game.recruit(leader.id, "soldier", destination);
  assert.deepEqual(saved().state, game.captureState(), "Prior successful actions persist");
  assert.equal(saved().moves.length, 0, "Recruitment clears move undo");

  call({ op: "act", actions: [], endTurn: true });
  game.endTurn();
  game.enemyRecruit();
  for (const id of enemyActionIds(game.units)) {
    game.enemyAct(id);
    if (game.winner) break;
  }
  if (!game.winner) game.endTurn();
  game.advanceDialogue(true);
  assert.deepEqual(
    saved().state,
    game.captureState(),
    "Headless enemy response matches live rules",
  );
  console.log(
    "PASS: headless state persistence, pure inspection, partial action errors and enemy turns",
  );
} finally {
  rmSync(directory, { recursive: true, force: true });
}
