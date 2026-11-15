"""Current report/Arrow boundary and two-seat presentation projection.

The regression comparator uses lossless decoding without presentation changes.
"""

import gzip
import hashlib
import json
from pathlib import Path

import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from simulation_journal import (
    decode_detail,
    validate_report,
    validate_phase as validate_phase,
    arrow_batch as arrow_batch,
)


decode = decode_detail


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def identity(value):
    return value["kind"] + ":" + canonical(value["config"])


def label(config, side):
    blocks = config.get(side + "Blocks")
    return (
        "-".join(blocks[k] for k in ("recruiter", "campaign", "planner"))
        if blocks
        else config[side]
    )


def read_json(path):
    with gzip.open(path, "rt") if path.suffix == ".gz" else path.open() as stream:
        result = json.load(stream)
    validate_report(result)
    return result


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
        if not name.endswith(".arrow"):
            raise ValueError("Gameplay journals must use the current Arrow format")
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
                    for n in ("report.json", "report.json.gz")
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
            if path.name not in ("report.json", "report.json.gz"):
                raise ValueError(
                    "Expected a current report.json index with Arrow gameplay journals"
                )
            result.append(path)
    return result
