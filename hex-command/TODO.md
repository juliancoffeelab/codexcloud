# PR followups

Updated on 2026-10-04 for [PR #6](https://github.com/juliancoffeelab/codexcloud/pull/6).
The original requests came from [PR #5](https://github.com/juliancoffeelab/codexcloud/pull/5)
and subsequent playtest feedback. Discussion comments do not have a resolution flag.

Sources:

- [Original TODO and design followups](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5973837666)
- [Playtest issues](https://github.com/juliancoffeelab/codexcloud/pull/5#issuecomment-5980397700)

## Completed gameplay and UI fixes

- [x] Visit rewards fire on a player entering the named hex, once per mission.
      Starting occupancy, unrelated moves, turn changes and reloads do not count.
      The Lantern finale's reward grove is now at unoccupied `5,1`, beside the
      northern bridge, rather than underneath the starting adept at `4,3`.
      Old saves keep units, gold and earned rewards when the site moves. Checks
      cover visit placement, arrival, starting occupancy, undo and repeat visits.
- [x] Lantern Road's final binding stones require adepts. Other campaigns retain
      their existing sealing rule. Objective text and contextual terrain help
      describe the required unit and sealed state.
- [x] Permanently completed stones and ledger objectives lose their active map
      markers. Reversible control/escort/hold goals remain marked until battle ends.
- [x] Leader and required-companion survival objectives show completion at victory.
- [x] Dain cannot be recalled as a duplicate unnamed recruit when the following
      chapter already includes him as a story companion.

## Completed pixel work

- [x] All 26 faction unit frames and 12 terrain frames have editable 48×48 matrices.
      One unit atlas and one terrain atlas in `public/assets` are rendered from
      those sources and used by the ordinary game renderer.
- [x] Wolves and skeleton warriors have dark silhouette outlines, retaining their
      approved poses. The boar has darker hide; thorncaller fur has saturated orange.
- [x] Correct material artifacts in the transcriptions, including smooth gold trim,
      cloak/cloth planes, thorncaller paws and armored boots. Remove the green flecks
      inside the keep's stonework and grey specks from the fort's wooden fence.
      Plains grass tufts use coherent flat regions. Latest renders were inspected
      on dark backgrounds and on the map; matrix/palette equality alone is not
      evidence of correct material placement.
- [x] Civilian speakers use a neutral worker portrait, stored as an editable matrix
      and packed in the shared unit atlas. It is a portrait, not a combat unit.
      Reed gatherers and the village elder no longer borrow the scout design.
- [x] Pixel units use integer SVG crops with transparent atlas gutters, removing
      the oversized HTML image/compositor layer from the pixel rendering path.

## Completed woodland campaign rewrite

Story and mechanics are specified in [design/green-accord.md](design/green-accord.md).
The user explicitly authorized rewriting this campaign; that authorization does
not extend to rewriting the other campaigns.

- [x] The first mission captures both logging supply camps and recovers the missing
      workers' pay ledger. Merely standing at cutting markers no longer wins.
- [x] The Overseer is the council's timber/toll officer. He diverts wages and hands
      protesting workers to the Rootbinder, who kills and binds them. The bargain
      gives the Overseer unpaid labor and the Rootbinder bodies and river access.
- [x] Dain's personal truce covers his patrol. Other charter guards attack the vigil
      to suppress evidence. Living Dain is present as a playable ally in stage two;
      his death persists into later stages and changes the story.
- [x] The vigil produces the workers' memories in a living branch. Stage three
      secures a crossing and escorts Lark to the council, which revokes the charter.
      Tamsin joins here. The ending does not make an undefeated Rootbinder disappear.
- [x] Old conversations remain recognizable to the save loader; earned victories
      remain earned. Old vigil saves gain living Dain without resurrecting a Dain
      whose death was recorded. Current-stage deaths and retries are checked.

## Still requiring design input

- [ ] Review inconsistent low-poly artwork with the user before replacing designs.
- [ ] Clarify Dain's allegiance in the skeleton campaign, preserving the praised
      immortality/service exchange and the undecided future conversation.
- [ ] Clarify why lighting the Lantern Road lanterns forces the necromancer away.
- [ ] Review the remaining optional quest rewards with the user. The woodland pay
      ledger now serves a required evidence objective and has no arbitrary gold bonus.
- [ ] Design unit progression and branching advancement, beginning with peasant →
      soldier → heavy infantry, before adding progression rules or new unit artwork.
- [ ] Review resistance/attack/ability balance with the user. These are design
      choices, not permission for blanket stat increases.

## Pending device confirmation

- [ ] Diagnose the reported blank mobile startup from the on-device error trace.
      The startup panel now displays script-load errors, runtime backtraces and
      rejected promises until the app has mounted. Reloading preserves the saved
      battle and requests fresh HTML. Local client mounting succeeds; the phone's
      failure cause has not been established.
- [ ] Confirm the reported mobile stray lines on the user's phone after the SVG
      crop change. Integer bounds, transparent gutters and SVG-only pixel rendering
      are verified in source; they cannot establish the phone compositor's result.
      Repository instructions prohibit browser automation/screenshot suites.

## Earlier fixes retained

Withdrawal based on lethal retaliation; one-movement forest/grove travel for
woodland beasts; numeric terrain movement costs plus unit modifiers; matching
dialogue navigation arrows with no skip button; developer showcase as an ordinary
map listed last in the picker. These are separate from the fixes above.
