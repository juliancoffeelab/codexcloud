"""Explicit workloads; seed ranges separate train/validation/closed-loop evaluation."""
import json

FACTIONS = ("human", "skeletons", "woodland", "coastal")
MAPS = ("open", "wooded", "crossings", "islands", "wood-edge")


def arena(seeds, teacher="balanced", opponent="balanced", rounds=30, all_pairs=False):
    return [dict(seed=seed, armies=dict(player=faction, enemy=enemy),
                 landscape=landscape, player=teacher, enemy=opponent,
                 first=first, maxRounds=rounds, recruitLimit=None, objective="deathmatch")
            for seed in seeds for landscape in MAPS
            for i, faction in enumerate(FACTIONS)
            for enemy in (FACTIONS if all_pairs else (FACTIONS[(i+1) % 4],))
            if enemy != faction for first in ("player", "enemy")]


def encode(configs):
    return [json.dumps(c) for c in configs]
