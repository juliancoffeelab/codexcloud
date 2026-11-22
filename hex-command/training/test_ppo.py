"""Episode boundaries must stop PPO credit from crossing into another episode."""

import unittest
import numpy as np
import torch
from hex_command_rl.ppo import advantages, categorical


class AdvantageBoundaries(unittest.TestCase):
    def test_terminal_stops_bootstrap_and_following_episode_credit(self):
        rewards = np.array([[0.0, 1.0], [1.0, 0.0]], np.float32)
        values = np.zeros_like(rewards)
        dones = np.array([[0.0, 1.0], [1.0, 0.0]], np.float32)
        advantage, returns = advantages(rewards, values, dones, [100.0, 2.0], 0.5, 1.0)
        np.testing.assert_allclose(advantage, [[0.5, 1.0], [1.0, 1.0]])
        np.testing.assert_allclose(returns, advantage)

    def test_critic_baseline_changes_advantage_but_not_monte_carlo_target(self):
        rewards = np.array([[0.0], [-1.0]], np.float32)
        dones = np.array([[0.0], [1.0]], np.float32)
        first, target = advantages(
            rewards, np.zeros_like(rewards), dones, [0.0], 1.0, 1.0
        )
        second, same_target = advantages(
            rewards, np.full_like(rewards, -0.8), dones, [0.0], 1.0, 1.0
        )
        np.testing.assert_allclose(target, same_target, atol=1e-6)
        np.testing.assert_allclose(first, -1.0)
        np.testing.assert_allclose(second, -0.2, atol=1e-6)

    def test_padding_has_zero_probability_and_finite_policy_gradients(self):
        scores = torch.tensor([[1.0, 2.0, 30.0], [3.0, 1.0, 2.0]], requires_grad=True)
        mask = torch.tensor([[True, True, False], [True, True, True]])
        policy = categorical(scores, mask, 1.25)
        reference = categorical(scores.detach() + 0.1, mask, 1.25)
        loss = (
            torch.distributions.kl_divergence(policy, reference).mean()
            - 0.01 * policy.entropy().mean()
        )
        loss.backward()
        self.assertEqual(float(policy.probs[0, 2]), 0.0)
        self.assertTrue(torch.isfinite(scores.grad).all())
        self.assertGreater(float(scores.grad.abs().sum()), 0.0)


class RecurrentBoundaries(unittest.TestCase):
    def test_memory_is_faction_scoped_and_resets_without_double_advancing(self):
        from hex_command_rl.recurrent import MemoryBank
        bank = MemoryBank()
        context = ("player", "map", 1, 0, 1)
        before, reset = bank.inputs([0], [context])
        self.assertTrue(reset[0])
        bank.commit([0], [context], before, torch.ones(1, 64))
        repeated, _ = bank.inputs([0], [context])
        self.assertEqual(float(repeated.sum()), 0.)
        advanced, _ = bank.inputs([0], [("player", "map", 1, 1, 1)])
        self.assertEqual(float(advanced.sum()), 64.)
        for changed in [("enemy", "map", 1, 1, 1), ("player", "map", 1, 1, 2), ("player", "other", 1, 1, 1)]:
            memory, reset = bank.inputs([0], [changed])
            self.assertTrue(reset[0]); self.assertEqual(float(memory.sum()), 0.)
        bank.reset(0)
        self.assertTrue(bank.inputs([0], [context])[1][0])

    def test_later_decision_loss_reaches_the_earlier_observation(self):
        import hex_command_training as native
        from hex_command_rl.spatial import SpatialPlanner
        torch.manual_seed(732)
        model = SpatialPlanner(dropout=0)
        x = torch.randn(1, 2, len(native.FEATURE_NAMES))
        first = torch.randn(1, len(native.SPATIAL_CHANNEL_NAMES), 3, 4)
        first[:, 0] = 1; first.requires_grad_()
        second = first.detach().clone()
        indices = torch.tensor([[[0, 1, -1], [2, 3, -1]]])
        _, _, memory = model.forward_recurrent(x, first, indices)
        scores, _, _ = model.forward_recurrent(x, second, indices, memory)
        scores.sum().backward()
        self.assertGreater(float(first.grad.abs().sum()), 0.)
        self.assertGreater(float(model.gru.weight_hh.grad.abs().sum()), 0.)

    def test_replay_windows_preserve_order_and_episode_boundaries(self):
        from hex_command_rl.recurrent import windows
        samples = [dict(slot=slot, context=("player", f"map{step//5}", 1, step, step//5), memory_reset=step in (0, 5), step=step)
                   for step in range(10) for slot in range(2)]
        sequences = windows(samples, np.random.default_rng(17), length=8)
        seen = [(s['slot'], s['step']) for seq in sequences for s in seq]
        self.assertEqual(sorted(seen), sorted((s['slot'],s['step']) for s in samples))
        for seq in sequences:
            self.assertLessEqual(len(seq), 8)
            self.assertEqual(len({(s['slot'],s['context'][1]) for s in seq}), 1)
            steps = [s['step'] for s in seq]
            self.assertEqual(steps, list(range(steps[0], steps[0]+len(steps))))
