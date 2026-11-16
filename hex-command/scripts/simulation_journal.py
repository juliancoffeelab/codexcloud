"""Readers for the current native Arrow gameplay journal contract."""

import json

FORMAT = b"arrow-ipc-snapshot-dictionaries-v5"


def validate_report(report):
    if report.get("detailFormat") != FORMAT.decode():
        raise ValueError("Report must index current Arrow gameplay format v5")
    if not isinstance(report.get("unitProfiles"), dict) or not isinstance(
        report.get("unitFamilies"), dict
    ):
        raise ValueError("Report must contain its recorded unit catalog")
    if any(not isinstance(p.get("role"), str) for p in report["unitProfiles"].values()):
        raise ValueError(
            "Recorded unit profiles must include their current catalog role"
        )
    for outcome in report["outcomes"]:
        if outcome["kind"] not in ("arena", "campaign") or not outcome["detail"][
            "file"
        ].endswith(".arrow"):
            raise ValueError("Gameplay journals must use the current Arrow format")


def hex_(value):
    if (
        not isinstance(value, dict)
        or set(value) != {"col", "row"}
        or any(
            type(value[k]) is not int or not -(2**31) <= value[k] < 2**31
            for k in ("col", "row")
        )
    ):
        raise ValueError("Gameplay coordinates must be current structured hexes")


def validate_header(result):
    if result.get("kind") not in ("arena", "campaign"):
        raise ValueError("Invalid gameplay kind")
    chapters = [result] if result["kind"] == "arena" else result["missions"]

    for chapter in chapters:
        for state in (chapter["initial"], chapter["final"]):
            for unit in state["units"]:
                hex_(unit["tile"])
                if not isinstance(unit.get("faction"), str) or "side" in unit:
                    raise ValueError("Unit must record current faction ownership")
        for goal in chapter.get("goals", []):
            for tile in goal["tiles"]:
                hex_(tile)


def validate_phase(phase):
    for unit in phase.get("losses", []):
        hex_(unit["tile"])
    for action in phase.get("actions", []):
        if "tile" in action:
            hex_(action["tile"])
    for field in ("goals", "objectives"):
        for goal in phase.get(field, []):
            for tile in goal["tiles"]:
                hex_(tile)
    for order in (
        ((phase.get("planning") or {}).get("orders") or {})
        .get("tactics", {})
        .get("objectives", [])
    ):
        for body in order["objective"].values():
            for field in ("destination", "location"):
                if field in body:
                    hex_(body[field])
    for audit in phase.get("leaderAudit", []):
        for field in ("before", "after"):
            if audit.get(field) is not None:
                hex_(audit[field]["tile"])
        if "minTile" in (audit.get("exposure") or {}):
            hex_(audit["exposure"]["minTile"])


def arrow_batch(data, name):
    if not name.endswith(".arrow"):
        raise ValueError("Gameplay journals must use the current Arrow format")
    import pyarrow as pa

    reader = pa.ipc.open_stream(data)
    if (reader.schema.metadata or {}).get(b"hex-command.detail-format") != FORMAT:
        raise ValueError("Gameplay journals must use Arrow gameplay format v5")
    batches = iter(reader)
    batch = next(batches, None)
    if batch is None or next(batches, None) is not None or not batch.num_rows:
        raise ValueError("Invalid Arrow gameplay batch")
    columns = {
        "header_json": pa.string(),
        "chapter": pa.uint32(),
        "phase_json": pa.string(),
        "before": pa.list_(pa.uint32()),
        "units": pa.list_(pa.uint32()),
        "ownership": pa.uint32(),
        "unit_snapshots": None,
        "ownership_snapshots": pa.list_(pa.list_(pa.string())),
    }
    if batch.schema.names != list(columns) or any(
        dtype is not None and batch.schema.field(k).type != dtype
        for k, dtype in columns.items()
    ):
        raise ValueError("Unexpected Arrow gameplay schema")
    dictionary = batch.schema.field("unit_snapshots").type
    if not pa.types.is_list(dictionary) or not pa.types.is_struct(
        dictionary.value_type
    ):
        raise ValueError("Invalid Arrow unit dictionary schema")
    unit = dictionary.value_type
    tile = pa.struct(
        [
            pa.field("col", pa.int32(), nullable=False),
            pa.field("row", pa.int32(), nullable=False),
        ]
    )
    required = {
        "id": pa.string(),
        "type": pa.string(),
        "tile": tile,
        "hp": pa.int32(),
        "faction": pa.string(),
        "role": pa.string(),
        "rebuilding": pa.bool_(),
    }
    if "side" in unit.names or any(
        k not in unit.names or unit.field(k).type != t for k, t in required.items()
    ):
        raise ValueError("Invalid Arrow unit dictionary schema")
    if (
        not batch["header_json"][0].is_valid
        or not batch["unit_snapshots"][0].is_valid
        or not batch["ownership_snapshots"][0].is_valid
    ):
        raise ValueError("Missing Arrow header or snapshot dictionaries")
    values = batch["unit_snapshots"][0].values
    if any(values.field(k).null_count for k in ("id", "type", "tile", "hp", "faction")):
        raise ValueError("Missing Arrow unit facts")
    validate_header(json.loads(batch["header_json"][0].as_py()))
    return batch


def decode_detail(data, name):
    batch = arrow_batch(data, name)
    columns = batch.to_pydict()
    result = json.loads(columns["header_json"][0])
    validate_header(result)
    units = []
    for saved in columns["unit_snapshots"][0]:
        unit = {k: v for k, v in saved.items() if v is not None}
        for k in list(unit):
            if k.endswith("_json"):
                value = json.loads(unit.pop(k))
                if not isinstance(value, list):
                    raise ValueError("Invalid Arrow unit array")
                if k != "items_json" or value:
                    unit[k[:-5]] = value
        units.append(unit)
    owners = columns["ownership_snapshots"][0]
    for i in range(1, batch.num_rows):
        phase = json.loads(columns["phase_json"][i])
        validate_phase(phase)
        chapter = columns["chapter"][i]
        if (
            chapter is None
            or (result["kind"] == "arena" and chapter != 0)
            or (result["kind"] == "campaign" and chapter >= len(result["missions"]))
        ):
            raise ValueError("Invalid Arrow chapter")
        target = result if result["kind"] == "arena" else result["missions"][chapter]
        for k in ("before", "units"):
            indices = columns[k][i]
            if indices is None or any(
                j is None or not 0 <= j < len(units) for j in indices
            ):
                raise ValueError("Invalid Arrow roster index")
            phase[k] = [units[j] for j in indices]
        j = columns["ownership"][i]
        if j is None or not 0 <= j < len(owners) or owners[j] is None:
            raise ValueError("Invalid Arrow ownership index")
        tiles = target["initial"]["tiles"]
        if len(owners[j]) != len(tiles):
            raise ValueError("Invalid Arrow ownership snapshot")
        phase["ownership"] = [
            {"tile": {"col": t["col"], "row": t["row"]}, "owner": o}
            for t, o in zip(tiles, owners[j])
        ]
        target["phases"].append(phase)
    return result


def analysis_view(value, factions=None):
    """Present current phases/results as the analyzer's stable two-seat labels.

    This is an analysis-only copy. Lossless decoding, exports, checksums and
    replay inputs retain their original engine representation unchanged.
    """
    if isinstance(value, list):
        return [analysis_view(item, factions) for item in value]
    if not isinstance(value, dict):
        return value
    declared = value.get("factions") or value.get("initial", {}).get("factions")
    if declared:
        factions = {f["id"]: f for f in declared}
    result = {key: analysis_view(item, factions) for key, item in value.items()}
    # Unit ownership is faction-only. The two-seat report labels are
    # derived in the analysis copy, never written back into replay checkpoints.
    if "faction" in value and "hp" in value and "tile" in value and "side" not in value:
        result["side"] = factions[value["faction"]]["phase"]
    for key in ("side", "turn", "first"):
        if result.get(key) == "ai":
            result[key] = "enemy"
    if result.get("winner") in ("playerVictory", "playerDefeat"):
        result["winner"] = "player" if result["winner"] == "playerVictory" else "enemy"
    return result
