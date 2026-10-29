//! The mode's screens, in bevy_ui since FrameXML is off: the class and hero pick, the in-run HUD,
//! the level-up cards, pause, and the results.

use std::collections::HashMap;

use benilla_ui::script::nameplate;
use benilla_world::view::WorldCamera;
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use bevy::ui_render::ui_material::{MaterialNode, UiMaterial};
use bevy::window::PrimaryWindow;

use super::cards::Rarity;
use super::data::{self, AbilityDef, Effect, Kind, PassiveDef, Stat};
use super::hero::{self, Hero};
use super::localization;
use super::minimap::{self, MiniMap};
use super::skill_descriptions;
use super::{Owned, Phase, Rng, Run, Selection, Stats};

const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);
/// The mobility button's edge, an action button's 36 units.
const MOBILITY_SIZE: f32 = 40.0;
const EDGE: Color = Color::srgb(0.55, 0.45, 0.25);
const BUTTON: Color = Color::srgba(0.18, 0.06, 0.04, 0.95);
const BUTTON_HOT: Color = Color::srgba(0.36, 0.12, 0.06, 0.98);
const DIM: Color = Color::srgb(0.6, 0.6, 0.6);
pub(super) const DIM_GREY: Color = DIM;
pub(super) const GOLD_TONE: Color = GOLD;

#[derive(Component)]
pub(super) struct MenuRoot;
#[derive(Component)]
pub(super) struct HudRoot;
#[derive(Component)]
pub(super) struct LevelUpRoot;
#[derive(Component)]
pub(super) struct PauseRoot;
#[derive(Component)]
pub(super) struct OverRoot;
#[derive(Component)]
pub(super) struct XpBar;
#[derive(Component)]
pub(super) struct XpFill;
#[derive(Component)]
pub(super) struct XpText;
#[derive(Component)]
pub(super) struct LevelText;
#[derive(Component)]
pub(super) struct TimerText;
#[derive(Component)]
pub(super) struct ObjectiveText;
#[derive(Component)]
pub(super) struct KillText;
#[derive(Component)]
pub(super) struct HpFrame;
#[derive(Component)]
pub(super) struct HpBar;
#[derive(Component)]
pub(super) struct HpLevelSeat;
#[derive(Component)]
pub(super) struct HpLevel;
#[derive(Component)]
pub(super) struct HpFill;
#[derive(Component)]
pub(super) struct ShieldFill;
#[derive(Component)]
pub(super) struct BannerText;
#[derive(Component)]
pub(super) struct AbilityRow;
#[derive(Component)]
pub(super) struct PassiveRow;
#[derive(Component)]
pub(super) struct OfferRow;
/// The line under LEVEL UP: which level this is.
#[derive(Component)]
pub(super) struct LevelUpSub;

/// A radial cooldown sweep over one ability icon: `sweep` is the remaining fraction
/// (1.0 = just cast, 0.0 = ready), drawn by `ui_sweep.wgsl`.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub(super) struct SweepUiMaterial {
    #[uniform(0)]
    pub(super) sweep: f32,
}

impl UiMaterial for SweepUiMaterial {
    fn fragment_shader() -> ShaderRef {
        "embedded://benilla_app/shaders/ui_sweep.wgsl".into()
    }
}

/// The cooldown overlay on one ability slot, driven every frame.
#[derive(Component)]
pub(super) struct AbilitySweep {
    handle: Handle<SweepUiMaterial>,
    index: usize,
}

/// The class mobility's action button, bottom centre over the XP bar; shown once it is learned.
#[derive(Component)]
pub(super) struct MobilitySlot;
#[derive(Component)]
pub(super) struct MobilityIcon;
#[derive(Component)]
pub(super) struct MobilitySweep(Handle<SweepUiMaterial>);
#[derive(Component)]
pub(super) struct OverTitle;
#[derive(Component)]
pub(super) struct OverStats;
#[derive(Component)]
pub(super) struct OverLine;
#[derive(Component)]
pub(super) struct OverStat(usize);

/// An icon slot filled from the spell's `SpellIcon` once the client data is up.
#[derive(Component)]
pub(super) struct SpellIcon(pub(super) u32);
#[derive(Component)]
pub(super) struct IconDone;

/// A FrameXML `Texture` region: a client art file, its `TexCoords` (left, right, top, bottom) and
/// vertex colour, drawn once the archives are readable.
#[derive(Component)]
pub(super) struct UiArt {
    pub(super) path: &'static str,
    coords: [f32; 4],
    tint: Color,
}

impl UiArt {
    pub(super) fn crop(path: &'static str, coords: [f32; 4]) -> Self {
        Self {
            path,
            coords,
            tint: Color::WHITE,
        }
    }

    pub(super) fn tinted(path: &'static str, tint: Color) -> Self {
        Self {
            path,
            coords: [0.0, 1.0, 0.0, 1.0],
            tint,
        }
    }
}

#[derive(Component)]
pub(super) struct ArtDone;

/// What an icon slot's tooltip describes: an index into the run's abilities or passives.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum TipFor {
    Ability(usize),
    Passive(usize),
    Mobility,
}

/// The one `GameTooltip`, and the column its lines go in.
#[derive(Component)]
pub(super) struct GameTooltip;
#[derive(Component)]
pub(super) struct TipBody;

#[derive(Component, Clone, Copy, PartialEq)]
pub(super) enum UiAction {
    Resume,
    Gear,
    MainMenu,
    Quit,
}

/// The colour a button returns to when not hovered.
#[derive(Component)]
pub(super) struct BaseColor(Color);

#[derive(Clone, Copy)]
pub(super) enum Choice {
    NewAbility(&'static AbilityDef),
    Upgrade(usize),
    NewPassive(&'static PassiveDef),
    UpPassive(usize),
    Heal,
}

/// The level-up cards on offer.
#[derive(Resource, Default)]
pub(super) struct Offer {
    choices: Vec<Choice>,
    qualities: Vec<Rarity>,
}

impl Offer {
    pub(super) fn len(&self) -> usize {
        self.choices.len()
    }

    pub(super) fn choices(&self) -> &[Choice] {
        &self.choices
    }

    pub(super) fn quality(&self, index: usize) -> Rarity {
        self.qualities.get(index).copied().unwrap_or(Rarity::Common)
    }

    fn roll(&mut self, run: &Run, rng: &mut Rng) {
        self.choices = choices_for(run, rng);
        let luck = run.stats().luck;
        self.qualities = self
            .choices
            .iter()
            .map(|_| Rarity::roll(rng, luck))
            .collect();
    }
}

#[derive(Resource)]
pub(super) struct HudFont(pub(super) Handle<Font>);

pub(super) fn rgb(hex: u32) -> Color {
    Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn text(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::splat(1.5),
            color: Color::BLACK,
        },
    )
}

pub(super) fn text_bundle(
    font: &Handle<Font>,
    s: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Bundle {
    text(font, s, size, color)
}

pub(super) fn centered_bundle(
    font: &Handle<Font>,
    s: impl Into<String>,
    size: f32,
    color: Color,
) -> impl Bundle {
    centered(font, s, size, color)
}

fn centered(font: &Handle<Font>, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        text(font, s, size, color),
        TextLayout::new_with_justify(Justify::Center),
    )
}

fn button(action: UiAction, base: Color) -> impl Bundle {
    edged_button(action, base, EDGE)
}

fn edged_button(action: UiAction, base: Color, edge: Color) -> impl Bundle {
    (
        Button,
        action,
        BaseColor(base),
        BackgroundColor(base),
        BorderColor::all(edge),
    )
}

fn icon(spell: u32, size: f32) -> impl Bundle {
    (
        Node {
            width: Val::Px(size),
            height: Val::Px(size),
            ..default()
        },
        ImageNode {
            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
            ..default()
        },
        SpellIcon(spell),
    )
}

pub(super) fn spell_name(
    spells: Option<&crate::ui_action::Spells>,
    spell: u32,
    fallback: &str,
) -> String {
    let dbc = spells
        .and_then(|s| s.catalog.get(spell))
        .map(|d| d.name.as_str());
    skill_descriptions::ability_name_for(&crate::char_select::ui_locale(), fallback, dbc)
}

pub(super) fn setup_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut sweeps_init: ResMut<Assets<SweepUiMaterial>>,
    mut fills: ResMut<Assets<super::fill::FillMaterial>>,
) {
    let font = crate::char_select::wow_font(&asset_server);
    let number_font: Handle<Font> = asset_server.load("mpq://Fonts/ARIALN.ttf");
    commands.insert_resource(HudFont(font.clone()));
    let f = &font;
    let map = MiniMap::new(&mut images);

    // --- The in-run HUD. ---
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            GlobalZIndex(400),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            // `MainMenuExpBar`, 1024×13 at the bottom centre. The reference stacks it on the
            // 40-unit action bar; with no action bar here it rests on the screen's edge.
            root.spawn((
                XpBar,
                Interaction::default(),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(0.0),
                    margin: UiRect::left(Val::Px(-512.0)),
                    width: Val::Px(1024.0),
                    height: Val::Px(13.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    XpFill,
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ImageNode {
                        color: Color::NONE,
                        ..default()
                    },
                    UiArt::tinted(
                        "Interface\\TargetingFrame\\UI-StatusBar",
                        Color::srgb(0.58, 0.0, 0.55),
                    ),
                ));
                // `MainMenuXPBarTexture0`..`3`: the segment rules, 256×10 on the bar's top.
                for (i, top) in [0.792_968_75, 0.542_968_75, 0.292_968_75, 0.042_968_75]
                    .into_iter()
                    .enumerate()
                {
                    bar.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(256.0 * i as f32),
                            top: Val::Px(0.0),
                            width: Val::Px(256.0),
                            height: Val::Px(10.0),
                            ..default()
                        },
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
                        UiArt::crop(
                            "Interface\\MainMenuBar\\UI-MainMenuBar-Dwarf",
                            [0.0, 1.0, top, top + 0.039_062_5],
                        ),
                    ));
                }
                // `MainMenuBarExpText`, `TextStatusBarText` centred one unit up, shown on hover.
                bar.spawn(Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    top: Val::Px(-1.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|n| {
                    n.spawn((
                        XpText,
                        Text::new(""),
                        TextFont {
                            font: number_font.clone(),
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextShadow {
                            offset: Vec2::new(1.0, 1.0),
                            color: Color::BLACK,
                        },
                        TextLayout::new(Justify::Center, LineBreak::NoWrap),
                        Visibility::Hidden,
                    ));
                });
            });
            minimap::spawn_cluster(root, f, &map);
            // The mobility button: a 36-unit action slot in `UI-Quickslot2`, its key over it.
            root.spawn((
                MobilitySlot,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    bottom: Val::Px(24.0),
                    margin: UiRect::left(Val::Px(-MOBILITY_SIZE / 2.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    display: Display::None,
                    ..default()
                },
            ))
            .with_children(|col| {
                col.spawn(centered(f, "F", 18.0, Color::WHITE));
                col.spawn((
                    Node {
                        width: Val::Px(MOBILITY_SIZE),
                        height: Val::Px(MOBILITY_SIZE),
                        ..default()
                    },
                    Interaction::default(),
                    TipFor::Mobility,
                ))
                .with_children(|slot| {
                    // `UI-Quickslot2` is the empty slot's 64×64 frame, the button 36 inside it.
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(-MOBILITY_SIZE * 0.42),
                            top: Val::Px(-MOBILITY_SIZE * 0.42),
                            width: Val::Px(MOBILITY_SIZE * 1.84),
                            height: Val::Px(MOBILITY_SIZE * 1.84),
                            ..default()
                        },
                        ImageNode {
                            color: Color::NONE,
                            ..default()
                        },
                        UiArt::crop("Interface\\Buttons\\UI-Quickslot2", [0.0, 1.0, 0.0, 1.0]),
                    ));
                    slot.spawn((
                        MobilityIcon,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        ImageNode {
                            color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                            ..default()
                        },
                    ));
                    let handle = sweeps_init.add(SweepUiMaterial { sweep: 0.0 });
                    slot.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        MaterialNode(handle.clone()),
                        MobilitySweep(handle),
                    ));
                });
            });
            root.spawn(Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|row| {
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    width: Val::Px(348.0),
                    margin: UiRect::top(Val::Px(100.0)),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((
                        AbilityRow,
                        Node {
                            column_gap: Val::Px(4.0),
                            height: Val::Px(40.0),
                            ..default()
                        },
                    ));
                    col.spawn((
                        PassiveRow,
                        Node {
                            column_gap: Val::Px(4.0),
                            row_gap: Val::Px(4.0),
                            width: Val::Px(200.0),
                            flex_wrap: FlexWrap::Wrap,
                            ..default()
                        },
                    ));
                });
                // The clock, with the level and the kill count on one line under it.
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(0.0),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn((TimerText, centered(f, "00:00", 34.0, Color::WHITE)));
                    col.spawn((ObjectiveText, centered(f, "", 12.0, GOLD)));
                    col.spawn(Node {
                        column_gap: Val::Px(14.0),
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((
                            LevelText,
                            text(
                                f,
                                format!("{} 1", localization::text("Level", "等级")),
                                14.0,
                                GOLD,
                            ),
                        ));
                        line.spawn((
                            KillText,
                            text(
                                f,
                                format!("{} 0", localization::text("Kills", "击杀")),
                                14.0,
                                Color::WHITE,
                            ),
                        ));
                    });
                });
                // The minimap cluster's column.
                row.spawn(Node {
                    width: Val::Px(300.0),
                    ..default()
                });
            });
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(22.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|n| {
                n.spawn((BannerText, centered(f, "", 30.0, GOLD)));
            });
            // The hero's health as a nameplate (`CGNamePlateFrame`): the bar under the border that
            // caps it, the level in the border's seat; [`hero_plate`] sizes and seats it.
            let absolute = || Node {
                position_type: PositionType::Absolute,
                ..default()
            };
            let unpainted = || ImageNode {
                color: Color::NONE,
                ..default()
            };
            let [r, g, b, _] = crate::vplates::PLATE_FRIENDLY;
            root.spawn((HpFrame, absolute(), Visibility::Hidden))
                .with_children(|plate| {
                    plate.spawn((HpBar, absolute())).with_children(|bar| {
                        bar.spawn((
                            HpFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            unpainted(),
                            UiArt::tinted(nameplate::BAR_FILL_TEXTURE, Color::srgb(r, g, b)),
                        ));
                        bar.spawn((
                            ShieldFill,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..absolute()
                            },
                            unpainted(),
                            UiArt::tinted(
                                nameplate::BAR_FILL_TEXTURE,
                                Color::srgba(0.85, 0.9, 1.0, 0.75),
                            ),
                        ));
                    });
                    plate.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..absolute()
                        },
                        unpainted(),
                        UiArt::crop(nameplate::BORDER_TEXTURE, [0.0, 1.0, 0.0, 1.0]),
                    ));
                    let [r, g, b, _] = crate::vplates::CON_YELLOW;
                    plate
                        .spawn((
                            HpLevelSeat,
                            Node {
                                width: Val::Px(0.0),
                                height: Val::Px(0.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                overflow: Overflow::visible(),
                                ..absolute()
                            },
                        ))
                        .with_children(|seat| {
                            seat.spawn((
                                HpLevel,
                                Text::new("1"),
                                TextFont {
                                    font: f.clone(),
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(r, g, b)),
                                TextShadow {
                                    offset: Vec2::new(1.0, 1.0),
                                    color: Color::BLACK,
                                },
                                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                            ));
                        });
                });
        });
    commands.insert_resource(map);

    // --- The run's pick screen; [`super::menu`] builds into it once the glue art is in. ---
    commands.spawn((
        MenuRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(410),
        bevy::ui::FocusPolicy::Pass,
    ));

    // --- Level up. ---
    commands
        .spawn((
            LevelUpRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            // A dark vignette closing in on the cards, a slow golden light at its heart (the
            // 1400-unit burst's radius over the 768-unit canvas).
            MaterialNode(
                fills.add(
                    super::fill::FillMaterial::radial(
                        Color::srgba(0.04, 0.025, 0.0, 0.72),
                        Color::srgba(0.0, 0.0, 0.0, 0.9),
                    )
                    .with_glow(
                        Color::srgba(1.0, 0.75, 0.25, 0.22),
                        Vec2::splat(0.5),
                        Vec2::splat(700.0 / 768.0),
                    ),
                ),
            ),
            GlobalZIndex(420),
            Visibility::Hidden,
            // The build's icons stay hoverable under the cards.
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            // The slow golden burst behind everything.
            root.spawn((
                super::cards::Rays,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(1400.0),
                    height: Val::Px(1400.0),
                    left: Val::Percent(50.0),
                    top: Val::Percent(50.0),
                    margin: UiRect::new(
                        Val::Px(-700.0),
                        Val::Px(0.0),
                        Val::Px(-700.0),
                        Val::Px(0.0),
                    ),
                    ..default()
                },
                BackgroundGradient(vec![Gradient::Conic(ConicGradient::new(
                    UiPosition::CENTER,
                    (0..24)
                        .map(|i| {
                            let a = i as f32 / 24.0 * std::f32::consts::TAU;
                            let on = i % 2 == 0;
                            AngularColorStop::new(
                                if on {
                                    Color::srgba(1.0, 0.8, 0.3, 0.07)
                                } else {
                                    Color::srgba(1.0, 0.8, 0.3, 0.0)
                                },
                                a,
                            )
                        })
                        .collect(),
                ))]),
                UiTransform::default(),
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn(centered(
                f,
                localization::text("LEVEL UP", "升级"),
                52.0,
                GOLD,
            ));
            root.spawn((
                LevelUpSub,
                centered(
                    f,
                    localization::text("Choose your power", "选择一项强化"),
                    16.0,
                    Color::srgb(0.85, 0.78, 0.6),
                ),
            ));
            root.spawn((
                OfferRow,
                Node {
                    column_gap: Val::Px(22.0),
                    // Room for a hovered card's lift.
                    margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(28.0), Val::Px(16.0)),
                    ..default()
                },
            ));
            root.spawn(centered(
                f,
                localization::text("Click a card, or press 1, 2 or 3", "点击卡牌，或按 1、2、3"),
                14.0,
                DIM,
            ));
        });

    // --- Pause. ---
    commands
        .spawn((
            PauseRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(430),
            Visibility::Hidden,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|root| {
            root.spawn(centered(
                f,
                localization::text("Paused", "已暂停"),
                46.0,
                GOLD,
            ));
            for (label, action) in [
                (localization::text("Resume", "继续"), UiAction::Resume),
                (
                    localization::text("Equipment (B)", "装备 (B)"),
                    UiAction::Gear,
                ),
                (
                    localization::text("Abandon run", "放弃本局"),
                    UiAction::MainMenu,
                ),
                (localization::text("Quit game", "退出游戏"), UiAction::Quit),
            ] {
                root.spawn((
                    button(action, BUTTON),
                    Node {
                        width: Val::Px(240.0),
                        justify_content: JustifyContent::Center,
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                ))
                .with_children(|b| {
                    b.spawn(text(f, label, 20.0, Color::WHITE));
                });
            }
        });

    // --- Results. ---
    commands
        .spawn((
            OverRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                ..default()
            },
            MaterialNode(fills.add(super::fill::FillMaterial::radial(
                Color::srgba(0.07, 0.02, 0.0, 0.8),
                Color::srgba(0.0, 0.0, 0.0, 0.94),
            ))),
            GlobalZIndex(440),
            Visibility::Hidden,
        ))
        .with_children(|root| {
            root.spawn((OverTitle, centered(f, "", 64.0, GOLD)));
            root.spawn((
                OverLine,
                centered(f, "", 18.0, Color::srgb(0.85, 0.78, 0.6)),
            ));
            // The run in numbers, one tile each.
            root.spawn(Node {
                column_gap: Val::Px(12.0),
                margin: UiRect::vertical(Val::Px(14.0)),
                ..default()
            })
            .with_children(|row| {
                for (i, label) in [
                    localization::text("Survived", "存活时间"),
                    localization::text("Level", "等级"),
                    localization::text("Kills", "击杀"),
                    localization::text("Damage", "伤害"),
                    localization::text("Bosses", "首领"),
                ]
                .into_iter()
                .enumerate()
                {
                    row.spawn((
                        Node {
                            width: Val::Px(128.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(10.0), Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundGradient::from(LinearGradient::to_bottom(vec![
                            ColorStop::percent(Color::srgba(0.2, 0.14, 0.05, 0.92), 0.0),
                            ColorStop::percent(Color::srgba(0.04, 0.03, 0.02, 0.94), 100.0),
                        ])),
                        BorderColor::all(EDGE),
                    ))
                    .with_children(|tile| {
                        tile.spawn((OverStat(i), centered(f, "", 26.0, Color::WHITE)));
                        tile.spawn(centered(f, label, 12.0, DIM));
                    });
                }
            });
            root.spawn((OverStats, centered(f, "", 15.0, DIM)));
            root.spawn((
                edged_button(UiAction::MainMenu, BUTTON, GOLD),
                Node {
                    width: Val::Px(260.0),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(11.0)),
                    margin: UiRect::top(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BoxShadow::new(
                    Color::srgba(1.0, 0.7, 0.2, 0.35),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(16.0),
                ),
            ))
            .with_children(|b| {
                b.spawn(text(
                    f,
                    localization::text("Play again", "再来一局"),
                    22.0,
                    GOLD,
                ));
            });
            root.spawn((
                button(UiAction::MainMenu, Color::NONE),
                Node {
                    justify_content: JustifyContent::Center,
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                    ..default()
                },
            ))
            .with_children(|b| {
                b.spawn(text(
                    f,
                    localization::text("Return to main menu", "返回主菜单"),
                    15.0,
                    DIM,
                ));
            });
        });
}

/// The menu's hero is the run's hero: respawned whenever the pick changes.
pub(super) fn menu_preview(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    mut run: ResMut<Run>,
    heroes: Query<Entity, With<Hero>>,
    world: benilla_world::world_point::WorldPoint,
    gear: Res<super::gear::Gear>,
    mut shown: Local<Option<(usize, u32, usize, u32)>>,
) {
    if *phase.get() != Phase::Menu {
        return;
    }
    let want = selection
        .class
        .zip(selection.look.and_then(|i| selection.looks.get(i)).copied());
    let key = want.map(|(c, l)| (c, l.0, selection.stage, gear.version));
    if key == *shown && (key.is_none() || !heroes.is_empty()) {
        return;
    }
    for e in &heroes {
        commands.entity(e).despawn();
    }
    *shown = key;
    let Some((class, (display, race, sex))) = want else {
        return;
    };
    let mut pos = hero::arena(selection.stage_def());
    if let Some(h) = world.terrain_height_under(pos) {
        pos.y = h;
    }
    let serial = run.guid();
    hero::spawn_hero(
        &mut commands,
        &data::CLASSES[class],
        display,
        race,
        sex,
        &gear.weapons(&data::CLASSES[class]),
        serial,
        pos,
    );
}

pub(super) fn start_run(run: &mut Run, selection: &mut Selection) -> bool {
    let (Some(class), Some(&(_, race, _))) = (
        selection.class_def(),
        selection.look.and_then(|i| selection.looks.get(i)),
    ) else {
        return false;
    };
    // The world goes to the battleground picked; until now it stood in the last one.
    selection.stage = selection.map;
    let next_guid = run.next_guid;
    *run = Run {
        class,
        race,
        next_guid,
        ..Run::default()
    };
    run.abilities.push(Owned {
        def: data::ability(class.start),
        level: 1,
        growth: 1.0,
        cd: 0.5,
    });
    run.hp = run.stats().max_hp;
    run.version += 1;
    true
}

fn choices_for(run: &Run, rng: &mut Rng) -> Vec<Choice> {
    let mut pool: Vec<(Choice, f32)> = Vec::new();
    for (i, _) in run.abilities.iter().enumerate() {
        pool.push((Choice::Upgrade(i), 1.2));
    }
    if run.abilities.len() < data::MAX_ABILITIES {
        for key in run.class.pool {
            if !run.abilities.iter().any(|o| o.def.key == *key) {
                pool.push((Choice::NewAbility(data::ability(key)), 1.0));
            }
        }
    }
    for (i, _) in run.passives.iter().enumerate() {
        pool.push((Choice::UpPassive(i), 0.8));
    }
    if run.passives.len() < data::MAX_PASSIVES {
        for p in data::PASSIVES {
            if !run.passives.iter().any(|(q, _)| q.key == p.key) {
                pool.push((Choice::NewPassive(p), 0.5));
            }
        }
    }
    let mut out = Vec::with_capacity(3);
    while out.len() < 3 && !pool.is_empty() {
        let total: f32 = pool.iter().map(|(_, w)| w).sum();
        let mut pick = rng.f32() * total;
        let mut idx = pool.len() - 1;
        for (i, (_, w)) in pool.iter().enumerate() {
            if pick < *w {
                idx = i;
                break;
            }
            pick -= w;
        }
        out.push(pool.swap_remove(idx).0);
    }
    if out.is_empty() {
        out.push(Choice::Heal);
    }
    out
}

pub(super) fn roll_offer(run: Res<Run>, mut rng: ResMut<Rng>, mut offer: ResMut<Offer>) {
    offer.roll(&run, &mut rng);
}

pub(super) fn pick(
    i: usize,
    run: &mut Run,
    offer: &mut Offer,
    rng: &mut Rng,
    next: &mut NextState<Phase>,
) {
    let Some(&choice) = offer.choices.get(i) else {
        return;
    };
    let growth = offer.quality(i).growth();
    match choice {
        Choice::NewAbility(def) => run.abilities.push(Owned {
            def,
            level: 1,
            growth,
            cd: 0.3,
        }),
        Choice::Upgrade(k) => {
            run.abilities[k].level = run.abilities[k].level.saturating_add(1);
            run.abilities[k].growth += growth;
        }
        Choice::NewPassive(p) => {
            run.passives.push((p, 1));
            run.passive_growth.insert(p.key, growth);
        }
        Choice::UpPassive(k) => {
            let (p, level) = run.passives[k];
            let total = run.passive_growth(p) + growth;
            run.passives[k].1 = level.saturating_add(1);
            run.passive_growth.insert(p.key, total);
        }
        Choice::Heal => run.hp = run.stats().max_hp,
    }
    run.version += 1;
    run.pending_levels = run.pending_levels.saturating_sub(1);
    if run.pending_levels > 0 {
        offer.roll(run, rng);
    } else {
        offer.choices.clear();
        offer.qualities.clear();
        next.set(Phase::Playing);
    }
}

pub(super) fn looks_for(
    class: &data::ClassDef,
    creatures: &crate::entities::Creatures,
) -> Vec<(u32, u8, u8)> {
    let mut out: Vec<(u32, u8, u8)> = Vec::new();
    for &d in class.looks {
        if let Some((race, sex)) = creatures.display_race_sex(d) {
            if !out.iter().any(|&(_, r, s)| r == race && s == sex) {
                out.push((d, race, sex));
            }
        }
    }
    out.sort_by_key(|&(_, r, s)| (r, s));
    out
}

#[allow(clippy::too_many_arguments)]
pub(super) fn buttons(
    mut presses: Query<
        (&Interaction, &UiAction, &BaseColor, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut next: ResMut<NextState<Phase>>,
    mut exit: MessageWriter<AppExit>,
    mut gear: ResMut<super::gear_ui::GearUi>,
    phase: Res<State<Phase>>,
) {
    for (interaction, action, base, mut bg) in &mut presses {
        match interaction {
            Interaction::Hovered => bg.0 = BUTTON_HOT,
            Interaction::None => bg.0 = base.0,
            Interaction::Pressed => {
                bg.0 = BUTTON_HOT;
                match (*action, *phase.get()) {
                    (UiAction::Resume, Phase::Paused) => next.set(Phase::Playing),
                    (UiAction::Gear, Phase::Paused) => gear.open = true,
                    (UiAction::MainMenu, Phase::Paused | Phase::GameOver) => next.set(Phase::Menu),
                    (UiAction::Quit, Phase::Paused | Phase::GameOver) => {
                        exit.write(AppExit::Success);
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(super) fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    phase: Res<State<Phase>>,
    mut selection: ResMut<Selection>,
    mut run: ResMut<Run>,
    mut next: ResMut<NextState<Phase>>,
    talents: Res<super::talents::TalentUi>,
    mut gear: ResMut<super::gear_ui::GearUi>,
    menu: Res<super::menu::MenuState>,
) {
    if gear.open && keys.just_pressed(KeyCode::Escape) {
        gear.open = false;
        return;
    }
    match phase.get() {
        Phase::Menu => {
            if !talents.open
                && !gear.open
                && !menu.modal_open()
                && keys.just_pressed(KeyCode::Enter)
                && start_run(&mut run, &mut selection)
            {
                next.set(Phase::Playing);
            }
        }
        Phase::Playing => {
            if keys.just_pressed(KeyCode::Escape) {
                next.set(Phase::Paused);
            } else if keys.just_pressed(KeyCode::KeyB) {
                gear.open = true;
                next.set(Phase::Paused);
            }
        }
        Phase::Paused => {
            if keys.just_pressed(KeyCode::Escape) {
                next.set(Phase::Playing);
            } else if keys.just_pressed(KeyCode::KeyB) {
                gear.open = true;
            }
        }
        Phase::LevelUp => {}
        Phase::GameOver => {
            if keys.just_pressed(KeyCode::Enter) {
                next.set(Phase::Menu);
            }
        }
    }
}

/// The class's looks once the creature data is in, defaulting to the first. The auto-runner picks
/// a class with no look; the menu itself always sets both.
pub(super) fn refresh_menu(
    mut selection: ResMut<Selection>,
    creatures: Option<Res<crate::entities::Creatures>>,
) {
    if let (Some(class), Some(creatures)) = (selection.class_def(), creatures.as_deref()) {
        if selection.looks.is_empty() {
            let looks = looks_for(class, creatures);
            if !looks.is_empty() {
                selection.looks = looks;
                selection.look = Some(0);
                selection.version += 1;
            }
        }
    }
}

/// What a level-up card shows.
pub(super) struct CardText {
    pub spell: u32,
    pub name: String,
    pub tag: String,
    pub tag_color: Color,
    pub desc: String,
}

pub(super) fn describe_card(
    choice: Choice,
    quality: Rarity,
    run: &Run,
    spells: Option<&crate::ui_action::Spells>,
) -> CardText {
    let new = Color::srgb(0.3, 1.0, 0.3);
    let added = quality.growth();
    match choice {
        Choice::NewAbility(def) => {
            let c = super::growth::cast_for_run(def, 1, added, run);
            CardText {
                spell: def.spell,
                name: spell_name(spells, def.spell, def.key),
                tag: localization::text("New spell · Rank 1", "新法术 · 等级 1").into(),
                tag_color: new,
                desc: format!("{}.", ability_body(def, &c)),
            }
        }
        Choice::Upgrade(k) => {
            let o = &run.abilities[k];
            let next = o.level.saturating_add(1);
            let (a, b) = (
                super::growth::cast_for_run(o.def, o.level, o.growth, run),
                super::growth::cast_for_run(o.def, next, o.growth + added, run),
            );
            // What actually changes, as before → after.
            let mut lines = Vec::new();
            let (da, db) = if a.dmg > 0.0 {
                (a.dmg, b.dmg)
            } else {
                (a.dot, b.dot)
            };
            if db > da {
                let label = match o.def.kind {
                    Kind::Shield | Kind::Immune => localization::text("Absorb", "吸收"),
                    Kind::Heal => localization::text("Healing", "治疗"),
                    _ => localization::text("Damage", "伤害"),
                };
                lines.push(if localization::is_zh_cn() {
                    format!("{label} {} → {}", num(da), num(db))
                } else {
                    format!("{label} {} » {}", num(da), num(db))
                });
            }
            if b.cooldown < a.cooldown {
                lines.push(if localization::is_zh_cn() {
                    format!("冷却 {}秒 → {}秒", num(a.cooldown), num(b.cooldown))
                } else {
                    format!("Cooldown {}s » {}s", num(a.cooldown), num(b.cooldown))
                });
            }
            if b.count > a.count {
                lines.push(if localization::is_zh_cn() {
                    format!("目标数 {} → {}", a.count, b.count)
                } else {
                    format!("Targets {} » {}", a.count, b.count)
                });
            }
            if b.radius > a.radius {
                lines.push(if localization::is_zh_cn() {
                    format!("范围 {} → {}码", num(a.radius), num(b.radius))
                } else {
                    format!("Area {} » {} yd", num(a.radius), num(b.radius))
                });
            }
            if b.duration > a.duration {
                lines.push(if localization::is_zh_cn() {
                    format!("持续 {}秒 → {}秒", num(a.duration), num(b.duration))
                } else {
                    format!("Lasts {}s » {}s", num(a.duration), num(b.duration))
                });
            }
            CardText {
                spell: o.def.spell,
                name: spell_name(spells, o.def.spell, o.def.key),
                tag: if localization::is_zh_cn() {
                    format!("等级 {} → {next}", o.level)
                } else {
                    format!("Rank {} » {next}", o.level)
                },
                tag_color: Color::WHITE,
                desc: format!(
                    "{}\n{} {:.2} → {:.2}",
                    lines.join("\n"),
                    localization::text("Total growth", "累计成长"),
                    o.growth,
                    o.growth + added,
                ),
            }
        }
        Choice::NewPassive(p) => CardText {
            spell: p.spell,
            name: passive_name(p).into(),
            tag: localization::text("New passive · Rank 1", "新被动 · 等级 1").into(),
            tag_color: new,
            desc: passive_preview(p, added, run),
        },
        Choice::UpPassive(k) => {
            let (p, level) = run.passives[k];
            CardText {
                spell: p.spell,
                name: passive_name(p).into(),
                tag: if localization::is_zh_cn() {
                    format!("等级 {level} → {}", level.saturating_add(1))
                } else {
                    format!("Rank {level} » {}", level.saturating_add(1))
                },
                tag_color: Color::WHITE,
                desc: passive_preview(p, added, run),
            }
        }
        Choice::Heal => CardText {
            spell: 5720,
            name: localization::text("Healthstone", "治疗石").into(),
            tag: localization::text("Restore", "恢复").into(),
            tag_color: Color::WHITE,
            desc: localization::text("Restores all health.", "恢复全部生命值。").into(),
        },
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn refresh_hud(
    mut commands: Commands,
    time: Res<Time>,
    phase: Res<State<Phase>>,
    mut run: ResMut<Run>,
    selection: Res<Selection>,
    font: Res<HudFont>,
    progression: Res<super::talents::Progression>,
    gear: Option<Res<super::gear::Gear>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<LevelText>>,
        Query<&mut Text, With<TimerText>>,
        Query<&mut Text, With<KillText>>,
        Query<&mut Text, With<BannerText>>,
        Query<&mut Text, With<OverTitle>>,
        Query<&mut Text, With<OverStats>>,
        Query<&mut Text, With<XpText>>,
        Query<&mut Text, With<ObjectiveText>>,
    )>,
    mut xp_fill: Query<&mut Node, With<XpFill>>,
    rows: Query<(Entity, Has<AbilityRow>), Or<(With<AbilityRow>, With<PassiveRow>)>>,
    mut sweeps: ResMut<Assets<SweepUiMaterial>>,
    mut last_version: Local<Option<u32>>,
) {
    let need = data::xp_to_next(run.level);
    if let Ok(mut n) = xp_fill.single_mut() {
        n.width = Val::Percent((run.xp / need * 100.0).clamp(0.0, 100.0));
    }
    if let Ok(mut t) = texts.p6().single_mut() {
        let xp = if localization::is_zh_cn() {
            format!("经验 {} / {}", run.xp as u32, need as u32)
        } else {
            format!("XP {} / {}", run.xp as u32, need as u32)
        };
        if **t != xp {
            **t = xp;
        }
    }
    if let Ok(mut t) = texts.p0().single_mut() {
        **t = format!("{} {}", localization::text("Level", "等级"), run.level);
    }
    let secs = run.elapsed as u32;
    if let Ok(mut t) = texts.p1().single_mut() {
        **t = format!("{:02}:{:02}", secs / 60, secs % 60);
    }
    if let Ok(mut t) = texts.p7().single_mut() {
        let objective = battle_objective(&run);
        if **t != objective {
            **t = objective;
        }
    }
    let kills = run.kills;
    if let Ok(mut t) = texts.p2().single_mut() {
        **t = format!("{} {kills}", localization::text("Kills", "击杀"));
    }
    let banner = match run.banner.as_mut() {
        Some((msg, t)) => {
            *t -= time.delta_secs();
            if *t > 0.0 {
                msg.clone()
            } else {
                String::new()
            }
        }
        None => String::new(),
    };
    if banner.is_empty() {
        run.banner = None;
    }
    // The picker owns the screen: an announcement waits until it closes.
    let banner = if *phase.get() == Phase::LevelUp {
        String::new()
    } else {
        banner
    };
    if let Ok(mut t) = texts.p3().single_mut() {
        if **t != banner {
            **t = banner;
        }
    }
    let title = if run.won {
        localization::text("Victory!", "胜利！")
    } else {
        localization::text("You have fallen", "你已倒下")
    }
    .to_string();
    if let Ok(mut t) = texts.p4().single_mut() {
        **t = title;
    }
    let mut summary = if localization::is_zh_cn() {
        format!(
            "{} {} · {} · 按 Enter 再来一局",
            localization::race_name(run.race, crate::char_select::race_name(run.race)),
            localization::class_name(run.class.id, run.class.name),
            localization::map_name(selection.map_def().name),
        )
    } else {
        format!(
            "{} {} in {}   —   press Enter to play again",
            crate::char_select::race_name(run.race),
            run.class.name,
            selection.map_def().name,
        )
    };
    summary.push_str(&format!(
        "\n{} +{} · {} {}/51 · {} {}",
        localization::text("Talent points this run", "本局天赋点"),
        run.talent_points_gained,
        localization::text("Earned", "职业已获得"),
        progression.points(run.class.id),
        localization::text("Kill credit", "累计击杀进度"),
        progression.score(run.class.id),
    ));
    if let Some(error) = &progression.error {
        summary.push_str(&format!("\n{error}"));
    }
    append_reward_summary(&mut summary, &run, gear.as_deref());
    if let Ok(mut t) = texts.p5().single_mut() {
        if **t != summary {
            **t = summary;
        }
    }

    // The ability and passive icons, rebuilt when the build changes.
    if *last_version == Some(run.version) {
        return;
    }
    *last_version = Some(run.version);
    let f = &font.0;
    for (row, is_abilities) in &rows {
        commands.entity(row).despawn_related::<Children>();
        commands.entity(row).with_children(|row| {
            let entries: Vec<(u32, u32, f32, TipFor)> = if is_abilities {
                run.abilities
                    .iter()
                    .enumerate()
                    .map(|(i, o)| (o.def.spell, o.level, 40.0, TipFor::Ability(i)))
                    .collect()
            } else {
                run.passives
                    .iter()
                    .enumerate()
                    .map(|(i, (p, l))| (p.spell, *l, 30.0, TipFor::Passive(i)))
                    .collect()
            };
            for (spell, level, size, tip) in entries {
                let ability = match &tip {
                    TipFor::Ability(i) => Some(*i),
                    _ => None,
                };
                row.spawn((
                    Node {
                        width: Val::Px(size),
                        height: Val::Px(size),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(EDGE),
                    Interaction::default(),
                    tip,
                ))
                .with_children(|slot| {
                    slot.spawn(icon(spell, size - 2.0));
                    // The cooldown sweep sits over the icon, under the level text.
                    if let Some(i) = ability {
                        let handle = sweeps.add(SweepUiMaterial { sweep: 0.0 });
                        slot.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            MaterialNode(handle.clone()),
                            AbilitySweep { handle, index: i },
                        ));
                    }
                    slot.spawn(Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(2.0),
                        bottom: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|n| {
                        n.spawn(text(f, level.to_string(), 12.0, Color::WHITE));
                    });
                });
            }
        });
    }
}

/// Fill icon slots from `SpellIcon.dbc` once the client data and the asset reader are up.
pub(super) fn fill_icons(
    mut commands: Commands,
    mut slots: Query<(Entity, &mut ImageNode, &SpellIcon), Without<IconDone>>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut cache: Local<HashMap<u32, Handle<Image>>>,
) {
    let (Some(spells), Some(assets)) = (spells.as_deref(), assets.as_deref_mut()) else {
        return;
    };
    for (e, mut img, icon) in &mut slots {
        let handle = match cache.get(&icon.0) {
            Some(h) => Some(h.clone()),
            None => {
                let path = spells
                    .catalog
                    .get(icon.0)
                    .and_then(|d| d.icon.clone())
                    .unwrap_or_else(|| "Interface\\Icons\\INV_Misc_QuestionMark".into());
                let h = assets.sprite_texture(&format!("{path}.blp"), &mut images);
                if let Some(h) = &h {
                    cache.insert(icon.0, h.clone());
                }
                h
            }
        };
        if let Some(h) = handle {
            img.image = h;
            img.color = Color::WHITE;
        }
        commands.entity(e).insert(IconDone);
    }
}

/// The mobility button: shown once learned, its icon the class's spell, its sweep the cooldown.
#[allow(clippy::type_complexity)]
pub(super) fn refresh_mobility(
    run: Res<Run>,
    mut commands: Commands,
    mut slot: Query<&mut Node, With<MobilitySlot>>,
    icon: Query<(Entity, Option<&SpellIcon>), With<MobilityIcon>>,
    sweep: Query<&MobilitySweep>,
    mut mats: ResMut<Assets<SweepUiMaterial>>,
) {
    let def = run.mobility();
    if let Ok(mut node) = slot.single_mut() {
        let display = if def.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    let Some(def) = def else { return };
    // A new spell re-arms the icon fill ([`fill_icons`]).
    if let Ok((e, have)) = icon.single() {
        if have.is_none_or(|s| s.0 != def.spell) {
            commands
                .entity(e)
                .remove::<IconDone>()
                .insert(SpellIcon(def.spell));
        }
    }
    if let Ok(s) = sweep.single() {
        let total = def.cooldown * run.mobility_cooldown();
        let f = (run.mobile.cd / total.max(0.01)).clamp(0.0, 1.0);
        if let Some(m) = mats.get_mut(&s.0) {
            if m.sweep != f {
                m.sweep = f;
            }
        }
    }
}

/// Drive each ability icon's cooldown sweep from its remaining cooldown.
pub(super) fn update_cooldown_sweeps(
    run: Res<Run>,
    mut mats: ResMut<Assets<SweepUiMaterial>>,
    sweeps: Query<&AbilitySweep>,
) {
    for s in &sweeps {
        let Some(o) = run.abilities.get(s.index) else {
            continue;
        };
        let total = super::talents::cast_for_run(o.def, o.level, &run)
            .cooldown
            .max(0.01);
        let f = (o.cd / total).clamp(0.0, 1.0);
        if let Some(m) = mats.get_mut(&s.handle) {
            m.sweep = f;
        }
    }
}

/// Draw each [`UiArt`] region once its file can be read.
pub(super) fn fill_art(
    mut commands: Commands,
    mut regions: Query<(Entity, &mut ImageNode, &UiArt), Without<ArtDone>>,
    assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut assets) = assets else { return };
    for (e, mut img, art) in &mut regions {
        commands.entity(e).insert(ArtDone);
        let Some(h) = assets.sprite_texture(art.path, &mut images) else {
            warn!("survivors: no UI art {}", art.path);
            continue;
        };
        let [l, r, t, b] = art.coords;
        if let Some(size) = images.get(&h).map(Image::size_f32) {
            img.rect = Some(Rect::new(l * size.x, t * size.y, r * size.x, b * size.y));
        }
        img.image = h;
        img.color = art.tint;
        // A tiled region is spawned with a coarse stretch, since the placeholder 1x1 texture at
        // the real one would tile into millions of quads; the real texture gets the fine one.
        if let NodeImageMode::Tiled { stretch_value, .. } = &mut img.image_mode {
            *stretch_value = 0.5;
        }
    }
}

/// The reference lays its UI out on a screen 768 units tall at any resolution (`useUiScale` off).
pub(super) fn apply_ui_scale(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut scale: ResMut<UiScale>,
) {
    let Ok(window) = windows.single() else { return };
    let want = window.height() / 768.0;
    if want > 0.0 && (scale.0 - want).abs() > 1e-4 {
        scale.0 = want;
    }
}

/// Between the hero's feet and the top of its plate's bar, in gx units.
const PLATE_FEET_GAP: f32 = 0.004;
/// A couple of screen pixels lower than the gap alone sets it.
const PLATE_NUDGE: f32 = 2.0;

/// The hero's nameplate under its feet, sized from the screen diagonal as [`crate::vplates`]
/// sizes a plate, outside the UI scale. The border's top half seats a name the hero has none of,
/// so the plate rises until its bar sits just under the feet.
#[allow(clippy::type_complexity)]
pub(super) fn hero_plate(
    run: Res<Run>,
    ui_scale: Res<UiScale>,
    hero: Query<&Transform, With<Hero>>,
    cams: Query<(&Camera, &Transform), With<WorldCamera>>,
    mut nodes: ParamSet<(
        Query<(&mut Node, &mut Visibility), With<HpFrame>>,
        Query<&mut Node, With<HpBar>>,
        Query<&mut Node, With<HpFill>>,
        Query<&mut Node, With<ShieldFill>>,
        Query<&mut Node, With<HpLevelSeat>>,
    )>,
    mut level: Query<(&mut Text, &mut TextFont), With<HpLevel>>,
) {
    use crate::vplates::{
        gx_px, plate_basis, text_px, BAR_H, BAR_OFF_X, BAR_OFF_Y, BAR_W, LEVEL_H, LEVEL_OFF_X,
        LEVEL_OFF_Y, PLATE_H, PLATE_W,
    };
    let seat = hero.single().ok().and_then(|h| {
        let (cam, tf) = cams.single().ok()?;
        let viewport = cam.logical_viewport_size()?;
        let feet = cam
            .world_to_viewport(&GlobalTransform::from(*tf), h.translation)
            .ok()?;
        Some((feet, viewport))
    });
    let mut frames = nodes.p0();
    let Ok((mut frame, mut vis)) = frames.single_mut() else {
        return;
    };
    let Some((feet, viewport)) = seat else {
        *vis = Visibility::Hidden;
        return;
    };
    *vis = Visibility::Inherited;
    let s = ui_scale.0;
    let basis = plate_basis(viewport);
    let gx = |v: f32| gx_px(v, basis) / s;
    let (pw, ph) = (gx(PLATE_W), gx(PLATE_H));
    let bar_top = ph - gx(BAR_OFF_Y) - gx(BAR_H);
    frame.width = Val::Px(pw);
    frame.height = Val::Px(ph);
    frame.left = Val::Px(feet.x / s - pw / 2.0);
    frame.top = Val::Px(feet.y / s + gx(PLATE_FEET_GAP) - bar_top + PLATE_NUDGE / s);
    if let Ok(mut bar) = nodes.p1().single_mut() {
        bar.left = Val::Px(gx(BAR_OFF_X));
        bar.top = Val::Px(bar_top);
        bar.width = Val::Px(gx(BAR_W));
        bar.height = Val::Px(gx(BAR_H));
    }
    let stats = run.stats();
    let hp = (run.hp / stats.max_hp).clamp(0.0, 1.0);
    let shield = if run.shield_t > 0.0 {
        (run.shield / stats.max_hp).clamp(0.0, 1.0)
    } else {
        0.0
    };
    if let Ok(mut n) = nodes.p2().single_mut() {
        n.width = Val::Percent(hp * 100.0);
    }
    if let Ok(mut n) = nodes.p3().single_mut() {
        n.width = Val::Percent(shield * 100.0);
    }
    if let Ok(mut seat) = nodes.p4().single_mut() {
        seat.left = Val::Px(pw - gx(LEVEL_OFF_X));
        seat.top = Val::Px(ph - gx(LEVEL_OFF_Y));
    }
    if let Ok((mut text, mut font)) = level.single_mut() {
        let n = run.level.to_string();
        if text.0 != n {
            text.0 = n;
        }
        let size = text_px(LEVEL_H, basis) / s;
        if font.font_size != size {
            font.font_size = size;
        }
    }
}

/// `MainMenuExpBar`'s `OnEnter` and `OnLeave`: the numbers show while the cursor is on the bar.
pub(super) fn xp_hover(
    bars: Query<&Interaction, (With<XpBar>, Changed<Interaction>)>,
    mut texts: Query<&mut Visibility, With<XpText>>,
) {
    for interaction in &bars {
        let want = if *interaction == Interaction::None {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        for mut v in &mut texts {
            *v = want;
        }
    }
}

// ── The build icons' `GameTooltip` ──────────────────────────────────────────────────────────────

/// `TOOLTIP_DEFAULT_BACKGROUND_COLOR`, the `GameTooltip` backdrop tint (`GameTooltip.lua`).
const TIP_FILL: Color = Color::srgb(0.09, 0.09, 0.19);
/// The grey a spell tooltip prints its rank in, right of the name.
const TIP_GREY: Color = Color::srgb(0.5, 0.5, 0.5);

/// One tooltip line: the left text, an optional right-aligned one, the colour and font size
/// (`GameTooltipHeaderText` 14, `GameTooltipText` 12).
struct TipLine {
    left: String,
    right: Option<(String, Color)>,
    color: Color,
    size: f32,
}

impl TipLine {
    fn new(left: impl Into<String>, color: Color) -> Self {
        Self {
            left: left.into(),
            right: None,
            color,
            size: 12.0,
        }
    }

    fn right(mut self, text: impl Into<String>, color: Color) -> Self {
        self.right = Some((text.into(), color));
        self
    }

    fn header(mut self) -> Self {
        self.size = 14.0;
        self
    }
}

/// The results screen's tiles and its line under the title.
#[allow(clippy::type_complexity)]
pub(super) fn refresh_results(
    run: Res<Run>,
    selection: Res<Selection>,
    mut over_texts: ParamSet<(
        Query<(&OverStat, &mut Text)>,
        Query<&mut Text, With<OverLine>>,
    )>,
) {
    let secs = run.elapsed as u32;
    let stats_line = if run.won {
        localization::victory_line(selection.map_def().victory)
    } else {
        localization::text(
            "The horde was too much this time.",
            "这一次，兽潮还是压倒了你。",
        )
    };
    let values = [
        format!("{:02}:{:02}", secs / 60, secs % 60),
        run.level.to_string(),
        run.kills.to_string(),
        compact(run.damage_done),
        run.boss_kills.to_string(),
    ];
    for (stat, mut t) in &mut over_texts.p0() {
        if let Some(v) = values.get(stat.0) {
            if **t != *v {
                **t = v.clone();
            }
        }
    }
    if let Ok(mut t) = over_texts.p1().single_mut() {
        if **t != stats_line {
            **t = stats_line.to_string();
        }
    }
}

fn battle_objective(run: &Run) -> String {
    let objective = if run.final_boss_defeated {
        localization::text("Final boss defeated", "最终首领已击败")
    } else if run.elapsed >= data::VICTORY_SECS {
        localization::text(
            "Final battle: defeat the final boss",
            "最终决战：击败最终首领",
        )
    } else {
        localization::text("Defeat the final boss at 10:00", "10:00迎战最终首领")
    };
    format!(
        "{objective} · {} {}/{}",
        localization::text("Bosses", "首领"),
        run.boss_kills.min(data::BOSS_COUNT),
        data::BOSS_COUNT,
    )
}

fn append_reward_summary(summary: &mut String, run: &Run, gear: Option<&super::gear::Gear>) {
    if let Some(reward) = &run.final_reward {
        summary.push_str(&format!(
            "\n{}: {reward}",
            localization::text("Final boss reward", "最终首领奖励")
        ));
    }
    if let Some(error) = &run.reward_error {
        summary.push_str(&format!(
            "\n{}: {error}",
            localization::text("Reward error", "奖励错误")
        ));
    }
    if let Some(error) = gear.and_then(|g| g.error.as_ref()) {
        summary.push_str(&format!(
            "\n{}: {error}",
            localization::text("Equipment save warning", "装备存档警告")
        ));
    }
}

/// 12.3k, 4.5M.
fn compact(v: f64) -> String {
    if v >= 1e6 {
        format!("{:.1}M", v / 1e6)
    } else if v >= 1e4 {
        format!("{:.1}k", v / 1e3)
    } else {
        format!("{}", v.round() as u64)
    }
}

/// A number as the tooltips print it: whole when near enough, else to one decimal.
fn num(v: f32) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

fn pct(v: f32) -> String {
    format!("{}%", num(v * 100.0))
}

/// The numbers an ability casts with at a level, as [`super::abilities::cast_abilities`] takes
/// them from [`Owned`].
type Cast = super::talents::Cast;

fn effect_text(effect: Effect) -> Option<String> {
    if localization::is_zh_cn() {
        return Some(match effect {
            Effect::None => return None,
            Effect::Slow(m, t) => format!("使其移动速度降低{}，持续{}秒", pct(1.0 - m), num(t)),
            Effect::Root(t) => format!("将其定身{}秒", num(t)),
            Effect::Stun(t) => format!("使其昏迷{}秒", num(t)),
            Effect::Fear(t) => format!("使其恐惧并逃跑{}秒", num(t)),
            Effect::Weaken(m, t) => {
                format!("使其造成的伤害降低{}，持续{}秒", pct(1.0 - m), num(t))
            }
            Effect::Drain(f) => format!("将所造成伤害的{}转化为生命值", pct(f)),
            Effect::Execute(f) => format!("目标生命值低于{}时直接将其击杀", pct(f)),
        });
    }
    Some(match effect {
        Effect::None => return None,
        Effect::Slow(m, t) => format!("slowing them by {} for {} sec", pct(1.0 - m), num(t)),
        Effect::Root(t) => format!("freezing them in place for {} sec", num(t)),
        Effect::Stun(t) => format!("stunning them for {} sec", num(t)),
        Effect::Fear(t) => format!("sending them fleeing for {} sec", num(t)),
        Effect::Weaken(m, t) => format!(
            "cutting their damage by {} for {} sec",
            pct(1.0 - m),
            num(t)
        ),
        Effect::Drain(f) => format!("healing you for {} of the damage", pct(f)),
        Effect::Execute(f) => format!("killing them outright below {} health", pct(f)),
    })
}

/// What an ability does at these numbers, in the voice of a spell description.
fn ability_body(def: &AbilityDef, c: &Cast) -> String {
    if localization::is_zh_cn() {
        return ability_body_zh(def, c);
    }
    let d = num(c.dmg);
    let burst = def.duration <= 0.0;
    let mut s = match def.kind {
        Kind::Bolt | Kind::Strike | Kind::Melee => {
            let toughest = matches!(
                def.key,
                "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
            );
            let melee = def.kind == Kind::Melee;
            let who = match (toughest, melee, c.count) {
                (true, _, 1) => "the toughest enemy in range".to_string(),
                (true, _, n) => format!("the {n} toughest enemies in range"),
                (false, true, 1) => "the closest enemy".into(),
                (false, true, n) => format!("the {n} closest enemies"),
                (false, false, 1) => "the nearest enemy".into(),
                (false, false, n) => format!("the {n} nearest enemies"),
            };
            let mut s = format!(
                "{} {who} for {d} damage",
                if melee { "Strikes" } else { "Hits" }
            );
            if c.radius > 0.0 && !melee {
                s += &format!(", and everything within {} yd of the hit", num(c.radius));
            }
            s
        }
        Kind::Nova => format!(
            "Deals {d} damage to every enemy within {} yd of you",
            num(c.radius)
        ),
        Kind::Cone => format!(
            "Deals {d} damage to every enemy in a {} yd cone toward the closest",
            num(c.radius)
        ),
        Kind::Ground | Kind::GroundSelf => {
            let at = if def.kind == Kind::GroundSelf {
                "around you"
            } else {
                "on the thickest pack"
            };
            if burst {
                format!("Strikes a {} yd area {at} for {d} damage", num(c.radius))
            } else {
                format!(
                    "Covers a {} yd area {at} for {} sec, dealing {d} damage every {} sec",
                    num(c.radius),
                    num(c.duration),
                    num(c.modifiers.interval(1.0))
                )
            }
        }
        Kind::Chain => format!(
            "Strikes the nearest enemy for {d} damage, then jumps to {} more within {} yd of each other, {} weaker with every jump",
            c.count,
            num(c.radius),
            pct((0.15 - c.modifiers.chain).max(0.0))
        ),
        Kind::Dot => format!(
            "Afflicts {} enemies in range with {} damage every {} sec for {} sec",
            c.count,
            num(c.dot),
            num(c.modifiers.interval(1.0)),
            num(c.duration)
        ),
        Kind::Aura => format!(
            "Every {} sec, deals {d} damage to every enemy within {} yd of you",
            num(c.cooldown),
            num(c.radius)
        ),
        Kind::Orbit => format!(
            "Every {} sec, zaps {} within {} yd of you for {d} damage",
            num(c.cooldown),
            if c.count == 1 {
                "an enemy".to_string()
            } else {
                format!("{} enemies", c.count)
            },
            num(c.radius)
        ),
        Kind::Shield => format!(
            "Absorbs the next {d} damage you take within {} sec",
            num(c.duration)
        ),
        Kind::Immune => format!(
            "When an enemy closes in, grants immunity for {} sec and a {d} absorb shield for {} sec",
            num(c.duration),
            num(c.duration + 3.0),
        ),
        Kind::Thorns => format!(
            "When enemies close in, deals {d} damage to every attacker for {} sec",
            num(c.duration)
        ),
        Kind::Heal => format!(
            "Below 85% health, heals you for {d} over {} sec",
            num(c.duration)
        ),
        Kind::Totem if def.key == "magma_totem" => format!(
            "Drops a totem for {} sec that burns every enemy within {} yd for {d} damage every {} sec",
            num(c.duration),
            num(c.radius),
            num(c.modifiers.interval(2.0))
        ),
        Kind::Totem => format!(
            "Drops a totem for {} sec that hits the nearest enemy within {} yd for {d} damage every {} sec",
            num(c.duration),
            num(c.range),
            num(c.modifiers.interval(1.0))
        ),
    };
    // Areas that tick carry no over-time part; a burst leaves a 5 second one.
    let ground = matches!(def.kind, Kind::Ground | Kind::GroundSelf);
    if c.dot > 0.0 && def.kind != Kind::Dot && (!ground || burst) {
        let secs = if ground { 5.0 } else { c.duration };
        s += &format!(
            ", then {} damage every {} sec for {} sec",
            num(c.dot),
            num(c.modifiers.interval(1.0)),
            num(secs)
        );
    }
    if let Some(e) = effect_text(c.modifiers.effect(def.effect)) {
        s += ", ";
        s += &e;
    }
    s.push('.');
    s
}

fn ability_body_zh(def: &AbilityDef, c: &Cast) -> String {
    let damage = num(c.dmg);
    let burst = def.duration <= 0.0;
    let mut body = match def.kind {
        Kind::Bolt | Kind::Strike | Kind::Melee => {
            let toughest = matches!(
                def.key,
                "pyroblast" | "aimed_shot" | "exorcism" | "starfire"
            );
            let target = match (toughest, def.kind == Kind::Melee, c.count) {
                (true, _, 1) => "射程内最强的敌人".to_string(),
                (true, _, count) => format!("射程内最强的{count}名敌人"),
                (false, true, 1) => "最近的敌人".to_string(),
                (false, true, count) => format!("最近的{count}名敌人"),
                (false, false, 1) => "距离最近的敌人".to_string(),
                (false, false, count) => format!("距离最近的{count}名敌人"),
            };
            let mut text = format!(
                "{}，造成{}点伤害",
                if def.kind == Kind::Melee {
                    format!("攻击{target}")
                } else {
                    format!("命中{target}")
                },
                damage
            );
            if c.radius > 0.0 && def.kind != Kind::Melee {
                text += &format!("，并对命中点{}码内的敌人造成伤害", num(c.radius));
            }
            text
        }
        Kind::Nova => format!(
            "对你周围{}码内的所有敌人造成{}点伤害",
            num(c.radius),
            damage
        ),
        Kind::Cone => format!(
            "朝最近的敌人方向释放{}码锥形攻击，造成{}点伤害",
            num(c.radius),
            damage
        ),
        Kind::Ground | Kind::GroundSelf => {
            let at = if def.kind == Kind::GroundSelf {
                "你周围"
            } else {
                "敌人最密集处"
            };
            if burst {
                format!(
                    "攻击{at}{}码范围内的敌人，造成{}点伤害",
                    num(c.radius),
                    damage
                )
            } else {
                format!(
                    "覆盖{at}{}码范围，持续{}秒，每{}秒造成{}点伤害",
                    num(c.radius),
                    num(c.duration),
                    num(c.modifiers.interval(1.0)),
                    damage
                )
            }
        }
        Kind::Chain => format!(
            "对最近的敌人造成{}点伤害，随后在彼此{}码内跳跃至另外{}个目标，每次跳跃伤害降低{}",
            damage,
            num(c.radius),
            c.count,
            pct((0.15 - c.modifiers.chain).max(0.0))
        ),
        Kind::Dot => format!(
            "使射程内{}名敌人每{}秒受到{}点伤害，持续{}秒",
            c.count,
            num(c.modifiers.interval(1.0)),
            num(c.dot),
            num(c.duration)
        ),
        Kind::Aura => format!(
            "每{}秒对你周围{}码内的所有敌人造成{}点伤害",
            num(c.cooldown),
            num(c.radius),
            damage
        ),
        Kind::Orbit => format!(
            "每{}秒电击你周围{}码内的{}名敌人，造成{}点伤害",
            num(c.cooldown),
            num(c.radius),
            c.count,
            damage
        ),
        Kind::Shield => format!(
            "吸收你在{}秒内受到的下一次{}点伤害",
            num(c.duration),
            damage
        ),
        Kind::Immune => format!(
            "敌人靠近时，免疫所有伤害{}秒，并获得{}点吸收护盾，持续{}秒",
            num(c.duration),
            damage,
            num(c.duration + 3.0),
        ),
        Kind::Thorns => format!(
            "敌人靠近时，对所有攻击者造成{}点伤害，持续{}秒",
            damage,
            num(c.duration)
        ),
        Kind::Heal => format!(
            "生命值低于85%时，在{}秒内恢复{}点生命值",
            num(c.duration),
            damage
        ),
        Kind::Totem if def.key == "magma_totem" => format!(
            "放置图腾，持续{}秒；每{}秒灼烧{}码内的所有敌人，造成{}点伤害",
            num(c.duration),
            num(c.modifiers.interval(2.0)),
            num(c.radius),
            damage
        ),
        Kind::Totem => format!(
            "放置图腾，持续{}秒；每{}秒攻击{}码内最近的敌人，造成{}点伤害",
            num(c.duration),
            num(c.modifiers.interval(1.0)),
            num(c.range),
            damage
        ),
    };
    let ground = matches!(def.kind, Kind::Ground | Kind::GroundSelf);
    if c.dot > 0.0 && def.kind != Kind::Dot && (!ground || burst) {
        let secs = if ground { 5.0 } else { c.duration };
        body += &format!(
            "，之后每{}秒造成{}点伤害，持续{}秒",
            num(c.modifiers.interval(1.0)),
            num(c.dot),
            num(secs)
        );
    }
    if let Some(effect) = effect_text(c.modifiers.effect(def.effect)) {
        body.push('，');
        body.push_str(&effect);
    }
    body.push('。');
    body
}

/// What the next rank changes, at the current stats.
fn next_rank(def: &AbilityDef, now: &Cast, next: &Cast) -> String {
    if localization::is_zh_cn() {
        let mut gains = Vec::new();
        if next.dmg > now.dmg && def.kind != Kind::Dot {
            gains.push(format!(
                "{}{}",
                num(next.dmg),
                match def.kind {
                    Kind::Shield | Kind::Immune => "点吸收量",
                    Kind::Heal => "点治疗量",
                    _ => "点伤害",
                }
            ));
        }
        if next.dot > now.dot {
            gains.push(format!("每跳{}点伤害", num(next.dot)));
        }
        if next.count != now.count
            && matches!(
                def.kind,
                Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Dot | Kind::Chain | Kind::Orbit
            )
        {
            gains.push(match def.kind {
                Kind::Chain => format!("{}次跳跃", next.count),
                _ => format!("{}个目标", next.count),
            });
        }
        if next.radius > now.radius {
            gains.push(format!("{}码范围", num(next.radius)));
        }
        if next.duration > now.duration {
            gains.push(format!("持续{}秒", num(next.duration)));
        }
        gains.push(format!("冷却{}秒", num(next.cooldown)));
        return gains.join("，");
    }
    let what = match def.kind {
        Kind::Shield | Kind::Immune => "absorbed",
        Kind::Heal => "healing",
        _ => "damage",
    };
    let mut gains = Vec::new();
    if next.dmg > 0.0 && def.kind != Kind::Dot {
        gains.push(format!("{} {what}", num(next.dmg)));
    }
    if next.dot > 0.0 {
        gains.push(format!("{} damage per tick", num(next.dot)));
    }
    if next.count != now.count
        && matches!(
            def.kind,
            Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Dot | Kind::Chain | Kind::Orbit
        )
    {
        gains.push(match def.kind {
            Kind::Chain => format!("{} jumps", next.count),
            _ => format!("{} targets", next.count),
        });
    }
    if next.radius > 0.0 {
        gains.push(format!("{} yd", num(next.radius)));
    }
    if next.duration > 0.0 {
        gains.push(format!("{} sec", num(next.duration)));
    }
    gains.push(format!("{} sec cooldown", num(next.cooldown)));
    gains.join(", ")
}

fn ability_tip(
    o: &Owned,
    stats: &Stats,
    spells: Option<&crate::ui_action::Spells>,
    run: &Run,
    gear: Option<&super::gear::Gear>,
) -> Vec<TipLine> {
    let def = o.def;
    let mut cast_stats = *stats;
    let mut now = super::talents::cast_with_growth(def, o.level, o.growth, &cast_stats);
    if def.kind == Kind::Immune {
        now.dmg = o.growth;
    }
    super::class_mechanics::modify_cast(run, def.key, &mut now, &mut cast_stats);
    super::spell_expansion::modify_cast(run, def.key, &mut now, &mut cast_stats);
    let stats = &cast_stats;
    let mut lines = vec![
        TipLine::new(spell_name(spells, def.spell, def.key), Color::WHITE)
            .header()
            .right(
                if localization::is_zh_cn() {
                    format!("等级 {}", o.level)
                } else {
                    format!("Rank {}", o.level)
                },
                TIP_GREY,
            ),
    ];
    let range = match def.kind {
        Kind::Melee => Some(localization::text("Melee Range", "近战距离").to_string()),
        Kind::Bolt | Kind::Strike | Kind::Chain | Kind::Dot | Kind::Ground | Kind::Totem => {
            Some(if localization::is_zh_cn() {
                format!("射程{}码", num(now.range))
            } else {
                format!("{} yd range", num(now.range))
            })
        }
        _ => None,
    };
    let mut cast = TipLine::new(localization::text("Casts itself", "自动施放"), Color::WHITE);
    if let Some(range) = range {
        cast = cast.right(range, Color::WHITE);
    }
    lines.push(cast);
    lines.push(
        TipLine::new(localization::text("Instant", "瞬发"), Color::WHITE).right(
            if localization::is_zh_cn() {
                format!("冷却{}秒", num(now.cooldown))
            } else {
                format!("{} sec cooldown", num(now.cooldown))
            },
            Color::WHITE,
        ),
    );
    lines.push(TipLine::new(ability_body(def, &now), GOLD));
    lines.push(TipLine::new(
        localization::text(
            "Keep the pointer on this icon and scroll to read all enhancements.",
            "指针停留在图标上，滚动鼠标查看全部强化。",
        ),
        DIM,
    ));
    let periodic_crit = super::talents::periodic_crits(
        stats,
        super::skill_effects::Source {
            ability: Some(def.key),
            ..default()
        },
    );
    if periodic_crit
        || !matches!(
            def.kind,
            Kind::Shield | Kind::Immune | Kind::Heal | Kind::Dot
        )
    {
        lines.push(TipLine::new(
            if localization::is_zh_cn() {
                format!(
                    "{}几率造成{}倍暴击伤害",
                    pct(stats.crit),
                    num(stats.crit_multiplier())
                )
            } else {
                format!(
                    "{} chance to critically strike for {}x damage",
                    pct(stats.crit),
                    num(stats.crit_multiplier())
                )
            },
            DIM,
        ));
    }
    if periodic_crit {
        lines.push(TipLine::new(
            localization::text(
                "Each eligible damage-over-time tick rolls its own critical strike.",
                "符合天赋条件的每次持续伤害独立判定暴击。",
            ),
            DIM,
        ));
    }
    lines.extend(
        skill_descriptions::details(run, def, gear)
            .into_iter()
            .map(|line| TipLine::new(line, DIM)),
    );
    let next = super::growth::cast_for_run(def, o.level.saturating_add(1), o.growth + 1.0, run);
    lines.push(TipLine::new(
        format!(
            "{}: {:.2}",
            localization::text("Total growth", "累计成长"),
            o.growth
        ),
        GOLD,
    ));
    lines.push(TipLine::new(
        localization::text("Next common card (+1 growth):", "下张普通卡（+1成长）："),
        Color::WHITE,
    ));
    lines.push(TipLine::new(next_rank(def, &now, &next), GOLD));
    lines
}

/// What a passive's bonus `v` does, in the voice of a buff.
/// A stat's short name, for the card's before » after line.
fn stat_label(stat: Stat) -> &'static str {
    match stat {
        Stat::Might => localization::text("All damage", "全部伤害"),
        Stat::MaxHp => localization::text("Maximum health", "生命上限"),
        Stat::Speed => localization::text("Movement speed", "移动速度"),
        Stat::Regen => localization::text("Health per second", "每秒恢复生命"),
        Stat::Haste => localization::text("Recharge speed", "冷却速度"),
        Stat::Area => localization::text("Area size", "范围大小"),
        Stat::Magnet => localization::text("Pickup range", "拾取范围"),
        Stat::Growth => localization::text("Experience gained", "经验获取"),
        Stat::Armor => localization::text("Damage blocked per hit", "每次格挡伤害"),
        Stat::Crit => localization::text("Critical strike chance", "暴击几率"),
        Stat::Luck => localization::text("Luck", "幸运"),
        Stat::Targets => localization::text("Extra targets", "额外目标"),
        Stat::Echo => localization::text("Echo chance and power", "回响几率与威力"),
        Stat::Duration => localization::text("Duration", "持续时间"),
        Stat::Range => localization::text("Cast range", "施法距离"),
        Stat::CritDamage => localization::text("Critical damage", "暴击伤害"),
        Stat::Leech => localization::text("Life steal", "生命汲取"),
        Stat::Shield => localization::text("Shield power", "护盾强度"),
    }
}

fn passive_name(passive: &PassiveDef) -> &'static str {
    let chinese = skill_descriptions::ability_name(passive.key)
        .or_else(|| super::spell_expansion::name(passive.key))
        .unwrap_or(passive.name);
    localization::text(passive.name, chinese)
}

fn passive_text(stat: Stat, v: f32) -> String {
    let extra = match stat {
        Stat::Luck => Some(if localization::is_zh_cn() {
            format!("幸运成长+{}；收益渐减并增加伤害。", num(v))
        } else {
            format!(
                "+{} raw luck; diminishing fortune and additional damage.",
                num(v)
            )
        }),
        Stat::Targets => Some(if localization::is_zh_cn() {
            format!(
                "目标成长+{}；每3点增加1个目标，每点额外提高3%伤害。",
                num(v)
            )
        } else {
            format!(
                "+{} target growth; +1 target per 3 points and +3% damage per point.",
                num(v)
            )
        }),
        Stat::Echo => Some(if localization::is_zh_cn() {
            format!(
                "回响成长+{}；几率渐近50%，额外成长继续提高回响伤害。",
                pct(v)
            )
        } else {
            format!(
                "+{} echo growth; chance approaches 50% and echo damage keeps growing.",
                pct(v)
            )
        }),
        Stat::Duration => Some(format!("{} +{}.", stat_label(stat), pct(v))),
        Stat::Range => Some(if localization::is_zh_cn() {
            format!("射程成长+{}，收益渐减并持续提高伤害。", pct(v))
        } else {
            format!(
                "+{} range growth, with diminishing reach and additional damage.",
                pct(v)
            )
        }),
        Stat::CritDamage => Some(format!("{} +{}.", stat_label(stat), pct(v))),
        Stat::Leech => Some(if localization::is_zh_cn() {
            format!("汲取成长+{}，几率收益渐减并持续提高生命恢复。", pct(v))
        } else {
            format!(
                "+{} leech growth, with diminishing life steal and additional regeneration.",
                pct(v)
            )
        }),
        Stat::Shield => Some(format!("{} +{}.", stat_label(stat), pct(v))),
        _ => None,
    };
    if let Some(text) = extra {
        return text;
    }
    if localization::is_zh_cn() {
        return match stat {
            Stat::Might => format!("造成的所有伤害提高{}。", pct(v)),
            Stat::MaxHp => format!("生命上限提高{}。", pct(v)),
            Stat::Speed => format!("移动速度提高{}。", pct(v)),
            Stat::Regen => format!("每秒恢复{}点生命值。", num(v)),
            Stat::Haste => format!("所有技能冷却速度提高{}。", pct(v)),
            Stat::Area => format!("范围效果扩大{}。", pct(v)),
            Stat::Magnet => format!("拾取范围扩大{}。", pct(v)),
            Stat::Growth => format!("获得的经验提高{}。", pct(v)),
            Stat::Armor => format!("每次受到攻击时减免{}点伤害。", num(v)),
            Stat::Crit => format!("造成双倍暴击伤害的几率提高{}。", pct(v)),
            _ => unreachable!("additional passive stats were described above"),
        };
    }
    match stat {
        Stat::Might => format!("Increases all damage you deal by {}.", pct(v)),
        Stat::MaxHp => format!("Increases your maximum health by {}.", pct(v)),
        Stat::Speed => format!("Increases your movement speed by {}.", pct(v)),
        Stat::Regen => format!("Restores {} health every second.", num(v)),
        Stat::Haste => format!("Your abilities recharge {} faster.", pct(v)),
        Stat::Area => format!("Increases the size of your areas of effect by {}.", pct(v)),
        Stat::Magnet => format!("Increases your pickup range by {}.", pct(v)),
        Stat::Growth => format!("Increases the experience you gain by {}.", pct(v)),
        Stat::Armor => format!("Reduces the damage of every hit you take by {}.", num(v)),
        Stat::Crit => format!(
            "Increases your chance to critically strike for double damage by {}.",
            pct(v)
        ),
        _ => unreachable!("additional passive stats were described above"),
    }
}

/// The hero's total for the stat a passive raises, every source counted.
fn stat_total(stat: Stat, s: &Stats) -> String {
    let extra = match stat {
        Stat::Luck => Some(format!(
            "{}: {:.1} ({} {:.1}) · {}",
            stat_label(stat),
            s.luck,
            localization::text("raw", "原始"),
            s.luck_raw,
            stat_total(Stat::Might, s),
        )),
        Stat::Targets => Some(format!(
            "{}: +{} · {}",
            stat_label(stat),
            s.targets,
            stat_total(Stat::Might, s),
        )),
        Stat::Echo => Some(format!(
            "{}: {} / {}",
            stat_label(stat),
            pct(s.echo),
            pct(0.5 + s.echo_damage),
        )),
        Stat::Duration => Some(format!(
            "{}: {} · {}",
            stat_label(stat),
            pct(1.0 + s.duration),
            stat_total(Stat::Might, s),
        )),
        Stat::Range => Some(format!(
            "{}: {} · {}",
            stat_label(stat),
            pct(1.0 + s.range),
            stat_total(Stat::Might, s),
        )),
        Stat::CritDamage => Some(format!(
            "{}: {}×",
            stat_label(stat),
            num(s.crit_multiplier()),
        )),
        Stat::Leech => Some(format!(
            "{}: {} · {}",
            stat_label(stat),
            pct(s.leech),
            stat_total(Stat::Regen, s),
        )),
        Stat::Shield => Some(format!("{}: {}", stat_label(stat), pct(s.shield_power))),
        _ => None,
    };
    if let Some(text) = extra {
        return text;
    }
    if localization::is_zh_cn() {
        return match stat {
            Stat::Might => format!("伤害加成：{}", pct(s.might)),
            Stat::MaxHp => format!("生命上限：{}", num(s.max_hp)),
            Stat::Speed => format!(
                "移动速度：{}码/秒；生命恢复：{:.2}/秒",
                num(s.speed),
                s.regen
            ),
            Stat::Regen => format!("生命恢复：{}点/秒", num(s.regen)),
            Stat::Haste => format!(
                "冷却缩减：{}；伤害：{}",
                pct(s.haste.min(0.65)),
                pct(s.might)
            ),
            Stat::Area => format!("范围效果：{}；伤害：{}", pct(s.area), pct(s.might)),
            Stat::Magnet => format!("拾取范围：{}码；经验：{}", num(s.magnet), pct(s.growth)),
            Stat::Growth => format!("经验获取：{}", pct(s.growth)),
            Stat::Armor => format!("每次格挡伤害：{}", num(s.armor)),
            Stat::Crit => format!(
                "暴击几率：{}；暴击伤害：{}倍",
                pct(s.crit),
                num(s.crit_multiplier())
            ),
            _ => unreachable!("additional passive stats were described above"),
        };
    }
    match stat {
        Stat::Might => format!("Damage dealt: {}", pct(s.might)),
        Stat::MaxHp => format!("Maximum health: {}", num(s.max_hp)),
        Stat::Speed => format!(
            "Speed: {} yd/s; regeneration: {:.2}/s",
            num(s.speed),
            s.regen
        ),
        Stat::Regen => format!("Health restored: {} per second", num(s.regen)),
        Stat::Haste => format!(
            "Cooldowns: {} shorter; damage: {}",
            pct(s.haste.min(0.65)),
            pct(s.might)
        ),
        Stat::Area => format!("Area: {}; damage: {}", pct(s.area), pct(s.might)),
        Stat::Magnet => format!(
            "Pickup: {} yd; experience: {}",
            num(s.magnet),
            pct(s.growth)
        ),
        Stat::Growth => format!("Experience gained: {}", pct(s.growth)),
        Stat::Armor => format!("Damage blocked per hit: {}", num(s.armor)),
        Stat::Crit => format!(
            "Critical chance: {}; damage: {}×",
            pct(s.crit),
            num(s.crit_multiplier())
        ),
        _ => unreachable!("additional passive stats were described above"),
    }
}

fn passive_preview(p: &PassiveDef, added: f32, run: &Run) -> String {
    let before = run.stats();
    let after = run.stats_with_passive_growth(p, added);
    format!(
        "{}\n↓\n{}\n{} {:.2} → {:.2}",
        stat_total(p.stat, &before),
        stat_total(p.stat, &after),
        localization::text("Total growth", "累计成长"),
        run.passive_growth(p),
        run.passive_growth(p) + added,
    )
}

fn passive_tip(p: &PassiveDef, level: u32, stats: &Stats, run: &Run) -> Vec<TipLine> {
    vec![
        TipLine::new(passive_name(p), Color::WHITE).header().right(
            if localization::is_zh_cn() {
                format!("等级 {level}")
            } else {
                format!("Rank {level}")
            },
            TIP_GREY,
        ),
        TipLine::new(
            passive_text(p.stat, p.per_level * run.passive_growth(p)),
            GOLD,
        ),
        TipLine::new(stat_total(p.stat, stats), Color::WHITE),
        TipLine::new(
            localization::text("Next common card (+1 growth):", "下张普通卡（+1成长）："),
            Color::WHITE,
        ),
        TipLine::new(
            stat_total(p.stat, &run.stats_with_passive_growth(p, 1.0)),
            GOLD,
        ),
        TipLine::new(
            format!(
                "{}: {:.2}",
                localization::text("Total growth", "累计成长"),
                run.passive_growth(p)
            ),
            DIM,
        ),
    ]
}

/// The `GameTooltip` frame: `UI-Tooltip-Background` tiled under `UI-Tooltip-Border`, 16-unit edges
/// and 5-unit insets, its lines 10 units in. Returns the frame and its line column.
fn spawn_tooltip(
    commands: &mut Commands,
    assets: &mut benilla_assets::WorldAssets,
    images: &mut Assets<Image>,
    ui_scale: f32,
) -> (Entity, Entity) {
    use crate::glue::backdrop::{backdrop_border, backdrop_edges, tiled_bg_node};
    let bg = assets.sprite_texture("Interface\\Tooltips\\UI-Tooltip-Background", images);
    let edges = backdrop_edges(assets, "Interface\\Tooltips\\UI-Tooltip-Border", images);
    let mut body = Entity::PLACEHOLDER;
    let mut root = commands.spawn((
        GameTooltip,
        Node {
            position_type: PositionType::Absolute,
            max_width: Val::Px(300.0),
            ..default()
        },
        GlobalZIndex(1000),
        Visibility::Hidden,
        bevy::ui::FocusPolicy::Pass,
    ));
    if bg.is_none() || edges.is_none() {
        root.insert(BackgroundColor(TIP_FILL.with_alpha(0.95)));
    }
    root.with_children(|t| {
        if let Some(bg) = bg {
            t.spawn((
                tiled_bg_node(bg, 16.0, ui_scale, TIP_FILL),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(5.0),
                    right: Val::Px(5.0),
                    top: Val::Px(5.0),
                    bottom: Val::Px(5.0),
                    ..default()
                },
            ));
        }
        if let Some(edges) = &edges {
            backdrop_border(t, edges, 16.0, Color::WHITE);
        }
        body = t
            .spawn((
                TipBody,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(2.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    overflow: Overflow::scroll_y(),
                    min_height: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                ScrollPosition::default(),
            ))
            .id();
    });
    (root.id(), body)
}

/// Hovering a build icon raises its `GameTooltip` under the icon, every number from the current
/// rank and stats, and what the next rank brings.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(super) fn icon_tooltip(
    mut commands: Commands,
    phase: Res<State<Phase>>,
    run: Res<Run>,
    gear: Option<Res<super::gear::Gear>>,
    spells: Option<Res<crate::ui_action::Spells>>,
    font: Res<HudFont>,
    ui_scale: Res<UiScale>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: ResMut<Assets<Image>>,
    slots: Query<(
        &Interaction,
        &TipFor,
        &ComputedNode,
        &bevy::ui::UiGlobalTransform,
    )>,
    mut frames: Query<
        (&mut Node, &mut Visibility, &ComputedNode),
        (With<GameTooltip>, Without<TipBody>),
    >,
    mut bodies: Query<(&mut Node, &ComputedNode, &mut ScrollPosition), With<TipBody>>,
    mut viewport: (
        Query<&Window, With<PrimaryWindow>>,
        MessageReader<bevy::input::mouse::MouseWheel>,
    ),
    mut built: Local<Option<(Entity, Entity)>>,
    mut shown: Local<Option<(TipFor, u32, u32)>>,
) {
    let Some((frame, body)) = *built else {
        if let Some(assets) = assets.as_deref_mut() {
            *built = Some(spawn_tooltip(
                &mut commands,
                assets,
                &mut images,
                ui_scale.0,
            ));
        }
        return;
    };
    let Ok((mut node, mut vis, frame_size)) = frames.get_mut(frame) else {
        return;
    };
    let scroll_delta: f32 = viewport
        .1
        .read()
        .map(|event| {
            if event.unit == bevy::input::mouse::MouseScrollUnit::Line {
                event.y * 42.0
            } else {
                event.y
            }
        })
        .sum();
    let hovered = slots
        .iter()
        .find(|(i, ..)| **i != Interaction::None)
        .filter(|_| *phase.get() != Phase::Menu);
    let Some((_, &tip, computed, at)) = hovered else {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
        }
        *shown = None;
        return;
    };
    // Physical pixels to the 768-unit canvas the `Val`s are on.
    let inv = computed.inverse_scale_factor;
    let screen = viewport.0.single().map_or(Vec2::new(1024.0, 768.0), |w| {
        Vec2::new(w.width(), w.height()) / ui_scale.0
    });
    let width = 420.0_f32.min((screen.x - 16.0).max(1.0));
    let height = (screen.y - 16.0).max(1.0);
    node.width = Val::Px(width);
    node.max_width = Val::Px(width);
    node.max_height = Val::Px(height);
    let at = Vec2::new(
        (at.translation.x - computed.size().x / 2.0) * inv,
        (at.translation.y + computed.size().y / 2.0) * inv + 4.0,
    );
    let anchor = tooltip_anchor(screen, at, Vec2::new(width, frame_size.size().y * inv));
    let (left, top) = (Val::Px(anchor.x), Val::Px(anchor.y));
    if node.left != left || node.top != top {
        node.left = left;
        node.top = top;
    }
    if let Ok((mut body_node, computed, mut scroll)) = bodies.get_mut(body) {
        body_node.max_height = Val::Px(height);
        if shown.is_none_or(|(previous, ..)| previous != tip) {
            scroll.0.y = 0.0;
        }
        let max =
            ((computed.content_size.y - computed.size.y) * computed.inverse_scale_factor).max(0.0);
        scroll.0.y = (scroll.0.y - scroll_delta).clamp(0.0, max);
    }
    let revision = (tip, run.version, (run.elapsed * 5.0) as u32);
    if *shown == Some(revision) {
        *vis = Visibility::Inherited;
        return;
    }
    let stats = run.stats();
    let lines = match tip {
        TipFor::Ability(i) => run
            .abilities
            .get(i)
            .map(|o| ability_tip(o, &stats, spells.as_deref(), &run, gear.as_deref())),
        TipFor::Passive(i) => run
            .passives
            .get(i)
            .map(|&(p, level)| passive_tip(p, level, &stats, &run)),
        TipFor::Mobility => run.mobility().map(|m| {
            let mut lines = vec![
                TipLine::new(spell_name(spells.as_deref(), m.spell, m.name), Color::WHITE).header(),
                TipLine::new(localization::text("Press F", "按 F"), Color::WHITE).right(
                    if localization::is_zh_cn() {
                        format!("冷却{}秒", num(m.cooldown * run.mobility_cooldown()))
                    } else {
                        format!("{} sec cooldown", num(m.cooldown * run.mobility_cooldown()))
                    },
                    Color::WHITE,
                ),
                TipLine::new(skill_descriptions::mobility_description(run.class.id), GOLD),
                TipLine::new(
                    localization::text(
                        "Keep the pointer on the icon and scroll for details.",
                        "指针停留在图标上，滚动鼠标查看详情。",
                    ),
                    DIM,
                ),
            ];
            lines.extend(
                skill_descriptions::mobility_details(&run, gear.as_deref())
                    .into_iter()
                    .map(|line| TipLine::new(line, DIM)),
            );
            lines
        }),
    };
    let Some(lines) = lines else { return };
    *shown = Some(revision);
    // Shown from the next frame, once the new lines have laid out.
    *vis = Visibility::Hidden;
    let f = &font.0;
    commands.entity(body).despawn_related::<Children>();
    commands.entity(body).with_children(|b| {
        for line in lines {
            let face = |s: String, size: f32, color: Color| {
                (
                    Text::new(s),
                    TextFont {
                        font: f.clone(),
                        font_size: size,
                        ..default()
                    },
                    TextColor(color),
                    TextShadow {
                        offset: Vec2::splat(1.0),
                        color: Color::BLACK,
                    },
                )
            };
            b.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                column_gap: Val::Px(16.0),
                flex_shrink: 0.0,
                width: Val::Percent(100.0),
                min_width: Val::Px(0.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    face(line.left, line.size, line.color),
                    Node {
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                ));
                if let Some((text, color)) = line.right {
                    row.spawn((
                        face(text, line.size, color),
                        TextLayout::new(Justify::Right, LineBreak::NoWrap),
                    ));
                }
            });
        }
    });
}

fn tooltip_anchor(screen: Vec2, requested: Vec2, size: Vec2) -> Vec2 {
    let margin = 8.0;
    Vec2::new(
        requested
            .x
            .clamp(margin, (screen.x - size.x - margin).max(margin)),
        requested
            .y
            .clamp(margin, (screen.y - size.y - margin).max(margin)),
    )
}

#[allow(clippy::type_complexity)]
pub(super) fn show_panels(
    phase: Res<State<Phase>>,
    mut roots: ParamSet<(
        Query<(&mut Visibility, &mut Node), With<MenuRoot>>,
        Query<(&mut Visibility, &mut Node), With<HudRoot>>,
        Query<(&mut Visibility, &mut Node), With<LevelUpRoot>>,
        Query<(&mut Visibility, &mut Node), With<PauseRoot>>,
        Query<(&mut Visibility, &mut Node), With<OverRoot>>,
    )>,
) {
    let p = *phase.get();
    // A hidden panel leaves the layout too: a merely invisible button still takes clicks.
    let set = |(mut v, mut node): (Mut<Visibility>, Mut<Node>), on: bool| {
        let want = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    };
    for root in &mut roots.p0() {
        set(root, p == Phase::Menu);
    }
    for root in &mut roots.p1() {
        set(root, p != Phase::Menu && p != Phase::GameOver);
    }
    for root in &mut roots.p2() {
        set(root, p == Phase::LevelUp);
    }
    for root in &mut roots.p3() {
        set(root, p == Phase::Paused);
    }
    for root in &mut roots.p4() {
        set(root, p == Phase::GameOver);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take_card(run: &mut Run, choice: Choice, quality: Rarity) {
        let mut offer = Offer {
            choices: vec![choice],
            qualities: vec![quality],
        };
        let mut rng = Rng(42);
        let mut next = NextState::Unchanged;
        pick(0, run, &mut offer, &mut rng, &mut next);
    }

    #[test]
    fn initial_learning_and_same_rank_upgrades_use_frozen_quality() {
        let def = data::ability("fireball");
        let mut common = Run::default();
        let mut legendary = Run::default();
        take_card(&mut common, Choice::NewAbility(def), Rarity::Common);
        take_card(&mut legendary, Choice::NewAbility(def), Rarity::Legendary);
        assert_eq!(common.abilities[0].level, 1);
        assert_eq!(legendary.abilities[0].level, 1);
        assert!(legendary.abilities[0].damage_mult() > common.abilities[0].damage_mult());
        take_card(&mut common, Choice::Upgrade(0), Rarity::Common);
        take_card(&mut legendary, Choice::Upgrade(0), Rarity::Legendary);
        assert_eq!(common.abilities[0].level, legendary.abilities[0].level);
        assert_eq!(common.abilities[0].growth, 2.0);
        assert_eq!(legendary.abilities[0].growth, 6.4);
        let offer = Offer {
            choices: vec![Choice::Upgrade(0)],
            qualities: vec![Rarity::Epic],
        };
        let first = describe_card(offer.choices[0], offer.quality(0), &common, None);
        for _ in 0..10 {
            let next = describe_card(offer.choices[0], offer.quality(0), &common, None);
            assert_eq!(first.desc, next.desc);
            assert_eq!(offer.quality(0), Rarity::Epic);
        }
    }

    #[test]
    fn ability_and_passive_growth_continue_after_255() {
        let mut run = Run::default();
        take_card(
            &mut run,
            Choice::NewAbility(data::ability("fireball")),
            Rarity::Common,
        );
        take_card(
            &mut run,
            Choice::NewPassive(&data::PASSIVES[0]),
            Rarity::Common,
        );
        for _ in 1..300 {
            take_card(&mut run, Choice::Upgrade(0), Rarity::Common);
            take_card(&mut run, Choice::UpPassive(0), Rarity::Common);
        }
        assert_eq!(run.abilities[0].level, 300);
        assert_eq!(run.passives[0].1, 300);
        assert_eq!(run.abilities[0].growth, 300.0);
        assert_eq!(run.passive_growth(&data::PASSIVES[0]), 300.0);
        let before = run.stats().might;
        take_card(&mut run, Choice::UpPassive(0), Rarity::Legendary);
        assert!(run.stats().might > before);
    }

    #[test]
    fn a_full_build_still_offers_upgrades_beyond_old_caps() {
        let mut run = Run::default();
        run.abilities = run
            .class
            .pool
            .iter()
            .take(data::MAX_ABILITIES)
            .map(|key| Owned {
                def: data::ability(key),
                level: 300,
                growth: 300.0,
                cd: 0.0,
            })
            .collect();
        run.passives = data::PASSIVES
            .iter()
            .take(data::MAX_PASSIVES)
            .map(|p| (p, 300))
            .collect();
        let mut rng = Rng(42);
        for _ in 0..100 {
            let choices = choices_for(&run, &mut rng);
            assert_eq!(choices.len(), 3);
            assert!(choices
                .iter()
                .all(|c| matches!(c, Choice::Upgrade(_) | Choice::UpPassive(_))));
        }
    }

    #[test]
    fn normal_offers_acquire_eight_active_twelve_passive_slots_then_only_upgrade() {
        let mut run = Run::default();
        let mut rng = Rng(42);
        let mut offer = Offer::default();
        let mut next = NextState::Unchanged;
        for _ in 0..10_000 {
            offer.roll(&run, &mut rng);
            if run.abilities.len() == 8 {
                assert!(!offer
                    .choices()
                    .iter()
                    .any(|c| matches!(c, Choice::NewAbility(_))));
            }
            if run.passives.len() == 12 {
                assert!(!offer
                    .choices()
                    .iter()
                    .any(|c| matches!(c, Choice::NewPassive(_))));
            }
            let pick_index = offer.choices().iter().position(|choice| match choice {
                Choice::NewAbility(_) => run.abilities.len() < 8,
                Choice::NewPassive(_) => run.passives.len() < 12,
                _ => false,
            });
            if let Some(index) = pick_index {
                pick(index, &mut run, &mut offer, &mut rng, &mut next);
            }
            if run.abilities.len() == 8 && run.passives.len() == 12 {
                break;
            }
        }
        assert_eq!(run.abilities.len(), 8);
        assert_eq!(run.passives.len(), 12);
        for owned in &mut run.abilities {
            owned.level = 300;
        }
        for (_, level) in &mut run.passives {
            *level = 300;
        }
        let mut upgraded_active = false;
        let mut upgraded_passive = false;
        for _ in 0..100 {
            offer.roll(&run, &mut rng);
            assert_eq!(offer.len(), 3);
            assert!(offer
                .choices()
                .iter()
                .all(|c| matches!(c, Choice::Upgrade(_) | Choice::UpPassive(_))));
            let choice = offer.choices()[0];
            match choice {
                Choice::Upgrade(index) => {
                    let before = run.abilities[index].level;
                    pick(0, &mut run, &mut offer, &mut rng, &mut next);
                    assert_eq!(run.abilities[index].level, before + 1);
                    upgraded_active = true;
                }
                Choice::UpPassive(index) => {
                    let before = run.passives[index].1;
                    pick(0, &mut run, &mut offer, &mut rng, &mut next);
                    assert_eq!(run.passives[index].1, before + 1);
                    upgraded_passive = true;
                }
                _ => unreachable!("a full build must only offer upgrades"),
            }
        }
        assert!(upgraded_active && upgraded_passive);
        assert_eq!(run.abilities.len(), 8);
        assert_eq!(run.passives.len(), 12);
    }

    #[test]
    fn passive_targets_accumulate_fractional_quality_before_milestones() {
        let p = data::PASSIVES
            .iter()
            .find(|p| p.stat == Stat::Targets)
            .unwrap();
        let mut run = Run::default();
        take_card(&mut run, Choice::NewPassive(p), Rarity::Uncommon);
        assert_eq!(run.stats().targets, 0);
        let first_damage = run.stats().might;
        take_card(&mut run, Choice::UpPassive(0), Rarity::Rare);
        assert!((run.passive_growth(p) - 3.15).abs() < 0.0001);
        assert_eq!(run.stats().targets, 1);
        assert!(run.stats().might > first_damage);
    }

    #[test]
    fn every_passive_has_meaningful_late_growth_and_an_exact_preview() {
        for p in data::PASSIVES {
            let mut run = Run::default();
            run.passives.push((p, 300));
            run.passive_growth.insert(p.key, 500.0);
            let expected = run.stats_with_passive_growth(p, Rarity::Legendary.growth());
            let before = stat_total(p.stat, &run.stats());
            take_card(&mut run, Choice::UpPassive(0), Rarity::Legendary);
            assert_eq!(
                stat_total(p.stat, &run.stats()),
                stat_total(p.stat, &expected),
                "{}",
                p.key
            );
            assert_ne!(
                before,
                stat_total(p.stat, &run.stats()),
                "late growth should be visible: {}",
                p.key
            );
        }
    }

    #[test]
    fn objective_changes_to_final_battle_without_claiming_timed_victory() {
        let mut run = Run::default();
        let before = battle_objective(&run);
        assert!(before.contains("10:00") && before.contains("0/5"));
        run.elapsed = data::VICTORY_SECS;
        run.boss_kills = 4;
        let final_battle = battle_objective(&run);
        assert!(final_battle.contains(localization::text("Final battle", "最终决战")));
        assert!(final_battle.contains("4/5"));
        assert!(!run.won);
        run.final_boss_defeated = true;
        run.boss_kills = 5;
        let defeated = battle_objective(&run);
        assert!(defeated.contains(localization::text("Final boss defeated", "最终首领已击败")));
        assert!(defeated.contains("5/5"));
    }

    #[test]
    fn tooltip_position_stays_inside_small_and_large_viewports() {
        for screen in [Vec2::new(360.0, 768.0), Vec2::new(1366.0, 768.0)] {
            let size = Vec2::new(420.0_f32.min(screen.x - 16.0), screen.y - 16.0);
            for requested in [Vec2::splat(-200.0), Vec2::new(2000.0, 2000.0)] {
                let at = tooltip_anchor(screen, requested, size);
                assert!(at.x >= 8.0 && at.y >= 8.0);
                assert!(at.x + size.x <= screen.x - 8.0);
                assert!(at.y + size.y <= screen.y - 8.0);
            }
        }
    }

    #[test]
    fn final_reward_and_save_failures_remain_visible_even_on_defeat() {
        let run = Run {
            won: false,
            final_boss_defeated: true,
            final_reward: Some("Primal Iron Helm".into()),
            reward_error: Some("Reward could not be saved".into()),
            ..default()
        };
        let mut gear = super::super::gear::Gear::default();
        gear.error = Some("Disk write failed; retry pending".into());
        let mut summary = String::new();
        append_reward_summary(&mut summary, &run, Some(&gear));
        assert!(summary.contains("Primal Iron Helm"));
        assert!(summary.contains("Reward could not be saved"));
        assert!(summary.contains("Disk write failed; retry pending"));
        assert!(!run.won);
    }

    #[test]
    fn results_buttons_return_to_menu_without_exiting_and_pause_keeps_quit() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::asset::AssetPlugin::default()))
            .init_asset::<Font>()
            .init_asset::<Image>()
            .init_asset::<SweepUiMaterial>()
            .init_asset::<super::super::fill::FillMaterial>()
            .insert_resource(State::new(Phase::GameOver))
            .init_resource::<NextState<Phase>>()
            .init_resource::<super::super::gear_ui::GearUi>()
            .add_message::<AppExit>()
            .add_systems(Startup, setup_hud)
            .add_systems(Update, buttons);
        app.update();
        let world = app.world_mut();
        let root = world
            .query_filtered::<Entity, With<OverRoot>>()
            .single(world)
            .unwrap();
        let actions: Vec<_> = world
            .get::<Children>(root)
            .unwrap()
            .iter()
            .filter_map(|e| world.get::<UiAction>(e).map(|a| (e, *a)))
            .collect();
        assert_eq!(actions.len(), 2);
        assert!(actions.iter().all(|(_, a)| matches!(a, UiAction::MainMenu)));
        assert!(world
            .query::<(&UiAction, &Interaction)>()
            .iter(world)
            .any(|(a, _)| matches!(a, UiAction::Quit)));
        world.entity_mut(actions[1].0).insert(Interaction::Pressed);
        app.update();
        assert!(matches!(
            app.world().resource::<NextState<Phase>>(),
            NextState::Pending(Phase::Menu)
        ));
        assert!(app.world().resource::<Messages<AppExit>>().is_empty());
    }
}
