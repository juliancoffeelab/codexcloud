#!/usr/bin/env python3
"""Sample story kinds, objective/result trees, and correlated event groups.

Run from the project root:
    python prompts/chapter/generate.py --position opening --count 10 --seed 42 --output tmp/chapters
    python prompts/chapter/generate.py --position middle --previous-position opening \
        --previous-kind "Success, Then Trouble" --count 10 --seed 42

Without --output, print JSON. With --output, save sample.json and table.md in a
new directory. Classified data live beside the scripts;
story-kind counts and transitions are inline. No research files in tmp/ are read.

The coordinator supplies the existing situation and subjects. Luna connects the
ingredients and may flag incompatible combinations. Independent marginals do
not establish causality, objective/event ordering, a coherent arc or engine support.
Objective sequences use conditional walks and results depend on their terminal
objectives. Story kinds use battle-to-battle transitions. Interludes and story forks
are not generated. Choose a candidate, then pass its position and kind as the
previous step to generate the following battle.

Finales use deliberately equal story-kind weights. Event preferences depend on
objectives/results, not story kind. Recorded groups and alternatives describe
authored content, not per-playthrough probabilities. Sparse event correlations
shrink toward their baseline; --event-prior-strength controls that design choice.
The JSON retains methods, source references, adjustments and limitations.
"""

import argparse
from collections import Counter
import json
import math
from pathlib import Path
import random
import sys

if __package__:
    from . import objectives as objective_sampler
    from . import events as event_sampler
else:
    import objectives as objective_sampler
    import events as event_sampler


# Openings use the position census; middle chapters use adjacent battle links.
# Intervening interludes are crossed when finding the next battle, but are not
# sampled. One count per unique authored battle endpoint pair, not a playthrough
# frequency. Dynamic hubs and unresolved successors are excluded, not guessed.
# Source: the scene-aware battle projection of the classified chapter corpus.
# The unsupported Launch opening is omitted from generation.
STORY_KINDS_BY_POSITION = {
    "opening": {
        "Planned progress": 5,
        "Hard-won progress": 1,
        "Success, Then Trouble": 8,
        "Redirection": 2,
        "Setback": 2,
    },
    "middle": {
        "Planned progress": 90,
        "Hard-won progress": 50,
        "Success, Then Trouble": 22,
        "Resolved Detour": 19,
        "Redirection": 11,
        "Fundamental realignment": 7,
        "Setback": 6,
        "Costly victory": 2,
        "False victory": 2,
    },
    # Deliberately equal design weights, not the sparse empirical finale rows.
    "finale": dict.fromkeys((
        "Step Forward", "Resolution", "Fundamental realignment",
        "Costly victory", "Cycle continues", "Irreversible loss",
    ), 1),
}
BATTLE_TRANSITIONS = {
    "opening-middle": {
        "Planned progress": {
            "Hard-won progress": 2,
            "Planned progress": 2,
            "Fundamental realignment": 1
        },
        "Hard-won progress": {
            "Planned progress": 1
        },
        "Success, Then Trouble": {
            "Hard-won progress": 2,
            "Planned progress": 2,
            "Redirection": 2,
            "Setback": 2,
            "Resolved Detour": 1
        },
        "Redirection": {
            "Planned progress": 2
        },
        "Setback": {
            "Planned progress": 2
        }
    },
    "middle-middle": {
        "Planned progress": {
            "Hard-won progress": 25,
            "Planned progress": 23,
            "Success, Then Trouble": 11,
            "Redirection": 3,
            "Resolved Detour": 3,
            "Setback": 2,
            "Costly victory": 1,
            "False victory": 1
        },
        "Hard-won progress": {
            "Planned progress": 22,
            "Hard-won progress": 8,
            "Redirection": 5,
            "Fundamental realignment": 2,
            "Resolved Detour": 2,
            "Success, Then Trouble": 2,
            "Costly victory": 1
        },
        "Success, Then Trouble": {
            "Planned progress": 6,
            "Hard-won progress": 3,
            "Success, Then Trouble": 3,
            "Fundamental realignment": 2,
            "Redirection": 1
        },
        "Resolved Detour": {
            "Planned progress": 2,
            "False victory": 1,
            "Hard-won progress": 1,
            "Redirection": 1,
            "Success, Then Trouble": 1
        },
        "Redirection": {
            "Planned progress": 6,
            "Hard-won progress": 2,
            "Redirection": 2,
            "Success, Then Trouble": 2,
            "Setback": 1
        },
        "Fundamental realignment": {
            "Planned progress": 2,
            "Fundamental realignment": 1,
            "Hard-won progress": 1,
            "Success, Then Trouble": 1
        },
        "Setback": {
            "Planned progress": 4,
            "Hard-won progress": 1
        },
        "False victory": {
            "Planned progress": 1,
            "Setback": 1
        },
        "Costly victory": {
            "Fundamental realignment": 1
        }
    }
}
ENDING_KINDS = {
    "Aftermath": 17,
    "Fundamental realignment": 1,
    "Costly victory": 1,
}
STORY_KIND_DEFINITIONS = {
    "Planned progress": "Complete a planned step toward the existing goal.",
    "Hard-won progress": "The problem proves harder than expected; struggle brings the protagonists closer to solving it.",
    "Success, Then Trouble": "Solve the current problem, but uncover another one.",
    "Resolved Detour": "Encounter and resolve a local problem, then resume the larger pursuit.",
    "Redirection": "New circumstances change the plan or immediate target while the larger purpose remains.",
    "Fundamental realignment": "Established loyalties or purpose change radically and durably.",
    "Setback": "Fail to achieve the intended step or lose ground, though something may be salvaged.",
    "Costly victory": "Genuinely succeed, but the cost or consequences undermine something important.",
    "False victory": "Think the goal was achieved, then discover deception or that the success served the enemy's plan.",
    "Resolution": "Achieve the central goal and settle its conflict.",
    "Cycle continues": "Actions perpetuate the same unresolved situation.",
    "Irreversible loss": "The undertaking ends in irreversible loss rather than its intended fulfillment.",
    "Aftermath": "Show what the completed conflict leaves behind: consequences, changed lives and the world afterward.",
}
STORY_KIND_DEFINITIONS["Step Forward"] = STORY_KIND_DEFINITIONS["Planned progress"]


def shuffled_bag(weights, count, rng):
    """Same oversized/rounded/shuffled/cut method as the enemy sampler."""
    if count == 0:
        return [], {"reference_counts": weights, "target": 0, "bag_counts": {}, "selected_counts": {}}
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


def story_weights(position, previous_position=None, previous_kind=None):
    if position not in STORY_KINDS_BY_POSITION:
        raise ValueError("position must be opening, middle or finale")
    if position == "opening":
        if previous_position is not None or previous_kind is not None:
            raise ValueError("an opening has no previous battle")
        return STORY_KINDS_BY_POSITION[position]
    if previous_position not in ("opening", "middle") or previous_kind is None:
        raise ValueError("middle/finale requires previous_position (opening or middle) and previous_kind")
    if previous_kind not in STORY_KINDS_BY_POSITION[previous_position]:
        raise ValueError(f"{previous_kind!r} is not a {previous_position} battle kind")
    if position == "finale":
        return STORY_KINDS_BY_POSITION[position]
    group = f"{previous_position}-middle"
    if previous_kind not in BATTLE_TRANSITIONS[group]:
        raise ValueError(f"No resolved {group} transitions for {previous_kind!r}; no marginal fallback")
    return BATTLE_TRANSITIONS[group][previous_kind]


def sample(count, seed, position="middle", previous_position=None, previous_kind=None,
           *, event_prior_strength=20):
    if count < 1:
        raise ValueError("count must be positive")
    weights = story_weights(position, previous_position, previous_kind)
    objective_batch = objective_sampler.sample(count, seed)
    event_batch = event_sampler.attach(objective_batch, seed, event_prior_strength)
    chapters = event_batch["chapters"]

    # Story kinds have their own seeded draw stream. Changing event counts does
    # not silently change the battle-to-battle story draw for the same seed.
    rng = random.Random(seed)
    story_kinds, story_audit = shuffled_bag(weights, count, rng)
    for chapter, kind in zip(chapters, story_kinds):
        chapter["story_kind"] = kind
        chapter["story_kind_definition"] = STORY_KIND_DEFINITIONS[kind]
    story_bags = {"story_kind": story_audit}
    if position == "finale":
        ending_kinds, story_bags["ending_kind"] = shuffled_bag(ENDING_KINDS, count, rng)
        for chapter, kind in zip(chapters, ending_kinds):
            chapter["ending_kind"] = kind
            chapter["ending_kind_definition"] = STORY_KIND_DEFINITIONS[kind]
    return {
        "schema_version": 2,
        "seed": seed,
        "count": count,
        "position": position,
        "previous_step": ({"position": previous_position, "story_kind": previous_kind}
                          if previous_kind is not None else None),
        "method": "One chapter pipeline: story transitions, conditional objective/result "
                  "walks, linked event profiles, correlated independent groups and optional encounters.",
        "objective_sampling": {key: value for key, value in objective_batch.items()
                               if key != "objectives"},
        "event_sampling": {key: value for key, value in event_batch.items()
                           if key != "chapters"},
        "scope": {
            "story_kind_reference": "Openings: 18 templates after dropping the unsupported Launch from the 19-template census. Middles: 19 opening-to-middle and 158 middle-to-middle authored battle links, conditioned on the previous battle. Finales: deliberately equal weights.",
            "story_kind": "One overall development pattern per chapter, not an objective or event. Its name and definition accompany every chapter.",
            "ending_reference": "19 campaign closing sequences; sampled separately only for finales. Includes embedded epilogues and the conditional book ending; shared epilogues count once.",
            "story_composition": "Story kind depends on position and previous battle kind, except equal-weight finales. Event correlations currently depend on objectives/results, not story kind. A batch supplies alternatives for one next battle. Interludes are not sampled.",
            "objectives": "Objective trees and terminal-conditioned results use objective_model.json; no independent result marginal is drawn.",
            "events": "Connected classified groups from event_corpus.json. Required-objective links retain establishes/enacts/resolves roles. Independent developments use empirical group counts and softened objective co-occurrence preferences. Optional actions, rewards and encounter alternatives stay connected.",
            "counts": "Event counts retain all 250 classified templates and recorded alternatives. These are authored-content counts, not execution or playthrough probabilities.",
            "alternatives": "Alternative victory conditions and encounter variants do not automatically constitute story forks.",
            "missing_links": "No additional mapped event may mean routine automatic resolution or incomplete annotation; it is not a claim that nothing happens.",
            "conditions": "Source conditionality is retained as provenance. Luna grounds conditions in the new situation; a source casualty does not require inventing a forced sacrifice.",
            "interpretation": "Related ingredients, not a fully specified story or engine-supported scenario. Actors, effects and consequences must be grounded by the designer.",
        },
        "bags": story_bags,
        "chapters": chapters,
    }


def render_chapters(result):
    lines = [f"**Chapter position:** {result['position']}.", ""]
    if result["previous_step"] is not None:
        previous = result["previous_step"]
        lines.extend([f"**Previous battle:** {previous['position']} — {previous['story_kind']}.", ""])
    for row in result["chapters"]:
        lines.extend([f"### {row['id']}. {row['objective_expression']}", "",
                      f"**Story kind:** {row['story_kind']}.",
                      row["story_kind_definition"], "",
                      objective_sampler.render_outcomes(row["outcomes"]), ""])
        lines.extend([event_sampler.render_events(row), ""])
        if "ending_kind" in row:
            lines.extend([f"**Ending kind:** {row['ending_kind']}.",
                          row["ending_kind_definition"], ""])
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--count", type=int, default=10)
    parser.add_argument("--position", choices=tuple(STORY_KINDS_BY_POSITION), default="middle",
                        help="Next battle position (default: middle); opening uses census, middle uses transitions, finale uses equal weights")
    parser.add_argument("--previous-position", choices=("opening", "middle"),
                        help="Position of the previous battle; required for middle/finale")
    parser.add_argument("--previous-kind", choices=tuple(STORY_KIND_DEFINITIONS),
                        help="Story kind of the previous battle; required for middle/finale")
    parser.add_argument("--seed", type=int, help="Reproduce a batch; otherwise use a fresh seed")
    parser.add_argument("--output", type=Path, help="New directory for sample.json and table.md")
    parser.add_argument("--event-prior-strength", type=float, default=20,
                        help="Shrink sparse event correlations toward baseline (default: 20)")
    args = parser.parse_args()
    if args.count < 1:
        parser.error("--count must be positive")
    if args.output and args.output.exists():
        parser.error("--output must be a new directory; existing files will not be overwritten")
    seed = args.seed if args.seed is not None else random.SystemRandom().getrandbits(64)
    try:
        result = sample(args.count, seed, args.position, args.previous_position, args.previous_kind,
                        event_prior_strength=args.event_prior_strength)
    except ValueError as error:
        parser.error(str(error))
    serialized = json.dumps(result, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        args.output.mkdir(parents=True)
        (args.output / "sample.json").write_text(serialized, encoding="utf-8")
        (args.output / "table.md").write_text(render_chapters(result), encoding="utf-8")
        print(f"Seed: {seed}\n{args.output / 'sample.json'}\n{args.output / 'table.md'}")
    else:
        print(serialized, end="")


if __name__ == "__main__":
    main()
