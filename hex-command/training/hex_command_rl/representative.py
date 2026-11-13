"""Prepare isolated, event-free combat variants of the requested authored maps."""

import argparse, json, shutil
from pathlib import Path
from .workloads import FACTIONS

MAPS = ("open", "river-valley", "villages", "sluice")
AUTHORED = {"villages", "sluice"}


def prepare_catalog(source, destination):
    # Scenarios and workload definitions are already resolved through one engine catalog.
    shutil.copytree(Path(source), Path(destination), dirs_exist_ok=True)
    return dict(maps=MAPS)


def workload(start, seeds, all_pairs=False, rounds=30, opponent="classic"):
    configs = []
    for name in MAPS:
        multiplier = (12 if all_pairs else 4) if name in AUTHORED else 1
        for seed in range(start, start + seeds * multiplier):
            for recruiter in ("situational", "predictor"):
                pairs = (
                    [("human", "skeletons")]
                    if name == "villages"
                    else [("coastal", "human")]
                    if name == "sluice"
                    else [
                        (f, e)
                        for i, f in enumerate(FACTIONS)
                        for e in (FACTIONS if all_pairs else (FACTIONS[(i + 1) % 4],))
                        if e != f
                    ]
                )
                for player, enemy in pairs:
                    for first in ("player", "enemy"):
                        configs.append(
                            dict(
                                seed=seed,
                                armies=dict(player=player, enemy=enemy),
                                landscape=name,
                                player="balanced",
                                enemy=opponent,
                                playerBlocks=dict(
                                    recruiter=recruiter,
                                    campaign="mission",
                                    planner="queued",
                                ),
                                first=first,
                                maxRounds=rounds,
                                recruitLimit=None,
                                objective="deathmatch",
                            )
                        )
    return configs


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--out", type=Path, default=Path("tmp/rl/representative"))
    p.add_argument("--collect", action="store_true")
    p.add_argument("--catalog", type=Path, default=Path("rust/data"))
    p.add_argument("--train-seeds", type=int, default=16)
    p.add_argument("--eval-seeds", type=int, default=3)
    args = p.parse_args()
    if min(args.train_seeds, args.eval_seeds) < 1:
        p.error("seed counts must be positive")
    args.out.mkdir(parents=True, exist_ok=True)
    meta = prepare_catalog(args.catalog, args.out / "catalog")
    train = workload(1, args.train_seeds)
    valid = workload(10001, 1)
    evaluation = workload(20001, args.eval_seeds, all_pairs=True, rounds=100)
    partitions = [{c["seed"] for c in cases} for cases in (train, valid, evaluation)]
    if any(partitions[i] & partitions[j] for i in range(3) for j in range(i)):
        raise ValueError("seed partitions overlap")
    import os

    os.environ["HEX_COMMAND_DATA_ROOT"] = str((args.out / "catalog").resolve())
    import hex_command_training as native

    teacher = [
        json.loads(r)
        for r in native.evaluate([json.dumps(c) for c in evaluation], None)
    ]
    source = dict(
        **meta,
        collect_side="player",
        balance_maps=True,
        teacher="queued",
        opponent="classic",
        fingerprints=json.loads(native.fingerprints()),
        train_configs=train,
        validation_configs=valid,
        evaluation_configs=evaluation,
        teacher_results=teacher,
    )
    (args.out / "source.json").write_text(json.dumps(source, indent=2))
    if args.collect:
        from .spatial_data import collect_spatial_dataset

        for split, cases in [("train", train), ("validation", valid)]:
            collect_spatial_dataset(
                cases,
                args.out / "corpus" / split,
                progress=lambda row: print(json.dumps(row), flush=True),
            )
    print(
        json.dumps(
            dict(
                train_games=len(train),
                validation_games=len(valid),
                evaluation_games=len(evaluation),
                catalog=str((args.out / "catalog").resolve()),
            )
        ),
        flush=True,
    )


if __name__ == "__main__":
    main()
