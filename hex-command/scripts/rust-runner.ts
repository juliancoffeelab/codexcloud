import { rustFingerprint } from "./ai-fingerprint";
import { spawn, execFileSync } from "node:child_process";
import type { ChildProcessWithoutNullStreams } from "node:child_process";
import { createInterface } from "node:readline";
import type {
  ArenaConfig,
  ArenaResult,
  CampaignConfig,
  CampaignResult,
} from "../src/game/ai/simulation";

/** One persistent native process per worker; no JS game logic runs inside a Rust match. */
export class RustRunner {
  private readonly process: ChildProcessWithoutNullStreams;
  private readonly lines: AsyncIterator<string>;
  private error: Error | undefined;
  constructor() {
    const binary = new URL("../rust/target/release/hex-command-sim", import.meta.url).pathname;
    if (execFileSync(binary, ["--fingerprint"], { encoding: "utf8" }).trim() !== rustFingerprint())
      throw new Error("Rust binary is stale. Run npm run ai:rust:build.");
    this.process = spawn(
      new URL("../rust/target/release/hex-command-sim", import.meta.url).pathname,
      [],
      { stdio: ["pipe", "pipe", "pipe"] },
    );
    this.process.stderr.pipe(process.stderr);
    this.process.once("error", (error) => {
      this.error = error;
    });
    this.lines = createInterface({ input: this.process.stdout })[Symbol.asyncIterator]();
  }
  async run(config: ArenaConfig): Promise<ArenaResult>;
  async run(config: CampaignConfig): Promise<CampaignResult>;
  async run(config: ArenaConfig | CampaignConfig): Promise<ArenaResult | CampaignResult>;
  async run(config: ArenaConfig | CampaignConfig): Promise<ArenaResult | CampaignResult> {
    if (this.error) throw this.error;
    this.process.stdin.write(JSON.stringify(config) + "\n");
    const next = await this.lines.next();
    if (next.done)
      throw (
        this.error ??
        new Error(
          "Rust simulation process terminated before returning a result. Run npm run ai:rust:build.",
        )
      );
    return JSON.parse(next.value) as ArenaResult | CampaignResult;
  }
  close(): void {
    this.process.stdin.end();
    this.process.kill();
  }
}
