"""RL-only training with existing arenas/campaigns held out for selection."""

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
        raise ValueError("training catalog must contain only rl_ tasks")
    races = ["human", "skeletons", "woodland", "coastal"]
    held_out = dict(
        arenas=[
            dict(
                id=name,
                pairs=[
                    dict(player=p, enemy=races[(i + 1) % 4])
                    for i, p in enumerate(races)
                ],
            )
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
    # Reserve the entire coastal layout and chapter sequence for the final test.
    coastal = "rl-coastal-assault"
    if coastal in training["campaigns"]:
        raise ValueError("the coastal campaign is reserved for final evaluation")
    test_panel = {
        **held_out,
        "campaigns": {
            **held_out["campaigns"],
            coastal: [
                dict(objective=o, diplomacy="unrestricted")
                for o in ("authored", "fog")
            ],
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
        balance="50% arena / 50% campaign decisions; equal tasks then variants; coastal campaign final test only",
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
