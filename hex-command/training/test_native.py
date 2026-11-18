"""Small integration checks; run after maturin develop."""

import json
import unittest
import tempfile
import numpy as np
import torch
import hex_command_training as native
from hex_command_rl.model import Planner
from hex_command_rl.workloads import arena, encode


class NativeContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        torch.set_num_threads(1)
        cls.config = encode(arena([901]))[0]
        cls.data = native.collect([cls.config], 1000)

    def test_teacher_coverage_and_owned_buffers(self):
        x, offsets, labels, missing = self.data
        self.assertEqual(missing, 0)
        self.assertGreater(len(labels), 0)
        self.assertEqual(x.dtype, np.float32)
        self.assertEqual(offsets[-1], len(x))
        self.assertTrue(np.all(np.diff(offsets) > labels))
        self.assertTrue(np.isfinite(x).all())
        saved = x.copy()
        native.collect([self.config], 10)
        np.testing.assert_array_equal(x, saved)

    def test_inference_parity_and_schema_rejection(self):
        torch.manual_seed(7)
        model = Planner()
        x = self.data[0][:128].copy()
        with torch.no_grad():
            expected = model(torch.from_numpy(x)).numpy()
        np.testing.assert_allclose(
            native.score(model.export(), x), expected, atol=2e-6, rtol=2e-5
        )
        bad = json.loads(model.export())
        bad["schema"] += 1
        with self.assertRaises(ValueError):
            native.score(json.dumps(bad), x)
        bad = json.loads(model.export())
        bad["layers"][0]["weights"][0].pop()
        with self.assertRaises(ValueError):
            native.score(json.dumps(bad), x)

    def test_linear_model_exports_and_matches_native(self):
        model = Planner(architecture="linear")
        self.assertEqual(
            sum(p.numel() for p in model.parameters()), len(native.FEATURE_NAMES)
        )
        with torch.no_grad():
            model.net[0].weight.copy_(
                torch.linspace(-0.5, 0.5, len(native.FEATURE_NAMES))[None, :]
            )
        exported = model.export()
        self.assertEqual(len(json.loads(exported)["layers"]), 1)
        x = self.data[0][:128].copy()
        with torch.no_grad():
            expected = model(torch.from_numpy(x)).numpy()
        np.testing.assert_allclose(
            native.score(exported, x), expected, atol=2e-5, rtol=2e-5
        )

    def test_tree_model_native_validation(self):
        model = dict(
            schema=native.SCHEMA,
            feature_names=native.FEATURE_NAMES,
            trees=[
                dict(
                    nodes=[
                        dict(
                            kind="split",
                            feature=0,
                            threshold=0.5,
                            left=1,
                            right=2,
                            default_left=True,
                        ),
                        dict(kind="leaf", value=0.25),
                        dict(kind="leaf", value=-0.5),
                    ]
                )
            ],
        )
        x = np.zeros((2, len(native.FEATURE_NAMES)), dtype=np.float32)
        x[1, 0] = 1
        np.testing.assert_array_equal(native.score(json.dumps(model), x), [0.25, -0.5])
        x[0, 0] = np.nan
        np.testing.assert_array_equal(native.score(json.dumps(model), x), [0.25, -0.5])
        # Keep double-precision thresholds/leaves, including values that round
        # to the same float32 value. Otherwise valid rankings can change.
        exact = json.loads(json.dumps(model))
        exact["trees"][0]["nodes"][0]["threshold"] = 1.0 - 1e-9
        exact["trees"][0]["nodes"][2]["value"] = 0.25 + 1e-9
        self.assertGreater(native.score(json.dumps(exact), x)[1], 0.25)
        for field, value in [
            ("left", 0),
            ("right", 99),
            ("feature", len(native.FEATURE_NAMES)),
        ]:
            bad = json.loads(json.dumps(model))
            bad["trees"][0]["nodes"][0][field] = value
            with self.assertRaises(ValueError):
                native.score(json.dumps(bad), x)

    @unittest.skipUnless(
        __import__("importlib.util", fromlist=["find_spec"]).find_spec("lightgbm"),
        "optional LightGBM not installed",
    )
    def test_lightgbm_export_matches_native(self):
        import lightgbm as lgb
        from hex_command_rl.trees import export_model, grouped_metrics

        x = np.zeros((12, len(native.FEATURE_NAMES)), dtype=np.float32)
        x[:, 0] = np.tile([0.0, 0.2, 0.7, 1.0], 3)
        y = np.tile([0, 0, 0, 1], 3)
        data = lgb.Dataset(
            x, label=y, group=[4, 4, 4], params=dict(feature_pre_filter=False)
        )
        booster = lgb.train(
            dict(
                objective="lambdarank",
                verbosity=-1,
                num_threads=1,
                max_depth=2,
                num_leaves=4,
                min_data_in_leaf=1,
                min_data_in_bin=1,
                feature_pre_filter=False,
            ),
            data,
            num_boost_round=4,
        )
        model = export_model(booster)
        self.assertTrue(
            any(n["kind"] == "split" for t in model["trees"] for n in t["nodes"])
        )
        scores = booster.predict(x, num_threads=1)
        np.testing.assert_allclose(
            native.score(json.dumps(model), x), scores, atol=2e-5, rtol=2e-5
        )
        self.assertEqual(
            grouped_metrics(scores, np.array([0, 4, 8, 12]), np.array([3, 3, 3]))[
                "accuracy"
            ],
            1.0,
        )
        self.assertEqual(
            grouped_metrics(np.zeros(4), np.array([0, 4]), np.array([0]))["accuracy"],
            1.0,
        )

    def test_spatial_collection_parity_padding_and_gradients(self):
        from hex_command_rl.spatial import SpatialPlanner
        from hex_command_rl.spatial_data import collect_spatial_dataset, batches

        configs = [arena([906], rounds=3)[i] for i in (0, 32)]
        dense, spatial = native.collect_spatial(encode(configs), 100000, side="player")
        original = native.collect(encode(configs), 100000, side="player")
        for a, b in zip(dense[:3], original[:3]):
            np.testing.assert_array_equal(a, b)
        with tempfile.TemporaryDirectory() as directory:
            data = collect_spatial_dataset(configs, directory, batch_games=1)
            for a, b in zip(data.dense[:3], dense[:3]):
                np.testing.assert_array_equal(a, b)
            for a, b in zip(
                (
                    data.boards,
                    data.offsets,
                    data.shapes,
                    data.indices,
                    data.context_values,
                    data.contexts,
                ),
                spatial,
            ):
                np.testing.assert_array_equal(a, b)
            torch.manual_seed(19)
            model = SpatialPlanner().eval()
            expected = []
            with torch.no_grad():
                for x, board, ix, mask, y in batches(
                    data, np.arange(len(data.dense[2])), batch_size=8
                ):
                    expected.extend(model(x, board, ix)[mask].numpy())
            actual = native.score_spatial(
                model.export(),
                data.dense[0],
                data.dense[1],
                np.asarray(data.boards),
                data.offsets,
                data.shapes,
                data.indices,
                data.context_values,
                data.contexts,
            )
            np.testing.assert_allclose(actual, expected, atol=2e-6, rtol=2e-5)
            x, board, ix, mask, y = next(batches(data, np.array([0])))
            with torch.no_grad():
                before = model(x, board, ix)
                base, contexts, owners, sites = board
                old_width = base.shape[-1]
                new_width = old_width + 2
                padded = (
                    torch.nn.functional.pad(base, (0, 2, 0, 1)),
                    torch.nn.functional.pad(contexts, (0, 2, 0, 1)),
                    owners,
                    torch.where(
                        sites < 0,
                        sites,
                        (sites // old_width) * new_width + sites % old_width,
                    ),
                )
                shifted = torch.where(
                    ix < 0, ix, (ix // old_width) * new_width + ix % old_width
                )
                np.testing.assert_allclose(
                    model(x, padded, shifted).numpy(),
                    before.numpy(),
                    atol=2e-6,
                    rtol=2e-5,
                )
            model.train()
            loss = torch.nn.functional.cross_entropy(model(x, board, ix), y)
            loss.backward()
            self.assertGreater(float(model.convs[0].weight.grad.abs().sum()), 0.0)
            with self.assertRaises(ValueError):
                native.score(model.export(), data.dense[0][:1])
            bad = json.loads(model.export())
            bad["spatial"]["convolutions"][0]["weights"].pop()
            with self.assertRaises(ValueError):
                native.evaluate(encode(configs[:1]), json.dumps(bad))
            wrong = data.indices.copy()
            wrong[0, 0] = 10**9
            with self.assertRaises(ValueError):
                native.score_spatial(
                    model.export(),
                    data.dense[0],
                    data.dense[1],
                    np.asarray(data.boards),
                    data.offsets,
                    data.shapes,
                    wrong,
                )

    def test_map_balance_preserves_equal_aggregate_weight(self):
        from types import SimpleNamespace
        from hex_command_rl.spatial_data import map_weights
        from hex_command_rl.trees import grouped_metrics

        data = SimpleNamespace(
            dense=(None, None, np.zeros(4, dtype=np.int64), 0),
            manifest=dict(
                configs=[dict(landscape="a"), dict(landscape="b")],
                shards=[
                    dict(
                        config_start=0, config_end=1, decision_start=0, decision_end=1
                    ),
                    dict(
                        config_start=1, config_end=2, decision_start=1, decision_end=4
                    ),
                ],
            ),
        )
        weights = map_weights(data, True)
        self.assertAlmostEqual(float(weights[0]), float(weights[1:].sum()), places=6)
        result = grouped_metrics(
            np.array([1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0]),
            np.array([0, 2, 4, 6, 8]),
            np.zeros(4, dtype=np.int64),
            weights,
        )
        self.assertEqual(result["accuracy"], 0.25)
        self.assertAlmostEqual(result["map_balanced_accuracy"], 0.5, places=6)

    def test_expanded_cnn_checkpoint_preserves_logits_and_trains_new_inputs(self):
        from pathlib import Path
        from hex_command_rl.spatial import SpatialPlanner, HexConv
        from hex_command_rl.checkpoint import warm_start

        for schema, names, channel_names in [
            (3, native.LEGACY_FEATURE_NAMES, native.LEGACY_SPATIAL_CHANNEL_NAMES),
            (5, native.DUTY_FEATURE_NAMES, native.DUTY_SPATIAL_CHANNEL_NAMES),
            (7, native.DIPLOMACY_FEATURE_NAMES, native.CAMPAIGN_SPATIAL_CHANNEL_NAMES),
            (9, native.CAMPAIGN_FEATURE_NAMES, native.CAMPAIGN_SPATIAL_CHANNEL_NAMES),
        ]:
            width = len(names)
            channels = len(channel_names)
            torch.manual_seed(81)
            old = SpatialPlanner().eval()
            old.convs[0] = HexConv(channels)
            old.head[0] = torch.nn.Linear(width + 64, 64)
            x = torch.randn(2, 3, width)
            board = torch.randn(2, channels, 3, 4)
            board[:, 0] = 1
            indices = torch.tensor([[[0, 1, -1], [1, 2, 3], [4, 5, -1]]] * 2)
            with torch.no_grad():
                encoded, pooled = old.encode(board)
                gathered = encoded.flatten(2).gather(
                    2, indices.clamp_min(0).reshape(2, 1, -1).expand(-1, 16, -1)
                )
                gathered = gathered.transpose(1, 2).reshape(2, 3, 3, 16) * (
                    indices >= 0
                ).unsqueeze(-1)
                expected = old.head(
                    torch.cat(
                        [x, gathered.flatten(2), pooled[:, None, :].expand(-1, 3, -1)],
                        -1,
                    )
                ).squeeze(-1)
            exported = json.loads(old.export())
            exported.update(schema=schema, feature_names=names)
            exported["spatial"]["channel_names"] = channel_names
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "old.pt"
                torch.save(old.state_dict(), path)
                model = SpatialPlanner().eval()
                warm_start(model, path)
                expanded_x = torch.cat(
                    [x, torch.randn(2, 3, len(native.FEATURE_NAMES) - width)], -1
                )
                expanded_board = torch.cat(
                    [
                        board,
                        torch.randn(
                            2, len(native.SPATIAL_CHANNEL_NAMES) - channels, 3, 4
                        ),
                    ],
                    1,
                )
                with torch.no_grad():
                    actual = model(expanded_x, expanded_board, indices)
                np.testing.assert_allclose(
                    actual.numpy(), expected.numpy(), atol=2e-6, rtol=2e-5
                )
                upgraded = native.upgrade_model(json.dumps(exported))
                actual = native.score_spatial(
                    upgraded,
                    expanded_x.reshape(-1, len(native.FEATURE_NAMES)).numpy(),
                    np.array([0, 3, 6]),
                    expanded_board.flatten().numpy(),
                    np.array([0, expanded_board[0].numel(), expanded_board.numel()]),
                    np.array([[3, 4], [3, 4]]),
                    indices.reshape(-1, 3).numpy(),
                )
                np.testing.assert_allclose(
                    actual, expected.numpy().flatten(), atol=2e-6, rtol=2e-5
                )
                model.train()
                model(expanded_x, expanded_board, indices).sum().backward()
                if channels < len(native.SPATIAL_CHANNEL_NAMES):
                    self.assertGreater(
                        float(model.convs[0].weight.grad[:, channels:].abs().sum()), 0.0
                    )
                self.assertGreater(
                    float(
                        model.head[0]
                        .weight.grad[:, width : len(native.FEATURE_NAMES)]
                        .abs()
                        .sum()
                    ),
                    0.0,
                )

    def test_spatial_step_buffers_and_terminal_slots(self):
        env = native.BatchEnv(spatial=True)
        try:
            obs = env.reset([self.config])
            spatial = env.spatial_observation()
            self.assertEqual(len(spatial[3]), len(obs[0]))
            self.assertEqual(spatial[2].shape, (1, 2))
            snapshot = spatial[0].copy()
            for _ in range(40):
                obs = env.step([0])
                if obs[3][0] or obs[4][0]:
                    break
            self.assertTrue(obs[3][0] or obs[4][0])
            np.testing.assert_array_equal(env.spatial_observation()[2], [[0, 0]])
            from hex_command_rl.spatial import SpatialPlanner

            self.assertEqual(
                len(
                    native.score_spatial(
                        SpatialPlanner().export(),
                        obs[0],
                        obs[1],
                        *env.spatial_observation(),
                    )
                ),
                0,
            )
            np.testing.assert_array_equal(spatial[0], snapshot)
        finally:
            env.close()

    def test_batch_reset_step_validation_and_close(self):
        env = native.BatchEnv()
        obs = env.reset([self.config, self.config])
        snapshot = obs[0].copy()
        self.assertEqual(len(obs[1]), 3)
        with self.assertRaises(ValueError):
            env.step([10**9, 0])
        # Invalid action validation must not partially advance either game.
        np.testing.assert_array_equal(obs[0], snapshot)
        for _ in range(40):
            obs = env.step([0, 0])
            if all(t or u for t, u in zip(obs[3], obs[4])):
                break
        self.assertTrue(all(t or u for t, u in zip(obs[3], obs[4])))
        self.assertEqual(obs[1][-1], 0)
        env.reset([self.config])
        env.close()  # cancellation while a native worker is awaiting an action
        with self.assertRaises(ValueError):
            env.step([0])

    def test_random_candidates_remain_executable(self):
        env = native.BatchEnv()
        configs = encode(arena([902], rounds=8))[::20]
        obs = env.reset(configs)
        rng = np.random.default_rng(91)
        try:
            for _ in range(600):
                lengths = np.diff(obs[1])
                actions = [int(rng.integers(n)) if n else 0 for n in lengths]
                obs = env.step(actions)
                if all(t or u for t, u in zip(obs[3], obs[4])):
                    break
            self.assertTrue(all(t or u for t, u in zip(obs[3], obs[4])))
        finally:
            env.close()

    def test_sharded_collection_matches_native_and_rejects_truncation(self):
        from pathlib import Path
        from hex_command_rl.data import collect_dataset, load_dataset

        configs = [arena([903], recruiters=(r,))[0] for r in ("balanced", "predictor")]
        direct = native.collect(encode(configs), 100000)
        with tempfile.TemporaryDirectory() as directory:
            mapped = collect_dataset(configs, directory, batch_games=1)
            self.assertIsInstance(mapped[0], np.memmap)
            for a, b in zip(mapped[:3], direct[:3]):
                np.testing.assert_array_equal(a, b)
            self.assertEqual(load_dataset(directory)[0].shape, direct[0].shape)
            with self.assertRaises(RuntimeError):
                collect_dataset(configs, Path(directory) / "limited", max_samples=1)

    def test_collection_side_filter_partitions_decisions(self):
        from hex_command_rl.data import collect_dataset

        configs = [arena([904], recruiters=("situational",))[0]]
        both = native.collect(encode(configs), 100000)
        player = native.collect(encode(configs), 100000, side="player")
        enemy = native.collect(encode(configs), 100000, side="enemy")
        self.assertGreater(len(player[2]), 0)
        self.assertGreater(len(enemy[2]), 0)
        self.assertEqual(len(both[2]), len(player[2]) + len(enemy[2]))
        with tempfile.TemporaryDirectory() as directory:
            mapped = collect_dataset(configs, directory, side="player")
            for a, b in zip(mapped[:3], player[:3]):
                np.testing.assert_array_equal(a, b)
        with self.assertRaises(ValueError):
            native.collect(encode(configs), side="invalid")

    def test_masked_metrics_and_action_frequencies(self):
        from hex_command_rl.train import metrics

        width = len(native.FEATURE_NAMES)
        features = np.zeros((3, width), dtype=np.float32)
        features[0, 0] = features[2, 0] = 1
        features[1, 1] = 1
        data = (features, np.array([0, 2, 3]), np.array([1, 0]), 0)
        model = Planner()
        with torch.no_grad():
            for parameter in model.parameters():
                parameter.zero_()
        result = metrics(model, data)
        self.assertAlmostEqual(result["cross_entropy"], np.log(2) / 2, places=6)
        self.assertEqual(result["accuracy"], 0.5)
        self.assertEqual(result["predicted_actions"]["end_turn"], 2)
        self.assertEqual(result["by_action"]["move"], [0, 1])

    def test_collection_does_not_change_teacher_outcomes(self):
        before = native.evaluate([self.config], None)
        native.collect([self.config], 100)
        self.assertEqual(before, native.evaluate([self.config], None))


if __name__ == "__main__":
    unittest.main()
