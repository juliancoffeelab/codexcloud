import load,{NativeEngineFactory} from "@pkg/hex_command.js";
import {createNativeEngineFactory}from"@upstream/game/native-engine";
import {startEngine}from"@upstream/game/engine-startup";
let factory, LiveGame, nativeValue;
export async function initialize(wasm) {
  await load({ module_or_path: wasm });
  factory=createNativeEngineFactory(new NativeEngineFactory(),{responseOwnership:"transferred"});
  startEngine(()=>Promise.resolve(factory));
  ({ LiveGame, nativeValue } = await import("@upstream/game/live-game"));
}
export function phase(input) {
  const battle = LiveGame.restore(factory, input.checkpoint, {
    first: "player",
    enemy: { policy: input.policy, diplomacy: "diplomatic" },
  });
  try {
    while (battle.turn === "enemy" && !battle.winner) battle.enemyStep();
    return nativeValue(battle.serializedCheckpoint());
  } finally {
    battle.dispose();
  }
}
export function close() {
  factory.dispose();
}
