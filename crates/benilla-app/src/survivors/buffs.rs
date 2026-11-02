//! A single timed-buff tray beside the minimap; permanent passives remain in their own rows.

use bevy::prelude::*;

use super::{data, gear, hud, localization as l, Phase, Run};

pub(super) fn shield_spell(source: Option<&str>) -> u32 {
    let key = match source {
        Some("ice_barrier_talent") => "ice_barrier",
        Some(key) => key,
        None => return 17,
    };
    data::ABILITIES
        .iter()
        .find(|def| def.key == key)
        .map_or(17, |def| def.spell)
}

struct Entry {
    name: String,
    description: String,
    remaining: f32,
    stacks: usize,
    spell: u32,
    art: Option<&'static str>,
}

#[derive(Resource, Default)]
struct Tray {
    entries: Vec<Entry>,
    slots: usize,
}

#[derive(Component)]
struct Root;
#[derive(Component)]
struct Slot(usize);
#[derive(Component)]
struct Icon {
    slot: usize,
    shown: Option<(u32, Option<&'static str>)>,
}
#[derive(Component)]
struct Timer(usize);
#[derive(Component)]
struct Stack(usize);
#[derive(Component)]
struct Tip;
#[derive(Component)]
struct TipText;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Tray>().add_systems(
        Update,
        (build, refresh, hover).chain().after(hud::refresh_hud),
    );
}

fn entries(run: &Run, _gear: &gear::Gear) -> Vec<Entry> {
    let mut entries = Vec::new();
    let mut add = |name: String, desc: String, remaining: f32, spell| {
        if remaining > 0.0 {
            entries.push(Entry {
                name,
                description: desc,
                remaining,
                stacks: 1,
                spell,
                art: None,
            });
        }
    };
    if run.shield > 0.0 {
        let name = match run.shield_source {
            Some("ice_barrier") => l::text("Ice Barrier", "寒冰护体"),
            Some("ice_barrier_talent") => l::text("Ice Barrier talent", "寒冰护体天赋"),
            Some("mana_shield") => l::text("Mana Shield", "法力护盾"),
            Some("power_word_shield") => l::text("Power Word: Shield", "真言术：盾"),
            Some("last_stand") => l::text("Last Stand", "破釜沉舟"),
            Some("ice_block") => l::text("Ice Block · Absorption", "寒冰屏障·附加护盾"),
            Some("divine_shield") => l::text("Divine Shield", "圣盾术"),
            Some("sacred_shield") => l::text("Sacred Shield", "神圣护盾"),
            Some("ancestral_guardian") => l::text("Ancestral Guardian", "先祖守护"),
            Some("living_seed") => l::text("Living Seed", "生命之种"),
            Some("cold_snap") => l::text("Cold Snap absorption", "急速冷却护盾"),
            _ => l::text("Absorption shield", "吸收护盾"),
        };
        add(
            name.into(),
            format!(
                "{}: {:.0}",
                l::text("Remaining absorption", "剩余吸收量"),
                run.shield
            ),
            run.shield_t,
            shield_spell(run.shield_source),
        );
    }
    if run.ice_barrier.present() {
        add(
            l::text("Ice Barrier · Skill", "寒冰护体·技能").into(),
            format!("{}: {:.0} · {}",
                l::text("Remaining absorption", "剩余吸收量"), run.ice_barrier.capacity,
                l::text("Independent cumulative shield; absorbs before other shields after damage reduction.", "独立累计吸收池；减伤后先于其他盾吸收。")),
            run.ice_barrier.left,
            data::ability("ice_barrier").spell,
        );
    }
    add(
        hud::spell_name(
            None,
            run.immune_spell,
            if run.immune_spell == 11958 {
                "ice_block"
            } else {
                "divine_shield"
            },
        ),
        if run.ice_blocked() {
            l::text("Immune to damage; cannot move, cast or use F Blink. Existing missiles, areas and periodic damage continue; cooldowns recover normally.", "免疫伤害；不能移动、施法或F闪现。已有投射物、区域、持续伤害继续，冷却正常恢复。")
        } else {
            l::text("Immune to incoming damage.", "持续期间免疫受到的伤害。")
        }.into(),
        run.immune_t,
        run.immune_spell,
    );
    add(
        hud::spell_name(
            None,
            run.thorns_spell,
            data::ABILITIES
                .iter()
                .find(|def| def.spell == run.thorns_spell)
                .map_or("thorns", |def| def.key),
        ),
        format!(
            "{}: {:.0}{}",
            l::text("Contact retaliation damage", "接触攻击反伤"),
            run.thorns,
            if run.thorns_every > 0.0 {
                format!(
                    " · {}: {:.2}s",
                    l::text("Interval", "触发间隔"),
                    run.thorns_every
                )
            } else {
                String::new()
            }
        ),
        run.thorns_t,
        run.thorns_spell,
    );
    add(
        hud::spell_name(
            None,
            run.heal_spell,
            data::ABILITIES
                .iter()
                .find(|def| def.spell == run.heal_spell)
                .map_or("rejuvenation", |def| def.key),
        ),
        format!(
            "{}: {:.1}",
            l::text("Healing per second", "每秒治疗量"),
            run.heal_rate
        ),
        run.heal_t,
        run.heal_spell,
    );
    if let Some(mobility) = run.mobility() {
        add(
            hud::spell_name(None, mobility.spell, mobility.name),
            format!(
                "{}: +{:.0}%",
                l::text("Movement speed", "移动速度"),
                (run.mobile.haste - 1.0) * 100.0
            ),
            run.mobile.haste_t,
            mobility.spell,
        );
        add(
            l::text("Faded", "渐隐").into(),
            l::text(
                "Enemies temporarily lose their target.",
                "敌人暂时失去追踪目标。",
            )
            .into(),
            run.mobile.vanish_t,
            mobility.spell,
        );
    }
    let haste = super::warlock::haste(run);
    if haste > 0.0 {
        entries.push(Entry {
            name: l::text("Emberstorm", "琥珀风暴").into(),
            description: format!("{}: +{:.0}%", l::text("Haste", "急速"), haste * 100.0),
            remaining: super::warlock::haste_remaining(run),
            stacks: 1,
            spell: 0,
            art: Some(super::talents::NODES[22].icon),
        });
    }
    for (node, en, zh, remaining) in super::class_mechanics::buffs(run) {
        if let Some(nodes) = super::talents::nodes(run.class.id) {
            entries.push(Entry {
                name: if run.class.id == 8 {
                    format!(
                        "{} · {}",
                        l::text(nodes[node].name, nodes[node].zh),
                        l::text("Talent", "天赋")
                    )
                } else {
                    l::text(en, zh).into()
                },
                description: format!(
                    "{} · {} {}\n{}",
                    l::text(nodes[node].name, nodes[node].zh),
                    l::text("Rank", "等级"),
                    run.talents.0[node],
                    l::text(nodes[node].desc, nodes[node].desc_zh)
                ),
                remaining,
                stacks: 1,
                spell: 0,
                art: Some(nodes[node].icon),
            });
        }
    }
    let active = run.effects.active_buffs(run.elapsed);
    for kind in gear::ENH_KINDS.into_iter().filter(|kind| kind.is_buff()) {
        let sources: Vec<_> = active.iter().filter(|(_, k, _, _)| *k == kind).collect();
        if sources.is_empty() {
            continue;
        }
        let remaining = sources
            .iter()
            .map(|(_, _, _, left)| *left)
            .fold(0.0, f32::max);
        let total = run.effects.bonus(kind, run.elapsed);
        let mut description = format!(
            "{}: {}\n",
            l::text("Effective total", "当前实际合计"),
            kind.show(total)
        );
        description.push_str(l::text(
            "Totals respect the shared damage/haste/crit/speed/defense caps: 100/40/30/50/40%.",
            "合计遵循伤害/急速/暴击/移速/减伤上限：100/40/30/50/40%。",
        ));
        entries.push(Entry {
            name: format!("{} · {}", l::text("Equipment", "装备"), kind.label()),
            description,
            remaining,
            stacks: sources.len(),
            spell: match kind {
                gear::EnhKind::BuffDamage => 1719,
                gear::EnhKind::BuffHaste => 10060,
                gear::EnhKind::BuffCrit => 18288,
                gear::EnhKind::BuffSpeed => 2983,
                _ => 871,
            },
            art: None,
        });
    }
    for buff in super::spell_expansion::buffs(run) {
        debug_assert!(
            !buff.key.is_empty(),
            "a timed spell buff must identify its source"
        );
        entries.push(Entry {
            name: if l::is_zh_cn() {
                buff.name.into()
            } else {
                hud::spell_name(None, buff.spell, buff.key)
            },
            description: buff.description,
            remaining: buff.remaining,
            stacks: buff.stacks as usize,
            spell: buff.spell,
            art: None,
        });
    }
    if run.shield > 0.0 && run.shield_t > 0.0 {
        let talent = match run.shield_source {
            Some("sacred_shield") => Some((2, 14)),
            Some("ancestral_guardian") => Some((7, 23)),
            Some("living_seed") => Some((11, 22)),
            _ => None,
        };
        if let Some((class, node)) = talent {
            if let Some(entry) = entries.first_mut() {
                entry.art = super::talents::nodes(class).map(|nodes| nodes[node].icon);
            }
        }
    }
    entries
}

fn time_label(seconds: f32) -> String {
    let seconds = seconds.ceil().max(0.0) as u32;
    if seconds >= 60 {
        format!("{}:{:02}", seconds / 60, seconds % 60)
    } else {
        format!("{seconds}s")
    }
}

fn build(
    mut commands: Commands,
    root: Query<Entity, With<hud::HudRoot>>,
    existing: Query<Entity, With<Root>>,
    font: Option<Res<hud::HudFont>>,
) {
    if !existing.is_empty() {
        return;
    }
    let (Ok(root), Some(font)) = (root.single(), font) else {
        return;
    };
    commands.entity(root).with_children(|parent| {
        parent.spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(184.0),
                top: Val::Px(24.0),
                width: Val::Px(252.0),
                flex_direction: FlexDirection::RowReverse,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(6.0),
                ..default()
            },
        ));
        parent
            .spawn((
                Tip,
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    right: Val::Px(184.0),
                    top: Val::Px(210.0),
                    width: Val::Px(360.0),
                    max_width: Val::Vw(90.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.025, 0.015, 0.97)),
                BorderColor::all(Color::srgb(0.7, 0.55, 0.2)),
                GlobalZIndex(110),
            ))
            .with_children(|tip| {
                tip.spawn((TipText, hud::text_bundle(&font.0, "", 13.0, Color::WHITE)));
            });
    });
}

fn refresh(
    mut commands: Commands,
    run: Res<Run>,
    gear: Res<gear::Gear>,
    phase: Res<State<Phase>>,
    font: Option<Res<hud::HudFont>>,
    mut tray: ResMut<Tray>,
    root: Query<Entity, With<Root>>,
    mut slots: Query<(&Slot, &mut Node)>,
    mut icons: Query<(Entity, &mut Icon)>,
    mut timers: Query<(&Timer, &mut Text, &mut TextColor), Without<Stack>>,
    mut stacks: Query<(&Stack, &mut Text), Without<Timer>>,
) {
    tray.entries = if matches!(phase.get(), Phase::Playing | Phase::Paused | Phase::LevelUp) {
        entries(&run, &gear)
    } else {
        Vec::new()
    };
    let (Ok(root), Some(font)) = (root.single(), font) else {
        return;
    };
    while tray.slots < tray.entries.len() {
        let index = tray.slots;
        commands.entity(root).with_children(|parent| {
            parent
                .spawn((
                    Slot(index),
                    Interaction::None,
                    Node {
                        width: Val::Px(36.0),
                        height: Val::Px(51.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                ))
                .with_children(|slot| {
                    slot.spawn((
                        Icon {
                            slot: index,
                            shown: None,
                        },
                        ImageNode::default(),
                        Node {
                            width: Val::Px(34.0),
                            height: Val::Px(34.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(Color::srgb(0.8, 0.65, 0.3)),
                    ));
                    slot.spawn((
                        Timer(index),
                        hud::text_bundle(&font.0, "", 11.0, Color::WHITE),
                    ));
                    slot.spawn((
                        Stack(index),
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(2.0),
                            top: Val::Px(17.0),
                            ..default()
                        },
                        hud::text_bundle(&font.0, "", 12.0, Color::WHITE),
                    ));
                });
        });
        tray.slots += 1;
    }
    for (slot, mut node) in &mut slots {
        let display = if slot.0 < tray.entries.len() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (entity, mut icon) in &mut icons {
        let Some(entry) = tray.entries.get(icon.slot) else {
            continue;
        };
        let identity = (entry.spell, entry.art);
        if icon.shown == Some(identity) {
            continue;
        }
        if let Some(path) = entry.art {
            commands
                .entity(entity)
                .remove::<(hud::SpellIcon, hud::IconDone, hud::ArtDone)>()
                .insert(hud::UiArt::crop(path, [0.0, 1.0, 0.0, 1.0]));
        } else {
            commands
                .entity(entity)
                .remove::<(hud::UiArt, hud::IconDone, hud::ArtDone)>()
                .insert(hud::SpellIcon(entry.spell));
        }
        icon.shown = Some(identity);
    }
    for (timer, mut text, mut color) in &mut timers {
        if let Some(entry) = tray.entries.get(timer.0) {
            let label = time_label(entry.remaining);
            if text.0 != label {
                text.0 = label;
            }
            let tint = if entry.remaining <= 5.0 {
                Color::srgb(1.0, 0.3, 0.2)
            } else {
                Color::WHITE
            };
            if color.0 != tint {
                color.0 = tint;
            }
        }
    }
    for (stack, mut text) in &mut stacks {
        let label = tray
            .entries
            .get(stack.0)
            .filter(|entry| entry.stacks > 1)
            .map(|entry| entry.stacks.to_string())
            .unwrap_or_default();
        if text.0 != label {
            text.0 = label;
        }
    }
}

fn hover(
    tray: Res<Tray>,
    slots: Query<(&Slot, &Interaction)>,
    mut tooltip: Query<&mut Node, With<Tip>>,
    mut labels: Query<&mut Text, With<TipText>>,
) {
    let entry = slots
        .iter()
        .find(|(_, interaction)| **interaction != Interaction::None)
        .and_then(|(slot, _)| tray.entries.get(slot.0));
    for mut node in &mut tooltip {
        let display = if entry.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    if let Some(entry) = entry {
        for mut text in &mut labels {
            let label = format!(
                "{}{}\n{}\n{}: {}",
                entry.name,
                if entry.stacks > 1 {
                    format!(" ×{}", entry.stacks)
                } else {
                    String::new()
                },
                entry.description,
                l::text("Remaining", "剩余时间"),
                time_label(entry.remaining)
            );
            if text.0 != label {
                text.0 = label;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mage_active_and_talent_arcane_power_keep_separate_rules_and_names() {
        let mut run = Run {
            class: data::CLASSES.iter().find(|c| c.id == 8).unwrap(),
            ..default()
        };
        run.talents.0[7] = 1;
        super::super::class_mechanics::on_mobility(&mut run);
        let cast = super::super::growth::cast_for_run(data::ability("arcane_power"), 1, 1.0, &run);
        assert_eq!(
            super::super::spell_expansion::utility(&mut run, "arcane_power", &cast, true),
            Some(true)
        );
        let tray = entries(&run, &gear::Gear::default());
        let talent = tray
            .iter()
            .find(|e| e.name.contains("天赋") || e.name.contains("Talent"))
            .unwrap();
        let skill = tray.iter().find(|e| e.spell == 12042).unwrap();
        assert_ne!(talent.name, skill.name);
        assert!(talent.description.contains("20%") && talent.description.contains("15%"));
        assert!(skill.description.contains("30%"));
        assert_eq!((talent.remaining, skill.remaining), (5.0, 6.0));
    }

    #[test]
    fn mage_skill_and_talent_shields_have_independent_capacity_and_timers() {
        let mut run = Run {
            shield: 20.0,
            shield_t: 6.0,
            shield_source: Some("ice_barrier_talent"),
            ..default()
        };
        run.ice_barrier.cast(55.0, 12.0);
        let gear = gear::Gear::default();
        let tray = entries(&run, &gear);
        assert_eq!(tray.len(), 2);
        assert_ne!(tray[0].name, tray[1].name);
        assert!(tray[0].description.contains("20"));
        assert!(tray[1].description.contains("55"));
        assert_eq!((tray[0].remaining, tray[1].remaining), (6.0, 12.0));
        run.ice_barrier.tick(12.0);
        let tray = entries(&run, &gear);
        assert_eq!(tray.len(), 1);
        assert_eq!(tray[0].remaining, 6.0);
    }

    #[test]
    fn timed_effects_expire_and_do_not_include_permanent_passives() {
        let mut run = Run {
            shield_t: 3.2,
            shield: 40.0,
            immune_t: 2.0,
            ..default()
        };
        let gear = gear::Gear::default();
        assert_eq!(entries(&run, &gear).len(), 2);
        run.shield = 0.0;
        run.immune_t = 0.0;
        assert!(entries(&run, &gear).is_empty());
        assert_eq!(time_label(60.1), "1:01");
        assert_eq!(time_label(4.1), "5s");
    }

    #[test]
    fn absorb_buffs_use_the_source_spell_icon() {
        for key in [
            "ice_barrier",
            "mana_shield",
            "power_word_shield",
            "last_stand",
        ] {
            let run = Run {
                shield: 40.0,
                shield_t: 5.0,
                shield_source: Some(key),
                ..default()
            };
            let buffs = entries(&run, &gear::Gear::default());
            assert_eq!(buffs.len(), 1);
            assert_eq!(
                buffs[0].spell,
                super::super::data::ability(key).spell,
                "{key}"
            );
        }
    }

    #[test]
    fn recycled_buff_slots_invalidate_previously_loaded_icons() {
        let mut app = App::new();
        app.init_resource::<Run>()
            .init_resource::<gear::Gear>()
            .init_resource::<Tray>()
            .insert_resource(State::new(Phase::Playing))
            .insert_resource(hud::HudFont(Handle::default()))
            .add_systems(Update, (build, refresh).chain());
        app.world_mut().spawn(hud::HudRoot);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            let cast = super::super::growth::cast_for_run(
                super::super::data::ability("combustion"),
                1,
                1.0,
                &run,
            );
            super::super::spell_expansion::utility(&mut run, "combustion", &cast, true);
        }
        app.update();
        app.update();
        let world = app.world_mut();
        let icon = world
            .query_filtered::<Entity, With<Icon>>()
            .single(world)
            .unwrap();
        world.entity_mut(icon).insert(hud::IconDone);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.elapsed = 100.0;
            let cast = super::super::growth::cast_for_run(
                super::super::data::ability("frost_armor"),
                1,
                1.0,
                &run,
            );
            super::super::spell_expansion::utility(&mut run, "frost_armor", &cast, true);
        }
        app.update();
        assert_eq!(app.world().get::<hud::SpellIcon>(icon).unwrap().0, 168);
        assert!(app.world().get::<hud::IconDone>(icon).is_none());
    }

    #[test]
    fn tray_reuses_icons_and_hover_shows_actual_effect_and_duration() {
        let mut app = App::new();
        app.init_resource::<Run>()
            .init_resource::<gear::Gear>()
            .init_resource::<Tray>()
            .insert_resource(State::new(Phase::Playing))
            .insert_resource(hud::HudFont(Handle::default()))
            .add_systems(Update, (build, refresh, hover).chain());
        app.world_mut().spawn(hud::HudRoot);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.shield = 72.0;
            run.shield_t = 3.2;
        }
        app.update();
        app.update();
        let world = app.world_mut();
        let slot = world
            .query_filtered::<Entity, With<Slot>>()
            .single(world)
            .unwrap();
        world.entity_mut(slot).insert(Interaction::Hovered);
        app.update();
        let world = app.world_mut();
        let tip = world
            .query_filtered::<&Text, With<TipText>>()
            .single(world)
            .unwrap();
        assert!(tip.0.contains("72"));
        assert!(tip.0.contains("4s"));
        app.world_mut().resource_mut::<Run>().shield_t = 0.0;
        app.update();
        assert_eq!(app.world().resource::<Tray>().entries.len(), 0);
        let world = app.world_mut();
        assert_eq!(
            world
                .query_filtered::<Entity, With<Slot>>()
                .single(world)
                .unwrap(),
            slot
        );
        assert_eq!(world.get::<Node>(slot).unwrap().display, Display::None);
    }
}
