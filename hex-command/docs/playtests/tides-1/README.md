# First progression balance trials

All trials ran against commit `51372fd`. `results.json` was reconstructed by
replaying the recorded actions through that exact game implementation. These
artifacts are playtest evidence, not fixtures added to the unit test suite.

| Campaign | Recruitment policy | Chapter outcomes (winning turn or failure) |
| --- | --- | --- |
| Borrowed Tide | Mixed | 9, 6, 8; no retry; four optional losses |
| Borrowed Tide | Reefguards | Opening failure, then 6, 6, 9; three optional losses in winning chapters |
| Lantern Road | Mixed | 4; village stage failed twice, at 12 and 9 |
| Lantern Road | Swordsmen | 5; village stage failed twice, at 11 and 12 |
| Green Accord | Mixed | Opening failed twice, at 4 and 3 |
| Green Accord | Boars | Opening failed twice, at 3 and 2 |
| Ashen Oath | Mixed | Opening failed, then 6; refuge failed twice, at 7 and 10 |
| Ashen Oath | Skeletons | 5, 6; finale failed twice, at 7 and 8 |

Policies restrict new recruitment, not the authored starting/story cast. Promotions
of permitted recruits are allowed. The same starting gold, army, map, enemy rules
and one-retry allowance apply to each pair. Each trial used an independent Luna
agent choosing real commands, not a replacement player AI. This small sample is
sensitive to tactical mistakes and cannot establish universal strategy dominance.

The skeletal finale retry has no attacks after an invalid command batch; it is
retained transparently but excluded as useful balance evidence. Other attempts
also contain exposed couriers and poor crossings. Completed coastal results favor
mixed forces on first-attempt completion, while mono recruitment spent less on
optional casualties. They do not prove that every mixed composition is optimal.

The next pass separates early rookies from heavy guards, strengthens the human
archer's bow relative to the cheap scout's sling, lowers the starting skeleton's
blunt attack while retaining the veteran upgrade, gives rookie infantry one more
HP, and makes adepts available when Lantern first needs a ranged bone counter.
Green's first patrol uses spear recruits and has its forward bowman one hex back.
The next trials must determine whether those changes solve the pressure and role
problems, rather than assuming that passing mechanical checks means good balance.
