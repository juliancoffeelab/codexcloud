import type { GameEngineFactory } from "./native-engine";

let ready: Promise<GameEngineFactory> | undefined;

/** The boot owner starts initialization; catalog imports only consume it. */
export function startEngine(load: () => Promise<GameEngineFactory>) {
  if (ready)
    throw new Error("This page already owns an engine boot. Reload to start a new application.");
  ready = load();
  return ready;
}

export function engineReady() {
  if (!ready) throw new Error("Start the engine before loading application metadata.");
  return ready;
}
