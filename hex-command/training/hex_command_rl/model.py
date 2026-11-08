"""A shared scorer, independent of board size and candidate count."""
import json
import torch
from torch import nn
import hex_command_training as native


class Planner(nn.Module):
    def __init__(self, hidden=64, architecture="mlp"):
        super().__init__()
        if architecture == "linear":
            # A shared intercept cancels across candidate logits, so omit it.
            self.net = nn.Sequential(nn.Linear(len(native.FEATURE_NAMES), 1, bias=False))
            nn.init.zeros_(self.net[0].weight)
        elif architecture == "mlp":
            self.net = nn.Sequential(
                nn.Linear(len(native.FEATURE_NAMES), hidden), nn.ReLU(),
                nn.Linear(hidden, hidden), nn.ReLU(), nn.Linear(hidden, 1),
            )
        else:
            raise ValueError(f"unknown architecture: {architecture}")

    def forward(self, features):
        if features.shape[-1] in (len(native.LEGACY_FEATURE_NAMES),len(native.DUTY_FEATURE_NAMES),len(native.DIPLOMACY_FEATURE_NAMES)):
            features=torch.nn.functional.pad(features,(0,len(native.FEATURE_NAMES)-features.shape[-1]))
        return self.net(features).squeeze(-1)

    def export(self):
        return json.dumps({
            "schema": native.SCHEMA,
            "feature_names": native.FEATURE_NAMES,
            "layers": [{"weights": layer.weight.detach().tolist(),
                        "bias": layer.bias.detach().tolist() if layer.bias is not None else [0.]*layer.out_features}
                       for layer in self.net if isinstance(layer, nn.Linear)],
        })
