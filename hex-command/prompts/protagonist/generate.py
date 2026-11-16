#!/usr/bin/env python3
"""Sample protagonist ingredients and render a Luna prompt against a selected enemy.

Background, faction, primary goal and means count use independent oversized
weighted bags, shuffled and cut before zipping rows. Means kinds use quotas for
the actual slot budget without within-row repeats; feasibility and rounding
adjustments are reported in the audit. Auxiliary inclusion uses 4/19, including
one interpretive source case, as a provisional per-protagonist proxy; included
kinds follow the four auxiliary occurrences.

The reference covers 19 focal protagonists, 32 primary-goal and 44 means-kind
occurrences. One primary goal per generated row is a design choice; source
purposes can evolve. Raids / pillage replaces Captives / servants / bodies with
its inherited weight, not a new raid census. These are separate marginals,
not a measured joint model or validated story.

Weights are inline; no research files in tmp/ are read. Definitions live in
.agents/skills/design-campaign/reference/protagonist_distribution.md relative
to the project root.
"""
import argparse
from collections import Counter
import json
import math
from pathlib import Path
import random
import sys

if __package__:
    from .means import assign_means
else:
    from means import assign_means

HERE = Path(__file__).resolve().parent
WEIGHTS = {
    "background": {
        "Officer / captain": 4,
        "Commoner / local leader": 4,
        "Ruler / chief": 3,
        "Successor / claimant": 3,
        "Scholar / mage": 2,
        "Noble": 1,
        "Trader / commission organizer": 1,
        "Hunter / aspiring war-leader": 1,
    },
    "faction": {
        "People / community": 11,
        "Military group": 4,
        "Professional company / commission": 2,
        "Dynastic rebel coalition": 1,
        "Research expedition": 1,
    },
    "goal": {
        "Defend people / possessions": 14,
        "Revenge against someone": 2,
        "Gain recognition / acceptance": 2,
        "Magical power / knowledge": 1,
        "Explore / discover / uncover truth": 2,
        "Seize leadership": 1,
        "Win freedom / independence": 3,
        "Acquire wealth": 1,
        "Create an artifact": 1,
        "Preserve / extend own life": 1,
        "Retain rule": 1,
        "Find / establish a new home": 3,
    },
    "means_count": {1: 4, 2: 6, 3: 8, 4: 1},
}
MEANS_WEIGHTS = {
    "Gain forces / allies": 15,
    "Gain expertise": 8,
    "Magical power / knowledge": 9,
    "Raids / pillage": 2,
    "Acquire wealth": 4,
    "Seize leadership": 2,
    "Conquer / subjugate": 2,
    "Explore / discover / uncover truth": 2,
}
AUXILIARY_WEIGHTS = {
    "Defend people / possessions": 1,
    "Revenge against someone": 1,
    "Gain recognition / acceptance": 1,
    "Explore / discover / uncover truth": 1,
}
AUXILIARY_PROBABILITY = 4 / 19


def shuffled_bag(weights, count, rng):
    """Oversize, round common cards, chance-roll rare cards, shuffle and cut."""
    target = count + max(2, math.ceil(math.sqrt(count)))
    while True:
        total = sum(weights.values())
        expected = {kind: target * weight / total for kind, weight in weights.items()}
        amounts = {
            kind: int(rng.random() < amount) if amount < 1 else math.floor(amount + 0.5)
            for kind, amount in expected.items()
        }
        bag = [kind for kind, amount in amounts.items() for _ in range(amount)]
        if len(bag) >= count:
            break
        target += count - len(bag) + 1
    rng.shuffle(bag)
    selected = bag[:count]
    return selected, {
        "reference_counts": weights,
        "target": target,
        "expected_counts": expected,
        "bag_counts": amounts,
        "selected_counts": dict(Counter(selected)),
    }


def sample(count, seed):
    if count < 1:
        raise ValueError("count must be positive")
    rng = random.Random(seed)
    bags, audits = {}, {}
    for field, weights in WEIGHTS.items():
        bags[field], audits[field] = shuffled_bag(weights, count, rng)
    means, means_audit = assign_means(MEANS_WEIGHTS, bags["means_count"], rng)
    auxiliary_pool = [kind for kind, weight in AUXILIARY_WEIGHTS.items() for _ in range(weight)]
    rows = []
    for i in range(count):
        row = {field: bag[i] for field, bag in bags.items()}
        row["means"] = means[i]
        row["auxiliary_goal"] = rng.choice(auxiliary_pool) if rng.random() < AUXILIARY_PROBABILITY else None
        rows.append(row)
    return {
        "seed": seed,
        "count": count,
        "method": "Independent shuffled bags for background, faction, goal and means count; fixed means quotas assigned without within-row repeats; auxiliary drawn per protagonist.",
        "scope": {
            "sample": "19 focal protagonists and their player factions; 32 primary, 4 auxiliary and 44 means-kind occurrences.",
            "means": "Counts describe distinct means kinds across a campaign, not requirements for every chapter. Count and kind distributions are separate.",
            "adaptation": "Raids / pillage is the requested generation-facing replacement of Captives / servants / bodies; its weight is inherited, not a new raid census.",
            "auxiliary": "4/19 campaign incidence, including one interpretive auxiliary, is a provisional per-protagonist proxy.",
            "composition": "One primary goal per generated row. Independently sampled ingredients still need a coherent faction and conflict against the selected enemy.",
        },
        "auxiliary_probability": AUXILIARY_PROBABILITY,
        "bags": audits,
        "means_allocation": means_audit,
        "protagonists": rows,
    }


ENEMY_FIELDS = ("faction", "motives", "means", "actions", "leader", "forces")


def read_enemy(path):
    enemy = json.loads(path.read_text())
    if not isinstance(enemy, dict) or set(enemy) != set(ENEMY_FIELDS):
        raise ValueError(
            "Enemy JSON must contain exactly: " + ", ".join(ENEMY_FIELDS)
        )
    for field in ENEMY_FIELDS:
        if not isinstance(enemy[field], str) or not enemy[field].strip():
            raise ValueError(f"Enemy field {field!r} must be nonempty text")
    return enemy


def render_prompt(result, skill_path, enemy, player_race=None):
    enemy_text = "\n".join(
        f"- {field.capitalize()}: {enemy[field]}" for field in ENEMY_FIELDS
    )
    lines = [
        "| # | Background | Player faction | Goal | Aux | Means |",
        "|---|---|---|---|---|---|",
    ]
    for i, row in enumerate(result["protagonists"], 1):
        cells = [str(i), row["background"], row["faction"], row["goal"],
                 row["auxiliary_goal"] or "-", "; ".join(row["means"])]
        lines.append("| " + " | ".join(cells) + " |")
    words = {1: "one", 2: "two", 3: "three", 4: "four", 5: "five",
             6: "six", 7: "seven", 8: "eight", 9: "nine", 10: "ten"}
    return ((HERE / "prompt.md").read_text()
            .replace("{{skill_path}}", str(skill_path))
            .replace("{{count}}", words.get(result["count"], str(result["count"])))
            .replace("{{enemy}}", enemy_text)
            .replace("{{player_race}}", f"Player commands {player_race}." if player_race else "")
            .replace("{{table}}", "\n".join(lines)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--seed", type=int, help="Reproduce a batch; defaults to a fresh seed")
    parser.add_argument("--enemy", type=Path, required=True,
                        help="Selected enemy description-only JSON")
    parser.add_argument("--race", help="Player race, when specified")
    parser.add_argument("--output", type=Path, help="New directory for sample.json and prompt.md")
    parser.add_argument("--skill-path", type=Path,
                        default=HERE.parents[1] / ".agents/skills/design-campaign/world-design.md")
    args = parser.parse_args()
    if args.count < 1:
        parser.error("count must be positive")
    if args.output and args.output.exists():
        parser.error("output must be a new directory")
    if not args.enemy.is_file():
        parser.error(f"Selected enemy not found: {args.enemy}")
    if not args.skill_path.is_file():
        parser.error(f"Campaign design rules not found: {args.skill_path}")
    try:
        enemy = read_enemy(args.enemy)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    seed = args.seed if args.seed is not None else random.SystemRandom().getrandbits(64)
    result = sample(args.count, seed)
    prompt = render_prompt(result, args.skill_path.resolve(), enemy, args.race)
    if args.output:
        args.output.mkdir(parents=True)
        (args.output / "sample.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n")
        (args.output / "prompt.md").write_text(prompt)
        print(f"Seed: {seed}\n{args.output / 'sample.json'}\n{args.output / 'prompt.md'}")
    else:
        print(f"Seed: {seed}", file=sys.stderr)
        print(prompt, end="")


if __name__ == "__main__":
    main()
