"""Presentation only; computations are owned by the dataframe modules."""

import csv
import json
from frames import group
from journal import label
from report_exports import remove_obsolete_csv

DISPLAY = {
    "human": "Humans",
    "skeletons": "Skeletons",
    "woodland": "Woodland",
    "coastal": "Merfolk",
    "open": "Plains",
    "wooded": "Forests",
    "crossings": "Crossing",
    "islands": "Islands",
    "wood-edge": "Wood Edge",
}
FACTIONS = ["human", "skeletons", "woodland", "coastal"]
MAPS = ["open", "wooded", "crossings", "islands", "wood-edge"]


def names(rows):
    return list(dict.fromkeys(k for r in rows for k in r))


def cell(k, v):
    if v is None:
        return "policy default" if k == "recruitLimit" else "—"
    if isinstance(v, str):
        text = (
            DISPLAY.get(v, v) if k in ["race", "opponentRace", "Faction", "map"] else v
        )
    elif isinstance(v, float):
        text = f"{v:.2f}"
    elif isinstance(v, (dict, list)):
        text = json.dumps(v, separators=(",", ":"))
    else:
        text = str(v)
    return text.replace("|", "\\|").replace("\n", " ")


def markdown(rows, headers, comparison=()):
    def value(row, field):
        current = cell(field, row.get(field))
        return (
            cell(field, row.get(field + "Before")) + " → " + current
            if field in comparison
            else current
        )

    return "\n".join(
        [
            "| "
            + " | ".join(
                k + " (baseline → current)" if k in comparison else k for k in headers
            )
            + " |",
            "| " + " | ".join("---" for _ in headers) + " |",
        ]
        + ["| " + " | ".join(value(r, k) for k in headers) + " |" for r in rows]
    )


def findings(tables, physical):
    rows = tables["matchups"]
    factions = sorted(
        {r["race"] for r in rows},
        key=lambda x: (FACTIONS.index(x) if x in FACTIONS else len(FACTIONS), x),
    )
    maps = sorted(
        {r["map"] for r in rows},
        key=lambda x: (MAPS.index(x) if x in MAPS else len(MAPS), x),
    )
    lines = [
        "# Combat findings",
        "",
        f"Coverage: {len(physical)} physical matches. Caps remain in all win-rate denominators. Participant views are not additional games.",
        "",
        "## Arena faction matchups",
        "",
    ]
    for p, q in sorted({(r["policy"], r["opponentPolicy"]) for r in rows}):
        for map_name in maps:
            selected = {
                (r["race"], r["opponentRace"]): r
                for r in rows
                if r["policy"] == p
                and r["opponentPolicy"] == q
                and r["map"] == map_name
            }
            if not selected:
                continue
            matrix = []
            for race in factions:
                row = {"Faction": race}
                for enemy in factions:
                    item = selected.get((race, enemy))
                    row[enemy] = (
                        "Mirror"
                        if race == enemy
                        else f"{item['winRate']:.1f}% ({item['caps']})"
                        if item
                        else "—"
                    )
                matrix.append(row)
            lines += [
                f"### {DISPLAY.get(map_name, map_name)}: {p} against {q}",
                "",
                markdown(matrix, ["Faction"] + factions),
                "",
            ]
    mirrors = []
    for o in physical:
        c = o["config"]
        if (
            o["kind"] == "arena"
            and c["armies"]["player"] == c["armies"]["enemy"]
            and label(c, "player") == label(c, "enemy")
        ):
            mirrors.append(
                dict(
                    map=c["landscape"],
                    race=c["armies"]["player"],
                    policy=label(c, "player"),
                    games=1,
                    firstWins=int(
                        o["winner"] is not None and o["winner"] == c.get("first")
                    ),
                    caps=int(o["winner"] is None),
                )
            )
    mirrors = group(mirrors, ["map", "race", "policy"], ["games", "firstWins", "caps"])
    for r in mirrors:
        r["firstWinRate"] = 100 * r["firstWins"] / r["games"]
    lines += [
        "## Matching-policy mirrors: first to act",
        "",
        "Each physical game is counted once, including caps.",
        "",
        markdown(mirrors, ["map", "race", "policy", "games", "firstWinRate", "caps"]),
        "",
        "## Campaign completion",
        "",
        markdown(
            tables["campaign_summary"],
            [
                "campaign",
                "player",
                "enemy",
                "diplomacy",
                "objective",
                "maxRounds",
                "recruitLimit",
                "attempts",
                "completed",
                "defeats",
                "caps",
                "unclassified",
            ],
        ),
        "",
        "## Campaign stages",
        "",
        "Stage denominators include only attempts reaching that stage. Failure observations and forecast exposure do not establish causes.",
        "",
        markdown(
            tables["campaign_stages"],
            [
                "campaign",
                "objective",
                "diplomacy",
                "player",
                "enemy",
                "stage",
                "map",
                "reached",
                "victories",
                "defeats",
                "caps",
                "idleCaps",
                "repeatedCaps",
                "zeroProgressDefeats",
                "actorAttempts",
                "actorArrivals",
                "actorDefeatsBeforeArrival",
                "defeatsWithSurvivors",
                "playerDamage",
                "enemyDamage",
            ],
        ),
        "",
        "## Recorded terminal reasons",
        "",
        markdown(
            tables["campaign_failures"],
            [
                "campaign",
                "objective",
                "diplomacy",
                "player",
                "enemy",
                "map",
                "category",
                "reason",
                "attempts",
                "exampleSeeds",
            ],
        ),
        "",
    ]
    if tables.get("unit_damage"):
        lines += [
            "## Per-unit damage",
            "",
            "Presence is living nonleader unit-turn exposure. Weapon HP damage includes retaliation and excludes overkill and thorns. Thorns are attributed separately. D/P is damage share divided by presence share; undefined rates remain unknown.",
            "",
            markdown(
                tables.get("unit_damage_changes") or tables["unit_damage"],
                [
                    "race",
                    "policy",
                    "opponentPolicy",
                    "map",
                    "name",
                    "unitKind",
                    "unitTurns",
                    "presenceSharePercent",
                    "damagePerUnitTurn",
                    "damagePresenceRatio",
                    "thornDamagePerUnitTurn",
                ],
                comparison=(
                    "unitTurns",
                    "presenceSharePercent",
                    "damagePerUnitTurn",
                    "damagePresenceRatio",
                    "thornDamagePerUnitTurn",
                )
                if tables.get("unit_damage_changes")
                else (),
            ),
            "",
        ]
    fields = [
        "campaign",
        "player",
        "enemy",
        "diplomacy",
        "objective",
        "maxRounds",
        "recruitLimit",
    ]
    bottlenecks = []
    for campaign in tables["campaign_summary"]:
        stages = [
            r
            for r in tables["campaign_stages"]
            if all(r[k] == campaign[k] for k in fields)
        ]
        if stages:
            bottlenecks.append(max(stages, key=lambda r: r["defeats"] + r["caps"]))
    if bottlenecks:
        lines += [
            "## Largest recorded campaign bottlenecks",
            "",
            markdown(
                bottlenecks,
                [
                    "campaign",
                    "objective",
                    "diplomacy",
                    "player",
                    "enemy",
                    "maxRounds",
                    "recruitLimit",
                    "stage",
                    "map",
                    "reached",
                    "victories",
                    "defeats",
                    "caps",
                ],
            ),
            "",
        ]
    runs = tables["campaign_runs"]
    if any(r["recordedPlanning"] for r in runs):
        facts = [
            dict(reason=k, observations=n)
            for r in runs
            for k, n in r["unmetOrders"].items()
        ] + [
            dict(
                reason="protected retaliation exchanges",
                observations=r["protectedRetaliationExchanges"],
            )
            for r in runs
        ]
        lines += [
            "## Recorded planner diagnostics",
            "",
            "Counts are observations per phase. An unmet forecast does not establish an avoidable defeat.",
            "",
            markdown(
                group(facts, ["reason"], ["observations"]), ["reason", "observations"]
            ),
            "",
        ]
    stalled = [r for r in runs if r["idleTail"] >= 5 and r["stalledActors"]]
    if stalled:
        lines += [
            "## Recorded idle-cap actor positions",
            "",
            markdown(
                stalled,
                [
                    "campaign",
                    "objective",
                    "diplomacy",
                    "player",
                    "enemy",
                    "map",
                    "seed",
                    "stalledActors",
                ],
            ),
            "",
        ]
    sections = [
        (
            "army_usage",
            "Battlefield presence and recruitment flow",
            [
                "race",
                "policy",
                "opponentPolicy",
                "map",
                "family",
                "sharePercent",
                "unitTurns",
                "purchaseSharePercent",
                "recruited",
                "lost",
                "attackSharePercent",
                "damageSharePercent",
            ],
            lambda r: r["map"] != "all",
            "Purchases measure recruitment flow; presence and contribution use recorded nonleader exposure.",
        ),
        (
            "support_effects",
            "Actual support effects",
            [
                "race",
                "policy",
                "opponentPolicy",
                "map",
                "ability",
                "providerTurns",
                "adjacentAlliesPerTurn",
                "healedHP",
                "healedHPPerTurn",
                "reflectedDamage",
                "effectiveReflectedDamage",
                "overlapPercent",
            ],
            lambda r: r["unit"] == "all" and r["map"] != "all",
            "Provider turns include zero-benefit turns. Tied providers split credit; effective reflection excludes overkill.",
        ),
        (
            "battlefield_value",
            "Protection, objectives and casualty value",
            [
                "race",
                "policy",
                "opponentPolicy",
                "family",
                "unitTurns",
                "hpDamageDealt",
                "hpDamageTaken",
                "armourDamagePrevented",
                "terrainDamagePrevented",
                "captures",
                "objectiveOccupationTurns",
                "casualties",
                "recruitGoldLost",
                "casualtyReplacementValue",
            ],
            lambda r: r["kind"] == "arena" and r["map"] == "all",
            "Protection is signed and capped at available HP. Captors retain control credit after death. Replacement value includes free starters and promotions.",
        ),
        (
            "leader_deaths",
            "Leader death evidence",
            [
                "campaign",
                "mission",
                "map",
                "policy",
                "seed",
                "round",
                "armyUnitsAfter",
                "armyHPValueAfter",
                "lastDecisionArmyValueRatioBefore",
                "lastDecisionForecastLethalWithSafeAlternative",
                "deathDuringOwnAttack",
            ],
            lambda r: True,
            "Death boundaries link to the same leader's last own-phase decision. A forecast-safe tile is not proof of an avoidable death.",
        ),
    ]
    for name, title, fields, predicate, note in sections:
        rows = [r for r in tables.get(name, []) if predicate(r)]
        if rows:
            lines += ["## " + title, "", note, "", markdown(rows, fields), ""]
    if not tables.get("battlefield_presence"):
        lines += [
            "Field composition is unavailable. Purchases measure recruitment flow and do not establish composition or survival.",
            "",
        ]
    return "\n".join(lines)


def write(output, current, previous, tables, seconds):
    remove_obsolete_csv(output, tables)
    for name, rows in tables.items():
        with (output / (name + ".csv")).open("w", newline="") as f:
            writer = csv.DictWriter(f, fieldnames=names(rows))
            writer.writeheader()
            writer.writerows(
                {
                    k: json.dumps(v, separators=(",", ":"))
                    if isinstance(v, (dict, list))
                    else v
                    for k, v in r.items()
                }
                for r in rows
            )
    rows = tables["matchups"]
    factions = sorted(
        {r["race"] for r in rows},
        key=lambda x: (FACTIONS.index(x) if x in FACTIONS else len(FACTIONS), x),
    )
    metadata = dict(
        sources=current["sources"],
        baselineSources=previous["sources"],
        physicalMatches=len(current["outcomes"]),
        arenaMatches=sum(o["kind"] == "arena" for o in current["outcomes"]),
        campaignAttempts=len(tables["campaigns"]),
        factions=factions,
        maps=sorted({r["map"] for r in rows + tables["campaign_stages"]}),
        policies=sorted(
            {r["policy"] for r in rows}
            | {r["player"] for r in tables["campaign_summary"]}
        ),
        participantRows=sum(r["games"] for r in rows),
    )
    (output / "metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    (output / "tables.json").write_text(
        json.dumps(tables, separators=(",", ":")) + "\n"
    )
    text = findings(tables, current["outcomes"])
    (output / "findings.md").write_text(text)
    (output / "summary.md").write_text(
        text
        + "\n# Complete combat data\n\n"
        + "\n\n".join(
            "## " + name + "\n\n" + markdown(rows, names(rows))
            for name, rows in tables.items()
            if name
            in [
                "matchups",
                "unit_events",
                "purchase_shares",
                "race_equal_policy",
                "race_map_policy",
            ]
        )
        + "\n\n## Complete exports\n\n"
        + "\n".join(f"- [{name}.csv]({name}.csv)" for name in tables)
        + "\n"
    )
    (output / "timings.json").write_text(
        json.dumps(
            dict(reportSeconds=seconds, physicalMatches=len(current["outcomes"])),
            indent=2,
        )
        + "\n"
    )
