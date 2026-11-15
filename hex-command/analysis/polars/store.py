from journal import paths, read_json, normalize, identity
from dataset import Dataset
from campaign_tables import observations

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


def load(inputs, *, telemetry=False):
    filenames = paths(inputs)
    if inputs and not filenames:
        raise ValueError("No recorded reports in the selected inputs")
    outcomes, sources, reports = {}, [], []
    for path in filenames:
        raw = read_json(path)
        if path.name == "outcomes.json":
            metadata = path.parent.parent / "metadata.json"
            raw = dict(
                read_json(metadata) if metadata.exists() else {},
                outcomes=raw,
                mode="campaign",
            )
        report = normalize(raw)
        reports.append((path, report))
        source = {k: report.get(k) for k in SOURCE_FIELDS}
        source.update(
            fingerprint=report.get("fingerprint", report.get("fingerprints")),
            file=str((path.parent if path.name == "outcomes.json" else path).resolve()),
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
            outcomes[key] = o
    dataset = Dataset(reports, telemetry=telemetry)
    diagnostics = observations(dataset)
    for key, outcome in outcomes.items():
        if outcome["kind"] == "campaign":
            outcome["analysis"] = diagnostics.get(key, [])
    return dict(
        outcomes=list(outcomes.values()),
        sources=sources,
        reports=reports,
        dataset=dataset,
    )
