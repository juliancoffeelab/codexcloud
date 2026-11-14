import tempfile
import unittest
from pathlib import Path
import torch
from hex_command_rl.recovery import save_bundle, resolve, load_resume
from hex_command_rl.spatial import SpatialPlanner
from hex_command_rl.mixed_workload import configs, group, mass


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
            sum(mass(c) for c in unique.values() if "campaign" in c), 0.5
        )

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
        self.assertLess(abs(totals["campaign"] / sum(totals.values()) - 0.5), 0.08)

    def test_campaign_cap_is_failure_and_arena_cap_remains_draw(self):
        from hex_command_rl.mixed_ppo import terminal_reward

        self.assertEqual(terminal_reward({"campaign": "test"}, 0.0, True), -1.0)
        self.assertEqual(terminal_reward({"landscape": "test"}, 0.0, True), 0.0)
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
