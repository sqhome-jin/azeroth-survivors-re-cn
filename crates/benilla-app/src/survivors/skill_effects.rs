//! Skill modifiers, damage provenance and equipment-owned, run-local proc cooldowns.

use std::collections::BTreeSet;

use bevy::prelude::*;

use super::gear::{Bonuses, EnhKind, SkillRoll};
use super::{data, Run, Stats};

#[derive(Clone, Copy, Default)]
pub(super) struct Modifiers {
    pub damage: f32,
    pub cooldown: f32,
    pub count: u32,
    pub radius: f32,
    pub duration: f32,
    pub frequency: f32,
    pub range: f32,
    pub chain: f32,
    pub echo: f32,
    pub echo_damage: f32,
    pub splash: f32,
    pub ember: f32,
    pub execute: f32,
    pub leech: f32,
    pub control: f32,
}

impl Modifiers {
    pub fn from_stats(stats: &Stats, def: &data::AbilityDef) -> Self {
        let mut out = Self::default();
        for (kind, value) in stats.gear.enhancements(def.key) {
            match kind {
                EnhKind::Damage => out.damage += value,
                EnhKind::Cooldown => out.cooldown += value,
                EnhKind::Count => out.count += value.round() as u32,
                EnhKind::Radius => out.radius += value,
                EnhKind::Duration => out.duration += value,
                EnhKind::Frequency => out.frequency += value,
                EnhKind::Range => out.range += value,
                EnhKind::ChainPower => out.chain += value,
                EnhKind::Echo => out.echo += value,
                EnhKind::Splash => out.splash += value,
                EnhKind::Ember => out.ember += value,
                EnhKind::Execute => out.execute += value,
                EnhKind::Leech => out.leech += value,
                EnhKind::Control => out.control += value,
                _ => {}
            }
        }
        if EnhKind::Count.supports(def) {
            out.count = out.count.saturating_add(stats.targets);
        }
        if EnhKind::Echo.supports(def) {
            out.echo += stats.echo;
            out.echo_damage = stats.echo_damage;
        }
        if EnhKind::Range.supports(def) {
            out.range += stats.range;
        }
        if EnhKind::Duration.supports(def) {
            out.duration += stats.duration;
        }
        out.cooldown = out.cooldown.min(0.6);
        out.count = out
            .count
            .min(if def.kind == data::Kind::Chain { 8 } else { 6 });
        out.radius = out.radius.min(1.0);
        out.duration = out.duration.min(1.0);
        out.frequency = out.frequency.min(2.0);
        out.range = out.range.min(1.0);
        out.chain = out.chain.min(0.15);
        out.echo = out.echo.min(0.5);
        out.splash = out.splash.min(1.0);
        out.ember = out.ember.min(1.0);
        out.execute = out.execute.min(2.0);
        out.leech = out.leech.min(0.2);
        out.control = out.control.min(0.5);
        out
    }

    pub fn interval(self, base: f32) -> f32 {
        base / (1.0 + self.frequency)
    }

    pub fn effect(self, effect: data::Effect) -> data::Effect {
        use data::Effect;
        let mult = 1.0 + self.control;
        match effect {
            Effect::Slow(m, t) => Effect::Slow(m, t * mult),
            Effect::Root(t) => Effect::Root(t * mult),
            Effect::Stun(t) => Effect::Stun(t * mult),
            Effect::Fear(t) => Effect::Fear(t * mult),
            Effect::Weaken(m, t) => Effect::Weaken(m, t * mult),
            other => other,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub(super) struct Source {
    pub ability: Option<&'static str>,
    pub batch: u64,
    pub pulse: u32,
    pub modifiers: Modifiers,
    pub secondary: bool,
    pub echo: bool,
    pub periodic: bool,
    pub talent_frozen: bool,
}

/// Returns every scheduled tick in the active part of this frame, including the final tick.
pub(super) fn ticks(left: &mut f64, next: &mut f64, every: f64, dt: f32) -> u32 {
    let dt = f64::from(dt);
    let active = dt.min((*left).max(0.0));
    *left = (*left - dt).max(0.0);
    if active <= 0.0 {
        return 0;
    }
    *next -= active;
    let mut count = 0;
    while *next <= 1e-9 {
        count += 1;
        *next += every;
    }
    count
}

struct Proc {
    item: u64,
    index: u8,
    kind: EnhKind,
    value: f32,
    ready_at: f32,
    until: f32,
    attempted: BTreeSet<(u64, u32)>,
}

#[derive(Default)]
pub(super) struct Runtime {
    procs: Vec<Proc>,
}

impl Runtime {
    pub fn active_buffs(&self, now: f32) -> Vec<(u64, EnhKind, f32, f32)> {
        self.procs
            .iter()
            .filter(|proc| proc.until > now)
            .map(|proc| (proc.item, proc.kind, proc.value, proc.until - now))
            .collect()
    }

    pub fn reconcile(&mut self, gear: &Bonuses, now: f32) {
        for proc in &mut self.procs {
            if !gear
                .enh
                .iter()
                .flatten()
                .any(|r| r.item == proc.item && r.index == proc.index)
            {
                proc.until = now;
            }
        }
    }

    pub fn bonus(&self, kind: EnhKind, now: f32) -> f32 {
        self.procs
            .iter()
            .filter(|p| p.kind == kind && p.until > now)
            .map(|p| p.value)
            .sum::<f32>()
            .min(match kind {
                EnhKind::BuffDamage => 1.0,
                EnhKind::BuffHaste => 0.4,
                EnhKind::BuffCrit => 0.3,
                EnhKind::BuffSpeed => 0.5,
                EnhKind::BuffDefense => 0.4,
                _ => 0.0,
            })
    }

    fn attempt(&mut self, roll: SkillRoll, source: Source, now: f32, random: f32) -> bool {
        let at = self
            .procs
            .iter()
            .position(|p| p.item == roll.item && p.index == roll.index);
        let at = at.unwrap_or_else(|| {
            self.procs.push(Proc {
                item: roll.item,
                index: roll.index,
                kind: roll.kind,
                value: roll.value,
                ready_at: 0.0,
                until: 0.0,
                attempted: BTreeSet::new(),
            });
            self.procs.len() - 1
        });
        let p = &mut self.procs[at];
        if now < p.ready_at || !p.attempted.insert((source.batch, source.pulse)) {
            return false;
        }
        let chance = if roll.ability.is_some() { 0.3 } else { 0.15 };
        if random >= chance {
            return false;
        }
        p.until = now + 30.0;
        p.ready_at = now + 30.0;
        true
    }
}

pub(super) fn damage_effects(run: &mut Run, source: Source, effective: f32, rng: &mut super::Rng) {
    if effective <= 0.0 || run.dead_t.is_some() || run.hp <= 0.0 {
        return;
    }
    let Some(key) = source.ability else { return };
    let def = data::ability(key);
    let leech = source.modifiers.leech;
    let base_leech = if source.periodic {
        run.leech()
    } else {
        run.gear.leech() + run.stats().leech
    };
    let heal = (effective * (base_leech + leech)).min(run.talent_leech_budget);
    run.talent_leech_budget -= heal;
    run.hp = (run.hp + heal).min(run.stats().max_hp);
    if source.secondary {
        return;
    }
    for roll in run
        .gear
        .enh
        .into_iter()
        .flatten()
        .filter(|r| r.kind.is_buff() && r.matches(def))
    {
        run.effects.attempt(roll, source, run.elapsed, rng.f32());
    }
}

pub(super) fn plugin(app: &mut App) {
    super::buffs::plugin(app);
}

#[cfg(test)]
mod tests {
    use super::super::{gear, Rng};
    use super::*;

    fn roll(kind: EnhKind, ability: Option<&'static str>, value: f32, item: u64) -> SkillRoll {
        SkillRoll {
            item,
            index: 0,
            ability,
            kind,
            value,
        }
    }

    fn source(key: &'static str, batch: u64) -> Source {
        Source {
            ability: Some(key),
            batch,
            ..default()
        }
    }

    fn close(a: f32, b: f32) {
        assert!((a - b).abs() < 0.0001, "{a} != {b}");
    }

    #[test]
    fn twice_the_frequency_preserves_tick_damage_and_doubles_total_even_with_slow_frames() {
        fn simulate(interval: f32, dt: f32) -> (u32, f32) {
            let interval = f64::from(interval);
            let (mut left, mut next, mut count) = (6.0, interval, 0);
            while left > 0.0 {
                count += ticks(&mut left, &mut next, interval, dt);
            }
            (count, count as f32 * 100.0)
        }
        for dt in [1.0 / 60.0, 0.5, 1.7, 10.0] {
            assert_eq!(simulate(1.0, dt), (6, 600.0));
            let m = Modifiers {
                frequency: 1.0,
                ..default()
            };
            assert_eq!(simulate(m.interval(1.0), dt), (12, 1200.0));
        }
        let (mut left, mut next) = (6.0, 0.5);
        assert_eq!(ticks(&mut left, &mut next, 0.5, 0.0), 0);
    }

    #[test]
    fn passive_modifiers_apply_once_independently_of_equipment_count() {
        let mut stats = Run::default().stats();
        stats.targets = 2;
        stats.echo = 0.2;
        stats.echo_damage = 0.15;
        stats.range = 0.3;
        stats.duration = 0.4;
        let verify = |stats: &Stats| {
            let frost = Modifiers::from_stats(stats, data::ability("frostbolt"));
            assert_eq!(frost.count, 2);
            close(frost.echo, 0.2);
            close(frost.echo_damage, 0.15);
            close(frost.range, 0.3);
            let corruption = Modifiers::from_stats(stats, data::ability("corruption"));
            close(corruption.duration, 0.4);
            let shield = Modifiers::from_stats(stats, data::ability("divine_shield"));
            assert_eq!(shield.count, 0);
            close(shield.echo, 0.0);
        };
        verify(&stats);
        for (i, kind) in [EnhKind::Damage, EnhKind::Cooldown, EnhKind::Leech]
            .into_iter()
            .enumerate()
        {
            stats.gear.enh[i] = Some(roll(kind, None, 0.01, i as u64 + 1));
            verify(&stats);
        }
    }

    #[test]
    fn global_and_specific_modifiers_combine_only_on_compatible_skills() {
        let mut run = Run::default();
        run.gear.enh[0] = Some(roll(EnhKind::Frequency, None, 0.3, 1));
        run.gear.enh[1] = Some(roll(EnhKind::Frequency, Some("corruption"), 1.0, 2));
        run.gear.enh[2] = Some(roll(EnhKind::Count, None, 1.0, 3));
        run.gear.enh[3] = Some(roll(EnhKind::Radius, None, 0.15, 4));
        let get = |key| Modifiers::from_stats(&run.stats(), data::ability(key));
        close(get("corruption").frequency, 1.3);
        close(get("blizzard").frequency, 0.3);
        close(get("frostbolt").frequency, 0.0);
        assert_eq!(get("frostbolt").count, 1);
        assert_eq!(get("blizzard").count, 0);
        close(get("frostbolt").radius, 0.0);
        close(get("blizzard").radius, 0.15);
        close(get("divine_shield").radius, 0.0);
    }

    #[test]
    fn new_modifier_parameters_change_and_respect_caps() {
        let mut run = Run::default();
        let kinds = [
            EnhKind::Frequency,
            EnhKind::Range,
            EnhKind::ChainPower,
            EnhKind::Echo,
            EnhKind::Splash,
            EnhKind::Ember,
            EnhKind::Execute,
            EnhKind::Leech,
            EnhKind::Control,
        ];
        let keys = [
            "corruption",
            "frostbolt",
            "chain_lightning",
            "frostbolt",
            "frostbolt",
            "frostbolt",
            "frostbolt",
            "frostbolt",
            "frostbolt",
        ];
        for (i, (&kind, key)) in kinds.iter().zip(keys).enumerate() {
            assert!(kind.supports(data::ability(key)));
            run.gear.enh[i] = Some(roll(kind, None, 0.1, i as u64 + 1));
        }
        let stats = run.stats();
        let frost = Modifiers::from_stats(&stats, data::ability("frostbolt"));
        close(frost.range, 0.1);
        close(frost.echo, 0.1);
        close(frost.splash, 0.1);
        close(frost.ember, 0.1);
        close(frost.execute, 0.1);
        close(frost.leech, 0.1);
        assert!(
            matches!(frost.effect(data::Effect::Root(2.0)), data::Effect::Root(t) if (t - 2.2).abs() < 1e-4)
        );
        close(
            Modifiers::from_stats(&stats, data::ability("chain_lightning")).chain,
            0.1,
        );
        for slot in &mut run.gear.enh {
            *slot = Some(roll(EnhKind::Frequency, None, 1.0, 1));
        }
        close(
            Modifiers::from_stats(&run.stats(), data::ability("corruption")).frequency,
            2.0,
        );
    }

    #[test]
    fn procs_have_exact_chances_duration_and_cooldown_without_stacking_or_refresh() {
        let mut runtime = Runtime::default();
        let targeted = roll(EnhKind::BuffDamage, Some("corruption"), 0.3, 1);
        let s = source("corruption", 1);
        assert!(!runtime.attempt(targeted, s, 0.0, 0.30));
        assert!(
            !runtime.attempt(targeted, s, 0.0, 0.0),
            "one roll per area/tick batch"
        );
        assert!(runtime.attempt(targeted, source("corruption", 2), 0.0, 0.299));
        close(runtime.bonus(EnhKind::BuffDamage, 0.0), 0.3);
        assert!(!runtime.attempt(targeted, source("corruption", 3), 29.99, 0.0));
        close(runtime.procs[0].until, 30.0);
        close(runtime.bonus(EnhKind::BuffDamage, 30.0), 0.0);
        assert!(runtime.attempt(targeted, source("corruption", 4), 30.0, 0.0));
        close(runtime.bonus(EnhKind::BuffDamage, 30.0), 0.3);
        assert_eq!(runtime.procs.len(), 1);
        let global = roll(EnhKind::BuffDamage, None, 0.15, 2);
        assert!(!runtime.attempt(global, source("corruption", 5), 30.0, 0.15));
        assert!(runtime.attempt(global, source("corruption", 6), 30.0, 0.149));
        close(runtime.bonus(EnhKind::BuffDamage, 30.0), 0.45);
    }

    #[test]
    fn unequipping_removes_buff_but_reequipping_does_not_reset_cooldown() {
        let mut runtime = Runtime::default();
        let r = roll(EnhKind::BuffDamage, None, 0.3, 99);
        assert!(runtime.attempt(r, source("frostbolt", 1), 0.0, 0.0));
        runtime.reconcile(&Bonuses::default(), 5.0);
        close(runtime.bonus(EnhKind::BuffDamage, 5.0), 0.0);
        let mut gear = Bonuses::default();
        gear.enh[0] = Some(r);
        runtime.reconcile(&gear, 6.0);
        assert!(!runtime.attempt(r, source("frostbolt", 2), 6.0, 0.0));
        assert!(runtime.attempt(r, source("frostbolt", 3), 30.0, 0.0));
    }

    #[test]
    fn buffs_feed_real_stats_and_damage_reduction_and_expire_on_run_clock() {
        let mut run = Run::default();
        let base = run.stats();
        for (i, kind) in gear::ENH_KINDS
            .into_iter()
            .filter(|k| k.is_buff())
            .enumerate()
        {
            let r = roll(kind, None, 0.1, i as u64);
            assert!(run.effects.attempt(r, source("frostbolt", 1), 0.0, 0.0));
        }
        close(run.stats().might, base.might * 1.1);
        close(run.stats().haste, base.haste + 0.1);
        close(run.stats().crit, base.crit + 0.1);
        close(run.stats().speed, base.speed * 1.1);
        close(run.incoming(), 0.9);
        run.elapsed = 30.0;
        close(run.stats().might, base.might);
        close(run.incoming(), 1.0);
    }

    #[test]
    fn damage_healing_has_one_shared_budget_and_never_revives_or_exceeds_max_health() {
        let mut run = Run {
            hp: 50.0,
            ..default()
        };
        let mut rng = Rng(12345);
        let mut s = source("corruption", 1);
        s.modifiers.leech = 0.05;
        super::super::talents::periodic_heal(&mut run, 0.0, 1.0);
        let budget = run.talent_leech_budget;
        damage_effects(&mut run, s, 100.0, &mut rng);
        close(run.hp, 55.0);
        damage_effects(&mut run, s, 10_000.0, &mut rng);
        close(run.hp, 50.0 + budget);
        damage_effects(&mut run, s, 10_000.0, &mut rng);
        close(run.hp, 50.0 + budget);
        run.dead_t = Some(0.0);
        run.hp = 0.0;
        damage_effects(&mut run, s, 1000.0, &mut rng);
        close(run.hp, 0.0);
        run.dead_t = None;
        run.hp = run.stats().max_hp - 1.0;
        run.talent_leech_budget = 10.0;
        damage_effects(&mut run, s, 1000.0, &mut rng);
        close(run.hp, run.stats().max_hp);
    }

    #[test]
    fn secondary_and_zero_damage_never_trigger_equipment_buffs() {
        let mut run = Run::default();
        run.gear.enh[0] = Some(roll(EnhKind::BuffDamage, None, 0.15, 1));
        let mut rng = Rng(12345);
        damage_effects(&mut run, source("frostbolt", 1), 0.0, &mut rng);
        assert!(run.effects.procs.is_empty());
        for batch in 1..100 {
            let s = Source {
                secondary: true,
                ..source("frostbolt", batch)
            };
            damage_effects(&mut run, s, 100.0, &mut rng);
        }
        assert!(run.effects.procs.is_empty());
        let mut run = Run::default();
        run.gear.enh[0] = Some(roll(EnhKind::BuffDamage, Some("corruption"), 0.3, 1));
        damage_effects(&mut run, source("frostbolt", 200), 100.0, &mut rng);
        assert!(run.effects.procs.is_empty());
    }
}
