"""Function-preserving input expansion for native-era PyTorch checkpoints."""

import hashlib
from pathlib import Path
import torch
import hex_command_training as native


def warm_start(model, path):
    path = Path(path)
    state = torch.load(path, map_location="cpu", weights_only=True)
    return expand_state(model, state) | dict(
        path=str(path.resolve()), sha256=hashlib.sha256(path.read_bytes()).hexdigest()
    )


def expand_state(model, state):
    from .spatial import SpatialPlanner

    target = model.state_dict()
    spatial = isinstance(model, SpatialPlanner)
    old_spatial = spatial and not any(k.startswith("coarse.") for k in state)
    old_recurrent = spatial and not any(k.startswith("gru.") for k in state)
    expected = {k for k in target if not (old_spatial and k.startswith("coarse.")) and not (old_recurrent and k.startswith("gru."))}
    if set(state) != expected:
        raise ValueError("checkpoint architecture or parameter names differ")
    key = "head.0.weight" if spatial else "net.0.weight"
    embeddings = ((64 if old_spatial else 128) + (0 if old_recurrent else 64)) if spatial else 0
    old_width = state[key].shape[1] - embeddings
    width = len(native.FEATURE_NAMES)
    if old_width not in (
        len(native.LEGACY_FEATURE_NAMES),
        len(native.DUTY_FEATURE_NAMES),
        len(native.DIPLOMACY_FEATURE_NAMES),
        len(native.CAMPAIGN_FEATURE_NAMES),
        len(native.VISIBILITY_FEATURE_NAMES),
        len(native.IDLE_FEATURE_NAMES),
        width,
    ):
        raise ValueError("unknown checkpoint feature width")
    old_channels = state["convs.0.weight"].shape[1] if spatial else None
    if spatial and (old_width, old_channels) not in (
        (len(native.LEGACY_FEATURE_NAMES), len(native.LEGACY_SPATIAL_CHANNEL_NAMES)),
        (len(native.DUTY_FEATURE_NAMES), len(native.DUTY_SPATIAL_CHANNEL_NAMES)),
        (
            len(native.DIPLOMACY_FEATURE_NAMES),
            len(native.CAMPAIGN_SPATIAL_CHANNEL_NAMES),
        ),
        (
            len(native.CAMPAIGN_FEATURE_NAMES),
            len(native.CAMPAIGN_SPATIAL_CHANNEL_NAMES),
        ),
        (len(native.VISIBILITY_FEATURE_NAMES), len(native.VISIBILITY_SPATIAL_CHANNEL_NAMES)),
        (len(native.IDLE_FEATURE_NAMES), len(native.VISIBILITY_SPATIAL_CHANNEL_NAMES)),
        (len(native.IDLE_FEATURE_NAMES), len(native.HOLD_SPATIAL_CHANNEL_NAMES)),
        (len(native.IDLE_FEATURE_NAMES), len(native.SPATIAL_CHANNEL_NAMES)),
        (width, len(native.SPATIAL_CHANNEL_NAMES)),
    ):
        raise ValueError("checkpoint feature/channel schemas differ")
    copied = {}
    for name, value in target.items():
        if (old_spatial and name.startswith("coarse.")) or (old_recurrent and name.startswith("gru.")):
            copied[name] = value.clone()
            continue
        source = state[name]
        if not torch.isfinite(source).all():
            raise ValueError("non-finite checkpoint parameter")
        if name == key:
            if source.shape[0] != value.shape[0]:
                raise ValueError("checkpoint hidden width differs")
            expanded = torch.zeros_like(value)
            expanded[:, :old_width] = source[:, :old_width]
            if spatial:
                expanded[:, width : width + embeddings] = source[:, old_width:]
            copied[name] = expanded
        elif spatial and name == "convs.0.weight":
            if source.shape[0] != value.shape[0] or source.shape[2:] != value.shape[2:]:
                raise ValueError("checkpoint convolution differs")
            expanded = torch.zeros_like(value)
            expanded[:, :old_channels] = source
            copied[name] = expanded
        else:
            if source.shape != value.shape:
                raise ValueError("checkpoint parameter shape differs: " + name)
            copied[name] = source
    model.load_state_dict(copied)
    return dict(
        source_features=old_width,
        source_channels=old_channels,
        optimizer="new Adam state",
    )
