"""Campaign diagnostics over the shared decoded relations.

Only terminal explanations and stalled-actor presentation are assembled as
objects. Phase scans, damage, progress, arrivals and planning counts use Polars.
"""

import polars as pl
from dataset import concat
from journal import identity

K = ["m", "ch"]


def observations(d):
    if not d.headers:
        return {}
    c = pl.col
    campaign_ids = d.meta.filter(c("kind") == "campaign").select("m")
    if campaign_ids.is_empty():
        return {}
    campaign_matches = set(campaign_ids["m"].to_list())
    phases = d.phases.join(campaign_ids, on="m", how="semi")
    before = d.rosters["before"].join(campaign_ids, on="m", how="semi")
    casualties = d.losses.join(campaign_ids, on="m", how="semi")
    own = phases.filter(c("side") == "player")
    end = d.rosters["units"].join(campaign_ids, on="m", how="semi")
    losses = casualties.filter(c("side") == "player")

    def index(df, columns=K):
        return {tuple(row[k] for k in columns): row for row in df.iter_rows(named=True)}

    final = index(
        d.final.join(campaign_ids, on="m", how="semi")
        .group_by(K)
        .agg(
            (c("side") == "player").sum().alias("player"),
            (c("side") == "enemy").sum().alias("enemy"),
        )
    )
    counts = index(phases.group_by(K).len(name="n"))
    last = index(own.sort("p").group_by(K, maintain_order=True).last())
    idle = index(
        own.sort("p", descending=True)
        .with_columns(
            c("actions")
            .list.len()
            .fill_null(0)
            .eq(0)
            .cast(pl.Int64)
            .cum_prod()
            .over(K)
            .alias("emptyTail")
        )
        .group_by(K)
        .agg(c("emptyTail").sum())
    )
    # Equality of ordered typed lists preserves roster/goal order. Ownership
    # values are compared, rather than assuming dictionary indices are unique.
    owner_parts = []
    for m, values in d.owners.items():
        if m not in campaign_matches:
            continue
        owner_parts.append(
            pl.from_arrow(values)
            .rename("owners")
            .to_frame()
            .with_row_index("ownership")
            .with_columns(pl.lit(m).alias("m"))
            .select(
                "m",
                "ownership",
                pl.struct("owners").struct.json_encode().alias("owners"),
            )
        )
    owners = concat(
        owner_parts, schema={"m": pl.Int64, "ownership": pl.Int64, "owners": pl.String}
    )
    rosters = end.group_by("p", maintain_order=True).agg(
        pl.struct("id", "type", "tile", "hp").alias("roster")
    )
    tail = (
        own.sort("p", descending=True)
        .group_by(K, maintain_order=True)
        .head(10)
        .join(rosters, on="p", how="left")
        .join(d.refs.select("p", "ownership"), on="p")
        .join(owners, on=["m", "ownership"], how="left")
    )
    signatures = tail.select(
        *K,
        pl.struct("roster", "owners", "goals").struct.json_encode().alias("signature"),
    )
    repeated = index(
        signatures.group_by(K + ["signature"])
        .len()
        .group_by(K)
        .agg(c("len").max().alias("repeated"))
    )
    diag_lookup = concat([before, end, casualties]).unique(["p", "id"], keep="last")
    combat = d.combats.join(
        diag_lookup.select("p", c("id").alias("unit"), c("side").alias("actorSide")),
        on=["p", "unit"],
        how="left",
    ).join(phases.select("p", *K, "side"), on="p")
    own_combat = pl.coalesce("actorSide", "side") == "player"
    damage = index(
        combat.group_by(K).agg(
            pl.when(own_combat)
            .then("damage")
            .otherwise("retaliation")
            .sum()
            .alias("player"),
            pl.when(own_combat)
            .then("retaliation")
            .otherwise("damage")
            .sum()
            .alias("enemy"),
        )
    )
    goals = (
        phases.select(*K, "p", "goals")
        .with_columns(pl.int_ranges(0, c("goals").list.len()).alias("goal"))
        .explode("goals", "goal")
        .filter(c("goals").is_not_null())
        .with_columns(c("goals").struct.field("current").alias("current"))
    )
    maxima = {}
    for m, ch, i, n in goals.group_by(K + ["goal"]).agg(c("current").max()).iter_rows():
        maxima.setdefault((m, ch), {})[i] = n

    def frequencies(q, field):
        result = {}
        for m, ch, key, n in (
            q.filter(c(field).is_not_null())
            .group_by(K + [field])
            .len()
            .sort(K + [field])
            .iter_rows()
        ):
            result.setdefault((m, ch), {})[key] = n
        return result

    named = frequencies(losses, "name")
    casualties = frequencies(losses, "type")
    recruited = frequencies(
        d.actions.filter(c("kind") == "recruit").join(
            own.select("p", *K), on="p", how="inner"
        ),
        "type",
    )
    planning = own.filter(c("planning").is_not_null())
    duties = (
        planning.select(
            *K,
            c("planning").struct.field("report").struct.field("duties").alias("duty"),
        )
        .explode("duty")
        .filter(c("duty").is_not_null())
        .unnest("duty")
        .with_columns(c("reason").fill_null("unspecified"))
    )
    unmet = frequencies(duties.filter(c("status") == "Unmet"), "reason")
    deferred = frequencies(duties.filter(c("status") == "Deferred"), "reason")
    rescue = index(
        planning.group_by(K).agg(
            c("planning")
            .struct.field("report")
            .struct.field("candidates")
            .fill_null(0)
            .sum()
            .alias("candidates"),
            pl.len().alias("recorded"),
        )
    )
    protected = (
        planning.select(
            "p",
            c("planning")
            .struct.field("orders")
            .struct.field("tactics")
            .struct.field("constraints")
            .alias("constraint"),
        )
        .explode("constraint")
        .filter(c("constraint").is_not_null())
        .select(
            "p",
            pl.concat_str(
                pl.lit("u"),
                c("constraint").struct.field("Survive").struct.field("actor"),
            ).alias("unit"),
        )
        .drop_nulls()
        .unique()
    )
    protected_counts = index(
        combat.filter((c("side") == "player") & (c("retaliation") > 0))
        .join(protected, on=["p", "unit"], how="inner")
        .group_by(K)
        .len(name="n")
    )
    actor_rows = []
    for m, (_, result, _) in enumerate(d.headers):
        if result["kind"] != "campaign":
            continue
        for ch, chapter in enumerate(result.get("missions", [])):
            for u in chapter.get("initial", {}).get("units", []):
                if u.get("name") and u["side"] == "player":
                    for goal in chapter.get("goals", []):
                        if u["name"] in goal.get("label", ""):
                            for t in goal.get("tiles", []):
                                actor_rows.append(
                                    dict(
                                        m=m,
                                        ch=ch,
                                        name=u["name"],
                                        cell=f"{t['col']},{t['row']}",
                                    )
                                )
    actors = pl.from_dicts(
        actor_rows,
        schema={"m": pl.Int64, "ch": pl.Int64, "name": pl.String, "cell": pl.String},
    )
    arrived = set(
        end.select(*K, "name", "cell")
        .join(actors, on=K + ["name", "cell"], how="inner")
        .select(*K, "name")
        .unique()
        .iter_rows()
    )
    # Presentation projections are restricted to the final own-phase snapshots.
    last_ids = pl.DataFrame(
        {"p": [row["p"] for row in last.values()]}, schema={"p": pl.Int64}
    )
    final_rosters = {
        k[0]: v.to_dicts()
        for k, v in end.join(last_ids, on="p", how="inner")
        .partition_by("p", as_dict=True)
        .items()
    }
    loss_flags = index(
        losses.group_by(K).agg(
            (c("role") == "leader").any().alias("leader"),
            (c("role") == "courier").any().alias("courier"),
        )
    )
    loss_names = {k: set(v) for k, v in named.items()}
    output = {}
    for m, (saved, result, _) in enumerate(d.headers):
        if result["kind"] != "campaign":
            continue
        observations_ = []
        for ch, chapter in enumerate(result.get("missions", [])):
            key = m, ch
            winner = chapter["winner"]
            state = chapter.get("final", {})
            units = final.get(key, {})
            reason = (state.get("campaign") or {}).get("outcome")
            if winner is None and not reason:
                reason = "Round limit reached with unfinished objectives"
            goals_ = chapter.get("goals", [])
            named_actors = [
                u["name"]
                for u in chapter.get("initial", {}).get("units", [])
                if u.get("name")
                and u["side"] == "player"
                and any(u["name"] in g.get("label", "") for g in goals_)
            ]
            arrivals = {name: (m, ch, name) in arrived for name in named_actors}
            flags = loss_flags.get(key, {})
            if winner == "player":
                category = "victory"
            elif winner is None:
                category = "round cap"
            elif "units" in state and not units.get("player", 0):
                category = "all player units lost"
            elif flags.get("leader"):
                category = "leader lost"
            elif (
                any(name in loss_names.get(key, set()) for name in named_actors)
                or flags.get("courier")
                or (
                    reason
                    and any(w in reason.lower() for w in ("courier", "companion"))
                )
            ):
                category = "required actor lost"
            else:
                category = (
                    "defeat; see recorded reason"
                    if reason
                    else "defeat; cause not recorded"
                )
            stalled = []
            last_ = last.get(key, {})
            if winner is None and last_.get("planning"):
                roster = final_rosters.get(last_["p"], [])
                by_id = {u["id"]: u for u in roster}
                plan = last_["planning"]
                duties_ = {v["id"]: v for v in plan["report"].get("duties", []) or []}
                for order in plan["orders"]["tactics"].get("objectives", []) or []:
                    body = order["objective"].get("GoTo") or order["objective"].get(
                        "Occupy"
                    )
                    if body:
                        actor = by_id.get(f"u{body['actor']}", {})
                        stalled.append(
                            dict(
                                actor=actor.get("name")
                                or actor.get("type")
                                or f"u{body['actor']}",
                                tile=actor.get("tile"),
                                hp=actor.get("hp"),
                                target=body.get("destination") or body.get("location"),
                                reason=duties_.get(order["id"], {}).get("reason"),
                                allies=[
                                    dict(
                                        id=u["id"],
                                        type=u["type"],
                                        tile=u["tile"],
                                        rebuilding=u.get("rebuilding"),
                                    )
                                    for u in roster
                                    if u["side"] == "player"
                                ],
                            )
                        )
            observations_.append(
                dict(
                    id=chapter["id"],
                    round=chapter["round"],
                    winner=winner,
                    category=category,
                    reason=reason,
                    idleTail=idle.get(key, {}).get("emptyTail", 0),
                    repeatedTail=repeated.get(key, {}).get("repeated", 0),
                    finalPlayerUnits=units.get("player", 0),
                    finalEnemyUnits=units.get("enemy", 0),
                    maxGoalProgress=[
                        maxima.get(key, {}).get(i, g.get("current", 0))
                        for i, g in enumerate(goals_)
                    ],
                    goals=goals_,
                    namedLosses=named.get(key, {}),
                    recruits=recruited.get(key, {}),
                    casualties=casualties.get(key, {}),
                    playerDamage=damage.get(key, {}).get("player", 0),
                    enemyDamage=damage.get(key, {}).get("enemy", 0),
                    namedGoalActors=named_actors,
                    actorArrivals=arrivals,
                    stalledActors=stalled,
                    recordedPhases=bool(counts.get(key, {}).get("n", 0)),
                    unmetOrders=unmet.get(key, {}),
                    deferredOrders=deferred.get(key, {}),
                    rescueCandidates=rescue.get(key, {}).get("candidates", 0),
                    protectedRetaliationExchanges=protected_counts.get(key, {}).get(
                        "n", 0
                    ),
                    recordedPlanning=bool(rescue.get(key, {}).get("recorded", 0)),
                )
            )
        output[identity(saved)] = observations_
    return output
