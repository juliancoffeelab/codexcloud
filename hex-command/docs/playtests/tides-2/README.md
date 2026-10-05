# Second balance pass

These are independent Luna runs on the frozen `a1c60ef` rules. Each transcript was
replayed by that checkout's actual game engine; [results.json](results.json)
contains the resulting attempts, casualties, recruitment and promotions. No failed
attempt was removed. Two sword-only Lantern runs are still in progress and will
be added when they stop.

Policies restrict new recruitment. Starting troops remain usable, so "one type"
does not mean deleting the authored army. Each failed chapter permits one retry;
survivors must be earned and recalled through the ordinary transition.

The skeleton recruitment run completed Ashen Oath; the mixed run failed the refuge
twice. Both used their recruited units in combat. This result triggered another
balance change: bone archers now cost 7 instead of 8, allowing the 14-gold opening
to buy a skeleton and a ranged troop. A fresh matched pair will test that change.
This is a hypothesis to test, not proof that the problem is fixed.

The woodland opening defeated both policies. The boar-only agent made no boar
attacks before dying, so that run is particularly weak evidence about offensive
strength. Guards subsequently gained an authored objective-defense stance rather
than chasing the starting party away from the camps. Fresh runs use those rules.

The first mixed Lantern run lost its courier early. The additional `round2b` run
used an explicit required-unit safety protocol and still failed both attempts.
Its conservative threat bounds encouraged prolonged retreats: possible damage
from independently positioned enemies is not the damage the actual AI will deal.
New comparisons have a read-only actual enemy-turn forecast available to both
policies. The earlier results remain here, with that limitation visible.

Each policy is one agent sample, with different tactical decisions and carryover.
These reports do not establish a universal ranking or a statistical win rate.
