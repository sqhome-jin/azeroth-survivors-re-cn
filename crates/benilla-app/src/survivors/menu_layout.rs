//! Survivors' launcher chrome; all art is read from the player's existing installation.

use bevy::prelude::*;

use super::hud::{SpellIcon, UiArt, text_bundle};
use super::menu::{MenuAction, MenuCopy, RaceFace};
use super::gear_ui::{MenuSlotButton, MenuSlotIcon};
use super::{data, localization};
use crate::char_select::race_name;
use crate::glue::art::{GOLD, GlueArt, class_tc, race_tc, tc_rect};
use crate::glue::widgets::{LockHighlight, icon_button};

pub(super) const PANEL_TOP: f32 = 94.0;
const EDGE: Color = Color::srgb(0.39, 0.30, 0.11);
const DIM: Color = Color::srgb(0.65, 0.60, 0.43);

#[derive(Component)]
pub(super) struct PoolIcon(pub usize);
#[derive(Component)]
pub(super) struct ClassBadge;
#[derive(Component)]
pub(super) struct RaceBadge;
#[derive(Component)]
pub(super) struct ZoneList;
#[derive(Component)]
pub(super) struct ZoneThumb;
#[derive(Component)]
pub(super) struct ZoneScrollThumb;
#[derive(Component)]
pub(super) struct NoticeRoot;
#[derive(Component)]
pub(super) struct NoticeText;
#[derive(Component)]
pub(super) struct AppearanceRoot;

pub(super) fn abs(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(width),
        height: Val::Px(height),
        ..default()
    }
}

fn copy(
    parent: &mut ChildSpawnerCommands,
    f: &Handle<Font>,
    marker: MenuCopy,
    node: Node,
    size: f32,
    color: Color,
) {
    parent.spawn((marker, node, text_bundle(f, "", size, color)));
}

fn label(
    parent: &mut ChildSpawnerCommands,
    f: &Handle<Font>,
    value: &str,
    node: Node,
    size: f32,
    color: Color,
) {
    let centered = node.width == Val::Percent(100.0);
    parent.spawn((
        node,
        text_bundle(f, value, size, color),
        TextLayout::new_with_justify(if centered {
            Justify::Center
        } else {
            Justify::Left
        }),
    ));
}

const LEATHER: &str = "Interface\\DialogFrame\\UI-DialogBox-Background";

/// The client's tiled leather as a surface under a panel's or button's contents; `alpha` sets how
/// much of the panel colour shows through.
fn grain(parent: &mut ChildSpawnerCommands, alpha: f32, radius: f32) {
    parent.spawn((
        UiArt::tinted(LEATHER, Color::srgba(1.0, 0.92, 0.8, alpha)),
        ImageNode {
            image_mode: NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 128.0,
            },
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(1.0),
            right: Val::Px(1.0),
            top: Val::Px(1.0),
            bottom: Val::Px(1.0),
            border_radius: BorderRadius::all(Val::Px(radius)),
            ..default()
        },
    ));
}

/// A warm light falling from the top of a panel, so the leather beneath shows by contrast.
fn sheen(parent: &mut ChildSpawnerCommands, radius: f32) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(1.0),
            right: Val::Px(1.0),
            top: Val::Px(1.0),
            bottom: Val::Px(1.0),
            border_radius: BorderRadius::all(Val::Px(radius)),
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::percent(Color::srgba(0.78, 0.56, 0.26, 0.20), 0.0),
            ColorStop::percent(Color::srgba(0.55, 0.38, 0.16, 0.08), 45.0),
            ColorStop::percent(Color::srgba(0.35, 0.22, 0.08, 0.14), 100.0),
        ])),
    ));
}

/// A 1 px bright edge along the top of a surface, as a bevel.
fn bevel(parent: &mut ChildSpawnerCommands, alpha: f32) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(3.0),
            right: Val::Px(3.0),
            top: Val::Px(1.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 0.82, 0.4, alpha)),
    ));
}

fn rule(parent: &mut ChildSpawnerCommands, top: f32) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            top: Val::Px(top),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor(EDGE),
    ));
}

fn panel(left: f32, width: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(left),
            width: Val::Percent(width),
            top: Val::Px(PANEL_TOP),
            bottom: Val::Px(130.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.025, 0.020, 0.012, 0.97)),
        BorderColor::all(EDGE),
    )
}

pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    f: &Handle<Font>,
    action: MenuAction,
    title: &str,
    subtitle: Option<&str>,
    node: Node,
) -> Entity {
    parent
        .spawn((
            Button,
            action,
            LockHighlight::default(),
            node,
            BorderColor::all(EDGE),
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::percent(Color::srgb(0.23, 0.055, 0.028), 0.0),
                ColorStop::percent(Color::srgb(0.065, 0.015, 0.01), 100.0),
            ])),
            BackgroundColor(Color::NONE),
        ))
        .with_children(|b| {
            grain(b, 1.0, 3.0);
            // The button's colour goes over its texture so both read.
            b.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(1.0),
                    right: Val::Px(1.0),
                    top: Val::Px(1.0),
                    bottom: Val::Px(1.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundGradient::from(LinearGradient::to_bottom(vec![
                    ColorStop::percent(Color::srgba(0.42, 0.10, 0.04, 0.62), 0.0),
                    ColorStop::percent(Color::srgba(0.10, 0.02, 0.01, 0.55), 100.0),
                ])),
            ));
            bevel(b, 0.45);
            b.spawn(text_bundle(
                f,
                title,
                if subtitle.is_some() { 20.0 } else { 14.0 },
                GOLD,
            ));
            if let Some(subtitle) = subtitle {
                b.spawn(text_bundle(f, subtitle, 10.0, DIM));
            }
        })
        .id()
}

fn button_node(width: Val, height: f32) -> Node {
    Node {
        width,
        height: Val::Px(height),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        ..default()
    }
}

pub(super) fn build(ui: &mut ChildSpawnerCommands, art: &GlueArt, f: &Handle<Font>) {
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.48)),
    ));
    ui.spawn(Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.0),
        width: Val::Percent(100.0),
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(0.0),
        ..default()
    })
    .with_children(|header| {
        header.spawn(text_bundle(
            f,
            localization::text("AZEROTH SURVIVORS", "艾泽拉斯幸存者"),
            42.0,
            GOLD,
        ));
        header
            .spawn(Node {
                column_gap: Val::Px(7.0),
                ..default()
            })
            .with_children(|tabs| {
                for (en, zh, action) in [
                    ("Character", "角色", MenuAction::Character),
                    (
                        "Collection",
                        "收藏",
                        MenuAction::Feature("Collection", "收藏"),
                    ),
                    (
                        "Drops",
                        "掉落表",
                        MenuAction::Feature("Drop table", "掉落表"),
                    ),
                    ("Codex", "图鉴", MenuAction::Feature("Codex", "图鉴")),
                    ("Talents", "天赋", MenuAction::Talents),
                    (
                        "Achievements",
                        "成就",
                        MenuAction::Feature("Achievements", "成就"),
                    ),
                    ("Rewards", "奖励", MenuAction::Feature("Rewards", "奖励")),
                ] {
                    let mut node = button_node(Val::Auto, 27.0);
                    node.padding = UiRect::horizontal(Val::Px(12.0));
                    button(tabs, f, action, localization::text(en, zh), None, node);
                }
            });
    });
    classes(ui, art, f);
    champion(ui, art, f);
    zones(ui, f);
    ui.spawn(Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(48.0),
        width: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        column_gap: Val::Px(16.0),
        ..default()
    })
    .with_children(|footer| {
        button(
            footer,
            f,
            MenuAction::Feature("Difficulty", "难度"),
            localization::text("Difficulty", "难度"),
            Some(localization::text("Normal", "普通难度")),
            button_node(Val::Px(78.0), 46.0),
        );
        let enter = button(
            footer,
            f,
            MenuAction::Enter,
            "",
            None,
            button_node(Val::Px(326.0), 66.0),
        );
        footer.commands().entity(enter).with_children(|b| {
            b.spawn((MenuCopy::Destination, text_bundle(f, "", 27.0, GOLD)));
            b.spawn(text_bundle(
                f,
                localization::text(
                    "Survive 15 minutes · Bosses every 2:15",
                    "存活15分钟 · 每2分15秒迎战首领",
                ),
                10.0,
                DIM,
            ));
        });
        button(
            footer,
            f,
            MenuAction::Feature("Co-op", "合作"),
            localization::text("Co-op", "合作"),
            Some(localization::text("Coming soon", "和好友一起玩")),
            button_node(Val::Px(96.0), 46.0),
        );
    });
    label(
        ui,
        f,
        localization::text(
            "WASD to move · 1 / 2 / 3 choose cards · Enter to begin",
            "WASD或同时按住鼠标双键移动 · 1 / 2 / 3选择卡牌 · 回车开始",
        ),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(26.0),
            width: Val::Percent(100.0),
            ..default()
        },
        11.0,
        DIM,
    );
    modals(ui, art, f);
}

fn classes(ui: &mut ChildSpawnerCommands, art: &GlueArt, f: &Handle<Font>) {
    ui.spawn(panel(0.9, 30.1)).with_children(|p| {
        grain(p, 1.0, 7.0);
        sheen(p, 7.0);
        bevel(p, 0.2);
        rule(p, 23.0);
        label(
            p,
            f,
            localization::text("CLASS", "职业"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                width: Val::Percent(100.0),
                ..default()
            },
            20.0,
            GOLD,
        );
        p.spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Percent(20.0),
            width: Val::Percent(60.0),
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(16.0),
            ..default()
        })
        .with_children(|grid| {
            for (i, c) in data::CLASSES.iter().enumerate() {
                grid.spawn(Node {
                    width: Val::Percent(33.333),
                    height: Val::Px(64.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|cell| {
                    let face = art.classes.as_ref().and_then(|(sheet, size)| {
                        class_tc(c.id).map(|tc| (sheet.clone(), tc_rect(*size, tc)))
                    });
                    icon_button(
                        cell,
                        f,
                        MenuAction::Class(i),
                        None::<()>,
                        face,
                        None::<()>,
                        localization::class_name(c.id, c.name),
                        art,
                        1.0,
                    );
                    cell.spawn(text_bundle(
                        f,
                        localization::class_name(c.id, c.name),
                        11.0,
                        DIM,
                    ));
                });
            }
        });
        rule(p, 288.0);
        copy(
            p,
            f,
            MenuCopy::ClassName,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(300.0),
                ..default()
            },
            22.0,
            GOLD,
        );
        copy(
            p,
            f,
            MenuCopy::ClassDetails,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                right: Val::Px(16.0),
                top: Val::Px(332.0),
                ..default()
            },
            13.0,
            DIM,
        );
        p.spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            top: Val::Px(376.0),
            height: Val::Px(41.0),
            column_gap: Val::Px(6.0),
            ..default()
        })
        .with_children(|tiles| {
            for (marker, en, zh) in [
                (MenuCopy::Health, "Health", "生命值"),
                (MenuCopy::StartingAbility, "Starting ability", "初始技能"),
                (MenuCopy::ZoneCount, "Zones", "可选区域"),
            ] {
                tiles
                    .spawn((
                        Node {
                            flex_grow: 1.0,
                            flex_basis: Val::Px(0.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BorderColor::all(EDGE),
                        BackgroundColor(Color::BLACK),
                    ))
                    .with_children(|tile| {
                        tile.spawn(text_bundle(f, localization::text(en, zh), 9.0, DIM));
                        copy(tile, f, marker, Node::default(), 12.0, GOLD);
                    });
            }
        });
        label(
            p,
            f,
            localization::text("Spell pool", "法术池"),
            abs(16.0, 428.0, 100.0, 16.0),
            10.0,
            DIM,
        );
        p.spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            top: Val::Px(448.0),
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(5.0),
            row_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|pool| {
            for i in 0..12 {
                pool.spawn((
                    PoolIcon(i),
                    SpellIcon(686),
                    ImageNode::default(),
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                ));
            }
        });
    });
}

fn champion(ui: &mut ChildSpawnerCommands, art: &GlueArt, f: &Handle<Font>) {
    // The preview cut-out reveals the existing live world hero, framed by menu::frame_preview.
    ui.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(32.5),
            width: Val::Percent(35.0),
            top: Val::Px(PANEL_TOP),
            height: Val::Px(178.0),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BorderColor::all(EDGE),
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(12.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.020, 0.012, 0.97)),
        ));
        // The hero is filmed on its own layer (see preview.rs) over a dark stage, then darkened
        // toward the edges and warmed toward the ground, like a stage under a spot.
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                width: Val::Px(110.0),
                top: Val::Px(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.035, 0.026, 0.018)),
        ));
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                width: Val::Px(110.0),
                top: Val::Px(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ImageNode::default().with_color(Color::NONE),
            super::preview::HeroPreviewView,
        ));
        for (angle, stops) in [
            (
                LinearGradient::to_bottom as fn(Vec<ColorStop>) -> LinearGradient,
                vec![
                    ColorStop::percent(Color::srgba(0.02, 0.012, 0.008, 0.88), 0.0),
                    ColorStop::percent(Color::srgba(0.05, 0.03, 0.02, 0.18), 38.0),
                    ColorStop::percent(Color::srgba(0.12, 0.07, 0.03, 0.18), 70.0),
                    ColorStop::percent(Color::srgba(0.30, 0.17, 0.05, 0.72), 100.0),
                ],
            ),
            (
                LinearGradient::to_right as fn(Vec<ColorStop>) -> LinearGradient,
                vec![
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.6), 0.0),
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.0), 30.0),
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.0), 70.0),
                    ColorStop::percent(Color::srgba(0.0, 0.0, 0.0, 0.6), 100.0),
                ],
            ),
        ] {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(12.0),
                    width: Val::Px(110.0),
                    top: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundGradient::from(angle(stops)),
            ));
        }
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                width: Val::Px(110.0),
                top: Val::Px(0.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(Color::srgba(0.75, 0.58, 0.2, 0.55)),
        ));
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(122.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(14.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.020, 0.012, 0.97)),
        ))
        .with_children(|info| {
            grain(info, 1.0, 0.0);
            sheen(info, 0.0);
            let name = button(
                info,
                f,
                MenuAction::RandomName,
                "",
                None,
                button_node(Val::Percent(100.0), 36.0),
            );
            info.commands().entity(name).with_children(|b| {
                b.spawn((MenuCopy::Name, text_bundle(f, "", 21.0, GOLD)));
            });
            copy(info, f, MenuCopy::Champion, Node::default(), 16.0, GOLD);
            button(
                info,
                f,
                MenuAction::Appearance,
                localization::text("Race / appearance", "种族 / 外观"),
                None,
                button_node(Val::Percent(100.0), 25.0),
            );
        });
        copy(
            p,
            f,
            MenuCopy::Faction,
            abs(15.0, 12.0, 44.0, 15.0),
            9.0,
            GOLD,
        );
        let face = art
            .classes
            .as_ref()
            .and_then(|(sheet, size)| class_tc(9).map(|tc| (sheet.clone(), tc_rect(*size, tc))));
        if let Some((image, rect)) = face {
            p.spawn((
                ClassBadge,
                ImageNode {
                    image,
                    rect: Some(rect),
                    ..default()
                },
                abs(90.0, 129.0, 36.0, 36.0),
            ));
        }
    });
    ui.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(32.5), width: Val::Percent(35.0),
        top: Val::Px(272.0), bottom: Val::Px(130.0), border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(8.0)), ..default() },
        BackgroundColor(Color::srgba(0.025, 0.020, 0.012, 0.97)), BorderColor::all(EDGE),
    )).with_children(|p| {
        grain(p, 1.0, 7.0);
        sheen(p, 7.0);
        bevel(p, 0.2);
        rule(p, 10.0);
        label(p, f, localization::text("EQUIPMENT", "装备"), Node { position_type: PositionType::Absolute,
            top: Val::Px(-5.0), width: Val::Percent(100.0), ..default() }, 20.0, GOLD);
        p.spawn(Node { position_type: PositionType::Absolute, left: Val::Px(16.0), right: Val::Px(16.0),
            top: Val::Px(28.0), justify_content: JustifyContent::Center, column_gap: Val::Px(4.0), ..default()
        }).with_children(|slots| {
            for index in 0..super::gear::SLOT_COUNT {
                slots.spawn((Button, MenuAction::Gear, MenuSlotButton(index), Node {
                    width: Val::Percent(10.0), aspect_ratio: Some(1.0),
                    border: UiRect::all(Val::Px(1.0)), ..default()
                }, BorderColor::all(EDGE))).with_children(|slot| {
                    slot.spawn((MenuSlotIcon(index), ImageNode::default(),
                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
                });
            }
        });
        label(p, f, localization::text("Gear", "装备"), abs(16.0, 74.0, 100.0, 14.0), 10.0, DIM);
        p.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(16.0), right: Val::Px(16.0),
            top: Val::Px(92.0), height: Val::Px(4.0), ..default() }, BackgroundColor(EDGE)));
        label(p, f, localization::text("Gear drops from kills and is kept. Click a slot to manage it.", "装备由击杀掉落并永久保留，点击装备栏进行管理。"),
            Node { position_type: PositionType::Absolute, left: Val::Px(16.0), top: Val::Px(103.0), right: Val::Px(16.0), ..default() }, 11.0, DIM);
        p.spawn(Node { position_type: PositionType::Absolute, top: Val::Px(130.0), left: Val::Px(16.0), right: Val::Px(16.0),
            column_gap: Val::Px(8.0), ..default()
        }).with_children(|buttons| {
            button(buttons, f, MenuAction::Gear, localization::text("Armory", "军械库"),
                Some(localization::text("Wear and manage the gear you have found", "穿戴与管理已获得的装备")), button_node(Val::Percent(50.0), 49.0));
            let talent = button(buttons, f, MenuAction::Talents, localization::text("Talents", "天赋"), None,
                button_node(Val::Percent(50.0), 49.0));
            buttons.commands().entity(talent).with_children(|b| { b.spawn((MenuCopy::TalentSummary, text_bundle(f, "", 10.0, DIM))); });
        });
        rule(p, 190.0);
        if let Some((sheet, size)) = &art.races {
            if let Some(tc) = race_tc(1, 0) {
                p.spawn((RaceBadge, ImageNode { image: sheet.clone(), rect: Some(tc_rect(*size, tc)), ..default() },
                    abs(16.0, 205.0, 36.0, 36.0)));
            }
        }
        copy(p, f, MenuCopy::RaceName, Node { position_type: PositionType::Absolute, left: Val::Px(63.0),
            top: Val::Px(203.0), right: Val::Px(16.0), ..default() }, 20.0, GOLD);
        label(p, f, localization::text("• Appearance selection only; racial bonuses are not yet available.\n• Free talent respec outside a run.\n• Each class keeps its own kill progress.",
            "• 当前种族仅影响外观，种族特长尚未开放。\n• 局外天赋可免费重置。\n• 击杀成长按职业独立保存。"),
            Node { position_type: PositionType::Absolute, left: Val::Px(63.0), right: Val::Px(16.0),
                top: Val::Px(232.0), ..default() }, 12.0, DIM);
        copy(p, f, MenuCopy::PersistenceError, Node { position_type: PositionType::Absolute,
            left: Val::Px(16.0), right: Val::Px(16.0), bottom: Val::Px(12.0), ..default() }, 12.0, Color::srgb(1.0, 0.3, 0.2));
    });
}

fn zones(ui: &mut ChildSpawnerCommands, f: &Handle<Font>) {
    ui.spawn(panel(69.0, 30.1)).with_children(|p| {
        grain(p, 1.0, 7.0);
        sheen(p, 7.0);
        bevel(p, 0.2);
        rule(p, 23.0);
        label(
            p,
            f,
            localization::text("ZONE", "区域"),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                width: Val::Percent(100.0),
                ..default()
            },
            20.0,
            GOLD,
        );
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                right: Val::Px(28.0),
                top: Val::Px(45.0),
                height: Val::Px(170.0),
                overflow: Overflow::clip(),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                ..default()
            },
            BorderColor::all(GOLD),
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|card| {
            card.spawn((
                ZoneThumb,
                UiArt::crop(
                    "Interface\\Glues\\LoadingScreens\\LoadScreenEasternKingdom",
                    [0.0, 1.0, 0.0, 1.0],
                ),
                ImageNode::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            ));
            copy(
                card,
                f,
                MenuCopy::ZoneName,
                abs(12.0, 8.0, 350.0, 27.0),
                23.0,
                GOLD,
            );
            copy(
                card,
                f,
                MenuCopy::ZoneDetails,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                    top: Val::Px(40.0),
                    ..default()
                },
                11.0,
                DIM,
            );
        });
        p.spawn((
            ZoneList,
            Interaction::default(),
            ScrollPosition::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                right: Val::Px(28.0),
                top: Val::Px(224.0),
                bottom: Val::Px(12.0),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ))
        .with_children(|list| {
            let mut order: Vec<_> = (0..data::MAPS.len()).collect();
            order.sort_by_key(|&i| {
                (
                    data::roster(i)
                        .enemies
                        .iter()
                        .map(|e| e.level)
                        .min()
                        .unwrap_or(1),
                    data::MAPS[i].name,
                )
            });
            for i in order {
                let map = &data::MAPS[i];
                list.spawn((
                    Button,
                    MenuAction::Map(i),
                    LockHighlight::default(),
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(40.0),
                        flex_shrink: 0.0,
                        padding: UiRect::axes(Val::Px(9.0), Val::Px(4.0)),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
                ))
                .with_children(|row| {
                    grain(row, 0.85, 3.0);
                    row.spawn(text_bundle(f, localization::map_name(map.name), 16.0, GOLD));
                    row.spawn(text_bundle(
                        f,
                        format!(
                            "{} · {}",
                            localization::zone_name(map.zone),
                            localization::text("Available", "可进入")
                        ),
                        10.0,
                        DIM,
                    ));
                });
            }
        });
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(14.0),
                top: Val::Px(45.0),
                bottom: Val::Px(12.0),
                width: Val::Px(5.0),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.10, 0.08, 0.03)),
        ))
        .with_children(|track| {
            track.spawn((
                ZoneScrollThumb,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(35.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(GOLD),
            ));
        });
    });
}

fn modals(ui: &mut ChildSpawnerCommands, art: &GlueArt, f: &Handle<Font>) {
    ui.spawn((
        NoticeRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.78)),
        GlobalZIndex(450),
    ))
    .with_children(|shade| {
        shade
            .spawn((
                Node {
                    width: Val::Px(410.0),
                    padding: UiRect::all(Val::Px(26.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(22.0),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BorderColor::all(GOLD),
                BackgroundColor(Color::srgb(0.04, 0.025, 0.015)),
            ))
            .with_children(|modal| {
                modal.spawn((NoticeText, text_bundle(f, "", 20.0, GOLD)));
                button(
                    modal,
                    f,
                    MenuAction::Close,
                    localization::text("Back", "返回"),
                    None,
                    button_node(Val::Px(100.0), 32.0),
                );
            });
    });
    ui.spawn((
        AppearanceRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        GlobalZIndex(450),
    ))
    .with_children(|shade| {
        shade
            .spawn((
                Node {
                    width: Val::Px(390.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(GOLD),
                BackgroundColor(Color::srgb(0.04, 0.025, 0.015)),
            ))
            .with_children(|p| {
                p.spawn(text_bundle(
                    f,
                    localization::text("Race and appearance", "种族与外观"),
                    24.0,
                    GOLD,
                ));
                for races in [[1, 3, 4, 7], [2, 5, 6, 8]] {
                    p.spawn(Node {
                        column_gap: Val::Px(22.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for race in races {
                            let face = art.races.as_ref().and_then(|(sheet, size)| {
                                race_tc(race, 0).map(|tc| (sheet.clone(), tc_rect(*size, tc)))
                            });
                            row.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                ..default()
                            })
                            .with_children(|cell| {
                                icon_button(
                                    cell,
                                    f,
                                    MenuAction::Race(race),
                                    Some(RaceFace(race)),
                                    face,
                                    None::<()>,
                                    localization::race_name(race, race_name(race)),
                                    art,
                                    1.0,
                                );
                                cell.spawn(text_bundle(
                                    f,
                                    localization::race_name(race, race_name(race)),
                                    11.0,
                                    DIM,
                                ));
                            });
                        }
                    });
                }
                p.spawn(Node {
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    button(
                        row,
                        f,
                        MenuAction::Sex(0),
                        localization::text("Male", "男性"),
                        None,
                        button_node(Val::Px(90.0), 32.0),
                    );
                    button(
                        row,
                        f,
                        MenuAction::Sex(1),
                        localization::text("Female", "女性"),
                        None,
                        button_node(Val::Px(90.0), 32.0),
                    );
                });
                p.spawn(text_bundle(
                    f,
                    localization::text(
                        "Changing race may select a compatible class.",
                        "切换种族可能自动选择兼容职业。",
                    ),
                    12.0,
                    DIM,
                ));
                button(
                    p,
                    f,
                    MenuAction::Randomize,
                    localization::text("Random appearance", "随机角色"),
                    None,
                    button_node(Val::Px(140.0), 32.0),
                );
                button(
                    p,
                    f,
                    MenuAction::Close,
                    localization::text("Done", "完成"),
                    None,
                    button_node(Val::Px(100.0), 32.0),
                );
            });
    });
}
