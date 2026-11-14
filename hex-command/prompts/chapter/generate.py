#!/usr/bin/env python3
"""Sample chapter ingredients, not stories or compatible objective/event sequences.

Run from the project root:
    python prompts/chapter/generate.py --count 10 --seed 42 --output tmp/chapters

Without --output, print JSON. With --output, save sample.json and table.md in a
new directory. Counts are inline: this script does not read research in tmp/.

The coordinator supplies the existing situation and subjects. Luna connects the
ingredients and may flag incompatible combinations. Independent marginals do
not establish causality, ordering, correlations, story kinds or engine support.
"""

import argparse
from collections import Counter
import json
import math
from pathlib import Path
import random


# Sole winning objectives, not the total-task column: optional/intermediate
# tasks cannot silently become the chapter's finishing condition.
# Source: design-campaign/reference/objective_distribution.md, 272 chapters.
OBJECTIVES = {
    "Defeat enemies": 139,
    "Army escape": 34,
    "Hold / survive / protect": 22,
    "Reposition units to X": 10,
    "Control territory / gather wealth": 3,
    "Acquire / deliver items": 4,
    "Free / rescue people": 2,
    "Evacuate units": 2,
    "Capture units": 2,
    "Explore / find someone": 3,
    "Transport cargo / materials": 2,
    "Operate / destroy structures": 2,
    "Scripted / tutorial tasks": 1,
}

# One primary chapter-ending result per battle template, 250 chapters.
# Ten route-dependent records have no single categorized result; exclude them
# and normalize the remaining 240, rather than inventing a fork or its result.
RESULTS = {
    "Passage / arrival": 40,
    "Safety / defense": 37,
    "Enemy / threat removed": 37,
    "Forces / allies": 28,
    "Escape / withdrawal": 26,
    "Item / material assets": 18,
    "People freed / recovered": 14,
    "Territory / position": 14,
    "Information / discovery": 11,
    "Capability / training": 7,
    "Authority / political status": 4,
    "Setback / no durable gain": 4,
}

# Chapters containing each event, not event-beat counts. Events overlap.
# Updated recount: normal battle openings are not hostility events.
EVENT_DENOMINATOR = 250
EVENTS = {
    "Discovery / revelation": 141,
    "Negotiation": 54,
    "Choice": 36,
    "New orders": 36,
    "Retreat": 73,
    "Surrender": 11,
    "Departure": 44,
    "Our reinforcements arrive": 25,
    "Enemy reinforcements arrive": 58,
    "Third-party reinforcements arrive": 1,
    "Neutral force becomes hostile": 8,
    "Local combat encounter": 18,
    "Ambush": 12,
    "Gaining an ally": 84,
    "Gaining an item / resource": 69,
    "Transformation / ability change": 58,
    "Capture": 17,
    "Rescue": 28,
    "Release": 7,
    "Terrain / environment change": 44,
    "Character death": 48,
    "Sacrifice": 7,
    "Betrayal / broken trust": 9,
}


def shuffled_bag(weights, count, rng):
    """Same oversized/rounded/shuffled/cut method as the enemy sampler."""
    target = count + max(2, math.ceil(math.sqrt(count)))
    total = sum(weights.values())
    while True:
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
        "bag_counts": amounts,
        "selected_counts": dict(Counter(selected)),
    }


def sample(count, seed):
    if count < 1:
        raise ValueError("count must be positive")
    rng = random.Random(seed)
    objectives, objective_audit = shuffled_bag(OBJECTIVES, count, rng)
    results, result_audit = shuffled_bag(RESULTS, count, rng)
    chapters = [
        {"id": i + 1, "finishing_objective": objective, "primary_result": result,
         "events": []}
        for i, (objective, result) in enumerate(zip(objectives, results))
    ]
    event_audits = {}
    for event, frequency in EVENTS.items():
        present, event_audits[event] = shuffled_bag(
            {"present": frequency, "absent": EVENT_DENOMINATOR - frequency},
            count, rng,
        )
        for chapter, value in zip(chapters, present):
            if value == "present":
                chapter["events"].append(event)
    # The event list is an unordered set of ingredients, not chronology. Shuffle
    # its display too, so the declaration order cannot suggest a story sequence.
    for chapter in chapters:
        rng.shuffle(chapter["events"])
    return {
        "seed": seed,
        "count": count,
        "method": "Independent oversized weighted bags, shuffled, cut and zipped; "
                  "each event has its own present/absent bag.",
        "scope": {
            "objective_reference": "272 battle chapters; 226 sole winning tasks",
            "result_reference": "250 battle chapters; 240 categorized primary results",
            "event_reference": "250 battle chapters; chapter incidence, not beat counts",
            "excluded": {"route_dependent_results": 10, "other_unclear_event_chapters": 6},
            "events": "Conditional/optional authored prevalence, not observed per-playthrough probability. "
                      "No fixed event count, forced minimum, ordering or duplicate event kinds per row.",
            "interpretation": "Unfiltered ingredients for alternatives from an established situation, "
                              "not a validated chapter or mandatory script. Actor, target and result "
                              "must be grounded by the designer; incompatible combinations can be rejected.",
        },
        "bags": {"finishing_objective": objective_audit, "primary_result": result_audit,
                 "events": event_audits},
        "chapters": chapters,
    }


def render_table(result):
    lines = [
        "| # | Finishing objective | Primary result | Event ingredients (unordered) |",
        "|---|---|---|---|",
    ]
    for row in result["chapters"]:
        lines.append(
            f"| {row['id']} | {row['finishing_objective']} | {row['primary_result']} | "
            f"{'; '.join(row['events']) or '-'} |"
        )
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--seed", type=int, help="Reproduce a batch; otherwise use a fresh seed")
    parser.add_argument("--output", type=Path, help="New directory for sample.json and table.md")
    args = parser.parse_args()
    if args.count < 1:
        parser.error("--count must be positive")
    if args.output and args.output.exists():
        parser.error("--output must be a new directory; existing files will not be overwritten")
    seed = args.seed if args.seed is not None else random.SystemRandom().getrandbits(64)
    result = sample(args.count, seed)
    serialized = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.mkdir(parents=True)
        (args.output / "sample.json").write_text(serialized, encoding="utf-8")
        (args.output / "table.md").write_text(render_table(result), encoding="utf-8")
        print(f"Seed: {seed}\n{args.output / 'sample.json'}\n{args.output / 'table.md'}")
    else:
        print(serialized, end="")


if __name__ == "__main__":
    main()
