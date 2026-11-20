"""RL-only training with separate selection and final evaluation panels."""

import argparse
import hashlib
import json
import os
from pathlib import Path
from .mixed_workload import configs, group
from .recovery import atomic_json
from .representative import prepare_catalog


def prepare(root):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    catalog = root / "catalog"
    if (root / "workload.json").exists():
        raise ValueError("workload already exists; use it unchanged when resuming")
    prepare_catalog("rust/data", catalog)
    os.environ["HEX_COMMAND_DATA_ROOT"] = str(catalog.resolve())
    import hex_command_training as native

    training = json.loads((catalog / "simulation/rl.json").read_text())
    if not all(a["id"].startswith("rl-") for a in training["arenas"]) or not all(
        c.startswith("rl-") for c in training["campaigns"]
    ):
        raise ValueError("training catalog must contain only rl- tasks")
    races = ["human", "skeletons", "woodland", "coastal"]
    cross_faction_pairs = [
        dict(player=p, enemy=races[(i + 1) % len(races)]) for i, p in enumerate(races)
    ]
    held_out = dict(
        arenas=[
            dict(id=name, pairs=cross_faction_pairs)
            for name in (
                "open",
                "wooded",
                "crossings",
                "islands",
                "wood-edge",
                "river-valley",
            )
        ]
        + [
            dict(id="villages", pairs=[dict(player="human", enemy="skeletons")]),
            dict(id="sluice", pairs=[dict(player="coastal", enemy="human")]),
        ],
        campaigns={
            "lantern-road": [
                dict(objective=o, diplomacy="unrestricted")
                for o in ("authored", "flight", "fight")
            ]
        },
        opponents=training["opponents"],
        recruiters=training["recruiters"],
        objectives=training["objectives"],
    )
    # Reserve novel maps and chapter sequences for the final test.
    final_campaigns = ("rl-coastal-assault", "rl-whitebank-relief")
    final_arenas = (
        "alder-fen",
        "drowned-kilns",
        "hart-saddle",
        "herring-road",
        "ochre-wells",
        "siltbar-bend",
    )
    if set(final_campaigns) & training["campaigns"].keys():
        raise ValueError("final-test campaigns cannot be used for training")
    arena_catalog = json.loads((catalog / "simulation/arena.json").read_text())
    layouts = {
        arena["id"]: {variant["map"] for variant in arena["variants"]}
        for arena in arena_catalog["scenarios"]
    }
    exposed_layouts = set().union(
        *(layouts[arena["id"]] for arena in training["arenas"] + held_out["arenas"])
    )
    if exposed_layouts & set().union(*(layouts[name] for name in final_arenas)):
        raise ValueError(
            "final-test arenas cannot share layouts with training or validation"
        )
    test_panel = {
        **held_out,
        "arenas": held_out["arenas"]
        + [dict(id=name, pairs=cross_faction_pairs) for name in final_arenas],
        "campaigns": {
            **held_out["campaigns"],
            **{
                campaign: [
                    dict(objective=o, diplomacy="unrestricted")
                    for o in ("authored", "fog")
                ]
                for campaign in final_campaigns
            },
        },
    }
    # Campaigns are a small number of correlated routes; use more independent
    # combat seeds before choosing between apparent gains on the tiny panel.
    validation = configs("validation", held_out, campaign_seeds=12)
    periodic, seen = [], set()
    for c in validation:
        key = group(c)
        if key not in seen:
            # Rotate the four races across each map's objective/opponent/recruiter groups.
            if "armies" in c:
                names = [
                    p["player"]
                    for a in held_out["arenas"]
                    if a["id"] == c["landscape"]
                    for p in a["pairs"]
                ]
                if c["armies"]["player"] != names[(len(seen) // 2) % len(names)]:
                    continue
            periodic.append(c)
            seen.add(key)
    spec = dict(
        catalog=str(catalog.resolve()),
        fingerprints=json.loads(native.fingerprints()),
        train=configs("train", training),
        periodic=periodic,
        validation=validation,
        test=configs("test", test_panel),
        balance="50% arena / 50% campaign decisions; equal tasks then variants; novel arena and campaign layouts reserved for the final test",
    )
    spec["workload_sha256"] = hashlib.sha256(
        json.dumps(
            {k: spec[k] for k in ("train", "periodic", "validation", "test")},
            sort_keys=True,
        ).encode()
    ).hexdigest()
    atomic_json(root / "workload.json", spec)
    print(
        json.dumps(
            {k: len(spec[k]) for k in ("train", "periodic", "validation", "test")}
        ),
        flush=True,
    )
    return spec


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--root", type=Path, required=True)
    prepare(p.parse_args().root)


if __name__ == "__main__":
    main()
