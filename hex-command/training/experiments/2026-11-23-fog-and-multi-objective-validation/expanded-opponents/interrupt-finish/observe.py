"""Audit actual frozen observations for both seats and returning fog."""
import hashlib
import itertools
import json
import os
import sys

from build import CATALOG, OUT, POST, RUN, STARTS, VISIBILITY, arena, location, read, write
from evaluate import DISCOVERY, configuration


def site(col, row, width, height):
    return row * width + col - row // 2 + (height - 1) // 2


def facts(board, col, row, channels):
    flat_site = site(col, row, board.shape[2], board.shape[1])
    return {name: round(float(board[index].flat[flat_site]), 6) for name, index in channels.items()}


def main():
    os.environ['HEX_COMMAND_DATA_ROOT'] = str(CATALOG)
    sys.path.insert(0, str(RUN / 'native'))
    import hex_command_training as native

    channels = {name: native.SPATIAL_CHANNEL_NAMES.index(name) for name in (
        'tile_remembered', 'tile_in_sight', 'friendly', 'hostile', 'enemy_owned', 'defeat_goal',
        'opponent_hold_objective', 'opponent_hold_turns_remaining')}
    seat_coords = {'player': {'col': 11, 'row': 7}, 'enemy': None}
    opening_digests = {}
    rows = []
    for start, visibility, observer in itertools.product(STARTS, VISIBILITY, seat_coords):
        config = configuration((start, visibility, 'queued'), 'queued', DISCOVERY[0])
        scenario = location(CATALOG, arena(start, visibility))
        roster = read(scenario / 'roster.json')
        target = seat_coords['player'] if observer == 'enemy' else roster['factions'][1]['units'][0]['tile']
        _, observations, contexts = native.collect_spatial_sequences([json.dumps(config)], 120, observer)
        values, offsets, shapes, *_ = observations
        frames = []
        first_seen = {}
        return_example = None
        sight_index = channels['tile_in_sight']
        remembered_index = channels['tile_remembered']
        for index, decision in enumerate(json.loads(contexts)):
            context = decision['context']
            height, width = map(int, shapes[index])
            board = values[offsets[index]:offsets[index+1]].reshape(len(native.SPATIAL_CHANNEL_NAMES), height, width)
            living_friendly = int(board[channels['friendly']].sum())
            assert living_friendly > 0, (observer, context)
            context = dict(context, friendlyUnits=living_friendly)
            frame = dict(round=context['round'], decision=context['decision'], friendlyUnits=living_friendly,
                         opposingStartingTile=facts(board, **target, channels=channels),
                         post=facts(board, **POST, channels=channels))
            if index == 0:
                opening_visible_tiles = int(board[sight_index].sum())
                opening_hostile_count = int(board[channels['hostile']].sum())
                opening_digests[start, visibility, observer] = hashlib.sha256(board.tobytes()).hexdigest()
            frames.append(frame)
            if return_example is None:
                for row in range(height):
                    for col in range(width - (height - 1) // 2):
                        slot = site(col, row, width, height)
                        if board[sight_index].flat[slot] == 1:
                            first_seen.setdefault((col, row), context)
                        elif (col, row) in first_seen:
                            return_example = dict(tile=dict(col=col, row=row), revealed=first_seen[col, row],
                                                  hidden=context, remembered=int(board[remembered_index].flat[slot]))
                            break
                    if return_example:
                        break
        opening = frames[0]
        limited = visibility == 'both-fog' or visibility == 'player-fog' and observer == 'player'
        if observer == 'player':
            assert opening['post']['opponent_hold_objective'] == 1
            assert opening['post']['opponent_hold_turns_remaining'] == 0.2
            assert (opening['opposingStartingTile']['hostile'] == 0) == limited
            assert (opening['opposingStartingTile']['defeat_goal'] == 0) == limited
        assert (return_example is not None) == limited
        if return_example:
            assert return_example['remembered'] == 0
            assert return_example['revealed']['friendlyUnits'] > 0 and return_example['hidden']['friendlyUnits'] > 0
        rows.append(dict(start=start, visibility=visibility, observer=observer, target=target,
                         limited=limited, opening=opening, openingVisibleTileCount=opening_visible_tiles,
                         openingVisibleHostileCount=opening_hostile_count,
                         openingSpatialSha256=opening_digests[start, visibility, observer],
                         returningFogExample=return_example, frames=frames, config=config))
    for mode in ('player-fog', 'both-fog'):
        assert opening_digests['interrupt', mode, 'player'] == opening_digests['finish', mode, 'player']
    write(OUT / 'observation-evidence.json', dict(nativeSpatialSchema=native.SPATIAL_SCHEMA,
                                                  channelIndices=channels, cases=rows))
    print('Verified 12 native seat/start/sight combinations.')


if __name__ == '__main__':
    main()
