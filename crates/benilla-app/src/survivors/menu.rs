//! Class, appearance, destination and talent selection for the offline launcher.

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;

use super::hud::{looks_for, rgb, start_run, HudFont, MenuRoot, SpellIcon, UiArt};
use super::menu_layout::{
    self as layout, AppearanceRoot, ClassBadge, NoticeRoot, NoticeText, PoolIcon, RaceBadge,
    ZoneList, ZoneScrollThumb, ZoneThumb,
};
use super::{data, localization, Phase, Rng, Run, Selection};
use crate::char_select::race_name;
use crate::glue::art::{class_tc, race_tc, tc_rect, GlueArt, DIM, GOLD};
use crate::glue::widgets::{HoverLabel, LockHighlight};
use crate::glue::GlueClicks;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum MenuAction {
    Race(u8),
    Sex(u8),
    Class(usize),
    Map(usize),
    Randomize,
    RandomName,
    Talents,
    Gear,
    Enter,
    Character,
    Appearance,
    Close,
    Feature(&'static str, &'static str),
}

#[derive(Resource, Default)]
pub(super) struct MenuState {
    pub notice: Option<String>,
    pub appearance: bool,
    name_index: usize,
}

impl MenuState {
    pub fn modal_open(&self) -> bool {
        self.notice.is_some() || self.appearance
    }
}

#[derive(Component)]
pub(super) struct MenuBuilt;

#[derive(Component)]
pub(super) struct RaceFace(pub u8);

#[derive(Component)]
pub(super) enum MenuCopy {
    Name,
    Champion,
    Faction,
    ClassName,
    ClassDetails,
    TalentSummary,
    Destination,
    PersistenceError,
    Health,
    StartingAbility,
    ZoneCount,
    RaceName,
    ZoneName,
    ZoneDetails,
}

fn pick_path() -> Option<std::path::PathBuf> {
    crate::local_state::home().map(|h| h.join("survivors-pick.txt"))
}

fn save_pick(sel: &Selection) {
    let (Some(class), Some(&(_, race, sex))) = (sel.class, sel.look.and_then(|i| sel.looks.get(i)))
    else {
        return;
    };
    if let Some(path) = pick_path() {
        let _ =
            crate::local_state::write_atomic(&path, &format!("{class} {race} {sex} {}", sel.map));
    }
}

pub(super) fn restore_pick(
    creatures: Option<Res<crate::entities::Creatures>>,
    mut selection: ResMut<Selection>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(creatures) = creatures.as_deref() else {
        return;
    };
    *done = true;
    let Some(text) = pick_path().and_then(|p| crate::local_state::read_to_string(&p).ok()) else {
        return;
    };
    let v: Vec<usize> = text
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    let [class, race, sex, map] = v[..] else {
        return;
    };
    if map < data::MAPS.len() {
        selection.map = map;
        selection.stage = map;
        selection.version += 1;
    }
    if class < data::CLASSES.len() {
        let all = all_looks(creatures);
        if let Some(l) = find(&all[class], Some(race as u8), Some(sex as u8)) {
            set(&mut selection, &all, class, l);
        }
    }
}

fn all_looks(creatures: &crate::entities::Creatures) -> Vec<Vec<(u32, u8, u8)>> {
    data::CLASSES
        .iter()
        .map(|c| looks_for(c, creatures))
        .collect()
}

fn find(looks: &[(u32, u8, u8)], race: Option<u8>, sex: Option<u8>) -> Option<usize> {
    looks
        .iter()
        .position(|&(_, r, s)| race.is_none_or(|x| x == r) && sex.is_none_or(|x| x == s))
}

fn set(sel: &mut Selection, all: &[Vec<(u32, u8, u8)>], class: usize, look: usize) {
    sel.class = Some(class);
    sel.looks = all[class].clone();
    sel.look = Some(look);
    sel.version += 1;
}

pub(super) fn build_menu(
    mut commands: Commands,
    roots: Query<Entity, (With<MenuRoot>, Without<MenuBuilt>)>,
    assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut art: ResMut<GlueArt>,
    mut images: ResMut<Assets<Image>>,
    mut add_mats: ResMut<Assets<crate::glue::add_material::AddUiMaterial>>,
    font: Res<HudFont>,
) {
    let Ok(root) = roots.single() else { return };
    let Some(mut assets) = assets else { return };
    art.ensure_loaded(&mut assets, &mut images, &mut add_mats);
    commands
        .entity(root)
        .insert(MenuBuilt)
        .with_children(|ui| layout::build(ui, &art, &font.0));
}

#[allow(clippy::too_many_arguments)]
pub(super) fn menu_clicks(
    clicks: Res<GlueClicks>,
    actions: Query<(Entity, &MenuAction)>,
    phase: Res<State<Phase>>,
    creatures: Option<Res<crate::entities::Creatures>>,
    mut selection: ResMut<Selection>,
    mut run: ResMut<Run>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<Phase>>,
    mut talents: ResMut<super::talents::TalentUi>,
    mut gear: ResMut<super::gear_ui::GearUi>,
    mut menu: ResMut<MenuState>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if *phase.get() != Phase::Menu || talents.open || gear.open {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        menu.notice = None;
        menu.appearance = false;
        return;
    }
    let Some(action) = actions
        .iter()
        .find(|(e, _)| clicks.hit(*e))
        .map(|(_, a)| *a)
    else {
        return;
    };
    if !action_allowed(action, &menu) {
        return;
    }
    match action {
        MenuAction::Close => {
            menu.notice = None;
            menu.appearance = false;
            return;
        }
        MenuAction::Feature(en, zh) => {
            menu.notice = Some(format!(
                "{}\n\n{}",
                localization::text(en, zh),
                localization::text(
                    "Not yet available. This entrance is reserved for a future update.",
                    "尚未开放，入口已预留，敬请期待。"
                )
            ));
            return;
        }
        MenuAction::Character | MenuAction::Appearance => {
            menu.appearance = true;
            return;
        }
        MenuAction::RandomName => {
            menu.name_index = (menu.name_index + 1) % 8;
            return;
        }
        MenuAction::Gear => {
            gear.open = true;
            return;
        }
        MenuAction::Talents => {
            if selection
                .class_def()
                .is_some_and(|c| super::talents::nodes(c.id).is_some())
            {
                talents.open = true;
            } else {
                menu.notice = Some(
                    localization::text(
                        "Choose a class before opening talents.",
                        "请先选择职业，再打开天赋。",
                    )
                    .into(),
                );
            }
            return;
        }
        _ => {}
    }
    let Some(creatures) = creatures.as_deref() else {
        return;
    };
    let all = all_looks(creatures);
    let cur = selection
        .look
        .and_then(|i| selection.looks.get(i))
        .map(|&(_, r, s)| (r, s));
    let (race, sex) = (cur.map(|c| c.0), cur.map_or(0, |c| c.1));
    match action {
        MenuAction::Map(i) => {
            if selection.map != i {
                selection.map = i;
                selection.version += 1;
            }
        }
        MenuAction::Enter => {
            if start_run(&mut run, &mut selection) {
                next.set(Phase::Playing);
            }
        }
        MenuAction::Randomize => {
            let picks: Vec<_> = all
                .iter()
                .enumerate()
                .flat_map(|(c, looks)| (0..looks.len()).map(move |l| (c, l)))
                .collect();
            if !picks.is_empty() {
                let (c, l) = picks[rng.index(picks.len())];
                set(&mut selection, &all, c, l);
            }
        }
        MenuAction::Class(c) => {
            let looks = &all[c];
            let l = find(looks, race, Some(sex))
                .or_else(|| find(looks, race, None))
                .or_else(|| find(looks, None, Some(sex)))
                .or_else(|| find(looks, None, None));
            if let Some(l) = l {
                set(&mut selection, &all, c, l);
            }
        }
        MenuAction::Race(r) => {
            for c in selection.class.into_iter().chain(0..all.len()) {
                let l = find(&all[c], Some(r), Some(sex)).or_else(|| find(&all[c], Some(r), None));
                if let Some(l) = l {
                    set(&mut selection, &all, c, l);
                    break;
                }
            }
        }
        MenuAction::Sex(s) => {
            if let Some(c) = selection.class {
                if let Some(l) = find(&all[c], race, Some(s)) {
                    set(&mut selection, &all, c, l);
                }
            }
        }
        _ => {}
    }
    save_pick(&selection);
}

fn action_allowed(action: MenuAction, menu: &MenuState) -> bool {
    if menu.notice.is_some() {
        return action == MenuAction::Close;
    }
    !menu.appearance
        || matches!(
            action,
            MenuAction::Race(_) | MenuAction::Sex(_) | MenuAction::Randomize | MenuAction::Close
        )
}

fn class_description(id: u8) -> &'static str {
    match id {
        1 => localization::text(
            "A frontline fighter who cleaves through crowds.",
            "近战战士，以强健体魄与横扫攻击突破敌阵。",
        ),
        2 => localization::text(
            "A holy defender who burns the undead with sacred light.",
            "圣光卫士，以神圣法术与坚韧体魄迎战敌潮。",
        ),
        3 => localization::text(
            "A ranged marksman with traps and precision shots.",
            "远程猎手，以精准射击与陷阱掌控战场。",
        ),
        4 => localization::text(
            "A swift assassin who strikes with blades and poison.",
            "敏捷刺客，以利刃、毒药和连击迅速清场。",
        ),
        5 => localization::text(
            "A spellcaster wielding both holy light and shadow.",
            "驾驭圣光与暗影，以精神之力对抗无尽敌潮。",
        ),
        7 => localization::text(
            "An elemental warrior wielding storms and totems.",
            "元素使者，以闪电、烈焰和图腾席卷战场。",
        ),
        8 => localization::text(
            "A master of arcane, fire and frost area spells.",
            "奥术大师，以烈焰、冰霜与奥术范围法术歼敌。",
        ),
        9 => localization::text(
            "A dark caster who wears enemies down with curses and fire.",
            "暗影法师，以诅咒、持续伤害和邪焰逐步吞噬敌人。",
        ),
        _ => localization::text(
            "A guardian of nature commanding moonlight and thorns.",
            "自然守护者，以月光、荆棘和自然法术清扫战场。",
        ),
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn refresh_menu_ui(
    mut commands: Commands,
    selection: Res<Selection>,
    art: Res<GlueArt>,
    progression: Res<super::talents::Progression>,
    menu: Res<MenuState>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut buttons: Query<(
        &MenuAction,
        &Interaction,
        Option<&mut LockHighlight>,
        Option<&mut BorderColor>,
        Option<&mut BackgroundColor>,
    )>,
    mut labels: Query<&mut Visibility, With<HoverLabel>>,
    mut faces: Query<(&RaceFace, &mut ImageNode), (Without<RaceBadge>, Without<ClassBadge>)>,
    mut race_badges: Query<
        &mut ImageNode,
        (With<RaceBadge>, Without<RaceFace>, Without<ClassBadge>),
    >,
    mut class_badges: Query<
        &mut ImageNode,
        (With<ClassBadge>, Without<RaceFace>, Without<RaceBadge>),
    >,
    mut pool: Query<
        (Entity, &PoolIcon, &mut SpellIcon, &mut Node),
        (Without<NoticeRoot>, Without<AppearanceRoot>),
    >,
    mut copy: Query<(&MenuCopy, &mut Text, &mut TextColor), Without<NoticeText>>,
    mut modals: Query<
        (&mut Node, Has<NoticeRoot>, Has<AppearanceRoot>),
        Or<(With<NoticeRoot>, With<AppearanceRoot>)>,
    >,
    mut notice: Query<&mut Text, (With<NoticeText>, Without<MenuCopy>)>,
    mut zone_art: Query<(Entity, &mut UiArt), With<ZoneThumb>>,
) {
    let cur = selection
        .look
        .and_then(|i| selection.looks.get(i))
        .map(|&(_, r, s)| (r, s));
    let class = selection.class_def();
    for (action, interaction, lock, border, background) in &mut buttons {
        let selected = match *action {
            MenuAction::Race(r) => cur.is_some_and(|c| c.0 == r),
            MenuAction::Sex(s) => cur.is_some_and(|c| c.1 == s),
            MenuAction::Class(i) => selection.class == Some(i),
            MenuAction::Map(i) => selection.map == i,
            _ => false,
        };
        if let Some(mut lock) = lock {
            lock.0 = selected;
        }
        if let Some(mut border) = border {
            *border = BorderColor::all(if selected || *interaction != Interaction::None {
                GOLD
            } else {
                Color::srgb(0.39, 0.30, 0.11)
            });
        }
        if let Some(mut bg) = background {
            bg.0 = if selected {
                Color::srgba(0.6, 0.4, 0.12, 0.20)
            } else if *interaction != Interaction::None {
                Color::srgba(0.65, 0.49, 0.24, 0.12)
            } else {
                Color::NONE
            };
        }
    }
    // Keep icon hover labels out of the permanently-labelled class grid.
    for mut visibility in &mut labels {
        *visibility = Visibility::Hidden;
    }
    if let (Some((_, size)), Some((race, sex))) = (&art.races, cur) {
        for (face, mut img) in &mut faces {
            img.rect = race_tc(face.0, sex).map(|tc| tc_rect(*size, tc));
        }
        for mut img in &mut race_badges {
            img.rect = race_tc(race, sex).map(|tc| tc_rect(*size, tc));
        }
    }
    if let (Some((_, size)), Some(c)) = (&art.classes, class) {
        for mut img in &mut class_badges {
            img.rect = class_tc(c.id).map(|tc| tc_rect(*size, tc));
        }
    }
    for (entity, slot, mut icon, mut node) in &mut pool {
        let ability = class.and_then(|c| c.pool.get(slot.0));
        node.display = if ability.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if let Some(id) = ability {
            let spell = data::ability(id).spell;
            if icon.0 != spell {
                icon.0 = spell;
                commands.entity(entity).remove::<super::hud::IconDone>();
            }
        }
    }
    for (mut node, is_notice, is_appearance) in &mut modals {
        node.display = if (is_notice && menu.notice.is_some()) || (is_appearance && menu.appearance)
        {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut text in &mut notice {
        text.0 = menu.notice.clone().unwrap_or_default();
    }
    let map = &data::MAPS[selection.map];
    for (entity, mut art) in &mut zone_art {
        if selection.is_changed() || art.is_added() {
            *art = UiArt::crop(
                if map.map == 1 {
                    "Interface\\Glues\\LoadingScreens\\LoadScreenKalimdor"
                } else {
                    "Interface\\Glues\\LoadingScreens\\LoadScreenEasternKingdom"
                },
                [0.0, 1.0, 0.0, 1.0],
            );
            commands.entity(entity).remove::<super::hud::ArtDone>();
        }
    }
    let roster = data::roster(selection.map);
    for (marker, mut text, mut color) in &mut copy {
        let want = match marker {
            MenuCopy::Name => format!(
                "{}  ⟳",
                localization::text(
                    [
                        "Aeloria", "Thalion", "Sylvara", "Darian", "Morwen", "Kaelan", "Aldric",
                        "Serenna"
                    ][menu.name_index],
                    [
                        "艾洛莉娅",
                        "萨里昂",
                        "希尔瓦拉",
                        "达里安",
                        "莫温",
                        "凯兰",
                        "奥尔德里克",
                        "塞蕾娜"
                    ][menu.name_index]
                )
            ),
            MenuCopy::Champion => match (class, cur) {
                (Some(c), Some((r, s))) => format!(
                    "{} · {} · {}",
                    localization::race_name(r, race_name(r)),
                    localization::class_name(c.id, c.name),
                    if s == 0 {
                        localization::text("Male", "男")
                    } else {
                        localization::text("Female", "女")
                    }
                ),
                _ => localization::text("Choose a class", "请选择职业").into(),
            },
            MenuCopy::Faction => cur.map_or_else(String::new, |(r, _)| {
                if matches!(r, 1 | 3 | 4 | 7) {
                    localization::text("ALLIANCE", "联盟")
                } else {
                    localization::text("HORDE", "部落")
                }
                .into()
            }),
            MenuCopy::ClassName => class.map_or_else(String::new, |c| {
                localization::class_name(c.id, c.name).into()
            }),
            MenuCopy::ClassDetails => {
                class.map_or_else(String::new, |c| class_description(c.id).into())
            }
            MenuCopy::TalentSummary => class.map_or_else(String::new, |c| {
                if super::talents::nodes(c.id).is_some() {
                    format!(
                        "{}: {}/{} · {}/51",
                        localization::text("Spent", "已投入"),
                        progression.spent(c.id),
                        progression.points(c.id),
                        progression.points(c.id)
                    )
                } else {
                    localization::text("Coming soon", "尚未开放").into()
                }
            }),
            MenuCopy::Destination => format!(
                "{}{}",
                localization::text("Enter ", "进入"),
                localization::map_name(map.name)
            ),
            MenuCopy::PersistenceError => progression.error.clone().unwrap_or_default(),
            MenuCopy::Health => class.map_or_else(String::new, |c| {
                let preview = Run {
                    class: c,
                    talents: progression.loadout(c.id),
                    ..Run::default()
                };
                format!("{:.0}", preview.stats().max_hp)
            }),
            MenuCopy::StartingAbility => class.map_or_else(String::new, |c| {
                spells
                    .as_deref()
                    .and_then(|s| s.catalog.get(data::ability(c.start).spell))
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| c.start.replace('_', " "))
            }),
            MenuCopy::ZoneCount => data::MAPS.len().to_string(),
            MenuCopy::RaceName => cur.map_or_else(String::new, |(r, _)| {
                localization::race_name(r, race_name(r)).into()
            }),
            MenuCopy::ZoneName => localization::map_name(map.name).into(),
            MenuCopy::ZoneDetails => {
                let min = roster.enemies.iter().map(|e| e.level).min().unwrap_or(1);
                let max = roster.enemies.iter().map(|e| e.level).max().unwrap_or(min);
                format!(
                    "{} · Lv.{min}–{max}\n{}\n\n{}\n{}",
                    localization::zone_name(map.zone),
                    localization::map_blurb(map.name),
                    localization::text("Boss arrivals", "首领出现时间"),
                    "2:15  ·  4:30  ·  6:45\n9:00  ·  11:15  ·  13:30"
                )
            }
        };
        if text.0 != want {
            text.0 = want;
        }
        color.0 = match marker {
            MenuCopy::Champion | MenuCopy::ClassName => class.map_or(GOLD, |c| rgb(c.color)),
            MenuCopy::PersistenceError => Color::srgb(1.0, 0.38, 0.3),
            MenuCopy::Faction => {
                if cur.is_some_and(|(r, _)| matches!(r, 1 | 3 | 4 | 7)) {
                    Color::srgb(0.35, 0.65, 1.0)
                } else {
                    Color::srgb(1.0, 0.35, 0.25)
                }
            }
            MenuCopy::ClassDetails | MenuCopy::TalentSummary | MenuCopy::ZoneDetails => DIM,
            _ => GOLD,
        };
    }
}

pub(super) fn scroll_zones(
    mut wheel: MessageReader<MouseWheel>,
    phase: Res<State<Phase>>,
    menu: Res<MenuState>,
    talents: Res<super::talents::TalentUi>,
    mut lists: Query<(&Interaction, &ComputedNode, &mut ScrollPosition), With<ZoneList>>,
    mut thumbs: Query<&mut Node, With<ZoneScrollThumb>>,
    rows: Query<(&MenuAction, &Interaction)>,
) {
    for ev in wheel.read() {
        if *phase.get() != Phase::Menu || menu.modal_open() || talents.open {
            continue;
        }
        for (interaction, node, mut pos) in &mut lists {
            if *interaction == Interaction::None
                && !rows
                    .iter()
                    .any(|(a, i)| matches!(a, MenuAction::Map(_)) && *i != Interaction::None)
            {
                continue;
            }
            let max = ((node.content_size.y - node.size.y) * node.inverse_scale_factor).max(0.0);
            let delta = if ev.unit == MouseScrollUnit::Line {
                ev.y * 42.0
            } else {
                ev.y
            };
            pos.0.y = (pos.0.y - delta).clamp(0.0, max);
        }
    }
    for (_, computed, pos) in &lists {
        let ratio = (computed.size.y / computed.content_size.y.max(1.0)).clamp(0.05, 1.0);
        let max =
            ((computed.content_size.y - computed.size.y) * computed.inverse_scale_factor).max(0.0);
        for mut thumb in &mut thumbs {
            thumb.height = Val::Percent(ratio * 100.0);
            thumb.top = Val::Percent(if max > 0.0 {
                pos.0.y / max * (1.0 - ratio) * 100.0
            } else {
                0.0
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;
    #[test]
    fn launcher_builds_nine_classes_and_refreshes_icons_and_modals() {
        let mut app = App::new();
        app.init_resource::<Selection>()
            .init_resource::<GlueArt>()
            .init_resource::<super::super::talents::Progression>()
            .init_resource::<MenuState>()
            .add_systems(Update, refresh_menu_ui);
        app.world_mut()
            .run_system_once(|mut commands: Commands, art: Res<GlueArt>| {
                commands.spawn(Node::default()).with_children(|ui| {
                    layout::build(ui, &art, &Handle::default());
                });
            })
            .unwrap();
        app.update();
        let world = app.world_mut();
        let mut actions = world.query::<&MenuAction>();
        assert_eq!(
            actions
                .iter(world)
                .filter(|a| matches!(a, MenuAction::Class(_)))
                .count(),
            9
        );
        let mut state = world.resource_mut::<MenuState>();
        state.notice = Some("Not yet available".into());
        app.update();
        let world = app.world_mut();
        let mut panels = world.query_filtered::<&Node, With<NoticeRoot>>();
        assert_eq!(panels.single(world).unwrap().display, Display::Flex);
        let warlock = data::CLASSES.iter().position(|c| c.id == 9).unwrap();
        world.resource_mut::<Selection>().class = Some(warlock);
        app.update();
        let world = app.world_mut();
        let mut icons = world.query::<(&PoolIcon, &SpellIcon, &Node)>();
        for (slot, icon, node) in icons.iter(world) {
            if let Some(id) = data::CLASSES[warlock].pool.get(slot.0) {
                assert_eq!(icon.0, data::ability(id).spell);
                assert_eq!(node.display, Display::Flex);
            } else {
                assert_eq!(node.display, Display::None);
            }
        }
    }

    #[test]
    fn modal_actions_do_not_start_or_mutate_a_run() {
        let mut menu = MenuState::default();
        assert!(action_allowed(MenuAction::Enter, &menu));
        menu.appearance = true;
        assert!(!action_allowed(MenuAction::Enter, &menu));
        assert!(!action_allowed(MenuAction::Class(0), &menu));
        assert!(action_allowed(MenuAction::Race(1), &menu));
        menu.notice = Some("Coming soon".into());
        assert!(!action_allowed(MenuAction::Race(1), &menu));
        assert!(!action_allowed(MenuAction::Talents, &menu));
        assert!(action_allowed(MenuAction::Close, &menu));
    }
}
