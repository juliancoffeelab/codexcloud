"""Small current-format Arrow encoder for synthetic journal boundary tests."""

import copy
import json
from simulation_journal import FORMAT


def arrow_detail(value):
    import pyarrow as pa

    header = copy.deepcopy(value)
    chapters = [header] if header["kind"] == "arena" else header["missions"]
    units, owners, phases = [], [], []
    for ch, chapter in enumerate(chapters):
        for phase in chapter["phases"]:
            phase = dict(phase)
            row = dict(
                header_json=None,
                chapter=ch,
                unit_snapshots=None,
                ownership_snapshots=None,
            )
            for key in ("before", "units"):
                roster = phase.pop(key)
                row[key] = list(range(len(units), len(units) + len(roster)))
                units.extend(roster)
            row["ownership"] = len(owners)
            owners.append([o["owner"] for o in phase.pop("ownership")])
            row["phase_json"] = json.dumps(phase)
            phases.append(row)
        chapter["phases"] = []
    tile = pa.struct(
        [
            pa.field("col", pa.int32(), nullable=False),
            pa.field("row", pa.int32(), nullable=False),
        ]
    )
    unit = pa.struct(
        [
            ("id", pa.string()),
            ("type", pa.string()),
            ("tile", tile),
            ("hp", pa.int32()),
            ("faction", pa.string()),
            ("role", pa.string()),
            ("character", pa.string()),
            ("rebuilding", pa.bool_()),
        ]
    )
    schema = pa.schema(
        [
            ("header_json", pa.string()),
            ("chapter", pa.uint32()),
            ("phase_json", pa.string()),
            ("before", pa.list_(pa.uint32())),
            ("units", pa.list_(pa.uint32())),
            ("ownership", pa.uint32()),
            ("unit_snapshots", pa.list_(unit)),
            ("ownership_snapshots", pa.list_(pa.list_(pa.string()))),
        ],
        metadata={b"hex-command.detail-format": FORMAT},
    )
    rows = [
        dict(
            header_json=json.dumps(header),
            chapter=0,
            phase_json=None,
            before=None,
            units=None,
            ownership=None,
            unit_snapshots=units,
            ownership_snapshots=owners,
        )
    ] + phases
    batch = pa.RecordBatch.from_pylist(rows, schema=schema)
    sink = pa.BufferOutputStream()
    with pa.ipc.new_stream(sink, schema) as writer:
        writer.write_batch(batch)
    return sink.getvalue().to_pybytes()
