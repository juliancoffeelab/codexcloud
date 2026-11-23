# Exact matched teacher scores for existing targeted maps

All528 teacher games pair exactly with the learner configs from the fresh hour10 audit: same seed, map, objective route, enemy blocks, player recruiter, diplomacy, recruitment limit and round cap.66 conditions ×8 seeds, all complete; zero caps. Reproduced the24 previously recorded teacher results including terminal round. No training or production code changes.

Teacher victories447/528 versus learner383/528. Equal-map averages85.99% versus70.55%. Teachers are worse on seven maps, equal on six and better on eleven. Aggregated rows pool each map’s configured conditions, not different comparison panels.

| Map ID | Hour10 | Configured teacher | Teacher wins |
|---|---:|---|---:|
|rl-urgent-crossing|12/16|objective_progress|10/16|
|rl-deadline-wait7|8/8|objective_progress|7/8|
|rl-deadline-pines-relief8|0/24|objective_progress|16/24|
|rl-route-ambush|0/8|full|8/8|
|rl-route-pocket-leader|7/8|queued|8/8|
|rl-route-pass-road|0/8|queued|8/8|
|rl-escort-lantern-maze|24/24|full|24/24|
|rl-escort-mistfen-patrol|24/24|full|24/24|
|rl-escort-splitwake-patrol|24/24|full|24/24|
|rl-defend-tidegate-stand-110|31/32|full|21/32|
|rl-defend-bracken-stand-010|26/40|full|34/40|
|rl-interrupt-siltbar-approach|6/8|path_to {"col": 13, "row": 5}|8/8|
|rl-interrupt-herring-tide-road-three|4/8|full|8/8|
|rl-defend-lower-gallery-0|0/8|full|3/8|
|rl-interrupt-south-ford-west-post|8/8|full|8/8|
|rl-briar-saddle|7/8|queued|6/8|
|rl-whitecap-ford|22/24|queued|24/24|
|rl-slate-gallery|37/48|queued|39/48|
|rl-breakthrough-cave5|8/8|full|8/8|
|rl-breakthrough-ford-heal08|8/8|full|8/8|
|rl-bellmarket-courts|32/32|classic|28/32|
|rl-thorn-verge|72/72|classic|68/72|
|rl-lair-outworks|18/72|full|51/72|
|rl-vault-rally|5/8|classic|4/8|

Named Classic/Queued/Full here identify planner selection with the same Situational recruiter and Mission Campaigner as the learner panel; they do not silently replace the full tactic. Custom objective-progress and path-to teachers use the actual current native implementations and legal faction-observed choices.

Splitpine teacher by enemy, all clear:Classic7/8,Predictor5/8,Full4/8. Echo Gate clearClassic8/8; North of Lantern Road player-onlyfogClassic8/8; Lower Gallery clearClassic3/8. All four corresponding learner rows are zero.

This supports trying imitation on several zero-score cases, but does not justify treating every teacher as stronger. Tidegate teacher21/32 versus learner31/32; Bellmarket28/32 versus32/32; Thorn Verge68/72 versus72/72. Any new imitation run still needs retention evaluation; no such run was launched. The separately measured new multi-objective teacher scores remain in the fog/multi-objective archive.
