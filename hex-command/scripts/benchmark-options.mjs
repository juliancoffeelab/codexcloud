import Benchmark from "benchmark";

const args = process.argv.slice(2);
const at = args.indexOf("--time");
export const maxTime = at === -1 ? 1 : Number(args[at + 1]);
if (!Number.isFinite(maxTime) || maxTime <= 0)
  throw Error("--time must be a positive number of seconds");
Benchmark.options.maxTime = maxTime;
export default Benchmark;
