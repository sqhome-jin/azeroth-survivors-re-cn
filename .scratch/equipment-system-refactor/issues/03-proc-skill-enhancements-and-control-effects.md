# 03: Proc, skill enhancements, and control effects

**What to build:** Gear can carry typed proc effects, skill enhancements bound to compatible skills, and control effects that share the same reduction state as the rest of combat. Trigger conditions, cooldown sharing, and boss-specific exceptions remain explicit and data-driven instead of hidden in ad hoc rules.

**Blocked by:** 01: Equipment identity and slot schema; 02: Loot generation and item-instance math

**Status:** ready-for-agent

- [ ] Each proc effect is explicit, typed, and generated from configuration with valid trigger conditions and target behavior.
- [ ] Skill enhancements bind to a compatible skill and remain dormant when the skill is absent rather than breaking the item or forcing special casing.
- [ ] Control effects like slow, root, stun, and fear share the same reduction state and deterministic cooldown windows with boss exceptions handled by data rules.
- [ ] Duplicate proc names share a single run-level cooldown and proc chains are excluded to prevent runaway loops.
