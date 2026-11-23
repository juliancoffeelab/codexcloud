"""Select RL checkpoints on synthetic layouts and showcase on authored maps."""

import argparse
import hashlib
import json
import os
from pathlib import Path
from .mixed_workload import configs, group, pretrain_imitator, balance_description
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
    evaluation = dict(
        opponents=training["opponents"],
        recruiters=training["recruiters"],
        objectives=training["objectives"],
    )
    selection_panel = dict(
        **evaluation,
        arenas=[
            dict(id=name, pairs=cross_faction_pairs)
            for name in (
                "open",
                "wooded",
                "crossings",
                "islands",
                "wood-edge",
                "alder-fen",
                "drowned-kilns",
                "hart-saddle",
                "herring-road",
                "ochre-wells",
                "siltbar-bend",
            )
        ],
        campaigns={
            campaign: [
                dict(objective=o, diplomacy="unrestricted") for o in ("authored", "fog")
            ]
            for campaign in ("rl-coastal-assault", "rl-whitebank-relief")
        },
    )
    showcase_panel = dict(
        **evaluation,
        arenas=[
            dict(id="river-valley", pairs=cross_faction_pairs),
            dict(id="toll-road", pairs=[dict(player="human", enemy="skeletons")]),
            dict(id="a-tide-for-both", pairs=[dict(player="coastal", enemy="human")]),
        ],
        campaigns={
            "lantern-road": [
                dict(objective=o, diplomacy="unrestricted")
                for o in ("authored", "flight", "fight")
            ]
        },
    )
    arena_catalog = json.loads((catalog / "simulation/arena.json").read_text())
    layouts = {
        arena["id"]: {variant["map"] for variant in arena["variants"]}
        for arena in arena_catalog["scenarios"]
    }
    used_layouts, used_campaigns = set(), set()
    for panel in (training, selection_panel, showcase_panel):
        arena_layouts = set().union(*(layouts[a["id"]] for a in panel["arenas"]))
        campaign_ids = panel["campaigns"].keys()
        if arena_layouts & used_layouts or campaign_ids & used_campaigns:
            raise ValueError("training, selection, and showcase must use distinct maps")
        used_layouts.update(arena_layouts)
        used_campaigns.update(campaign_ids)
    # Campaigns are a small number of correlated routes; use more independent
    # combat seeds before choosing between apparent gains on the tiny panel.
    validation = configs("validation", selection_panel, campaign_seeds=12)
    periodic, seen = [], set()
    for c in validation:
        key = group(c)
        if key not in seen:
            # Rotate the four races across each map's objective/opponent/recruiter groups.
            if "armies" in c:
                names = [
                    p["player"]
                    for a in selection_panel["arenas"]
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
        imitation_validation=configs("validation", training),
        sampling_weights=training["samplingWeights"],
        map_weights=training["mapWeights"],
        reward_profiles=training["rewardProfiles"],
        adaptive_sampling=training["adaptiveSampling"],
        test=configs("test", showcase_panel),
        balance=balance_description(training) + "; synthetic selection and authored showcase",
    )
    spec["pretrain_imitators"] = {
        split: [pretrain_imitator(c, training) for c in spec[split]]
        for split in ("train", "imitation_validation")
    }
    spec["workload_sha256"] = hashlib.sha256(
        json.dumps(
            {k: spec[k] for k in ("train", "periodic", "validation", "test", "imitation_validation", "pretrain_imitators", "sampling_weights", "map_weights", "reward_profiles", "adaptive_sampling")},
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
