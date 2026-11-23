"""Read the six-round public score from the frozen learner observation layout."""
import json
import os
from pathlib import Path
import sys

from build import OUT, POST, RUN, read, write


def main():
    protocol = read(OUT / 'confirmation/protocol.json')
    catalog = Path(protocol['catalog'])
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(catalog)
    os.environ['RAYON_NUM_THREADS'] = '1'
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    names = ('opponent_hold_objective', 'opponent_hold_turns_remaining', 'enemy_owned', 'capture_goal')
    channels = {name: native.SPATIAL_CHANNEL_NAMES.index(name) for name in names}
    selected = [case for case in protocol['cases']['queued'] if case['seed'] == protocol['seeds'][0]]
    samples = []
    for case in selected:
        map_data = read(catalog / f'game/scenario/{case["landscape"]}/map.json')
        _, arrays, contexts = native.collect_spatial_sequences([json.dumps(case)], 100, 'player')
        values, offsets, shapes, *_ = arrays
        contexts = json.loads(contexts)
        post_col, post_row = POST
        min_q = min(tile['col'] - tile['row'] // 2 for tile in map_data['tiles'])
        min_r = min(tile['row'] for tile in map_data['tiles'])
        frames = []
        previous = None
        for index, context in enumerate(contexts):
            height, width = (int(x) for x in shapes[index])
            site = (post_row - min_r) * width + (post_col - post_row // 2 - min_q)
            cell_count = height * width
            frame = {'round': context['context']['round']}
            for name, channel in channels.items():
                frame[name] = round(float(values[int(offsets[index]) + channel * cell_count + site]), 6)
            frame['enemyHoldTurns'] = round(frame['opponent_hold_turns_remaining'] * 30)
            if frame != previous:
                frames.append(frame)
                previous = frame
        assert frames[0]['enemyHoldTurns'] == 6 and frames[0]['opponent_hold_objective'] == 1
        assert frames[0]['enemy_owned'] == 1 and frames[0]['capture_goal'] > 0, frames[0]
        assert any(frame['enemyHoldTurns'] == 5 for frame in frames)
        if case['landscape'].endswith('interrupt'):
            assert any(frame['enemyHoldTurns'] == 6 and frame['enemy_owned'] == 0 for frame in frames)
        samples.append(dict(config=case, frames=frames))
    write(OUT / 'confirmation/observation-evidence.json',
          dict(spatialSchema=native.SPATIAL_SCHEMA, post=dict(col=POST[0], row=POST[1]),
               channels=channels, denominator=30, samples=samples,
               nativeFingerprints=json.loads(native.fingerprints())))


if __name__ == '__main__':
    main()
