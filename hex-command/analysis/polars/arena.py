import polars as pl
from frames import empty

KEYS = ["race", "opponentRace", "policy", "opponentPolicy", "map"]
COUNTS = ["games", "wins", "losses", "caps", "leaderWins", "territoryWins", "rounds"]
EVENTS = ["recruited", "promoted", "lost", "attacks", "retaliations", "damage"]


def calculate(df):
    arena = df.lazy().filter(pl.col("kind") == "arena")
    c = pl.col("config")
    armies = c.struct.field("armies")

    def dimensions(side, other):
        return [
            armies.struct.field(side).alias("race"),
            armies.struct.field(other).alias("opponentRace"),
            c.struct.field(side).alias("policy"),
            c.struct.field(other).alias("opponentPolicy"),
            c.struct.field("landscape").alias("map"),
        ]

    def flag(x):
        return x.fill_null(False).cast(pl.Int64)

    sides = [("player", "enemy"), ("enemy", "player")]
    matches = []
    events = []
    for side, other in sides:
        wins = pl.col("winner") == side
        matches.append(
            arena.select(
                *dimensions(side, other),
                pl.lit(1, dtype=pl.Int64).alias("games"),
                flag(wins).alias("wins"),
                flag(pl.col("winner") == other).alias("losses"),
                flag(pl.col("winner").is_null()).alias("caps"),
                flag(wins & (pl.col("reason") == "leader")).alias("leaderWins"),
                flag(wins & (pl.col("reason") == "territory")).alias("territoryWins"),
                pl.col("rounds"),
            )
        )
        for source, label in [
            ("recruits", "recruited"),
            ("promotions", "promoted"),
            ("casualties", "lost"),
            ("combatUsage", ""),
        ]:
            e = pl.col("event")
            q = (
                arena.select(*dimensions(side, other), pl.col(source).alias("event"))
                .explode("event")
                .filter(e.struct.field("side") == side)
            )
            stats = [
                (
                    e.struct.field(n)
                    if source == "combatUsage"
                    and n in ["attacks", "retaliations", "damage"]
                    else pl.lit(int(n == label), dtype=pl.Int64)
                ).alias(n)
                for n in EVENTS
            ]
            events.append(q.select(*KEYS, e.struct.field("type").alias("unit"), *stats))
    matchup = (
        pl.concat(matches)
        .group_by(KEYS)
        .agg(pl.col(COUNTS).sum())
        .with_columns(
            (100 * pl.col("wins") / pl.col("games")).alias("winRate"),
            (100 * pl.col("losses") / pl.col("games")).alias("lossRate"),
            (100 * pl.col("caps") / pl.col("games")).alias("capRate"),
            (pl.col("rounds") / pl.col("games")).alias("meanRounds"),
        )
        .sort(KEYS)
        .collect()
    )
    units = (
        pl.concat(events)
        .group_by(KEYS + ["unit"])
        .agg(pl.col(EVENTS).sum())
        .sort(KEYS + ["unit"])
        .collect()
    )
    return matchup, units


from journal import label, identity


def tally(outcomes):
    ledger, campaigns, records = [], [], []
    event_type = pl.List(pl.Struct({"side": pl.String, "type": pl.String}))
    usage_type = pl.List(
        pl.Struct(
            {
                "side": pl.String,
                "type": pl.String,
                "attacks": pl.Int64,
                "retaliations": pl.Int64,
                "damage": pl.Int64,
            }
        )
    )
    for o in outcomes:
        c = o["config"]
        row = dict(
            matchId=identity(o),
            kind=o["kind"],
            **{k: v for k, v in c.items() if k != "armies"},
            winnerSeat=o.get("winner"),
            **{
                k: o.get(k)
                for k in (
                    "reason",
                    "rounds",
                    "stateHash",
                    "detail",
                    "recruits",
                    "promotions",
                    "casualties",
                    "combatUsage",
                )
            },
        )
        if o["kind"] == "campaign":
            row.update(missions=o.get("missions"), completed=o.get("completed"))
            campaigns.append(row)
        else:
            row.update(
                playerRace=c["armies"]["player"],
                enemyRace=c["armies"]["enemy"],
                race=c["armies"]["player"],
                opponentRace=c["armies"]["enemy"],
                policy=label(c, "player"),
                opponentPolicy=label(c, "enemy"),
                map=c["landscape"],
            )
            rec = dict(
                kind="arena",
                config=dict(
                    player=label(c, "player"),
                    enemy=label(c, "enemy"),
                    armies=c["armies"],
                    landscape=c["landscape"],
                ),
                winner=o["winner"],
                reason=o["reason"],
                rounds=o["rounds"],
            )
            for k in ("recruits", "promotions", "casualties", "combatUsage"):
                fields = ["side", "type"] + (
                    ["attacks", "retaliations", "damage"] if k == "combatUsage" else []
                )
                rec[k] = [{f: u[f] for f in fields} for u in o.get(k, [])]
            records.append(rec)
        ledger.append(row)
    if not records:
        return (
            empty(KEYS, COUNTS + ["winRate", "lossRate", "capRate", "meanRounds"]),
            empty(KEYS + ["unit"], EVENTS),
            ledger,
            campaigns,
        )
    df = pl.from_dicts(
        records,
        infer_schema_length=None,
        schema_overrides={
            "winner": pl.String,
            "reason": pl.String,
            "rounds": pl.Int64,
            "recruits": event_type,
            "promotions": event_type,
            "casualties": event_type,
            "combatUsage": usage_type,
        },
    )
    matches, units = calculate(df)
    return matches, units, ledger, campaigns


def combine(rows, keys, equal=False, cross=True):
    if rows.is_empty():
        return empty(keys, COUNTS + ["winRate"])
    q = rows.lazy()
    if equal:
        q = q.filter(pl.col("policy") == pl.col("opponentPolicy"))
    if cross:
        q = q.filter(pl.col("race") != pl.col("opponentRace"))
    return (
        q.group_by(keys)
        .agg(pl.col(COUNTS).sum())
        .with_columns((100 * pl.col("wins") / pl.col("games")).alias("winRate"))
        .sort(keys, nulls_last=True)
        .collect()
    )


def purchase_shares(events, families):
    if events.is_empty():
        return empty(
            ["race", "policy", "opponentPolicy", "map", "family"],
            ["purchased", "totalPurchases", "sharePercent"],
        )
    q = (
        events.lazy()
        .filter(pl.col("race") != pl.col("opponentRace"))
        .with_columns(pl.col("unit").replace(families).alias("family"))
    )
    q = pl.concat([q, q.with_columns(pl.lit("all").alias("map"))])
    keys = ["race", "policy", "opponentPolicy", "map"]
    return (
        q.group_by(keys + ["family"])
        .agg(pl.col("recruited").sum().alias("purchased"))
        .with_columns(pl.col("purchased").sum().over(keys).alias("totalPurchases"))
        .filter(pl.col("purchased") > 0)
        .with_columns(
            (100 * pl.col("purchased") / pl.col("totalPurchases")).alias("sharePercent")
        )
        .sort(keys + ["family"])
        .collect()
    )
