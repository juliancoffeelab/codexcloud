#!/usr/bin/env bash
set -euo pipefail
source /workspace/.setup/env.sh
cd /workspace/codexcloud/hex-command
run="$PWD/tmp/rl/ppo-from-pretrain-43m-9h-20261121"
pretrain="$PWD/tmp/rl/v2-predictor-queued-pretrain-1h"
warm="$pretrain/posttune/checkpoints/epoch-0006-1795199024586818373/resume.pt"
py="$PWD/tmp/rl/runtime/venv/bin/python"
export PYTHONPATH="$run/native:$PWD/training"
export MPLCONFIGDIR="$run/mpl" MPLBACKEND=Agg
export PYTHONUNBUFFERED=1 OPENBLAS_NUM_THREADS=1 MKL_NUM_THREADS=1
if [[ ! -f "$run/workload.json" ]]; then
  "$py" -m hex_command_rl.diverse_workload --root "$run"
fi
"$py" - "$run" "$pretrain" "$warm" <<'PY'
import hashlib
import json
from pathlib import Path
import sys
from hex_command_rl.recovery import atomic_json, load_resume

run, origin, checkpoint = map(Path, sys.argv[1:])
read = lambda p: json.loads(p.read_text())
spec = read(run / "workload.json")
source = read(origin / "workload.json")
catalog = Path(spec["catalog"])
manifest = read(checkpoint.with_name("manifest.json"))
digest = hashlib.sha256(checkpoint.read_bytes()).hexdigest()
assert manifest["label"] == "epoch-0006"
assert digest == manifest["files"]["resume.pt"]["sha256"]
warm = load_resume(checkpoint)
assert warm["kind"] == "mixed_imitation" and warm["progress"]["epoch"] == 6
assert warm["fingerprints"] == spec["fingerprints"] == source["fingerprints"]
assert warm["workload_sha256"] == source["workload_sha256"]
assert 42 * 60 <= warm["elapsed"] < 45 * 60
campaigns = read(catalog / "simulation/rl.json")["campaigns"]
assert set(campaigns) == {"rl-woodland-defense", "rl-underground-escape", "rl-splitwake-crossing"}
matches = spec["train"]
assert len({m["landscape"] for m in matches if "landscape" in m}) == 14
assert len(matches) == 33120
routes = read(catalog / "simulation/campaign.json")["campaigns"]
for campaign in campaigns:
    chapter_paths = list((catalog / "game/campaigns" / campaign / "stages").glob("*/roster.json"))
    assert len(chapter_paths) == 3
    for path in chapter_paths:
        roster = read(path)
        assert {v["id"]:v["sight"]["penetrates"] for v in roster["visionGroups"]} == {"player": [], "enemy": ["fog"]}, path
    assert len(routes[campaign]["routes"]["fog"]["stages"]) == 3
    for chapter in routes[campaign]["routes"]["fog"]["stages"].values():
        assert any(a.get("addConcealment", {}).get("kind") == "fog"
                   for e in chapter["scriptEvents"] for a in e["actions"])
assert all(m["seed"] < 26000001 for m in matches + spec["validation"] + spec["test"])
atomic_json(run / "launch-verification.json", {
    "checkpoint": str(checkpoint), "sha256": digest, "pretrain_seconds": warm["elapsed"],
    "fingerprints": spec["fingerprints"], "train_cases": len(matches),
    "fog": {campaign: "player only" for campaign in campaigns},
    "periodic_cases": len(spec["periodic"]), "validation_cases": len(spec["validation"]),
    "showcase_cases": len(spec["test"]), "ppo_budget_seconds": 32400,
    "rollout_seed_start": 26000001,
})
print("Verified checkpoint, content, training workload, and fog", flush=True)
PY
exec "$py" -m hex_command_rl.diverse_run --root "$run" --warm-start "$warm" --seconds 32400 --seed-start 26000001
