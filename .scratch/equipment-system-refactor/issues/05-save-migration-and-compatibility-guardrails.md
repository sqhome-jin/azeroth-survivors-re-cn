# 05: Save migration and compatibility guardrails

**What to build:** The migration system upgrades legacy v3 save data safely by creating backups, validating mappings, and converting old equipment without silently guessing wrong values. Invalid or incompatible legacy attributes, effects, or skill bonuses are preserved when possible, removed with diagnostics when not safe, and never dropped without explicit handling.

**Blocked by:** 01: Equipment identity and slot schema; 02: Loot generation and item-instance math; 03: Proc, skill enhancements, and control effects

**Status:** ready-for-agent

- [ ] Legacy saves are backed up before any rewrite and migration aborts safely without discarding data if a mapping is unsafe.
- [ ] Old equipment, rarity, shared inventory state, and nine legacy slots map to the new schema with explicit deterministic rules and no silent guesswork.
- [ ] Invalid legacy attributes or skill/proc effects are removed with diagnostic records rather than guessed replacements.
- [ ] Duplicate effect pairs keep the strongest valid entry while preserving useful pre-upgrade gear information.
