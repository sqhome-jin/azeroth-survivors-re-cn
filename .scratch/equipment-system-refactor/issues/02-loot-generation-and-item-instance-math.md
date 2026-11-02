# 02: Loot generation and item-instance math

**What to build:** Items generate a configured number of randomized property entries based on rarity using a shared weighted attribute pool, with capped and uncapped value tracking kept visible to the player. The inventory flow preserves the current retention policy while making loot quality, stat math, and replacement logic coherent across all slots.

**Blocked by:** 01: Equipment identity and slot schema

**Status:** ready-for-agent

- [x] Item instances produce the correct number of primary and secondary stat rolls for each rarity, with no duplicate entries in a single item.
- [x] Main and secondary stat conversions flow through a shared data-driven table before combat resolution and remain capped per configured thresholds.
- [x] Inventory behavior preserves the current bag policy: the lowest-quality unequipped item is dropped when full, and refusal happens when no valid unequipped candidate exists.
- [x] The resulting item state is understandable in both raw and capped forms so the Armory can explain source values versus effective gains.
