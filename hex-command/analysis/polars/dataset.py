"""Decode checked journals once into typed relations.

Snapshot dictionaries stay Arrow-backed. Phase JSON is projected by Polars into
one typed struct column; rosters are index joins, never expanded Python objects.
The object decoder is used only at the boundary for historical JSON/v1-v3 input.
"""

import json
import gzip
import polars as pl
import pyarrow as pa
from journal import FORMATS, Readers, decode, identity, label, normalize, read_bytes

TILE = pl.Struct({"col": pl.Int64, "row": pl.Int64})
UNIT = pl.Struct(
    {
        "id": pl.String,
        "type": pl.String,
        "tile": TILE,
        "hp": pl.Int64,
        "side": pl.String,
        "faction": pl.String,
        "role": pl.String,
        "name": pl.String,
        "rebuilding": pl.Boolean,
    }
)
DAMAGE = pl.Struct(
    {
        k: pl.Float64
        for k in (
            "effectiveDamage",
            "afterTerrainDamage",
            "availableHp",
            "effectiveArmourPrevented",
            "effectiveTerrainPrevented",
            "rawDamage",
            "afterArmourDamage",
        )
    }
)
THORNS = pl.Struct(
    {
        "damage": pl.Float64,
        "auraDamage": pl.Float64,
        "effectiveDamage": pl.Float64,
        "auraSources": pl.List(pl.String),
    }
)
COMBAT = pl.Struct(
    {
        "unit": pl.String,
        "target": pl.String,
        "damage": pl.Float64,
        "retaliation": pl.Float64,
        "retaliationCount": pl.Int64,
        "mitigation": pl.Struct({"attack": DAMAGE, "retaliation": DAMAGE}),
        "thorns": THORNS,
    }
)
ACTION = pl.Struct(
    {
        "kind": pl.String,
        "unit": pl.String,
        "type": pl.String,
        "tile": TILE,
        "cost": pl.Float64,
    }
)
HEALING = pl.Struct({"providers": pl.List(pl.String), "amount": pl.Float64})
GOAL = pl.Struct(
    {
        "label": pl.String,
        "tiles": pl.List(TILE),
        "current": pl.Int64,
        "total": pl.Int64,
        "done": pl.Boolean,
    }
)
OBJECTIVE = pl.Struct({"kind": pl.String, "tiles": pl.List(TILE)})
ARMY = pl.Struct({"units": pl.Int64, "hp": pl.Int64, "hpValue": pl.Float64})
LEADER_STATE = pl.Struct({"tile": TILE, "hp": pl.Int64})
EXPOSURE = pl.Struct(
    {
        **{
            k: pl.Int64
            for k in (
                "beforeDamage",
                "chosenTileDamage",
                "minDamage",
                "reachableTiles",
                "safeTiles",
                "afterDamage",
            )
        },
        "minTile": TILE,
        "chosenTileReachable": pl.Boolean,
    }
)
AUDIT = pl.Struct(
    {
        "leader": pl.String,
        "side": pl.String,
        "before": LEADER_STATE,
        "after": LEADER_STATE,
        **{
            k: ARMY
            for k in ("armyBefore", "armyAfter", "opponentBefore", "opponentAfter")
        },
        "recruitmentBefore": pl.Struct(
            {
                "gold": pl.Int64,
                "onOwnedKeep": pl.Boolean,
                "legalOptions": pl.Int64,
                "affordableOptions": pl.Int64,
            }
        ),
        "exposure": EXPOSURE,
    }
)
DESTINATION = pl.Struct({"actor": pl.Int64, "destination": TILE, "location": TILE})
PLANNING = pl.Struct(
    {
        "report": pl.Struct(
            {
                "candidates": pl.Int64,
                "duties": pl.List(
                    pl.Struct(
                        {"id": pl.String, "status": pl.String, "reason": pl.String}
                    )
                ),
            }
        ),
        "orders": pl.Struct(
            {
                "tactics": pl.Struct(
                    {
                        "objectives": pl.List(
                            pl.Struct(
                                {
                                    "id": pl.String,
                                    "objective": pl.Struct(
                                        {"GoTo": DESTINATION, "Occupy": DESTINATION}
                                    ),
                                }
                            )
                        ),
                        "constraints": pl.List(
                            pl.Struct({"Survive": pl.Struct({"actor": pl.Int64})})
                        ),
                    }
                )
            }
        ),
    }
)
PHASE = pl.Struct(
    {
        "side": pl.String,
        "round": pl.Int64,
        "actions": pl.List(ACTION),
        "combats": pl.List(COMBAT),
        "losses": pl.List(UNIT),
        "healing": pl.List(HEALING),
        "leaderAudit": pl.List(AUDIT),
        "objectives": pl.List(OBJECTIVE),
        "goals": pl.List(GOAL),
        "planning": PLANNING,
    }
)
UNIT_SCHEMA = dict(UNIT.to_schema())


def concat(frames, schema=None):
    return (
        pl.concat(frames, how="diagonal_relaxed", rechunk=False)
        if frames
        else pl.DataFrame(schema=schema or {})
    )


def side(expr):
    return expr.replace({"ai": "enemy"})


def cell(expr):
    return pl.concat_str(
        expr.struct.field("col"), expr.struct.field("row"), separator=","
    )


def expand(df, name):
    return (
        df.select("p", pl.col(name))
        .explode(name)
        .filter(pl.col(name).is_not_null())
        .unnest(name)
    )


def legacy_tiles(result):
    """Historical wire adapter; no modern journal passes through this path."""
    chapters = [result] if result["kind"] == "arena" else result["missions"]

    def tile(t):
        if isinstance(t, str):
            x, y = t.split(",")
            return dict(col=int(x), row=int(y))
        return t

    for chapter in chapters:
        for state in [
            chapter.get("initial", {}),
            chapter.get("final", {}),
        ] + chapter.get("phases", []):
            for field in ("units", "before", "losses"):
                for u in state.get(field, []):
                    u["tile"] = tile(u["tile"])
            for a in state.get("actions", []):
                if "tile" in a:
                    a["tile"] = tile(a["tile"])
            for audit in state.get("leaderAudit", []):
                for key in ("before", "after"):
                    if audit.get(key) is not None:
                        audit[key]["tile"] = tile(audit[key]["tile"])
                exposure = audit.get("exposure") or {}
                if "minTile" in exposure:
                    exposure["minTile"] = tile(exposure["minTile"])
            for field in ("goals", "objectives"):
                for g in state.get(field, []):
                    g["tiles"] = [tile(t) for t in g.get("tiles", [])]
    return result


class Dataset:
    def __init__(self, reports, *, telemetry=False, readers=None):
        if readers is not None:
            self._load(reports, telemetry=telemetry, readers=readers)
        else:
            with Readers() as streams:
                self._load(reports, telemetry=telemetry, readers=streams)

    def _load(self, reports, *, telemetry, readers):
        snapshots, refs, phase_parts = [], [], []
        self.headers = []
        meta, chapters, tiles, factions, initial, final, owners = (
            [],
            [],
            [],
            [],
            [],
            [],
            [],
        )
        profile_rows = []
        phase_base = 0
        for source, (path, report) in enumerate(reports):
            if telemetry and report.get("status") != "complete":
                raise ValueError("Telemetry requires a completed run")
            legacy = {}
            legacy_path = path.parent / "details.jsonl.gz"
            if path.name == "outcomes.json" and legacy_path.exists():
                with gzip.open(legacy_path, "rt") as stream:
                    for line in stream:
                        value = normalize(json.loads(line))
                        value.setdefault(
                            "kind", "campaign" if "missions" in value else "arena"
                        )
                        legacy[identity(value)] = value
            for unit, profile in (
                report.get("unitProfiles")
                or {
                    u: {"family": family}
                    for u, family in (report.get("unitFamilies") or {}).items()
                }
            ).items():
                profile_rows.append(dict(source=source, type=unit, **profile))
            for saved in sorted(
                report["outcomes"],
                key=lambda o: (
                    o.get("detail", {}).get("file", ""),
                    o.get("detail", {}).get("offset", 0),
                ),
            ):
                kind = saved.get(
                    "kind", "campaign" if "campaign" in saved["config"] else "arena"
                )
                if kind == "arena" and (
                    not telemetry
                    or saved["config"]["armies"]["player"]
                    == saved["config"]["armies"]["enemy"]
                ):
                    continue
                ref = saved.get("detail", {})
                if not all(k in ref for k in ("file", "offset", "bytes", "sha256")):
                    if kind == "arena":
                        raise ValueError(
                            "Telemetry requires recorded gameplay journals"
                        )
                    result, batch = (
                        legacy_tiles(legacy.get(identity(saved), saved)),
                        None,
                    )
                else:
                    raw = read_bytes(path.parent, ref, readers=readers)
                    batch = None
                    if ref["file"].endswith(".arrow"):
                        reader = pa.ipc.open_stream(raw)
                        version = (reader.schema.metadata or {}).get(
                            b"hex-command.detail-format"
                        )
                        if version not in FORMATS:
                            raise ValueError("Unknown Arrow gameplay format")
                        batches = list(reader)
                        if len(batches) != 1 or not batches[0].num_rows:
                            raise ValueError("Invalid Arrow gameplay batch")
                        batch = batches[0]
                        if batch.schema.names != [
                            "header_json",
                            "chapter",
                            "phase_json",
                            "before",
                            "units",
                            "ownership",
                            "unit_snapshots",
                            "ownership_snapshots",
                        ]:
                            raise ValueError("Unexpected Arrow gameplay columns")
                        if version not in FORMATS[3:]:
                            batch = None
                    if batch is None:
                        result = legacy_tiles(normalize(decode(raw, ref["file"])))
                    else:
                        result = normalize(json.loads(batch["header_json"][0].as_py()))
                result.setdefault("kind", kind)
                if identity(result) != identity(normalize(saved)):
                    raise ValueError("Journal configuration mismatch")
                m = len(meta)
                config = result["config"]
                meta.append(
                    dict(
                        m=m,
                        source=source,
                        kind=kind,
                        matchId=identity(result),
                        campaign=config.get("campaign", ""),
                        diplomacy=config.get("diplomacy", ""),
                        objective=config.get("objective", "authored"),
                        seed=config.get("seed"),
                        first=config.get("first", ""),
                        landscape=config.get("landscape", ""),
                        player=label(config, "player"),
                        enemy=label(config, "enemy"),
                        playerRace=config.get("armies", {}).get("player", ""),
                        enemyRace=config.get("armies", {}).get("enemy", ""),
                    )
                )
                self.headers.append((saved, result, report))
                missions = [result] if kind == "arena" else result.get("missions", [])
                for ch, chapter in enumerate(missions):
                    chapters.append(dict(m=m, ch=ch, mission=chapter.get("id", "")))
                    state = chapter.get("initial", {})
                    for ordinal, t in enumerate(state.get("tiles", [])):
                        tiles.append(
                            dict(
                                m=m,
                                ch=ch,
                                ordinal=ordinal,
                                tile=t["id"],
                                col=t["col"],
                                row=t["row"],
                                terrain=t["terrain"],
                                owner=t.get("owner"),
                            )
                        )
                    for f in state.get("factions", []):
                        factions.append(
                            dict(
                                m=m,
                                ch=ch,
                                faction=f["id"],
                                factionSide="enemy"
                                if f["phase"] == "ai"
                                else f["phase"],
                            )
                        )
                    for dest, state_ in (
                        (initial, state),
                        (final, chapter.get("final", {})),
                    ):
                        for u in state_.get("units", []):
                            dest.append(
                                dict(m=m, ch=ch, **{k: u.get(k) for k in UNIT_SCHEMA})
                            )
                if batch is not None:
                    dictionary = batch["unit_snapshots"][0]
                    ownership = batch["ownership_snapshots"][0]
                    if not dictionary.is_valid or not ownership.is_valid:
                        raise ValueError("Invalid Arrow snapshot dictionaries")
                    units = pl.from_arrow(
                        pa.Table.from_struct_array(dictionary.values)
                    ).select(
                        *[
                            pl.col(k)
                            if k in dictionary.values.type.names
                            else pl.lit(None, dtype=ty).alias(k)
                            for k, ty in UNIT_SCHEMA.items()
                        ]
                    )
                    snapshots.append(
                        units.with_row_index("snapshot").with_columns(
                            pl.lit(m).alias("m")
                        )
                    )
                    r = (
                        pl.from_arrow(
                            batch.select(
                                ["chapter", "before", "units", "ownership"]
                            ).slice(1)
                        )
                        .rename({"chapter": "ch"})
                        .with_row_index("p", offset=phase_base)
                        .with_columns(pl.lit(m).alias("m"))
                    )
                    if r.filter(
                        pl.col("ch").is_null()
                        | (pl.col("ch") < 0)
                        | (pl.col("ch") >= len(missions))
                    ).height:
                        raise ValueError("Invalid Arrow chapter")
                    for field in ("before", "units"):
                        if (
                            r[field].null_count()
                            or r.select(
                                pl.col(field)
                                .list.eval(
                                    pl.element().is_null()
                                    | (pl.element() < 0)
                                    | (pl.element() >= units.height)
                                )
                                .list.any()
                                .any()
                            ).item()
                        ):
                            raise ValueError("Invalid Arrow roster index")
                    if (
                        r["ownership"].null_count()
                        or r.filter(
                            (pl.col("ownership") < 0)
                            | (pl.col("ownership") >= len(ownership.values))
                        ).height
                    ):
                        raise ValueError("Invalid Arrow ownership index")
                    owner_values = ownership.values
                    if owner_values.null_count or not (
                        pa.types.is_string(owner_values.values.type)
                        or pa.types.is_null(owner_values.values.type)
                    ):
                        raise ValueError("Invalid Arrow owner")
                    lengths = pl.from_arrow(owner_values).list.len()
                    for ch, owner in r.select("ch", "ownership").unique().iter_rows():
                        if lengths[owner] != len(
                            missions[ch].get("initial", {}).get("tiles", [])
                        ):
                            raise ValueError("Invalid Arrow ownership snapshot")
                    # Ownership is needed only by the sequential capture pass and
                    # campaign repetition signatures. Keep dictionary indices.
                    owners.append((m, ownership.values))
                    refs.append(r)
                    phase_parts.append(
                        pl.from_arrow(batch["phase_json"].slice(1))
                        .rename("json")
                        .to_frame()
                    )
                    phase_base += r.height
                else:
                    # Decode historical objects once into the same typed tables.
                    local_units, local_refs, local_phases, local_owners = [], [], [], []
                    for ch, chapter in enumerate(missions):
                        for phase in chapter.get("phases", []):
                            row = dict(
                                m=m, ch=ch, p=phase_base, ownership=len(local_owners)
                            )
                            for field in ("before", "units"):
                                row[field] = list(
                                    range(
                                        len(local_units),
                                        len(local_units) + len(phase.get(field, [])),
                                    )
                                )
                                local_units.extend(phase.get(field, []))
                            local_refs.append(row)
                            local_owners.append(
                                [o.get("owner") for o in phase.get("ownership", [])]
                            )
                            local_phases.append(
                                json.dumps({k: phase.get(k) for k in PHASE.to_schema()})
                            )
                            phase_base += 1
                    if local_refs:
                        snapshots.append(
                            pl.from_dicts(local_units, schema=UNIT_SCHEMA, strict=False)
                            .with_row_index("snapshot")
                            .with_columns(pl.lit(m).alias("m"))
                        )
                        refs.append(pl.from_dicts(local_refs))
                        phase_parts.append(pl.DataFrame({"json": local_phases}))
                        owners.append(
                            (m, pa.array(local_owners, type=pa.list_(pa.string())))
                        )
        self.meta = (
            pl.from_dicts(meta, infer_schema_length=None)
            if meta
            else pl.DataFrame(schema={"m": pl.Int64})
        )
        self.chapters = (
            pl.from_dicts(chapters)
            if chapters
            else pl.DataFrame(
                schema={"m": pl.Int64, "ch": pl.Int64, "mission": pl.String}
            )
        )
        self.tiles = (
            pl.from_dicts(tiles, infer_schema_length=None)
            if tiles
            else pl.DataFrame(
                schema={
                    "m": pl.Int64,
                    "ch": pl.Int64,
                    "tile": pl.String,
                    "col": pl.Int64,
                    "row": pl.Int64,
                    "terrain": pl.String,
                    "owner": pl.String,
                    "ordinal": pl.Int64,
                }
            )
        )
        self.factions = (
            pl.from_dicts(factions)
            if factions
            else pl.DataFrame(
                schema={
                    "m": pl.Int64,
                    "ch": pl.Int64,
                    "faction": pl.String,
                    "factionSide": pl.String,
                }
            )
        )
        self.profiles = (
            pl.from_dicts(profile_rows, infer_schema_length=None)
            if profile_rows
            else pl.DataFrame(schema={"source": pl.Int64, "type": pl.String})
        )
        required_profiles = {
            "source": pl.Int64,
            "type": pl.String,
            "family": pl.String,
            "race": pl.String,
            "cost": pl.Float64,
            "healing": pl.Float64,
            "thornAura": pl.Float64,
            "thorns": pl.Float64,
        }
        self.profiles = self.profiles.with_columns(
            pl.lit(None, dtype=ty).alias(k)
            for k, ty in required_profiles.items()
            if k not in self.profiles.columns
        )
        self.initial = self.unit_sides(
            pl.from_dicts(
                initial,
                schema={"m": pl.Int64, "ch": pl.Int64, **UNIT_SCHEMA},
                strict=False,
            )
        )
        self.final = self.unit_sides(
            pl.from_dicts(
                final,
                schema={"m": pl.Int64, "ch": pl.Int64, **UNIT_SCHEMA},
                strict=False,
            )
        )
        self.owners = dict(owners)
        self.refs = concat(
            refs,
            schema={
                "p": pl.Int64,
                "m": pl.Int64,
                "ch": pl.Int64,
                "before": pl.List(pl.Int64),
                "units": pl.List(pl.Int64),
                "ownership": pl.Int64,
            },
        )
        payload = (
            concat(phase_parts)
            .select(pl.col("json").str.json_decode(PHASE).alias("phase"))
            .unnest("phase")
            if phase_parts
            else pl.DataFrame(schema=PHASE.to_schema())
        )
        self.phases = (
            self.refs.select("p", "m", "ch")
            .hstack(payload)
            .with_columns(
                side(pl.col("side")),
                (pl.col("p").rank("ordinal").over(["m", "ch"]) - 1)
                .cast(pl.Int64)
                .alias("phase"),
            )
        )
        self.snapshots = concat(
            snapshots, schema={"m": pl.Int64, "snapshot": pl.Int64, **UNIT_SCHEMA}
        )
        self.rosters = {}
        for name in ("before", "units"):
            indices = (
                self.refs.select("p", "m", "ch", pl.col(name).alias("snapshot"))
                .explode("snapshot")
                .filter(pl.col("snapshot").is_not_null())
            )
            self.rosters[name] = self.unit_sides(
                indices.join(
                    self.snapshots, on=["m", "snapshot"], how="inner", validate="m:1"
                )
            )
        self.losses = self.unit_sides(
            expand(self.phases, "losses").join(
                self.phases.select("p", "m", "ch"), on="p"
            )
        )
        self.actions = expand(self.phases, "actions").with_row_index("action")
        self.combats = expand(self.phases, "combats").with_row_index("combat")
        self.healing = expand(self.phases, "healing")
        self.audits = expand(self.phases, "leaderAudit").with_columns(
            side(pl.col("side"))
        )
        identity_columns = [
            "p",
            "m",
            "ch",
            "id",
            "type",
            "side",
            "faction",
            "role",
            "cell",
        ]
        before = self.rosters["before"].select(identity_columns)
        after = self.rosters["units"].select(identity_columns)
        losses = self.losses.select(identity_columns)
        self.lookup = (
            concat([before, losses, after]).unique(
                ["p", "id"], keep="last", maintain_order=True
            )
            if telemetry
            else after.head(0)
        )
        self.support_lookup = (
            concat([losses, after, before]).unique(
                ["p", "id"], keep="last", maintain_order=True
            )
            if telemetry
            else before.head(0)
        )

    def unit_sides(self, units):
        result = (
            units.join(
                self.factions, on=["m", "ch", "faction"], how="left", validate="m:1"
            )
            .with_columns(side(pl.coalesce("side", "factionSide")).alias("side"))
            .drop("factionSide")
            .with_columns(cell(pl.col("tile")).alias("cell"))
        )

        if result["side"].null_count():
            raise ValueError("Unit snapshot has no recorded faction/side mapping")
        return result

    def describe(self, units, *, columns=None):
        """Attach dimensions by foreign keys, preserving instance roles."""
        q = (
            units.lazy()
            .join(self.meta.lazy(), on="m", how="left", validate="m:1")
            .join(self.chapters.lazy(), on=["m", "ch"], how="left", validate="m:1")
        )
        profiles = (
            self.profiles.drop("leader")
            if "leader" in self.profiles.columns
            else self.profiles
        )
        q = q.join(
            profiles.lazy().rename({"race": "profileRace"}),
            on=["source", "type"],
            how="left",
            validate="m:1",
            suffix="_profile",
        )
        own = pl.col("side") == "player"
        q = q.with_columns(pl.coalesce("family", "type").alias("family")).with_columns(
            pl.when(pl.col("kind") == "arena")
            .then(pl.when(own).then("playerRace").otherwise("enemyRace"))
            .otherwise("profileRace")
            .alias("race"),
            pl.when(own).then("player").otherwise("enemy").alias("policy"),
            pl.when(own).then("enemy").otherwise("player").alias("opponentPolicy"),
            pl.when(pl.col("kind") == "arena")
            .then("landscape")
            .otherwise("mission")
            .alias("map"),
        )

        return (q.select(columns) if columns is not None else q).collect()
