"""Explicit PyArrow schema for the existing TS and Rust JSONL match journals.

The schema describes recorded values, including derived hashes and timing
metadata. It performs no matching or comparison. Optional JSON fields become
nullable Arrow fields. The current authored hold-counter key is 5,4.
"""
import pyarrow as pa

STRING = pa.string()
INTEGER = pa.int64()
BOOLEAN = pa.bool_()
NUMBER = pa.float64()
STRINGS = pa.list_(STRING)

UNIT = pa.struct([
    ("id", STRING), ("type", STRING), ("side", STRING), ("tile", STRING),
    ("hp", INTEGER), ("moved", BOOLEAN), ("attacked", BOOLEAN),
    ("experience", INTEGER), ("name", STRING), ("role", STRING),
    ("retreating", BOOLEAN), ("rebuilding", BOOLEAN),
])
VETERAN = pa.struct([
    ("type", STRING), ("experience", INTEGER), ("name", STRING), ("role", STRING),
])
DIALOGUE = pa.struct([
    ("speaker", STRING), ("portrait", STRING), ("text", STRING),
])
COUNTER = pa.struct([
    ("heldTurns", INTEGER), ("lastCountedRound", INTEGER),
])
REFUGE = pa.struct([
    ("heldTurns", INTEGER), ("lastCountedRound", INTEGER), ("escalated", BOOLEAN),
])
CAMPAIGN = pa.struct([
    ("id", STRING), ("mission", INTEGER), ("recall", STRINGS),
    ("veterans", pa.list_(VETERAN)), ("storyVeterans", pa.list_(VETERAN)),
    ("fallenCompanions", STRINGS), ("carryGold", INTEGER), ("marks", STRINGS),
    ("dialogue", pa.list_(DIALOGUE)), ("dialogueIndex", INTEGER),
    ("outcome", STRING), ("completed", BOOLEAN), ("refuge", REFUGE),
    ("vigil", COUNTER), ("holds", pa.struct([("5,4", COUNTER)])),
    ("choices", STRINGS),
])
TILE = pa.struct([
    ("id", STRING), ("col", INTEGER), ("row", INTEGER),
    ("terrain", STRING), ("owner", STRING),
])
GOLD = pa.struct([("player", INTEGER), ("enemy", INTEGER)])
LAST_COMBAT = pa.struct([
    ("attackerId", STRING), ("attackId", STRING), ("sound", STRING),
])
STATE = pa.struct([
    ("mapId", STRING), ("mapRevision", INTEGER), ("cols", INTEGER),
    ("rows", INTEGER), ("round", INTEGER), ("turn", STRING), ("gold", GOLD),
    ("winner", STRING), ("nextId", INTEGER), ("combatRng", INTEGER), ("log", STRINGS),
    ("tiles", pa.list_(TILE)), ("units", pa.list_(UNIT)),
    ("campaign", CAMPAIGN), ("lastCombat", LAST_COMBAT),
])
PROGRESS = pa.struct([
    ("label", STRING), ("current", INTEGER), ("total", INTEGER),
    ("done", BOOLEAN), ("tiles", STRINGS),
])
OBJECTIVE = pa.struct([
    ("id", STRING), ("tiles", STRINGS), ("weight", NUMBER), ("kind", STRING),
    ("actor", STRING), ("family", STRING), ("hold", BOOLEAN),
])
ACTION = pa.struct([
    ("kind", STRING), ("unit", STRING), ("tile", STRING),
    ("target", STRING), ("weapon", STRING), ("type", STRING),
])
DAMAGE_MITIGATION = pa.struct([
    ("availableHp", INTEGER), ("effectiveDamage", INTEGER),
    ("rawDamage", INTEGER), ("afterArmourDamage", INTEGER), ("afterTerrainDamage", INTEGER),
    ("effectiveArmourPrevented", INTEGER), ("effectiveTerrainPrevented", INTEGER),
])
COMBAT = pa.struct([
    ("unit", STRING), ("target", STRING), ("weapon", STRING), ("from", STRING),
    ("targetTile", STRING), ("damage", INTEGER), ("retaliation", INTEGER),
    ("count", INTEGER), ("damagePerHit", INTEGER),
    ("rawRolls", pa.list_(INTEGER)), ("rolls", pa.list_(INTEGER)),
    ("responseRawRolls", pa.list_(INTEGER)), ("responseRolls", pa.list_(INTEGER)),
    ("retaliationCount", INTEGER), ("retaliationPerHit", INTEGER),
    ("mediationAtRisk", BOOLEAN),
    ("thorns", pa.struct([("damage",INTEGER),("auraDamage",INTEGER),("effectiveDamage",INTEGER),("auraSources",STRINGS)])),
    ("mitigation", pa.struct([("attack", DAMAGE_MITIGATION), ("retaliation", DAMAGE_MITIGATION)])),
])
OWNERSHIP = pa.struct([("tile", STRING), ("owner", STRING)])
ARMY_STRENGTH = pa.struct([
    ("units", INTEGER), ("cost", INTEGER), ("hp", INTEGER),
    ("maxHp", INTEGER), ("hpValue", pa.float64()),
])
LEADER_STATE = pa.struct([("tile", STRING), ("hp", INTEGER), ("maxHp", INTEGER)])
LEADER_AUDIT = pa.struct([
    ("leader", STRING), ("side", STRING), ("before", LEADER_STATE), ("after", LEADER_STATE),
    ("armyBefore", ARMY_STRENGTH), ("armyAfter", ARMY_STRENGTH),
    ("opponentBefore", ARMY_STRENGTH), ("opponentAfter", ARMY_STRENGTH),
    ("recruitmentBefore", pa.struct([
        ("gold", INTEGER), ("onOwnedKeep", BOOLEAN),
        ("legalOptions", INTEGER), ("affordableOptions", INTEGER),
    ])),
    ("exposure", pa.struct([
        ("beforeDamage", INTEGER), ("chosenTileDamage", INTEGER),
        ("minDamage", INTEGER), ("minTile", STRING),
        ("reachableTiles", INTEGER), ("safeTiles", INTEGER),
        ("chosenTileReachable", BOOLEAN), ("afterDamage", INTEGER),
    ])),
])
PHASE = pa.struct([
    ("leaderAudit", pa.list_(LEADER_AUDIT)),
    ("side", STRING), ("round", INTEGER), ("actions", pa.list_(ACTION)),
    ("losses", pa.list_(UNIT)), ("combats", pa.list_(COMBAT)),
    ("healing", pa.list_(pa.struct([("target",STRING),("providers",STRINGS),("amount",INTEGER)]))),
    ("living", GOLD), ("health", GOLD), ("gold", GOLD),
    ("before", pa.list_(UNIT)), ("units", pa.list_(UNIT)),
    ("ownership", pa.list_(OWNERSHIP)), ("goals", pa.list_(PROGRESS)),
    ("objectives", pa.list_(OBJECTIVE)), ("campaign", CAMPAIGN),
])
EVENT = pa.struct([("side", STRING), ("type", STRING)])
SCORE = pa.struct([
    ("controlled", INTEGER), ("held", INTEGER), ("optionalVisits", INTEGER),
])
BLOCKS = pa.struct([(name, STRING) for name in ("recruiter", "campaign", "planner")])
CONFIG = pa.struct([
    ("seed", INTEGER),
    ("armies", pa.struct([("player", STRING), ("enemy", STRING)])),
    ("landscape", STRING), ("player", STRING), ("enemy", STRING),
    ("first", STRING), ("maxRounds", INTEGER), ("recruitLimit", INTEGER),
    ("diplomacy", STRING), ("campaign", STRING),
    ("playerBlocks", BLOCKS), ("enemyBlocks", BLOCKS),
])
DETAIL_REFERENCE = pa.struct([
    ("file", STRING), ("offset", INTEGER), ("bytes", INTEGER), ("sha256", STRING),
])
MISSION = pa.struct([
    ("id", STRING), ("winner", STRING), ("round", INTEGER),
    ("goals", pa.list_(PROGRESS)), ("phases", pa.list_(PHASE)),
    ("initial", STATE), ("final", STATE), ("casualties", pa.list_(UNIT)),
    ("recruits", pa.list_(EVENT)), ("promotions", pa.list_(EVENT)),
    ("stateHash", STRING), ("elapsedMs", NUMBER), ("elapsedSeconds", NUMBER),
    ("detail", DETAIL_REFERENCE), ("escalated", BOOLEAN),
])
SIMULATION_DUMP_SCHEMA = pa.schema([
    ("kind", STRING), ("config", CONFIG), ("winner", STRING), ("reason", STRING),
    ("rounds", INTEGER), ("phases", pa.list_(PHASE)),
    ("initial", STATE), ("final", STATE),
    ("objectives", pa.struct([("player", SCORE), ("enemy", SCORE)])),
    ("casualties", pa.list_(UNIT)), ("recruits", pa.list_(EVENT)),
    ("promotions", pa.list_(EVENT)), ("completed", BOOLEAN),
    ("missions", pa.list_(MISSION)), ("stateHash", STRING),
    ("elapsedMs", NUMBER), ("elapsedSeconds", NUMBER),
    ("detail", DETAIL_REFERENCE),
])
