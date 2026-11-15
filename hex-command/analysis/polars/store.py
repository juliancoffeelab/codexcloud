from journal import paths, read_json, normalize, identity, Readers
import telemetry
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


def load(
    inputs,
    *,
    unit_usage=False,
    presence=False,
    support=False,
    value=False,
    leaders=False,
):
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
    flags = dict(
        presence_=unit_usage or presence,
        support_=unit_usage or support,
        value_=value
        or (unit_usage and all(s.get("combatTelemetryVersion") == 1 for s in sources)),
        leaders_=leaders
        or (unit_usage and all(s.get("leaderAuditVersion") == 1 for s in sources)),
    )
    detailed = any(flags.values())
    if unit_usage and any(s.get("combatTelemetryVersion") != 1 for s in sources):
        raise ValueError("Damage tables require fresh combat telemetry")
    catalogs = [s.get("unitProfiles") or {} for s in sources]
    if detailed and any(p != catalogs[0] for p in catalogs):
        raise ValueError("Cannot pool telemetry across different recorded catalogs")
    diagnostics, partials = {}, []
    with Readers() as readers:
        for path, report in reports:
            if detailed and report.get("status") != "complete":
                raise ValueError("Telemetry requires a completed run")
            selected = sorted(
                (
                    o
                    for o in report["outcomes"]
                    if o["kind"] == "campaign"
                    or detailed
                    and o["config"]["armies"]["player"]
                    != o["config"]["armies"]["enemy"]
                ),
                key=lambda o: (
                    o.get("detail", {}).get("file", ""),
                    o.get("detail", {}).get("offset", 0),
                ),
            )
            # Unindexed historical JSONL is decoded once as a whole source.
            size = max(1, len(selected)) if path.name == "outcomes.json" else 32
            for offset in range(0, max(1, len(selected)), size):
                batch = dict(report, outcomes=selected[offset : offset + size])
                data = Dataset([(path, batch)], telemetry=detailed, readers=readers)
                diagnostics.update(observations(data))
                if detailed:
                    partials.append(telemetry.analyze(data, **flags))
                del data
        if detailed and not partials:
            partials.append(telemetry.analyze(Dataset([], readers=readers), **flags))
    for key, outcome in outcomes.items():
        if outcome["kind"] == "campaign":
            outcome["analysis"] = diagnostics.get(key, [])
    return dict(
        outcomes=list(outcomes.values()),
        sources=sources,
        reports=reports,
        telemetry=telemetry.merge(partials) if detailed else {},
    )
