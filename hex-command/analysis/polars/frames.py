"""Columnar reductions for batches of interpreted journal events."""

import polars as pl


def group(rows, keys, measures):
    if not rows:
        return []
    frame = pl.from_dicts(rows, infer_schema_length=None, strict=False)
    missing = [
        pl.lit(0, dtype=pl.Int64).alias(k) for k in measures if k not in frame.columns
    ]
    result = (
        frame.lazy()
        .with_columns(missing)
        .group_by(keys, maintain_order=False)
        .agg(pl.col(k).fill_null(0).sum().alias(k) for k in measures)
        .sort(keys, nulls_last=True)
        .collect()
    )
    return result.to_dicts()


class Accumulator:
    """Bound memory while retaining columnar group-by, rather than row counters."""

    def __init__(self, keys, measures):
        self.keys, self.measures = list(keys), list(measures)
        self.pending, self.chunks = [], []

    def add(self, row):
        self.pending.append(row)
        if len(self.pending) >= 8192:
            self.flush()

    def flush(self):
        if self.pending:
            self.chunks.extend(group(self.pending, self.keys, self.measures))
            self.pending.clear()

    def finish(self):
        self.flush()
        return group(self.chunks, self.keys, self.measures)
