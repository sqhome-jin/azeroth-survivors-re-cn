# 06: End-to-end simulation and tuning validation

**What to build:** Deterministic simulations and live-tuned tests verify caps, skill expectations, proc behavior, replacement recommendations, and save migration against the full 16-slot environment. This gives the balance team a reliable evidence base for tuning while keeping the data-configurable model as the single source of truth.

**Blocked by:** 02: Loot generation and item-instance math; 03: Proc, skill enhancements, and control effects; 04: Armory comparison and replacement flow; 05: Save migration and compatibility guardrails

**Status:** ready-for-agent

- [ ] Seeded item-generation tests cover rarity property counts, duplicate prevention, and balanced stat distribution.
- [ ] Proc and control-effect scenarios validate trigger behavior, run cooldown rules, boss exceptions, and effect merging without coupling tests to internals.
- [ ] Armory and replacement scoring tests compare full equipment sets and confirm the chosen replacement matches runtime behavior.
- [ ] Golden-save migration tests confirm backup creation, successful conversion, and safe abort behavior for invalid mappings.
- [ ] Simulation coverage confirms the full 16-slot build is the tuning environment and that balance changes remain data-driven.
