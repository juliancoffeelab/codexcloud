# Marlow Cut under fog and different opponents

**The confirmed Hour 10 gap is in Finish against Classic with clear sight.** Queued won 20/20, Hour 10 won 14/20 and Hour 8 won 15/20 on fresh matched seeds. Queued won every game by defeating the commander. The learners won by holding the post and lost when Classic recaptured it for six rounds. The Hour 8 gap was five wins, below the six-win confirmation threshold. [Confirmation rows](confirmation/games.jsonl) · [Qualification and fixture paths](final-summary.json)

The original comparison held Classic fixed to isolate the effect of the enemy commander's position. This expansion keeps the same forces, terrain geometry, initial ownership and six-round victory rule, and changes opponent composition and vision. Classic uses its native blocks. Queued uses Balanced recruitment and the Queued planner. Predictor uses Predictor recruitment and the Queued planner. Full uses Predictor recruitment and the Full planner. All opponents use the Mission campaigner. Recruitment remains disabled and earned promotions can still occur. [Exact opponent blocks, map and model hashes](discovery/protocol.json)

## Eight-seed discovery

The 960-game matrix uses matched seeds 661136001–008. Each cell below lists wins out of eight in the order **Hour 8 / Hour 10 / Queued / Full / Classic**. All games reached a rules-defined terminal result. [Whole-game rows and terminal reasons](discovery/games.jsonl) · [Selection](selection.json)

| Sight | Opponent | Interrupt | Finish |
| --- | --- | --- | --- |
| Clear | Classic | 8 / 8 / 8 / 8 / 8 | 5 / 5 / 8 / 8 / 8 |
| Clear | Queued | 8 / 8 / 8 / 8 / 0 | 8 / 8 / 8 / 8 / 8 |
| Clear | Full | 8 / 8 / 8 / 8 / 0 | 8 / 8 / 8 / 8 / 8 |
| Clear | Predictor | 8 / 8 / 8 / 8 / 0 | 8 / 8 / 8 / 8 / 8 |
| Player-only fog | Classic | 8 / 8 / 8 / 8 / 8 | 8 / 8 / 8 / 8 / 8 |
| Player-only fog | Queued | 7 / 7 / 7 / 8 / 8 | 7 / 7 / 7 / 8 / 8 |
| Player-only fog | Full | 7 / 7 / 7 / 8 / 8 | 7 / 7 / 7 / 8 / 8 |
| Player-only fog | Predictor | 7 / 7 / 7 / 8 / 8 | 7 / 7 / 7 / 8 / 8 |
| Both-side fog | Classic | 8 / 8 / 8 / 8 / 8 | 8 / 8 / 8 / 8 / 8 |
| Both-side fog | Queued | 7 / 8 / 7 / 8 / 8 | 7 / 8 / 7 / 8 / 8 |
| Both-side fog | Full | 7 / 8 / 7 / 8 / 8 | 7 / 8 / 7 / 8 / 8 |
| Both-side fog | Predictor | 7 / 8 / 7 / 8 / 8 | 7 / 8 / 7 / 8 / 8 |

A reference needed at least six discovery wins and a lead of three over Hour 10. Finish with clear sight against Classic was the sole qualifying cell. Queued was selected before running confirmation seeds 661136101–120. Both learners won 7–8 of eight games in every fog cell, so the fog cells stayed at discovery coverage. [Screening and confirmation thresholds](selection.json)

## What changed the outcome

On discovery seed 661136002, Hour 10 opened toward the watch post in Finish against all four opponents. Classic's bone archer fired repeatedly at the knight, killed it on round 5, retook the post on round 7 and won on round 13. The other three opponents initially moved their archer onto the post. Hour 10 killed the archer, captured on round 3 and won on round 8. In confirmation seed 661136101 against Classic, Queued took the eastern approach and defeated the commander on round 2. Both learners took the western approach and lost to the renewed hold. [Matched actions and post history](traces/objective-progress.json)

Under radius-three fog, both player openings produced byte-identical native spatial observations across the pair. The public watch-post objective still showed six rounds remaining, while the enemy commander and its defeat target were hidden. The limited player saw 51 of 475 tiles at its first decision. A limited enemy saw 67–81 tiles at its first decision. Each limited seat later had a visible tile return to fog with friendly observers still alive. Clear-sight seats saw 475 tiles at the opening and had no such transitions in these replays. This fog variant delays the information that distinguishes the two starts. [Frozen encoder and returning-fog observations](observation-evidence.json)

Queued and Predictor had identical terminal winner, cause and round in all 240 matched discovery pairs. Twelve sampled pairs also had identical actions and post history. Each opponent made an earned promotion in the same sampled game despite the zero recruitment limit. The pilots do not establish equivalence outside this fixture. [Replay protocol and promotion audit](final-summary.json) · [Raw phase records](traces/raw.jsonl.gz)
