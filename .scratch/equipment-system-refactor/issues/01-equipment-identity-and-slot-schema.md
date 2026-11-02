# 01: Equipment identity and slot schema

**What to build:** The equipment system exposes a stable item identity separate from each rolled instance, supports the full 16-slot roster from day one, preserves clear slot semantics, and establishes the data contract for rarity, stat weighting, profession affinity, and effect slots. This creates the end-to-end foundation players can reason about and the rest of the system can build on.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The system distinguishes a base item identity from a concrete item instance and keeps instance-level randomness separate from fixed identity.
- [ ] All 16 equipment slots are present with consistent semantics and no additional unlock gating in the initial release.
- [ ] Rarity, profession affinity, preferred main attributes, and base item metadata are driven by data rather than hard-coded assumptions.
- [ ] The item schema enables deterministic stat/prop generation and comparison without collapsing identity into a single rolled object.
