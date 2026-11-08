"""Disk-backed boards stored once per decision, with ragged candidate rows."""
import json
from pathlib import Path
import numpy as np
import torch
import hex_command_training as native
from .data import load_dataset
from .workloads import encode


class SpatialData:
    def __init__(self,directory):
        directory=Path(directory);self.manifest=json.loads((directory/'manifest.json').read_text())
        current=self.manifest.get('spatial_schema')==native.SPATIAL_SCHEMA and self.manifest.get('channel_names')==native.SPATIAL_CHANNEL_NAMES
        legacy=self.manifest.get('spatial_schema')==3 and self.manifest.get('channel_names')==native.LEGACY_SPATIAL_CHANNEL_NAMES
        if not current and not legacy:
            raise ValueError('spatial dataset schema mismatch')
        self.dense=load_dataset(directory)
        self.offsets=np.load(directory/'board_offsets.npy');self.shapes=np.load(directory/'shapes.npy');self.indices=np.load(directory/'indices.npy')
        n=len(self.dense[2]);channels=len(self.manifest['channel_names']);self.channels=channels
        if legacy != (self.dense[0].shape[1]==len(native.LEGACY_FEATURE_NAMES)):raise ValueError('mixed dense/spatial dataset schemas')
        if self.shapes.shape!=(n,2) or self.offsets.shape!=(n+1,) or self.indices.shape!=(len(self.dense[0]),3):raise ValueError('spatial dataset dimensions mismatch')
        if self.offsets[0]!=0 or np.any(self.shapes<=0) or np.any(self.shapes>256) or np.any(self.shapes.prod(1)>16384) or np.any(np.diff(self.offsets)!=self.shapes.prod(1)*channels):raise ValueError('invalid spatial board offsets/shapes')
        if (directory/'boards.bin').stat().st_size!=int(self.offsets[-1])*4:raise ValueError('incomplete spatial board buffer')
        cells=np.repeat(self.shapes.prod(1),np.diff(self.dense[1]))
        if np.any(self.indices < -1) or np.any(self.indices>=cells[:,None]):raise ValueError('invalid spatial candidate indices')
        self.boards=np.memmap(directory/'boards.bin',dtype=np.float32,mode='c',shape=(int(self.offsets[-1]),))


def collect_spatial_dataset(configs,directory,progress=None,batch_games=8):
    directory=Path(directory);directory.mkdir(parents=True,exist_ok=True)
    (directory/'manifest.json').unlink(missing_ok=True)
    offsets=[0];labels=[];board_offsets=[0];shapes=[];indices=[];shards=[];rows=0;values=0
    with (directory/'features.bin').open('wb') as xf,(directory/'boards.bin').open('wb') as bf:
        for start in range(0,len(configs),batch_games):
            dense,spatial=native.collect_spatial(encode(configs[start:start+batch_games]),1_000_000,side='player')
            x,bounds,y,missing=dense;boards,bbounds,shape,ix=spatial
            if missing or len(y)>=1_000_000:raise RuntimeError('incomplete spatial teacher shard')
            assert len(shape)==len(y) and len(ix)==len(x)
            shards.append(dict(config_start=start,config_end=min(start+batch_games,len(configs)),decision_start=len(labels),decision_end=len(labels)+len(y)))
            x.tofile(xf);boards.tofile(bf)
            offsets.extend((bounds[1:]+rows).tolist());labels.extend(y.tolist());board_offsets.extend((bbounds[1:]+values).tolist());shapes.extend(shape.tolist());indices.append(ix)
            rows+=len(x);values+=len(boards)
            if progress:progress(dict(event='collection',directory=str(directory),games=min(start+batch_games,len(configs)),total_games=len(configs),decisions=len(labels),candidates=rows))
    if not labels:raise ValueError('empty spatial teacher data')
    for name,array in [('offsets',offsets),('labels',labels),('board_offsets',board_offsets),('shapes',shapes)]:np.save(directory/f'{name}.npy',np.asarray(array,dtype=np.int64))
    np.save(directory/'indices.npy',np.concatenate(indices))
    manifest=dict(schema=native.SCHEMA,feature_names=native.FEATURE_NAMES,spatial_schema=native.SPATIAL_SCHEMA,channel_names=native.SPATIAL_CHANNEL_NAMES,
        fingerprints=json.loads(native.fingerprints()),configs=configs,collection_side='player',decisions=len(labels),candidates=rows,shards=shards)
    (directory/'manifest.json').write_text(json.dumps(manifest,indent=2))
    return SpatialData(directory)


def batches(data,ids,batch_size=64):
    features,offsets,labels,_=data.dense
    channels=len(native.SPATIAL_CHANNEL_NAMES)
    for start in range(0,len(ids),batch_size):
        selected=np.asarray(ids[start:start+batch_size]);sizes=offsets[selected+1]-offsets[selected]
        width=int(sizes.max());h,w=data.shapes[selected].max(0);h=int(h);w=int(w)
        board=np.zeros((len(selected),channels,h,w),dtype=np.float32)
        ix=np.full((len(selected),width,3),-1,dtype=np.int64)
        x=np.zeros((len(selected),width,len(native.FEATURE_NAMES)),dtype=np.float32)
        mask=np.arange(width)[None,:]<sizes[:,None]
        for row,(i,size) in enumerate(zip(selected,sizes)):
            bh,bw=map(int,data.shapes[i]);a=int(offsets[i]);b=int(offsets[i+1])
            board[row,:data.channels,:bh,:bw]=data.boards[data.offsets[i]:data.offsets[i+1]].reshape(data.channels,bh,bw)
            x[row,:size,:features.shape[1]]=features[a:b]
            positions=data.indices[a:b]
            ix[row,:size]=np.where(positions<0,-1,(positions//bw)*w+positions%bw)
        yield torch.from_numpy(x),torch.from_numpy(board),torch.from_numpy(ix),torch.from_numpy(mask),torch.from_numpy(labels[selected].copy())


def map_weights(data,enabled=False):
    """Equal aggregate loss weight per map, retaining every collected decision."""
    count=len(data.dense[2])
    if not enabled:return np.ones(count,dtype=np.float32)
    groups=np.full(count,-1,dtype=np.int64);names={};cursor=0
    for shard in data.manifest.get('shards',[]):
        names_in_shard={c.get('landscape',c.get('campaign')) for c in data.manifest['configs'][shard['config_start']:shard['config_end']]}
        if len(names_in_shard)!=1 or shard['decision_start']!=cursor:raise ValueError('map balancing requires contiguous homogeneous-map collection shards')
        name=next(iter(names_in_shard));group=names.setdefault(name,len(names))
        cursor=shard['decision_end'];groups[shard['decision_start']:cursor]=group
    if cursor!=count or np.any(groups<0):raise ValueError('incomplete map-balancing metadata')
    sizes=np.bincount(groups)
    if len(sizes)!=len(names) or np.any(sizes==0):raise ValueError('empty training map')
    return (count/(len(sizes)*sizes[groups])).astype(np.float32)
