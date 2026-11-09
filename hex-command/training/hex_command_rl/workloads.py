"""Seed/recruiter dimensions are explicit and preserved in match configurations."""

import json

FACTIONS = ("human", "skeletons", "woodland", "coastal")
MAPS = ("open", "wooded", "crossings", "islands", "wood-edge")
RECRUITERS = ("balanced", "situational", "predictor")


def recruiter_pair(config):
    def kind(side):
        return config.get(side + "Blocks", {}).get("recruiter", config[side])

    a, b = kind("player"), kind("enemy")
    return a if a == b else f"{a}-vs-{b}"


def arena(
    seeds,
    teacher="balanced",
    opponent="classic",
    rounds=30,
    all_pairs=False,
    recruiters=RECRUITERS,
    mirror_recruiters=False,
):
    # Recruiter is an independent axis. None retains each policy's preset blocks.
    configs = []
    for seed in seeds:
        for recruiter in recruiters or (None,):
            for landscape in MAPS:
                for i, faction in enumerate(FACTIONS):
                    for enemy in FACTIONS if all_pairs else (FACTIONS[(i + 1) % 4],):
                        if enemy == faction:
                            continue
                        for first in ("player", "enemy"):
                            c = dict(
                                seed=seed,
                                armies=dict(player=faction, enemy=enemy),
                                landscape=landscape,
                                player=teacher,
                                enemy=opponent,
                                first=first,
                                maxRounds=rounds,
                                recruitLimit=None,
                                objective="deathmatch",
                            )
                            if recruiter is not None:
                                # These three presets share Mission/Queued. Explicit
                                # blocks keep recruitment independent of teacher selection.
                                for side, policy in [("player", teacher)] + (
                                    [("enemy", opponent)] if mirror_recruiters else []
                                ):
                                    planner = (
                                        "classic" if policy == "classic" else "queued"
                                    )
                                    c[side + "Blocks"] = dict(
                                        recruiter=recruiter,
                                        campaign="mission",
                                        planner=planner,
                                    )
                            configs.append(c)
    return configs


def encode(configs):
    return [json.dumps(c) for c in configs]
