"""Exposure-weighted metrics expressed as joins, grouped sums and expressions."""

import polars as pl

DIMS = ["race", "policy", "opponentPolicy", "map"]
EVENTS = ["recruited", "promoted", "lost", "attacks", "retaliations", "damage"]
ROLE = {
    "warrior": "infantry",
    "flank": "flank",
    "ranger": "ranger",
    "healer": "support",
}


def frame(rows, dimensions, numeric):
    if isinstance(rows, pl.DataFrame):
        return rows.lazy()
    if rows:
        return pl.from_dicts(rows, infer_schema_length=None).lazy()
    return pl.DataFrame(
        schema={
            **{k: pl.String for k in dimensions},
            **{k: pl.Float64 for k in numeric},
        }
    ).lazy()


def army_usage(events, purchases, presence, families):
    if presence.is_empty():
        return pl.DataFrame()
    q = frame(events, DIMS + ["unit", "opponentRace"], EVENTS)
    if "opponentRace" in events.columns:
        q = q.filter(pl.col("race") != pl.col("opponentRace"))
    q = q.with_columns(pl.col("unit").replace(families).alias("family"))
    q = pl.concat([q, q.with_columns(pl.lit("all").alias("map"))])
    counts = (
        q.group_by(DIMS + ["family"])
        .agg(pl.col(EVENTS).sum())
        .with_columns(
            pl.col("attacks").sum().over(DIMS).alias("__attacks"),
            pl.col("damage").sum().over(DIMS).alias("__damage"),
        )
    )
    buys = frame(purchases, DIMS + ["family"], ["sharePercent"]).select(
        *DIMS, "family", pl.col("sharePercent").alias("purchaseSharePercent")
    )
    q = (
        presence.lazy()
        .join(counts, on=DIMS + ["family"], how="left", validate="1:1")
        .join(buys, on=DIMS + ["family"], how="left", validate="1:1")
        .with_columns(
            pl.col(
                EVENTS + ["__attacks", "__damage", "purchaseSharePercent"]
            ).fill_null(0)
        )
        .with_columns(
            (100 * pl.col("attacks") / pl.col("__attacks").clip(lower_bound=1)).alias(
                "attackSharePercent"
            ),
            (100 * pl.col("damage") / pl.col("__damage").clip(lower_bound=1)).alias(
                "damageSharePercent"
            ),
            (pl.col("attacks") / pl.col("unitTurns").clip(lower_bound=1)).alias(
                "attacksPerUnitTurn"
            ),
            (pl.col("retaliations") / pl.col("unitTurns").clip(lower_bound=1)).alias(
                "retaliationsPerUnitTurn"
            ),
        )
        .drop("__attacks", "__damage")
        .sort(DIMS + ["family"])
    )
    return q.collect()


def unit_damage(presence, values, profiles, membership=()):
    if presence.is_empty() and values.is_empty():
        return pl.DataFrame()
    modern = "nonleaderUnitTurns" in values.columns
    if modern:
        values = values.with_columns(
            pl.col(source).alias(target)
            for source, target in zip(
                [
                    "nonleaderUnitTurns",
                    "nonleaderWeaponHPDamageDealt",
                    "nonleaderThornHPDamageDealt",
                ],
                ["unitTurns", "weaponHPDamageDealt", "thornHPDamageDealt"],
            )
        )
    missing = pl.concat([presence.select("unit"), values.select("unit")]).filter(
        ~pl.col("unit").is_in(list(profiles))
    )
    if missing.height:
        raise ValueError("Missing recorded unit profile: " + missing["unit"][0])
    profile_rows = [
        dict(
            unit=u,
            name=p.get("name", u),
            family=p["family"],
            level=p.get("level"),
            unitKind=ROLE.get(p.get("role"), "unknown"),
            leader=p.get("leader", False) if not modern else False,
        )
        for u, p in profiles.items()
    ]
    prof = pl.from_dicts(profile_rows, infer_schema_length=None).lazy()
    p = frame(presence, DIMS + ["unit"], ["unitTurns"]).select(
        *DIMS, "unit", "unitTurns"
    )
    v = frame(
        values,
        DIMS + ["unit", "kind"],
        ["unitTurns", "weaponHPDamageDealt", "thornHPDamageDealt"],
    ).filter(pl.col("kind") == "arena")
    known = pl.concat(
        [
            frame(rows, DIMS + ["unit"], ["unitTurns"]).filter(
                pl.col("kind") == "arena"
            )
            if "kind" in rows.columns
            else frame(rows, DIMS + ["unit"], ["unitTurns"])
            for rows in [presence, values, membership]
        ],
        how="diagonal_relaxed",
    )
    if modern:
        known = known.filter(
            pl.col("nonleaderUnitTurns").is_null()
            | pl.any_horizontal(
                pl.col(k).fill_null(0) != 0
                for k in [
                    "nonleaderUnitTurns",
                    "nonleaderWeaponHPDamageDealt",
                    "nonleaderThornHPDamageDealt",
                ]
            )
        )
    membership_df = (
        known.select("race", "unit")
        .unique()
        .join(prof.select("unit", "leader"), on="unit", how="inner")
        .filter(~pl.col("leader"))
        .drop("leader")
    )
    group_scopes = pl.concat([p.select(DIMS), v.select(DIMS)]).unique()
    p = (
        p.join(prof.select("unit", "leader"), on="unit", how="left")
        .filter(~pl.col("leader"))
        .drop("leader")
    )
    v = (
        v.join(prof.select("unit", "leader"), on="unit", how="left")
        .filter(~pl.col("leader"))
        .drop("leader")
    )
    checks = (
        p.join(
            v.select(*DIMS, "unit", pl.col("unitTurns").alias("__valueTurns")),
            on=DIMS + ["unit"],
            how="full",
            coalesce=True,
        )
        .filter(pl.col("unitTurns").fill_null(0) != pl.col("__valueTurns").fill_null(0))
        .collect()
    )
    if checks.height:
        raise ValueError("Damage/presence census mismatch: " + checks["unit"][0])
    totals_p = p.group_by(DIMS).agg(pl.col("unitTurns").sum().alias("totalUnitTurns"))
    totals_v = v.group_by(DIMS).agg(
        pl.col("weaponHPDamageDealt").sum().alias("totalDamage")
    )
    q = (
        group_scopes.join(membership_df, on="race", how="inner")
        .join(prof.drop("leader"), on="unit", how="left")
        .join(p, on=DIMS + ["unit"], how="left", validate="1:1")
        .join(
            v.select(
                *DIMS,
                "unit",
                pl.col("weaponHPDamageDealt").alias("weaponDamage"),
                pl.col("thornHPDamageDealt").alias("thornDamage"),
            ),
            on=DIMS + ["unit"],
            how="left",
            validate="1:1",
        )
        .join(totals_p, on=DIMS, how="left", validate="m:1")
        .join(totals_v, on=DIMS, how="left", validate="m:1")
        .with_columns(
            pl.col(
                "unitTurns",
                "totalUnitTurns",
                "weaponDamage",
                "thornDamage",
                "totalDamage",
            ).fill_null(0)
        )
        .with_columns(
            pl.col("weaponDamage").alias("damage"),
            pl.when(pl.col("totalUnitTurns") > 0)
            .then(100 * pl.col("unitTurns") / pl.col("totalUnitTurns"))
            .otherwise(0)
            .alias("presenceSharePercent"),
            pl.when(pl.col("totalDamage") > 0)
            .then(100 * pl.col("weaponDamage") / pl.col("totalDamage"))
            .otherwise(None)
            .alias("damageSharePercent"),
            pl.when(pl.col("unitTurns") > 0)
            .then(pl.col("weaponDamage") / pl.col("unitTurns"))
            .otherwise(None)
            .alias("damagePerUnitTurn"),
            pl.when(pl.col("unitTurns") > 0)
            .then(pl.col("thornDamage") / pl.col("unitTurns"))
            .otherwise(None)
            .alias("thornDamagePerUnitTurn"),
            pl.when(pl.col("totalUnitTurns") > 0)
            .then(pl.col("totalDamage") / pl.col("totalUnitTurns"))
            .otherwise(None)
            .alias("factionDamagePerUnitTurn"),
        )
        .with_columns(
            pl.when(pl.col("presenceSharePercent") > 0)
            .then(pl.col("damageSharePercent") / pl.col("presenceSharePercent"))
            .otherwise(None)
            .alias("damagePresenceRatio"),
            pl.col("unitKind")
            .replace_strict(
                {"infantry": 0, "flank": 1, "ranger": 2, "support": 3, "unknown": 4}
            )
            .alias("__kind"),
        )
        .sort(DIMS + ["__kind", "family", "level", "name"])
        .drop("__kind")
    )
    return q.collect()


def compare_damage(current, baseline):
    if current.is_empty():
        return pl.DataFrame()
    fields = [
        "unitTurns",
        "damage",
        "presenceSharePercent",
        "damageSharePercent",
        "damagePerUnitTurn",
        "damagePresenceRatio",
        "thornDamage",
        "thornDamagePerUnitTurn",
    ]
    q = current.lazy()
    if not baseline.is_empty():
        old = baseline.lazy().select(
            *DIMS, "unit", *[pl.col(k).alias(k + "Before") for k in fields]
        )
        q = q.join(old, on=DIMS + ["unit"], how="left", validate="1:1")
    else:
        q = q.with_columns(pl.lit(None).alias(k + "Before") for k in fields)
    return q.with_columns(
        (pl.col(k) - pl.col(k + "Before")).alias(k + "Delta") for k in fields
    ).collect()
