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
        self.weight=nn.Parameter(torch.empty(outputs,inputs,7))
        self.bias=nn.Parameter(torch.empty(outputs))
        nn.init.kaiming_uniform_(self.weight,a=math.sqrt(5))
        nn.init.uniform_(self.bias,-1/math.sqrt(inputs*7),1/math.sqrt(inputs*7))
        self.register_buffer('rows',torch.tensor([p[0]+1 for p in native.SPATIAL_KERNEL]))
        self.register_buffer('cols',torch.tensor([p[1]+1 for p in native.SPATIAL_KERNEL]))

    def forward(self,x):
        kernel=self.weight.new_zeros((*self.weight.shape[:2],3,3))
        kernel[:,:,self.rows,self.cols]=self.weight
        return F.conv2d(x,kernel,self.bias,padding=1)


class SpatialPlanner(nn.Module):
    def __init__(self,dropout=0.1):
        super().__init__()
        self.convs=nn.ModuleList([HexConv(len(native.SPATIAL_CHANNEL_NAMES)),HexConv(16)])
        self.head=nn.Sequential(nn.Linear(len(native.FEATURE_NAMES)+64,64),nn.ReLU(),nn.Dropout(dropout),
                                nn.Linear(64,64),nn.ReLU(),nn.Linear(64,1))
        self.dropout=dropout

    def encode(self,boards):
        mask=boards[:,:1]
        x=boards
        for layer in self.convs:x=F.relu(layer(x))*mask
        return x, (x.sum((-2,-1))/mask.sum((-2,-1)).clamp_min(1))

    def forward(self,features,boards,indices):
        if features.shape[-1] in (len(native.LEGACY_FEATURE_NAMES),len(native.DUTY_FEATURE_NAMES),len(native.DIPLOMACY_FEATURE_NAMES)):
            features=F.pad(features,(0,len(native.FEATURE_NAMES)-features.shape[-1]))
        if boards.shape[1] in (len(native.LEGACY_SPATIAL_CHANNEL_NAMES),len(native.DUTY_SPATIAL_CHANNEL_NAMES)):
            boards=F.pad(boards,(0,0,0,0,0,len(native.SPATIAL_CHANNEL_NAMES)-boards.shape[1]))
        spatial,pooled=self.encode(boards)
        batch,count,_=features.shape
        gathered=spatial.flatten(2).gather(2,indices.clamp_min(0).reshape(batch,1,-1).expand(-1,16,-1))
        gathered=gathered.transpose(1,2).reshape(batch,count,3,16)
        gathered=gathered*(indices>=0).unsqueeze(-1)
        inputs=torch.cat([features,gathered.flatten(2),pooled[:,None,:].expand(-1,count,-1)],dim=-1)
        return self.head(inputs).squeeze(-1)

    def export(self):
        return json.dumps(dict(schema=native.SPATIAL_SCHEMA,feature_names=native.FEATURE_NAMES,
            spatial=dict(channel_names=native.SPATIAL_CHANNEL_NAMES,convolutions=[dict(
                input_channels=c.weight.shape[1],output_channels=c.weight.shape[0],weights=c.weight.detach().flatten().tolist(),bias=c.bias.detach().tolist()) for c in self.convs]),
            layers=[dict(weights=l.weight.detach().tolist(),bias=l.bias.detach().tolist()) for l in self.head if isinstance(l,nn.Linear)]))
