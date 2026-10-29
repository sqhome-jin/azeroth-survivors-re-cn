//! Auto-cast: every owned ability fires on its own cooldown at the targets its kind picks, with
//! the spell's own visuals; damage lands when its missile would.

use bevy::prelude::*;

use super::data::{AbilityDef, Effect, Kind};
use super::enemies::{self, Enemy, Loot};
use super::hero::{self, Hero};
use super::skill_effects::{self, Source};
use super::units::{self, flat, Fx, Marks, UnitSpec, COLOR_HEAL, COLOR_MELEE, COLOR_SPELL};
use super::{class_mechanics, warlock, Rng, Run, RunEntity, Stats};
use crate::net::{FieldChanged, Guid, ObjectStore};

/// One blow landing on one enemy.
#[derive(Message, Clone)]
pub(super) struct Hit {
    pub target: Entity,
    pub damage: f32,
    pub dot: f32,
    pub dot_dur: f32,
    pub effect: Effect,
    pub mark: u32,
    pub mark_dur: f32,
    /// Also hits everything this close to the target, for less.
    pub splash: f32,
    pub crit: bool,
    pub color: u32,
    pub source: Source,
    pub interval: f32,
    pub splash_ratio: f32,
}

impl Hit {
    pub fn direct(target: Entity, damage: f32, crit: bool) -> Self {
        Self {
            target,
            damage,
            dot: 0.0,
            dot_dur: 0.0,
            effect: Effect::None,
            mark: 0,
            mark_dur: 0.0,
            splash: 0.0,
            crit,
            color: COLOR_MELEE,
            source: Source::default(),
            interval: 1.0,
            splash_ratio: 0.7,
        }
    }
}

/// Hits in flight, by the run time they land.
#[derive(Resource, Default)]
pub(super) struct PendingHits(pub Vec<(f32, Hit)>);

/// A ground spell's area, ticking on whatever stands in it.
#[derive(Component)]
pub(super) struct GroundArea {
    damage: f32,
    dot: f32,
    dot_dur: f32,
    radius: f32,
    left: f64,
    every: f64,
    next_tick: f64,
    effect: Effect,
    mark: u32,
    /// The spell whose impact plays on each enemy a tick hits, 0 for none.
    impact: u32,
    source: Source,
}

#[derive(Component)]
pub(super) struct Totem {
    cast: u32,
    damage: f32,
    range: f32,
    radius: f32,
    left: f64,
    tick: f64,
    pulse: bool,
    every: f64,
    source: Source,
}

#[derive(Clone, Copy)]
struct Foe {
    e: Entity,
    pos: Vec3,
    hp: f32,
    dist: f32,
    radius: f32,
}

/// The numbers one cast uses, from the ability's level and the hero's stats.
struct Params {
    dmg: f32,
    dot: f32,
    count: usize,
    radius: f32,
    duration: f32,
    cooldown: f32,
    range: f32,
    source: Source,
}

fn mark_secs(def: &AbilityDef, duration: f32) -> f32 {
    match def.effect {
        Effect::Slow(_, t) | Effect::Weaken(_, t) => t,
        Effect::Root(t) | Effect::Stun(t) | Effect::Fear(t) => t,
        _ if duration > 0.0 => duration,
        _ => 3.0,
    }
}

fn hit_for(def: &AbilityDef, p: &Params, target: Entity, damage: f32, crit: bool) -> Hit {
    Hit {
        target,
        damage,
        dot: p.dot,
        dot_dur: p.duration,
        effect: p.source.modifiers.effect(def.effect),
        mark: def.mark,
        mark_dur: mark_secs(def, p.duration) * (1.0 + p.source.modifiers.control),
        splash: if matches!(def.kind, Kind::Bolt | Kind::Strike) {
            if p.source.modifiers.splash > 0.0 {
                3.0
            } else {
                p.radius
            }
        } else {
            0.0
        },
        crit,
        color: if def.kind == Kind::Melee {
            COLOR_MELEE
        } else {
            COLOR_SPELL
        },
        source: p.source,
        interval: p.source.modifiers.interval(1.0),
        splash_ratio: if p.source.modifiers.splash > 0.0 {
            p.source.modifiers.splash
        } else {
            0.7
        },
    }
}

fn schedule(
    pending: &mut PendingHits,
    hits: &mut MessageWriter<Hit>,
    now: f32,
    delay: f32,
    hit: Hit,
) {
    if hit.source.echo && !hit.source.secondary {
        let mut echo = hit.clone();
        echo.damage *= 0.5;
        echo.dot *= 0.5;
        echo.source.secondary = true;
        echo.source.echo = false;
        echo.splash = 0.0;
        pending.0.push((now + delay + 0.25, echo));
    }
    if delay <= 0.0 {
        hits.write(hit);
    } else {
        pending.0.push((now + delay, hit));
    }
}

fn roll(rng: &mut Rng, stats: &Stats, dmg: f32) -> (f32, bool) {
    if rng.f32() < stats.crit {
        (dmg * stats.crit_multiplier(), true)
    } else {
        (dmg, false)
    }
}

fn nearest(foes: &[Foe], range: f32, n: usize) -> Vec<Foe> {
    foes.iter()
        .filter(|f| f.dist <= range + f.radius)
        .take(n)
        .copied()
        .collect()
}

fn toughest(foes: &[Foe], range: f32, n: usize) -> Vec<Foe> {
    let mut v: Vec<Foe> = foes
        .iter()
        .filter(|f| f.dist <= range + f.radius)
        .copied()
        .collect();
    v.sort_by(|a, b| b.hp.total_cmp(&a.hp));
    v.truncate(n);
    v
}

fn around(foes: &[Foe], center: Vec3, r: f32) -> Vec<Foe> {
    foes.iter()
        .filter(|f| flat(f.pos - center).length() <= r + f.radius * 0.5)
        .copied()
        .collect()
}

/// The point among the nearest enemies in range with the most others within `r` of it.
fn densest(foes: &[Foe], range: f32, r: f32) -> Option<Vec3> {
    let near: Vec<&Foe> = foes.iter().filter(|f| f.dist <= range).take(40).collect();
    near.iter()
        .map(|f| {
            (
                f.pos,
                near.iter()
                    .filter(|o| flat(o.pos - f.pos).length() <= r)
                    .count(),
            )
        })
        .max_by_key(|&(_, n)| n)
        .map(|(p, _)| p)
}

fn entities(foes: &[Foe]) -> Vec<Entity> {
    foes.iter().map(|f| f.e).collect()
}

pub(super) fn cast_abilities(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    spells: Option<Res<crate::ui_action::Spells>>,
    hero: Query<(Entity, &Transform), With<Hero>>,
    enemies: Query<(Entity, &Transform, &Enemy)>,
    mut fx: Fx,
    mut pending: ResMut<PendingHits>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    if run.dead_t.is_some() || dt <= 0.0 {
        return;
    }
    let Ok((me, htf)) = hero.single() else { return };
    let hpos = htf.translation;
    let now = run.elapsed;
    let mut foes: Vec<Foe> = enemies
        .iter()
        .filter(|(_, _, e)| !e.dead)
        .map(|(e, tf, en)| Foe {
            e,
            pos: tf.translation,
            hp: en.hp,
            dist: hero::reach(hpos, tf.translation),
            radius: en.radius,
        })
        .collect();
    foes.sort_by(|a, b| a.dist.total_cmp(&b.dist));
    let speed_of = |spell: u32| {
        spells
            .as_deref()
            .and_then(|s| s.catalog.get(spell))
            .map_or(0.0, |d| d.speed)
    };

    for i in 0..run.abilities.len() {
        run.abilities[i].cd -= dt;
        if run.abilities[i].cd > 0.0 {
            continue;
        }
        let o = &run.abilities[i];
        let def = o.def;
        let mut stats = run.stats();
        let mut cast = super::talents::cast(def, o.level, &stats);
        class_mechanics::modify_cast(&run, def.key, &mut cast, &mut stats);
        let source = Source {
            ability: Some(def.key),
            batch: run.guid(),
            modifiers: cast.modifiers,
            echo: cast.modifiers.echo > 0.0 && rng.f32() < cast.modifiers.echo,
            talent_frozen: class_mechanics::frozen_cast(&run, def.key),
            ..default()
        };
        let p = Params {
            dmg: cast.dmg,
            dot: cast.dot,
            count: cast.count as usize,
            radius: cast.radius,
            duration: cast.duration,
            cooldown: cast.cooldown,
            range: cast.range,
            source,
        };
        let speed = speed_of(def.spell);
        let fired = match def.kind {
            Kind::Bolt | Kind::Strike | Kind::Melee => {
                let reach = if def.kind == Kind::Melee {
                    p.range + 0.5
                } else {
                    p.range
                };
                let targets = if matches!(
                    def.key,
                    "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
                ) {
                    toughest(&foes, reach, p.count)
                } else {
                    nearest(&foes, reach, p.count)
                };
                if targets.is_empty() {
                    false
                } else {
                    if def.kind == Kind::Melee {
                        fx.swing(me, targets[0].e, p.dmg);
                        for f in &targets {
                            fx.impact(f.e, def.spell);
                        }
                    } else {
                        fx.cast(me, def.spell, &entities(&targets), None);
                    }
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, &stats, p.dmg);
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            }
            Kind::Nova | Kind::Cone => {
                let mut targets = around(&foes, hpos, p.radius);
                if def.kind == Kind::Cone {
                    if let Some(first) = targets.first().copied() {
                        let aim = flat(first.pos - hpos).normalize_or_zero();
                        targets.retain(|f| {
                            let d = flat(f.pos - hpos);
                            d.length() < 2.0 || d.normalize_or_zero().dot(aim) >= 0.8
                        });
                    }
                }
                if targets.is_empty() {
                    false
                } else {
                    fx.cast(me, def.spell, &entities(&targets), None);
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, &stats, p.dmg);
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            }
            Kind::Ground | Kind::GroundSelf => {
                let center = if def.kind == Kind::GroundSelf {
                    foes.first()
                        .filter(|f| f.dist <= p.radius + 4.0)
                        .map(|_| hpos)
                } else {
                    densest(&foes, p.range, p.radius)
                };
                match center {
                    None => false,
                    Some(center) => {
                        fx.cast(me, def.spell, &[], Some(center));
                        let area = if def.overhead {
                            commands
                                .spawn((Transform::from_translation(center), RunEntity))
                                .id()
                        } else {
                            let serial = run.guid();
                            units::spawn_area_object(
                                &mut commands,
                                serial,
                                def.spell,
                                p.radius,
                                center,
                            )
                        };
                        let burst = def.duration <= 0.0;
                        commands.entity(area).insert(GroundArea {
                            damage: p.dmg,
                            dot: p.dot,
                            dot_dur: if burst { 5.0 } else { 0.0 },
                            radius: p.radius,
                            left: if burst { 1.5 } else { f64::from(p.duration) },
                            every: if burst {
                                f64::INFINITY
                            } else {
                                f64::from(p.source.modifiers.interval(1.0))
                            },
                            next_tick: if burst { 0.0 } else { 0.4 },
                            effect: p.source.modifiers.effect(def.effect),
                            mark: def.mark,
                            impact: if def.overhead { def.spell } else { 0 },
                            source,
                        });
                        true
                    }
                }
            }
            Kind::Chain => match nearest(&foes, p.range, 1).first().copied() {
                None => false,
                Some(first) => {
                    let mut chain = vec![first];
                    while chain.len() < p.count + 1 {
                        let last = chain[chain.len() - 1].pos;
                        let next = foes
                            .iter()
                            .filter(|f| !chain.iter().any(|c| c.e == f.e))
                            .filter(|f| flat(f.pos - last).length() <= p.radius)
                            .min_by(|a, b| {
                                flat(a.pos - last)
                                    .length()
                                    .total_cmp(&flat(b.pos - last).length())
                            });
                        match next {
                            Some(f) => chain.push(*f),
                            None => break,
                        }
                    }
                    fx.cast(me, def.spell, &entities(&chain), None);
                    for (k, f) in chain.iter().enumerate() {
                        let (dmg, crit) = roll(
                            &mut rng,
                            &stats,
                            p.dmg * (0.85 + p.source.modifiers.chain).min(1.0).powi(k as i32),
                        );
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            0.08 * k as f32,
                            hit_for(def, &p, f.e, dmg, crit),
                        );
                    }
                    true
                }
            },
            Kind::Dot => {
                let mut pool: Vec<Foe> = foes
                    .iter()
                    .filter(|f| f.dist <= p.range)
                    .take(p.count * 3)
                    .copied()
                    .collect();
                if pool.is_empty() {
                    false
                } else {
                    let mut targets = Vec::with_capacity(p.count);
                    while targets.len() < p.count && !pool.is_empty() {
                        targets.push(pool.swap_remove(rng.index(pool.len())));
                    }
                    fx.cast(me, def.spell, &entities(&targets), None);
                    for f in &targets {
                        let delay = if speed > 0.0 { f.dist / speed } else { 0.0 };
                        schedule(
                            &mut pending,
                            &mut hits,
                            now,
                            delay,
                            hit_for(def, &p, f.e, 0.0, false),
                        );
                    }
                    true
                }
            }
            Kind::Aura => {
                let targets = around(&foes, hpos, p.radius);
                if targets.is_empty() {
                    false
                } else {
                    for f in targets.iter().take(6) {
                        fx.impact(f.e, def.spell);
                    }
                    for f in &targets {
                        let (dmg, crit) = roll(&mut rng, &stats, p.dmg);
                        hits.write(hit_for(def, &p, f.e, dmg, crit));
                    }
                    true
                }
            }
            Kind::Orbit => {
                let targets = nearest(&foes, p.radius, p.count);
                if targets.is_empty() {
                    false
                } else {
                    for f in &targets {
                        fx.impact(f.e, 403);
                        let (dmg, crit) = roll(&mut rng, &stats, p.dmg);
                        hits.write(hit_for(def, &p, f.e, dmg, crit));
                    }
                    true
                }
            }
            Kind::Shield => {
                run.shield = p.dmg;
                run.shield_t = p.duration;
                fx.cast(me, def.spell, &[me], None);
                true
            }
            Kind::Immune => {
                if foes.first().is_some_and(|f| f.dist <= 5.0) {
                    run.immune_t = p.duration;
                    fx.cast(me, def.spell, &[me], None);
                    true
                } else {
                    false
                }
            }
            Kind::Thorns => {
                if foes.first().is_some_and(|f| f.dist <= 8.0) {
                    run.thorns = p.dmg;
                    run.thorns_t = p.duration;
                    run.thorns_spell = def.spell;
                    run.thorns_source = source;
                    fx.cast(me, def.spell, &[me], None);
                    true
                } else {
                    false
                }
            }
            Kind::Heal => {
                if run.hp < stats.max_hp * 0.85 {
                    run.heal_rate = p.dmg / p.duration.max(1.0);
                    run.heal_t = p.duration;
                    fx.cast(me, def.spell, &[me], None);
                    fx.number(me, p.dmg, false, COLOR_HEAL);
                    true
                } else {
                    false
                }
            }
            Kind::Totem => {
                if foes
                    .first()
                    .is_some_and(|f| f.dist <= p.range.max(p.radius) + 6.0)
                {
                    fx.cast(me, def.spell, &[], None);
                    let a = rng.range(0.0, std::f32::consts::TAU);
                    let pos = hpos + Vec3::new(a.cos(), 0.0, a.sin()) * 2.0;
                    let serial = run.guid();
                    let totem = units::spawn_unit(
                        &mut commands,
                        &UnitSpec {
                            serial,
                            display: def.totem.0,
                            scale: 1.0,
                            level: 60,
                            faction: units::FACTION_FRIENDLY,
                            bytes0: 0,
                            weapons: &[],
                            sheath: 0,
                        },
                        pos,
                        0.0,
                    );
                    commands.entity(totem).insert(Totem {
                        cast: def.totem.1,
                        damage: p.dmg,
                        range: p.range,
                        radius: p.radius,
                        left: f64::from(p.duration),
                        tick: 0.3,
                        pulse: def.key == "magma_totem",
                        every: f64::from(
                            p.source.modifiers.interval(if def.key == "magma_totem" {
                                2.0
                            } else {
                                1.0
                            }),
                        ),
                        source,
                    });
                    true
                } else {
                    false
                }
            }
        };
        if fired {
            class_mechanics::on_cast(&mut run, def.key, source.batch);
        }
        run.abilities[i].cd = if fired { p.cooldown } else { 0.2 };
    }
}

pub(super) fn update_talent_environment(
    mut run: ResMut<Run>,
    hero: Query<&Transform, With<Hero>>,
    areas: Query<(&Transform, &GroundArea)>,
    totems: Query<(&Transform, &Totem)>,
) {
    let Ok(hero) = hero.single() else { return };
    let consecration = areas.iter().any(|(at, a)| {
        a.left > 0.0
            && a.source.ability == Some("consecration")
            && hero::reach(hero.translation, at.translation) <= a.radius
    });
    let totem = totems
        .iter()
        .any(|(at, t)| t.left > 0.0 && hero::reach(hero.translation, at.translation) <= 10.0);
    class_mechanics::set_environment(&mut run, consecration, totem);
}

/// Ground areas tick, and totems fight, until they run out.
#[allow(clippy::type_complexity)]
pub(super) fn tick_areas(
    mut commands: Commands,
    time: Res<Time>,
    run: Res<Run>,
    mut rng: ResMut<Rng>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut areas: Query<(Entity, &Transform, &mut GroundArea)>,
    mut totems: Query<(Entity, &Transform, &mut Totem), Without<GroundArea>>,
    enemies: Query<(Entity, &Transform, &Enemy), (Without<GroundArea>, Without<Totem>)>,
    mut fx: Fx,
    mut pending: ResMut<PendingHits>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let stats = run.stats();
    let now = run.elapsed;
    let live: Vec<Foe> = enemies
        .iter()
        .filter(|(_, _, e)| !e.dead)
        .map(|(e, tf, en)| Foe {
            e,
            pos: tf.translation,
            hp: en.hp,
            dist: 0.0,
            radius: en.radius,
        })
        .collect();

    for (e, tf, mut area) in &mut areas {
        let a = &mut *area;
        let count = skill_effects::ticks(&mut a.left, &mut a.next_tick, a.every, dt);
        for _ in 0..count {
            area.source.pulse += 1;
            for (i, f) in around(&live, tf.translation, area.radius)
                .into_iter()
                .enumerate()
            {
                if area.impact != 0 && i < 12 {
                    fx.impact(f.e, area.impact);
                }
                let (dmg, crit) = roll(&mut rng, &stats, area.damage);
                hits.write(Hit {
                    target: f.e,
                    damage: dmg,
                    dot: area.dot,
                    dot_dur: area.dot_dur,
                    effect: area.effect,
                    mark: area.mark,
                    mark_dur: 1.5,
                    splash: 0.0,
                    crit,
                    color: COLOR_SPELL,
                    source: area.source,
                    interval: area.source.modifiers.interval(1.0),
                    splash_ratio: 0.7,
                });
            }
        }
        if area.left <= 0.0 {
            commands.entity(e).despawn();
        }
    }

    for (e, tf, mut totem) in &mut totems {
        let t = &mut *totem;
        let count = skill_effects::ticks(&mut t.left, &mut t.tick, t.every, dt);
        for _ in 0..count {
            totem.source.pulse += 1;
            let pos = tf.translation;
            if totem.pulse {
                let targets = around(&live, pos, totem.radius);
                if targets.is_empty() {
                    continue;
                }
                fx.cast(e, totem.cast, &[], None);
                for f in &targets {
                    let (dmg, crit) = roll(&mut rng, &stats, totem.damage);
                    hits.write(Hit {
                        color: COLOR_SPELL,
                        source: totem.source,
                        ..Hit::direct(f.e, dmg, crit)
                    });
                }
            } else {
                let target = live
                    .iter()
                    .map(|f| (flat(f.pos - pos).length(), f))
                    .filter(|(d, _)| *d <= totem.range)
                    .min_by(|a, b| a.0.total_cmp(&b.0));
                let Some((dist, f)) = target else { continue };
                fx.cast(e, totem.cast, &[f.e], None);
                let speed = spells
                    .as_deref()
                    .and_then(|s| s.catalog.get(totem.cast))
                    .map_or(0.0, |d| d.speed);
                let delay = if speed > 0.0 { dist / speed } else { 0.0 };
                let (dmg, crit) = roll(&mut rng, &stats, totem.damage);
                schedule(
                    &mut pending,
                    &mut hits,
                    now,
                    delay,
                    Hit {
                        color: COLOR_SPELL,
                        source: totem.source,
                        ..Hit::direct(f.e, dmg, crit)
                    },
                );
            }
        }
        if totem.left <= 0.0 {
            commands
                .entity(e)
                .remove::<Totem>()
                .insert(benilla_world::model_fade::DespawnFade::default());
        }
    }
}

/// Land every hit due: damage, periodic effects, crowd control and death with loot.
pub(super) fn apply_hits(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    mut pending: ResMut<PendingHits>,
    mut incoming: MessageReader<Hit>,
    mut enemies: Query<(
        Entity,
        &Guid,
        &Transform,
        &mut Enemy,
        &mut ObjectStore,
        &mut Marks,
    )>,
    mut fx: Fx,
    mut loot: MessageWriter<Loot>,
    mut fields: MessageWriter<FieldChanged>,
) {
    let dt = time.delta_secs();
    let now = run.elapsed;
    super::talents::periodic_heal(&mut run, 0.0, dt);
    let mut due: Vec<Hit> = incoming.read().cloned().collect();
    pending.0.retain(|(t, h)| {
        if *t <= now {
            due.push(h.clone());
            false
        } else {
            true
        }
    });

    // Splash never hits its original target a second time.
    let mut all = Vec::with_capacity(due.len());
    for h in due {
        if h.splash > 0.0 {
            if let Ok((_, _, center, ..)) = enemies.get(h.target) {
                let c = center.translation;
                for (e, _, tf, en, ..) in enemies.iter() {
                    if e != h.target && !en.dead && flat(tf.translation - c).length() <= h.splash {
                        all.push(Hit {
                            target: e,
                            damage: h.damage * h.splash_ratio,
                            splash: 0.0,
                            mark: 0,
                            source: Source {
                                secondary: true,
                                ..h.source
                            },
                            ..h.clone()
                        });
                    }
                }
            }
        }
        all.push(Hit { splash: 0.0, ..h });
    }

    let mut kill =
        |commands: &mut Commands, run: &mut Run, e: Entity, pos: Vec3, en: &mut Enemy| {
            en.dead = true;
            en.hp = 0.0;
            en.dots.clear();
            enemies::mark_dying(commands, e);
            loot.write(Loot {
                pos,
                xp: en.xp,
                boss: en.boss,
            });
            run.kills += 1;
            if en.boss {
                run.boss_kills += 1;
                run.banner = Some((format!("{} is slain!", en.def.name), 3.0));
            }
        };

    for h in all {
        let Ok((e, guid, tf, mut en, mut store, mut marks)) = enemies.get_mut(h.target) else {
            continue;
        };
        if en.dead {
            continue;
        }
        let (damage, critical) =
            class_mechanics::adjust_critical(&run, h.source, &en, h.damage, h.crit, &mut rng);
        let mut dmg = damage
            * warlock::damage_multiplier(&run, h.source, &en)
            * class_mechanics::damage_multiplier(&run, h.source, &en);
        if !h.source.secondary && en.hp / en.max_hp < 0.3 {
            dmg *= 1.0 + h.source.modifiers.execute;
        }
        if let Effect::Execute(threshold) = h.effect {
            if !en.boss && en.hp / en.max_hp <= threshold {
                dmg = dmg.max(en.hp);
            }
        }
        if dmg > 0.0 {
            let effective = dmg.min(en.hp.max(0.0));
            en.hp -= dmg;
            run.damage_done += f64::from(dmg);
            fx.number(e, dmg, critical, h.color);
            if let Effect::Drain(frac) = h.effect {
                let frac = class_mechanics::drain_fraction(&run, h.source.ability, frac);
                let heal = class_mechanics::cap_drain(&mut run, h.source, effective * frac);
                let max = run.stats().max_hp;
                if run.dead_t.is_none() && run.hp > 0.0 {
                    run.hp = (run.hp + heal).min(max);
                }
            }
            skill_effects::damage_effects(&mut run, h.source, effective, &mut rng);
            warlock::on_damage(&mut run, h.source, critical, &mut en, &mut marks, &mut rng);
            if let Some(extra) = class_mechanics::on_damage(
                &mut run, h.source, critical, e, &mut en, effective, &mut rng,
            ) {
                pending.0.push((now, extra));
            }
            if !h.source.secondary && h.source.modifiers.ember > 0.0 && en.hp > 0.0 {
                if en.dots.len() >= 6 {
                    en.dots.remove(0);
                }
                en.dots.push(enemies::Dot {
                    dps: dmg * h.source.modifiers.ember / 3.0,
                    left: 3.0,
                    tick: 1.0,
                    every: 1.0,
                    extension_left: 0.0,
                    source: Source {
                        secondary: true,
                        ..h.source
                    },
                });
            }
        }
        if h.dot > 0.0 && h.dot_dur > 0.0 {
            if en.dots.len() >= 6 {
                en.dots.remove(0);
            }
            en.dots.push(enemies::Dot {
                dps: h.dot,
                left: f64::from(h.dot_dur),
                tick: f64::from(h.interval),
                every: f64::from(h.interval),
                extension_left: f64::from(h.dot_dur) * 0.5,
                source: h.source,
            });
        }
        let boss = en.boss;
        match h.effect {
            Effect::Slow(m, t) => {
                let m = if boss { m.max(0.75) } else { m };
                en.slow = if en.slow_t > 0.0 { en.slow.min(m) } else { m };
                en.slow_t = en.slow_t.max(t);
            }
            Effect::Root(t) => en.root_t = en.root_t.max(if boss { t * 0.3 } else { t }),
            Effect::Stun(t) => en.stun_t = en.stun_t.max(if boss { t * 0.3 } else { t }),
            Effect::Fear(t) if !boss => en.fear_t = en.fear_t.max(t),
            Effect::Weaken(m, t) => {
                en.weaken = if en.weaken_t > 0.0 {
                    en.weaken.min(m)
                } else {
                    m
                };
                en.weaken_t = en.weaken_t.max(t);
            }
            _ => {}
        }
        if h.mark != 0 {
            marks.add(h.mark, h.mark_dur);
        }
        if en.hp <= 0.0 {
            kill(&mut commands, &mut run, e, tf.translation, &mut en);
            units::kill_unit(&mut store);
            marks.clear();
        }
        marks.flush(e, guid.0, &mut store, &mut fields);
    }

    // Tick counts depend on the active time window, not the number of rendered frames.
    for (e, guid, tf, mut en, mut store, mut marks) in &mut enemies {
        if en.dead || en.dots.is_empty() {
            continue;
        }
        let max_hp = en.max_hp;
        let mut dealt = 0.0;
        let mut dots = std::mem::take(&mut en.dots);
        for d in &mut dots {
            let mut frame_left = f64::from(dt);
            while d.next_tick(&mut frame_left) {
                if en.hp <= 0.0 {
                    break;
                }
                d.source.pulse += 1;
                d.source.periodic = true;
                let base = d.dps
                    * warlock::damage_multiplier(&run, d.source, &en)
                    * class_mechanics::damage_multiplier(&run, d.source, &en)
                    * if !d.source.secondary && en.hp / max_hp < 0.3 {
                        1.0 + d.source.modifiers.execute
                    } else {
                        1.0
                    };
                let (damage, crit) = if (run.class.id == 9
                    && warlock::periodic_crits(run.talents, d.source))
                    || class_mechanics::periodic_crits(&run, d.source)
                {
                    roll(&mut rng, &run.stats(), base)
                } else {
                    (base, false)
                };
                let effective = damage.min(en.hp.max(0.0));
                en.hp -= damage;
                dealt += damage;
                fx.number(e, damage, crit, COLOR_SPELL);
                skill_effects::damage_effects(&mut run, d.source, effective, &mut rng);
                if effective > 0.0 {
                    warlock::on_damage(&mut run, d.source, crit, &mut en, &mut marks, &mut rng);
                    if let Some(extra) = class_mechanics::on_damage(
                        &mut run, d.source, crit, e, &mut en, effective, &mut rng,
                    ) {
                        pending.0.push((now, extra));
                    }
                    if run.class.id == 9 && run.dead_t.is_none() && run.talents.0[2] > 0 {
                        warlock::extend_dot(d, run.talents, rng.f32());
                    }
                }
            }
        }
        // Trigger-created effects must survive the detached periodic iteration.
        dots.append(&mut en.dots);
        en.dots = dots;
        en.dots.retain(|d| d.left > 0.0);
        for dot in &en.dots {
            if let Some(key) = dot.source.ability {
                let mark = super::data::ability(key).mark;
                if mark != 0 && !dot.source.secondary {
                    marks.add(mark, dot.left as f32 + dt);
                }
            }
        }
        if dealt > 0.0 {
            run.damage_done += f64::from(dealt);
            if en.hp <= 0.0 {
                kill(&mut commands, &mut run, e, tf.translation, &mut en);
                units::kill_unit(&mut store);
                marks.clear();
            }
        }
        marks.flush(e, guid.0, &mut store, &mut fields);
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        data,
        gear::{EnhKind, SkillRoll},
        Owned,
    };
    use super::*;
    use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
    use benilla_protocol::messages::ObjectFields;
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Run>()
            .insert_resource(Rng(12345))
            .init_resource::<Time>()
            .init_resource::<PendingHits>()
            .init_resource::<PlaySeq>()
            .add_message::<Hit>()
            .add_message::<Loot>()
            .add_message::<FieldChanged>()
            .add_message::<CastEvent>()
            .add_message::<SpellGoTargets>()
            .add_message::<SwingMessage>()
            .add_message::<crate::combat_text::CombatTextSpawn>();
        app
    }

    fn foe(app: &mut App, x: f32, hp: f32) -> Entity {
        app.world_mut()
            .spawn((
                Transform::from_xyz(x, 0.0, 0.0),
                Guid(123),
                ObjectStore(ObjectFields::from_pairs(&[])),
                Marks::debuffs(),
                Enemy {
                    def: &data::ENEMIES[0],
                    hp,
                    max_hp: hp,
                    speed: 0.0,
                    damage: 0.0,
                    radius: 0.0,
                    xp: 0,
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
                    warlock: warlock::Debuffs::default(),
                    class_effects: class_mechanics::Debuffs::default(),
                    dead: false,
                },
            ))
            .id()
    }

    fn step(app: &mut App, dt: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(dt));
        app.update();
        app.world_mut().resource_mut::<Run>().elapsed += dt;
    }

    fn enhance(app: &mut App, kind: EnhKind, ability: Option<&'static str>, value: f32) {
        let mut run = app.world_mut().resource_mut::<Run>();
        let at = run.gear.enh.iter().position(Option::is_none).unwrap();
        run.gear.enh[at] = Some(SkillRoll {
            item: at as u64 + 1,
            index: 0,
            ability,
            kind,
            value,
        });
    }

    fn warlock(app: &mut App) {
        app.world_mut().resource_mut::<Run>().class =
            data::CLASSES.iter().find(|c| c.id == 9).unwrap();
    }

    fn damage(app: &mut App, e: Entity, key: &'static str, amount: f32, crit: bool) {
        app.world_mut().write_message(Hit {
            source: Source {
                ability: Some(key),
                ..default()
            },
            ..Hit::direct(e, amount, crit)
        });
        step(app, 0.0);
    }

    fn add_dot(app: &mut App, e: Entity, key: &'static str, duration: f64) {
        app.world_mut()
            .get_mut::<Enemy>(e)
            .unwrap()
            .dots
            .push(enemies::Dot {
                dps: 100.0,
                left: duration,
                tick: 1.0,
                every: 1.0,
                extension_left: duration * 0.5,
                source: Source {
                    ability: Some(key),
                    batch: 1,
                    ..default()
                },
            });
    }

    #[test]
    fn new_class_periodic_crit_gates_use_the_real_hit_pipeline() {
        for (class, key, index) in [
            (3, "serpent_sting", 23),
            (4, "rupture", 7),
            (11, "moonfire", 7),
            (11, "insect_swarm", 7),
        ] {
            for enabled in [false, true] {
                let mut app = app();
                app.add_systems(Update, apply_hits);
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.class = data::CLASSES.iter().find(|c| c.id == class).unwrap();
                    run.talents.0[index] = u8::from(enabled);
                }
                let stats = app.world().resource::<Run>().stats();
                let mut rng = Rng(12345);
                let expected: f32 = (0..100)
                    .map(|_| {
                        if enabled {
                            roll(&mut rng, &stats, 100.0).0
                        } else {
                            100.0
                        }
                    })
                    .sum();
                let e = foe(&mut app, 1.0, 100_000.0);
                add_dot(&mut app, e, key, 100.0);
                step(&mut app, 100.0);
                assert_eq!(
                    app.world().get::<Enemy>(e).unwrap().hp,
                    100_000.0 - expected,
                    "class {class}, {key}, enabled {enabled}"
                );
            }
        }
    }

    #[test]
    fn paladin_healing_and_rogue_target_bonus_reach_real_damage_events() {
        let mut paladin = app();
        paladin.add_systems(Update, apply_hits);
        {
            let mut run = paladin.world_mut().resource_mut::<Run>();
            run.class = data::CLASSES.iter().find(|c| c.id == 2).unwrap();
            run.hp = 50.0;
            run.talents.0[5] = 5;
        }
        let e = foe(&mut paladin, 1.0, 100.0);
        paladin.world_mut().write_message(Hit {
            source: Source {
                ability: Some("holy_shock"),
                batch: 1,
                ..default()
            },
            ..Hit::direct(e, 1000.0, false)
        });
        step(&mut paladin, 1.0);
        assert!(
            (paladin.world().resource::<Run>().hp - 55.0).abs() < 0.001,
            "heals from 100 effective damage, not 1000 overkill"
        );

        let mut rogue = app();
        rogue.add_systems(Update, apply_hits);
        {
            let mut run = rogue.world_mut().resource_mut::<Run>();
            run.class = data::CLASSES.iter().find(|c| c.id == 4).unwrap();
            run.talents.0[5] = 5;
        }
        let e = foe(&mut rogue, 1.0, 1000.0);
        add_dot(&mut rogue, e, "rupture", 6.0);
        rogue.world_mut().write_message(Hit {
            source: Source {
                ability: Some("sinister_strike"),
                batch: 2,
                ..default()
            },
            ..Hit::direct(e, 100.0, false)
        });
        step(&mut rogue, 0.0);
        assert!((rogue.world().get::<Enemy>(e).unwrap().hp - 880.0).abs() < 0.001);
        rogue.world_mut().write_message(Hit {
            source: Source {
                ability: Some("sinister_strike"),
                batch: 3,
                secondary: true,
                ..default()
            },
            ..Hit::direct(e, 100.0, false)
        });
        step(&mut rogue, 0.0);
        assert!((rogue.world().get::<Enemy>(e).unwrap().hp - 780.0).abs() < 0.001);
    }

    #[test]
    fn environment_tracks_live_consecration_and_nearby_totems() {
        let mut app = app();
        app.add_systems(Update, update_talent_environment);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = data::CLASSES.iter().find(|c| c.id == 2).unwrap();
            run.talents.0[13] = 5;
        }
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        let area = app
            .world_mut()
            .spawn((
                Transform::from_xyz(3.0, 0.0, 0.0),
                GroundArea {
                    damage: 1.0,
                    dot: 0.0,
                    dot_dur: 0.0,
                    radius: 5.0,
                    left: 4.0,
                    every: 1.0,
                    next_tick: 1.0,
                    effect: Effect::None,
                    mark: 0,
                    impact: 0,
                    source: Source {
                        ability: Some("consecration"),
                        ..default()
                    },
                },
            ))
            .id();
        app.update();
        assert!((app.world().resource::<Run>().incoming() - 0.9).abs() < 0.001);
        app.world_mut().get_mut::<GroundArea>(area).unwrap().left = 0.0;
        app.update();
        assert_eq!(app.world().resource::<Run>().incoming(), 1.0);
    }

    #[test]
    fn warlock_periodic_crits_share_actual_direct_chance_and_multiplier() {
        for key in ["corruption", "curse_of_agony", "immolate"] {
            for enabled in [false, true] {
                let mut app = app();
                app.add_systems(Update, apply_hits);
                warlock(&mut app);
                {
                    let mut run = app.world_mut().resource_mut::<Run>();
                    run.talents.0[7] = u8::from(enabled);
                    run.talents.0[18] = 5;
                    run.talents.0[23] = 1;
                }
                let stats = app.world().resource::<Run>().stats();
                let mut rng = Rng(12345);
                let expected: f32 = (0..100)
                    .map(|_| {
                        if enabled {
                            roll(&mut rng, &stats, 100.0).0
                        } else {
                            100.0
                        }
                    })
                    .sum();
                assert!(stats.crit > 0.05);
                assert_eq!(stats.crit_multiplier(), 3.0);
                let e = foe(&mut app, 1.0, 100_000.0);
                add_dot(&mut app, e, key, 100.0);
                step(&mut app, 100.0);
                assert_eq!(
                    app.world().get::<Enemy>(e).unwrap().hp,
                    100_000.0 - expected
                );
                assert!(app.world().get::<Enemy>(e).unwrap().dots.is_empty());
            }
        }
    }

    #[test]
    fn dot_extension_is_wired_into_real_damage_and_preserves_equipment_tick_frequency() {
        for interval in [1.0, 0.5] {
            let mut app = app();
            app.add_systems(Update, apply_hits);
            warlock(&mut app);
            app.world_mut().resource_mut::<Run>().talents.0[2] = 5;
            let e = foe(&mut app, 1.0, 10_000.0);
            add_dot(&mut app, e, "corruption", 10.0);
            {
                let mut enemy = app.world_mut().get_mut::<Enemy>(e).unwrap();
                enemy.dots[0].tick = interval;
                enemy.dots[0].every = interval;
            }
            let mut reference = enemies::Dot {
                dps: 100.0,
                left: 10.0,
                tick: interval,
                every: interval,
                extension_left: 5.0,
                source: Source {
                    ability: Some("corruption"),
                    ..default()
                },
            };
            let mut rng = Rng(12345);
            let mut frame = 20.0;
            let mut expected = 0.0;
            while reference.next_tick(&mut frame) {
                expected += reference.dps;
                warlock::extend_dot(
                    &mut reference,
                    app.world().resource::<Run>().talents,
                    rng.f32(),
                );
            }
            assert!(expected > 1000.0 / interval as f32);
            assert!(expected <= 1500.0 / interval as f32);
            step(&mut app, 20.0);
            assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 10_000.0 - expected);
            assert!(app.world().get::<Enemy>(e).unwrap().dots.is_empty());
        }
    }

    #[test]
    fn actual_periodic_hits_refresh_only_one_direct_spell_across_multiple_targets() {
        let mut app = app();
        app.add_systems(Update, apply_hits);
        warlock(&mut app);
        let seed = (1..100).find(|seed| Rng(*seed).f32() < 0.1).unwrap();
        app.world_mut().insert_resource(Rng(seed));
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.talents.0[5] = 5;
            for (key, cd) in [
                ("shadow_bolt", 2.0),
                ("death_coil", 4.0),
                ("rain_of_fire", 9.0),
            ] {
                run.abilities.push(Owned {
                    def: data::ability(key),
                    level: 1,
                    cd,
                });
            }
        }
        for x in [1.0, 2.0] {
            let e = foe(&mut app, x, 10_000.0);
            add_dot(&mut app, e, "corruption", 1.0);
        }
        step(&mut app, 1.0);
        let run = app.world().resource::<Run>();
        assert_eq!(run.abilities.len(), 3);
        assert_eq!(run.abilities[0].cd, 2.0);
        assert_eq!(run.abilities[1].cd, 0.0);
        assert_eq!(run.abilities[2].cd, 9.0);
    }

    #[test]
    fn warlock_procs_do_not_activate_from_secondary_damage_or_for_other_classes() {
        for secondary in [false, true] {
            let mut app = app();
            app.add_systems(Update, apply_hits);
            if secondary {
                warlock(&mut app);
            }
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.talents.0[6] = 5;
                run.talents.0[14] = 5;
                run.talents.0[16] = 5;
            }
            let e = foe(&mut app, 1.0, 10_000.0);
            app.world_mut().write_message(Hit {
                source: Source {
                    ability: Some("shadow_bolt"),
                    secondary,
                    ..default()
                },
                ..Hit::direct(e, 100.0, true)
            });
            add_dot(&mut app, e, "corruption", 1.0);
            app.world_mut().get_mut::<Enemy>(e).unwrap().dots[0]
                .source
                .secondary = secondary;
            step(&mut app, 1.0);
            let enemy = app.world().get::<Enemy>(e).unwrap();
            assert_eq!(enemy.hp, 9800.0);
            assert_eq!(enemy.slow_t, 0.0);
            assert_eq!(enemy.weaken_t, 0.0);
        }
    }

    #[test]
    fn shadow_bolt_vulnerability_applies_after_crit_refreshes_and_expires() {
        let mut app = app();
        app.add_systems(Update, apply_hits);
        warlock(&mut app);
        app.world_mut().resource_mut::<Run>().talents.0[16] = 5;
        let e = foe(&mut app, 1.0, 10_000.0);
        damage(&mut app, e, "shadow_bolt", 100.0, true);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9900.0);
        damage(&mut app, e, "shadow_bolt", 100.0, true);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9785.0);
        add_dot(&mut app, e, "corruption", 1.0);
        step(&mut app, 1.0);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9670.0);
        app.world_mut().resource_mut::<Run>().elapsed = 6.0;
        damage(&mut app, e, "searing_pain", 100.0, false);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9570.0);
    }

    #[test]
    fn immolate_synergy_requires_existing_dot_and_does_not_amplify_secondary_damage() {
        let mut app = app();
        app.add_systems(Update, apply_hits);
        warlock(&mut app);
        app.world_mut().resource_mut::<Run>().talents.0[17] = 5;
        let e = foe(&mut app, 1.0, 10_000.0);
        damage(&mut app, e, "immolate", 100.0, false);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9900.0);
        add_dot(&mut app, e, "immolate", 1.0);
        damage(&mut app, e, "shadow_bolt", 100.0, false);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9780.0);
        app.world_mut().write_message(Hit {
            source: Source {
                ability: Some("shadow_bolt"),
                secondary: true,
                ..default()
            },
            ..Hit::direct(e, 100.0, true)
        });
        step(&mut app, 0.0);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9680.0);
        step(&mut app, 1.0);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9580.0);
        damage(&mut app, e, "shadow_bolt", 100.0, false);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 9480.0);
    }

    #[test]
    fn periodic_slow_and_spell_weakness_apply_without_stacking_and_boss_slow_is_halved() {
        for boss in [false, true] {
            let mut app = app();
            app.add_systems(Update, apply_hits);
            warlock(&mut app);
            {
                let mut run = app.world_mut().resource_mut::<Run>();
                run.talents.0[6] = 5;
                run.talents.0[14] = 5;
            }
            let e = foe(&mut app, 1.0, 10_000.0);
            app.world_mut().get_mut::<Enemy>(e).unwrap().boss = boss;
            add_dot(&mut app, e, "corruption", 2.0);
            step(&mut app, 2.0);
            let enemy = app.world().get::<Enemy>(e).unwrap();
            assert!((enemy.slow - if boss { 0.9 } else { 0.8 }).abs() < 0.00001);
            assert_eq!(enemy.slow_t, 3.0);
            assert_eq!(enemy.weaken, 0.9);
            assert_eq!(enemy.weaken_t, 6.0);
            assert_eq!(app.world().resource::<Run>().incoming(), 1.0);
        }
    }

    #[test]
    fn emberstorm_changes_real_cooldowns_has_shared_icd_and_excludes_secondary_procs() {
        let mut app = app();
        app.add_systems(Update, apply_hits);
        warlock(&mut app);
        app.world_mut().resource_mut::<Run>().talents.0[22] = 5;
        let e = foe(&mut app, 1.0, 10_000.0);
        let cooldown = |app: &App| {
            let run = app.world().resource::<Run>();
            super::super::talents::cast(data::ability("shadow_bolt"), 1, &run.stats()).cooldown
        };
        let base = cooldown(&app);
        damage(&mut app, e, "searing_pain", 100.0, false);
        assert_eq!(cooldown(&app), base);
        damage(&mut app, e, "immolate", 100.0, true);
        assert!((cooldown(&app) / base - 0.85).abs() < 0.00001);
        app.world_mut().resource_mut::<Run>().elapsed = 5.0;
        damage(&mut app, e, "searing_pain", 100.0, true);
        app.world_mut().resource_mut::<Run>().elapsed = 6.0;
        assert_eq!(cooldown(&app), base);
        damage(&mut app, e, "searing_pain", 100.0, true);
        assert_eq!(cooldown(&app), base);
        app.world_mut().resource_mut::<Run>().elapsed = 8.0;
        app.world_mut().write_message(Hit {
            source: Source {
                ability: Some("searing_pain"),
                secondary: true,
                ..default()
            },
            ..Hit::direct(e, 100.0, true)
        });
        step(&mut app, 0.0);
        assert_eq!(cooldown(&app), base);
        damage(&mut app, e, "searing_pain", 100.0, true);
        assert!((cooldown(&app) / base - 0.85).abs() < 0.00001);
        assert_eq!(warlock::haste_remaining(app.world().resource::<Run>()), 6.0);
    }

    #[test]
    fn actual_bolt_cast_adds_targets_and_only_supported_global_radius_changes_area_hits() {
        let mut app = app();
        app.add_systems(Update, (cast_abilities, apply_hits).chain());
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        let first = foe(&mut app, 4.0, 1000.0);
        let second = foe(&mut app, 7.0, 1000.0);
        enhance(&mut app, EnhKind::Count, Some("frostbolt"), 1.0);
        enhance(&mut app, EnhKind::Radius, None, 1.0);
        app.world_mut().resource_mut::<Run>().abilities.push(Owned {
            def: data::ability("frostbolt"),
            level: 1,
            cd: 0.0,
        });
        step(&mut app, 0.1);
        assert!(app.world().get::<Enemy>(first).unwrap().hp < 1000.0);
        assert!(app.world().get::<Enemy>(second).unwrap().hp < 1000.0);
        let mut run = app.world_mut().resource_mut::<Run>();
        run.abilities.clear();
        run.abilities.push(Owned {
            def: data::ability("arcane_explosion"),
            level: 1,
            cd: 0.0,
        });
        let distant = foe(&mut app, 12.0, 1000.0);
        step(&mut app, 0.1);
        assert!(app.world().get::<Enemy>(distant).unwrap().hp < 1000.0);
    }

    #[test]
    fn cast_frequency_reaches_dot_scheduler_without_reducing_single_tick_damage() {
        let mut app = app();
        app.add_systems(Update, (cast_abilities, apply_hits).chain());
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        let e = foe(&mut app, 4.0, 1000.0);
        enhance(&mut app, EnhKind::Frequency, Some("corruption"), 1.0);
        app.world_mut().resource_mut::<Run>().abilities.push(Owned {
            def: data::ability("corruption"),
            level: 1,
            cd: 0.0,
        });
        step(&mut app, 0.1);
        let dot = &app.world().get::<Enemy>(e).unwrap().dots[0];
        assert_eq!(dot.every, 0.5);
        assert_eq!(dot.dps, 4.0);
        app.world_mut().resource_mut::<Run>().abilities.clear();
        step(&mut app, 10.5);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 920.0);
    }

    #[test]
    fn echo_schedules_one_half_strength_hit_without_splash_or_recursive_echo() {
        use bevy::ecs::system::RunSystemOnce;
        let mut app = app();
        app.world_mut()
            .run_system_once(
                |mut pending: ResMut<PendingHits>, mut hits: MessageWriter<Hit>| {
                    let mut hit = Hit::direct(Entity::PLACEHOLDER, 100.0, false);
                    hit.source.echo = true;
                    hit.splash = 3.0;
                    schedule(&mut pending, &mut hits, 0.0, 0.0, hit);
                },
            )
            .unwrap();
        let pending = app.world().resource::<PendingHits>();
        assert_eq!(pending.0.len(), 1);
        let (at, hit) = &pending.0[0];
        assert_eq!(*at, 0.25);
        assert_eq!(hit.damage, 50.0);
        assert_eq!(hit.splash, 0.0);
        assert!(hit.source.secondary);
        assert!(!hit.source.echo);
    }

    #[test]
    fn actual_dot_ticks_double_damage_without_missing_ticks_on_long_frames() {
        for (interval, expected) in [(1.0, 600.0), (0.5, 1200.0)] {
            let mut app = app();
            app.add_systems(Update, apply_hits);
            let e = foe(&mut app, 1.0, 2000.0);
            app.world_mut()
                .get_mut::<Enemy>(e)
                .unwrap()
                .dots
                .push(enemies::Dot {
                    dps: 100.0,
                    left: 6.0,
                    tick: interval,
                    every: interval,
                    extension_left: 3.0,
                    source: Source {
                        ability: Some("corruption"),
                        batch: 1,
                        ..default()
                    },
                });
            for _ in 0..4 {
                step(&mut app, 1.7);
            }
            assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 2000.0 - expected);
            assert!(app.world().get::<Enemy>(e).unwrap().dots.is_empty());
        }
    }

    #[test]
    fn ground_and_totem_use_independent_tick_frequency_and_damage_provenance() {
        for interval in [1.0, 0.5] {
            let mut app = app();
            app.add_systems(Update, (tick_areas, apply_hits).chain());
            let e = foe(&mut app, 1.0, 5000.0);
            let source = Source {
                ability: Some("blizzard"),
                batch: 1,
                ..default()
            };
            app.world_mut().spawn((
                Transform::default(),
                GroundArea {
                    damage: 100.0,
                    dot: 0.0,
                    dot_dur: 0.0,
                    radius: 5.0,
                    left: 6.0,
                    every: interval,
                    next_tick: interval,
                    effect: Effect::None,
                    mark: 0,
                    impact: 0,
                    source,
                },
            ));
            app.world_mut().spawn((
                Transform::default(),
                Totem {
                    cast: 403,
                    damage: 100.0,
                    range: 20.0,
                    radius: 0.0,
                    left: 6.0,
                    tick: interval,
                    pulse: false,
                    every: interval,
                    source: Source {
                        ability: Some("searing_totem"),
                        batch: 2,
                        ..default()
                    },
                },
            ));
            for _ in 0..4 {
                step(&mut app, 1.7);
            }
            let mut rng = Rng(12345);
            let stats = Run::default().stats();
            let expected: f32 = (0..(2.0 * 6.0 / interval) as u32)
                .map(|_| roll(&mut rng, &stats, 100.0).0)
                .sum();
            assert_eq!(app.world().get::<Enemy>(e).unwrap().hp, 5000.0 - expected);
        }
    }

    #[test]
    fn healing_uses_effective_damage_instead_of_overkill_in_the_real_hit_pipeline() {
        let mut app = app();
        app.add_systems(Update, apply_hits);
        let e = foe(&mut app, 1.0, 10.0);
        app.world_mut().resource_mut::<Run>().hp = 50.0;
        let mut hit = Hit::direct(e, 1000.0, false);
        hit.source = Source {
            ability: Some("frostbolt"),
            batch: 1,
            modifiers: skill_effects::Modifiers {
                leech: 0.1,
                ..default()
            },
            ..default()
        };
        app.world_mut().resource_mut::<Messages<Hit>>().write(hit);
        step(&mut app, 1.0);
        assert_eq!(app.world().resource::<Run>().hp, 51.0);
    }

    #[test]
    fn echo_and_ember_do_not_recursively_generate_secondary_effects() {
        let mut app = app();
        app.add_systems(Update, (cast_abilities, apply_hits).chain());
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        let e = foe(&mut app, 4.0, 10000.0);
        enhance(&mut app, EnhKind::Ember, Some("frostbolt"), 0.3);
        enhance(&mut app, EnhKind::Echo, Some("frostbolt"), 0.5);
        app.world_mut().resource_mut::<Run>().abilities.push(Owned {
            def: data::ability("frostbolt"),
            level: 1,
            cd: 0.0,
        });
        step(&mut app, 0.1);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().dots.len(), 1);
        app.world_mut().resource_mut::<Run>().abilities.clear();
        step(&mut app, 0.5);
        assert_eq!(app.world().get::<Enemy>(e).unwrap().dots.len(), 1);
        assert!(app.world().resource::<PendingHits>().0.is_empty());
        for _ in 0..4 {
            step(&mut app, 1.0);
        }
        assert!(app.world().get::<Enemy>(e).unwrap().dots.is_empty());
    }
}
