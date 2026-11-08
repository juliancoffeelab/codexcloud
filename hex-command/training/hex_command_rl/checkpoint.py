"""Function-preserving input expansion for native-era PyTorch checkpoints."""
import hashlib
from pathlib import Path
import torch
import hex_command_training as native


def warm_start(model, path):
    from .spatial import SpatialPlanner
    path=Path(path)
    state=torch.load(path,map_location='cpu',weights_only=True)
    target=model.state_dict()
    if state.keys()!=target.keys():raise ValueError('checkpoint architecture or parameter names differ')
    spatial=isinstance(model,SpatialPlanner)
    key='head.0.weight' if spatial else 'net.0.weight'
    old_width=state[key].shape[1]-(64 if spatial else 0)
    width=len(native.FEATURE_NAMES)
    if old_width not in (len(native.LEGACY_FEATURE_NAMES),len(native.DUTY_FEATURE_NAMES),width):raise ValueError('unknown checkpoint feature width')
    old_channels=state['convs.0.weight'].shape[1] if spatial else None
    if spatial and (old_width,old_channels) not in ((len(native.LEGACY_FEATURE_NAMES),len(native.LEGACY_SPATIAL_CHANNEL_NAMES)),(len(native.DUTY_FEATURE_NAMES),len(native.DUTY_SPATIAL_CHANNEL_NAMES)),(width,len(native.SPATIAL_CHANNEL_NAMES))):
        raise ValueError('checkpoint feature/channel schemas differ')
    copied={}
    for name,value in target.items():
        source=state[name]
        if not torch.isfinite(source).all():raise ValueError('non-finite checkpoint parameter')
        if name==key:
            if source.shape[0]!=value.shape[0]:raise ValueError('checkpoint hidden width differs')
            expanded=torch.zeros_like(value)
            expanded[:,:old_width]=source[:,:old_width]
            if spatial:expanded[:,width:]=source[:,old_width:]
            copied[name]=expanded
        elif spatial and name=='convs.0.weight':
            if source.shape[0]!=value.shape[0] or source.shape[2:]!=value.shape[2:]:raise ValueError('checkpoint convolution differs')
            expanded=torch.zeros_like(value);expanded[:,:old_channels]=source;copied[name]=expanded
        else:
            if source.shape!=value.shape:raise ValueError('checkpoint parameter shape differs: '+name)
            copied[name]=source
    model.load_state_dict(copied)
    return dict(path=str(path.resolve()),sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                source_features=old_width,source_channels=old_channels,optimizer='new Adam state')
