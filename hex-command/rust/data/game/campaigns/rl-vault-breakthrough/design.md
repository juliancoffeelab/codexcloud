# Vault Breakthrough

A targeted, one-chapter RL situation derived from an actual failed advance.
Source: Underground Escape opening, seed 16010718, turn 17, PPO hour 9.
The source trace and complete comparative runs are retained under
`tmp/rl/objective-stall-env/`. No held-out campaign supplied this encounter.

The terrain and surviving forces come from the source position. The map is
cropped to columns 4–19 and rows 8–27, then translated to a 16×20 board.
Ownership, health, promotions, equipment, allegiance and controller rules remain.
Turn/action history resets as a new episode; the source RNG is replaced by the
evaluation/training seed. This is an extracted encounter, not an exact save replay.

Reach (10,2) with any player unit before turn 25. Losing the commander loses
the operation. There are no scripted movement overrides or inactive defenders.
Both sides have clear visibility. Keep the original economy and legal actions.

Twenty matched seeds against Predictor verified a feasible crossing and
reproduced prolonged waiting in weaker policies. Cropping changes behavior:
validation applies to this extracted map, not to the historical run.
It is retained only as a simulation regression fixture. It was removed from
targeted training because the final hour9 checkpoint already solves20/20.
