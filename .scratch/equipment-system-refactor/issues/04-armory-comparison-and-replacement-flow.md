# 04: Armory comparison and replacement flow

**What to build:** Players can compare equipment sets and replacements in menu and pause flows with clear deltas, effective gain calculations, and replacement recommendations that match runtime rules. The Armory explains value between current gear and a candidate item, including skill enhancement targets, proc conditions, and total build impact.

**Blocked by:** 02: Loot generation and item-instance math; 03: Proc, skill enhancements, and control effects

**Status:** ready-for-agent

- [ ] The Armory shows raw values, capped outcomes, enhancement targets, and proc trigger details for the current and candidate gear.
- [ ] Comparison scoring reflects capped effective gain, current build compatibility, and expected proc value using the same underlying item state as combat evaluation.
- [ ] Dual-slot replacement decisions choose the lower-loss option when both slots are filled and the UI is responsive without unnecessary confirmation friction.
- [ ] The comparison flow works in both menu and pause states and is consistent with the player-facing inventory flow.
