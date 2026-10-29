//! Card growth is the sum of acquired qualities, never an exponential rank multiplier.
//! Unbounded potency accompanies bounded utility, so late cards still improve a build.

pub(super) fn damage(growth: f32) -> f32 {
    1.0 + 0.3 * (growth - 1.0)
}

pub(super) fn bounded(raw: f32, limit: f32) -> f32 {
    let raw = raw.max(0.0);
    limit * raw / (limit + raw)
}

pub(super) fn cooldown_multiplier(growth: f32) -> f32 {
    0.2 + 0.8 / (1.0 + 0.1 * (growth - 1.0).max(0.0))
}

pub(super) fn radius_multiplier(growth: f32) -> f32 {
    1.0 + bounded((growth - 1.0).max(0.0) * 0.08, 0.75)
}

pub(super) fn duration_multiplier(growth: f32) -> f32 {
    1.0 + 0.1 * (growth - 1.0).max(0.0)
}

pub(super) fn control_multiplier(growth: f32) -> f32 {
    1.0 + bounded((growth - 1.0).max(0.0) * 0.08, 0.6)
}

pub(super) fn bonus_targets(growth: f32) -> u32 {
    bounded((growth - 1.0).max(0.0) * 0.5, 4.0).floor() as u32
}

pub(super) fn effective_luck(raw: f32) -> f32 {
    bounded(raw, 200.0)
}

/// Shared fortune tilt, normalized by the caller when choosing a tier.
pub(super) fn quality_weights(luck: f32) -> [f32; 5] {
    let base = [50.0, 30.0, 13.0, 5.0, 2.0];
    std::array::from_fn(|i| base[i] * (1.0 + luck.max(0.0) / 100.0 * i as f32 / 5.0))
}

/// Rarity-specific previews share the same order as the live cast pipeline.
pub(super) fn cast_for_run(
    def: &'static super::data::AbilityDef,
    level: u32,
    growth: f32,
    run: &super::Run,
) -> super::talents::Cast {
    let mut stats = run.stats();
    let mut cast = super::talents::cast_with_growth(def, level, growth, &stats);
    if def.kind == super::data::Kind::Immune {
        cast.dmg = growth;
    }
    super::class_mechanics::modify_cast(run, def.key, &mut cast, &mut stats);
    super::spell_expansion::modify_cast(run, def.key, &mut cast, &mut stats);
    cast
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn additive_growth_has_no_old_rank_ceiling() {
        assert!(damage(300.0) > damage(255.0));
        assert!(damage(9.0) > damage(8.0));
        assert!(duration_multiplier(300.0) > duration_multiplier(255.0));
        assert!(cooldown_multiplier(300.0) < cooldown_multiplier(255.0));
        assert!(control_multiplier(300.0) > control_multiplier(255.0));
        assert!(bonus_targets(10000.0) <= 4);
    }

    #[test]
    fn luck_preserves_base_weights_and_only_tilts_upward() {
        assert_eq!(quality_weights(0.0), [50.0, 30.0, 13.0, 5.0, 2.0]);
        let low = quality_weights(0.0);
        let high = quality_weights(effective_luck(100.0));
        assert!(high[4] / high.iter().sum::<f32>() > low[4] / 100.0);
        assert!(effective_luck(10000.0) < 200.0);
    }
}
