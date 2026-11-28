from collections import Counter
from hashlib import sha256
from itertools import product
import json
import os
from pathlib import Path
from shutil import copytree
import sys

import torch
from hex_command_rl.mixed_workload import configs, group
from hex_command_rl.recovery import atomic_json

out = Path(sys.argv[1]).resolve()
catalog = out / "catalog"
copytree("rust/data", catalog, dirs_exist_ok=True)
os.environ["HEX_COMMAND_DATA_ROOT"] = str(catalog)
import hex_command_training as native

spec = json.loads((catalog / "simulation/rl.json").read_text())
assert json.loads(native.training_workload()) == spec
for campaign in ("rl-woodland-defense", "rl-underground-escape", "rl-splitwake-crossing"):
    for path in (catalog / "game/campaigns" / campaign / "stages").glob("*/roster.json"):
        groups = {v["id"]: v["sight"]["penetrates"] for v in json.loads(path.read_text())["visionGroups"]}
        assert groups == {"player": [], "enemy": ["fog"]}, path

templates = {}
for case in configs("train", spec):
    if case["playerBlocks"]["recruiter"] != "predictor":
        continue
    k = group(case)
    cases = templates.setdefault(k, {})
    if k[0] == "arena":
        armies = case["armies"]
        signature = (armies["player"], armies["enemy"], case["first"])
        cases[signature] = case
    else:
        cases[()] = case
assert len(templates) == 102

train, validation = [], []
train_seed = 22000001
validation_seed = 23000001
for i, arena in enumerate(spec["arenas"]):
    name = arena["id"]
    pairs = [(pair["player"], pair["enemy"]) for pair in arena["pairs"]]
    for j, (objective, opponent) in enumerate(product(spec["objectives"], spec["opponents"])):
        group_templates = templates["arena", name, objective, opponent, "predictor"]
        for k, first in enumerate(("player", "player", "ai", "ai")):
            player, enemy = pairs[(i + 4 * j + k) % len(pairs)]
            train.append(dict(group_templates[player, enemy, first], seed=train_seed, maxRounds=100))
            train_seed += 1
        if j == (i % 2) * len(spec["opponents"]):
            for k, first in enumerate(("player", "player", "ai", "ai")):
                player, enemy = pairs[(i + 6 + k) % len(pairs)]
                validation.append(dict(group_templates[player, enemy, first], seed=validation_seed, maxRounds=100))
                validation_seed += 1

for key, group_templates in templates.items():
    if key[0] != "campaign":
        continue
    for k in range(4):
        train.append(dict(group_templates[()], seed=train_seed))
        train_seed += 1
    if key[-2] == "classic":
        for k in range(4):
            validation.append(dict(group_templates[()], seed=validation_seed))
            validation_seed += 1

for cases in (train, validation):
    assert len(cases) % 4 == 0
    assert all(len({group(c) for c in cases[i:i+4]}) == 1 for i in range(0, len(cases), 4))
    assert all(c["playerBlocks"] == {"recruiter": "predictor", "campaign": "mission", "planner": "queued"} for c in cases)
assert {c["seed"] for c in train}.isdisjoint(c["seed"] for c in validation)
assert {group(c)[1] for c in train} == {a["id"] for a in spec["arenas"]} | spec["campaigns"].keys()

warm = Path("training/checkpoints/campaign-cnn-ppo-v2/resume.pt")
checkpoint = torch.load(warm, map_location="cpu", weights_only=True)
torch.save(checkpoint["policy"], out / "v2-policy.pt")
summary = {
    "catalog": str(catalog),
    "fingerprints": json.loads(native.fingerprints()),
    "teacher": {"recruiter": "predictor", "planner": "queued", "campaign": "mission"},
    "warm_start": {"path": str(warm), "sha256": sha256(warm.read_bytes()).hexdigest()},
    "source_commit": (out / "source-commit.txt").read_text().strip(),
    "balance": "equal arena/campaign mass, then equal map or campaign and recorded variants within each family",
    "train": train,
    "validation": validation,
}
summary["workload_sha256"] = sha256(json.dumps(summary, sort_keys=True).encode()).hexdigest()
atomic_json(out / "workload.json", summary)
for label, cases in (("train", train), ("validation", validation)):
    print(label, len(cases), dict(Counter(group(c)[0] for c in cases)), flush=True)
print("fingerprints", summary["fingerprints"], flush=True)
