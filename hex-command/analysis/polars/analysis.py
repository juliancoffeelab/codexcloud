"""Complete report computation, including baseline and optional telemetry."""

import polars as pl
import arena
import campaign
import telemetry
import metrics
from journal import identity
from frames import empty


def changes(rows, before, keys, fields):
    if rows.is_empty() or before.is_empty():
        return empty(keys, fields + [k + "Before" for k in fields])
    return (
        rows.lazy()
        .join(
            before.lazy().select(
                *keys, *[pl.col(k).alias(k + "Before") for k in fields]
            ),
            on=keys,
            how="inner",
            nulls_equal=True,
            validate="1:1",
        )
        .collect()
    )


def match_changes(current, previous):
    if not previous:
        return []
    old = {identity(o): o for o in previous}
    if {identity(o) for o in current} != old.keys():
        raise ValueError("Unmatched configurations")
    rows = []
    for o in current:
        key = identity(o)
        a = old[key]
        if o["kind"] == "campaign":

            def result(x):
                return x["completed"], [(m["id"], m["winner"]) for m in x["missions"]]

            def gameplay(x):
                return x["completed"], [
                    (m["id"], m["round"], m["stateHash"]) for m in x["missions"]
                ]

            rows.append(
                dict(
                    matchId=key,
                    kind="campaign",
                    beforeCompleted=a["completed"],
                    completed=o["completed"],
                    beforeMissions=a["missions"],
                    missions=o["missions"],
                    resultChanged=result(a) != result(o),
                    gameplayChanged=gameplay(a) != gameplay(o),
                )
            )
        else:
            rows.append(
                dict(
                    matchId=key,
                    kind="arena",
                    beforeWinner=a.get("winner"),
                    winner=o.get("winner"),
                    beforeReason=a.get("reason"),
                    reason=o.get("reason"),
                    beforeRounds=a.get("rounds"),
                    rounds=o.get("rounds"),
                    resultChanged=(a.get("winner"), a.get("reason"))
                    != (o.get("winner"), o.get("reason")),
                    gameplayChanged=a.get("stateHash") != o.get("stateHash"),
                )
            )
    return rows


def build(
    current,
    previous,
    *,
    unit_usage=False,
    presence=False,
    support=False,
    value=False,
    leaders=False,
):
    rows, units, ledger, campaigns = arena.tally(current["outcomes"])
    before, before_units, _, _ = arena.tally(previous["outcomes"])
    transitions = match_changes(current["outcomes"], previous["outcomes"])
    families = {}
    for source in current["sources"]:
        for unit, family in (source.get("unitFamilies") or {}).items():
            if unit in families and families[unit] != family:
                raise ValueError("Conflicting unit family: " + unit)
            families[unit] = family
    purchases = arena.purchase_shares(units, families)
    tables = campaign.tables(current["outcomes"])
    old_campaigns = campaign.tables(previous["outcomes"])
    matchup_changes = changes(
        rows, before, arena.KEYS, ["wins", "losses", "caps", "winRate"]
    )
    matchup_changes = matchup_changes.with_columns(
        (pl.col("winRate") - pl.col("winRateBefore")).alias("winRateDelta")
    )
    campaign_changes = changes(
        tables["campaign_summary"],
        old_campaigns["campaign_summary"],
        campaign.FIELDS,
        ["completed", "completionRate"],
    )
    campaign_changes = campaign_changes.with_columns(
        (pl.col("completionRate") - pl.col("completionRateBefore")).alias(
            "completionRateDelta"
        )
    )

    def race_map(opponent):
        return arena.combine(
            rows.filter(
                (pl.col("opponentPolicy") == opponent)
                & (
                    (pl.col("policy") != opponent)
                    | (pl.col("race") != pl.col("opponentRace"))
                )
            ),
            ["race", "map", "policy"],
        )

    timings = []
    for label, sources in [
        ("current", current["sources"]),
        ("baseline", previous["sources"]),
    ]:
        for s in sources:
            timing = s.get("turnTiming") or {}
            timings.extend(
                dict(r, run=label, clock=timing.get("clock"), source=s["file"])
                for r in timing.get("policies", [])
            )
    tables.update(
        matchups=rows,
        unit_events=units,
        matches=ledger,
        campaigns=campaigns,
        policy_matchups=arena.combine(
            rows.filter(pl.col("policy") != pl.col("opponentPolicy")),
            ["policy", "opponentPolicy"],
            cross=False,
        ),
        race_map_vs_classic=race_map("classic"),
        race_map_vs_balanced=race_map("balanced"),
        purchase_shares=purchases,
        race_equal_policy=arena.combine(rows, ["race"], equal=True),
        race_map_equal_policy=arena.combine(rows, ["race", "map"], equal=True),
        race_map_policy=arena.combine(rows, ["race", "map", "policy"], equal=True),
        race_all_policies=arena.combine(rows, ["race"]),
        baseline_matchups=before,
        matchup_changes=matchup_changes,
        match_changes=transitions,
        baseline_unit_events=before_units,
        campaign_changes=campaign_changes,
        policy_timings=timings,
    )
    if not any([unit_usage, presence, support, value, leaders]):
        return tables
    if unit_usage and any(
        s.get("combatTelemetryVersion") != 1
        for s in current["sources"] + previous["sources"]
    ):
        raise ValueError("Damage tables require fresh combat telemetry")
    value = value or (
        unit_usage
        and all(s.get("combatTelemetryVersion") == 1 for s in current["sources"])
    )
    leaders = leaders or (
        unit_usage and all(s.get("leaderAuditVersion") == 1 for s in current["sources"])
    )
    tables.update(
        telemetry.analyze(
            current["dataset"],
            presence_=unit_usage or presence,
            support_=unit_usage or support,
            value_=value,
            leaders_=leaders,
        )
    )
    if unit_usage or presence:
        nonleaders = tables["nonleader_unit_events"]
        tables["army_usage"] = metrics.army_usage(
            nonleaders, purchases, tables["battlefield_presence"], families
        )
    if unit_usage:
        profiles = {
            u: dict(p) for u, p in current["sources"][0]["unitProfiles"].items()
        }
        old = {}
        if previous["outcomes"]:
            old.update(
                telemetry.analyze(previous["dataset"], presence_=True, value_=True)
            )
        membership = pl.concat(
            [
                tables["battlefield_units"],
                old.get("battlefield_units", tables["battlefield_units"].head(0)),
            ]
        )
        tables["unit_damage"] = metrics.unit_damage(
            tables["battlefield_units"],
            tables["battlefield_unit_value"],
            profiles,
            membership,
        )
        if previous["outcomes"]:
            tables["baseline_unit_damage"] = metrics.unit_damage(
                old["battlefield_units"],
                old["battlefield_unit_value"],
                profiles,
                membership,
            )
            tables["unit_damage_changes"] = metrics.compare_damage(
                tables["unit_damage"], tables["baseline_unit_damage"]
            )
    return tables
