"""Select the same seed from two full Arrow dumps, preserving each source order.

uv run --with pyarrow scripts/sample-simulation-dumps.py A.arrow B.arrow OUT_DIR
"""
import argparse
import json
from pathlib import Path
from time import perf_counter

import pyarrow as pa
import pyarrow.compute as pc


def sample(source: Path, output: Path, seed: int) -> int:
    """Copy matching record batches with every original field intact."""
    rows = 0
    temporary = output.with_suffix(".arrow.tmp")
    with pa.memory_map(str(source), "r") as mapping:
        reader = pa.ipc.open_file(mapping)
        with pa.OSFile(str(temporary), "wb") as sink:
            with pa.ipc.new_file(sink, reader.schema) as writer:
                for index in range(reader.num_record_batches):
                    batch = reader.get_batch(index)
                    seeds = pc.struct_field(batch.column("config"), "seed")
                    keep = pc.equal(seeds, seed)
                    if not pc.any(keep).as_py():
                        continue
                    selected = batch.filter(keep)
                    writer.write_batch(selected)
                    rows += selected.num_rows
    temporary.replace(output)
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("a", type=Path)
    parser.add_argument("b", type=Path)
    parser.add_argument("output_dir", type=Path)
    parser.add_argument("--seed", type=int)
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)
    seed = args.seed
    if seed is None:
        with pa.memory_map(str(args.a), "r") as mapping:
            first = pa.ipc.open_file(mapping).get_batch(0)
            seed = pc.struct_field(first.column("config"), "seed")[0].as_py()
    results = []
    for source, name in ((args.a, "ts.arrow"), (args.b, "rust.arrow")):
        output = args.output_dir / name
        start = perf_counter()
        rows = sample(source, output, seed)
        result = {"source": str(source), "output": str(output), "seed": seed,
                  "matches": rows, "seconds": perf_counter() - start,
                  "bytes": output.stat().st_size}
        results.append(result)
        print(json.dumps(result), flush=True)
    manifest = {"selection": "config.seed only", "row_order": "original source order",
                "payload_changes": "none", "samples": results}
    (args.output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")


if __name__ == "__main__":
    main()
