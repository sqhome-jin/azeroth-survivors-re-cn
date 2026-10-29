//! Run-local class interactions; previews never consume a charge or advance a proc.

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::Entity;

use super::{
    abilities::Hit,
    enemies::{Dot, Enemy},
    skill_effects::Source,
    talents, Rng, Run, Stats,
};

#[derive(Default)]
pub(super) struct Runtime {
    until: [f32; 24],
    ready: [f32; 24],
    attempted: BTreeSet<(u8, u64, u32)>,
    arcane: u8,
    fire: u8,
    fire_streak: u8,
    frozen: u8,
    ambush: u8,
    consecration: bool,
    totem: bool,
    rejuvenation: bool,
    rejuvenation_heal: f32,
    rejuvenation_extension: f32,
    nova_heals: BTreeMap<u64, f32>,
}

#[derive(Default)]
pub(super) struct Debuffs {
    mark_until: f32,
    shadow_until: f32,
    shadow_stacks: u8,
}

pub(super) fn target_buffs(run: &Run, enemy: &Enemy) -> Vec<(&'static str, u32, f32, f32)> {
    use super::localization::text;
    let mut effects = Vec::new();
    let mark = enemy.class_effects.mark_until - run.elapsed;
    if mark > 0.0 {
        let (name, spell, amount) = if run.class.id == 2 {
            (
                text("Holy vulnerability", "神圣易伤"),
                20271,
                0.03 * rank(run, 21),
            )
        } else {
            (
                text("Hunter's mark", "猎人印记"),
                1130,
                0.03 * rank(run, 13),
            )
        };
        effects.push((name, spell, amount, mark));
    }
    let shadow = enemy.class_effects.shadow_until - run.elapsed;
    if shadow > 0.0 {
        effects.push((
            text("Shadow weaving", "暗影交织"),
            15258,
            0.01 * rank(run, 21) * f32::from(enemy.class_effects.shadow_stacks),
            shadow,
        ));
    }
    effects
}

fn rank(run: &Run, slot: usize) -> f32 {
    f32::from(run.talents.0[slot])
}

fn active(run: &Run, slot: usize) -> bool {
    run.class_effects.until[slot] > run.elapsed
}

fn melee(key: &str) -> bool {
    matches!(
        key,
        "cleave"
            | "heroic_strike"
            | "whirlwind"
            | "sweeping_strikes"
            | "execute"
            | "mortal_strike"
            | "bloodthirst"
            | "overpower"
            | "slam"
            | "shield_slam"
            | "shield_bash"
    )
}

fn shot(key: &str) -> bool {
    matches!(
        key,
        "arcane_shot" | "multi_shot" | "aimed_shot" | "concussive_shot"
    )
}

fn holy(class: u8, key: &str) -> bool {
    match class {
        2 => matches!(
            key,
            "holy_shock"
                | "hammer_of_wrath"
                | "exorcism"
                | "holy_wrath"
                | "hammer_of_justice"
                | "judgement"
        ),
        5 => matches!(
            key,
            "smite" | "holy_fire" | "holy_nova" | "flash_heal" | "prayer_of_healing"
        ),
        _ => false,
    }
}

fn shadow(key: &str) -> bool {
    matches!(
        key,
        "mind_blast" | "shadow_word_pain" | "mind_flay" | "devouring_plague" | "shadowguard"
    )
}

fn shock(key: &str) -> bool {
    matches!(key, "earth_shock" | "flame_shock" | "frost_shock")
}

fn lightning(key: &str) -> bool {
    matches!(key, "lightning_bolt" | "chain_lightning")
}

fn arcane(key: &str) -> bool {
    matches!(key, "arcane_missiles" | "arcane_explosion")
}

fn fire(key: &str) -> bool {
    matches!(key, "fireball" | "fire_blast" | "blast_wave" | "pyroblast")
}

fn frost(key: &str) -> bool {
    matches!(
        key,
        "frostbolt" | "frost_nova" | "blizzard" | "cone_of_cold"
    )
}

fn frost_direct(key: &str) -> bool {
    matches!(key, "frostbolt" | "frost_nova" | "cone_of_cold")
}

fn rogue_direct(key: &str) -> bool {
    matches!(
        key,
        "sinister_strike"
            | "eviscerate"
            | "blade_flurry"
            | "gouge"
            | "kidney_shot"
            | "backstab"
            | "ambush"
    )
}

fn druid_direct(key: &str) -> bool {
    matches!(
        key,
        "wrath" | "starfire" | "moonfire" | "insect_swarm" | "entangling_roots"
    )
}

fn dot(enemy: &Enemy, key: &str) -> bool {
    enemy
        .dots
        .iter()
        .any(|d| d.left > 0.0 && !d.source.secondary && d.source.ability == Some(key))
}

fn bleeding(enemy: &Enemy) -> bool {
    dot(enemy, "rupture")
        || dot(enemy, "garrote")
        || dot(enemy, "rend")
        || dot(enemy, "rake")
        || dot(enemy, "rip")
        || enemy.dots.iter().any(|d| {
            d.left > 0.0 && d.source.secondary && d.source.ability == Some("warrior_bleed")
        })
}

fn slowed(enemy: &Enemy) -> bool {
    enemy.root_t > 0.0 || (enemy.slow_t > 0.0 && enemy.slow < 1.0)
}

fn once(run: &mut Run, slot: u8, source: Source, whole_cast: bool) -> bool {
    run.class_effects.attempted.insert((
        slot,
        source.batch,
        if whole_cast { 0 } else { source.pulse },
    ))
}

fn reduce_cooldown(run: &mut Run, key: &str, seconds: f32) -> bool {
    if let Some(owned) = run.abilities.iter_mut().find(|o| o.def.key == key) {
        owned.cd = (owned.cd - seconds).max(0.0);
        true
    } else {
        false
    }
}

fn refresh(run: &mut Run, key: &str) -> bool {
    reduce_cooldown(run, key, f32::MAX)
}

fn capped_heal(run: &mut Run, amount: f32) {
    if run.dead_t.is_some() || run.hp <= 0.0 || !amount.is_finite() || amount <= 0.0 {
        return;
    }
    let max = run.stats().max_hp;
    let heal = amount
        .min(run.talent_leech_budget.max(0.0))
        .min((max - run.hp).max(0.0));
    run.talent_leech_budget -= heal;
    run.hp = (run.hp + heal).min(max);
}

fn shield(run: &mut Run, grant: f32, duration: f32) {
    if grant <= 0.0 || run.dead_t.is_some() || run.hp <= 0.0 {
        return;
    }
    let existing = if run.shield_t > 0.0 { run.shield } else { 0.0 };
    run.shield = existing.max(grant);
    run.shield_t = run.shield_t.max(duration);
}

pub(super) fn modify_stats(run: &Run, stats: &mut Stats) {
    match run.class.id {
        1 => {
            if active(run, 13) {
                stats.haste += 0.03 * rank(run, 13);
            }
            if active(run, 15) && rank(run, 15) > 0.0 {
                stats.crit += 0.20;
            }
        }
        2 if active(run, 22) => stats.might *= 1.0 + 0.03 * rank(run, 22),
        3 => {
            if active(run, 5) {
                stats.haste += 0.03 * rank(run, 5);
            }
            if active(run, 7) && rank(run, 7) > 0.0 {
                stats.might *= 1.20;
            }
        }
        4 => {
            if active(run, 14) {
                stats.haste += 0.03 * rank(run, 14);
            }
            if active(run, 15) && rank(run, 15) > 0.0 {
                stats.haste += 0.25;
            }
        }
        5 => {
            if run.shield_t > 0.0 && run.shield > 0.0 {
                stats.haste += 0.02 * rank(run, 5);
            }
            if active(run, 7) && rank(run, 7) > 0.0 {
                stats.haste += 0.20;
            }
        }
        7 if active(run, 14) => stats.haste += 0.03 * rank(run, 14),
        8 if active(run, 7) && rank(run, 7) > 0.0 => {
            stats.might *= 1.20;
            stats.haste += 0.15;
        }
        _ => {}
    }
    stats.crit = stats.crit.clamp(0.0, 1.0);
}

/// Applies the run's current snapshot to this cast, without changing run state.
pub(super) fn modify_cast(run: &Run, key: &str, cast: &mut talents::Cast, stats: &mut Stats) {
    let mut mult = 1.0;
    match run.class.id {
        2 => {
            if key == "hammer_of_wrath" && active(run, 6) {
                mult *= 1.0 + 0.05 * rank(run, 6);
            }
            if key == "holy_shock" && active(run, 7) && rank(run, 7) > 0.0 {
                stats.crit = 1.0;
            }
        }
        4 if key == "sinister_strike" && active(run, 23) && run.class_effects.ambush > 0 => {
            mult *= 1.40;
        }
        5 if holy(5, key) && active(run, 15) && rank(run, 15) > 0.0 => mult *= 1.15,
        5 if shadow(key) && rank(run, 23) > 0.0 => mult *= 1.20,
        7 => {
            if lightning(key) && active(run, 5) {
                mult *= 1.0 + 0.05 * rank(run, 5);
            }
            if shock(key) && active(run, 13) {
                mult *= 1.0 + 0.05 * rank(run, 13);
            }
            if active(run, 15) && rank(run, 15) > 0.0 {
                if shock(key) {
                    mult *= 1.20;
                }
                if key == "lightning_shield" {
                    cast.cooldown = (cast.cooldown / 1.50).max(0.2);
                }
            }
        }
        8 => {
            if arcane(key) && active(run, 5) {
                mult *= 1.0 + 0.01 * rank(run, 5) * f32::from(run.class_effects.arcane);
            }
            if fire(key) && active(run, 15) && rank(run, 15) > 0.0 {
                stats.crit = (stats.crit + 0.05 * f32::from(run.class_effects.fire)).min(1.0);
            }
        }
        11 if (key == "starfire" && active(run, 5)) || (key == "wrath" && active(run, 6)) => {
            mult *= 1.0 + 0.05 * rank(run, 5);
        }
        _ => {}
    }
    cast.dmg *= mult;
    cast.dot *= mult;
}

/// Frozen Fingers is snapshotted onto the projectile before its charge is consumed.
pub(super) fn frozen_cast(run: &Run, key: &str) -> bool {
    run.class.id == 8
        && rank(run, 23) > 0.0
        && frost_direct(key)
        && active(run, 23)
        && run.class_effects.frozen > 0
}

pub(super) fn on_cast(run: &mut Run, key: &str, _batch: u64) {
    match run.class.id {
        2 => match key {
            "hammer_of_wrath" => run.class_effects.until[6] = 0.0,
            "holy_shock" => run.class_effects.until[7] = 0.0,
            _ => {}
        },
        4 if key == "sinister_strike" && active(run, 23) => {
            run.class_effects.ambush = run.class_effects.ambush.saturating_sub(1);
            if run.class_effects.ambush == 0 {
                run.class_effects.until[23] = 0.0;
            }
        }
        7 => {
            if lightning(key) {
                run.class_effects.until[5] = 0.0;
            }
            if shock(key) {
                run.class_effects.until[13] = 0.0;
            }
        }
        8 => {
            if arcane(key) && rank(run, 5) > 0.0 {
                let stacks = if active(run, 5) {
                    run.class_effects.arcane
                } else {
                    0
                };
                run.class_effects.arcane = (stacks + 1).min(3);
                run.class_effects.until[5] = run.elapsed + 6.0;
            }
            if frozen_cast(run, key) {
                run.class_effects.frozen -= 1;
                if run.class_effects.frozen == 0 {
                    run.class_effects.until[23] = 0.0;
                }
            }
        }
        11 => {
            if key == "starfire" {
                run.class_effects.until[5] = 0.0;
            }
            if key == "wrath" {
                run.class_effects.until[6] = 0.0;
            }
            if key == "rejuvenation" {
                run.class_effects.rejuvenation = run.heal_t > 0.0;
                run.class_effects.rejuvenation_heal = 0.0;
                run.class_effects.rejuvenation_extension = run.heal_t * 0.5;
            }
        }
        _ => {}
    }
}

pub(super) fn on_mobility(run: &mut Run) {
    if run.dead_t.is_some() || run.hp <= 0.0 {
        return;
    }
    let slots: &[(usize, f32)] = match run.class.id {
        1 => &[(15, 5.0)],
        2 => &[(7, 6.0)],
        3 => &[(5, 5.0), (7, 5.0)],
        4 => &[(15, 4.0), (22, 4.0), (23, 6.0)],
        5 => &[(7, 6.0)],
        7 => &[(15, 6.0)],
        8 => &[(7, 5.0)],
        11 => &[(15, 4.0)],
        _ => &[],
    };
    for &(slot, duration) in slots {
        if rank(run, slot) > 0.0 {
            run.class_effects.until[slot] = run.elapsed + duration;
        }
    }
    if run.class.id == 4 && rank(run, 23) > 0.0 {
        run.class_effects.ambush = 2;
    }
}

pub(super) fn damage_multiplier(run: &Run, source: Source, en: &Enemy) -> f32 {
    let Some(key) = source.ability else {
        return 1.0;
    };
    if source.secondary {
        return 1.0;
    }
    let direct = !source.periodic;
    let mut mult = 1.0;
    match run.class.id {
        1 if direct && melee(key) && bleeding(en) => mult *= 1.0 + 0.04 * rank(run, 6),
        2 => {
            if direct && holy(2, key) && en.class_effects.mark_until > run.elapsed {
                mult *= 1.0 + 0.03 * rank(run, 21);
            }
            if direct && key == "hammer_of_wrath" && rank(run, 23) > 0.0 && en.hp < en.max_hp * 0.30
            {
                mult *= 1.40;
            }
        }
        3 => {
            if direct && shot(key) {
                if en.class_effects.mark_until > run.elapsed {
                    mult *= 1.0 + 0.03 * rank(run, 13);
                }
                if dot(en, "serpent_sting") {
                    mult *= 1.0 + 0.04 * rank(run, 21);
                }
            }
            if source.periodic && key == "serpent_sting" && rank(run, 23) > 0.0 && slowed(en) {
                mult *= 1.20;
            }
        }
        4 if direct && rogue_direct(key) => {
            if dot(en, "rupture") {
                mult *= 1.0 + 0.04 * rank(run, 5);
            }
            if en.root_t > 0.0 || en.stun_t > 0.0 {
                mult *= 1.0 + 0.04 * rank(run, 21);
            }
        }
        5 => {
            if direct && key == "smite" && dot(en, "holy_fire") {
                mult *= 1.0 + 0.04 * rank(run, 13);
            }
            if shadow(key) && en.class_effects.shadow_until > run.elapsed {
                mult *= 1.0 + 0.01 * rank(run, 21) * f32::from(en.class_effects.shadow_stacks);
            }
        }
        7 if direct && lightning(key) && dot(en, "flame_shock") => {
            mult *= 1.0 + 0.04 * rank(run, 6);
        }
        11 if direct && druid_direct(key) && en.root_t > 0.0 => mult *= 1.0 + 0.04 * rank(run, 14),
        _ => {}
    }
    mult
}

pub(super) fn adjust_critical(
    run: &Run,
    source: Source,
    en: &Enemy,
    damage: f32,
    critical: bool,
    rng: &mut Rng,
) -> (f32, bool) {
    if critical
        || source.secondary
        || source.periodic
        || run.class.id != 8
        || !source.ability.is_some_and(frost_direct)
        || !(source.talent_frozen || slowed(en))
    {
        return (damage, critical);
    }
    let extra = 0.04 * rank(run, 21);
    if extra <= 0.0 {
        return (damage, false);
    }
    // The base roll already failed; use a conditional roll, not a second base crit roll.
    let stats = run.stats();
    let chance = (extra / (1.0 - stats.crit).max(f32::EPSILON)).min(1.0);
    if rng.f32() < chance {
        (damage * stats.crit_multiplier(), true)
    } else {
        (damage, false)
    }
}

pub(super) fn periodic_crits(run: &Run, source: Source) -> bool {
    if source.secondary {
        return false;
    }
    match run.class.id {
        3 => rank(run, 23) > 0.0 && source.ability == Some("serpent_sting"),
        4 => rank(run, 7) > 0.0 && source.ability == Some("rupture"),
        11 => rank(run, 7) > 0.0 && matches!(source.ability, Some("moonfire" | "insect_swarm")),
        _ => false,
    }
}

fn extra_hit(target: Entity, source: Source, effective: f32) -> Hit {
    let mut hit = Hit::direct(target, effective * 0.50, false);
    hit.source = Source {
        ability: source.ability,
        batch: source.batch,
        pulse: source.pulse,
        secondary: true,
        ..Default::default()
    };
    hit
}

fn bleed(en: &mut Enemy, source: Source, effective: f32, rank: f32, key: &'static str) {
    if en.hp <= 0.0 {
        return;
    }
    // Independent contributions preserve the exact total even when another crit lands early.
    en.dots.push(Dot {
        dps: effective * 0.08 * rank / 4.0,
        left: 4.0,
        tick: 1.0,
        every: 1.0,
        extension_left: 0.0,
        source: Source {
            ability: Some(key),
            secondary: true,
            periodic: true,
            ..source
        },
    });
}

pub(super) fn on_damage(
    run: &mut Run,
    source: Source,
    critical: bool,
    target: Entity,
    en: &mut Enemy,
    effective: f32,
    rng: &mut Rng,
) -> Option<Hit> {
    let key = source.ability?;
    if source.secondary
        || en.dead
        || run.dead_t.is_some()
        || run.hp <= 0.0
        || !effective.is_finite()
        || effective <= 0.0
    {
        return None;
    }
    let direct = !source.periodic;
    let now = run.elapsed;
    match run.class.id {
        1 if direct && melee(key) => {
            if critical {
                if matches!(key, "heroic_strike" | "mortal_strike") && rank(run, 5) > 0.0 {
                    bleed(en, source, effective, rank(run, 5), "warrior_bleed");
                }
                if rank(run, 7) > 0.0
                    && now >= run.class_effects.ready[7]
                    && once(run, 7, source, true)
                    && rng.f32() < 0.20
                    && refresh(run, "execute")
                {
                    run.class_effects.ready[7] = now + 6.0;
                }
                if rank(run, 13) > 0.0 && now >= run.class_effects.ready[13] {
                    run.class_effects.until[13] = now + 4.0;
                    run.class_effects.ready[13] = now + 6.0;
                }
            }
            capped_heal(run, effective * 0.004 * rank(run, 14));
        }
        2 => {
            if direct && key == "holy_shock" {
                capped_heal(run, effective * 0.01 * rank(run, 5));
                if critical && rank(run, 6) > 0.0 {
                    run.class_effects.until[6] = now + 6.0;
                }
            }
            if key == "consecration" && rank(run, 14) > 0.0 {
                let cap = run.stats().max_hp * 0.10;
                let existing = if run.shield_t > 0.0 { run.shield } else { 0.0 };
                if existing < cap {
                    shield(
                        run,
                        (existing + effective * 0.005 * rank(run, 14)).min(cap),
                        8.0,
                    );
                }
            }
            if direct && key == "judgement" && rank(run, 21) > 0.0 {
                en.class_effects.mark_until = now + 6.0;
            }
            if direct
                && holy(2, key)
                && critical
                && rank(run, 22) > 0.0
                && now >= run.class_effects.ready[22]
            {
                run.class_effects.until[22] = now + 4.0;
                run.class_effects.ready[22] = now + 6.0;
            }
        }
        3 => {
            if direct && shot(key) {
                capped_heal(run, effective * 0.004 * rank(run, 6));
            }
            if direct && key == "aimed_shot" && rank(run, 13) > 0.0 {
                en.class_effects.mark_until = now + 6.0;
            }
            if direct
                && key == "arcane_shot"
                && critical
                && rank(run, 14) > 0.0
                && now >= run.class_effects.ready[14]
                && once(run, 14, source, false)
            {
                reduce_cooldown(run, "aimed_shot", 0.2 * rank(run, 14));
                run.class_effects.ready[14] = now + 1.0;
            }
            if direct
                && key == "multi_shot"
                && rank(run, 15) > 0.0
                && once(run, 15, source, true)
                && rng.f32() < 0.20
            {
                return Some(extra_hit(target, source, effective));
            }
            if direct && key == "explosive_trap" && rank(run, 22) > 0.0 {
                en.root_t = en
                    .root_t
                    .max(0.2 * rank(run, 22) * if en.boss { 0.3 } else { 1.0 });
            }
        }
        4 => {
            if source.periodic && key == "rupture" && rank(run, 7) > 0.0 {
                capped_heal(run, effective * 0.02);
            }
            if direct && key == "eviscerate" {
                if critical && rank(run, 6) > 0.0 {
                    for d in &mut en.dots {
                        if d.left > 0.0
                            && !d.source.secondary
                            && d.source.ability == Some("rupture")
                        {
                            let extra = d.extension_left.min(f64::from(0.2 * rank(run, 6)));
                            d.left += extra;
                            d.extension_left -= extra;
                        }
                    }
                }
                if rank(run, 14) > 0.0 && now >= run.class_effects.ready[14] {
                    run.class_effects.until[14] = now + 4.0;
                    run.class_effects.ready[14] = now + 6.0;
                }
            }
            if direct
                && key == "sinister_strike"
                && rank(run, 13) > 0.0
                && now >= run.class_effects.ready[13]
                && once(run, 13, source, false)
                && rng.f32() < 0.20
            {
                reduce_cooldown(run, "blade_flurry", 0.2 * rank(run, 13));
                run.class_effects.ready[13] = now + 1.0;
            }
        }
        5 => {
            if direct && holy(5, key) && critical {
                if rank(run, 14) > 0.0
                    && now >= run.class_effects.ready[14]
                    && once(run, 14, source, false)
                {
                    reduce_cooldown(run, "holy_nova", 0.3 * rank(run, 14));
                    run.class_effects.ready[14] = now + 2.0;
                }
                if key == "holy_nova" && rank(run, 15) > 0.0 && now >= run.class_effects.ready[15] {
                    run.class_effects.until[15] = now + 4.0;
                    run.class_effects.ready[15] = now + 6.0;
                }
            }
            if shadow(key) {
                if rank(run, 21) > 0.0 {
                    let old = if en.class_effects.shadow_until > now {
                        en.class_effects.shadow_stacks
                    } else {
                        0
                    };
                    en.class_effects.shadow_stacks = (old + 1).min(3);
                    en.class_effects.shadow_until = now + 6.0;
                }
                capped_heal(run, effective * 0.004 * rank(run, 22));
            }
        }
        7 => {
            if direct && shock(key) && critical {
                if rank(run, 5) > 0.0 {
                    run.class_effects.until[5] = now + 6.0;
                }
                if rank(run, 14) > 0.0 && now >= run.class_effects.ready[14] {
                    run.class_effects.until[14] = now + 4.0;
                    run.class_effects.ready[14] = now + 6.0;
                }
            }
            if direct
                && lightning(key)
                && rank(run, 7) > 0.0
                && once(run, 7, source, true)
                && rng.f32() < 0.20
            {
                return Some(extra_hit(target, source, effective));
            }
            if key == "lightning_shield"
                && rank(run, 13) > 0.0
                && now >= run.class_effects.ready[13]
            {
                run.class_effects.until[13] = now + 6.0;
                run.class_effects.ready[13] = now + 2.0;
            }
            if matches!(key, "searing_totem" | "magma_totem") {
                capped_heal(run, effective * 0.004 * rank(run, 22));
            }
        }
        8 => {
            if direct
                && key == "arcane_missiles"
                && rank(run, 6) > 0.0
                && once(run, 6, source, true)
                && rng.f32() < 0.04 * rank(run, 6)
            {
                return Some(extra_hit(target, source, effective));
            }
            if direct && fire(key) {
                if critical && rank(run, 13) > 0.0 {
                    bleed(en, source, effective, rank(run, 13), "ignite");
                }
                if rank(run, 14) > 0.0 && once(run, 14, source, true) {
                    run.class_effects.fire_streak = if critical {
                        run.class_effects.fire_streak.saturating_add(1)
                    } else {
                        0
                    };
                    if run.class_effects.fire_streak >= 2 {
                        run.class_effects.fire_streak = 0;
                        if now >= run.class_effects.ready[14]
                            && rng.f32() < 0.20 * rank(run, 14)
                            && refresh(run, "pyroblast")
                        {
                            run.class_effects.ready[14] = now + 8.0;
                        }
                    }
                }
                if critical && rank(run, 15) > 0.0 && once(run, 15, source, true) {
                    let old = if active(run, 15) {
                        run.class_effects.fire
                    } else {
                        0
                    };
                    run.class_effects.fire = (old + 1).min(3);
                    run.class_effects.until[15] = now + 6.0;
                }
            }
            if direct
                && key == "frost_nova"
                && rank(run, 22) > 0.0
                && now >= run.class_effects.ready[22]
            {
                shield(run, run.stats().max_hp * 0.02 * rank(run, 22), 6.0);
                run.class_effects.ready[22] = now + 6.0;
            }
            if frost(key) && rank(run, 23) > 0.0 && once(run, 23, source, false) && rng.f32() < 0.15
            {
                let old = if active(run, 23) {
                    run.class_effects.frozen
                } else {
                    0
                };
                run.class_effects.frozen = (old + 1).min(2);
                run.class_effects.until[23] = now + 6.0;
            }
        }
        11 => {
            if direct && critical && rank(run, 5) > 0.0 {
                if key == "wrath" {
                    run.class_effects.until[5] = now + 6.0;
                }
                if key == "starfire" {
                    run.class_effects.until[6] = now + 6.0;
                }
            }
            if source.periodic
                && key == "moonfire"
                && rank(run, 6) > 0.0
                && now >= run.class_effects.ready[6]
                && once(run, 6, source, false)
                && rng.f32() < 0.20
            {
                reduce_cooldown(run, "starfire", 0.2 * rank(run, 6));
                run.class_effects.ready[6] = now + 1.0;
            }
            if key == "thorns" && rank(run, 13) > 0.0 {
                let slow = 1.0 - 0.04 * rank(run, 13) * if en.boss { 0.5 } else { 1.0 };
                en.slow = if en.slow_t > 0.0 {
                    en.slow.min(slow)
                } else {
                    slow
                };
                en.slow_t = en.slow_t.max(2.0);
            }
            if direct
                && matches!(key, "wrath" | "starfire")
                && rank(run, 23) > 0.0
                && run.heal_t > 0.0
                && run.class_effects.rejuvenation
                && now >= run.class_effects.ready[23]
                && once(run, 23, source, false)
            {
                let extra = run.class_effects.rejuvenation_extension.min(1.0);
                run.heal_t += extra;
                run.class_effects.rejuvenation_extension -= extra;
                run.class_effects.ready[23] = now + 1.0;
            }
        }
        _ => {}
    }
    None
}

pub(super) fn incoming(run: &Run) -> f32 {
    let max = run.stats().max_hp;
    let mut mult = 1.0;
    match run.class.id {
        1 => {
            if run.hp < max * 0.35 {
                mult *= 1.0 - 0.02 * rank(run, 22);
            }
            if active(run, 23) && rank(run, 23) > 0.0 {
                mult *= 0.60;
            }
        }
        2 => {
            if run.class_effects.consecration {
                mult *= 1.0 - 0.02 * rank(run, 13);
            }
            if run.hp < max * 0.35 && rank(run, 15) > 0.0 {
                mult *= 0.80;
            }
        }
        3 if active(run, 7) && rank(run, 7) > 0.0 => mult *= 0.85,
        4 if active(run, 22) => mult *= 1.0 - 0.03 * rank(run, 22),
        5 if rank(run, 23) > 0.0 => mult *= 0.90,
        7 if run.class_effects.totem => mult *= 1.0 - 0.02 * rank(run, 21),
        11 => {
            if active(run, 15) && rank(run, 15) > 0.0 {
                mult *= 0.80;
            }
            if run.heal_t > 0.0 && run.class_effects.rejuvenation {
                mult *= 1.0 - 0.02 * rank(run, 21);
            }
        }
        _ => {}
    }
    mult.clamp(0.0, 1.0)
}

pub(super) fn before_contact(run: &mut Run, projected_hp: f32) {
    if run.dead_t.is_some() || run.hp <= 0.0 {
        return;
    }
    let now = run.elapsed;
    let max = run.stats().max_hp;
    if run.class.id == 1
        && rank(run, 23) > 0.0
        && run.hp >= max * 0.30
        && projected_hp < max * 0.30
        && now >= run.class_effects.ready[23]
    {
        run.class_effects.until[23] = now + 4.0;
        run.class_effects.ready[23] = now + 30.0;
    }
    if run.class.id == 7
        && rank(run, 23) > 0.0
        && run.class_effects.totem
        && projected_hp < max * 0.35
        && now >= run.class_effects.ready[23]
    {
        shield(run, max * 0.20, 8.0);
        run.class_effects.ready[23] = now + 30.0;
    }
}

pub(super) fn on_contact(run: &mut Run, absorbed: f32, damage: f32) -> f32 {
    if run.dead_t.is_some() || run.hp <= 0.0 {
        return 0.0;
    }
    if run.class.id == 1
        && damage > 0.0
        && rank(run, 21) > 0.0
        && run.elapsed >= run.class_effects.ready[21]
    {
        reduce_cooldown(run, "thunder_clap", 0.2 * rank(run, 21));
        run.class_effects.ready[21] = run.elapsed + 1.0;
    }
    if run.class.id == 5 {
        absorbed.max(0.0) * 0.05 * rank(run, 6)
    } else {
        0.0
    }
}

pub(super) fn tick_healing(run: &mut Run, _dt: f32, effective_heal: f32, was_healing: bool) {
    if run.class.id != 11
        || run.dead_t.is_some()
        || run.hp <= 0.0
        || !run.class_effects.rejuvenation
    {
        return;
    }
    if was_healing {
        run.class_effects.rejuvenation_heal += effective_heal.max(0.0);
        if run.heal_t <= 0.0 {
            let grant = (run.class_effects.rejuvenation_heal * 0.04 * rank(run, 22))
                .min(run.stats().max_hp * 0.10);
            shield(run, grant, 8.0);
            run.class_effects.rejuvenation = false;
            run.class_effects.rejuvenation_heal = 0.0;
            run.class_effects.rejuvenation_extension = 0.0;
        }
    }
}

pub(super) fn set_environment(run: &mut Run, consecration: bool, totem: bool) {
    run.class_effects.consecration = consecration;
    run.class_effects.totem = totem;
}

pub(super) fn thorns_damage(run: &Run) -> f32 {
    run.thorns
        * if run.class.id == 11 && rank(run, 15) > 0.0 && active(run, 15) {
            1.30
        } else {
            1.0
        }
}

pub(super) fn drain_fraction(run: &Run, key: Option<&str>, base: f32) -> f32 {
    if run.class.id == 5 && key == Some("holy_nova") && rank(run, 15) > 0.0 {
        base * 1.50
    } else {
        base
    }
}

/// Holy Nova's native Drain has its own per-cast cap, not the talent-leech budget.
pub(super) fn cap_drain(run: &mut Run, source: Source, heal: f32) -> f32 {
    if run.dead_t.is_some() || run.hp <= 0.0 || !heal.is_finite() || heal <= 0.0 {
        return 0.0;
    }
    if run.class.id != 5 || source.ability != Some("holy_nova") || rank(run, 15) <= 0.0 {
        return heal;
    }
    if source.secondary {
        return 0.0;
    }
    let cap = run.stats().max_hp * 0.04;
    let used = run
        .class_effects
        .nova_heals
        .entry(source.batch)
        .or_default();
    let grant = heal.min((cap - *used).max(0.0));
    *used += grant;
    grant
}

pub(super) fn buffs(run: &Run) -> Vec<(usize, &'static str, &'static str, f32)> {
    let mut out = Vec::new();
    let entries: &[(usize, &str, &str)] = match run.class.id {
        1 => &[
            (13, "Flurry", "乱舞"),
            (15, "Recklessness", "鲁莽"),
            (23, "Last Stand", "破釜沉舟"),
        ],
        2 => &[
            (6, "Infusion of Light", "圣光灌注"),
            (7, "Divine Favor", "神恩术"),
            (22, "Vengeance", "复仇"),
        ],
        3 => &[
            (5, "Rapid Fire", "急速射击"),
            (7, "Bestial Wrath", "狂野怒火"),
        ],
        4 => &[
            (14, "Slice and Dice", "切割"),
            (15, "Adrenaline Rush", "冲动"),
            (22, "Elusiveness", "飘忽不定"),
            (23, "Shadow Dance (charges)", "暗影之舞（层数）"),
        ],
        5 => &[
            (7, "Power Infusion", "能量灌注"),
            (15, "Holy Inspiration", "神圣鼓舞"),
        ],
        7 => &[
            (5, "Elemental Focus", "元素集中"),
            (13, "Static Charge", "静电充能"),
            (14, "Elemental Flurry", "元素乱舞"),
            (15, "Elemental Mastery", "元素掌握"),
        ],
        8 => &[
            (5, "Arcane Power (stacks)", "奥术能量（层数）"),
            (7, "Arcane Surge", "奥术涌动"),
            (15, "Combustion (stacks)", "燃烧（层数）"),
            (23, "Fingers of Frost (charges)", "寒冰指（层数）"),
        ],
        11 => &[
            (5, "Nature's Grace: Starfire", "自然之赐：星火术"),
            (6, "Nature's Grace: Wrath", "自然之赐：愤怒"),
            (15, "Barkskin", "树皮术"),
        ],
        _ => &[],
    };
    for &(slot, en, zh) in entries {
        let talent_slot = if run.class.id == 11 && slot == 6 {
            5
        } else {
            slot
        };
        if rank(run, talent_slot) > 0.0 && active(run, slot) {
            let (en, zh) = match (run.class.id, slot) {
                (4, 23) if run.class_effects.ambush == 2 => {
                    ("Shadow Dance: 2 casts", "暗影之舞：2次")
                }
                (4, 23) => ("Shadow Dance: 1 cast", "暗影之舞：1次"),
                (8, 5) => match run.class_effects.arcane {
                    1 => ("Arcane Power: 1", "奥术能量：1层"),
                    2 => ("Arcane Power: 2", "奥术能量：2层"),
                    _ => ("Arcane Power: 3", "奥术能量：3层"),
                },
                (8, 15) => match run.class_effects.fire {
                    1 => ("Combustion: 1", "燃烧：1层"),
                    2 => ("Combustion: 2", "燃烧：2层"),
                    _ => ("Combustion: 3", "燃烧：3层"),
                },
                (8, 23) if run.class_effects.frozen == 2 => {
                    ("Fingers of Frost: 2 casts", "寒冰指：2次")
                }
                (8, 23) => ("Fingers of Frost: 1 cast", "寒冰指：1次"),
                _ => (en, zh),
            };
            out.push((
                talent_slot,
                en,
                zh,
                run.class_effects.until[slot] - run.elapsed,
            ));
        }
    }
    if run.class.id == 5 && rank(run, 5) > 0.0 && run.shield_t > 0.0 && run.shield > 0.0 {
        out.push((5, "Borrowed Time", "争分夺秒", run.shield_t));
    }
    if run.class.id == 2 && rank(run, 13) > 0.0 && run.class_effects.consecration {
        out.push((13, "Consecrated Ground", "奉献之地", 0.1));
    }
    if run.class.id == 7 && rank(run, 21) > 0.0 && run.class_effects.totem {
        out.push((21, "Totemic Protection", "图腾庇护", 0.1));
    }
    if run.class.id == 11
        && rank(run, 21) > 0.0
        && run.class_effects.rejuvenation
        && run.heal_t > 0.0
    {
        out.push((21, "Rejuvenating Protection", "回春庇护", run.heal_t));
    }
    if run.class.id == 1 && rank(run, 22) > 0.0 && run.hp < run.stats().max_hp * 0.35 {
        out.push((22, "Bloodied Resolve", "浴血意志", 0.1));
    }
    if run.class.id == 2 && rank(run, 15) > 0.0 && run.hp < run.stats().max_hp * 0.35 {
        out.push((15, "Ardent Defender", "炽热防御者", 0.1));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::{data, Owned};
    use super::*;

    fn run(class: u8, ranks: &[(usize, u8)]) -> Run {
        let mut run = Run {
            class: data::CLASSES.iter().find(|c| c.id == class).unwrap(),
            ..Default::default()
        };
        for &(slot, rank) in ranks {
            run.talents.0[slot] = rank;
        }
        run
    }

    fn enemy() -> Enemy {
        Enemy {
            def: &data::ENEMIES[0],
            hp: 1000.0,
            max_hp: 1000.0,
            speed: 1.0,
            damage: 1.0,
            radius: 1.0,
            xp: 1,
            boss: false,
            attack_cd: 0.0,
            slow: 1.0,
            slow_t: 0.0,
            root_t: 0.0,
            stun_t: 0.0,
            fear_t: 0.0,
            weaken: 1.0,
            weaken_t: 0.0,
            dots: Vec::new(),
            warlock: Default::default(),
            class_effects: Default::default(),
            dead: false,
        }
    }

    fn source(key: &'static str, batch: u64) -> Source {
        Source {
            ability: Some(key),
            batch,
            ..Default::default()
        }
    }

    fn add_dot(en: &mut Enemy, key: &'static str, duration: f64) {
        en.dots.push(Dot {
            dps: 10.0,
            left: duration,
            tick: 1.0,
            every: 1.0,
            extension_left: duration * 0.5,
            source: source(key, 1),
        });
    }

    fn learn(run: &mut Run, key: &'static str, cd: f32) {
        run.abilities.push(Owned {
            def: data::ability(key),
            level: 1,
            growth: 1.0,
            cd,
        });
    }

    fn deal(run: &mut Run, en: &mut Enemy, source: Source, critical: bool) -> Option<Hit> {
        on_damage(
            run,
            source,
            critical,
            Entity::PLACEHOLDER,
            en,
            100.0,
            &mut Rng(0),
        )
    }

    fn cast(run: &Run, key: &'static str) -> (talents::Cast, Stats) {
        let mut stats = run.stats();
        let mut cast = talents::cast(data::ability(key), 1, &stats);
        modify_cast(run, key, &mut cast, &mut stats);
        (cast, stats)
    }

    fn near(a: f32, b: f32) {
        assert!((a - b).abs() < 0.0001, "{a} != {b}");
    }

    #[test]
    fn zero_talent_runtime_is_identity_for_all_eight_classes() {
        for id in [1, 2, 3, 4, 5, 7, 8, 11] {
            let mut r = run(id, &[]);
            r.hp = 10.0;
            r.talent_leech_budget = 100.0;
            r.shield = 10.0;
            r.shield_t = 3.0;
            r.heal_t = 5.0;
            let before = r.stats();
            on_mobility(&mut r);
            set_environment(&mut r, true, true);
            before_contact(&mut r, -100.0);
            let mut en = enemy();
            en.root_t = 2.0;
            for (batch, &key) in r.class.pool.iter().enumerate() {
                let src = source(key, batch as u64);
                near(damage_multiplier(&r, src, &en), 1.0);
                assert!(!periodic_crits(&r, src));
                assert!(!frozen_cast(&r, key));
                let baseline = talents::cast(data::ability(key), 1, &before);
                let (preview, _) = cast(&r, key);
                near(preview.dmg, baseline.dmg);
                near(preview.dot, baseline.dot);
                assert!(deal(&mut r, &mut en, src, true).is_none());
                on_cast(&mut r, key, batch as u64);
            }
            near(r.stats().might, before.might);
            near(r.stats().haste, before.haste);
            near(r.stats().crit, before.crit);
            near(incoming(&r), 1.0);
            near(on_contact(&mut r, 100.0, 100.0), 0.0);
            tick_healing(&mut r, 5.0, 100.0, true);
            near(r.hp, 10.0);
            near(r.shield, 10.0);
            assert!(en.dots.is_empty());
            assert!(buffs(&r).is_empty());
        }
    }

    #[test]
    fn warrior_bleed_totals_raw_rank_and_does_not_recurse() {
        for rank in [1, 5] {
            let mut r = run(1, &[(5, rank), (6, rank)]);
            let mut en = enemy();
            let src = source("heroic_strike", 1);
            near(damage_multiplier(&r, src, &en), 1.0);
            deal(&mut r, &mut en, src, true);
            assert_eq!(en.dots.len(), 1);
            near(en.dots[0].dps * 4.0, 8.0 * f32::from(rank));
            near(
                damage_multiplier(&r, src, &en),
                1.0 + 0.04 * f32::from(rank),
            );
            let secondary = en.dots[0].source;
            near(damage_multiplier(&r, secondary, &en), 1.0);
            assert!(deal(&mut r, &mut en, secondary, true).is_none());
            assert_eq!(en.dots.len(), 1);
            let mut total = 0.0;
            let d = &mut en.dots[0];
            let mut frame = 10.0;
            while d.next_tick(&mut frame) {
                total += d.dps;
            }
            near(total, 8.0 * f32::from(rank));
        }
    }

    #[test]
    fn warrior_resets_only_learned_execute_and_icd_boundary_is_inclusive() {
        let mut r = run(1, &[(7, 1), (13, 5)]);
        let mut en = enemy();
        learn(&mut r, "execute", 10.0);
        deal(&mut r, &mut en, source("cleave", 1), true);
        near(r.abilities[0].cd, 0.0);
        near(r.stats().haste, 0.15);
        r.abilities[0].cd = 10.0;
        r.elapsed = 5.999;
        deal(&mut r, &mut en, source("cleave", 2), true);
        near(r.abilities[0].cd, 10.0);
        r.elapsed = 6.0;
        deal(&mut r, &mut en, source("cleave", 1), true);
        near(r.abilities[0].cd, 10.0);
        deal(&mut r, &mut en, source("cleave", 3), true);
        near(r.abilities[0].cd, 0.0);
        assert_eq!(r.abilities.len(), 1);
        let mut unlearned = run(1, &[(7, 1)]);
        deal(&mut unlearned, &mut en, source("cleave", 4), true);
        assert!(unlearned.abilities.is_empty());
        near(unlearned.class_effects.ready[7], 0.0);
    }

    #[test]
    fn warrior_low_health_wall_starts_before_lethal_hit_without_revival() {
        let mut r = run(1, &[(21, 5), (22, 5), (23, 1)]);
        let max = r.stats().max_hp;
        learn(&mut r, "thunder_clap", 4.0);
        r.hp = max * 0.35;
        near(incoming(&r), 1.0);
        r.hp -= 0.001;
        near(incoming(&r), 0.90);
        r.hp = max * 0.30;
        before_contact(&mut r, max * 0.30);
        assert!(!active(&r, 23));
        before_contact(&mut r, -100.0);
        near(incoming(&r), 0.54);
        near(r.hp, max * 0.30);
        r.elapsed = 4.0;
        near(incoming(&r), 0.90);
        before_contact(&mut r, -100.0);
        assert!(!active(&r, 23));
        near(on_contact(&mut r, 100.0, 0.0), 0.0);
        near(r.abilities[0].cd, 4.0);
        on_contact(&mut r, 0.0, 1.0);
        near(r.abilities[0].cd, 3.0);
        on_contact(&mut r, 0.0, 1.0);
        near(r.abilities[0].cd, 3.0);
        r.elapsed = 5.0;
        on_contact(&mut r, 0.0, 1.0);
        near(r.abilities[0].cd, 2.0);
        r.hp = 0.0;
        before_contact(&mut r, -100.0);
        near(r.hp, 0.0);
    }

    #[test]
    fn paladin_charges_are_immutable_previews_and_consumed_per_successful_cast() {
        let mut r = run(2, &[(6, 5), (7, 1)]);
        let mut en = enemy();
        let base = cast(&r, "hammer_of_wrath").0.dmg;
        deal(&mut r, &mut en, source("holy_shock", 1), true);
        near(cast(&r, "hammer_of_wrath").0.dmg, base * 1.25);
        near(cast(&r, "hammer_of_wrath").0.dmg, base * 1.25);
        on_cast(&mut r, "exorcism", 2);
        near(cast(&r, "hammer_of_wrath").0.dmg, base * 1.25);
        on_cast(&mut r, "hammer_of_wrath", 3);
        near(cast(&r, "hammer_of_wrath").0.dmg, base);
        on_mobility(&mut r);
        near(cast(&r, "holy_shock").1.crit, 1.0);
        on_cast(&mut r, "holy_shock", 4);
        assert!(cast(&r, "holy_shock").1.crit < 1.0);
        on_mobility(&mut r);
        r.elapsed = 6.0;
        assert!(cast(&r, "holy_shock").1.crit < 1.0);
    }

    #[test]
    fn paladin_target_mark_applies_after_trigger_and_execute_includes_bosses() {
        let mut r = run(2, &[(21, 5), (23, 1)]);
        let mut en = enemy();
        let src = source("judgement", 1);
        near(damage_multiplier(&r, src, &en), 1.0);
        deal(&mut r, &mut en, src, false);
        near(damage_multiplier(&r, src, &en), 1.15);
        near(damage_multiplier(&r, source("consecration", 1), &en), 1.0);
        r.elapsed = 6.0;
        near(damage_multiplier(&r, src, &en), 1.0);
        en.boss = true;
        en.hp = 300.0;
        near(
            damage_multiplier(&r, source("hammer_of_wrath", 2), &en),
            1.0,
        );
        en.hp -= 0.001;
        near(
            damage_multiplier(&r, source("hammer_of_wrath", 2), &en),
            1.40,
        );
    }

    #[test]
    fn paladin_consecration_shield_has_bounded_addition_preserves_stronger_shields() {
        let mut r = run(2, &[(13, 5), (14, 5), (15, 1)]);
        let mut en = enemy();
        let cap = r.stats().max_hp * 0.10;
        for batch in 0..100 {
            deal(&mut r, &mut en, source("consecration", batch), false);
        }
        near(r.shield, cap);
        near(r.shield_t, 8.0);
        r.shield = cap * 2.0;
        deal(&mut r, &mut en, source("consecration", 101), false);
        near(r.shield, cap * 2.0);
        set_environment(&mut r, false, false);
        near(incoming(&r), 1.0);
        set_environment(&mut r, true, false);
        near(incoming(&r), 0.90);
        r.hp = r.stats().max_hp * 0.35 - 0.001;
        near(incoming(&r), 0.72);
    }

    #[test]
    fn hunter_mark_sting_bonus_and_roots_use_only_eligible_sources() {
        let mut r = run(3, &[(13, 5), (21, 5), (22, 5), (23, 1)]);
        let mut en = enemy();
        let shot = source("arcane_shot", 1);
        add_dot(&mut en, "serpent_sting", 6.0);
        near(damage_multiplier(&r, shot, &en), 1.20);
        deal(&mut r, &mut en, source("aimed_shot", 2), false);
        near(damage_multiplier(&r, shot, &en), 1.20 * 1.15);
        en.boss = true;
        deal(&mut r, &mut en, source("explosive_trap", 3), false);
        near(en.root_t, 0.30);
        let tick = Source {
            periodic: true,
            ..source("serpent_sting", 4)
        };
        assert!(periodic_crits(&r, tick));
        near(damage_multiplier(&r, tick, &en), 1.20);
        near(damage_multiplier(&r, source("volley", 4), &en), 1.0);
        r.elapsed = 6.0;
        near(damage_multiplier(&r, shot, &en), 1.20);
    }

    #[test]
    fn hunter_multishot_proc_is_once_per_cast_and_never_recursive() {
        let mut r = run(3, &[(15, 1)]);
        let mut en = enemy();
        let src = source("multi_shot", 42);
        let hit = deal(&mut r, &mut en, src, false).unwrap();
        near(hit.damage, 50.0);
        near(hit.dot, 0.0);
        near(hit.splash, 0.0);
        assert!(!hit.crit);
        assert!(hit.source.secondary);
        assert!(deal(&mut r, &mut en, src, false).is_none());
        assert!(deal(&mut r, &mut en, Source { pulse: 1, ..src }, false).is_none());
        assert!(deal(&mut r, &mut en, hit.source, true).is_none());
    }

    #[test]
    fn hunter_cooldown_reduction_has_one_second_icd_and_mobility_expires_exactly() {
        let mut r = run(3, &[(5, 5), (7, 1), (14, 5)]);
        let mut en = enemy();
        learn(&mut r, "aimed_shot", 4.0);
        deal(&mut r, &mut en, source("arcane_shot", 1), true);
        near(r.abilities[0].cd, 3.0);
        r.elapsed = 0.999;
        deal(&mut r, &mut en, source("arcane_shot", 2), true);
        near(r.abilities[0].cd, 3.0);
        r.elapsed = 1.0;
        deal(&mut r, &mut en, source("arcane_shot", 3), true);
        near(r.abilities[0].cd, 2.0);
        on_mobility(&mut r);
        near(r.stats().haste, 0.15);
        near(incoming(&r), 0.85);
        near(r.stats().might, 1.20);
        r.elapsed = 6.0;
        near(r.stats().haste, 0.0);
        near(incoming(&r), 1.0);
        near(r.stats().might, 1.0);
    }

    #[test]
    fn rogue_rupture_extension_has_a_lifetime_budget_and_preserves_tick_damage() {
        let mut r = run(4, &[(5, 5), (6, 5), (7, 1), (21, 5)]);
        let mut en = enemy();
        add_dot(&mut en, "rupture", 5.0);
        let src = source("eviscerate", 1);
        near(damage_multiplier(&r, src, &en), 1.20);
        en.stun_t = 2.0;
        near(damage_multiplier(&r, src, &en), 1.44);
        for batch in 1..10 {
            deal(&mut r, &mut en, source("eviscerate", batch), true);
        }
        assert_eq!(en.dots[0].left, 7.5);
        assert_eq!(en.dots[0].extension_left, 0.0);
        near(en.dots[0].dps, 10.0);
        assert!(periodic_crits(&r, source("rupture", 2)));
        assert!(!periodic_crits(&r, source("sinister_strike", 2)));
        near(damage_multiplier(&r, source("thorns", 3), &en), 1.0);
    }

    #[test]
    fn rogue_ambush_is_two_casts_not_two_hits_and_refreshes_instead_of_stacking() {
        let mut r = run(4, &[(15, 1), (22, 5), (23, 1)]);
        let mut en = enemy();
        let base = cast(&r, "sinister_strike").0.dmg;
        on_mobility(&mut r);
        near(cast(&r, "sinister_strike").0.dmg, base * 1.40);
        for batch in 1..10 {
            deal(&mut r, &mut en, source("sinister_strike", batch), false);
        }
        assert_eq!(r.class_effects.ambush, 2);
        on_cast(&mut r, "sinister_strike", 1);
        assert_eq!(r.class_effects.ambush, 1);
        near(cast(&r, "sinister_strike").0.dmg, base * 1.40);
        on_cast(&mut r, "sinister_strike", 2);
        near(cast(&r, "sinister_strike").0.dmg, base);
        near(r.stats().haste, 0.25);
        near(incoming(&r), 0.85);
        on_mobility(&mut r);
        assert_eq!(r.class_effects.ambush, 2);
        r.elapsed = 4.0;
        near(r.stats().haste, 0.0);
        near(incoming(&r), 1.0);
        r.elapsed = 6.0;
        near(cast(&r, "sinister_strike").0.dmg, base);
        assert!(buffs(&r).is_empty());
    }

    #[test]
    fn rogue_strike_reset_roll_and_finisher_haste_have_separate_cooldowns() {
        let mut r = run(4, &[(13, 5), (14, 5)]);
        let mut en = enemy();
        learn(&mut r, "blade_flurry", 3.0);
        let src = source("sinister_strike", 1);
        deal(&mut r, &mut en, src, false);
        near(r.abilities[0].cd, 2.0);
        r.elapsed = 0.999;
        deal(&mut r, &mut en, source("sinister_strike", 2), false);
        near(r.abilities[0].cd, 2.0);
        r.elapsed = 1.0;
        deal(&mut r, &mut en, src, false);
        near(r.abilities[0].cd, 2.0);
        deal(&mut r, &mut en, source("sinister_strike", 3), false);
        near(r.abilities[0].cd, 1.0);
        deal(&mut r, &mut en, source("eviscerate", 4), false);
        near(r.stats().haste, 0.15);
        r.elapsed = 5.0;
        near(r.stats().haste, 0.0);
        deal(&mut r, &mut en, source("eviscerate", 5), false);
        near(r.stats().haste, 0.0);
        r.elapsed = 7.0;
        deal(&mut r, &mut en, source("eviscerate", 6), false);
        near(r.stats().haste, 0.15);
    }

    #[test]
    fn priest_shield_haste_reflection_and_holy_shadow_branches_coexist() {
        let mut r = run(5, &[(5, 5), (6, 5), (7, 1), (13, 5), (23, 1)]);
        let mut en = enemy();
        r.shield = 10.0;
        r.shield_t = 2.0;
        near(r.stats().haste, 0.10);
        near(on_contact(&mut r, 8.0, 2.0), 2.0);
        r.shield = 0.0;
        near(r.stats().haste, 0.0);
        near(on_contact(&mut r, 0.0, 2.0), 0.0);
        add_dot(&mut en, "holy_fire", 4.0);
        near(damage_multiplier(&r, source("smite", 1), &en), 1.20);
        let pain = cast(&r, "shadow_word_pain").0.dot;
        r.talents.0[23] = 0;
        near(pain, cast(&r, "shadow_word_pain").0.dot * 1.20);
        r.talents.0[23] = 1;
        near(incoming(&r), 0.90);
        on_mobility(&mut r);
        near(r.stats().haste, 0.20);
        assert!(cast(&r, "smite").0.dmg > 0.0);
    }

    #[test]
    fn priest_shadow_vulnerability_stacks_after_trigger_and_expires_at_six_seconds() {
        let mut r = run(5, &[(21, 5)]);
        let mut en = enemy();
        let src = source("mind_blast", 1);
        near(damage_multiplier(&r, src, &en), 1.0);
        deal(&mut r, &mut en, src, false);
        near(damage_multiplier(&r, src, &en), 1.05);
        for batch in 2..6 {
            deal(
                &mut r,
                &mut en,
                Source {
                    periodic: true,
                    ..source("shadow_word_pain", batch)
                },
                false,
            );
        }
        near(damage_multiplier(&r, src, &en), 1.15);
        near(damage_multiplier(&r, source("smite", 2), &en), 1.0);
        r.elapsed = 6.0;
        near(damage_multiplier(&r, src, &en), 1.0);
        deal(&mut r, &mut en, source("mind_blast", 6), false);
        near(damage_multiplier(&r, src, &en), 1.05);
    }

    #[test]
    fn priest_holy_nova_drain_is_per_cast_and_other_drain_is_unchanged() {
        let mut r = run(5, &[(14, 5), (15, 1)]);
        let mut en = enemy();
        learn(&mut r, "holy_nova", 5.0);
        let src = source("holy_nova", 100);
        near(drain_fraction(&r, Some("holy_nova"), 0.10), 0.15);
        near(drain_fraction(&r, Some("death_coil"), 0.10), 0.10);
        let cap = r.stats().max_hp * 0.04;
        near(cap_drain(&mut r, src, cap * 0.75), cap * 0.75);
        near(cap_drain(&mut r, src, cap), cap * 0.25);
        near(cap_drain(&mut r, src, cap), 0.0);
        near(cap_drain(&mut r, source("holy_nova", 101), cap * 2.0), cap);
        near(cap_drain(&mut r, source("death_coil", 100), 100.0), 100.0);
        near(
            cap_drain(
                &mut r,
                Source {
                    secondary: true,
                    ..src
                },
                cap,
            ),
            0.0,
        );
        let base = cast(&r, "smite").0.dmg;
        deal(&mut r, &mut en, src, true);
        near(r.abilities[0].cd, 3.5);
        near(cast(&r, "smite").0.dmg, base * 1.15);
        r.elapsed = 2.0;
        deal(&mut r, &mut en, source("smite", 102), true);
        near(r.abilities[0].cd, 2.0);
        r.elapsed = 4.0;
        near(cast(&r, "smite").0.dmg, base);
    }

    #[test]
    fn shaman_charges_and_mobility_frequency_are_per_cast() {
        let mut r = run(7, &[(5, 5), (6, 5), (13, 5), (14, 5), (15, 1)]);
        let mut en = enemy();
        let bolt = cast(&r, "lightning_bolt").0.dmg;
        let shock = cast(&r, "earth_shock").0.dmg;
        let shield_cd = cast(&r, "lightning_shield").0.cooldown;
        deal(&mut r, &mut en, source("earth_shock", 1), true);
        near(
            cast(&r, "chain_lightning").0.dmg
                / talents::cast(data::ability("chain_lightning"), 1, &r.stats()).dmg,
            1.25,
        );
        near(cast(&r, "lightning_bolt").0.dmg, bolt * 1.25);
        on_cast(&mut r, "lightning_bolt", 2);
        near(cast(&r, "lightning_bolt").0.dmg, bolt);
        deal(&mut r, &mut en, source("lightning_shield", 3), false);
        near(cast(&r, "earth_shock").0.dmg, shock * 1.25);
        on_cast(&mut r, "earth_shock", 4);
        near(cast(&r, "earth_shock").0.dmg, shock);
        add_dot(&mut en, "flame_shock", 6.0);
        near(
            damage_multiplier(&r, source("lightning_bolt", 4), &en),
            1.20,
        );
        on_mobility(&mut r);
        near(cast(&r, "earth_shock").0.dmg, shock * 1.20);
        // The temporary haste also changes the cooldown; compare an unbuffed clone snapshot.
        let raw_cd = talents::cast(data::ability("lightning_shield"), 1, &r.stats()).cooldown;
        near(cast(&r, "lightning_shield").0.cooldown, raw_cd / 1.50);
        assert!(raw_cd <= shield_cd);
        r.elapsed = 6.0;
        near(cast(&r, "earth_shock").0.dmg, shock);
    }

    #[test]
    fn shaman_overload_is_one_nonrecursive_hit_per_chain_cast() {
        let mut r = run(7, &[(7, 1)]);
        let mut en = enemy();
        let src = source("chain_lightning", 7);
        let hit = deal(&mut r, &mut en, src, true).unwrap();
        near(hit.damage, 50.0);
        assert!(hit.source.secondary);
        assert!(deal(&mut r, &mut en, Source { pulse: 2, ..src }, true).is_none());
        assert!(deal(&mut r, &mut en, hit.source, true).is_none());
    }

    #[test]
    fn shaman_low_health_shield_requires_live_nearby_totem_and_respects_icd() {
        let mut r = run(7, &[(21, 5), (23, 1)]);
        let max = r.stats().max_hp;
        before_contact(&mut r, -10.0);
        near(r.shield, 0.0);
        set_environment(&mut r, false, true);
        near(incoming(&r), 0.90);
        before_contact(&mut r, max * 0.35);
        near(r.shield, 0.0);
        before_contact(&mut r, -10.0);
        near(r.shield, max * 0.20);
        near(r.shield_t, 8.0);
        r.shield = 0.0;
        r.elapsed = 29.999;
        before_contact(&mut r, -10.0);
        near(r.shield, 0.0);
        r.elapsed = 30.0;
        r.shield = max;
        r.shield_t = 10.0;
        before_contact(&mut r, -10.0);
        near(r.shield, max);
        near(r.shield_t, 10.0);
        set_environment(&mut r, false, false);
        near(incoming(&r), 1.0);
    }

    #[test]
    fn mage_arcane_stacks_apply_after_success_and_expire_at_exact_boundary() {
        let mut r = run(8, &[(5, 5)]);
        let base = cast(&r, "arcane_missiles").0.dmg;
        for stack in 0..4 {
            near(
                cast(&r, "arcane_missiles").0.dmg,
                base * (1.0 + 0.05 * stack.min(3) as f32),
            );
            near(
                cast(&r, "arcane_missiles").0.dmg,
                base * (1.0 + 0.05 * stack.min(3) as f32),
            );
            on_cast(&mut r, "arcane_missiles", stack);
        }
        assert_eq!(r.class_effects.arcane, 3);
        r.elapsed = 6.0;
        near(cast(&r, "arcane_missiles").0.dmg, base);
        on_cast(&mut r, "arcane_explosion", 5);
        assert_eq!(r.class_effects.arcane, 1);
    }

    #[test]
    fn mage_arcane_echo_uses_raw_four_percent_per_rank_and_one_roll_per_cast() {
        // Choose a deterministic first roll in [4%, 20%) to distinguish ranks 1 and 5.
        let seed = (1..10000)
            .find(|&seed| {
                let roll = Rng(seed).f32();
                (0.04..0.20).contains(&roll)
            })
            .unwrap();
        let mut en = enemy();
        let src = source("arcane_missiles", 7);
        for (rank, expect) in [(1, false), (5, true)] {
            let mut r = run(8, &[(6, rank)]);
            let hit = on_damage(
                &mut r,
                src,
                false,
                Entity::PLACEHOLDER,
                &mut en,
                80.0,
                &mut Rng(seed),
            );
            assert_eq!(hit.is_some(), expect);
            if let Some(hit) = hit {
                near(hit.damage, 40.0);
            }
            assert!(deal(&mut r, &mut en, Source { pulse: 2, ..src }, false).is_none());
        }
    }

    #[test]
    fn mage_fire_streak_counts_casts_not_targets_and_combustion_caps_at_three() {
        let mut r = run(8, &[(13, 5), (14, 5), (15, 1)]);
        let mut en = enemy();
        learn(&mut r, "pyroblast", 10.0);
        let base_crit = cast(&r, "fireball").1.crit;
        let src = source("fireball", 1);
        deal(&mut r, &mut en, src, true);
        deal(&mut r, &mut en, src, true);
        near(r.abilities[0].cd, 10.0);
        assert_eq!(r.class_effects.fire_streak, 1);
        assert_eq!(r.class_effects.fire, 1);
        deal(&mut r, &mut en, source("fire_blast", 2), false);
        assert_eq!(r.class_effects.fire_streak, 0);
        deal(&mut r, &mut en, source("fireball", 3), true);
        deal(&mut r, &mut en, source("blast_wave", 4), true);
        near(r.abilities[0].cd, 0.0);
        near(cast(&r, "fireball").1.crit, base_crit + 0.15);
        r.abilities[0].cd = 10.0;
        deal(&mut r, &mut en, source("fireball", 5), true);
        deal(&mut r, &mut en, source("fireball", 6), true);
        near(r.abilities[0].cd, 10.0);
        r.elapsed = 8.0;
        deal(&mut r, &mut en, source("fireball", 7), true);
        deal(&mut r, &mut en, source("fireball", 8), true);
        near(r.abilities[0].cd, 0.0);
        let count = en.dots.len();
        deal(&mut r, &mut en, source("flamestrike", 9), true);
        assert_eq!(en.dots.len(), count);
        let ignite = en.dots[0].source;
        deal(&mut r, &mut en, ignite, true);
        assert_eq!(en.dots.len(), count);
        near(en.dots[0].dps * 4.0, 40.0);
        r.elapsed = 14.0;
        near(cast(&r, "fireball").1.crit, base_crit);
    }

    #[test]
    fn mage_frozen_charges_survive_inflight_and_shatter_never_double_crits() {
        let mut r = run(8, &[(21, 5), (22, 5), (23, 1)]);
        let mut en = enemy();
        let frost_hit = source("frostbolt", 1);
        deal(&mut r, &mut en, frost_hit, false);
        assert!(frozen_cast(&r, "frostbolt"));
        assert!(!frozen_cast(&r, "blizzard"));
        let snapshot = Source {
            talent_frozen: frozen_cast(&r, "frostbolt"),
            ..source("frostbolt", 2)
        };
        on_cast(&mut r, "frostbolt", 2);
        assert!(!frozen_cast(&r, "frostbolt"));
        let stats = r.stats();
        let (damage, critical) = adjust_critical(&r, snapshot, &en, 100.0, false, &mut Rng(0));
        assert!(critical);
        near(damage, 100.0 * stats.crit_multiplier());
        let (damage, critical) = adjust_critical(&r, snapshot, &en, 100.0, true, &mut Rng(0));
        assert!(critical);
        near(damage, 100.0);
        let (damage, critical) = adjust_critical(&r, frost_hit, &en, 100.0, false, &mut Rng(0));
        assert!(!critical);
        near(damage, 100.0);
        en.root_t = 1.0;
        assert!(adjust_critical(&r, frost_hit, &en, 100.0, false, &mut Rng(0)).1);
        assert!(!adjust_critical(&r, source("blizzard", 4), &en, 100.0, false, &mut Rng(0)).1);
        deal(&mut r, &mut en, source("frost_nova", 5), false);
        near(r.shield, r.stats().max_hp * 0.10);
        r.shield = 0.0;
        r.elapsed = 5.999;
        deal(&mut r, &mut en, source("frost_nova", 6), false);
        near(r.shield, 0.0);
        r.elapsed = 6.0;
        deal(&mut r, &mut en, source("frost_nova", 7), false);
        near(r.shield, r.stats().max_hp * 0.10);
    }

    #[test]
    fn druid_natures_grace_swaps_charges_and_thorns_buff_is_dynamic() {
        let mut r = run(11, &[(5, 5), (13, 5), (14, 5), (15, 1)]);
        let mut en = enemy();
        let wrath = cast(&r, "wrath").0.dmg;
        let starfire = cast(&r, "starfire").0.dmg;
        deal(&mut r, &mut en, source("wrath", 1), true);
        near(cast(&r, "starfire").0.dmg, starfire * 1.25);
        near(cast(&r, "wrath").0.dmg, wrath);
        on_cast(&mut r, "starfire", 2);
        near(cast(&r, "starfire").0.dmg, starfire);
        deal(&mut r, &mut en, source("starfire", 2), true);
        near(cast(&r, "wrath").0.dmg, wrath * 1.25);
        en.boss = true;
        deal(&mut r, &mut en, source("thorns", 3), false);
        near(en.slow, 0.90);
        near(en.slow_t, 2.0);
        en.root_t = 1.0;
        near(damage_multiplier(&r, source("wrath", 4), &en), 1.20);
        near(damage_multiplier(&r, source("hurricane", 4), &en), 1.0);
        r.thorns = 10.0;
        on_mobility(&mut r);
        near(thorns_damage(&r), 13.0);
        near(incoming(&r), 0.80);
        r.elapsed = 4.0;
        near(thorns_damage(&r), 10.0);
        near(incoming(&r), 1.0);
    }

    #[test]
    fn druid_rejuvenation_recast_does_not_finish_and_natural_end_uses_effective_heal_only() {
        let mut r = run(11, &[(21, 5), (22, 5)]);
        r.heal_t = 6.0;
        on_cast(&mut r, "rejuvenation", 1);
        near(incoming(&r), 0.90);
        tick_healing(&mut r, 1.0, 10.0, true);
        r.heal_t = 6.0;
        on_cast(&mut r, "rejuvenation", 2);
        near(r.shield, 0.0);
        r.heal_t = 5.0;
        tick_healing(&mut r, 1.0, 5.0, true);
        r.heal_t = 0.0;
        tick_healing(&mut r, 10.0, 5.0, true);
        near(r.shield, 2.0);
        near(r.shield_t, 8.0);
        near(incoming(&r), 1.0);
        let old = r.shield;
        tick_healing(&mut r, 10.0, 100.0, false);
        near(r.shield, old);
        r.heal_t = 1.0;
        on_cast(&mut r, "rejuvenation", 3);
        r.shield = r.stats().max_hp;
        r.shield_t = 10.0;
        r.heal_t = 0.0;
        tick_healing(&mut r, 1.0, 1000.0, true);
        near(r.shield, r.stats().max_hp);
        near(r.shield_t, 10.0);
    }

    #[test]
    fn druid_rejuvenation_extensions_have_one_second_icd_and_fifty_percent_lifetime_cap() {
        let mut r = run(11, &[(23, 1)]);
        let mut en = enemy();
        r.heal_t = 5.0;
        on_cast(&mut r, "rejuvenation", 1);
        for batch in 0..10 {
            r.elapsed = batch as f32;
            deal(&mut r, &mut en, source("wrath", batch), false);
            let before = r.heal_t;
            deal(&mut r, &mut en, source("starfire", batch + 20), false);
            near(r.heal_t, before);
        }
        near(r.heal_t, 7.5);
        near(r.class_effects.rejuvenation_extension, 0.0);
        r.heal_t = 0.0;
        tick_healing(&mut r, 1.0, 0.0, true);
        deal(&mut r, &mut en, source("wrath", 50), false);
        near(r.heal_t, 0.0);
    }

    #[test]
    fn druid_natural_end_shield_caps_at_ten_percent_and_overheal_gives_nothing() {
        let mut r = run(11, &[(22, 5)]);
        r.heal_t = 1.0;
        on_cast(&mut r, "rejuvenation", 1);
        r.heal_t = 0.0;
        tick_healing(&mut r, 1.0, 1000.0, true);
        near(r.shield, r.stats().max_hp * 0.10);
        r.shield = 0.0;
        r.shield_t = 0.0;
        r.heal_t = 1.0;
        on_cast(&mut r, "rejuvenation", 2);
        r.heal_t = 0.0;
        tick_healing(&mut r, 1.0, 0.0, true);
        near(r.shield, 0.0);
        near(r.shield_t, 0.0);
    }

    #[test]
    fn druid_moonfire_cooldown_proc_deduplicates_batch_pulse_but_accepts_later_ticks() {
        let mut r = run(11, &[(6, 5), (7, 1)]);
        let mut en = enemy();
        learn(&mut r, "starfire", 5.0);
        let tick = Source {
            periodic: true,
            pulse: 1,
            ..source("moonfire", 1)
        };
        deal(&mut r, &mut en, tick, false);
        near(r.abilities[0].cd, 4.0);
        r.elapsed = 1.0;
        deal(&mut r, &mut en, tick, false);
        near(r.abilities[0].cd, 4.0);
        deal(&mut r, &mut en, Source { pulse: 2, ..tick }, false);
        near(r.abilities[0].cd, 3.0);
        assert!(periodic_crits(&r, tick));
        assert!(periodic_crits(&r, source("insect_swarm", 2)));
    }

    #[test]
    fn all_talent_healing_uses_shared_budget_and_never_revives_or_uses_overkill() {
        for (class, slot, key, periodic) in [
            (1, 14, "cleave", false),
            (2, 5, "holy_shock", false),
            (3, 6, "arcane_shot", false),
            (4, 7, "rupture", true),
            (5, 22, "mind_blast", false),
            (7, 22, "searing_totem", false),
        ] {
            let mut r = run(class, &[(slot, 5)]);
            let mut en = enemy();
            let budget = r.stats().max_hp * 0.04;
            r.hp = 1.0;
            r.talent_leech_budget = budget;
            let src = Source {
                periodic,
                ..source(key, 1)
            };
            for _ in 0..100 {
                deal(&mut r, &mut en, src, true);
            }
            near(r.hp, 1.0 + budget);
            near(r.talent_leech_budget, 0.0);
            r.talent_leech_budget = budget;
            let before = r.hp;
            on_damage(
                &mut r,
                src,
                true,
                Entity::PLACEHOLDER,
                &mut en,
                0.0,
                &mut Rng(0),
            );
            near(r.hp, before);
            let secondary = Source {
                secondary: true,
                ..src
            };
            deal(&mut r, &mut en, secondary, true);
            near(r.hp, before);
            r.hp = r.stats().max_hp - 0.1;
            deal(&mut r, &mut en, src, true);
            near(r.hp, r.stats().max_hp);
            near(r.talent_leech_budget, budget - 0.1);
            r.hp = 0.0;
            deal(&mut r, &mut en, src, true);
            near(r.hp, 0.0);
            r.hp = 1.0;
            r.dead_t = Some(0.0);
            deal(&mut r, &mut en, src, true);
            near(r.hp, 1.0);
        }
    }

    #[test]
    fn dead_targets_and_secondary_damage_cannot_create_procs() {
        for (class, slot, key) in [
            (1, 5, "heroic_strike"),
            (2, 21, "judgement"),
            (3, 15, "multi_shot"),
            (4, 14, "eviscerate"),
            (5, 21, "mind_blast"),
            (7, 7, "chain_lightning"),
            (8, 13, "fireball"),
            (11, 5, "wrath"),
        ] {
            let mut r = run(class, &[(slot, 5)]);
            let mut en = enemy();
            en.dead = true;
            assert!(deal(&mut r, &mut en, source(key, 1), true).is_none());
            en.dead = false;
            assert!(deal(
                &mut r,
                &mut en,
                Source {
                    secondary: true,
                    ..source(key, 2)
                },
                true
            )
            .is_none());
            assert!(en.dots.is_empty());
            near(en.class_effects.mark_until, 0.0);
            assert!(buffs(&r).is_empty());
        }
    }
}
