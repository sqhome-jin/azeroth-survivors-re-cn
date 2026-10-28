//! The gear panel: the worn set, the shared backpack and an item's rolls. It opens from the menu
//! and, paused, from a run; the launcher's slot row mirrors what the selected class wears.

use bevy::prelude::*;

use super::gear::{self, Gear, SLOT_ART, SLOT_COUNT};
use super::hud::{HudFont, text_bundle};
use super::{Phase, Run, Selection, localization};

const GOLD: Color = Color::srgb(0.94, 0.76, 0.34);
const EDGE: Color = Color::srgb(0.42, 0.32, 0.12);
const DIM: Color = Color::srgb(0.65, 0.60, 0.43);
const PAGE: usize = 64;
const COLUMNS: usize = 8;
const LINES: usize = 15;

#[derive(Resource, Default)]
pub(super) struct GearUi {
    pub open: bool,
    selected: Option<u64>,
    page: usize,
    message: String,
}

#[derive(Component)]
struct GearRoot;

#[derive(Component, Clone, Copy)]
enum Action {
    Slot(usize),
    Cell(usize),
    Equip,
    Unequip,
    Discard,
    Prev,
    Next,
    CleanCommon,
    CleanUncommon,
    Close,
}

#[derive(Component)]
struct CellIcon(usize);

#[derive(Component)]
struct SlotIcon(usize);

#[derive(Component)]
struct Line(usize);

#[derive(Component)]
struct PageText;

#[derive(Component)]
struct CellButton(usize);

#[derive(Component)]
struct SlotButton(usize);

/// A slot on the launcher's equipment panel, which shows the selected class's worn item.
#[derive(Component)]
pub(super) struct MenuSlotIcon(pub usize);

#[derive(Component)]
pub(super) struct MenuSlotButton(pub usize);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GearUi>().add_systems(
        Update,
        (build_panel, panel_input, refresh_panel, refresh_menu_slots)
            .chain()
            .after(super::menu::menu_clicks),
    );
}

fn current_class(phase: Phase, selection: &Selection, run: &Run) -> u8 {
    if phase == Phase::Menu {
        selection.class_def().map_or(run.class.id, |c| c.id)
    } else {
        run.class.id
    }
}

fn button(parent: &mut ChildSpawnerCommands, font: &Handle<Font>, action: Action, label: &str) {
    parent
        .spawn((
            Button,
            action,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                align_self: AlignSelf::Start,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.055, 0.035)),
            BorderColor::all(EDGE),
        ))
        .with_children(|p| {
            p.spawn(text_bundle(font, label, 15.0, GOLD));
        });
}

fn icon_box(size: f32) -> Node {
    Node {
        width: Val::Px(size),
        height: Val::Px(size),
        border: UiRect::all(Val::Px(2.0)),
        ..default()
    }
}

fn build_panel(
    mut commands: Commands,
    font: Option<Res<HudFont>>,
    roots: Query<(), With<GearRoot>>,
) {
    if !roots.is_empty() {
        return;
    }
    let Some(font) = font else { return };
    let f = &font.0;
    commands
        .spawn((
            GearRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(470),
            BackgroundColor(Color::srgba(0.015, 0.012, 0.008, 0.98)),
        ))
        .with_children(|root| {
            root.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    })
                    .with_children(|title| {
                        title.spawn(text_bundle(
                            f,
                            localization::text("EQUIPMENT", "装备"),
                            32.0,
                            GOLD,
                        ));
                        title.spawn(text_bundle(
                            f,
                            localization::text(
                                "Drops from kills · random affixes · shared backpack, each class wears its own set",
                                "击杀掉落 · 随机词缀 · 背包共享，每个职业独立穿戴",
                            ),
                            12.0,
                            DIM,
                        ));
                    });
                header
                    .spawn(Node {
                        column_gap: Val::Px(10.0),
                        ..default()
                    })
                    .with_children(|buttons| {
                        button(
                            buttons,
                            f,
                            Action::CleanCommon,
                            localization::text("Discard Common", "丢弃普通"),
                        );
                        button(
                            buttons,
                            f,
                            Action::CleanUncommon,
                            localization::text("Discard ≤ Uncommon", "丢弃优秀及以下"),
                        );
                        button(buttons, f, Action::Close, localization::text("Back", "返回"));
                    });
            });
            root.spawn(Node {
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                column_gap: Val::Px(14.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: Val::Px(230.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.035, 0.027, 0.017)),
                    BorderColor::all(EDGE),
                ))
                .with_children(|col| {
                    for slot in 0..SLOT_COUNT {
                        let (en, zh) = gear::SLOT_NAMES[slot];
                        col.spawn(Node {
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(10.0),
                            ..default()
                        })
                        .with_children(|r| {
                            r.spawn((
                                Button,
                                Action::Slot(slot),
                                SlotButton(slot),
                                icon_box(48.0),
                                BackgroundColor(Color::BLACK),
                                BorderColor::all(EDGE),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    SlotIcon(slot),
                                    ImageNode::default(),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                ));
                            });
                            r.spawn(text_bundle(f, localization::text(en, zh), 15.0, GOLD));
                        });
                    }
                });
                row.spawn((
                    Node {
                        width: Val::Px(COLUMNS as f32 * 52.0 + 20.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.035, 0.027, 0.017)),
                    BorderColor::all(EDGE),
                ))
                .with_children(|col| {
                    col.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(4.0),
                        row_gap: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|grid| {
                        for cell in 0..PAGE {
                            grid.spawn((
                                Button,
                                Action::Cell(cell),
                                CellButton(cell),
                                icon_box(48.0),
                                BackgroundColor(Color::BLACK),
                                BorderColor::all(EDGE),
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    CellIcon(cell),
                                    ImageNode::default(),
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                ));
                            });
                        }
                    });
                    col.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(12.0),
                        ..default()
                    })
                    .with_children(|pager| {
                        button(pager, f, Action::Prev, "<");
                        pager.spawn((PageText, text_bundle(f, "", 14.0, DIM)));
                        button(pager, f, Action::Next, ">");
                    });
                });
                row.spawn((
                    Node {
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(5.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.035, 0.027, 0.017)),
                    BorderColor::all(EDGE),
                ))
                .with_children(|col| {
                    for i in 0..LINES {
                        let size = if i == 0 { 22.0 } else { 14.0 };
                        col.spawn((Line(i), text_bundle(f, "", size, Color::WHITE)));
                    }
                    col.spawn(Node {
                        column_gap: Val::Px(10.0),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|buttons| {
                        button(buttons, f, Action::Equip, localization::text("Equip", "装备"));
                        button(buttons, f, Action::Unequip, localization::text("Unequip", "卸下"));
                        button(buttons, f, Action::Discard, localization::text("Discard", "丢弃"));
                    });
                });
            });
        });
}

/// Backpack order: rarest first, newest first within a rarity.
fn sorted_ids(gear: &Gear) -> Vec<u64> {
    let mut items: Vec<_> = gear.bag().iter().map(|i| (i.rarity, i.id)).collect();
    items.sort_by(|a, b| b.cmp(a));
    items.into_iter().map(|(_, id)| id).collect()
}

fn pages(len: usize) -> usize {
    len.div_ceil(PAGE).max(1)
}

#[allow(clippy::too_many_arguments)]
fn panel_input(
    mut ui: ResMut<GearUi>,
    mut gear: ResMut<Gear>,
    mut run: ResMut<Run>,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    presses: Query<(&Interaction, &Action), Changed<Interaction>>,
) {
    if !matches!(phase.get(), Phase::Menu | Phase::Paused) {
        ui.open = false;
    }
    if !ui.open {
        return;
    }
    let class = current_class(*phase.get(), &selection, &run);
    let ids = sorted_ids(&gear);
    for (interaction, action) in &presses {
        if *interaction != Interaction::Pressed {
            continue;
        }
        ui.message.clear();
        match *action {
            Action::Slot(slot) => {
                ui.selected = gear.worn(class, slot).map(|i| i.id).or(ui.selected);
            }
            Action::Cell(cell) => {
                ui.selected = ids.get(ui.page * PAGE + cell).copied().or(ui.selected);
            }
            Action::Equip => {
                if let Some(id) = ui.selected {
                    if !gear.equip(class, id) {
                        ui.message = localization::text(
                            "Cannot equip: save is read-only.",
                            "无法穿戴：存档处于只读保护状态。",
                        )
                        .into();
                    }
                }
            }
            Action::Unequip => {
                if let Some(slot) = ui
                    .selected
                    .and_then(|id| gear.item(id))
                    .map(|i| usize::from(i.slot))
                {
                    if gear
                        .worn(class, slot)
                        .is_some_and(|w| Some(w.id) == ui.selected)
                    {
                        gear.unequip(class, slot);
                    }
                }
            }
            Action::Discard => {
                if let Some(id) = ui.selected.take() {
                    gear.discard(id);
                }
            }
            Action::Prev => ui.page = ui.page.saturating_sub(1),
            Action::Next => ui.page = (ui.page + 1).min(pages(ids.len()) - 1),
            Action::CleanCommon | Action::CleanUncommon => {
                let top = u8::from(matches!(action, Action::CleanUncommon));
                let gone = gear.discard_up_to(top);
                ui.message = format!("{}{gone}", localization::text("Discarded: ", "已丢弃："));
            }
            Action::Close => ui.open = false,
        }
    }
    let len = gear.bag().len();
    ui.page = ui.page.min(pages(len) - 1);
    if ui.selected.is_some_and(|id| gear.item(id).is_none()) {
        ui.selected = None;
    }
    // Keep the hero's numbers current in the paused run.
    if *phase.get() == Phase::Paused {
        run.gear = gear.bonuses(class);
    }
}

fn set_icon(
    node: &mut ImageNode,
    path: Option<&str>,
    assets: &mut Option<ResMut<benilla_assets::WorldAssets>>,
    images: &mut Assets<Image>,
) {
    let Some(path) = path else {
        node.color = Color::NONE;
        return;
    };
    let Some(assets) = assets.as_mut() else {
        return;
    };
    if let Some(handle) = assets.sprite_texture(path, images) {
        if node.image != handle {
            node.image = handle;
        }
        node.color = Color::WHITE;
    }
}

fn tint(color: Color, scale: f32) -> Color {
    let c = color.to_srgba();
    Color::srgb(c.red * scale, c.green * scale, c.blue * scale)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn refresh_panel(
    ui: Res<GearUi>,
    gear: Res<Gear>,
    run: Res<Run>,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut root: Query<&mut Node, With<GearRoot>>,
    mut cells: ParamSet<(
        Query<(&CellButton, &mut BorderColor, &mut BackgroundColor)>,
        Query<(&CellIcon, &mut ImageNode)>,
        Query<(&SlotButton, &mut BorderColor, &mut BackgroundColor)>,
        Query<(&SlotIcon, &mut ImageNode)>,
    )>,
    mut texts: ParamSet<(
        Query<(&Line, &mut Text, &mut TextColor)>,
        Query<&mut Text, With<PageText>>,
    )>,
) {
    let Ok(mut node) = root.single_mut() else {
        return;
    };
    let want = if ui.open {
        Display::Flex
    } else {
        Display::None
    };
    if node.display != want {
        node.display = want;
    }
    if !ui.open {
        return;
    }
    let class = current_class(*phase.get(), &selection, &run);
    let ids = sorted_ids(&gear);
    let at = |cell: usize| ids.get(ui.page * PAGE + cell).and_then(|id| gear.item(*id));
    let edge = |item: Option<&gear::Item>, selected: bool| {
        if selected {
            Color::WHITE
        } else {
            item.map_or(EDGE, gear::Item::color)
        }
    };

    for (b, mut border, mut bg) in &mut cells.p0() {
        let item = at(b.0);
        *border = BorderColor::all(edge(item, item.is_some_and(|i| Some(i.id) == ui.selected)));
        bg.0 = match item {
            Some(i) if gear.wearer(i.id).is_some() => Color::srgb(0.12, 0.24, 0.08),
            Some(_) => Color::srgb(0.07, 0.06, 0.05),
            None => Color::BLACK,
        };
    }
    for (c, mut img) in &mut cells.p1() {
        set_icon(
            &mut img,
            at(c.0).map(gear::Item::icon),
            &mut assets,
            &mut images,
        );
    }
    for (b, mut border, mut bg) in &mut cells.p2() {
        let item = gear.worn(class, b.0);
        *border = BorderColor::all(edge(item, item.is_some_and(|i| Some(i.id) == ui.selected)));
        bg.0 = tint(item.map_or(Color::BLACK, gear::Item::color), 0.18);
    }
    for (s, mut img) in &mut cells.p3() {
        let worn = gear.worn(class, s.0);
        set_icon(
            &mut img,
            Some(worn.map_or(SLOT_ART[s.0], gear::Item::icon)),
            &mut assets,
            &mut images,
        );
        if worn.is_none() {
            img.color = Color::srgba(1.0, 1.0, 1.0, 0.45);
        }
    }
    if let Ok(mut page) = texts.p1().single_mut() {
        page.0 = format!(
            "{} / {}   ·   {} / {}",
            ui.page + 1,
            pages(ids.len()),
            ids.len(),
            gear::BAG_CAP
        );
    }

    let mut lines: Vec<(String, Color)> = Vec::new();
    if let Some(item) = ui.selected.and_then(|id| gear.item(id)) {
        lines.push((item.name().to_owned(), item.color()));
        lines.push((
            format!("{} · {}", item.rarity_name(), item.slot_name()),
            DIM,
        ));
        for roll in &item.affixes {
            lines.push((gear::describe(roll), Color::srgb(0.75, 0.85, 1.0)));
        }
        if let Some(enh) = &item.enhancement {
            let name = data_name(spells.as_deref(), &enh.ability);
            lines.push((
                gear::describe_enhancement(enh, &name),
                gear::RARITY_COLORS[5],
            ));
        }
        let note = match gear.wearer(item.id) {
            Some(c) if c == class => localization::text("Worn by this class", "当前职业已穿戴"),
            Some(_) => localization::text(
                "Worn by another class; equipping moves it here",
                "已被其他职业穿戴，穿戴后将转移到当前职业",
            ),
            None => "",
        };
        lines.push((note.to_owned(), Color::srgb(0.6, 0.9, 0.5)));
    } else {
        lines.push((
            localization::text("Select an item", "选择一件装备").to_owned(),
            DIM,
        ));
    }
    let message = if ui.message.is_empty() {
        gear.error.clone().unwrap_or_default()
    } else {
        ui.message.clone()
    };
    for (line, mut text, mut color) in &mut texts.p0() {
        let (s, c) = if line.0 == LINES - 1 {
            (message.clone(), Color::srgb(1.0, 0.3, 0.2))
        } else {
            lines.get(line.0).cloned().unwrap_or_default()
        };
        if text.0 != s {
            text.0 = s;
        }
        color.0 = c;
    }
}

fn data_name(spells: Option<&crate::ui_action::Spells>, key: &str) -> String {
    super::data::ABILITIES
        .iter()
        .find(|a| a.key == key)
        .map_or_else(
            || key.replace('_', " "),
            |a| super::hud::spell_name(spells, a.spell, a.key),
        )
}

/// Mirrors the selected class's worn items onto the launcher's slot row.
#[allow(clippy::type_complexity)]
fn refresh_menu_slots(
    gear: Res<Gear>,
    run: Res<Run>,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut borders: Query<(&MenuSlotButton, &mut BorderColor)>,
    mut icons: Query<(&MenuSlotIcon, &mut ImageNode)>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    let class = current_class(Phase::Menu, &selection, &run);
    for (b, mut border) in &mut borders {
        let color = gear.worn(class, b.0).map_or(EDGE, gear::Item::color);
        *border = BorderColor::all(color);
    }
    for (s, mut img) in &mut icons {
        let worn = gear.worn(class, s.0);
        set_icon(
            &mut img,
            Some(worn.map_or(SLOT_ART[s.0], gear::Item::icon)),
            &mut assets,
            &mut images,
        );
    }
}
