# Fork frequency, placement and effects

Working audit of **19 campaign story families**, using their source-cited chapter records and targeted script checks at revision `39f64458e811dbf98f1fdaf03ea2b30a4269cfda`. Counts describe identified authored opportunities, not player-choice probabilities or an exhaustive census of every handler. The duplicate classic campaign and procedural generator are excluded.

## What counts

- **Route fork:** a deliberate decision/action selects different playable battles, inserts/skips a battle, or selects a different finale.
- **Local battle fork:** the chapter sequence stays fixed, but an authored choice changes the tactical approach, agreement, opposition or objectives.
- **Recorded separately:** casualty/state-dependent progression, ordinary reward assignments, optional bonus maps, ordering hubs and terminal codas. A second way to satisfy the same finishing goal is not automatically a story fork.

A point counts once per decision opportunity, not once per leaf. Nested points need their parent choice and do not occur on every playthrough. The two South Guard decisions occur in the same chapter.

## Frequency

| Measure            | Campaigns | Share of 19 | Decision points |
| ------------------ | --------: | ----------: | --------------: |
| Finite route forks |         5 |       26.3% |               9 |
| Local battle forks |         4 |       21.1% |               8 |
| Either kind        |         8 |       42.1% |              17 |

The campaign sets overlap. Current Heir to the Throne also uses an expedition network; its repeated map selections have no single fixed fork count. The Rise of Wesnoth has a four-task ordering hub in addition to its route fork. Neither system is counted as many independent binary forks.

## Campaign position

| Point falls in        | Route points | Local points |
| --------------------- | -----------: | -----------: |
| Early                 |            4 |            0 |
| Early–middle boundary |            1 |            1 |
| Middle                |            3 |            2 |
| Late                  |            1 |            0 |
| Variable / hub order  |            0 |            5 |

Bands refer to normal-route battle progress: early/middle/late thirds, with boundary cases retained. Nonbattle scenes, optional tutorial, exceptional early endings and bonus codas are excluded from those position estimates. Hub scenario numbers do not identify their played order. These are placement examples, not a schedule or quota.

| Decision timing | Points |
| --------------- | -----: |
| During battle   |     11 |
| After battle    |      4 |
| Before battle   |      1 |
| Opening battle  |      1 |

The late route decision selects a different final battle **before** that battle is played; it is not a choice made after the finale. Consequences can occur later than their setup: the rescued/surviving sisters in Northern Rebirth determine an alliance and a subsequent battle skip.

## Gameplay effects

Effects overlap. Each cell counts decision points with that identified effect, not all consequences or affected chapters.

| Effect                                             | Route points (9) | Local points (8) |
| -------------------------------------------------- | ---------------: | ---------------: |
| Replace one or more battles                        |                7 |                0 |
| Change available forces / recruitment              |                8 |                5 |
| Change opposition                                  |                8 |                1 |
| Use a new or changed battlefield layout            |                9 |                2 |
| Insert / skip a battle                             |                2 |                0 |
| Change gold, income or carried items               |                2 |                4 |
| Change objectives, targets or victory requirements |                5 |                3 |
| Change the finale                                  |                3 |                0 |
| Change starting positions or unit entry timing     |                2 |                2 |
| Change diplomacy                                   |                3 |                5 |
| Change a map variant / route on the same chapter   |                1 |                2 |

Deployment means authored starting keeps, positions or entry timing; ordinary XP differences do not count. Availability of a new character alone belongs under forces, not deployment. Some effects are conditional on rescues or surviving companions.

## Route points

| Campaign / decision                                                                                                                                                                                                    | Position              | What changes                                                                                                                                                |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Eastern Invasion — Which sorcerer falls](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/03_An_Unexpected_Appearance.cfg#L19)              | Early                 | Northwest: elven forest encounter, then outpost / East: swamp/herb encounter, with a conditional capital detour, then outpost                               |
| [Eastern Invasion — Capital detour](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/04b_Ill_Humors.cfg#L902)                                | Early                 | Collect herbs and pay: assault the capital and potentially rescue its knights / Decline: return west without the capital battle                             |
| [Eastern Invasion — Eastern or western crossing](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/05_Northern_Outpost.cfg#L11)               | Early–middle boundary | Undead Crossing then Capturing the Ogres / Soradoc then Ogre Crossing; saved garrison and surviving enemies change later rewards/pursuit                    |
| [Eastern Invasion — Duel or army finale](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/16_Eleventh_Hour.cfg#L19)                          | Late                  | Duel with a selected force / All-In with the army; defeat liches to expose the enemy leader                                                                 |
| [Under the Burning Suns — Dwarves or trolls](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Under_the_Burning_Suns/scenarios/05_A_Subterranean_Struggle.cfg#L1142)    | Middle                | Help dwarves, fight trolls; play the troll-domain battle and dwarven audience / Help trolls, fight dwarves; play the dwarf-domain battle and troll audience |
| [The South Guard — Trust Ethiliel or Afalas](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_South_Guard/scenarios/04_Choice_in_the_Fog.cfg#L490)                  | Middle                | Ethiliel: defeat Afalas; rear-guard battle followed by the undead assault / Afalas: defeat Mebrin now; forest withdrawal followed by the elven siege        |
| [The South Guard — Enter the cave](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_South_Guard/scenarios/04_Choice_in_the_Fog.cfg#L642)                            | Middle                | Continue the promised fight against Afalas / Enter the cave: Ethiliel leaves; play Into the Depths and a different configuration of Vengeance               |
| [The Rise Of Wesnoth — Swamp or Midlands](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_Rise_Of_Wesnoth/scenarios/03_A_Harrowing_Escape.cfg#L6)                  | Early                 | Swamp of Esten, with its encounters and potential mage companion / Midlands, with its different army/terrain and companion route                            |
| [Son Of The Black Eye — Fight the main human army or withdraw](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Son_Of_The_Black_Eye/scenarios/01_End_of_Peace.cfg#L11) | Early                 | Withdraw directly toward the mountains / Fight The Human Army first, then continue toward the mountains                                                     |

Most routes reunite after one or two changed battles. Eastern Invasion’s duel/army decision replaces the finale. The South Guard’s trust choice changes the following two battles; entering the cave is a second decision available only on the Ethiliel branch.

## Local points

| Decision                                                                                                                                                                                                                  | What the player faces differently                                                                                                                           |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [Heir To The Throne — Approach Elensefar from north or south](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Heir_To_The_Throne/overworld/00a_western_wesnoth.cfg#L1730) | North: Konrad’s keep at 29,6; thieves use the ambush plan / South: company near 35,28; a ford-versus-ambush choice is available                             |
| [Heir To The Throne — Ford or thieves’ ambush](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Heir_To_The_Throne/scenarios/11_Elensefar_After.cfg#L472)                  | Reveal the hidden ford for the army’s approach / Signal thieves to attack from inside the city by taking a village                                          |
| [Heir To The Throne — Work with Burlin or fight him](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Heir_To_The_Throne/scenarios/14_Gryphon_Mountain.cfg#L290)           | Kill the matriarch: recruit Gryphon Riders and Dwarvish Scouts / Fight Burlin: Asheviere loses income and cannot recruit Gryphon Riders in the final battle |
| [Heir To The Throne — Stoneskin bargain](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Heir_To_The_Throne/scenarios/22_Glamdrol.cfg#L418)                               | Pay for a truce / Pay more for fighters / Continue fighting without the bargain                                                                             |
| [Heir To The Throne — Resolve the Sceptre contest](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Heir_To_The_Throne/scenarios/30_The_Sceptre_of_Fire.cfg#L869)          | Hold for excavation and acquire the Sceptre / Defeat/capture Li’sar and acquire the Sceptre / Escape without the Sceptre                                    |
| [Eastern Invasion — Dolburras negotiates dwarf peace](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/08_Xenophobia.cfg#L58)                   | Bring Dolburras to Pelathsil and end the dwarven hostility / Proceed while the dwarves remain hostile                                                       |
| [The Deceivers Gambit — Pay the guides or fight them](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_Deceivers_Gambit/scenarios/08_Ruins_of_Saurgrath.cfg#L484)      | Pay for cooperation and poacher recruitment / Refuse and fight the bandits                                                                                  |
| [Winds of Fate — Resha’s naga pact](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Winds_of_Fate/scenarios/04_Journey.cfg#L560)                                          | Contact Shussek with Resha before any drake has killed a naga: ally with the nagas / Fight them or continue without the pact                                |

Concrete deployment cases: Elensefar puts the player’s keep at **29,6** from the north or the company near **35,28** from the south. The South Guard’s cave path selects **06c_Vengeance.map**, starts with town defenders under player control and delays Deoran’s arrival; its other route starts him outside town.

In Gryphon Mountain, cooperation unlocks Gryphon Riders and Dwarvish Scouts; fighting Burlin instead reduces Asheviere’s income and removes her Gryphon Rider recruitment in the finale. That is a carried military consequence, not a different chapter.

## Other branch mechanisms

| Mechanism                            | Reference case                                                                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Optional extra chapter               | [Dusk of Dawn](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Dusk_of_Dawn/scenarios/01_First_Steps.cfg#L879): The trapdoor offer inserts Secret (a bonus puzzle/golem encounter) before A Dream. Kept separate from the main military-route fork count.                                                                       |
| Outcome/state-driven chapter branch  | [Northern Rebirth](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Northern_Rebirth/scenarios/09a_Introductions.cfg#L96): Rescued/surviving sisters determine Eryssa’s alliance, later elves versus gold, and whether Get the Gold is played. The consequence resolves later than the choices and casualties that establish it. |
| Conditional post-campaign chapter    | [Eastern Invasion](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Eastern_Invasion/scenarios/18_Epilogue.cfg#L651): All-In and surviving named opponents can unlock Empire. Not an independently offered normal-route fork.                                                                                                    |
| Outcome-selected ending shortcut     | [The South Guard](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_South_Guard/scenarios/05b_Pebbles_in_the_Flood.cfg#L815): Killing Mebrin during the rear-guard battle goes straight to the epilogue rather than the later undead finale.                                                                                  |
| Repeatable coda / terminal ending    | [Descent Into Darkness](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Descent_Into_Darkness/scenarios/10_Endless_Night.cfg#L1306): Defeat heroes and repeat, or after enough iterations allow the book ending. Random Squidville and defeat endings are not player-authored fork options.                                     |
| Companion/carryover choice           | [The Rise Of Wesnoth](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_Rise_Of_Wesnoth/scenarios/02_The_Fall.cfg#L294): Accept the Wesfolk leader now or reject her and get gold; she can rejoin later. This is not a replacement-battle fork.                                                                               |
| Evacuation/carryover outcome         | [Son Of The Black Eye](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Son_Of_The_Black_Eye/scenarios/10_Saving_Inarix.cfg#L11): Bridge timing and saurian survival determine later recruitment. There is a path saving the company and the saurians; the loss is not a forced sacrifice.                                       |
| Party/mentor choice                  | [The South Guard](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_South_Guard/scenarios/02x_Westin.cfg#L9): Choose which officer accompanies Deoran. Party identity and dialogue vary without choosing a different chapter route.                                                                                           |
| Reward choice                        | [Two Brothers](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/Two_Brothers/scenarios/02_The_Chase.cfg#L671): Brena or gold on easy difficulty; not a replacement story path.                                                                                                                                                   |
| Veteran evacuation/carryover outcome | [The Deceivers Gambit](https://github.com/wesnoth/wesnoth/blob/39f64458e811dbf98f1fdaf03ea2b30a4269cfda/data/campaigns/The_Deceivers_Gambit/scenarios/13_Revelry_Revisited.cfg#L9): Escaping veterans remain available in the next battle; defeated commanders add starting gold. Not a separate A/B story route.                                                               |

The later Crosswind encounter with Shussek is not evidence that the earlier naga pact persists: its recorded spawn has no such condition. Avoid inventing a carried consequence from a returning character.

## Coverage

All nineteen story families’ recorded branch entries were reviewed. Zero below means no qualifying deliberate fork identified in those records, not proof that every optional event is absent.

| Campaign                | Route points | Local points |
| ----------------------- | -----------: | -----------: |
| Dead Water              |            0 |            0 |
| Descent Into Darkness   |            0 |            0 |
| Dusk of Dawn            |            0 |            0 |
| Eastern Invasion        |            4 |            1 |
| Heir To The Throne      |            0 |            5 |
| Legend of Wesmere       |            0 |            0 |
| Liberty                 |            0 |            0 |
| Northern Rebirth        |            0 |            0 |
| Of Pearls and Pirates   |            0 |            0 |
| Sceptre of Fire         |            0 |            0 |
| Secrets of the Ancients |            0 |            0 |
| Son Of The Black Eye    |            1 |            0 |
| The Deceivers Gambit    |            0 |            1 |
| The Hammer of Thursagan |            0 |            0 |
| The Rise Of Wesnoth     |            1 |            0 |
| The South Guard         |            2 |            0 |
| Two Brothers            |            0 |            0 |
| Under the Burning Suns  |            1 |            0 |
| Winds of Fate           |            0 |            1 |

These source structures need not fit the skill’s chapter-card format: some choose between whole maps, and alternative finales can share the same broad campaign achievement. Keep this empirical reference separate from requirements for a new design.
