# /// script
# requires-python = ">=3.11"
# dependencies = ["pyarrow==25.0.1", "polars==2.0.0"]
# ///
"""Analyze saved simulation outcomes; never run matches or read gameplay traces.

npm run ai:analyze -- artifacts/current/report.json \
    --baseline artifacts/previous/report.json --out artifacts/analytics

A report path selects its adjacent .outcomes.jsonl journal. Explicit-schema
PyArrow reads the JSON; Polars performs all per-match calculations and grouping.
Python only orchestrates stages and formats the small, aggregated output tables.
"""

import argparse
from contextlib import contextmanager
import json
from pathlib import Path
import resource
from time import perf_counter, process_time

IMPORT_STARTED = perf_counter()
IMPORT_CPU_STARTED = process_time()
import polars as pl
import pyarrow as pa
import pyarrow.json as arrow_json

from simulation_dump_schema import CONFIG, MISSION

IMPORT_SECONDS = perf_counter() - IMPORT_STARTED
IMPORT_CPU_SECONDS = process_time() - IMPORT_CPU_STARTED
IMPORT_RSS_MIB = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024

# Project just the compact outcome fields needed by analytics. Extra fields
# (casualties, recruits, hashes, trace references) are deliberately not loaded.
OUTCOME_SCHEMA = pa.struct([
    ("kind", pa.string()), ("config", CONFIG), ("winner", pa.string()),
    ("reason", pa.string()), ("rounds", pa.int64()), ("completed", pa.bool_()),
    ("missions", pa.list_(pa.struct([
        MISSION.field(name) for name in ("id", "winner", "round")
    ]))),
])
JOURNAL_SCHEMA = pa.schema([
    ("index", pa.int64()), ("outcome", OUTCOME_SCHEMA),
])


@contextmanager
def timed(name, timings):
    """Record wall/CPU time and process peak RSS for one actual pipeline stage."""
    wall, cpu = perf_counter(), process_time()
    yield
    timings[name] = {
        "wallSeconds": perf_counter() - wall,
        "cpuSeconds": process_time() - cpu,
        "peakRssMiB": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss / 1024,
    }


def journal_path(path):
    """Accept an outcome journal or the report.json beside it, including shards."""
    if path.suffix == ".json":
        path = path.with_suffix(".outcomes.jsonl")
    if not path.name.endswith(".outcomes.jsonl"):
        raise ValueError(f"Expected report.json or *.outcomes.jsonl: {path}")
    return path


def read_outcomes(paths, timings, label):
    """Return native Polars columns from projected, explicitly typed Arrow JSON."""
    with timed(f"{label}.arrowReadJson", timings):
        tables = [arrow_json.read_json(
            str(journal_path(path)),
            read_options=arrow_json.ReadOptions(block_size=8 * 1024 * 1024),
            parse_options=arrow_json.ParseOptions(
                explicit_schema=JOURNAL_SCHEMA, unexpected_field_behavior="ignore",
            ),
        ) for path in paths]
        table = pa.concat_tables(tables)
    with timed(f"{label}.arrowToPolars", timings):
        frame = pl.from_arrow(table).select(pl.col("outcome").struct.unnest())
    with timed(f"{label}.validateAndExpand", timings):
        if not frame.height:
            raise ValueError("No saved outcomes")
        if frame.select("kind", "config").is_duplicated().any():
            raise ValueError("Duplicate (kind, config) match identities")
        if frame.filter(
            pl.col("kind").is_null() | ~pl.col("kind").is_in(["arena", "campaign"])
            | pl.col("config").is_null()
        ).height:
            raise ValueError("Missing match identity or unknown match kind")
        config = pl.col("config")
        frame = frame.with_columns(
            *[config.struct.field(field).alias(alias) for field, alias in [
                ("seed", "seed"), ("player", "playerPolicy"),
                ("enemy", "enemyPolicy"), ("landscape", "map"),
                ("first", "first"), ("campaign", "campaign"),
                ("diplomacy", "diplomacy"),
            ]],
            config.struct.field("armies").struct.field("player").alias("playerRace"),
            config.struct.field("armies").struct.field("enemy").alias("enemyRace"),
        )
        required = ["seed", "playerPolicy", "enemyPolicy"]
        if frame.select(pl.any_horizontal(pl.col(required).is_null()).any()).item():
            raise ValueError("Missing seed or policy")
        arenas = frame.filter(pl.col("kind") == "arena")
        if arenas.select(pl.any_horizontal(
            pl.col("playerRace", "enemyRace", "map", "first", "rounds").is_null()
        ).any()).item():
            raise ValueError("Incomplete arena outcome")
        campaigns = frame.filter(pl.col("kind") == "campaign")
        if campaigns.select(pl.any_horizontal(
            pl.col("campaign", "diplomacy", "completed", "missions").is_null()
        ).any()).item() or campaigns.filter(pl.col("missions").list.len() == 0).height:
            raise ValueError("Incomplete campaign outcome")
    return frame


def win_counts(frame, keys):
    """Count both arena seats; capped games remain in the win-rate denominator."""
    return frame.group_by(keys).agg(
        pl.len().alias("games"), pl.col("won").sum().alias("wins"),
        pl.col("lost").sum().alias("losses"), pl.col("capped").sum().alias("caps"),
    ).with_columns((100 * pl.col("wins") / pl.col("games")).alias("winRate"))


def campaign_counts(frame, keys):
    """Later missions contribute only when reached; terminal caps are not defeats."""
    return frame.group_by(keys).agg(
        pl.len().alias("attempts"), pl.col("completed").sum().alias("completed"),
        (pl.col("terminalWinner") == "enemy").sum().alias("defeated"),
        pl.col("terminalWinner").is_null().sum().alias("caps"),
    ).with_columns(
        (100 * pl.col("completed") / pl.col("attempts")).alias("completionRate"),
    )


def analyze(frame):
    """Compute results with native Polars expressions, joins and aggregations."""
    arena = frame.filter(pl.col("kind") == "arena")
    seats = []
    for side, opponent in [("player", "enemy"), ("enemy", "player")]:
        seats.append(arena.select(
            "seed", "map", "first", "rounds",
            pl.lit(side).alias("seat"),
            pl.col(f"{side}Race").alias("race"),
            pl.col(f"{opponent}Race").alias("opponentRace"),
            pl.col(f"{side}Policy").alias("policy"),
            pl.col(f"{opponent}Policy").alias("opponentPolicy"),
            (pl.col("winner") == side).fill_null(False).alias("won"),
            (pl.col("winner") == opponent).fill_null(False).alias("lost"),
            pl.col("winner").is_null().alias("capped"),
        ))
    participants = pl.concat(seats)
    cross = participants.filter(pl.col("race") != pl.col("opponentRace"))
    equal_policy = cross.filter(pl.col("policy") == pl.col("opponentPolicy"))
    campaigns = frame.filter(pl.col("kind") == "campaign").with_columns(
        pl.col("missions").list.last().struct.field("winner").alias("terminalWinner"),
    )
    stages = campaigns.select(
        "campaign", "playerPolicy", "enemyPolicy", "diplomacy", "missions",
    ).with_columns(
        pl.int_ranges(1, pl.col("missions").list.len() + 1).alias("stageNumber"),
    ).explode("missions", "stageNumber").filter(pl.col("missions").is_not_null()).with_columns(
        pl.col("missions").struct.field("id").alias("stage"),
        pl.col("missions").struct.field("winner").alias("stageWinner"),
        pl.col("missions").struct.field("round").alias("rounds"),
    )
    stage_counts = stages.group_by("campaign", "stageNumber", "stage").agg(
        pl.len().alias("reached"),
        (pl.col("stageWinner") == "player").sum().alias("completed"),
        (pl.col("stageWinner") == "enemy").sum().alias("defeated"),
        pl.col("stageWinner").is_null().sum().alias("caps"),
        pl.col("rounds").mean().alias("meanRounds"),
    ).with_columns(
        (100 * pl.col("completed") / pl.col("reached")).alias("completionRate"),
    )
    return {
        "race": (win_counts(cross, ["race"]), ["race"]),
        "race_map": (win_counts(cross, ["race", "map"]), ["race", "map"]),
        "race_equal_policy": (win_counts(equal_policy, ["race"]), ["race"]),
        "policy": (win_counts(participants, ["policy"]), ["policy"]),
        "race_policy": (win_counts(participants, ["race", "policy"]), ["race", "policy"]),
        "race_matchups": (win_counts(cross, ["race", "opponentRace", "policy", "opponentPolicy", "map"]),
                          ["race", "opponentRace", "policy", "opponentPolicy", "map"]),
        "seats_equal_policy": (win_counts(equal_policy, ["race", "policy", "seat", "first"]),
                               ["race", "policy", "seat", "first"]),
        "victories": (arena.group_by("reason").agg(pl.len().alias("games")), ["reason"]),
        "turns": (arena.group_by("map").agg(
            pl.len().alias("games"), pl.col("rounds").mean().alias("meanRounds"),
            pl.col("rounds").median().alias("medianRounds"),
            pl.col("rounds").quantile(0.9, interpolation="nearest").alias("p90Rounds"),
            pl.col("winner").is_null().sum().alias("caps"),
        ), ["map"]),
        "turn_distribution": (arena.group_by("map", "rounds").agg(pl.len().alias("games")),
                              ["map", "rounds"]),
        "campaigns": (campaign_counts(campaigns, ["campaign"]), ["campaign"]),
        "campaign_policies": (campaign_counts(campaigns, ["campaign", "playerPolicy"]),
                              ["campaign", "playerPolicy"]),
        "campaign_matchups": (campaign_counts(campaigns, ["campaign", "playerPolicy", "enemyPolicy", "diplomacy"]),
                              ["campaign", "playerPolicy", "enemyPolicy", "diplomacy"]),
        "stages": (stage_counts, ["campaign", "stageNumber", "stage"]),
    }


def compare(current, baseline, keys):
    """Join aggregate rows and compute numeric deltas in Polars (rates in pp)."""
    result = current.join(baseline, on=keys, how="full", coalesce=True, suffix="Before")
    metrics = [name for name in current.columns if name not in keys]
    return result.with_columns(*[
        (pl.col(name).cast(pl.Float64) - pl.col(name + "Before").cast(pl.Float64))
        .alias(name + "Delta") for name in metrics
    ])


def markdown_table(frame):
    """Format aggregate rows only; gameplay records never become Python objects."""
    def cell(value):
        if value is None:
            return "—"
        text = f"{value:.2f}" if isinstance(value, float) else str(value)
        return text.replace("|", "\\|").replace("\n", " ")
    lines = ["| " + " | ".join(frame.columns) + " |",
             "| " + " | ".join("---" for _ in frame.columns) + " |"]
    lines.extend("| " + " | ".join(map(cell, row)) + " |" for row in frame.iter_rows())
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", type=Path, nargs="+")
    parser.add_argument("--baseline", type=Path, action="append", default=[])
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    started = perf_counter()
    timings = {"imports": {
        "wallSeconds": IMPORT_SECONDS, "cpuSeconds": IMPORT_CPU_SECONDS,
        "peakRssMiB": IMPORT_RSS_MIB,
    }}
    current = read_outcomes(args.inputs, timings, "current")
    baseline = read_outcomes(args.baseline, timings, "baseline") if args.baseline else None
    if baseline is not None:
        with timed("matchConfigurations", timings):
            keys = ["kind", "config"]
            if current.height != baseline.height or current.join(baseline, on=keys, how="anti").height:
                raise ValueError("Baseline configurations differ; refusing an unmatched comparison")
    with timed("polarsAnalytics", timings):
        tables = analyze(current)
        previous = analyze(baseline) if baseline is not None else None
    with timed("polarsComparisons", timings):
        comparisons = {
            name: compare(table, previous[name][0], keys)
            for name, (table, keys) in tables.items()
        } if previous else {}
    with timed("writeReports", timings):
        args.out.mkdir(parents=True, exist_ok=True)
        lines = ["# Simulation analytics", "", f"Saved matches: **{current.height:,}**.", "",
                 "Race tables exclude self-play; policy tables include both seats of self-play. "
                 "Win rates include capped matches in the denominator. Rates are percentages, "
                 "and rate deltas are percentage points. Campaign stages count only attempts "
                 "that reached them. P90 uses the nearest quantile. No simulations were run.", ""]
        lines += ["Inputs: " + ", ".join(f"`{journal_path(path)}`" for path in args.inputs), ""]
        if baseline is not None:
            lines += ["Matched baseline: " + ", ".join(
                f"`{journal_path(path)}`" for path in args.baseline
            ), ""]
        for name, (table, keys) in tables.items():
            table = table.sort(keys)
            table.write_csv(args.out / f"{name}.csv")
            if name in comparisons:
                comparisons[name].sort(keys).write_csv(args.out / f"{name}_comparison.csv")
            # Detailed strata and histograms remain in CSV; keep the report readable.
            if name not in ("race_matchups", "seats_equal_policy", "turn_distribution", "campaign_matchups"):
                lines += [f"## {name.replace('_', ' ').title()}", "", markdown_table(table), ""]
                if name in comparisons:
                    metric = "winRate" if "winRate" in table.columns else "completionRate" if "completionRate" in table.columns else None
                    if metric:
                        compact = comparisons[name].select(*keys, metric + "Before", metric, metric + "Delta").sort(keys)
                        lines += ["Baseline comparison:", "", markdown_table(compact), ""]
        (args.out / "summary.md").write_text("\n".join(lines))
    result = {
        "inputs": [str(journal_path(path)) for path in args.inputs],
        "baseline": [str(journal_path(path)) for path in args.baseline],
        "matches": current.height, "matchedConfigurations": baseline is not None,
        "pyarrow": pa.__version__, "polars": pl.__version__,
        "stages": timings, "pipelineWallSeconds": perf_counter() - started,
        "totalWallSeconds": perf_counter() - IMPORT_STARTED,
    }
    (args.out / "timings.json").write_text(json.dumps(result, indent=2) + "\n")
    print(f"Analyzed {current.height:,} saved matches in {result['pipelineWallSeconds']:.3f}s "
          f"({result['totalWallSeconds']:.3f}s including imports); {args.out / 'summary.md'}")
    for name, timing in timings.items():
        print(f"  {name}: {timing['wallSeconds']:.3f}s")


if __name__ == "__main__":
    main()
