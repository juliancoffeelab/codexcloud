"""Hierarchical arena/campaign workloads with disjoint seed partitions."""

import argparse
import hashlib
import json
import os
from collections import Counter
from functools import lru_cache
from pathlib import Path
from .representative import prepare_catalog
from .recovery import atomic_json

FULL_OPPONENT_BLOCKS = dict(recruiter="predictor", campaign="mission", planner="full")


def opponent_config(name):
    if name == "full":
        return dict(enemy="predictor", enemyBlocks=FULL_OPPONENT_BLOCKS)
    return dict(enemy=name)


@lru_cache(maxsize=8)
def _definition(catalog):
    import hex_command_training as native

    return json.loads(native.training_workload())


def definition():
    return _definition(os.environ.get("HEX_COMMAND_DATA_ROOT", ""))


def balance_description(spec):
    budgets = spec["samplingWeights"]
    total = sum(budgets.values())
    return ("Base decision budgets: " + " / ".join(
        f"{100 * value / total:g}% {pool}" for pool, value in budgets.items()
    ) + "; authored map weights and equal conditions within maps; adaptive PPO within pools")


def group(c):
    rec = c.get("playerBlocks", {}).get("recruiter", c["player"])
    enemy = (
        "full"
        if c["enemy"] == "predictor" and c.get("enemyBlocks") == FULL_OPPONENT_BLOCKS
        else c["enemy"]
    )
    return (
        ("campaign", c["campaign"], c["objective"], c["diplomacy"], enemy, rec)
        if "campaign" in c
        else ("arena", c["landscape"], c["objective"], enemy, rec)
    )


def variant_opponents(spec, variant):
    names = variant.get("opponents")
    if names is None:
        names = spec["opponents"]
    if (
        not isinstance(names, list)
        or not names
        or not all(isinstance(name, str) for name in names)
        or len(set(names)) != len(names)
        or not set(names).issubset(spec["opponents"])
    ):
        raise ValueError("opponents must be a nonempty, unique subset of global opponents")
    return names


def variant_for(config, spec):
    enemy = group(config)[4]
    matches = [v for v in spec["campaigns"][config["campaign"]]
               if v["objective"] == config["objective"] and v["diplomacy"] == config["diplomacy"]
               and enemy in variant_opponents(spec, v)]
    if len(matches) != 1:
        raise ValueError("training condition must match exactly one campaign variant")
    return matches[0]


def entry_for(config, spec):
    if "campaign" in config:
        return variant_for(config, spec)
    return next(a for a in spec["arenas"] if a["id"] == config["landscape"])


def task(config, spec):
    """Logical map identity, shared by physical visibility variants."""
    if config.get("campaign") in spec["campaigns"] or any(
        a["id"] == config.get("landscape") for a in spec.get("arenas", [])
    ):
        return entry_for(config, spec)["samplingMap"]
    # Evaluation-only layouts have no training allocation declarations.
    return group(config)[1]


def reward_rules(config, spec=None):
    spec = definition() if spec is None else spec
    return spec["rewardProfiles"][entry_for(config, spec)["rewardProfile"]]


def sampling_group(config, spec=None):
    spec = definition() if spec is None else spec
    if "campaign" not in config:
        arena = next((a for a in spec.get("arenas", []) if a["id"] == config["landscape"]), None)
        if arena is not None:
            return arena["samplingGroup"]
        return "arena"
    if config["campaign"] not in spec["campaigns"]:
        # Evaluation-only authored campaigns retain the broad-campaign budget.
        # They never acquire an imitation teacher or enter the training schedule.
        return "campaign"
    return variant_for(config, spec)["samplingGroup"]


def pretrain_imitator(config, spec=None):
    spec = definition() if spec is None else spec
    entry = entry_for(config, spec)
    teacher = entry["pretrain_imitator"]
    if isinstance(teacher, str) and teacher in ("classic", "queued", "full", "objective_progress", "visible_target_path"):
        return teacher
    if isinstance(teacher, dict) and set(teacher) == {"path_to"}:
        return teacher
    if isinstance(teacher, dict) and set(teacher) == {"tactical"}:
        return teacher
    if isinstance(teacher, dict) and set(teacher) == {"recruiter", "campaign", "planner"}:
        return teacher
    raise ValueError("unknown pretrain_imitator")


@lru_cache(maxsize=8)
def _masses(serialized_spec):
    spec = json.loads(serialized_spec)
    return weights(configs("validation", spec, campaign_seeds=1), spec)


def mass(c):
    return _masses(json.dumps(definition(), sort_keys=True))[group(c)]


def weights(cases, spec=None, *, evaluation=False):
    """Authored pool budgets and map weights, equal conditions per logical map."""
    spec = definition() if spec is None else spec
    templates = {group(c): c for c in cases}
    buckets = {k: sampling_group(c, spec) for k, c in templates.items()}
    active = set(buckets.values())
    # Fixed evaluation gives each represented family equal mass; training
    # curriculum changes must not move the checkpoint-selection goalposts.
    budget = {b: 1.0 for b in active} if evaluation else spec["samplingWeights"]
    total = sum(budget[b] for b in active)
    maps = {k: task(c, spec) for k, c in templates.items()}
    tasks = {b: {maps[k] for k in templates if buckets[k] == b} for b in active}
    def map_weight(name):
        return 1.0 if evaluation else spec["mapWeights"].get(name, 1.0)
    totals = {b: sum(map_weight(m) for m in tasks[b]) for b in active}
    condition_weights = {k: (1.0 if evaluation else entry_for(c, spec)["conditionWeight"])
                         for k, c in templates.items()
                         if c.get("campaign") in spec["campaigns"] or any(
                             a["id"] == c.get("landscape") for a in spec.get("arenas", []))}
    condition_weights = {k: condition_weights.get(k, 1.0) for k in templates}
    conditions = Counter()
    for k in templates:
        conditions[buckets[k], maps[k]] += condition_weights[k]
    return {k: budget[buckets[k]] / total * map_weight(maps[k]) /
            totals[buckets[k]] * condition_weights[k] / conditions[buckets[k], maps[k]]
            for k in templates}


def configs(split, spec=None, campaign_seeds=None):
    result = []
    arena_start = {"train": 1000001, "validation": 3000001, "test": 5000001}[split]
    campaign_start = {"train": 2000001, "validation": 4000001, "test": 6000001}[split]
    spec = definition() if spec is None else spec
    opponents = [opponent_config(name) for name in spec["opponents"]]
    max_pairs = max(len(a["pairs"]) for a in spec["arenas"])
    for arena in spec["arenas"]:
        name = arena["id"]
        pairs = [(p["player"], p["enemy"]) for p in arena["pairs"]]
        multiplier = max_pairs // len(pairs)
        n = {"train": 8, "validation": 1, "test": 3}[split] * multiplier
        conditions = arena.get("conditions")
        objectives = conditions["objectives"] if conditions else spec["objectives"]
        enemies = [opponent_config(o) for o in conditions["opponents"]] if conditions else opponents
        recruiters = conditions["recruiters"] if conditions else spec["recruiters"]
        for objective in objectives:
            for opponent in enemies:
                for recruiter in recruiters:
                    for seed in range(arena_start, arena_start + n):
                        for player, foe in pairs:
                            for first in conditions["first"] if conditions else ["player", "ai"]:
                                result.append(
                                    dict(
                                        seed=seed,
                                        armies=dict(player=player, enemy=foe),
                                        landscape=name,
                                        player="balanced",
                                        **opponent,
                                        playerBlocks=dict(
                                            recruiter=recruiter,
                                            campaign="mission",
                                            planner="queued",
                                        ),
                                        first=first,
                                        maxRounds=conditions["maxRounds"] if conditions else (30 if split == "train" else 100),
                                        recruitLimit=conditions["recruitLimit"] if conditions else None,
                                        objective=objective,
                                    )
                                )
    for campaign, variants in spec["campaigns"].items():
        n = campaign_seeds or {"train": 24, "validation": 4, "test": 25}[split]
        for variant in variants:
            objective, diplomacy = variant["objective"], variant["diplomacy"]
            for name in variant_opponents(spec, variant):
                opponent = opponent_config(name)
                for recruiter in variant.get("recruiters") or spec["recruiters"]:
                    for seed in range(campaign_start, campaign_start + n):
                        result.append(
                            dict(
                                seed=seed,
                                campaign=campaign,
                                diplomacy=diplomacy,
                                objective=objective,
                                player="balanced",
                                **opponent,
                                playerBlocks=dict(
                                    recruiter=recruiter,
                                    campaign="mission",
                                    planner="queued",
                                ),
                                maxRounds=100,
                                recruitLimit=variant.get("recruitLimit"),
                            )
                        )
    return result


def validation():
    # Fixed inexpensive panel; all families, variants, opponents and recruiters.
    cases = configs("validation")
    selected = []
    seen = set()
    for c in cases:
        k = group(c)
        if k not in seen:
            selected.append(c)
            seen.add(k)
    return selected


def prepare(root):
    import hex_command_training as native

    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    catalog = root / "catalog"
    if not catalog.exists():
        prepare_catalog("rust/data", catalog)
    os.environ["HEX_COMMAND_DATA_ROOT"] = str(catalog.resolve())
    spec = dict(
        catalog=str(catalog.resolve()),
        fingerprints=json.loads(native.fingerprints()),
        train=configs("train"),
        validation=configs("validation"),
        imitation_validation=configs("validation"),
        periodic=validation(),
        test=configs("test"),
        balance=balance_description(definition()),
        sampling_weights=definition()["samplingWeights"],
        map_weights=definition()["mapWeights"],
        reward_profiles=definition()["rewardProfiles"],
        adaptive_sampling=definition()["adaptiveSampling"],
    )
    spec["pretrain_imitators"] = {
        split: [pretrain_imitator(c) for c in spec[split]]
        for split in ("train", "imitation_validation")
    }
    spec["workload_sha256"] = hashlib.sha256(
        json.dumps(
            {k: spec[k] for k in ["train", "validation", "periodic", "test", "imitation_validation", "pretrain_imitators", "sampling_weights", "map_weights", "reward_profiles", "adaptive_sampling"]},
            sort_keys=True,
        ).encode()
    ).hexdigest()
    atomic_json(root / "workload.json", spec)
    return spec


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    a = p.parse_args()
    s = prepare(a.root)
    print(
        json.dumps({k: len(s[k]) for k in ["train", "validation", "periodic", "test"]})
    )


if __name__ == "__main__":
    main()
