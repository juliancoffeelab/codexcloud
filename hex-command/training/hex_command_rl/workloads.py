"""Explicit workloads; seed ranges separate train/validation/closed-loop evaluation."""
import json

FACTIONS = ("human", "skeletons", "woodland", "coastal")
MAPS = ("open", "wooded", "crossings", "islands", "wood-edge")


def arena(seeds, teacher="balanced", opponent="balanced"):
    return [dict(seed=seed, armies=dict(player=faction, enemy=FACTIONS[(i+1) % 4]),
                 landscape=landscape, player=teacher, enemy=opponent,
                 first=first, maxRounds=30, recruitLimit=None)
            for seed in seeds for landscape in MAPS
            for i, faction in enumerate(FACTIONS) for first in ("player", "enemy")]


def encode(configs):
    return [json.dumps(c) for c in configs]
