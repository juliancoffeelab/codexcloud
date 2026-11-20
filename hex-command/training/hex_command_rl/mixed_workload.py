"""Hierarchical arena/campaign workloads with disjoint seed partitions."""

import argparse
import hashlib
import json
import os
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


def mass(c):
    spec = definition()
    axes = len(spec["opponents"]) * len(spec["recruiters"])
    return (
        0.5 / len(spec["campaigns"]) / len(spec["campaigns"][c["campaign"]]) / axes
        if "campaign" in c
        else 0.5 / len(spec["arenas"]) / len(spec["objectives"]) / axes
    )


def weights(cases):
    """Equal family/task budgets, then split each task's actual variant groups."""
    keys = set(group(c) for c in cases)
    families = {k[0] for k in keys}
    tasks = {f: {k[1] for k in keys if k[0] == f} for f in families}
    return {
        k: 1 / len(families) / len(tasks[k[0]]) / sum(v[:2] == k[:2] for v in keys)
        for k in keys
    }


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
        for objective in spec["objectives"]:
            for opponent in opponents:
                for recruiter in spec["recruiters"]:
                    for seed in range(arena_start, arena_start + n):
                        for player, foe in pairs:
                            for first in ["player", "ai"]:
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
                                        maxRounds=30 if split == "train" else 100,
                                        recruitLimit=None,
                                        objective=objective,
                                    )
                                )
    for campaign, variants in spec["campaigns"].items():
        n = campaign_seeds or {"train": 24, "validation": 4, "test": 25}[split]
        for variant in variants:
            objective, diplomacy = variant["objective"], variant["diplomacy"]
            for opponent in opponents:
                for recruiter in spec["recruiters"]:
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
                                recruitLimit=None,
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
        periodic=validation(),
        test=configs("test"),
        balance="50% arenas / 50% campaigns; uniform maps/campaigns, then variants/opponents/recruiters",
    )
    spec["workload_sha256"] = hashlib.sha256(
        json.dumps(
            {k: spec[k] for k in ["train", "validation", "periodic", "test"]},
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
