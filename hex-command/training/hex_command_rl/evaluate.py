"""Evaluate an exported model and its baseline with identical match configurations."""
import argparse
import json
from pathlib import Path
import hex_command_training as native
from .workloads import arena, encode


def main():
    p = argparse.ArgumentParser()
    p.add_argument("model", type=Path)
    p.add_argument("--configs", type=Path, help="JSON array of native arena/campaign configs")
    p.add_argument("--seed", type=int, default=20002)
    p.add_argument("--out", type=Path, default=Path("tmp/rl/evaluation.json"))
    args = p.parse_args()
    configs = json.loads(args.configs.read_text()) if args.configs else arena([args.seed])
    teacher = [json.loads(r) for r in native.evaluate(encode(configs), None)]
    learned = [json.loads(r) for r in native.evaluate(encode(configs), args.model.read_text())]
    report = dict(fingerprints=json.loads(native.fingerprints()), configs=configs,
                  teacher_results=teacher, learned_results=learned)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2))
    print(json.dumps(dict(games=len(configs), teacher_successes=sum(r["success"] for r in teacher),
                          learned_successes=sum(r["success"] for r in learned),
                          learned_truncations=sum(r["truncated"] for r in learned))))


if __name__ == "__main__": main()
