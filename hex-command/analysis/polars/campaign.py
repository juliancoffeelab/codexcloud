"""Campaign observations and columnar attempt/stage reductions."""

import polars as pl
from frames import group
from journal import canonical, label

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


def frequencies(values):
    if not values:
        return {}
    return dict(pl.Series("key", values).value_counts(sort=True).iter_rows())


def diagnostics(chapter):
    phases = chapter.get("phases", [])
    own = [p for p in phases if p["side"] == "player"]
    final = chapter.get("final", {})
    units = final.get("units", [])
    winner = chapter["winner"]
    reason = (final.get("campaign") or {}).get("outcome")
    if winner is None and not reason:
        reason = "Round limit reached with unfinished objectives"
    goals = chapter.get("goals", [])
    actors = {
        u["name"]: u
        for u in chapter.get("initial", {}).get("units", [])
        if u.get("name")
        and u["side"] == "player"
        and any(u["name"] in g.get("label", "") for g in goals)
    }
    arrivals = {
        name: any(
            u.get("name") == name and u["tile"] in g.get("tiles", [])
            for p in phases
            for u in p.get("units", [])
            for g in goals
            if name in g.get("label", "")
        )
        for name in actors
    }
    losses = [u for p in phases for u in p.get("losses", []) if u["side"] == "player"]
    if winner == "player":
        category = "victory"
    elif winner is None:
        category = "round cap"
    elif "units" in final and not any(u["side"] == "player" for u in units):
        category = "all player units lost"
    elif any(u.get("role") == "leader" for u in losses):
        category = "leader lost"
    elif any(u.get("name") in actors or u.get("role") == "courier" for u in losses) or (
        reason and any(w in reason.lower() for w in ("courier", "companion"))
    ):
        category = "required actor lost"
    else:
        category = (
            "defeat; see recorded reason" if reason else "defeat; cause not recorded"
        )
    stalled = []
    if winner is None and own and own[-1].get("planning"):
        last = own[-1]
        roster = {u["id"]: u for u in last.get("units", [])}
        planning = last["planning"]
        duties = {d["id"]: d for d in planning["report"].get("duties", [])}
        for order in planning["orders"]["tactics"].get("objectives", []):
            body = order["objective"].get("GoTo") or order["objective"].get("Occupy")
            if body:
                actor = roster.get(f"u{body['actor']}", {})
                stalled.append(
                    dict(
                        actor=actor.get("name")
                        or actor.get("type")
                        or f"u{body['actor']}",
                        tile=actor.get("tile"),
                        hp=actor.get("hp"),
                        target=body.get("destination", body.get("location")),
                        reason=duties.get(order["id"], {}).get("reason"),
                        allies=[
                            {k: u.get(k) for k in ("id", "type", "tile", "rebuilding")}
                            for u in roster.values()
                            if u["side"] == "player"
                        ],
                    )
                )
    empty = pl.Series(
        "empty", [not p.get("actions") for p in reversed(own)], dtype=pl.Boolean
    )
    idle = empty.cast(pl.Int64).cum_prod().sum() if len(empty) else 0
    signatures = [
        canonical(
            [
                [[u["id"], u["type"], u["tile"], u["hp"]] for u in p.get("units", [])],
                p.get("ownership"),
                p.get("goals"),
            ]
        )
        for p in own[-10:]
    ]
    repeated = max(frequencies(signatures).values(), default=0)
    combat_rows, recruits, unmet, deferred = [], [], [], []
    rescue, retaliation = 0, 0
    for p in phases:
        roster = {
            u["id"]: u
            for u in p.get("before", []) + p.get("units", []) + p.get("losses", [])
        }
        if p["side"] == "player":
            recruits.extend(
                a["type"] for a in p.get("actions", []) if a["kind"] == "recruit"
            )
        combat_rows.extend(
            dict(
                own=roster.get(c["unit"], {}).get("side", p["side"]) == "player",
                damage=c["damage"],
                retaliation=c["retaliation"],
            )
            for c in p.get("combats", [])
        )
        if p["side"] == "player" and p.get("planning"):
            planning = p["planning"]
            duties = planning["report"].get("duties", [])
            unmet.extend(
                d.get("reason") or "unspecified"
                for d in duties
                if d["status"] == "Unmet"
            )
            deferred.extend(
                d.get("reason") or "unspecified"
                for d in duties
                if d["status"] == "Deferred"
            )
            rescue += planning["report"].get("candidates", 0)
            protected = {
                f"u{c['Survive']['actor']}"
                for c in planning["orders"]["tactics"].get("constraints", [])
                if "Survive" in c
            }
            retaliation += sum(
                c["unit"] in protected and c["retaliation"] > 0
                for c in p.get("combats", [])
            )
    if combat_rows:
        damage = (
            pl.from_dicts(combat_rows)
            .select(
                pl.when("own")
                .then("damage")
                .otherwise("retaliation")
                .sum()
                .alias("player"),
                pl.when("own")
                .then("retaliation")
                .otherwise("damage")
                .sum()
                .alias("enemy"),
            )
            .row(0, named=True)
        )
    else:
        damage = dict(player=0, enemy=0)
    progress = [
        dict(goal=i, current=g["current"])
        for p in phases
        for i, g in enumerate(p.get("goals", []))
    ]
    maxima = (
        dict(
            pl.from_dicts(progress)
            .group_by("goal")
            .agg(pl.col("current").max())
            .iter_rows()
        )
        if progress
        else {}
    )
    return dict(
        id=chapter["id"],
        round=chapter["round"],
        winner=winner,
        category=category,
        reason=reason,
        idleTail=idle,
        repeatedTail=repeated,
        finalPlayerUnits=sum(u["side"] == "player" for u in units),
        finalEnemyUnits=sum(u["side"] == "enemy" for u in units),
        maxGoalProgress=[
            maxima.get(i, g.get("current", 0)) for i, g in enumerate(goals)
        ],
        goals=goals,
        namedLosses=frequencies([u["name"] for u in losses if u.get("name")]),
        recruits=frequencies(recruits),
        casualties=frequencies([u["type"] for u in losses]),
        playerDamage=damage["player"],
        enemyDamage=damage["enemy"],
        namedGoalActors=list(actors),
        actorArrivals=arrivals,
        stalledActors=stalled,
        recordedPhases=bool(phases),
        unmetOrders=frequencies(unmet),
        deferredOrders=frequencies(deferred),
        rescueCandidates=rescue,
        protectedRetaliationExchanges=retaliation,
        recordedPlanning=any(p.get("planning") for p in own),
    )


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
    for r in campaigns:
        r["completionRate"] = 100 * r["completed"] / r["attempts"]
    stage_rows = group(stages, STAGES, COUNTS)
    if stage_rows:
        joined = (
            pl.from_dicts(stage_rows)
            .lazy()
            .join(
                pl.from_dicts(campaigns)
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
        stage_rows = joined.to_dicts()
    failure_rows = []
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
            .to_dicts()
        )
    return dict(
        campaign_summary=campaigns,
        campaign_stages=stage_rows,
        campaign_failures=failure_rows,
        campaign_unit_events=group(events, STAGES + ["event", "unit"], ["count"]),
        campaign_runs=runs,
    )
