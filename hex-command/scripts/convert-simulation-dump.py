"""Convert JSONL match journals directly to Arrow IPC using an explicit schema.

uv run --with pyarrow scripts/convert-simulation-dump.py OUTPUT.arrow INPUT.jsonl [INPUT.jsonl ...]

Inputs are the detailed JSONL journals, not report.json (the summary/index).
Each input is read in the supplied order. This script does not sort, align,
compare, filter, or decode records into Python objects.
"""
import argparse
from pathlib import Path
import sys
from time import perf_counter

import pyarrow as pa
import pyarrow.json as arrow_json

# The combat simulation skill owns the shared journal schema.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]
                       / ".agents/skills/combat-simulation/scripts"))
from simulation_dump_schema import SIMULATION_DUMP_SCHEMA


def convert(inputs, output):
    """Read native Arrow batches from JSONL and write them unchanged to IPC."""
    started = perf_counter()
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_name(output.name + ".tmp")
    rows = 0
    read_options = arrow_json.ReadOptions(
        block_size=8 * 1024 * 1024, use_threads=True,
    )
    parse_options = arrow_json.ParseOptions(
        explicit_schema=SIMULATION_DUMP_SCHEMA,
        unexpected_field_behavior="error",
    )
    with pa.OSFile(str(temporary), "wb") as sink:
        with pa.ipc.new_file(sink, SIMULATION_DUMP_SCHEMA) as writer:
            for path in inputs:
                with arrow_json.open_json(
                    path, read_options=read_options, parse_options=parse_options,
                ) as reader:
                    for batch in reader:
                        writer.write_batch(batch)
                        rows += batch.num_rows
    temporary.replace(output)
    print(f"Saved {output}; {rows} matches; {perf_counter() - started:.6f} seconds")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("inputs", nargs="+", type=Path)
    args = parser.parse_args()
    convert(args.inputs, args.output)


if __name__ == "__main__":
    main()
