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

    def test_resolve_device_validation(self):
        from hex_command_rl.ppo import resolve_device
        self.assertEqual(resolve_device("cpu").type, "cpu")
        self.assertIn(resolve_device("auto").type, ("cpu", "mps", "cuda"))
        with self.assertRaises(ValueError):
            resolve_device("tpu")

    def test_replay_batch_matches_unbatched_and_transfers_across_devices(self):
        import tempfile
        import hex_command_training as native
        from hex_command_rl.spatial import SpatialPlanner
        from hex_command_rl.ppo import ActorCritic, pack, categorical
        from hex_command_rl.recurrent import replay, replay_batch
        from hex_command_rl.recovery import save_bundle, load_resume

        torch.manual_seed(99)
        policy = SpatialPlanner(dropout=0.0).eval()
        ac = ActorCritic(policy).eval()

        def make_sample(n_cand, h, w, action):
            feats = np.random.default_rng(n_cand + h).standard_normal(
                (n_cand, len(native.FEATURE_NAMES))
            ).astype(np.float32)
            board = np.random.default_rng(n_cand + w).standard_normal(
                (len(native.SPATIAL_CHANNEL_NAMES), h, w)
            ).astype(np.float32)
            board[0] = 1.0
            ix = np.zeros((n_cand, 3), np.int64)
            obs = (feats, board, ix, [])
            mem = np.zeros(64, np.float32)
            with torch.no_grad():
                dist, _, _ = ac.forward_recurrent(*pack([obs]), 0.35, torch.zeros(1, 64))
                ref = categorical(dist.logits + 0.05, torch.ones(1, n_cand, dtype=torch.bool), 0.35)
                lp = float(dist.log_prob(torch.tensor([action])))
            return dict(
                memory=mem,
                observation=obs,
                action=action,
                old_log_prob=lp,
                advantage=0.25,
                target=0.5,
                reference_logits=ref.logits[0].numpy().copy(),
            )

        seq_a = [make_sample(3, 4, 5, 1), make_sample(4, 4, 5, 2)]
        seq_b = [make_sample(5, 5, 6, 0)]

        # Single-window parity: replay_batch([seq]) == replay(seq)
        single_ref = replay(ac, seq_a, 0.35)
        single_batched = replay_batch(ac, [seq_a], 0.35)
        for a, b in zip(single_ref, single_batched):
            self.assertAlmostEqual(float(a), float(b), places=5)

        # Uneven-length multi-window parity: weighted average of individual windows
        ref_b = replay(ac, seq_b, 0.35)
        multi_batched = replay_batch(ac, [seq_a, seq_b], 0.35)
        for a, b, m in zip(single_ref, ref_b, multi_batched):
            expected = (len(seq_a) * float(a) + len(seq_b) * float(b)) / (len(seq_a) + len(seq_b))
            self.assertAlmostEqual(float(m), expected, places=5)

        # Cross-device MPS <-> CPU parity & checkpoint portability
        if torch.backends.mps.is_available():
            mps = torch.device("mps")
            policy_mps = SpatialPlanner(dropout=0.0).eval()
            policy_mps.load_state_dict(policy.state_dict())
            ac_mps = ActorCritic(policy_mps).eval()
            ac_mps.critic.load_state_dict(ac.critic.state_dict())
            ac_mps = ac_mps.to(mps)

            mps_vals = replay_batch(ac_mps, [seq_a, seq_b], 0.35, device=mps)
            for c_val, m_val in zip(multi_batched, mps_vals):
                self.assertAlmostEqual(float(c_val), float(m_val.cpu()), places=4)

            mps_fp16_vals = replay_batch(
                ac_mps, [seq_a, seq_b], 0.35, device=mps, fp16=True
            )
            for c_val, m_val in zip(multi_batched, mps_fp16_vals):
                self.assertAlmostEqual(float(c_val), float(m_val.cpu()), places=2)

            opt_mps = torch.optim.Adam(ac_mps.parameters(), lr=1e-4)
            loss = mps_vals[0] + mps_vals[1]
            loss.backward()
            opt_mps.step()

            with tempfile.TemporaryDirectory() as d:
                save_bundle(
                    d,
                    dict(policy=policy_mps.state_dict(), opt=opt_mps.state_dict()),
                    policy_mps,
                    "mps-to-cpu",
                )
                loaded = load_resume(d)
                for tensor in loaded["policy"].values():
                    self.assertEqual(tensor.device.type, "cpu")
                policy_cpu = SpatialPlanner(dropout=0.0).eval()
                policy_cpu.load_state_dict(loaded["policy"])
                # And back to MPS
                policy_mps2 = SpatialPlanner(dropout=0.0).eval().to(mps)
                policy_mps2.load_state_dict(loaded["policy"])
                self.assertEqual(next(policy_mps2.parameters()).device.type, "mps")
