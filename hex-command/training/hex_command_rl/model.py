"""A shared scorer, independent of board size and candidate count."""
import json
import torch
from torch import nn
import hex_command_training as native


class Planner(nn.Module):
    def __init__(self, hidden=64):
        super().__init__()
        self.net = nn.Sequential(
            nn.Linear(len(native.FEATURE_NAMES), hidden), nn.ReLU(),
            nn.Linear(hidden, hidden), nn.ReLU(), nn.Linear(hidden, 1),
        )

    def forward(self, features):
        return self.net(features).squeeze(-1)

    def export(self):
        return json.dumps({
            "schema": native.SCHEMA,
            "feature_names": native.FEATURE_NAMES,
            "layers": [{"weights": layer.weight.detach().tolist(),
                        "bias": layer.bias.detach().tolist()}
                       for layer in self.net if isinstance(layer, nn.Linear)],
        })
