"""Typed table helpers. Row conversion belongs only to presentation."""

import polars as pl


def empty(keys, measures=()):
    return pl.DataFrame(
        schema={**{k: pl.String for k in keys}, **{k: pl.Float64 for k in measures}}
    )


def aggregate(rows, keys, measures):
    if isinstance(rows, pl.DataFrame):
        frame = rows
    elif rows:
        frame = pl.from_dicts(rows, infer_schema_length=None, strict=False)
    else:
        return empty(keys, measures)
    missing = [pl.lit(0).alias(k) for k in measures if k not in frame.columns]
    return (
        frame.with_columns(missing)
        .group_by(keys)
        .agg(pl.col(k).fill_null(0).sum().alias(k) for k in measures)
        .sort(keys, nulls_last=True)
    )


def group(rows, keys, measures):
    """Small presentation-only reductions used by Markdown rendering."""
    return aggregate(rows, keys, measures).to_dicts()
