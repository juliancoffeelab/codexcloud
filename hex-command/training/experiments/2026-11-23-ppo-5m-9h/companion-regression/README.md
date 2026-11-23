# Four Lantern crossing commander-loss regressions

Replayed the four fixed cases that passed the crossing at hour8 and ended in
leader_loss at hour9. All reached-chapter winners and terminal rounds match the
original evaluation. Frozen run catalog and the corresponding native planners
were used. This is an eight-game diagnosis, not a new strength panel.

All four terminal casualties were Nera, the swordsman commander.
- Seed480000010, turn18: after remaining at (6,4) through turn16, Nera retreats
  twice but the revenant follows and kills her (25→16→8→dead).
- Seed480000013, turn18: Nera retreats south on turns14–16, then takes no actions
  on17–18 as two revenants catch and kill her (25→11→dead).
- Seed480000018, turn46: Nera remains at (5,6) after turn2, no player actions
  on22–46, then a giant spider moves in and kills her at4HP.
- Seed480000024, turn55: Nera remains at (5,6) after turn2 (attacks once on5);
  no player actions on17–55. A giant spider attacks on53/54/55:7→4→1→dead.

The rest of the player army was already dead before these final phases.
These traces establish behavior and damage sequences, not whether a specific
alternative action would have guaranteed survival or victory. Full phase states,
actions, combats and losses are retained in gzip JSONL files.
