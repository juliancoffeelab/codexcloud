# Objective distribution

Reference sample: **272 battle chapters (source files/templates), containing 434 classified objective tasks**. Counts describe the relative prevalence of task kinds and the roles they serve. Use them as a rough reference when choosing objectives; the campaign's situation determines which tasks belong. The table is not a checklist or a quota for each campaign.

## Roles

- **One winning objective:** the chapter has one finishing goal; complete it and win.
- **Objective in alternative:** the chapter has several ways to finish; complete any one and win. This includes alternatives that lead to the same subsequent story.
- **Part:** complete this task in combination with other required tasks to win.
- **Intermediate step:** complete one or more earlier required tasks, then proceed to a finishing goal, an alternative, or a combination of parts.
- **Optional bonus:** complete zero or more optional tasks for additional rewards or opportunities.

An objective's role depends on the scenario's completion logic. A displayed objective can be an intermediate step or a bonus; displaying it does not make it a sufficient victory condition. A narrative consequence such as liberating a village can result from a defeat-enemies objective.

## Counts

Each cell counts **logical tasks**, merging repeated descriptions, counter updates and equivalent difficulty/configuration variants. A chapter with three alternative finishing goals contributes three tasks to the alternative column. A victory requiring two different tasks contributes two parts. A task receives one kind and one role.

| Goal kind                         |   Total | One winning objective | Objective in alternative |   Part | Intermediate step | Optional bonus |
| --------------------------------- | ------: | --------------------: | -----------------------: | -----: | ----------------: | -------------: |
| **All**                           | **434** |               **225** |                   **64** | **21** |            **80** |         **44** |
| Defeat enemies                    |     225 |                   138 |                       34 |     11 |                20 |             22 |
| Army escape                       |      47 |                    35 |                       10 |      1 |                 1 |              0 |
| Hold / survive / protect          |      50 |                    22 |                       13 |      1 |                 5 |              9 |
| Reposition units to X             |      33 |                    10 |                        2 |      0 |                17 |              4 |
| Control territory / gather wealth |       9 |                     3 |                        1 |      3 |                 1 |              1 |
| Acquire / deliver items           |      12 |                     4 |                        2 |      0 |                 6 |              0 |
| Free / rescue people              |      15 |                     2 |                        2 |      1 |                 4 |              6 |
| Evacuate units                    |       6 |                     2 |                        0 |      2 |                 0 |              2 |
| Capture units                     |       3 |                     2 |                        0 |      1 |                 0 |              0 |
| Find                              |      15 |                     2 |                        0 |      0 |                13 |              0 |
| Explore                           |       6 |                     0 |                        0 |      0 |                 6 |              0 |
| Pursue                            |       2 |                     1 |                        0 |      0 |                 1 |              0 |
| Negotiate                         |       1 |                     0 |                        0 |      0 |                 1 |              0 |
| Transport cargo / materials       |       4 |                     2 |                        0 |      1 |                 1 |              0 |
| Operate / destroy structures      |       6 |                     2 |                        0 |      0 |                 4 |              0 |

## Reading the distribution

- **How common is a task kind?** Divide its row total by 434. Defeating enemies accounts for 225/434, about 52% of classified tasks.
- **How is a kind usually used?** Divide its cells by its row total. Of 50 hold/survive/protect tasks, 22 (44%) are sole winning goals and 13 (26%) are alternative victories.
- **What kinds usually serve a role?** Divide by the column total. Of 225 sole winning goals, 138 (about 61%) involve defeating enemies.

The generic scripted/tutorial bucket is excluded from these task counts. The chapter reference count remains 272.

The chapter count provides scale. Task counts alone do not establish the probability that a chapter contains a kind, or which tasks appear together. A chapter can contain several tasks of the same kind.

## Kind distinctions

- **Find:** locate a target whose whereabouts are unknown. The target can be a person, group or exit.
- **Explore:** investigate a specified territory; discovering what is there is the task.
- **Pursue:** catch up with or intercept a moving target before it gets away or reaches its destination.
- **Negotiate:** persuade another actor or reach an agreement through an authored interaction. The counted example uses repeated contacts and scripted dialogue.
- **Army escape:** reach an exit or passage that advances or withdraws the company as a whole.
- **Evacuate units:** extract particular units to safety; preserving those units is the task.
- **Reposition units to X:** move specified units to a contact, staging area, formation or other tactical destination.
- **Free / rescue people:** release captives. Subsequent extraction can be a separate evacuation task.
- **Capture units:** take opposing units alive through an authored capture condition. Capture occurs only three times among the 434 classified tasks.

Known-location meetings belong under repositioning. Split compound tasks into their actual stages: find a character, negotiate their cooperation, then evacuate them. Count equivalent descriptions of the same stage once.

These are completion goals. Their implementation can use movement, combat, pickup/delivery or scripted conditions. Defeating an enemy can include an authored surrender.

For primary achievements after finishing the chapter, use the separate [result distribution](result_distribution.md), split into main and alternative finishing results.
