"""PPO-only mastery weighting within fixed, authored decision budgets."""

from collections import Counter, defaultdict
from copy import deepcopy
import hashlib
import json
import math

from .mixed_workload import definition, group, sampling_group, task, weights


def mastery_multiplier(completion, threshold, minimum):
    return max(minimum, min(1.0, (1.0 - completion) / (1.0 - threshold)))


class Sampler:
    """Balance decisions, reserving work for live episodes and retaining mastery."""

    SCHEMA = 3

    def __init__(self, configs, rng, state=None, *, spec=None):
        spec = definition() if spec is None else spec
        self.rng = rng
        self.settings = deepcopy(spec["adaptiveSampling"])
        self.templates = {}
        self.groups = {}
        self.buckets = {}
        masses = weights(configs, spec)
        for config in configs:
            base = deepcopy(config)
            base.pop("seed")
            conditions = next((a.get("conditions") for a in spec.get("arenas", [])
                               if a["id"] == base.get("landscape")), None)
            base["maxRounds"] = conditions["maxRounds"] if conditions else 100
            identity = json.dumps(base, sort_keys=True, separators=(",", ":"))
            key = hashlib.sha256(identity.encode()).hexdigest()
            self.templates[key] = base
            self.groups[key] = json.dumps(group(base))
            self.buckets[key] = sampling_group(base, spec)
        if not self.templates:
            raise ValueError("empty sampling workload")
        sizes = Counter(self.groups.values())
        self.base_weights = {
            k: masses[group(c)] / sizes[self.groups[k]]
            for k, c in self.templates.items()
        }
        self.bucket_weights = defaultdict(float)
        for k, value in self.base_weights.items():
            self.bucket_weights[self.buckets[k]] += value
        self.maps = {k: (self.buckets[k], task(c, spec)) for k, c in self.templates.items()}
        self.map_cases = defaultdict(list)
        for k, m in self.maps.items():
            self.map_cases[json.dumps(m)].append(k)
        self.map_keys = {k: json.dumps(m) for k, m in self.maps.items()}
        self.map_credits = {m: 0.0 for m in self.map_cases}
        self.case_credits = {k: 0.0 for k in self.templates}
        self.counts = {k: 0 for k in sizes}
        self.case_counts = {k: 0 for k in self.templates}
        self.allocation_counts = dict(self.case_counts)
        self.average = {
            k: 300.0 if "campaign" in c else 90.0
            for k, c in self.templates.items()
        }
        self.estimates = {
            k: dict(observations=0, completion=self.settings["initialCompletionRate"])
            for k in self.templates
        }
        self.multipliers = {k: 1.0 for k in self.templates}
        self.completed = 0
        self.recent = set()
        self.next_seed = 7000001
        # Live environments restart on resume; unfinished episodes supply no outcome.
        self.pending = {}
        if state is not None:
            if state.get("schema") not in (2, self.SCHEMA):
                raise ValueError("sampler checkpoint schema differs; use a fresh workload")
            if state["settings"] != self.settings:
                raise ValueError("adaptive sampling settings differ from checkpoint")
            for field in ("case_counts", "allocation_counts", "average", "estimates", "multipliers"):
                if set(state[field]) != set(self.templates):
                    raise ValueError("sampler checkpoint workload differs")
                setattr(self, field, deepcopy(state[field]))
            if set(state["counts"]) != set(self.counts):
                raise ValueError("sampler checkpoint groups differ")
            if state["schema"] == self.SCHEMA:
                for field in ("map_credits", "case_credits"):
                    values = state[field]
                    if set(values) != set(getattr(self, field)) or any(not math.isfinite(v) for v in values.values()):
                        raise ValueError("invalid checkpoint allocation credits")
                    setattr(self, field, deepcopy(values))
            # Schema 2 lifetime exposure remains evidence, not catch-up debt:
            # start prospective fair allocation when migrating the broken sampler.
            self.counts = deepcopy(state["counts"])
            self.completed = state["completed"]
            self.recent = set(state["recent"])
            self.next_seed = state["next_seed"]
            if any(not math.isfinite(v["completion"]) or not 0 <= v["completion"] <= 1
                   or v["observations"] < 0 for v in self.estimates.values()):
                raise ValueError("invalid checkpoint completion estimates")
            if any(not math.isfinite(v) or not self.settings["minimumMultiplier"] <= v <= 1
                   for v in self.multipliers.values()):
                raise ValueError("invalid checkpoint sampling multipliers")
            if any(not math.isfinite(v) or v <= 0 for v in self.average.values()):
                raise ValueError("invalid checkpoint episode lengths")
            if any(not isinstance(v, int) or v < 0
                   for counts in (self.counts, self.case_counts, self.allocation_counts)
                   for v in counts.values()):
                raise ValueError("invalid checkpoint allocation counters")
            if not self.recent.issubset(self.templates) or self.completed < 0:
                raise ValueError("invalid checkpoint sampling progress")
        self._reweight()

    def _reweight(self):
        raw = {k: self.base_weights[k] * self.multipliers[k] for k in self.templates}
        totals = Counter()
        for k, value in raw.items():
            totals[self.buckets[k]] += value
        self.weights = {
            k: value * self.bucket_weights[self.buckets[k]] / totals[self.buckets[k]]
            for k, value in raw.items()
        }
        self.map_weights = {m: sum(self.weights[k] for k in keys)
                            for m, keys in self.map_cases.items()}

    def choose(self, slot):
        if slot in self.pending:
            raise ValueError("sampling slot already has a live episode")
        bucket_loads = Counter()
        for k, value in self.case_counts.items():
            bucket_loads[self.buckets[k]] += value
        reserved_maps = Counter()
        reserved_cases = Counter()
        for key, remaining in self.pending.values():
            bucket_loads[self.buckets[key]] += remaining
            reserved_maps[self.map_keys[key]] += remaining
            reserved_cases[key] += remaining
        bucket = min(self.bucket_weights,
                     key=lambda b: bucket_loads[b] / self.bucket_weights[b])
        maps = [m for m, keys in self.map_cases.items() if self.buckets[keys[0]] == bucket]
        m = max(maps, key=lambda m: self.map_credits[m] +
                200 * self.map_weights[m] / self.bucket_weights[bucket] - reserved_maps[m] +
                float(self.rng.uniform(0, 1e-5)))
        key = max(self.map_cases[m], key=lambda k: self.case_credits[k] +
                  200 * self.weights[k] / self.map_weights[m] - reserved_cases[k] +
                  float(self.rng.uniform(0, 1e-5)))
        config = deepcopy(self.templates[key])
        config["seed"] = self.next_seed
        self.next_seed += 1
        self.pending[slot] = (key, self.average[key])
        return config, self.groups[key]

    def step(self, slot):
        key, remaining = self.pending[slot]
        m = self.map_keys[key]
        bucket = self.buckets[key]
        for other, keys in self.map_cases.items():
            if self.buckets[keys[0]] == bucket:
                self.map_credits[other] += self.map_weights[other] / self.bucket_weights[bucket]
        self.map_credits[m] -= 1
        for other in self.map_cases[m]:
            self.case_credits[other] += self.weights[other] / self.map_weights[m]
        self.case_credits[key] -= 1
        self.counts[self.groups[key]] += 1
        self.case_counts[key] += 1
        self.allocation_counts[key] += 1
        self.pending[slot] = (key, max(0.0, remaining - 1))

    def finish(self, slot, length, success=None, truncated=False):
        key, _ = self.pending.pop(slot)
        self.average[key] = 0.9 * self.average[key] + 0.1 * length
        if success is None:
            return None
        estimate = self.estimates[key]
        rate = self.settings["emaRate"]
        estimate["completion"] += rate * (float(bool(success) and not truncated) - estimate["completion"])
        estimate["observations"] += 1
        self.recent.add(key)
        self.completed += 1
        if self.completed % self.settings["updateEveryEpisodes"]:
            return None
        for k, value in self.estimates.items():
            eligible = self.settings["enabled"] and value["observations"] >= self.settings["warmupEpisodes"]
            self.multipliers[k] = mastery_multiplier(
                value["completion"], self.settings["completionThreshold"],
                self.settings["minimumMultiplier"],
            ) if eligible else 1.0
        self._reweight()
        # Credits integrate the weights in effect at each decision. Reweighting
        # must not erase unpaid map/condition exposure or restart a priority race.
        total_decisions = sum(self.case_counts.values())
        report = dict(completed=self.completed, cases=[self._row(k, total_decisions) for k in sorted(self.recent)])
        self.recent.clear()
        return report

    def _row(self, key, total):
        return dict(
            case=key, config=self.templates[key], bucket=self.buckets[key],
            completion_rate=self.estimates[key]["completion"],
            observations=self.estimates[key]["observations"],
            multiplier=self.multipliers[key], base_weight=self.base_weights[key],
            weight=self.weights[key], decisions=self.case_counts[key],
            decision_share=self.case_counts[key] / max(1, total),
        )

    def report(self):
        total = sum(self.case_counts.values())
        return dict(settings=self.settings, completed=self.completed,
                    cases=[self._row(k, total) for k in self.templates])

    def state(self):
        return deepcopy(dict(
            schema=self.SCHEMA, settings=self.settings, counts=self.counts,
            map_credits=self.map_credits, case_credits=self.case_credits,
            case_counts=self.case_counts, allocation_counts=self.allocation_counts,
            average=self.average, estimates=self.estimates,
            multipliers=self.multipliers, completed=self.completed,
            recent=sorted(self.recent), next_seed=self.next_seed,
        ))
