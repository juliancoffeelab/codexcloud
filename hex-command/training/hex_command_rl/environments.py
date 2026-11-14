"""Author the fixed, non-player RL catalog; never generates terrain at runtime.

Run from the game root: python -m hex_command_rl.environments. Generated JSON
is checked in beside ordinary engine-owned maps. Simulation manifests select it.
"""

import json
from pathlib import Path
import random

DATA = Path("rust/data")
BIOMES = (
    ("meadow", 9, 7, ("plains", "plains", "forest", "hill")),
    ("woodland", 11, 9, ("forest", "forest", "plains", "hill")),
    ("ridge", 13, 7, ("hill", "hill", "forest", "mountain")),
    ("wetland", 11, 9, ("marsh", "marsh", "plains", "water")),
    ("river", 13, 9, ("plains", "forest", "hill", "plains")),
    ("islands", 15, 11, ("water", "water", "plains", "forest")),
    ("cavern", 11, 7, ("cave", "cave", "cave-wall", "cave-passage")),
    ("ruins", 15, 9, ("ruins", "plains", "forest", "hill")),
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
            if name == "river" and col == cols // 2:
                kind = "water"
            # Two guaranteed land routes keep every faction's army mobile.
            if row in (mid, mid - 2):
                kind = "ford" if name in ("river", "islands", "wetland") else "road"
            if col in (0, 1, 2, cols - 3, cols - 2, cols - 1):
                kind = "plains" if name != "cavern" else "cave"
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
    return dict(
        mapId=identifier,
        mapRevision=1,
        name=identifier,
        description=f"RL {name} fixture.",
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
    if task != "survive":
        events.append(
            dict(
                id="deadline",
                priority=-100,
                when=dict(turn=16 if task == "reach" else 30),
                actions=[dict(defeat="The deadline expired.")],
            )
        )
    return b, r, dict(requiredCompanions=required, events=events)


def main():
    manifest = json.loads((DATA / "manifest.json").read_text())
    arena = json.loads((DATA / "simulation/arena.json").read_text())
    campaign = json.loads((DATA / "simulation/campaign.json").read_text())
    races = ["human", "skeletons", "woodland", "coastal"]
    arenas, campaigns = [], {}
    for i, biome in enumerate(BIOMES):
        identifier = "rl-" + biome[0]
        folder = DATA / "game/scenario" / identifier
        b = board(identifier, biome, 1729 + i)
        write(folder / "map.json", b)
        write(
            folder / "roster.json", roster("human", "skeletons", b["cols"], b["rows"])
        )
        write(
            folder / "scenario.json",
            dict(
                human=False,
                map="map.json",
                roster="roster.json",
                characters=[
                    dict(id=s + "-leader", name=s) for s in ("player", "enemy")
                ],
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
                        forces=dict(kind="selected", budget=36 + i % 3 * 3),
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
            base = folder / "stages" / name
            b, r, scripts = stage(
                name,
                BIOMES[(i + j) % len(BIOMES)],
                kind,
                player,
                enemy,
                2718 + i * 11 + j,
            )
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
            stages.append(f"stages/{name}/stage.json")
        chars = [
            dict(id=c, name=c) for c in ("player-leader", "enemy-leader", "charge")
        ]
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
                characters=chars,
                ending=[],
                stages=stages,
            ),
        )
        path = str((folder / "campaign.json").relative_to(DATA))
        if path not in manifest["campaigns"]:
            manifest["campaigns"].append(path)
        campaign["campaigns"][identifier] = dict(diplomacy=["unrestricted"], routes={})
        campaigns[identifier] = [dict(objective="authored", diplomacy="unrestricted")]
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
