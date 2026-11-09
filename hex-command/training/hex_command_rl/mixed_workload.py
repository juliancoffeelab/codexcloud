"""Hierarchical arena/campaign workloads with disjoint seed partitions."""

import argparse
import hashlib
import json
import os
from pathlib import Path
from .representative import prepare_catalog, AUTHORED
from .workloads import FACTIONS
from .recovery import atomic_json

MAPS = ("open", "river-valley", "villages", "sluice", "crossings")
VARIANTS = {
    "lantern-road": [
        ("flight", "unrestricted"),
        ("fight", "unrestricted"),
        ("authored", "unrestricted"),
    ],
    "borrowed-tide": [("authored", "unrestricted")],
    "green-accord": [("authored", "unrestricted")],
    "ashen-oath": [("authored", "unrestricted"), ("authored", "diplomatic")],
}
OPPONENTS = ("classic", "predictor")
RECRUITERS = ("situational", "predictor")


def group(c):
    rec = c["playerBlocks"]["recruiter"]
    return (
        ("campaign", c["campaign"], c["objective"], c["diplomacy"], c["enemy"], rec)
        if "campaign" in c
        else ("arena", c["landscape"], c["objective"], c["enemy"], rec)
    )


def mass(c):
    return (
        0.5 / 4 / len(VARIANTS[c["campaign"]]) / 4
        if "campaign" in c
        else 0.5 / 5 / 2 / 4
    )


def configs(split):
    result = []
    arena_start = {"train": 1000001, "validation": 3000001, "test": 5000001}[split]
    campaign_start = {"train": 2000001, "validation": 4000001, "test": 6000001}[split]
    for name in MAPS:
        pairs = (
            [("human", "skeletons")]
            if name == "villages"
            else [("coastal", "human")]
            if name == "sluice"
            else [(f, FACTIONS[(i + 1) % 4]) for i, f in enumerate(FACTIONS)]
        )
        n = (
            8 * (4 if name in AUTHORED else 1)
            if split == "train"
            else (4 if name in AUTHORED else 1)
            if split == "validation"
            else 3 * (4 if name in AUTHORED else 1)
        )
        for objective in ["deathmatch", "territory"]:
            for enemy in OPPONENTS:
                for recruiter in RECRUITERS:
                    for seed in range(arena_start, arena_start + n):
                        for player, foe in pairs:
                            for first in ["player", "enemy"]:
                                result.append(
                                    dict(
                                        seed=seed,
                                        armies=dict(player=player, enemy=foe),
                                        landscape=name,
                                        player="balanced",
                                        enemy=enemy,
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
    for campaign, variants in VARIANTS.items():
        n = {"train": 24, "validation": 4, "test": 25}[split]
        for objective, diplomacy in variants:
            for enemy in OPPONENTS:
                for recruiter in RECRUITERS:
                    for seed in range(campaign_start, campaign_start + n):
                        result.append(
                            dict(
                                seed=seed,
                                campaign=campaign,
                                diplomacy=diplomacy,
                                objective=objective,
                                player="balanced",
                                enemy=enemy,
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
