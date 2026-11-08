"""Episode boundaries must stop PPO credit from crossing into another episode."""
import unittest
import numpy as np
import torch
from hex_command_rl.ppo import advantages, categorical


class AdvantageBoundaries(unittest.TestCase):
    def test_terminal_stops_bootstrap_and_following_episode_credit(self):
        rewards = np.array([[0., 1.], [1., 0.]], np.float32)
        values = np.zeros_like(rewards)
        dones = np.array([[0., 1.], [1., 0.]], np.float32)
        advantage, returns = advantages(rewards, values, dones, [100., 2.], .5, 1.)
        np.testing.assert_allclose(advantage, [[.5, 1.], [1., 1.]])
        np.testing.assert_allclose(returns, advantage)

    def test_critic_baseline_changes_advantage_but_not_monte_carlo_target(self):
        rewards = np.array([[0.], [-1.]], np.float32)
        dones = np.array([[0.], [1.]], np.float32)
        first, target = advantages(rewards, np.zeros_like(rewards), dones, [0.], 1., 1.)
        second, same_target = advantages(rewards, np.full_like(rewards, -.8), dones, [0.], 1., 1.)
        np.testing.assert_allclose(target, same_target, atol=1e-6)
        np.testing.assert_allclose(first, -1.)
        np.testing.assert_allclose(second, -.2, atol=1e-6)

    def test_padding_has_zero_probability_and_finite_policy_gradients(self):
        scores = torch.tensor([[1., 2., 30.], [3., 1., 2.]], requires_grad=True)
        mask = torch.tensor([[True, True, False], [True, True, True]])
        policy = categorical(scores, mask, 1.25)
        reference = categorical(scores.detach() + .1, mask, 1.25)
        loss = torch.distributions.kl_divergence(policy, reference).mean() - .01 * policy.entropy().mean()
        loss.backward()
        self.assertEqual(float(policy.probs[0, 2]), 0.)
        self.assertTrue(torch.isfinite(scores.grad).all())
        self.assertGreater(float(scores.grad.abs().sum()), 0.)
