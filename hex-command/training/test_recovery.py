import tempfile
import unittest
from pathlib import Path
import torch
from hex_command_rl.recovery import save_bundle, resolve, load_resume
from hex_command_rl.spatial import SpatialPlanner
from hex_command_rl.mixed_workload import configs, group, mass, weights, sampling_group, definition


class RecoveryContracts(unittest.TestCase):
    def test_bundle_and_corruption_detection(self):
        with tempfile.TemporaryDirectory() as d:
            policy = SpatialPlanner()
            save_bundle(d, dict(value=torch.tensor([3.0])), policy, "test", best=True)
            self.assertEqual(float(load_resume(d)["value"][0]), 3.0)
            original = (Path(d) / "latest.json").read_bytes()
            with self.assertRaises(ValueError):
                save_bundle(
                    d, dict(value=torch.tensor([float("nan")])), policy, "invalid"
                )
            self.assertEqual((Path(d) / "latest.json").read_bytes(), original)
            folder = resolve(d, "best.json")
            (folder / "resume.pt").write_bytes(b"broken")
            with self.assertRaises(ValueError):
                resolve(d)

    def test_workload_disjoint_and_hierarchically_balanced(self):
        partitions = [configs(s) for s in ["train", "validation", "test"]]
        seeds = [{c["seed"] for c in cases} for cases in partitions]
        for i in range(3):
            for j in range(i):
                self.assertFalse(seeds[i] & seeds[j])
        unique = {group(c): c for c in partitions[0]}
        self.assertAlmostEqual(sum(mass(c) for c in unique.values()), 1.0)
        self.assertAlmostEqual(
            sum(mass(c) for c in unique.values() if "campaign" in c),
            1 - definition()["samplingWeights"]["arena"]
        )

        for bucket, budget in definition()["samplingWeights"].items():
            self.assertAlmostEqual(sum(mass(c) for c in unique.values() if sampling_group(c) == bucket), budget)
        self.assertEqual(set(unique), set(weights(partitions[0])))

    def test_pretrain_selector_does_not_change_rl_matches(self):
        from copy import deepcopy
        from hex_command_rl.mixed_workload import pretrain_imitator

        spec = deepcopy(definition())
        before = configs("validation", spec)
        sample = next(c for c in before if "campaign" not in c)
        arena = next(a for a in spec["arenas"] if a["id"] == sample["landscape"])
        previous = pretrain_imitator(sample, spec)
        arena["pretrain_imitator"] = "full" if previous != "full" else "classic"
        self.assertNotEqual(pretrain_imitator(sample, spec), previous)
        self.assertEqual(configs("validation", spec), before)

    def test_gameplay_holdouts_have_weights_but_no_imitation_teacher(self):
        from hex_command_rl.mixed_workload import pretrain_imitator
        arena = next(c for c in configs("validation") if "campaign" not in c)
        heldout = dict(campaign="__held_out_campaign__", objective="authored",
                       diplomacy="unrestricted", player="balanced", enemy="classic")
        result = weights([arena, heldout])
        self.assertAlmostEqual(result[group(arena)], 0.5)
        self.assertAlmostEqual(result[group(heldout)], 0.5)
        with self.assertRaises(KeyError):
            pretrain_imitator(heldout)

    def test_campaign_opponent_override_preserves_task_budget(self):
        from copy import deepcopy
        from unittest.mock import patch
        from hex_command_rl.mixed_workload import definition, variant_opponents

        spec = deepcopy(definition())
        campaign = next(iter(spec["campaigns"]))
        before = configs("train", spec)
        for variant in spec["campaigns"][campaign]:
            variant["opponents"] = ["classic"]
        for split in ("train", "validation", "test"):
            selected = configs(split, spec)
            target = [c for c in selected if c.get("campaign") == campaign]
            self.assertTrue(target)
            self.assertTrue(all(c["enemy"] == "classic" and "enemyBlocks" not in c for c in target))
        after = configs("train", spec)
        self.assertEqual(
            [c for c in before if c.get("campaign") != campaign],
            [c for c in after if c.get("campaign") != campaign],
        )
        unique = {group(c): c for c in after}
        with patch("hex_command_rl.mixed_workload.definition", return_value=spec):
            self.assertAlmostEqual(sum(mass(c) for c in unique.values()), 1.0)
            self.assertAlmostEqual(
                sum(mass(c) for c in unique.values() if c.get("campaign") == campaign),
                sum(weights(before)[group(c)] for c in {group(c): c for c in before if c.get("campaign") == campaign}.values()),
            )
        for invalid in ([], ["unknown"], ["classic", "classic"]):
            with self.assertRaises(ValueError):
                variant_opponents(spec, {"opponents": invalid})

    def test_sampler_budgets_follow_decisions_and_reserves_validation_seeds(self):
        import numpy as np
        from hex_command_rl.mixed_ppo import Sampler

        sampler = Sampler(configs("train"), np.random.default_rng(9))
        for _ in range(200):
            c, k = sampler.choose(0)
            self.assertGreaterEqual(c["seed"], 7000001)
            n = 100 if "campaign" in c else 10
            for i in range(n):
                sampler.step(0)
            sampler.finish(0, n)
        totals = {
            f: sum(
                n
                for k, n in sampler.counts.items()
                if __import__("json").loads(k)[0] == f
            )
            for f in ["arena", "campaign"]
        }
        self.assertLess(abs(totals["campaign"] / sum(totals.values()) - (1 - definition()["samplingWeights"]["arena"])), 0.08)

    def test_caps_are_failures_with_family_specific_penalties(self):
        from hex_command_rl.mixed_ppo import terminal_reward

        self.assertEqual(terminal_reward({"campaign": "test"}, 0.0, True), -2.0)
        for reason, value in (
            ("escort_loss", -2.0),
            ("protected_loss", -1.5),
            ("leader_loss", -1.0),
            ("defeat", -1.0),
        ):
            self.assertEqual(
                terminal_reward({"campaign": "test"}, -1.0, False, {"failure": reason}),
                value,
            )
            self.assertEqual(
                terminal_reward({"campaign": "test"}, 1.0, False, {"failure": reason}),
                1.0,
            )
        self.assertEqual(terminal_reward({"landscape": "test"}, 0.0, True), -1.0)
        self.assertEqual(terminal_reward({"campaign": "test"}, 1.0, False), 1.0)

    def test_selection_rejects_predictor_collapse_despite_high_pooled_score(self):
        from hex_command_rl.mixed_evaluate import reliability

        def report(wins):
            groups = []
            for family in ["arena", "campaign"]:
                for enemy in ["classic", "predictor"]:
                    key = (
                        ["arena", "open", "deathmatch", enemy, "predictor"]
                        if family == "arena"
                        else [
                            "campaign",
                            "lantern-road",
                            "authored",
                            "unrestricted",
                            enemy,
                            "predictor",
                        ]
                    )
                    groups.append(
                        dict(
                            group=key,
                            wins=wins[len(groups)],
                            runs=10,
                            caps=0,
                            mass=0.25,
                        )
                    )
            return dict(groups=groups, score=sum(wins) / 40, weighted_caps=0.0)

        collapsed = report([10, 10, 10, 0])
        balanced = report([4, 4, 4, 4])
        self.assertGreater(reliability(balanced)[0], reliability(collapsed)[0])

    def test_handoff_preserves_strength_when_worst_strata_are_near_tied(self):
        from hex_command_rl.handoff import reliable_choice

        stronger = dict(reliability_key=[0.36, 0.58, -0.1])
        noisy_edge = dict(reliability_key=[0.37, 0.50, -0.1])
        collapsed = dict(reliability_key=[0.0, 0.99, 0.0])
        self.assertIs(reliable_choice([stronger, noisy_edge, collapsed]), stronger)
