"""Campaign observations and columnar attempt/stage reductions."""

import polars as pl
from frames import aggregate as group
from journal import label

FIELDS = [
    "campaign",
    "player",
    "enemy",
    "diplomacy",
    "objective",
    "maxRounds",
    "recruitLimit",
]
STAGES = FIELDS + ["stage", "map"]
COUNTS = [
    "reached",
    "victories",
    "defeats",
    "caps",
    "rounds",
    "detailed",
    "idleCaps",
    "repeatedCaps",
    "zeroProgressDefeats",
    "actorAttempts",
    "actorArrivals",
    "actorDefeatsBeforeArrival",
    "defeatsWithSurvivors",
    "survivorsInDefeats",
    "playerDamage",
    "enemyDamage",
]


def tables(outcomes):
    attempts, stages, failures, events, runs = [], [], [], [], []
    for o in outcomes:
        if o["kind"] != "campaign":
            continue
        c = o["config"]
        d = {
            k: label(c, k)
            if k in ("player", "enemy")
            else c.get(k, "authored" if k == "objective" else "unrecorded")
            for k in FIELDS
        }
        missions = o["analysis"]
        last = missions[-1]["winner"] if missions else "unclassified"
        attempts.append(
            dict(
                d,
                attempts=1,
                completed=int(bool(o["completed"])),
                defeats=int(not o["completed"] and bool(missions) and last == "enemy"),
                caps=int(not o["completed"] and bool(missions) and last is None),
                unclassified=int(
                    not o["completed"] and (not missions or last not in ("enemy", None))
                ),
            )
        )
        for i, m in enumerate(missions):
            s = dict(d, stage=i + 1, map=m["id"])
            win, actor, arrived = (
                m["winner"],
                bool(m["namedGoalActors"]),
                all(m["actorArrivals"].values()),
            )
            stages.append(
                dict(
                    s,
                    reached=1,
                    victories=int(win == "player"),
                    defeats=int(win == "enemy"),
                    caps=int(win is None),
                    rounds=m["round"],
                    detailed=int(m["recordedPhases"]),
                    idleCaps=int(win is None and m["idleTail"] >= 5),
                    repeatedCaps=int(win is None and m["repeatedTail"] >= 5),
                    zeroProgressDefeats=int(
                        win == "enemy"
                        and bool(m["maxGoalProgress"])
                        and not any(m["maxGoalProgress"])
                    ),
                    actorAttempts=int(actor),
                    actorArrivals=int(actor and arrived),
                    actorDefeatsBeforeArrival=int(
                        win == "enemy" and actor and not arrived
                    ),
                    defeatsWithSurvivors=int(
                        win == "enemy" and m["finalPlayerUnits"] > 0
                    ),
                    survivorsInDefeats=m["finalPlayerUnits"] if win == "enemy" else 0,
                    playerDamage=m["playerDamage"],
                    enemyDamage=m["enemyDamage"],
                )
            )
            run_fields = (
                "winner",
                "category",
                "reason",
                "finalPlayerUnits",
                "idleTail",
                "repeatedTail",
                "maxGoalProgress",
                "goals",
                "namedLosses",
                "actorArrivals",
                "unmetOrders",
                "deferredOrders",
                "rescueCandidates",
                "protectedRetaliationExchanges",
                "recordedPlanning",
                "stalledActors",
            )
            runs.append(
                dict(
                    s,
                    seed=c.get("seed"),
                    round=m["round"],
                    detail=o.get("detail"),
                    **{k: m[k] for k in run_fields},
                )
            )
            if win != "player":
                failures.append(
                    dict(
                        s,
                        category=m["category"],
                        reason=m["reason"] or "No terminal explanation recorded",
                        seed=c.get("seed"),
                        attempts=1,
                    )
                )
            for event, field in [
                ("recruited", "recruits"),
                ("lost", "casualties"),
                ("namedLost", "namedLosses"),
            ]:
                events.extend(
                    dict(s, event=event, unit=u, count=n) for u, n in m[field].items()
                )
    campaigns = group(
        attempts, FIELDS, ["attempts", "completed", "defeats", "caps", "unclassified"]
    )
    campaigns = campaigns.with_columns((100 * pl.col("completed") / pl.col("attempts")).alias("completionRate"))
    stage_rows = group(stages, STAGES, COUNTS)
    if not stage_rows.is_empty():
        joined = (
            stage_rows
            .lazy()
            .join(
                campaigns
                .lazy()
                .select(*FIELDS, pl.col("attempts").alias("campaignAttempts")),
                on=FIELDS,
                how="left",
                nulls_equal=True,
                validate="m:1",
            )
            .with_columns(
                (100 * pl.col("victories") / pl.col("reached")).alias("victoryRate"),
                (pl.col("rounds") / pl.col("reached")).alias("meanRounds"),
            )
            .sort(STAGES, nulls_last=True)
            .collect()
        )
        stage_rows = joined
    failure_rows = pl.DataFrame()
    if failures:
        failure_rows = (
            pl.from_dicts(failures)
            .lazy()
            .group_by(STAGES + ["category", "reason"])
            .agg(
                pl.col("attempts").sum(),
                pl.col("seed")
                .drop_nulls()
                .unique()
                .sort()
                .head(3)
                .alias("exampleSeeds"),
            )
            .sort(STAGES + ["category", "reason"], nulls_last=True)
            .collect()
        )
    return dict(
        campaign_summary=campaigns,
        campaign_stages=stage_rows,
        campaign_failures=failure_rows,
        campaign_unit_events=group(events, STAGES + ["event", "unit"], ["count"]),
        campaign_runs=runs,
    )
