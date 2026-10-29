//! Survivors warlock talent interactions. Timers belong to the run, never the saved loadout.

use super::{Rng, Run, enemies::Enemy, skill_effects::Source, talents::Loadout};

#[derive(Default)]
pub(super) struct Runtime {
    nightfall_ready: f32,
    haste_ready: f32,
    haste_until: f32,
}

#[derive(Default)]
pub(super) struct Debuffs {
    vulnerability_until: f32,
    vulnerability: f32,
}

pub(super) fn affliction(key: Option<&str>) -> bool {
    matches!(key, Some("corruption" | "curse_of_agony" | "immolate"))
}

fn direct(key: Option<&str>) -> bool {
    matches!(
        key,
        Some("shadow_bolt" | "immolate" | "death_coil" | "searing_pain" | "shadowburn")
    )
}

pub(super) fn periodic_crits(talents: Loadout, source: Source) -> bool {
    talents.0[7] > 0 && !source.secondary && affliction(source.ability)
}

pub(super) fn haste(run: &Run) -> f32 {
    if run.class.id == 9 && run.warlock.haste_until > run.elapsed {
        0.03 * f32::from(run.talents.0[22])
    } else {
        0.0
    }
}

pub(super) fn haste_remaining(run: &Run) -> f32 {
    (run.warlock.haste_until - run.elapsed).max(0.0)
}

pub(super) fn damage_multiplier(run: &Run, source: Source, enemy: &Enemy) -> f32 {
    if run.class.id != 9 || source.secondary || source.ability.is_none() {
        return 1.0;
    }
    let vulnerability = if enemy.warlock.vulnerability_until > run.elapsed {
        enemy.warlock.vulnerability
    } else {
        0.0
    };
    let immolate = if !source.periodic
        && direct(source.ability)
        && enemy
            .dots
            .iter()
            .any(|d| d.left > 0.0 && !d.source.secondary && d.source.ability == Some("immolate"))
    {
        0.04 * f32::from(run.talents.0[17])
    } else {
        0.0
    };
    (1.0 + vulnerability) * (1.0 + immolate)
}

pub(super) fn extend_dot(dot: &mut super::enemies::Dot, talents: Loadout, random: f32) {
    if !dot.source.secondary
        && affliction(dot.source.ability)
        && random < 0.04 * f32::from(talents.0[2])
    {
        let extra = dot.extension_left.min(1.0);
        dot.left += extra;
        dot.extension_left -= extra;
    }
}

fn nightfall(run: &mut Run, random: f32) {
    if run.elapsed < run.warlock.nightfall_ready || random >= 0.02 * f32::from(run.talents.0[5]) {
        return;
    }
    let target = run
        .abilities
        .iter_mut()
        .filter(|o| direct(Some(o.def.key)) && o.cd > 0.0)
        .max_by(|a, b| a.cd.total_cmp(&b.cd));
    if let Some(target) = target {
        target.cd = 0.0;
        run.warlock.nightfall_ready = run.elapsed + 3.0;
        run.banner = Some((
            super::localization::text("Nightfall: spell ready!", "夜幕：直伤法术冷却刷新！").into(),
            1.5,
        ));
    }
}

pub(super) fn on_damage(
    run: &mut Run,
    source: Source,
    critical: bool,
    enemy: &mut Enemy,
    marks: &mut super::units::Marks,
    rng: &mut Rng,
) {
    if run.class.id != 9 || run.dead_t.is_some() || source.secondary || source.ability.is_none() {
        return;
    }
    let t = run.talents;
    if source.periodic && affliction(source.ability) {
        if t.0[5] > 0 && run.elapsed >= run.warlock.nightfall_ready {
            nightfall(run, rng.f32());
        }
        if t.0[6] > 0 {
            let slow = 0.04 * f32::from(t.0[6]) * if enemy.boss { 0.5 } else { 1.0 };
            enemy.slow = if enemy.slow_t > 0.0 {
                enemy.slow.min(1.0 - slow)
            } else {
                1.0 - slow
            };
            enemy.slow_t = enemy.slow_t.max(3.0);
            marks.add(18223, 3.0);
        }
    }

    if t.0[14] > 0 {
        let weaken = 1.0 - 0.02 * f32::from(t.0[14]);
        enemy.weaken = if enemy.weaken_t > 0.0 {
            enemy.weaken.min(weaken)
        } else {
            weaken
        };
        enemy.weaken_t = enemy.weaken_t.max(6.0);
        marks.add(702, 6.0);
    }
    if critical && !source.periodic {
        if source.ability == Some("shadow_bolt") && t.0[16] > 0 {
            enemy.warlock.vulnerability = 0.03 * f32::from(t.0[16]);
            enemy.warlock.vulnerability_until = run.elapsed + 6.0;
            marks.add(17800, 6.0);
        }
        if matches!(source.ability, Some("immolate" | "searing_pain"))
            && t.0[22] > 0
            && run.elapsed >= run.warlock.haste_ready
        {
            run.warlock.haste_until = run.elapsed + 6.0;
            run.warlock.haste_ready = run.elapsed + 8.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Owned, data, enemies::Dot};
    use super::*;

    fn dot(duration: f64, every: f64) -> Dot {
        Dot {
            dps: 100.0,
            left: duration,
            tick: every,
            every,
            extension_left: duration * 0.5,
            source: Source {
                ability: Some("corruption"),
                ..Default::default()
            },
        }
    }

    #[test]
    fn extension_has_exact_rank_chance_and_a_lifetime_budget() {
        for rank in [0, 1, 5] {
            let mut talents = Loadout::default();
            talents.0[2] = rank;
            let chance = 0.04 * f32::from(rank);
            let mut d = dot(11.0, 1.0);
            extend_dot(&mut d, talents, chance);
            assert_eq!(d.left, 11.0);
            if rank > 0 {
                extend_dot(&mut d, talents, chance - 0.00001);
                assert_eq!(d.left, 12.0);
                for _ in 0..100 {
                    extend_dot(&mut d, talents, 0.0);
                }
                assert_eq!(d.left, 16.5);
                assert_eq!(d.extension_left, 0.0);
            }
        }
        let mut talents = Loadout::default();
        talents.0[2] = 5;
        for key in ["frostbolt", "fireball"] {
            let mut d = dot(6.0, 1.0);
            d.source.ability = Some(key);
            extend_dot(&mut d, talents, 0.0);
            assert_eq!(d.left, 6.0);
        }
        let mut d = dot(6.0, 1.0);
        d.source.secondary = true;
        extend_dot(&mut d, talents, 0.0);
        assert_eq!(d.left, 6.0);
    }

    #[test]
    fn extension_keeps_tick_damage_and_handles_extra_ticks_in_the_same_long_frame() {
        let mut talents = Loadout::default();
        talents.0[2] = 5;
        for every in [1.0, 0.5] {
            for dt in [1.0 / 60.0, 0.5, 1.7, 10.0] {
                let mut d = dot(6.0, every);
                let mut damage = 0.0;
                while d.left > 0.0 {
                    let mut frame = dt;
                    while d.next_tick(&mut frame) {
                        damage += d.dps;
                        extend_dot(&mut d, talents, 0.0);
                    }
                }
                assert_eq!(f64::from(damage), 900.0 / every);
                assert_eq!(d.extension_left, 0.0);
            }
        }
    }

    fn caster() -> Run {
        let mut run = Run {
            class: data::CLASSES.iter().find(|c| c.id == 9).unwrap(),
            ..Default::default()
        };
        run.talents.0[5] = 5;
        for (key, cd) in [
            ("shadow_bolt", 1.0),
            ("death_coil", 4.0),
            ("rain_of_fire", 9.0),
        ] {
            run.abilities.push(Owned {
                def: data::ability(key),
                level: 1,
                cd,
            });
        }
        run
    }

    #[test]
    fn nightfall_resets_only_longest_learned_direct_cooldown_and_has_shared_icd() {
        let mut run = caster();
        nightfall(&mut run, 0.1);
        assert_eq!(run.abilities[1].cd, 4.0);
        nightfall(&mut run, 0.09999);
        assert_eq!(run.abilities[1].cd, 0.0);
        assert_eq!(run.abilities[2].cd, 9.0);
        run.elapsed = 2.999;
        nightfall(&mut run, 0.0);
        assert_eq!(run.abilities[0].cd, 1.0);
        run.elapsed = 3.0;
        nightfall(&mut run, 0.0);
        assert_eq!(run.abilities[0].cd, 0.0);
        assert_eq!(run.warlock.nightfall_ready, 6.0);
        run.elapsed = 6.0;
        nightfall(&mut run, 0.0);
        assert_eq!(run.warlock.nightfall_ready, 6.0);
        run.abilities.push(Owned {
            def: data::ability("searing_pain"),
            level: 1,
            cd: 2.0,
        });
        nightfall(&mut run, 0.0);
        assert_eq!(run.abilities.last().unwrap().cd, 0.0);
        assert_eq!(Run::default().warlock.nightfall_ready, 0.0);
    }

    #[test]
    fn periodic_crit_gate_excludes_equipment_secondaries_and_other_skills() {
        let mut talents = Loadout::default();
        let mut source = Source {
            ability: Some("corruption"),
            ..Default::default()
        };
        assert!(!periodic_crits(talents, source));
        talents.0[7] = 1;
        assert!(periodic_crits(talents, source));
        source.secondary = true;
        assert!(!periodic_crits(talents, source));
        source.secondary = false;
        source.ability = Some("fireball");
        assert!(!periodic_crits(talents, source));
    }
}
