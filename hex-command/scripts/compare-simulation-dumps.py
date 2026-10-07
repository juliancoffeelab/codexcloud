"""Compare unordered Arrow simulation dumps with native Arrow/Polars operations.

uv run --with pyarrow --with polars scripts/compare-simulation-dumps.py \
    A.arrow B.arrow

Polars joins match identities; Arrow gathers rows and compares gameplay values.
The same entry point handles samples and full dumps. Match rows are never
sorted, and records are never converted to Python objects.
"""
import argparse
import json
import resource
import sys
from pathlib import Path
from time import perf_counter, process_time

IMPORT_STARTED = perf_counter()

import polars as pl
import pyarrow as pa
import pyarrow.compute as pc

IMPORT_SECONDS = perf_counter() - IMPORT_STARTED

KEYS = ["kind", "config"]
DERIVED = ("stateHash", "elapsedMs", "elapsedSeconds", "detail")


def gameplay(table: pa.Table) -> pa.Table:
    """Exclude derived metadata at the match and mission level.

The stored input is unchanged. Array order, parent nulls, and every remaining
field are preserved. Loops visit schema fields and Arrow chunks, never records.
"""
    table = table.drop([name for name in table.column_names if name in DERIVED])
    if "missions" not in table.column_names:
        return table
    chunks = []
    for chunk in table.column("missions").chunks:
        values = chunk.values
        fields = [field for field in values.type if field.name not in DERIVED]
        values = pa.StructArray.from_arrays(
            [values.field(field.name) for field in fields], fields=fields,
            mask=pc.is_null(values),
        )
        list_class = pa.LargeListArray if pa.types.is_large_list(chunk.type) else pa.ListArray
        chunks.append(list_class.from_arrays(chunk.offsets, values, mask=pc.is_null(chunk)))
    index = table.schema.get_field_index("missions")
    original = table.schema.field(index)
    column = pa.chunked_array(chunks)
    return table.set_column(index, pa.field("missions", column.type, nullable=original.nullable), column)


def compare(a: pa.Table, b: pa.Table, trace=None):
    """Return equality and wall-time spans; reject missing/duplicate match keys."""
    timings = {}
    owned = []

    def stage(label, function):
        start = perf_counter()
        cpu_start = process_time()
        if trace:
            trace({"event": "stage_start", "stage": label})
        try:
            value = function()
        finally:
            timings[label] = perf_counter() - start
            if trace:
                trace({"event": "stage_end", "stage": label,
                       "wall_seconds": timings[label], "cpu_seconds": process_time() - cpu_start,
                       "peak_rss_mib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
        return value

    a, b = stage("exclude_derived_metadata", lambda: (gameplay(a), gameplay(b)))
    owned.extend((a, b))
    if not a.schema.equals(b.schema, check_metadata=False) or a.num_rows != b.num_rows:
        return False, timings, "schema or match count differs", owned
    if any(key not in table.column_names or table.column(key).null_count
           for table in (a, b) for key in KEYS):
        return False, timings, "missing match identity", owned

    def match_keys():
        left = pl.from_arrow(a.select(KEYS)).with_row_index("__a")
        right = pl.from_arrow(b.select(KEYS)).with_row_index("__b")
        joined = left.join(right, on=KEYS, how="left", validate="1:1",
                           maintain_order="left")
        return left, joined

    try:
        left_keys, matched = stage("match_keys", match_keys)
    except pl.exceptions.ComputeError as error:
        return False, timings, str(error), owned
    owned.extend((left_keys, matched))
    if matched["__b"].is_null().any():
        return False, timings, "missing match identity", owned

    def gather():
        indexes = matched["__b"].to_arrow()
        aligned = indexes.equals(left_keys["__a"].to_arrow())
        return (b if aligned else b.take(indexes)), aligned

    b, already_aligned = stage("gather_rows", gather)
    owned.append(b)
    equal = stage("arrow_equality", lambda: a.equals(b, check_metadata=False))
    return equal, timings, {"already_aligned": already_aligned}, owned


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("a", type=Path)
    parser.add_argument("b", type=Path)
    parser.add_argument("--out", type=Path, help="Save the result and stage timings as JSON")
    args = parser.parse_args()
    pa.set_cpu_count(pl.thread_pool_size())
    total_start = perf_counter()
    mappings = []
    readers = []
    tables = []
    loading = {}
    def trace(event):
        print(json.dumps(event), file=sys.stderr, flush=True)

    trace({"event": "imports", "wall_seconds": IMPORT_SECONDS})
    for name, path in (("a", args.a), ("b", args.b)):
        start = perf_counter()
        cpu_start = process_time()
        trace({"event": "stage_start", "stage": f"load_{name}"})
        mapping = pa.memory_map(str(path), "r")
        reader = pa.ipc.open_file(mapping)
        tables.append(reader.read_all())
        mappings.append(mapping)
        readers.append(reader)
        loading[f"load_{name}"] = perf_counter() - start
        trace({"event": "stage_end", "stage": f"load_{name}",
               "wall_seconds": loading[f"load_{name}"], "cpu_seconds": process_time() - cpu_start,
               "peak_rss_mib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
    rows = [table.num_rows for table in tables]
    equal, timings, detail, owned = compare(tables[0], tables[1], trace=trace)
    through_result = perf_counter() - total_start
    cleanup_start = perf_counter()
    cpu_start = process_time()
    trace({"event": "stage_start", "stage": "cleanup"})
    owned.clear()
    tables.clear()
    readers.clear()
    mappings.clear()
    del reader, mapping
    cleanup = perf_counter() - cleanup_start
    trace({"event": "stage_end", "stage": "cleanup", "wall_seconds": cleanup,
           "cpu_seconds": process_time() - cpu_start,
           "peak_rss_mib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024})
    result = {"method": "hybrid", "input_matches": rows, "equal": equal,
                      "comparison": "gameplay fields with ordered arrays; unordered match records",
                      "excluded_metadata": {"root": DERIVED, "missions[]": DERIVED},
                      "stage_seconds": {**loading, **timings}, "detail": detail,
                      "through_result_seconds": through_result,
                      "cleanup_seconds": cleanup,
                      "including_cleanup_seconds": perf_counter() - total_start,
                      "imports_seconds": IMPORT_SECONDS,
                      "peak_rss_mib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024}
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2), flush=True)
    return 0 if equal else 1


if __name__ == "__main__":
    raise SystemExit(main())
