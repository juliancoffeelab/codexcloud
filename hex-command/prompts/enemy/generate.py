#!/usr/bin/env python3
"""Generate shuffled enemy ingredients and a ready-to-send Luna prompt.

Each weighted column gets an independent oversized bag: round expected counts
of at least one, chance-roll smaller counts, shuffle, cut, then zip columns.
Auxiliary inclusion is drawn per enemy; its kind is drawn only when included.
Organization and presentation use separate marginals, so historical zero joint
cells do not prohibit combinations. Output is unfiltered ingredients.

Provenance: background weights cover 16 sole/allied-main enemy records; goals
cover 24 primary, three auxiliary and six means occurrences. Organization and
presentation each cover 19 campaigns. These are different samples, not a joint
model. Auxiliary inclusion uses three of 19 campaigns as a provisional proxy,
not a measured per-enemy frequency. One means per skeleton is a generation choice.
Raids / pillage inherits an earlier acquisition category's weight, not a new
raid census.

Weights are inline; only the standard library is used, with no tmp/ dependencies.
Definitions and research context live under
.agents/skills/design-campaign/reference/{opposition,motive}_distribution.md
relative to the project root.
"""

import argparse
from collections import Counter
import json
import math
from pathlib import Path
import random
import sys


HERE = Path(__file__).resolve().parent
WEIGHTS = {
    "background": {
        "Magical practitioner": 5,
        "Ruler / chief": 6,
        "Outlaw / pirate / slaver": 1,
        "Fighter / military commander": 2,
        "Noble / claimant": 1,
        "Monster": 1,
    },
    "primary_goal": {
        "Conquer / subjugate": 5,
        "Seize leadership": 3,
        "Retain rule": 3,
        "Acquire wealth": 3,
        "Magical power / knowledge": 3,
        "Revenge against someone": 3,
        "Defend people / possessions": 3,
        "Raids / pillage": 1,
    },
    "means": {"Conquer / subjugate": 2, "Raids / pillage": 4},
    "opposition": {
        "Principal individual": 11,
        "Regime": 1,
        "Coalition": 1,
        "Successive enemies": 2,
        "Independent opponents": 2,
        "External and personal conflict": 2,
    },
    "presentation": {
        "Open antagonist": 10,
        "Concealed manipulator": 2,
        "Hidden antagonist": 2,
        "Mixed / changing": 3,
        "Unconnected mix": 2,
    },
}
AUXILIARY_WEIGHTS = {
    "Conquer / subjugate": 1,
    "Acquire wealth": 1,
    "Magical power / knowledge": 1,
}
# Campaign-level incidence is a provisional proxy, not a per-enemy estimate.
AUXILIARY_PROBABILITY = 3 / 19


def shuffled_bag(weights, count, rng):
    target = count + max(2, math.ceil(math.sqrt(count)))
    while True:
        total = sum(weights.values())
        expected = {kind: target * weight / total for kind, weight in weights.items()}
        counts = {
            kind: int(rng.random() < amount) if amount < 1 else math.floor(amount + 0.5)
            for kind, amount in expected.items()
        }
        bag = [kind for kind, amount in counts.items() for _ in range(amount)]
        if len(bag) >= count:
            break
        target += count - len(bag) + 1
    rng.shuffle(bag)
    selected = bag[:count]
    return selected, {
        "reference_counts": weights,
        "target": target,
        "expected_counts": expected,
        "bag_counts": counts,
        "bag_size": len(bag),
        "selected_counts": dict(Counter(selected)),
    }


def sample(count, seed):
    rng = random.Random(seed)
    bags = {}
    audits = {}
    for field, weights in WEIGHTS.items():
        bags[field], audits[field] = shuffled_bag(weights, count, rng)
    auxiliary_pool = [
        kind for kind, weight in AUXILIARY_WEIGHTS.items() for _ in range(weight)
    ]
    enemies = []
    for values in zip(*(bags[field] for field in WEIGHTS)):
        enemy = dict(zip(WEIGHTS, values))
        enemy["auxiliary_goal"] = (
            rng.choice(auxiliary_pool) if rng.random() < AUXILIARY_PROBABILITY else None
        )
        enemies.append(enemy)
    return {
        "seed": seed,
        "count": count,
        "method": "Independent oversized weighted bags, shuffled, cut, then zipped; auxiliary drawn per enemy.",
        "auxiliary_probability": AUXILIARY_PROBABILITY,
        "bags": audits,
        "enemies": enemies,
    }


def render_prompt(result, skill_path, player_race=None):
    fields = [
        "background", "primary_goal", "auxiliary_goal", "means", "opposition", "presentation"
    ]
    lines = [
        "| # | Background | Goal | Aux | Means | Organization | Presentation |",
        "|---|---|---|---|---|---|---|",
    ]
    for index, enemy in enumerate(result["enemies"], 1):
        cells = [str(index), *(enemy[field] or "-" for field in fields)]
        lines.append("| " + " | ".join(cells) + " |")
    words = {
        1: "one", 2: "two", 3: "three", 4: "four", 5: "five",
        6: "six", 7: "seven", 8: "eight", 9: "nine", 10: "ten",
    }
    return (
        (HERE / "prompt.md").read_text()
        .replace("{{skill_path}}", str(skill_path))
        .replace("{{race}}", player_race or "an unspecified race")
        .replace("{{count}}", words.get(result["count"], str(result["count"])))
        .replace("{{table}}", "\n".join(lines))
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--race", help="Player race, when specified")
    parser.add_argument("--seed", type=int, help="Reproduce a batch; defaults to a fresh seed")
    parser.add_argument("--output", type=Path, help="Save prompt.md and sample.json in this directory")
    parser.add_argument(
        "--skill-path", type=Path,
        default=HERE.parents[1] / ".agents/skills/design-campaign/world-design.md",
    )
    args = parser.parse_args()
    if args.count < 1:
        parser.error("--count must be positive")
    skill_path = args.skill_path.resolve()
    if not skill_path.is_file():
        parser.error(f"Campaign design rules not found: {skill_path}")
    seed = args.seed if args.seed is not None else random.SystemRandom().getrandbits(64)
    result = sample(args.count, seed)
    prompt = render_prompt(result, skill_path, args.race)
    if args.output:
        output = args.output.resolve()
        if output == HERE:
            parser.error("Choose an output directory other than the prompt template directory")
        output.mkdir(parents=True, exist_ok=True)
        (output / "sample.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        (output / "prompt.md").write_text(prompt)
        print(f"Seed: {seed}\n{output / 'sample.json'}\n{output / 'prompt.md'}")
    else:
        print(f"Seed: {seed}", file=sys.stderr)
        print(prompt, end="")


if __name__ == "__main__":
    main()
