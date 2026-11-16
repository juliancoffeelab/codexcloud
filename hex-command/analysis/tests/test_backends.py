"""Small analyzer contracts: recorded roles and strict Arrow regression checks.

Run with the Polars project's Python. These checks do not simulate games.
"""

import copy
import csv
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import pyarrow as pa

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "analysis/polars"))
from journal import decode
from journal_test_fixture import arrow_detail


def dump(path, value):
    path.write_text(json.dumps(value))


def reference(path, value):
    raw = arrow_detail(value)
    path.write_bytes(raw)
    return dict(
        file=path.name, offset=0, bytes=len(raw), sha256=hashlib.sha256(raw).hexdigest()
    )


def role_fixture(directory):
    def unit(id, side, role):
        return dict(
            id=id,
            type="shared" if side == "player" else "foe",
            faction=side,
            role=role,
            tile=dict(col=0 if side == "player" else 1, row=0),
            hp=10,
        )

    leader, ordinary, foe = (
        unit("b", "player", "leader"),
        unit("t", "player", None),
        unit("e", "enemy", None),
    )
    units = [leader, ordinary, foe]
    tiles = [
        dict(id="0,0", col=0, row=0, terrain="plains", owner=None),
        dict(id="1,0", col=1, row=0, terrain="plains", owner=None),
    ]
    initial = dict(
        units=units,
        tiles=tiles,
        factions=[
            dict(id=s, phase="player" if s == "player" else "ai")
            for s in ("player", "enemy")
        ],
    )
    config = dict(
        seed=1,
        armies=dict(player="human", enemy="woodland"),
        player="balanced",
        enemy="balanced",
        landscape="open",
        first="player",
        maxRounds=30,
        recruitLimit=None,
    )

    def combat(actor, damage):
        event = dict(
            rawDamage=damage,
            afterArmourDamage=damage,
            afterTerrainDamage=damage,
            availableHp=10,
            effectiveDamage=damage,
            effectiveArmourPrevented=0,
            effectiveTerrainPrevented=0,
        )
        return dict(
            unit=actor,
            target="e",
            damage=damage,
            retaliation=0,
            retaliationCount=0,
            mitigation=dict(attack=event, retaliation=None),
            thorns=dict(damage=0, auraDamage=0, effectiveDamage=0, auraSources=[]),
        )

    phases = [
        dict(
            side=side,
            round=1,
            before=units,
            units=units,
            losses=[],
            actions=[],
            combats=[combat("b", 5), combat("t", 2)] if side == "player" else [],
            healing=[],
            ownership=[
                dict(tile=dict(col=t["col"], row=t["row"]), owner=None) for t in tiles
            ],
            leaderAudit=[],
        )
        for side in ("player", "enemy")
    ]
    result = dict(
        kind="arena",
        config=config,
        winner="player",
        reason="leader",
        rounds=1,
        initial=initial,
        final=initial,
        phases=phases,
    )
    ref = reference(directory / "details.arrow", result)
    outcome = dict(
        kind="arena",
        config=config,
        winner="player",
        reason="leader",
        rounds=1,
        stateHash="same",
        detail=ref,
        recruits=[],
        promotions=[],
        casualties=[],
        combatUsage=[
            dict(side="player", type="shared", attacks=2, retaliations=0, damage=7)
        ],
    )
    profiles = {
        u: dict(
            family=u,
            name=u,
            level=1,
            role="warrior",
            race=race,
            cost=5,
            hp=10,
            healing=0,
            thornAura=0,
            thorns=0,
        )
        for u, race in [("shared", "human"), ("foe", "woodland")]
    }
    # A stale type flag must not override either recorded instance role.
    profiles["shared"]["leader"] = True
    report = dict(
        detailFormat="arrow-ipc-snapshot-dictionaries-v5",
        status="complete",
        mode="arena",
        outcomes=[outcome],
        elapsedSeconds=1,
        unitProfiles=profiles,
        unitFamilies={u: u for u in profiles},
        combatTelemetryVersion=1,
        supportTelemetryVersion=1,
        leaderAuditVersion=1,
    )
    dump(directory / "report.json", report)
    return report, result


def commands(mode):
    result = [
        (
            [
                sys.executable,
                str(
                    ROOT
                    / "analysis/polars"
                    / ("main.py" if mode == "report" else "compare.py")
                ),
            ],
            "polars",
        )
    ]
    return result


class BackendContracts(unittest.TestCase):
    def run_command(self, command, succeeds=True):
        result = subprocess.run(
            command,
            text=True,
            capture_output=True,
            env={**os.environ, "POLARS_MAX_THREADS": "2"},
        )
        self.assertEqual(
            result.returncode == 0, succeeds, result.stdout + result.stderr
        )
        return result

    def test_empty_report_directory_is_not_a_successful_zero_game_run(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            for command, name in commands("report"):
                self.run_command(
                    command + [str(directory), "--out", str(directory / name)],
                    succeeds=False,
                )

    def test_instance_roles_survive_type_aggregation(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            role_fixture(directory)
            other = directory / "other"
            other.mkdir()
            second, detail = role_fixture(other)
            detail["config"]["landscape"] = "wooded"
            detail["initial"]["units"] = [
                u for u in detail["initial"]["units"] if u["id"] != "t"
            ]
            for phase in detail["phases"]:
                phase["before"] = detail["initial"]["units"]
                phase["units"] = detail["initial"]["units"]
                phase["combats"] = [c for c in phase["combats"] if c["unit"] != "t"]
            second["outcomes"][0]["detail"] = reference(other / "details.arrow", detail)
            second["outcomes"][0]["combatUsage"] = [
                dict(side="player", type="shared", attacks=1, retaliations=0, damage=5)
            ]
            dump(other / "report.json", second)
            backends = commands("report") + [
                (
                    [
                        sys.executable,
                        str(
                            ROOT
                            / ".agents/skills/combat-simulation/scripts/report-combat.py"
                        ),
                    ],
                    "legacy",
                )
            ]
            for command, name in backends:
                output = directory / name
                self.run_command(
                    command
                    + [
                        str(directory / "report.json"),
                        str(other / "report.json"),
                        "--out",
                        str(output),
                        "--unit-usage",
                    ]
                )
                with (output / "unit_damage.csv").open() as stream:
                    rows = list(csv.DictReader(stream))
                own = next(
                    r for r in rows if r["race"] == "human" and r["map"] == "all"
                )
                self.assertEqual(float(own["unitTurns"]), 1)
                self.assertEqual(float(own["weaponDamage"]), 2)
                self.assertEqual(float(own["damagePerUnitTurn"]), 2)
                empty_scope = next(
                    r for r in rows if r["race"] == "human" and r["map"] == "wooded"
                )
                self.assertEqual(float(empty_scope["unitTurns"]), 0)
                self.assertEqual(empty_scope["damagePerUnitTurn"], "")
                with (output / "nonleader_unit_events.csv").open() as stream:
                    events = list(csv.DictReader(stream))
                own = next(r for r in events if r["race"] == "human")
                self.assertEqual(float(own["attacks"]), 1)
                self.assertEqual(float(own["damage"]), 2)

    def test_report_comparison_checks_checksums_and_missing_fields(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            left, right = root / "left", root / "right"
            left.mkdir()
            right.mkdir()
            a, detail = role_fixture(left)
            b = copy.deepcopy(a)
            b["elapsedSeconds"] = 99
            b["outcomes"][0]["detail"] = reference(right / "details.arrow", detail)
            dump(right / "report.json", b)
            for command, _ in commands("compare"):
                args = command + [str(left / "report.json"), str(right / "report.json")]
                self.run_command(args)
                changed = copy.deepcopy(detail)
                changed["unexpectedGameplayField"] = None
                b["outcomes"][0]["detail"] = reference(right / "details.arrow", changed)
                dump(right / "report.json", b)
                self.run_command(args, succeeds=False)
                b["outcomes"][0]["detail"] = reference(right / "details.arrow", detail)
                dump(right / "report.json", b)
                (right / "details.arrow").write_bytes(b"corrupt")
                self.run_command(args, succeeds=False)
                b["outcomes"][0]["detail"] = reference(right / "details.arrow", detail)
                dump(right / "report.json", b)

    def test_journal_decoder_rejects_extra_batches_and_columns(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            schema = pa.schema(
                [
                    ("header_json", pa.string()),
                    ("chapter", pa.uint32()),
                    ("phase_json", pa.string()),
                    ("before", pa.list_(pa.uint32())),
                    ("units", pa.list_(pa.uint32())),
                    ("ownership", pa.uint32()),
                    (
                        "unit_snapshots",
                        pa.list_(
                            pa.struct(
                                [
                                    ("id", pa.string()),
                                    ("type", pa.string()),
                                    (
                                        "tile",
                                        pa.struct(
                                            [
                                                pa.field(
                                                    "col", pa.int32(), nullable=False
                                                ),
                                                pa.field(
                                                    "row", pa.int32(), nullable=False
                                                ),
                                            ]
                                        ),
                                    ),
                                    ("hp", pa.int32()),
                                    ("faction", pa.string()),
                                    ("role", pa.string()),
                                    ("character", pa.string()),
                                    ("rebuilding", pa.bool_()),
                                ]
                            )
                        ),
                    ),
                    ("ownership_snapshots", pa.list_(pa.list_(pa.string()))),
                ],
                metadata={
                    b"hex-command.detail-format": b"arrow-ipc-snapshot-dictionaries-v5"
                },
            )
            header = dict(
                kind="arena",
                config=dict(seed=1),
                initial=dict(tiles=[], units=[]),
                final=dict(units=[]),
                phases=[],
            )
            first = pa.RecordBatch.from_pylist(
                [
                    dict(
                        header_json=json.dumps(header),
                        chapter=0,
                        phase_json=None,
                        before=None,
                        units=None,
                        ownership=None,
                        unit_snapshots=[],
                        ownership_snapshots=[[]],
                    )
                ],
                schema=schema,
            )
            second = pa.RecordBatch.from_pylist(
                [
                    dict(
                        header_json=None,
                        chapter=0,
                        phase_json=json.dumps(dict(side="player")),
                        before=[],
                        units=[],
                        ownership=0,
                        unit_snapshots=None,
                        ownership_snapshots=None,
                    )
                ],
                schema=schema,
            )

            def write(batches):
                sink = pa.BufferOutputStream()
                with pa.ipc.new_stream(sink, batches[0].schema) as writer:
                    for batch in batches:
                        writer.write_batch(batch)
                raw = sink.getvalue().to_pybytes()
                (directory / "record.arrow").write_bytes(raw)
                return raw

            self.assertEqual(decode(write([first]), "record.arrow"), header)
            with self.assertRaises(ValueError):
                decode(write([first, second]), "record.arrow")
            with self.assertRaises(ValueError):
                decode(
                    write([first.append_column("unexpected", pa.array([1]))]),
                    "record.arrow",
                )

    def test_dump_comparison_preserves_array_order_and_nested_nulls(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            schema = pa.schema(
                [
                    ("kind", pa.string()),
                    ("config", pa.struct([("seed", pa.int64())])),
                    ("stateHash", pa.string()),
                    ("values", pa.list_(pa.int64())),
                    (
                        "missions",
                        pa.list_(
                            pa.struct(
                                [
                                    ("id", pa.string()),
                                    ("stateHash", pa.string()),
                                    ("round", pa.int64()),
                                ]
                            )
                        ),
                    ),
                ]
            )
            rows = [
                dict(
                    kind="arena",
                    config=dict(seed=i),
                    stateHash="old",
                    values=[1, 2],
                    missions=[None, dict(id="m", stateHash="old", round=1)]
                    if i
                    else None,
                )
                for i in range(2)
            ]

            def write(name, records):
                path = directory / name
                with pa.OSFile(str(path), "wb") as f:
                    with pa.ipc.new_file(f, schema) as writer:
                        writer.write_table(pa.Table.from_pylist(records, schema=schema))
                return path

            a = write("a.arrow", rows)
            changed = copy.deepcopy(rows[::-1])
            for row in changed:
                row["stateHash"] = "new"
                for mission in row["missions"] or []:
                    if mission:
                        mission["stateHash"] = "new"
            b = write("b.arrow", changed)
            args = [
                sys.executable,
                str(ROOT / "scripts/compare-simulation-dumps.py"),
                str(a),
                str(b),
            ]
            self.run_command(args)
            changed[0]["values"].reverse()
            write("b.arrow", changed)
            self.run_command(args, succeeds=False)


if __name__ == "__main__":
    unittest.main()
