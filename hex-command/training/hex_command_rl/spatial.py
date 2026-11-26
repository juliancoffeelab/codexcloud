"""Shared seven-neighbor board encoder and candidate scoring head."""

import json
import math
import torch
from torch import nn
from torch.nn import functional as F
import hex_command_training as native


class HexConv(nn.Module):
    def __init__(self, inputs, outputs=16):
        super().__init__()
        self.weight = nn.Parameter(torch.empty(outputs, inputs, 7))
        self.bias = nn.Parameter(torch.empty(outputs))
        nn.init.kaiming_uniform_(self.weight, a=math.sqrt(5))
        nn.init.uniform_(
            self.bias, -1 / math.sqrt(inputs * 7), 1 / math.sqrt(inputs * 7)
        )
        self.register_buffer(
            "rows", torch.tensor([p[0] + 1 for p in native.SPATIAL_KERNEL])
        )
        self.register_buffer(
            "cols", torch.tensor([p[1] + 1 for p in native.SPATIAL_KERNEL])
        )

    def forward(self, x, channels=None, bias=True, stride=1):
        weight = self.weight if channels is None else self.weight[:, channels]
        kernel = self.weight.new_zeros((*weight.shape[:2], 3, 3))
        kernel[:, :, self.rows, self.cols] = weight
        # Anchor coarse cells at the lower-left hex of each 2x2 axial block.
        # All four source cells then lie within the seven-hex stencil.
        if stride == 2:
            x = F.pad(x, (1, 1, 0, 2))
        return F.conv2d(x, kernel, self.bias if bias else None, padding=0 if stride == 2 else 1, stride=stride)


class SpatialPlanner(nn.Module):
    def __init__(self, dropout=0.1):
        super().__init__()
        self.convs = nn.ModuleList(
            [HexConv(len(native.SPATIAL_CHANNEL_NAMES)), HexConv(16)]
        )
        self.coarse = nn.ModuleList(
            [HexConv(len(native.SPATIAL_CHANNEL_NAMES)), HexConv(16)]
        )
        self.gru = nn.GRUCell(16, 64)
        self.head = nn.Sequential(
            nn.Linear(len(native.FEATURE_NAMES) + 192, 64),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(64, 64),
            nn.ReLU(),
            nn.Linear(64, 1),
        )
        with torch.no_grad():
            self.head[0].weight[
                :, len(native.VISIBILITY_FEATURE_NAMES) : len(native.FEATURE_NAMES)
            ] = 0
        self.dropout = dropout

    def encode(self, boards):
        mask = boards[:, :1]
        x = boards
        for layer in self.convs:
            x = F.relu(layer(x)) * mask
        return x, (x.sum((-2, -1)) / mask.sum((-2, -1)).clamp_min(1))

    def forward(self, features, boards, indices):
        return self.forward_with_pool(features, boards, indices)[0]

    def forward_with_pool(self, features, boards, indices):
        scores, pooled, _ = self.forward_recurrent(features, boards, indices)
        return scores, pooled

    def encode_spatial(self, features, boards, indices):
        if features.shape[-1] < len(native.FEATURE_NAMES):
            features = F.pad(features, (0, len(native.FEATURE_NAMES) - features.shape[-1]))
        context_lookup = None
        if isinstance(boards, tuple):
            base, _, owners, sites = boards
            batch_ids = torch.arange(len(base), device=owners.device)
            matches = (owners[None, None, :] == batch_ids[:, None, None]) & (
                sites[None, None, :] == indices[:, :, 0, None]
            )
            defaults = (owners[None, :] == batch_ids[:, None]).long().argmax(-1)
            context_ids = torch.where(
                matches.any(-1), matches.long().argmax(-1), defaults[:, None]
            )
            context_lookup = (context_ids, defaults)
        parts = []
        pooled = None
        for layers, stride in [(self.convs, 1), (self.coarse, 2)]:
            gathered, candidate_pool, summary = self.encode_candidates(
                boards, indices, layers, stride, context_lookup=context_lookup
            )
            parts.extend([gathered, candidate_pool])
            if stride == 1:
                pooled = summary  # Preserve the existing critic's 16-value input.
        return features, parts, pooled

    def forward_recurrent(self, features, boards, indices, memory=None):
        features, parts, pooled = self.encode_spatial(features, boards, indices)
        if memory is None:
            memory = pooled.new_zeros((len(pooled), 64))
        memory = self.gru(pooled, memory)
        parts.append(memory[:, None, :].expand(-1, features.shape[1], -1))
        return self.head(torch.cat([features, *parts], dim=-1)).squeeze(-1), pooled, memory

    def encode_candidates(self, boards, indices, layers, stride, context_lookup=None):
        base = boards[0] if isinstance(boards, tuple) else boards
        if base.shape[1] < len(native.SPATIAL_CHANNEL_NAMES):
            base = F.pad(base, (0, 0, 0, 0, 0, len(native.SPATIAL_CHANNEL_NAMES) - base.shape[1]))
        mask = base[:, :1]
        if stride == 2:
            # Non-overlapping blocks make masking independent of batch padding.
            mask = F.max_pool2d(mask, 2, stride=2, ceil_mode=True)
        ix = indices.clamp_min(0)
        width = (base.shape[-1] + stride - 1) // stride
        ix = (ix // base.shape[-1] // stride) * width + ix % base.shape[-1] // stride
        if isinstance(boards, tuple):
            _, contexts, owners, sites = boards
            shared = layers[0](base, stride=stride)
            relative = layers[0](contexts, native.ACTOR_SPATIAL_CHANNELS, bias=False, stride=stride)
            first = F.relu(shared[owners] + relative) * mask[owners]
            spatial = F.relu(layers[1](first)) * mask[owners]
            summaries = spatial.sum((-2, -1)) / mask[owners].sum((-2, -1)).clamp_min(1)
            if context_lookup is None:
                batch_ids = torch.arange(len(base), device=owners.device)
                matches = (owners[None, None, :] == batch_ids[:, None, None]) & (
                    sites[None, None, :] == indices[:, :, 0, None]
                )
                defaults = (owners[None, :] == batch_ids[:, None]).long().argmax(-1)
                context_ids = torch.where(
                    matches.any(-1), matches.long().argmax(-1), defaults[:, None]
                )
            else:
                context_ids, defaults = context_lookup
            gathered = spatial.flatten(2).permute(0, 2, 1)[context_ids[:, :, None], ix]
            candidate_pool = summaries[context_ids]
            pooled = summaries[defaults]
        else:
            first = F.relu(layers[0](base, stride=stride)) * mask
            spatial = F.relu(layers[1](first)) * mask
            pooled = spatial.sum((-2, -1)) / mask.sum((-2, -1)).clamp_min(1)
            batch, count, _ = indices.shape
            gathered = spatial.flatten(2).gather(2, ix.reshape(batch, 1, -1).expand(-1, 16, -1))
            gathered = gathered.transpose(1, 2).reshape(batch, count, 3, 16)
            candidate_pool = pooled[:, None, :].expand(-1, count, -1)
        gathered = gathered * (indices >= 0).unsqueeze(-1)
        return gathered.flatten(2), candidate_pool, pooled

    def export(self):
        return json.dumps(
            dict(
                schema=native.SPATIAL_SCHEMA,
                feature_names=native.FEATURE_NAMES,
                recurrent={name: value.detach().flatten().tolist() for name, value in (self.gru.named_parameters() if self.gru is not None else [])},
                spatial=dict(
                    channel_names=native.SPATIAL_CHANNEL_NAMES,
                    convolutions=[
                        dict(
                            input_channels=c.weight.shape[1],
                            output_channels=c.weight.shape[0],
                            weights=c.weight.detach().flatten().tolist(),
                            bias=c.bias.detach().tolist(),
                        )
                        for c in [*self.convs, *(self.coarse or [])]
                    ],
                ),
                layers=[
                    dict(
                        weights=l.weight.detach().tolist(),
                        bias=l.bias.detach().tolist(),
                    )
                    for l in self.head
                    if isinstance(l, nn.Linear)
                ],
            )
        )
