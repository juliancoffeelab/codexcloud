"""Lossless readers for checksummed JSONL and native Arrow gameplay journals."""
import json

ARROW_FORMAT = "arrow-ipc-snapshot-dictionaries-v1"
ARROW_TYPED_FORMAT = "arrow-ipc-snapshot-dictionaries-v2"
ARROW_FACTION_FORMAT = "arrow-ipc-snapshot-dictionaries-v3"


def decode_detail(data, name):
    if not name.endswith(".arrow"):
        return json.loads(data)
    try:
        import pyarrow as pa
    except ImportError as exc:
        raise RuntimeError("Arrow journals need pyarrow: run with `uv run --with pyarrow`") from exc
    reader = pa.ipc.open_stream(data)
    format_name = (reader.schema.metadata or {}).get(b"hex-command.detail-format")
    if format_name not in (ARROW_FORMAT.encode(), ARROW_TYPED_FORMAT.encode(), ARROW_FACTION_FORMAT.encode()):
        raise ValueError("Unknown Arrow gameplay format")
    batches = iter(reader)
    batch = next(batches, None)
    if batch is None or next(batches, None) is not None or batch.num_rows == 0:
        raise ValueError("Invalid Arrow gameplay batch")
    names = ["header_json", "chapter", "phase_json", "before", "units", "ownership",
             "unit_snapshots", "ownership_snapshots"]
    if batch.schema.names != names:
        raise ValueError("Unexpected Arrow gameplay columns")
    columns = {name: batch.column(name) for name in names}
    result = json.loads(columns["header_json"][0].as_py())
    if format_name in (ARROW_TYPED_FORMAT.encode(), ARROW_FACTION_FORMAT.encode()):
        units = [{key: value for key, value in item.items() if value is not None}
                 for item in columns["unit_snapshots"][0].as_py()]
        ownership = columns["ownership_snapshots"][0].as_py()
    else:
        units = [json.loads(item) for item in columns["unit_snapshots"][0].as_py()]
        ownership = [json.loads(item) for item in columns["ownership_snapshots"][0].as_py()]
    for row in range(1, batch.num_rows):
        phase = json.loads(columns["phase_json"][row].as_py())
        for key in ("before", "units"):
            indices = columns[key][row].as_py()
            if indices is None or any(index is None or not 0 <= index < len(units) for index in indices):
                raise ValueError("Invalid Arrow roster indices")
            phase[key] = [units[index] for index in indices]
        index = columns["ownership"][row].as_py()
        if index is None or not 0 <= index < len(ownership):
            raise ValueError("Invalid Arrow ownership index")
        chapter = columns["chapter"][row].as_py()
        if format_name in (ARROW_TYPED_FORMAT.encode(), ARROW_FACTION_FORMAT.encode()):
            tiles = result["initial"]["tiles"] if result["kind"] == "arena" else result["missions"][chapter]["initial"]["tiles"]
            owners = ownership[index]
            if len(owners) != len(tiles):
                raise ValueError("Invalid Arrow ownership snapshot")
            if format_name == ARROW_FACTION_FORMAT.encode():
                if any(owner is not None and not isinstance(owner, str) for owner in owners):
                    raise ValueError("Invalid Arrow faction owner")
                phase["ownership"] = [{"tile": tile["id"], "owner": owner} for tile, owner in zip(tiles, owners)]
            else:
                if any(owner not in (None, 0, 1) for owner in owners):
                    raise ValueError("Invalid Arrow ownership snapshot")
                phase["ownership"] = [{"tile": tile["id"], "owner": None if owner is None else ("player", "enemy")[owner]}
                                      for tile, owner in zip(tiles, owners)]
        else:
            phase["ownership"] = ownership[index]
        if result["kind"] == "arena":
            if chapter != 0:
                raise ValueError("Invalid Arrow arena chapter")
            result["phases"].append(phase)
        else:
            if chapter is None or not 0 <= chapter < len(result["missions"]):
                raise ValueError("Invalid Arrow campaign chapter")
            result["missions"][chapter]["phases"].append(phase)
    return result
