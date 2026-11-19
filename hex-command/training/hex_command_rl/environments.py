"""Author the fixed, non-player RL catalog; never generates terrain at runtime.

Run from the game root: python -m hex_command_rl.environments. Generated JSON
is checked in beside ordinary engine-owned maps. Simulation manifests select it.
"""

import json
from pathlib import Path
import random

DATA = Path("rust/data")
CORE_BIOMES = (
    ("meadow", 9, 7, ("plains", "plains", "forest", "hill")),
    ("woodland", 11, 9, ("forest", "forest", "plains", "hill")),
    ("ridge", 13, 7, ("hill", "hill", "forest", "mountain")),
    ("wetland", 11, 9, ("marsh", "marsh", "plains", "water")),
    ("river", 13, 9, ("plains", "forest", "hill", "plains")),
    ("islands", 15, 11, ("water", "water", "plains", "forest")),
    ("cavern", 11, 7, ("cave-passage",)),
    ("ruins", 15, 9, ("ruins", "plains", "forest", "hill")),
)
BIOMES = CORE_BIOMES + (
    ("broadfield", 19, 11, ("plains", "forest", "hill", "plains")),
    ("floodplain", 21, 13, ("marsh", "water", "forest", "plains")),
)


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def hex_(col, row):
    return dict(col=col, row=row)


def board(identifier, biome, seed):
    name, cols, rows, terrain = biome
    rng = random.Random(seed)
    mid = rows // 2
    tiles = []
    for row in range(rows):
        for col in range(cols):
            kind = rng.choice(terrain)
            if name == "cavern":
                # Two chambers and a central passage, with two separated links.
                # Mine entrances are landmarks, never underground ground cover.
                chamber = 1 <= row <= 5 and (1 <= col <= 3 or 7 <= col <= 9)
                passage = col == 5 and 1 <= row <= 5
                link = row in (2, 4) and 3 <= col <= 7
                kind = "cave-passage" if chamber or passage or link else "cave-wall"
            if name == "river" and col == cols // 2:
                kind = "water"
            # Two guaranteed land routes keep every faction's army mobile.
            if name != "cavern" and row in (mid, mid - 2):
                kind = (
                    "ford"
                    if name in ("river", "islands", "wetland", "floodplain")
                    else "road"
                )
            if name != "cavern" and col in (0, 1, 2, cols - 3, cols - 2, cols - 1):
                kind = "plains"
            if row == mid and col in (1, cols - 2):
                kind = "keep"
            if (col, row) in (
                (cols // 2, mid),
                (cols // 2 - 2, mid - 2),
                (cols // 2 + 2, mid + 2),
            ):
                kind = "village"
            tile = dict(col=col, row=row, terrain=kind, owner=None)
            if kind == "keep":
                tile["owner"] = "player" if col == 1 else "enemy"
            if kind in ("road", "ford"):
                tile["connections"] = ["e", "w"]
            elif kind in ("village", "cave-passage"):
                tile["connections"] = []
            tiles.append(tile)
    if name == "cavern":
        by_cell = {(t["col"], t["row"]): t for t in tiles}
        for tile in tiles:
            tile["floor"] = "cave-stone"
            if tile["terrain"] != "cave-passage":
                continue
            col, row = tile["col"], tile["row"]
            shift = row % 2
            neighbors = (
                ("e", col + 1, row),
                ("se", col + shift, row + 1),
                ("sw", col + shift - 1, row + 1),
                ("w", col - 1, row),
                ("nw", col + shift - 1, row - 1),
                ("ne", col + shift, row - 1),
            )
            tile["connections"] = [
                direction
                for direction, x, y in neighbors
                if (x, y) in by_cell and by_cell[x, y]["terrain"] != "cave-wall"
            ]
    return dict(
        mapId=identifier,
        mapRevision=2 if name == "cavern" else 1,
        name=identifier,
        description=(
            "Two underground chambers and a central passage, joined at two chokepoints."
            if name == "cavern"
            else f"RL {name} fixture."
        ),
        cols=cols,
        rows=rows,
        tiles=tiles,
    )


def roster(player, enemy, cols, rows, gold=18, escort=None, fog=False):
    groups, factions = [], []
    rosters = {
        r["id"]: r for r in json.loads((DATA / "simulation/rosters.json").read_text())
    }
    for side, race, x, phase in (
        ("player", player, 1, "player"),
        ("enemy", enemy, cols - 2, "ai"),
    ):
        spec = rosters[race]
        units = [
            dict(
                type=spec["leader"],
                tile=hex_(x, rows // 2),
                role="leader",
                character=side + "-leader",
            )
        ]
        for i, unit in enumerate(spec["pool"][:3]):
            units.append(
                dict(
                    type=unit,
                    tile=hex_(x + (1 if side == "player" else -1), rows // 2 - 1 + i),
                )
            )
        if side == "player" and escort:
            units.append(
                dict(
                    type="captives" if escort == "captive" else spec["pool"][0],
                    tile=hex_(x, rows // 2 + 1),
                    role=escort,
                    character="charge",
                )
            )
        factions.append(
            dict(
                id=side,
                phase=phase,
                visionGroup=side,
                controller=dict(kind="human" if side == "player" else "ai"),
                palette=side,
                relations={"enemy" if side == "player" else "player": "hostile"},
                gold=gold,
                recruits=[dict(unit=u) for u in spec["pool"]],
                units=units,
            )
        )
        groups.append(
            dict(
                id=side,
                sight=dict(
                    range=5 if fog else "unlimited", penetrates=[] if fog else ["fog"]
                ),
                explored=[],
            )
        )
    return dict(visionGroups=groups, factions=factions)


def objective(identifier, when, intent, label, actions=None):
    return dict(
        id=identifier,
        when=when,
        objective=dict(
            faction="player",
            label=label,
            visibleWhen=dict(always=True),
            intent=intent,
            playerMarker=None,
        ),
        actions=actions or [dict(victory="Task completed.")],
    )


def stage(identifier, biome, task, player, enemy, seed):
    b = board(identifier, biome, seed)
    cols, rows = b["cols"], b["rows"]
    destination = dict(hex=hex_(cols - 4, rows // 2 - 2))
    escorted = task in ("escort", "courier")
    r = roster(
        player,
        enemy,
        cols,
        rows,
        escort="captive"
        if task == "escort"
        else "courier"
        if task == "courier"
        else None,
        fog=task == "fog",
    )
    events = []
    required = []
    if escorted:
        required = ["charge"]
        selection = dict(tile=destination, character="charge", through=True)
        events.append(
            objective(
                "arrival",
                dict(stand=selection),
                dict(escort=selection),
                "Bring the charge to the exit",
            )
        )
    elif task == "reach":
        selection = dict(tile=destination, character="player-leader", through=True)
        events.append(
            objective(
                "arrival",
                dict(stand=selection),
                dict(goTo=selection),
                "Reach the exit before turn 16",
            )
        )
    elif task == "hold":
        selection = dict(
            tile=dict(hex=hex_(cols // 2, rows // 2)), side="player", turns=3
        )
        events.append(
            objective(
                "hold",
                dict(held=selection),
                dict(hold=selection),
                "Hold the central village for three turns",
            )
        )
    elif task == "capture":
        sites = [
            dict(hex=hex_(cols // 2 - 2, rows // 2 - 2)),
            dict(hex=hex_(cols // 2 + 2, rows // 2 + 2)),
        ]
        events.append(
            objective(
                "capture",
                dict(all=[dict(own=dict(tile=t, faction="player")) for t in sites]),
                dict(capture=sites),
                "Control both outer villages",
            )
        )
    elif task == "survive":
        events.append(
            dict(
                id="relief",
                when=dict(turn=10),
                actions=[dict(victory="Relief arrived.")],
            )
        )
        events.append(
            objective(
                "stay",
                dict(always=False),
                dict(
                    goTo=dict(
                        tile=dict(hex=hex_(2, rows // 2 - 2)), character="player-leader"
                    )
                ),
                "Survive until turn 10",
                [dict(log="Defend until relief.")],
            )
        )
    else:
        events.append(
            objective(
                "battle",
                dict(noUnits=dict(faction="enemy", role="leader")),
                dict(defeat=dict(faction="enemy", role="leader")),
                "Defeat the opposing leader",
            )
        )
    if task == "protect":
        protected = r["factions"][0]["units"][2]
        protected["character"] = "charge"
        required = ["charge"]
    if task == "reinforcements":
        events.append(
            dict(
                id="ambush",
                when=dict(proximity=dict(tile=destination, radius=2, faction="player")),
                actions=[
                    dict(
                        spawn=dict(
                            faction="enemy",
                            area=[
                                dict(hex=hex_(cols - 3, rows // 2 - 2)),
                                dict(hex=hex_(cols - 3, rows // 2 + 2)),
                            ],
                            units=[dict(type="skeleton"), dict(type="bonearcher")],
                        )
                    )
                ],
            )
        )
    if task == "fog":
        for tile in b["tiles"]:
            tile["perception"] = dict(
                remember=True,
                concealment=[dict(kind="fog", revealWithin=2, requires={})],
            )
    if task not in ("survive", "protect", "reinforcements"):
        events.append(
            dict(
                id="deadline",
                priority=-100,
                when=dict(turn=16 if task == "reach" else 30),
                actions=[dict(defeat="The deadline expired.")],
            )
        )
    return b, r, dict(requiredCompanions=required, events=events)


def compound_stage(identifier, biome, player, enemy, seed, chapter, mirrored):
    """Two linked tasks per chapter, optional supplies, and persistent survivors."""
    b = board(identifier, biome, seed)
    cols, rows = b["cols"], b["rows"]
    mid = rows // 2
    r = roster(player, enemy, cols, rows, gold=18, escort="captive")
    r["factions"][0]["gold"] = 36
    center = dict(hex=hex_(cols // 2, mid))
    exit_ = dict(
        tile=dict(hex=hex_(cols - 4, mid - 2)), character="charge", through=True
    )
    required = ["charge"]
    if chapter == 0:
        gate = objective(
            "route-open",
            dict(own=dict(tile=center, faction="player")),
            dict(capture=[center]),
            "Secure the crossing",
            [dict(setFlag="route-open")],
        )
        gate["objective"]["satisfiedWhen"] = dict(flag="route-open")
        arrival = objective(
            "arrival",
            dict(all=[dict(flag="route-open"), dict(stand=exit_)]),
            dict(escort=exit_),
            "Escort the company beyond the crossing",
        )
        arrival["objective"]["visibleWhen"] = dict(flag="route-open")
        pool = next(
            v["pool"]
            for v in json.loads((DATA / "simulation/rosters.json").read_text())
            if v["id"] == enemy
        )
        reinforcement = dict(
            id="reserve",
            when=dict(all=[dict(flag="route-open"), dict(turnAtLeast=9)]),
            actions=[
                dict(
                    spawn=dict(
                        faction="enemy",
                        area=[
                            dict(hex=hex_(cols - 3, mid - 2)),
                            dict(hex=hex_(cols - 3, mid + 2)),
                        ],
                        units=[dict(type=pool[0]), dict(type=pool[2])],
                    )
                )
            ],
        )
        events = [gate, arrival, reinforcement]
    else:
        r["factions"][0]["units"][2]["character"] = "guard"
        required.append("guard")
        selection = dict(tile=center, side="player", turns=3)
        hold = objective(
            "secure",
            dict(held=selection),
            dict(hold=selection),
            "Hold the crossing for three turns",
            [dict(setFlag="secured")],
        )
        hold["objective"]["satisfiedWhen"] = dict(flag="secured")
        boss = dict(faction="enemy", role="leader")
        battle = objective(
            "finish",
            dict(all=[dict(flag="secured"), dict(noUnits=boss)]),
            dict(defeat=boss),
            "Defeat the opposing leader after securing the crossing",
        )
        battle["objective"]["visibleWhen"] = dict(flag="secured")
        events = [hold, battle]
    site = dict(
        tile=dict(hex=hex_(cols // 2 - 2, mid - 2)), faction="player", through=True
    )
    supplies = objective(
        "supplies",
        dict(stand=site),
        dict(goTo=site),
        "Collect optional supplies",
        [dict(gold=dict(faction="player", amount=12))],
    )
    supplies["objective"]["optional"] = True
    events.append(supplies)
    # Include open-ended campaigns as well as authored deadlines, so the
    # campaign-cap penalty is actually encountered during training.
    if chapter == 0 or not mirrored:
        events.append(
            dict(
                id="deadline",
                priority=-100,
                when=dict(turn=38 if chapter == 0 else 34),
                actions=[dict(defeat="The deadline expired.")],
            )
        )
    scripts = dict(requiredCompanions=required, events=events)

    # Reflect offset hex coordinates, preserving adjacency. Staggered rows need
    # a one-column shift; the spare bounding column contains no extra tiles.
    def transform(value):
        if isinstance(value, list):
            return [transform(v) for v in value]
        if not isinstance(value, dict):
            return value
        result = {k: transform(v) for k, v in value.items()}
        if "col" in value and "row" in value:
            if mirrored:
                result["col"] = cols - value["col"] - value["row"] % 2
            if chapter:
                result["row"] = rows - 1 - value["row"]
        return result

    b, r, scripts = map(transform, (b, r, scripts))
    if mirrored:
        b["cols"] += 1
    return b, r, scripts


def publish_stage(folder, name, b, r, scripts):
    base = folder / "stages" / name
    write(base / "map.json", b)
    write(base / "roster.json", r)
    write(base / "scripts.json", scripts)
    write(
        base / "story.json",
        dict(
            subtitle="RL tactical fixture",
            briefing="Complete the marked tactical task and preserve required characters.",
            hints=[],
            intro=[],
            victory=[],
        ),
    )
    write(
        base / "stage.json",
        dict(
            id=name,
            map="map.json",
            roster="roster.json",
            story="story.json",
            scripts="scripts.json",
        ),
    )
    return f"stages/{name}/stage.json"


def publish_campaign(
    identifier, player, stages, characters, manifest, campaign, training
):
    folder = DATA / "game/campaigns" / identifier
    leader = next(
        v["leader"]
        for v in json.loads((DATA / "simulation/rosters.json").read_text())
        if v["id"] == player
    )
    write(
        folder / "campaign.json",
        dict(
            id=identifier,
            name=identifier,
            tagline="RL tactical fixtures",
            description="Training only.",
            shortDescription="Training only.",
            race="human"
            if player == "woodland"
            else "merfolk"
            if player == "coastal"
            else player,
            leader=leader,
            human=False,
            characters=[dict(id=c, name=c) for c in characters],
            ending=[],
            stages=stages,
        ),
    )
    path = str((folder / "campaign.json").relative_to(DATA))
    if path not in manifest["campaigns"]:
        manifest["campaigns"].append(path)
    campaign["campaigns"][identifier] = dict(diplomacy=["unrestricted"], routes={})
    training[identifier] = [dict(objective="authored", diplomacy="unrestricted")]


def main():
    manifest = json.loads((DATA / "manifest.json").read_text())
    arena = json.loads((DATA / "simulation/arena.json").read_text())
    campaign = json.loads((DATA / "simulation/campaign.json").read_text())
    races = ["human", "skeletons", "woodland", "coastal"]
    existing = json.loads((DATA / "simulation/rl.json").read_text())
    arenas, campaigns = [], {}
    for i, biome in enumerate(BIOMES):
        identifier = "rl-" + biome[0]
        folder = DATA / "game/scenario" / identifier
        b = board(identifier, biome, 1729 + i)
        write(folder / "map.json", b)
        write(
            folder / "scenario.json",
            dict(
                tags=["simulation"],
                fixedArmies=False,
                map="map.json",
                roster="../rl-arena-setup.json",
                characters=[],
            ),
        )
        path = str((folder / "scenario.json").relative_to(DATA))
        if path not in manifest["scenarios"]:
            manifest["scenarios"].append(path)
        arena["scenarios"] = [a for a in arena["scenarios"] if a["id"] != identifier]
        arena["scenarios"].append(
            dict(
                id=identifier,
                variants=[
                    dict(
                        map=identifier,
                        forces=dict(
                            kind="selected",
                            budget=54 if biome[1] >= 19 else 36 + i % 3 * 3,
                        ),
                    )
                ],
                territory=[hex_(b["cols"] // 2, b["rows"] // 2)],
                holdTurns=3 + i % 3,
            )
        )
        arenas.append(
            dict(
                id=identifier,
                pairs=[dict(player=p, enemy=e) for p in races for e in races if p != e],
            )
        )
    tasks = (
        "escort",
        "courier",
        "protect",
        "reach",
        "hold",
        "capture",
        "survive",
        "reinforcements",
        "fog",
        "relay",
    )
    for i, task in enumerate(tasks):
        identifier = "rl-" + task
        folder = DATA / "game/campaigns" / identifier
        player, enemy = races[i % 4], races[(i + 1) % 4]
        stages = []
        for j, kind in enumerate(("capture", "escort") if task == "relay" else (task,)):
            name = f"{identifier}-{j + 1}"
            b, r, scripts = stage(
                name,
                CORE_BIOMES[(i + j) % len(CORE_BIOMES)],
                kind,
                player,
                enemy,
                2718 + i * 11 + j,
            )
            stages.append(publish_stage(folder, name, b, r, scripts))
        publish_campaign(
            identifier,
            player,
            stages,
            ("player-leader", "enemy-leader", "charge"),
            manifest,
            campaign,
            campaigns,
        )
    for i, player in enumerate(races):
        identifier = "rl-company-" + player
        folder = DATA / "game/campaigns" / identifier
        stages = []
        for j in range(2):
            name = f"{identifier}-{j + 1}"
            values = compound_stage(
                name,
                BIOMES[8 + (i + j) % 2],
                player,
                races[(i + j + 1) % 4],
                5003 + i * 31 + j,
                j,
                i % 2 == 1,
            )
            stages.append(publish_stage(folder, name, *values))
        publish_campaign(
            identifier,
            player,
            stages,
            ("player-leader", "enemy-leader", "charge", "guard"),
            manifest,
            campaign,
            campaigns,
        )
    generated = {a["id"] for a in arenas}
    arenas.extend(a for a in existing["arenas"] if a["id"] not in generated)
    campaigns = existing["campaigns"] | campaigns
    write(DATA / "manifest.json", manifest)
    write(DATA / "simulation/arena.json", arena)
    write(DATA / "simulation/campaign.json", campaign)
    write(
        DATA / "simulation/rl.json",
        dict(
            arenas=arenas,
            campaigns=campaigns,
            opponents=["classic", "predictor"],
            recruiters=["situational", "predictor"],
            objectives=["deathmatch", "territory"],
        ),
    )


if __name__ == "__main__":
    main()
