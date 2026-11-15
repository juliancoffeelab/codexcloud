from journal import paths, read_json, normalize, identity, decode, read_bytes
from campaign import diagnostics
import gzip
import json

SOURCE_FIELDS = [
    "sourceCommit",
    "fingerprint",
    "status",
    "mode",
    "schedule",
    "seeds",
    "elapsedSeconds",
    "detailFormat",
    "turnTiming",
    "recruitmentLimits",
    "recruitmentProfiles",
    "unitFamilies",
    "unitProfiles",
    "supportTelemetryVersion",
    "combatTelemetryVersion",
    "leaderAuditVersion",
]


def load(inputs):
    outcomes, sources, reports = {}, [], []
    for path in paths(inputs):
        raw = read_json(path)
        legacy = {}
        if path.name == "outcomes.json":
            metadata = path.parent.parent / "metadata.json"
            raw = dict(
                read_json(metadata) if metadata.exists() else {},
                outcomes=raw,
                mode="campaign",
            )
            journal = path.parent / "details.jsonl.gz"
            if journal.exists():
                with gzip.open(journal, "rt") as f:
                    for line in f:
                        o = normalize(json.loads(line))
                        o.setdefault("kind", "campaign" if "missions" in o else "arena")
                        legacy[identity(o)] = [
                            diagnostics(m) for m in o.get("missions", [])
                        ]
        report = normalize(raw)
        reports.append((path, report))
        source = {k: report.get(k) for k in SOURCE_FIELDS}
        source.update(
            fingerprint=report.get("fingerprint", report.get("fingerprints")),
            file=str(path.resolve()),
            matches=len(report["outcomes"]),
        )
        sources.append(source)
        for o in report["outcomes"]:
            o.setdefault("kind", "campaign" if "campaign" in o["config"] else "arena")
            key = identity(o)
            if key in outcomes:
                raise ValueError("Duplicate match configuration: " + key)
            if o["kind"] == "arena":
                if o.get("winner") not in (None, "player", "enemy") or (
                    (o.get("winner") is None) != (o.get("reason") == "cap")
                ):
                    raise ValueError("Invalid winner/cap consistency: " + key)
            else:
                if key in legacy:
                    o["analysis"] = legacy[key]
                else:
                    ref = o.get("detail", {})
                    if all(k in ref for k in ("file", "offset", "bytes", "sha256")):
                        result = normalize(
                            decode(read_bytes(path.parent, ref), ref["file"])
                        )
                        if identity(result) != key:
                            raise ValueError("Journal configuration mismatch")
                    else:
                        result = o
                    o["analysis"] = [diagnostics(m) for m in result.get("missions", [])]
            outcomes[key] = o
    return dict(outcomes=list(outcomes.values()), sources=sources, reports=reports)
