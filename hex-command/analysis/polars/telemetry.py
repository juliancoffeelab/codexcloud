"""Interpret recorded events, then reduce bounded batches with Polars.

Stateful capture/recruit attribution follows the recorded phase order. Numerical
reductions, exposure joins, support rates and leader windows are columnar.
"""

import json
import polars as pl
from frames import Accumulator
from journal import journals

DIMS = ["race", "policy", "opponentPolicy", "map"]
EFFECT_FIELDS = [
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


def cell_key(tile):
    return f"{tile['col']},{tile['row']}" if isinstance(tile, dict) else tile


def adjacent(a, b, positions):
    x, y = positions[cell_key(a)]
    u, v = positions[cell_key(b)]
    q, r = x - y // 2 - u + v // 2, y - v
    return abs(q) + abs(r) + abs(q + r) == 2


def presence_support(reports, presence=True, support=True):
    census = Accumulator(DIMS + ["unit"], ["unitTurns"])
    totals = Accumulator(DIMS, ["battles", "phases", "unitTurns"])
    present = Accumulator(DIMS + ["grouping", "member"], ["battlesPresent"])
    role_events = Accumulator(
        DIMS + ["unit"],
        ["recruited", "promoted", "lost", "attacks", "retaliations", "damage"],
    )
    effects = Accumulator(DIMS + ["unit", "ability"], EFFECT_FIELDS)
    overlaps = Accumulator(
        DIMS + ["ability", "providerCount", "recipientsPerProvider"],
        [
            "opportunities",
            "eligiblePairs",
            "contacts",
            "coveredRecipients",
            "overlappingRecipients",
        ],
    )
    catalogs = None
    matches = 0
    for report, result in journals(reports):
        config = result["config"]
        if config["armies"]["player"] == config["armies"]["enemy"]:
            continue
        profiles = report.get("unitProfiles") or {}
        families = report.get("unitFamilies") or {}
        if support and (report.get("supportTelemetryVersion") != 1 or not profiles):
            raise ValueError(
                "Support effects require a fresh run with support telemetry and recorded unitProfiles"
            )
        if profiles:
            if catalogs is not None and catalogs != profiles:
                raise ValueError(
                    "Cannot pool unit usage across different recorded catalogs"
                )
            catalogs = profiles
        matches += 1

        def group(side, landscape):
            other = "enemy" if side == "player" else "player"

            def label(s):
                blocks = config.get(s + "Blocks")
                return (
                    "-".join(blocks[k] for k in ("recruiter", "campaign", "planner"))
                    if blocks
                    else config[s]
                )

            return config["armies"][side], label(side), label(other), landscape

        def leader(unit):
            return unit.get("role") == "leader"

        def family(unit):
            return profiles.get(unit, {}).get("family", families.get(unit, unit))

        def add_effect(unit, ability, **values):
            # All-unit rows aggregate the same measured events, not extra events.
            for landscape in (config["landscape"], "all"):
                key = group(unit["side"], landscape)
                for kind in (unit["type"], "all"):
                    effects.add(
                        dict(zip(DIMS, key), unit=kind, ability=ability, **values)
                    )

        positions = {t["id"]: (t["col"], t["row"]) for t in result["initial"]["tiles"]}
        seen = {side: set() for side in ("player", "enemy")}
        for unit in result["initial"]["units"]:
            if not leader(unit):
                seen[unit["side"]].add(unit["type"])
        for side in ("player", "enemy"):
            for landscape in (config["landscape"], "all"):
                totals.add(dict(zip(DIMS, group(side, landscape)), battles=1))
        for phase in result["phases"]:
            side = phase["side"]
            if presence:
                own = [u for u in phase["units"] if u["side"] == side and not leader(u)]
                for landscape in (config["landscape"], "all"):
                    key = group(side, landscape)
                    totals.add(dict(zip(DIMS, key), phases=1))
                    totals.add(dict(zip(DIMS, key), unitTurns=len(own)))
                    for unit in own:
                        census.add(dict(zip(DIMS, key), unit=unit["type"], unitTurns=1))
                for unit in (
                    phase.get("before", []) + phase["units"] + phase.get("losses", [])
                ):
                    if not leader(unit):
                        seen[unit["side"]].add(unit["type"])
            if presence:
                # Match the engine's post-promotion/casualty lookup order, but
                # keep instance roles through the aggregation boundary.
                roster = {
                    u["id"]: u
                    for u in phase.get("before", [])
                    + phase.get("losses", [])
                    + phase["units"]
                }

                def role_event(unit, **stats):
                    if leader(unit):
                        return
                    key = (*group(unit["side"], config["landscape"]), unit["type"])
                    role_events.add(dict(zip(DIMS + ["unit"], key), **stats))

                for loss in phase.get("losses", []):
                    role_event(loss, lost=1)
                for action in phase.get("actions", []):
                    if action["kind"] == "advance":
                        role_event(roster[action["unit"]], promoted=1)
                    elif action["kind"] == "recruit":
                        previous = {u["id"] for u in phase.get("before", [])}
                        spawned = [
                            u
                            for u in phase["units"]
                            if u["id"] not in previous
                            and u["type"] == action["type"]
                            and u["tile"] == action["tile"]
                            and u["side"] == side
                        ]
                        if len(spawned) != 1:
                            raise ValueError("Cannot identify recorded recruit role")
                        role_event(spawned[0], recruited=1)
                for combat in phase.get("combats", []):
                    role_event(
                        roster[combat["unit"]],
                        attacks=1,
                        damage=combat.get("damage", 0),
                    )
                    if combat.get("retaliationCount", 0) or combat.get(
                        "retaliation", 0
                    ):
                        role_event(
                            roster[combat["target"]],
                            retaliations=int(combat.get("retaliationCount", 0) > 0),
                            damage=combat.get("retaliation", 0),
                        )
            if not support:
                continue
            before = phase["before"]
            lookup = {
                u["id"]: u for u in phase.get("losses", []) + phase["units"] + before
            }
            for ability in ("healing", "thornAura", "thorns"):
                # Healing opportunities occur at own-turn start. Reflection
                # opportunities occur at opposing-turn start, after positioning.
                provider_side = (
                    side
                    if ability == "healing"
                    else ("enemy" if side == "player" else "player")
                )
                friends = [u for u in before if u["side"] == provider_side]
                providers = [
                    u for u in friends if profiles[u["type"]].get(ability, 0) > 0
                ]
                for provider in providers:
                    add_effect(provider, ability, providerTurns=1)
                if ability == "thorns":
                    continue
                composition = dict(
                    opportunities=1,
                    eligiblePairs=len(providers) * max(0, len(friends) - 1),
                    contacts=0,
                    coveredRecipients=0,
                    overlappingRecipients=0,
                )
                for ally in friends:
                    touching = [
                        p
                        for p in providers
                        if p["id"] != ally["id"]
                        and adjacent(p["tile"], ally["tile"], positions)
                    ]
                    composition["contacts"] += len(touching)
                    composition["coveredRecipients"] += bool(touching)
                    composition["overlappingRecipients"] += len(touching) > 1
                    for provider in touching:
                        add_effect(
                            provider,
                            ability,
                            adjacentContacts=1,
                            coveredRecipients=1 / len(touching),
                            overlappingRecipients=(
                                1 / len(touching) if len(touching) > 1 else 0
                            ),
                        )
                if providers:
                    for landscape in (config["landscape"], "all"):
                        overlaps.add(
                            dict(
                                zip(DIMS, group(provider_side, landscape)),
                                ability=ability,
                                providerCount=len(providers),
                                recipientsPerProvider=max(0, len(friends) - 1),
                                **composition,
                            )
                        )
            for event in phase.get("healing", []):
                providers = event["providers"]
                if not providers:
                    raise ValueError("Healing event has no provider")
                for provider in providers:
                    add_effect(
                        lookup[provider],
                        "healing",
                        healedHP=event["amount"] / len(providers),
                        healedRecipients=1 / len(providers),
                    )
            for combat in phase["combats"]:
                thorns = combat.get("thorns")
                if thorns is None:
                    raise ValueError("Support telemetry missing from combat record")
                raw = thorns["damage"]
                if raw <= 0:
                    continue
                own = raw - thorns["auraDamage"]
                effective = thorns["effectiveDamage"]
                if own:
                    add_effect(
                        lookup[combat["target"]],
                        "thorns",
                        triggeredCombats=1,
                        reflectedDamage=own,
                        effectiveReflectedDamage=effective * own / raw,
                    )
                sources = thorns["auraSources"]
                if thorns["auraDamage"] and not sources:
                    raise ValueError("Thorn aura event has no provider")
                for source in sources:
                    damage = thorns["auraDamage"] / len(sources)
                    add_effect(
                        lookup[source],
                        "thornAura",
                        triggeredCombats=1 / len(sources),
                        reflectedDamage=damage,
                        effectiveReflectedDamage=effective * damage / raw,
                    )
        for side, units in seen.items():
            for landscape in (config["landscape"], "all"):
                key = group(side, landscape)
                for unit in units:
                    present.add(
                        dict(
                            zip(DIMS, key),
                            grouping="unit",
                            member=unit,
                            battlesPresent=1,
                        )
                    )
                for fam in {family(unit) for unit in units}:
                    present.add(
                        dict(
                            zip(DIMS, key),
                            grouping="family",
                            member=fam,
                            battlesPresent=1,
                        )
                    )

    tables = {}
    if presence:
        tables["nonleader_unit_events"] = role_events.finish()
        c, t, p = census.finish(), totals.finish(), present.finish()
        if p:
            census_df = (
                pl.from_dicts(c)
                if c
                else pl.DataFrame(
                    schema={
                        **{k: pl.String for k in DIMS},
                        "unit": pl.String,
                        "unitTurns": pl.Int64,
                    }
                )
            )
            family_map = {
                k: v.get("family", k) for k, v in (catalogs or {}).items()
            } or families
            for grouping, name in [
                ("unit", "battlefield_units"),
                ("family", "battlefield_presence"),
            ]:
                exposure = (
                    census_df.lazy()
                    .with_columns(
                        pl.col("unit").replace(family_map).alias("member")
                        if grouping == "family"
                        else pl.col("unit").alias("member")
                    )
                    .group_by(DIMS + ["member"])
                    .agg(pl.col("unitTurns").sum())
                )
                rows = (
                    pl.from_dicts(p)
                    .lazy()
                    .filter(pl.col("grouping") == grouping)
                    .drop("grouping")
                    .join(exposure, on=DIMS + ["member"], how="left", validate="1:1")
                    .join(
                        pl.from_dicts(t)
                        .lazy()
                        .rename(
                            {"unitTurns": "totalUnitTurns", "phases": "observedPhases"}
                        ),
                        on=DIMS,
                        how="left",
                        validate="m:1",
                    )
                    .with_columns(pl.col("unitTurns").fill_null(0))
                    .with_columns(
                        (
                            100
                            * pl.col("unitTurns")
                            / pl.col("totalUnitTurns").clip(lower_bound=1)
                        ).alias("sharePercent"),
                        (
                            pl.col("unitTurns")
                            / pl.col("observedPhases").clip(lower_bound=1)
                        ).alias("meanUnitsPerPhase"),
                        (
                            100
                            * pl.col("battlesPresent")
                            / pl.col("battles").clip(lower_bound=1)
                        ).alias("battlePresencePercent"),
                    )
                    .rename({"member": grouping})
                    .sort(DIMS + [grouping])
                    .collect()
                    .to_dicts()
                )
                tables[name] = rows
        else:
            tables.update(battlefield_units=[], battlefield_presence=[])
    if support:
        rows = effects.finish()
        if rows:
            q = (
                pl.from_dicts(rows)
                .lazy()
                .with_columns(
                    pl.col("unit")
                    .replace({k: v["family"] for k, v in catalogs.items()})
                    .alias("family")
                )
            )
            turns = pl.col("providerTurns")
            ability = pl.col("ability")

            def rate(field, condition):
                return (
                    pl.when((turns > 0) & condition)
                    .then(pl.col(field) / turns)
                    .otherwise(None)
                )

            q = q.with_columns(
                rate("adjacentContacts", ability != "thorns").alias(
                    "adjacentAlliesPerTurn"
                ),
                rate("healedRecipients", ability == "healing").alias(
                    "usefulRecipientsPerTurn"
                ),
                rate("healedHP", ability == "healing").alias("healedHPPerTurn"),
                rate("reflectedDamage", ability != "healing").alias(
                    "reflectedDamagePerTurn"
                ),
                pl.when(pl.col("coveredRecipients") > 0)
                .then(
                    100 * pl.col("overlappingRecipients") / pl.col("coveredRecipients")
                )
                .otherwise(0)
                .alias("overlapPercent"),
            )
            tables["support_effects"] = q.collect().to_dicts()
        else:
            tables["support_effects"] = []
        rows = overlaps.finish()
        tables["support_overlap"] = (
            (
                pl.from_dicts(rows)
                .lazy()
                .with_columns(
                    (
                        pl.col("contacts") / pl.col("eligiblePairs").clip(lower_bound=1)
                    ).alias("pairCoverageProbability"),
                    (
                        100
                        * pl.col("overlappingRecipients")
                        / pl.col("coveredRecipients").clip(lower_bound=1)
                    ).alias("overlapPercent"),
                    (
                        100
                        * (pl.col("contacts") - pl.col("coveredRecipients"))
                        / pl.col("contacts").clip(lower_bound=1)
                    ).alias("redundantContactPercent"),
                )
                .collect()
                .to_dicts()
            )
            if rows
            else []
        )
    return tables, matches


CAPTURABLE = {"village", "grove", "fort", "ruins"}
NONLEADER_FIELDS = (
    "nonleaderUnitTurns",
    "nonleaderWeaponHPDamageDealt",
    "nonleaderThornHPDamageDealt",
)
FIELDS = (
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
)
VALUE_DIMS = (
    "kind",
    "campaign",
    "diplomacy",
    "objective",
    "mission",
    "race",
    "policy",
    "opponentPolicy",
    "map",
    "family",
)


def cell_key(tile):
    """Hashable analysis key for current structured hexes and historical journals."""
    return f"{tile['col']},{tile['row']}" if isinstance(tile, dict) else tile


def battlefield(reports, value=True, leaders=True):
    counts = Accumulator(list(VALUE_DIMS), list(FIELDS))
    unit_counts = Accumulator(
        list(VALUE_DIMS[:-1]) + ["unit"], list(FIELDS) + list(NONLEADER_FIELDS)
    )
    leader_rows = []
    catalogs = None
    for report, result in journals(reports):
        kind = result.get("kind", "campaign" if "missions" in result else "arena")
        config = result["config"]
        if kind == "arena" and config["armies"]["player"] == config["armies"]["enemy"]:
            continue
        if value and report.get("combatTelemetryVersion") != 1:
            raise ValueError(
                "Battlefield value requires a fresh run with combat telemetry"
            )
        if leaders and report.get("leaderAuditVersion") != 1:
            raise ValueError(
                "Leader audit requires a fresh run with leader-risk telemetry"
            )
        profiles = report.get("unitProfiles") or {}
        if catalogs is not None and profiles != catalogs:
            raise ValueError(
                "Cannot pool battlefield value across different recorded catalogs"
            )
        catalogs = profiles
        chapters = [result] if kind == "arena" else result["missions"]
        for chapter in chapters:
            mission = chapter.get("id", "")
            initial = chapter["initial"]
            tiles = {t["id"]: t for t in initial["tiles"]}
            owners = {t: u.get("owner") for t, u in tiles.items()}
            factions = {f["id"]: f for f in initial.get("factions", [])}

            def faction_id(unit):
                return unit.get("faction") or unit["side"]

            def hostile(first, second):
                if first == second:
                    return False
                relation = factions.get(first, {}).get("relations", {}).get(second)
                if relation is None:
                    relation = factions.get(second, {}).get("relations", {}).get(first)
                return (
                    relation == "hostile"
                    if relation is not None
                    else {first, second} == {"player", "enemy"}
                )

            def replacement_cost(unit):
                pool = factions.get(faction_id(unit), {}).get("recruits") or []
                return next(
                    (
                        entry.get("cost", profiles[unit["type"]]["cost"])
                        for entry in pool
                        if entry["unit"] == unit["type"]
                    ),
                    profiles[unit["type"]]["cost"],
                )

            captors, investments = {}, {}
            objective_tiles = {side: set() for side in ("player", "enemy")}
            for phase in chapter["phases"]:
                for objective in phase.get("objectives", []):
                    if objective.get("kind") == "capture":
                        objective_tiles[phase["side"]].update(
                            cell_key(t) for t in objective["tiles"]
                        )

            def policy(side):
                blocks = config.get(side + "Blocks")
                return (
                    "-".join(blocks[k] for k in ("recruiter", "campaign", "planner"))
                    if blocks
                    else config[side]
                )

            def identity(unit, landscape):
                side = unit["side"]
                other = "enemy" if side == "player" else "player"
                race = (
                    config["armies"][side]
                    if kind == "arena"
                    else profiles[unit["type"]]["race"]
                )
                return (
                    kind,
                    config.get("campaign", ""),
                    config.get("diplomacy", ""),
                    config.get("objective", "authored"),
                    mission,
                    race,
                    policy(side),
                    policy(other),
                    landscape,
                    profiles[unit["type"]]["family"],
                )

            def add(unit, **stats):
                landscape = config["landscape"] if kind == "arena" else mission
                for label in (landscape, "all"):
                    key = identity(unit, label)
                    counts.add(dict(zip(VALUE_DIMS, key), **stats))
                    unit_stats = dict(stats)
                    nonleader = unit.get("role") != "leader"
                    for source, target in zip(
                        ("unitTurns", "weaponHPDamageDealt", "thornHPDamageDealt"),
                        NONLEADER_FIELDS,
                    ):
                        unit_stats[target] = stats.get(source, 0) if nonleader else 0
                    unit_counts.add(
                        dict(
                            zip(VALUE_DIMS[:-1], key[:-1]),
                            unit=unit["type"],
                            **unit_stats,
                        )
                    )

            for phase_index, phase in enumerate(chapter["phases"]):
                side = phase["side"]
                lookup = {
                    u["id"]: u
                    for u in phase["before"] + phase.get("losses", []) + phase["units"]
                }
                if value:
                    for unit in phase["units"]:
                        if unit["side"] == side:
                            add(unit, unitTurns=1)
                            tile = cell_key(unit["tile"])
                            if tiles[tile]["terrain"] in CAPTURABLE:
                                add(unit, capturableOccupationTurns=1)
                            if tile in objective_tiles[side]:
                                add(unit, objectiveOccupationTurns=1)
                    for action in phase.get("actions", []):
                        if action["kind"] == "recruit":
                            # Action.unit is the recruiting leader, not the new
                            # troop. New recruits cannot move on their birth turn.
                            previous_ids = {u["id"] for u in phase["before"]}
                            recruits = [
                                u
                                for u in lookup.values()
                                if u["id"] not in previous_ids
                                and u["side"] == side
                                and u["tile"] == action["tile"]
                                and u["type"] == action["type"]
                                and u["id"] not in investments
                            ]
                            if len(recruits) != 1:
                                raise ValueError(
                                    "Cannot identify recorded new recruit from its deployment tile/type"
                                )
                            investments[recruits[0]["id"]] = (
                                action["cost"]
                                if action.get("cost") is not None
                                else replacement_cost(recruits[0])
                            )
                        elif action["kind"] == "move":
                            unit = lookup[action["unit"]]
                            tile = cell_key(action["tile"])
                            if tiles[tile]["terrain"] in CAPTURABLE and (
                                owners[tile] is None
                                or hostile(faction_id(unit), owners[tile])
                            ):
                                owners[tile] = faction_id(unit)
                                captors[tile] = dict(unit)
                                add(
                                    unit,
                                    captures=1,
                                    objectiveCaptures=int(
                                        tile in objective_tiles[unit["side"]]
                                    ),
                                )
                    for loss in phase.get("losses", []):
                        add(
                            loss,
                            casualties=1,
                            recruitGoldLost=investments.pop(loss["id"], 0),
                            casualtyReplacementValue=replacement_cost(loss),
                        )
                    for combat in phase.get("combats", []):
                        mitigation = combat.get("mitigation")
                        if mitigation is None:
                            raise ValueError(
                                "Combat mitigation telemetry missing from combat record"
                            )
                        attacker, target = (
                            lookup[combat["unit"]],
                            lookup[combat["target"]],
                        )
                        for event, actor, defender, response in (
                            (mitigation["attack"], attacker, target, False),
                            (mitigation.get("retaliation"), target, attacker, True),
                        ):
                            if event is None:
                                continue
                            actual = (
                                event["effectiveDamage"]
                                if event.get("effectiveDamage") is not None
                                else min(
                                    event["afterTerrainDamage"], event["availableHp"]
                                )
                            )
                            add(actor, hpDamageDealt=actual, weaponHPDamageDealt=actual)
                            add(
                                defender,
                                hpDamageTaken=actual,
                                weaponHPDamageTaken=actual,
                                armourDamagePrevented=event["effectiveArmourPrevented"],
                                terrainDamagePrevented=event[
                                    "effectiveTerrainPrevented"
                                ],
                                rawArmourDelta=event["rawDamage"]
                                - event["afterArmourDamage"],
                                rawTerrainDelta=event["afterArmourDamage"]
                                - event["afterTerrainDamage"],
                                attacksTaken=int(not response),
                                retaliationsTaken=int(response),
                            )
                        thorns = combat.get("thorns") or {}
                        effective = thorns.get("effectiveDamage", 0)
                        if effective:
                            raw = thorns["damage"]
                            own = raw - thorns["auraDamage"]
                            add(
                                attacker,
                                hpDamageTaken=effective,
                                thornHPDamageTaken=effective,
                            )
                            if own:
                                reflected = effective * own / raw
                                add(
                                    target,
                                    hpDamageDealt=reflected,
                                    thornHPDamageDealt=reflected,
                                )
                            sources = thorns["auraSources"]
                            if thorns["auraDamage"] and not sources:
                                raise ValueError("Thorn aura event has no provider")
                            for source in sources:
                                reflected = (
                                    effective
                                    * thorns["auraDamage"]
                                    / raw
                                    / len(sources)
                                )
                                add(
                                    lookup[source],
                                    hpDamageDealt=reflected,
                                    thornHPDamageDealt=reflected,
                                )
                    observed_owners = {
                        cell_key(o["tile"]): o["owner"] for o in phase["ownership"]
                    }
                    for tile, captor in captors.items():
                        if (
                            observed_owners.get(tile) == faction_id(captor)
                            and captor["side"] == side
                        ):
                            add(
                                captor,
                                ownedTileTurns=1,
                                objectiveControlTurns=int(
                                    tile in objective_tiles[side]
                                ),
                            )
                    owners.update(observed_owners)
                if leaders:
                    # Death diagnostics retain both the death phase and the last
                    # own-phase decision, rather than blaming the killing hit.
                    for audit in phase.get("leaderAudit", []):
                        leader = lookup[audit["leader"]]
                        row = dict(
                            zip(
                                VALUE_DIMS[:-1],
                                identity(leader, config.get("landscape", mission))[:-1],
                            )
                        )
                        before, after = audit["armyBefore"], audit["armyAfter"]
                        foe = audit["opponentBefore"]
                        exposure = audit.get("exposure") or {}
                        row.update(
                            matchId=kind
                            + ":"
                            + json.dumps(config, sort_keys=True, separators=(",", ":")),
                            seed=config["seed"],
                            first=config.get("first", ""),
                            phase=phase_index,
                            round=phase["round"],
                            actingSide=side,
                            side=audit["side"],
                            leader=audit["leader"],
                            unit=leader["type"],
                            died=audit["after"] is None,
                            beforeHP=audit["before"]["hp"],
                            afterHP=(audit["after"] or {}).get("hp", 0),
                            beforeTile=audit["before"]["tile"],
                            afterTile=(audit["after"] or {}).get("tile"),
                            ownDecision=side == audit["side"],
                            armyUnitsBefore=before["units"],
                            armyUnitsAfter=after["units"],
                            armyHPValueBefore=before["hpValue"],
                            armyHPValueAfter=after["hpValue"],
                            opponentHPValueBefore=foe["hpValue"],
                            opponentHPValueAfter=audit["opponentAfter"]["hpValue"],
                            armyValueRatioBefore=(
                                before["hpValue"] / foe["hpValue"]
                                if foe["hpValue"]
                                else None
                            ),
                            armyRetainedFraction=(
                                after["hpValue"] / before["hpValue"]
                                if before["hpValue"]
                                else None
                            ),
                            armyHPBefore=before["hp"],
                            armyHPAfter=after["hp"],
                            **audit["recruitmentBefore"],
                            **exposure,
                        )
                        last_combat = next(
                            (
                                c
                                for c in reversed(phase.get("combats", []))
                                if audit["leader"] in (c["unit"], c["target"])
                            ),
                            {},
                        )
                        row["deathDuringOwnAttack"] = bool(
                            row["died"] and last_combat.get("unit") == audit["leader"]
                        )
                        # This describes a snapshot forecast, not avoidable death.
                        chosen = exposure.get("chosenTileDamage")
                        minimum = exposure.get("minDamage")
                        row["forecastLethalWithSafeAlternative"] = (
                            chosen >= audit["before"]["hp"]
                            and minimum < audit["before"]["hp"]
                            if chosen is not None and minimum is not None
                            else None
                        )
                        row["endForecastLethal"] = (
                            exposure["afterDamage"] >= row["afterHP"]
                            if exposure.get("afterDamage") is not None
                            and audit["after"] is not None
                            else None
                        )
                        leader_rows.append(row)
    tables = {}
    if value:
        tables["battlefield_value"] = counts.finish()
        unit_rows = unit_counts.finish()
        family_map = {k: v["family"] for k, v in (catalogs or {}).items()}
        tables["battlefield_unit_value"] = (
            (
                pl.from_dicts(unit_rows)
                .with_columns(pl.col("unit").replace(family_map).alias("family"))
                .to_dicts()
            )
            if unit_rows
            else []
        )
    if leaders:
        if not leader_rows:
            tables.update(leader_audit=[], leader_deaths=[])
        else:
            keys = ["matchId", "mission", "side", "leader"]
            df = (
                pl.from_dicts(leader_rows, infer_schema_length=None)
                .with_row_index("__order")
                .with_columns(
                    pl.max_horizontal("armyHPValueBefore", "armyHPValueAfter")
                    .cum_max()
                    .over(keys)
                    .alias("armyPeakHPValue")
                )
                .with_columns(
                    pl.when(pl.col("armyPeakHPValue") > 0)
                    .then(pl.col("armyHPValueAfter") / pl.col("armyPeakHPValue"))
                    .otherwise(None)
                    .alias("armyFractionOfPeakAfter")
                )
            )
            tables["leader_audit"] = df.drop("__order").to_dicts()
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
            for field in fields:
                if field not in df.columns:
                    df = df.with_columns(pl.lit(None).alias(field))
            # Join to the latest complete decision row, so null fields in that
            # decision remain null rather than inheriting an earlier value.
            decisions = df.filter("ownDecision").select(
                *keys,
                pl.col("__order").alias("__decision"),
                *[
                    pl.col(f).alias("lastDecision" + f[0].upper() + f[1:])
                    for f in fields
                ],
            )
            deaths = (
                df.filter("died")
                .join_asof(
                    decisions,
                    left_on="__order",
                    right_on="__decision",
                    by=keys,
                    strategy="backward",
                    check_sortedness=False,
                )
                .drop("__order", "__decision")
            )
            death_fields = (
                list(
                    dict.fromkeys(k for row in leader_rows if row["died"] for k in row)
                )
                + ["armyPeakHPValue", "armyFractionOfPeakAfter"]
                + ["lastDecision" + f[0].upper() + f[1:] for f in fields]
            )
            tables["leader_deaths"] = deaths.select(
                list(dict.fromkeys(death_fields))
            ).to_dicts()
    return tables
