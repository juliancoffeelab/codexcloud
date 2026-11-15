"""Independent report/journal boundary; shared snapshots stay shared.

The decoder is an I/O adapter. No legacy analyzer or diagnostics are imported.
Analysis normalization visits each snapshot dictionary entry once, not once per
phase reference. The strict regression comparator uses the unmodified decoder.
"""

import gzip
import hashlib
import json
from pathlib import Path

import pyarrow as pa

FORMATS = tuple(f"arrow-ipc-snapshot-dictionaries-v{i}".encode() for i in range(1, 6))


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def identity(value):
    return value["kind"] + ":" + canonical(value["config"])


def label(config, side):
    blocks = config.get(side + "Blocks")
    return (
        "-".join(blocks[k] for k in ("recruiter", "campaign", "planner"))
        if blocks
        else config.get(side, "unrecorded")
    )


def read_json(path):
    with gzip.open(path, "rt") if path.suffix == ".gz" else path.open() as stream:
        return json.load(stream)


class Readers:
    """Keep physical journal streams open for one ordered ingestion pass."""

    def __init__(self):
        self.streams = {}

    def __enter__(self):
        return self

    def __exit__(self, *args):
        for stream in self.streams.values():
            stream.close()

    def read(self, directory, reference):
        name = reference["file"]
        if not name or Path(name).name != name or name in (".", ".."):
            raise ValueError("Detail reference must name an adjacent journal")
        if reference["offset"] < 0 or reference["bytes"] <= 0:
            raise ValueError("Invalid detail byte range")
        path = directory / name
        if path not in self.streams:
            self.streams[path] = (
                path.open("rb") if path.exists() else gzip.open(str(path) + ".gz", "rb")
            )
        reader = self.streams[path]
        reader.seek(reference["offset"])
        raw = reader.read(reference["bytes"])
        if (
            len(raw) != reference["bytes"]
            or hashlib.sha256(raw).hexdigest() != reference["sha256"]
        ):
            raise ValueError("Invalid gameplay checksum or byte range: " + str(path))
        return raw


def read_bytes(directory, reference, *, readers=None):
    if readers is not None:
        return readers.read(directory, reference)
    with Readers() as streams:
        return streams.read(directory, reference)


def decode(raw, name):
    if not name.endswith(".arrow"):
        return json.loads(raw)
    reader = pa.ipc.open_stream(raw)
    version = (reader.schema.metadata or {}).get(b"hex-command.detail-format")
    if version not in FORMATS:
        raise ValueError("Unknown Arrow gameplay format")
    batches = iter(reader)
    batch = next(batches, None)
    if batch is None or next(batches, None) is not None or not batch.num_rows:
        raise ValueError("Invalid Arrow gameplay batch")
    names = [
        "header_json",
        "chapter",
        "phase_json",
        "before",
        "units",
        "ownership",
        "unit_snapshots",
        "ownership_snapshots",
    ]
    if batch.schema.names != names:
        raise ValueError("Unexpected Arrow gameplay columns")
    # Only the snapshot entries become objects; repeated phase references share
    # them. Arrow remains responsible for validating offsets, buffers and types.
    columns = batch.to_pydict()
    result = json.loads(columns["header_json"][0])
    typed = version != FORMATS[0]
    if typed:
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
    else:
        units = [json.loads(x) for x in columns["unit_snapshots"][0]]
        owners = [json.loads(x) for x in columns["ownership_snapshots"][0]]
    for i in range(1, batch.num_rows):
        phase = json.loads(columns["phase_json"][i])
        chapter = columns["chapter"][i]
        if (
            chapter is None
            or chapter < 0
            or (result["kind"] == "arena" and chapter != 0)
            or (result["kind"] != "arena" and chapter >= len(result["missions"]))
        ):
            raise ValueError("Invalid Arrow chapter")
        for k in ("before", "units"):
            indices = columns[k][i]
            if indices is None or any(
                j is None or not 0 <= j < len(units) for j in indices
            ):
                raise ValueError("Invalid Arrow roster index")
            phase[k] = [units[j] for j in indices]
        j = columns["ownership"][i]
        if j is None or not 0 <= j < len(owners):
            raise ValueError("Invalid Arrow ownership index")
        if typed:
            initial = (
                result["initial"]
                if result["kind"] == "arena"
                else result["missions"][chapter]["initial"]
            )
            tiles = initial["tiles"]
            if len(owners[j]) != len(tiles):
                raise ValueError("Invalid Arrow ownership snapshot")
            values = owners[j]
            if version == FORMATS[1]:
                if any(x not in (None, 0, 1) for x in values):
                    raise ValueError("Invalid Arrow owner")
                values = [None if x is None else ("player", "enemy")[x] for x in values]
            elif any(x is not None and not isinstance(x, str) for x in values):
                raise ValueError("Invalid Arrow owner")
            phase["ownership"] = [
                {
                    "tile": {"col": t["col"], "row": t["row"]}
                    if version in FORMATS[3:]
                    else t["id"],
                    "owner": owner,
                }
                for t, owner in zip(tiles, values)
            ]
        else:
            phase["ownership"] = owners[j]
        target = result if result["kind"] == "arena" else result["missions"][chapter]
        target["phases"].append(phase)
    return result


def normalize(value):
    memo = {}

    def visit(x, factions=None):
        if isinstance(x, list):
            return [visit(v, factions) for v in x]
        if not isinstance(x, dict):
            return x
        is_unit = "hp" in x and "tile" in x and "faction" in x
        phase = (
            (factions or {}).get(x.get("faction"), {}).get("phase") if is_unit else None
        )
        cache_key = (id(x), phase)
        if is_unit and cache_key in memo:
            return memo[cache_key]
        declared = x.get("factions") or x.get("initial", {}).get("factions")
        if declared:
            factions = {f["id"]: f for f in declared}
        result = {k: visit(v, factions) for k, v in x.items()}
        if is_unit:
            memo[cache_key] = result
        if "faction" in x and "hp" in x and "tile" in x and "side" not in x:
            result["side"] = factions[x["faction"]]["phase"]
        for k in ("side", "turn", "first"):
            if result.get(k) == "ai":
                result[k] = "enemy"
        if result.get("winner") in ("playerVictory", "playerDefeat"):
            result["winner"] = (
                "player" if result["winner"] == "playerVictory" else "enemy"
            )
        return result

    return visit(value)


def paths(inputs):
    result = []
    for path in inputs:
        if path.is_dir():
            found = next(
                (
                    path / n
                    for n in ("report.json", "report.json.gz", "outcomes.json")
                    if (path / n).exists()
                ),
                None,
            )
            result.extend(
                [found]
                if found
                else paths(sorted(p for p in path.iterdir() if p.is_dir()))
            )
        else:
            result.append(path)
    return result


def journals(reports, kinds=("arena", "campaign")):
    for path, report in reports:
        if report.get("status") != "complete":
            raise ValueError("Unit usage requires a completed run")
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
            if kind not in kinds:
                continue
            ref = saved["detail"]
            result = normalize(decode(read_bytes(path.parent, ref), ref["file"]))
            result.setdefault(
                "kind", "campaign" if "campaign" in result["config"] else "arena"
            )
            if result["config"] != saved["config"]:
                raise ValueError("Journal configuration mismatch")
            yield report, result
