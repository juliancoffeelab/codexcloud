"""Sequence boundaries, ragged batching and recurrent credit must agree."""

import unittest
from types import SimpleNamespace
import numpy as np
import torch
import hex_command_training as native
from hex_command_rl.imitation_sequences import decision_streams, forward_window
from hex_command_rl.spatial import SpatialPlanner


def fixture():
    rng = np.random.default_rng(761)
    count, candidates, height, width = 5, 2, 3, 4
    channels = len(native.SPATIAL_CHANNEL_NAMES)
    boards = rng.normal(size=(count, channels, height, width)).astype(np.float32)
    boards[:, 0] = 1
    return SimpleNamespace(
        dense=(rng.normal(size=(count * candidates, len(native.FEATURE_NAMES))).astype(np.float32),
               np.arange(count + 1) * candidates, np.array([0, 1, 0, 1, 0]), 0),
        boards=boards.ravel(), offsets=np.arange(count + 1) * channels * height * width,
        shapes=np.array([[height, width]] * count),
        indices=np.tile([[0, 1, -1], [2, 3, -1]], (count, 1)),
        contexts=np.empty((0, 3), np.int64), context_values=np.empty(0, np.float32),
        channels=channels,
    )


class ImitationSequences(unittest.TestCase):
    def test_match_faction_and_repeated_chapter_are_distinct_streams(self):
        def row(game, observer, generation, decision):
            return dict(game=game, context=dict(observer=observer, generation=generation,
                        map="same-map", round=1 + decision, decision=decision))
        rows = [row(0, "player", 1, 0), row(0, "enemy", 1, 0),
                row(0, "player", 1, 1), row(0, "player", 2, 0),
                row(1, "player", 1, 0)]
        self.assertEqual([s.tolist() for s in decision_streams(rows, 5)],
                         [[0, 2], [1], [3], [4]])
        for invalid in (rows + [rows[0]], [rows[2], rows[0]]):
            with self.assertRaises(ValueError):
                decision_streams(invalid, len(invalid))
        with self.assertRaises(ValueError):
            decision_streams(rows, 4)

    def test_ragged_batch_and_saved_window_memory_match_separate_replays(self):
        torch.manual_seed(761)
        model, data = SpatialPlanner(dropout=0).eval(), fixture()
        streams = [np.array([0, 1, 2]), np.array([3, 4])]
        with torch.no_grad():
            loss, hits, count, memory = forward_window(model, data, streams, 0, length=8)
            separate = [forward_window(model, data, [s], 0, length=8) for s in streams]
            torch.testing.assert_close(loss, sum(s[0] for s in separate))
            torch.testing.assert_close(memory, torch.cat([s[3] for s in separate]))
            self.assertEqual((hits, count), (sum(s[1] for s in separate), 5))
            first, _, _, saved = forward_window(model, data, streams, 0, length=2)
            # Resume serializes detached values, rather than retaining an autograd graph.
            restored = torch.tensor(saved.detach().tolist())
            last, _, _, resumed = forward_window(model, data, streams, 2, restored, length=2)
            torch.testing.assert_close(first + last, loss)
            torch.testing.assert_close(resumed, memory)
            with self.assertRaises(ValueError):
                forward_window(model, data, streams, 2)

    def test_later_teacher_loss_trains_memory_across_earlier_decisions(self):
        torch.manual_seed(762)
        model, data = SpatialPlanner(dropout=0), fixture()
        stream = [np.array([0, 1, 2])]
        # Use the real imitation window path; retain the first state to inspect
        # credit from the following decisions, without adding its own loss.
        _, _, _, first = forward_window(model, data, stream, 0, length=1)
        first.retain_grad()
        loss, _, _, _ = forward_window(model, data, stream, 1, first, length=2)
        loss.backward()
        self.assertGreater(float(first.grad.abs().sum()), 0.)
        self.assertGreater(float(model.gru.weight_hh.grad.abs().sum()), 0.)
        self.assertGreater(float(model.convs[0].weight.grad.abs().sum()), 0.)


if __name__ == "__main__":
    unittest.main()
