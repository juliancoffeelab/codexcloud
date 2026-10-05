# Sixth coastal comparison

Both complete Luna campaigns replay against frozen commit
`8259b39d94369c2067bb78190e5d2c68fbd3c025`. Each policy uses the authored
starting company. The restricted policy buys only rookie reef guards; the mixed
policy chooses among the full unlocked recruit list.

| Recruitment     | Chapter turns | Casualties | Retries | Named survivors   |
| --------------- | ------------- | ---------- | ------- | ----------------- |
| Mixed           | 8 / 6 / 8     | 6          | 0       | Mira, Edda, Tomas |
| Reef guard only | 11 / 6 / 8    | 3          | 0       | Mira, Edda, Tomas |

The mixed army was faster in the opening, but lost more soldiers. This is a
tradeoff, not proof that mixed recruitment is better overall. The records retain
the avoidable commitments and actual casualties. Veteran Impact spells helped
against armor; several wounded ranged soldiers were left exposed.

The next revision gives wavemenders one additional HP while retaining their
weak melee and woodland exposure. It also replaces absolute-round evacuation
progress with three defended ferry crossings and a minimum high-tide round of
six. These transcripts validate neither revision; a fresh matched pair is
required. Individual attacks, purchases and promotions are in `results.json`;
all actual commands and enemy responses are in the accompanying trial files.

## Developer Ashen route study

`root-ashen6-study.json` uses the same frozen source and replays all 186 recorded
decisions. It completed the quarry in six turns, the peaceful refuge in six, and
the final confrontation in eight, with two ordinary-unit casualties overall.
Dain survived and the medicine shortened the refuge defense; neither Dain nor
retreating soldiers were attacked. The finale recruited all four available roles
and used spears, ranged spells, bone bows and melee skeletons together. Earned
promotions included revenants, a death archer, Orin's lich and a stone mage.

This is developer viability evidence, not an independent balance comparison.
The developer knows the rules and had read previous trials. Mistaken move/range
requests retained their earlier successful actions; no transcript was rewritten.
The finale's lost revenant followed an exposed attack against the Master on a
hill. Independent Luna outcomes remain the authority for the recruitment-policy
comparison.

## Independent Ashen mixed attempt

`round6-ashen-mixed.json` replays all 219 decisions against the same frozen
source. The quarry failed on turn eight, then succeeded on turn six. The refuge
failed on turns six and ten, exhausting its allowed retry. The campaign did not
reach the finale. Those failures are retained; the developer's successful route
does not replace them.

The player initially interpreted the mediation rule as prohibiting all attacks
against Dain and withdrawing soldiers. The actual rule prohibits killing them,
including lethal retaliation. Clarification arrived during the refuge retry;
the companion restricted trial received it at a different point. This limits
comparison of the two old refuge attempts. Fresh players receive the same lethal
boundary from the outset, on the revised rookie-reinforcement source.

The completed skeleton-only companion record, `round6-ashen-skeleton.json`,
replays all 232 decisions. It won the quarry on turn eleven with only Orin
surviving, lost its first refuge attempt on turn ten, then won peacefully on
turn six and completed the finale on turn eight. Orin and Dain survived the
finale. Its two final-chapter casualties and earlier losses remain recorded.
That completion, compared with the mixed player's exhausted refuge retry,
reinforces the need for the new matched revision; neither its earlier troop
losses nor the unequal clarification timing is omitted.
