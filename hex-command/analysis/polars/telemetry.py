"""Telemetry as typed joins and reductions; only capture attribution is sequential."""

import polars as pl
from dataset import cell, concat, expand

DIMS = ["race", "policy", "opponentPolicy", "map"]
VALUE_DIMS = ["kind", "campaign", "diplomacy", "objective", "mission", *DIMS, "family"]
FIELDS = [
    "unitTurns",
    "hpDamageDealt",
    "hpDamageTaken",
    "weaponHPDamageDealt",
    "weaponHPDamageTaken",
    "thornHPDamageDealt",
    "thornHPDamageTaken",
    "armourDamagePrevented",
    "terrainDamagePrevented",
    "rawArmourDelta",
    "rawTerrainDelta",
    "attacksTaken",
    "retaliationsTaken",
    "captures",
    "objectiveCaptures",
    "ownedTileTurns",
    "objectiveControlTurns",
    "capturableOccupationTurns",
    "objectiveOccupationTurns",
    "casualties",
    "recruitGoldLost",
    "casualtyReplacementValue",
]
NONLEADER = [
    "nonleaderUnitTurns",
    "nonleaderWeaponHPDamageDealt",
    "nonleaderThornHPDamageDealt",
]
EVENTS = ["recruited", "promoted", "lost", "attacks", "retaliations", "damage"]
EFFECTS = [
    "providerTurns",
    "adjacentContacts",
    "coveredRecipients",
    "overlappingRecipients",
    "healedHP",
    "healedRecipients",
    "triggeredCombats",
    "reflectedDamage",
    "effectiveReflectedDamage",
]
OVERLAPS = [
    "opportunities",
    "eligiblePairs",
    "contacts",
    "coveredRecipients",
    "overlappingRecipients",
]
CAPTURABLE = ["village", "grove", "fort", "ruins"]


def c(*names):
    return pl.col(*names)


def nonleader():
    return (c("role") != "leader").fill_null(True)


def all_maps(q):
    return pl.concat([q, q.with_columns(pl.lit("all").alias("map"))])


def reduce(q, keys, fields):
    return (
        q.group_by(keys).agg(c(fields).fill_null(0).sum()).sort(keys, nulls_last=True)
    )


def facts(q, fields, **stats):
    return q.select(
        *[n for n in q.columns if n not in fields],
        *[
            (pl.lit(v) if isinstance(v, (int, float)) else v).cast(pl.Float64).alias(k)
            if k in stats
            else pl.lit(0.0).alias(k)
            for k in fields
            for v in [stats.get(k)]
        ],
    )


def join_unit(events, lookup, id="unit"):
    result = events.join(
        lookup.rename({"id": id}),
        on=["p", id],
        how="left",
        validate="m:1",
        suffix="_unit",
    )
    if result["m"].null_count():
        raise ValueError("Recorded event references an unknown unit")
    return result


def recruit_rows(d):
    newborns = d.rosters["units"].join(
        d.rosters["before"].select("p", "id"), on=["p", "id"], how="anti"
    )
    a = (
        d.actions.filter(c("kind") == "recruit")
        .drop("kind", "unit")
        .with_columns(cell(c("tile")).alias("cell"))
        .drop("tile")
    )
    a = a.join(d.phases.select("p", pl.col("side").alias("side")), on="p")
    q = a.join(
        newborns.drop("tile"),
        on=["p", "side", "type", "cell"],
        how="left",
        validate="1:1",
    )
    if q["id"].null_count() or q.height != a.height:
        raise ValueError("Cannot identify recorded recruit role")
    return q


def presence(d, end, lookup, recruits):
    arena = c("kind") == "arena"
    own = end.filter(arena & (c("side") == c("actingSide")) & nonleader())
    dims = d.meta.filter(arena).select(
        "m", "player", "enemy", "playerRace", "enemyRace", "landscape"
    )
    sides = concat(
        [
            dims.select(
                "m",
                c(s).alias("policy"),
                c(o).alias("opponentPolicy"),
                c(s + "Race").alias("race"),
                c("landscape").alias("map"),
                pl.lit(s).alias("side"),
            )
            for s, o in [("player", "enemy"), ("enemy", "player")]
        ]
    )
    battles = all_maps(sides).group_by(DIMS).len(name="battles")
    phases = (
        all_maps(
            d.phases.select("m", "side", "p").join(sides, on=["m", "side"], how="inner")
        )
        .group_by(DIMS)
        .len(name="observedPhases")
    )
    census = all_maps(own).group_by(DIMS + ["type"]).len(name="unitTurns")
    totals = census.group_by(DIMS).agg(c("unitTurns").sum().alias("totalUnitTurns"))
    # Initial, before, casualties and final rosters all count as presence, even
    # when a type has no own-phase exposure (e.g. dies before its first turn).
    seen = concat(
        [
            d.describe(d.initial),
            d.describe(d.rosters["before"]),
            d.describe(d.losses),
            end,
        ],
        schema={},
    ).filter(arena & nonleader())
    tables = {}
    for member, name in [
        ("type", "battlefield_units"),
        ("family", "battlefield_presence"),
    ]:
        p = (
            all_maps(seen.select("m", *DIMS, c(member).alias("member")).unique())
            .group_by(DIMS + ["member"])
            .len(name="battlesPresent")
        )
        exposure = (
            all_maps(own)
            .group_by(DIMS + [member])
            .len(name="unitTurns")
            .rename({member: "member"})
        )
        tables[name] = (
            p.join(exposure, on=DIMS + ["member"], how="left")
            .join(battles, on=DIMS, how="left")
            .join(phases, on=DIMS, how="left")
            .join(totals, on=DIMS, how="left")
            .with_columns(
                c("unitTurns", "observedPhases", "totalUnitTurns").fill_null(0)
            )
            .with_columns(
                (100 * c("unitTurns") / c("totalUnitTurns").clip(lower_bound=1)).alias(
                    "sharePercent"
                ),
                (c("unitTurns") / c("observedPhases").clip(lower_bound=1)).alias(
                    "meanUnitsPerPhase"
                ),
                (100 * c("battlesPresent") / c("battles").clip(lower_bound=1)).alias(
                    "battlePresencePercent"
                ),
            )
            .rename({"member": "unit" if member == "type" else member})
            .sort(DIMS + ["unit" if member == "type" else member])
        )
    role = []

    def event(q, **stats):
        role.append(
            facts(q.filter(arena & nonleader()), EVENTS, **stats).select(
                *DIMS, c("type").alias("unit"), *EVENTS
            )
        )

    event(d.describe(d.losses), lost=1)
    event(d.describe(recruits), recruited=1)
    event(
        join_unit(
            d.actions.filter(c("kind") == "advance").drop(
                "kind", "type", "tile", "cost"
            ),
            lookup,
        ),
        promoted=1,
    )
    combat = d.combats
    event(
        join_unit(combat.select("p", "unit", c("damage").alias("__damage")), lookup),
        attacks=1,
        damage=c("__damage"),
    )
    event(
        join_unit(
            combat.filter(
                (c("retaliationCount").fill_null(0) > 0)
                | (c("retaliation").fill_null(0) > 0)
            ).select(
                "p",
                c("target").alias("unit"),
                c("retaliation").alias("__damage"),
                c("retaliationCount").fill_null(0).alias("__count"),
            ),
            lookup,
        ),
        retaliations=(c("__count") > 0),
        damage=c("__damage"),
    )
    # facts expressions need the event columns until projection.
    tables["nonleader_unit_events"] = reduce(concat(role), DIMS + ["unit"], EVENTS)
    return tables


def support(d, before, lookup):
    friends = before.filter(c("kind") == "arena").with_columns(
        (c("tile").struct.field("col") - c("tile").struct.field("row") // 2).alias("q"),
        c("tile").struct.field("row").alias("r"),
    )
    effect_parts, overlap_parts = [], []

    def effect(q, ability, **stats):
        q = q.with_columns(pl.lit(ability).alias("ability"))
        effect_parts.append(
            facts(q, EFFECTS, **stats).select(
                *DIMS, c("type").alias("unit"), "family", "ability", *EFFECTS
            )
        )

    for ability in ("healing", "thornAura", "thorns"):
        opportunity = friends.filter(
            c("side") == c("actingSide")
            if ability == "healing"
            else c("side") != c("actingSide")
        )
        providers = opportunity.filter(c(ability) > 0)
        effect(providers, ability, providerTurns=1)
        if ability == "thorns":
            continue
        sizes = opportunity.group_by("p", "side").len(name="friends")
        counts = (
            providers.group_by("p", "side", *DIMS)
            .len(name="providerCount")
            .join(sizes, on=["p", "side"])
        )
        recipients = opportunity.select(
            "p",
            "side",
            c("id").alias("recipient"),
            c("q").alias("rq"),
            c("r").alias("rr"),
        )
        pairs = (
            providers.join(recipients, on=["p", "side"])
            .filter(c("id") != c("recipient"))
            .with_columns(
                (c("q") - c("rq")).alias("dq"), (c("r") - c("rr")).alias("dr")
            )
        )
        touching = pairs.filter(
            c("dq").abs() + c("dr").abs() + (c("dq") + c("dr")).abs() == 2
        ).with_columns(pl.len().over(["p", "recipient"]).alias("touching"))
        effect(
            touching,
            ability,
            adjacentContacts=1,
            coveredRecipients=1 / c("touching"),
            overlappingRecipients=pl.when(c("touching") > 1)
            .then(1 / c("touching"))
            .otherwise(0),
        )
        measured = touching.group_by("p", "side").agg(
            pl.len().alias("contacts"),
            c("recipient").n_unique().alias("coveredRecipients"),
            c("recipient")
            .filter(c("touching") > 1)
            .n_unique()
            .alias("overlappingRecipients"),
        )
        overlap_parts.append(
            counts.join(measured, on=["p", "side"], how="left")
            .with_columns(
                c("contacts", "coveredRecipients", "overlappingRecipients").fill_null(0)
            )
            .select(
                *DIMS,
                pl.lit(ability).alias("ability"),
                "providerCount",
                (c("friends") - 1).clip(lower_bound=0).alias("recipientsPerProvider"),
                pl.lit(1).alias("opportunities"),
                (c("providerCount") * (c("friends") - 1).clip(lower_bound=0)).alias(
                    "eligiblePairs"
                ),
                "contacts",
                "coveredRecipients",
                "overlappingRecipients",
            )
        )
    arena_ids = (
        d.phases.select("p", "m")
        .join(d.meta.filter(c("kind") == "arena").select("m"), on="m", how="inner")
        .select("p")
    )
    healing = d.healing.join(arena_ids, on="p", how="semi").with_columns(
        c("providers").list.len().alias("n")
    )
    if healing.filter(c("n") == 0).height:
        raise ValueError("Healing event has no provider")
    healing = healing.explode("providers").rename({"providers": "unit"})
    effect(
        join_unit(healing, lookup),
        "healing",
        healedHP=c("amount") / c("n"),
        healedRecipients=1 / c("n"),
    )
    arena_combats = d.combats.join(arena_ids, on="p", how="semi")
    if arena_combats.filter(c("thorns").is_null()).height:
        raise ValueError("Support telemetry missing from combat record")
    thorns = (
        arena_combats.filter(c("thorns").is_not_null())
        .select(
            "p",
            "target",
            c("thorns").struct.field("damage").alias("raw"),
            c("thorns").struct.field("auraDamage").alias("aura"),
            c("thorns").struct.field("effectiveDamage").alias("effective"),
            c("thorns").struct.field("auraSources").alias("sources"),
        )
        .filter(c("raw") > 0)
        .with_columns(
            (c("raw") - c("aura")).alias("own"), c("sources").list.len().alias("n")
        )
    )
    if thorns.filter((c("aura") != 0) & (c("n") == 0)).height:
        raise ValueError("Thorn aura event has no provider")
    effect(
        join_unit(thorns.filter(c("own") != 0).rename({"target": "unit"}), lookup),
        "thorns",
        triggeredCombats=1,
        reflectedDamage=c("own"),
        effectiveReflectedDamage=c("effective") * c("own") / c("raw"),
    )
    effect(
        join_unit(
            thorns.explode("sources")
            .filter(c("sources").is_not_null())
            .rename({"sources": "unit"}),
            lookup,
        ),
        "thornAura",
        triggeredCombats=1 / c("n"),
        reflectedDamage=c("aura") / c("n"),
        effectiveReflectedDamage=c("effective") * c("aura") / c("n") / c("raw"),
    )
    effects = concat(effect_parts)
    effects = concat(
        [
            effects,
            effects.with_columns(
                pl.lit("all").alias("unit"), pl.lit("all").alias("family")
            ),
        ]
    )
    effects = reduce(all_maps(effects), DIMS + ["unit", "family", "ability"], EFFECTS)
    turns, ability = c("providerTurns"), c("ability")

    def rate(field, condition):
        return pl.when((turns > 0) & condition).then(c(field) / turns).otherwise(None)

    effects = effects.with_columns(
        rate("adjacentContacts", ability != "thorns").alias("adjacentAlliesPerTurn"),
        rate("healedRecipients", ability == "healing").alias("usefulRecipientsPerTurn"),
        rate("healedHP", ability == "healing").alias("healedHPPerTurn"),
        rate("reflectedDamage", ability != "healing").alias("reflectedDamagePerTurn"),
        pl.when(c("coveredRecipients") > 0)
        .then(100 * c("overlappingRecipients") / c("coveredRecipients"))
        .otherwise(0)
        .alias("overlapPercent"),
    )
    overlaps = reduce(
        all_maps(concat(overlap_parts)),
        DIMS + ["ability", "providerCount", "recipientsPerProvider"],
        OVERLAPS,
    ).with_columns(
        (c("contacts") / c("eligiblePairs").clip(lower_bound=1)).alias(
            "pairCoverageProbability"
        ),
        (
            100
            * c("overlappingRecipients")
            / c("coveredRecipients").clip(lower_bound=1)
        ).alias("overlapPercent"),
        (
            100
            * (c("contacts") - c("coveredRecipients"))
            / c("contacts").clip(lower_bound=1)
        ).alias("redundantContactPercent"),
    )
    return dict(support_effects=effects, support_overlap=overlaps)


def objectives(d):
    return (
        expand(d.phases, "objectives")
        .filter(c("kind") == "capture")
        .select("p", c("tiles").alias("tile"))
        .explode("tile")
        .filter(c("tile").is_not_null())
        .with_columns(cell(c("tile")).alias("cell"))
        .join(d.phases.select("p", "m", "ch", "side"), on="p")
        .select("m", "ch", "side", "cell")
        .unique()
    )


def capture_facts(d, lookup, objective_tiles):
    """The sole sequential metric pass: ownership changes and last captor.

    Its input is projected typed move events, not reconstructed phase trees.
    It returns compact capture/control facts for the ordinary columnar reduction.
    """
    moves = join_unit(
        d.actions.filter(c("kind") == "move").select(
            "p", "action", "unit", cell(c("tile")).alias("destination")
        ),
        lookup,
    ).sort("p", "action")
    by_phase = {
        k[0]: v.to_dicts() for k, v in moves.partition_by("p", as_dict=True).items()
    }
    targets = set(objective_tiles.iter_rows())
    tile_tables = {
        k: v for k, v in d.tiles.partition_by(["m", "ch"], as_dict=True).items()
    }
    headers = {m: result for m, (_, result, _) in enumerate(d.headers)}
    state = {}
    output = {
        k: []
        for k in [
            "p",
            "id",
            "type",
            "side",
            "role",
            "m",
            "ch",
            "captures",
            "objectiveCaptures",
            "ownedTileTurns",
            "objectiveControlTurns",
        ]
    }

    def emit(row, p, **stats):
        for k in output:
            output[k].append(
                p
                if k == "p"
                else stats.get(k, 0)
                if k
                in (
                    "captures",
                    "objectiveCaptures",
                    "ownedTileTurns",
                    "objectiveControlTurns",
                )
                else row.get(k)
            )

    for p, m, ch, side_, owner_index in (
        d.refs.join(d.phases.select("p", "side"), on="p")
        .select("p", "m", "ch", "side", "ownership")
        .iter_rows()
    ):
        key = m, ch
        if key not in state:
            tiles = tile_tables.get(key, d.tiles.head(0))
            owners = dict(tiles.select("tile", "owner").iter_rows())
            terrain = dict(tiles.select("tile", "terrain").iter_rows())
            result = headers[m]
            chapter = result if result["kind"] == "arena" else result["missions"][ch]
            factions = {
                f["id"]: f for f in chapter.get("initial", {}).get("factions", [])
            }
            state[key] = (owners, terrain, {}, factions, list(owners))
        owners, terrain, captors, factions, order = state[key]
        for row in by_phase.get(p, []):
            tile = row["destination"]
            faction = row.get("faction") or row["side"]
            owner = owners.get(tile)
            relation = factions.get(faction, {}).get("relations", {}).get(owner)
            if relation is None:
                relation = factions.get(owner, {}).get("relations", {}).get(faction)
            hostile = (
                relation == "hostile"
                if relation is not None
                else {faction, owner} == {"player", "enemy"}
            )
            if terrain[tile] in CAPTURABLE and (owner is None or hostile):
                owners[tile] = faction
                captors[tile] = row
                emit(
                    row,
                    p,
                    captures=1,
                    objectiveCaptures=int((m, ch, row["side"], tile) in targets),
                )
        values = d.owners[m][owner_index].as_py()
        if len(values) != len(order):
            raise ValueError("Invalid Arrow ownership snapshot")
        observed = dict(zip(order, values))
        for tile, row in captors.items():
            if (
                observed.get(tile) == (row.get("faction") or row["side"])
                and row["side"] == side_
            ):
                emit(
                    row,
                    p,
                    ownedTileTurns=1,
                    objectiveControlTurns=int((m, ch, side_, tile) in targets),
                )
        owners.update(observed)
    return pl.DataFrame(
        output,
        schema={
            "p": pl.Int64,
            "id": pl.String,
            "type": pl.String,
            "side": pl.String,
            "role": pl.String,
            "m": pl.Int64,
            "ch": pl.Int64,
            **{
                k: pl.Float64
                for k in (
                    "captures",
                    "objectiveCaptures",
                    "ownedTileTurns",
                    "objectiveControlTurns",
                )
            },
        },
    )


def value(d, end, lookup, recruits):
    parts = []

    def add(q, **stats):
        parts.append(
            facts(q, FIELDS, **stats).select(*VALUE_DIMS, "type", "role", *FIELDS)
        )

    objective_tiles = objectives(d)
    own = (
        end.filter(c("side") == c("actingSide"))
        .join(
            d.tiles.select("m", "ch", c("tile").alias("cell"), "terrain"),
            on=["m", "ch", "cell"],
            how="left",
        )
        .join(
            objective_tiles.with_columns(pl.lit(True).alias("isObjective")),
            on=["m", "ch", "side", "cell"],
            how="left",
        )
    )
    add(
        own,
        unitTurns=1,
        capturableOccupationTurns=c("terrain").is_in(CAPTURABLE),
        objectiveOccupationTurns=c("isObjective").fill_null(False),
    )
    # Replacement prices are faction-specific. Recruitment expenditure belongs
    # to an instance and survives promotions until its recorded casualty.
    costs = []
    for m, (_, result, _) in enumerate(d.headers):
        for ch, chapter in enumerate(
            [result] if result["kind"] == "arena" else result.get("missions", [])
        ):
            for f in chapter.get("initial", {}).get("factions", []):
                costs.extend(
                    dict(
                        m=m,
                        ch=ch,
                        faction=f["id"],
                        type=r["unit"],
                        replacement=r.get("cost"),
                    )
                    for r in f.get("recruits", [])
                )
    cost_table = pl.from_dicts(
        costs,
        schema={
            "m": pl.Int64,
            "ch": pl.Int64,
            "faction": pl.String,
            "type": pl.String,
            "replacement": pl.Float64,
        },
    )
    bought = (
        d.describe(recruits)
        .join(cost_table, on=["m", "ch", "faction", "type"], how="left")
        .select(
            "m",
            "ch",
            "id",
            pl.coalesce("cost", "replacement", "cost_profile").alias("invested"),
        )
    )
    losses = (
        d.describe(d.losses)
        .join(cost_table, on=["m", "ch", "faction", "type"], how="left")
        .join(bought, on=["m", "ch", "id"], how="left", validate="m:1")
    )
    add(
        losses,
        casualties=1,
        recruitGoldLost=c("invested").fill_null(0),
        casualtyReplacementValue=pl.coalesce("replacement", "cost"),
    )
    combats = d.combats
    if combats.filter(c("mitigation").is_null()).height:
        raise ValueError("Combat mitigation telemetry missing from combat record")
    for response, name, actor, defender in [
        (False, "attack", "unit", "target"),
        (True, "retaliation", "target", "unit"),
    ]:
        event = (
            combats.select(
                "p",
                c(actor).alias("actor"),
                c(defender).alias("defender"),
                c("mitigation").struct.field(name).alias("event"),
            )
            .filter(c("event").is_not_null())
            .unnest("event")
            .with_columns(
                pl.coalesce(
                    "effectiveDamage",
                    pl.min_horizontal("afterTerrainDamage", "availableHp"),
                ).alias("actual")
            )
        )
        add(
            join_unit(event.rename({"actor": "unit"}), lookup),
            hpDamageDealt=c("actual"),
            weaponHPDamageDealt=c("actual"),
        )
        add(
            join_unit(event.rename({"defender": "unit"}), lookup),
            hpDamageTaken=c("actual"),
            weaponHPDamageTaken=c("actual"),
            armourDamagePrevented=c("effectiveArmourPrevented"),
            terrainDamagePrevented=c("effectiveTerrainPrevented"),
            rawArmourDelta=c("rawDamage") - c("afterArmourDamage"),
            rawTerrainDelta=c("afterArmourDamage") - c("afterTerrainDamage"),
            attacksTaken=int(not response),
            retaliationsTaken=int(response),
        )
    thorns = (
        combats.select(
            "p",
            "unit",
            "target",
            c("thorns").struct.field("damage").alias("raw"),
            c("thorns").struct.field("auraDamage").alias("aura"),
            c("thorns").struct.field("effectiveDamage").alias("effective"),
            c("thorns").struct.field("auraSources").alias("sources"),
        )
        .filter(c("effective") > 0)
        .with_columns(
            (c("raw") - c("aura")).alias("own"), c("sources").list.len().alias("n")
        )
    )
    if thorns.filter((c("aura") != 0) & (c("n") == 0)).height:
        raise ValueError("Thorn aura event has no provider")
    add(
        join_unit(thorns, lookup),
        hpDamageTaken=c("effective"),
        thornHPDamageTaken=c("effective"),
    )
    reflected = join_unit(
        thorns.filter(c("own") != 0).drop("unit").rename({"target": "unit"}), lookup
    )
    add(
        reflected,
        hpDamageDealt=c("effective") * c("own") / c("raw"),
        thornHPDamageDealt=c("effective") * c("own") / c("raw"),
    )
    aura = join_unit(
        thorns.drop("unit")
        .explode("sources")
        .filter(c("sources").is_not_null())
        .rename({"sources": "unit"}),
        lookup,
    )
    add(
        aura,
        hpDamageDealt=c("effective") * c("aura") / c("raw") / c("n"),
        thornHPDamageDealt=c("effective") * c("aura") / c("raw") / c("n"),
    )
    captures = capture_facts(d, lookup, objective_tiles)
    add(
        d.describe(captures),
        **{
            k: c(k)
            for k in (
                "captures",
                "objectiveCaptures",
                "ownedTileTurns",
                "objectiveControlTurns",
            )
        },
    )
    q = all_maps(concat(parts))
    families = reduce(q, VALUE_DIMS, FIELDS)
    q = q.with_columns(
        pl.when(nonleader()).then(c(source)).otherwise(0).alias(target)
        for source, target in zip(
            ["unitTurns", "weaponHPDamageDealt", "thornHPDamageDealt"], NONLEADER
        )
    )
    units = reduce(q, VALUE_DIMS[:-1] + ["type"], FIELDS + NONLEADER).rename(
        {"type": "unit"}
    )
    family_map = d.profiles.select(c("type").alias("unit"), "family").unique()
    units = units.join(family_map, on="unit", how="left", validate="m:1")
    return dict(battlefield_value=families, battlefield_unit_value=units)


def leader_tables(d, lookup):
    a = join_unit(
        d.audits.rename({"leader": "unit", "side": "auditSide"}), lookup
    ).join(d.phases.select("p", c("side").alias("actingSide"), "round"), on="p")
    before = c("before")
    after = c("after")
    army = c("armyBefore")
    end = c("armyAfter")
    foe = c("opponentBefore")
    exposure = c("exposure")
    chosen = exposure.struct.field("chosenTileDamage")
    minimum = exposure.struct.field("minDamage")
    hp = before.struct.field("hp")
    last = d.combats.select("p", "combat", "unit", "target")
    last = (
        concat(
            [
                last.select(
                    "p",
                    "combat",
                    c("unit").alias("leader"),
                    pl.lit(True).alias("lastOwnAttack"),
                ),
                last.select(
                    "p",
                    "combat",
                    c("target").alias("leader"),
                    pl.lit(False).alias("lastOwnAttack"),
                ),
            ]
        )
        .sort("combat")
        .unique(["p", "leader"], keep="last")
    )
    a = a.rename({"unit": "leader"}).join(last, on=["p", "leader"], how="left")
    q = a.select(
        *VALUE_DIMS[:-1],
        "matchId",
        "seed",
        "first",
        c("p").alias("__order"),
        "m",
        "ch",
        "p",
        "round",
        "actingSide",
        c("auditSide").alias("side"),
        "leader",
        c("type").alias("unit"),
        after.is_null().alias("died"),
        hp.alias("beforeHP"),
        after.struct.field("hp").fill_null(0).alias("afterHP"),
        before.struct.field("tile").alias("beforeTile"),
        after.struct.field("tile").alias("afterTile"),
        (c("actingSide") == c("auditSide")).alias("ownDecision"),
        *[
            expr.alias(name)
            for name, expr in {
                "armyUnitsBefore": army.struct.field("units"),
                "armyUnitsAfter": end.struct.field("units"),
                "armyHPValueBefore": army.struct.field("hpValue"),
                "armyHPValueAfter": end.struct.field("hpValue"),
                "opponentHPValueBefore": foe.struct.field("hpValue"),
                "opponentHPValueAfter": c("opponentAfter").struct.field("hpValue"),
                "armyValueRatioBefore": pl.when(foe.struct.field("hpValue") != 0).then(
                    army.struct.field("hpValue") / foe.struct.field("hpValue")
                ),
                "armyRetainedFraction": pl.when(army.struct.field("hpValue") != 0).then(
                    end.struct.field("hpValue") / army.struct.field("hpValue")
                ),
                "armyHPBefore": army.struct.field("hp"),
                "armyHPAfter": end.struct.field("hp"),
                "deathDuringOwnAttack": after.is_null()
                & c("lastOwnAttack").fill_null(False),
                "forecastLethalWithSafeAlternative": pl.when(
                    chosen.is_not_null() & minimum.is_not_null()
                ).then((chosen >= hp) & (minimum < hp)),
                "endForecastLethal": pl.when(after.is_not_null()).then(
                    exposure.struct.field("afterDamage") >= after.struct.field("hp")
                ),
            }.items()
        ],
        *[
            c("recruitmentBefore").struct.field(k).alias(k)
            for k in ("gold", "onOwnedKeep", "legalOptions", "affordableOptions")
        ],
        *[exposure.struct.field(k).alias(k) for k in EXPOSURE_FIELDS],
    )
    q = q.with_columns(
        (c("p").rank("dense").over(["m", "ch"]) - 1).cast(pl.Int64).alias("phase")
    ).drop("m", "ch", "p")
    keys = ["matchId", "mission", "side", "leader"]
    q = (
        q.sort("__order", "leader")
        .with_columns(
            pl.max_horizontal("armyHPValueBefore", "armyHPValueAfter")
            .cum_max()
            .over(keys)
            .alias("armyPeakHPValue")
        )
        .with_columns(
            pl.when(c("armyPeakHPValue") > 0)
            .then(c("armyHPValueAfter") / c("armyPeakHPValue"))
            .alias("armyFractionOfPeakAfter")
        )
    )
    fields = [
        "round",
        "beforeHP",
        "armyUnitsBefore",
        "armyHPValueBefore",
        "armyValueRatioBefore",
        "beforeDamage",
        "chosenTileDamage",
        "minDamage",
        "safeTiles",
        "forecastLethalWithSafeAlternative",
        "afterHP",
        "afterDamage",
        "endForecastLethal",
    ]
    decisions = q.filter("ownDecision").select(
        *keys,
        c("__order").alias("__decision"),
        *[c(k).alias("lastDecision" + k[0].upper() + k[1:]) for k in fields],
    )
    deaths = (
        q.filter("died")
        .join_asof(
            decisions,
            left_on="__order",
            right_on="__decision",
            by=keys,
            check_sortedness=False,
        )
        .drop("__order", "__decision")
    )
    if a.filter(c("after").is_null() & c("exposure").is_not_null()).is_empty():
        deaths = deaths.drop(EXPOSURE_FIELDS)
    return dict(leader_audit=q.drop("__order"), leader_deaths=deaths)


EXPOSURE_FIELDS = [
    "beforeDamage",
    "chosenTileDamage",
    "minDamage",
    "minTile",
    "reachableTiles",
    "safeTiles",
    "chosenTileReachable",
    "afterDamage",
]


def analyze(d, *, presence_=False, support_=False, value_=False, leaders_=False):
    catalogs = None
    for _, result, report in d.headers:
        if (
            support_
            and result["kind"] == "arena"
            and (
                report.get("supportTelemetryVersion") != 1
                or not report.get("unitProfiles")
            )
        ):
            raise ValueError(
                "Support effects require fresh support telemetry and recorded unitProfiles"
            )
        if value_ and report.get("combatTelemetryVersion") != 1:
            raise ValueError("Battlefield value requires fresh combat telemetry")
        if leaders_ and report.get("leaderAuditVersion") != 1:
            raise ValueError("Leader audit requires fresh leader-risk telemetry")
        profiles = report.get("unitProfiles") or {}
        if catalogs is not None and catalogs != profiles:
            raise ValueError("Cannot pool telemetry across different recorded catalogs")
        catalogs = profiles
    if not d.phases.height:
        tables = {}
        if presence_:
            from frames import empty

            tables.update(
                battlefield_units=empty(DIMS + ["unit"], ["unitTurns"]),
                battlefield_presence=empty(DIMS + ["family"], ["unitTurns"]),
                nonleader_unit_events=empty(DIMS + ["unit"], EVENTS),
            )
        if support_:
            tables.update(
                support_effects=pl.DataFrame(), support_overlap=pl.DataFrame()
            )
        if value_:
            tables.update(
                battlefield_value=pl.DataFrame(), battlefield_unit_value=pl.DataFrame()
            )
        if leaders_:
            tables.update(leader_audit=pl.DataFrame(), leader_deaths=pl.DataFrame())
        return tables
    end = d.describe(d.rosters["units"]).join(
        d.phases.select("p", c("side").alias("actingSide")), on="p"
    )
    before = d.describe(d.rosters["before"]).join(
        d.phases.select("p", c("side").alias("actingSide")), on="p"
    )
    lookup = d.describe(d.lookup)
    recruits = recruit_rows(d)
    tables = {}
    if presence_:
        tables.update(presence(d, end, lookup, recruits))
    if support_:
        tables.update(support(d, before, d.describe(d.support_lookup)))
    if value_:
        tables.update(value(d, end, lookup, recruits))
    if leaders_:
        tables.update(leader_tables(d, lookup))
    return tables
