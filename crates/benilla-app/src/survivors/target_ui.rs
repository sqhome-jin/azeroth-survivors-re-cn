//! Enemy nameplates and the selectable target frame.
//!
//! Every live enemy on screen wears a pooled health plate (64×6 logical units, 120×12 with a name
//! for a boss); the pool only grows, and a plate node is written only when its value changes. A
//! left click on an enemy's projected body or its plate selects it, Tab cycles the nearby ones,
//! a click on empty ground clears. The selection is observation only: no spell priority reads it.
//! The target frame shows a live model portrait (a clone of the unit on its own render layer,
//! filmed into an image as [`super::preview`] films the menu hero), the name, level and rank,
//! health, a boss's cast and the debuffs the run has put on it.

use std::collections::HashMap;

use benilla_world::schedule::WorldStage;
use benilla_world::view::WorldCamera;
use bevy::app::Propagate;
use bevy::camera::primitives::Aabb;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;

use super::bosses::{self, Boss};
use super::data::{self, EnemyDef};
use super::enemies::Enemy;
use super::hero::Hero;
use super::localization::{is_zh_cn, text};
use super::units::{self, UnitSpec};
use super::{Phase, Run, Selection};

/// A plain enemy plate, logical UI units.
pub(super) const PLATE_W: f32 = 64.0;
pub(super) const PLATE_H: f32 = 6.0;
/// A boss plate.
pub(super) const BOSS_W: f32 = 120.0;
pub(super) const BOSS_H: f32 = 12.0;
/// Between the top of a body and the bottom of its plate.
const PLATE_GAP: f32 = 4.0;
/// The boss name rides this far over its plate.
const BOSS_NAME_H: f32 = 15.0;
/// The target frame, logical UI units from the top-left.
pub(super) const PANEL_LEFT: f32 = 20.0;
pub(super) const PANEL_TOP: f32 = 15.0;
pub(super) const PANEL_W: f32 = 300.0;
pub(super) const PANEL_H: f32 = 80.0;
/// A press that travels further than this (window logical px) is a camera drag, not a click.
pub(super) const DRAG_THRESHOLD: f32 = 6.0;
/// Seconds between two accepted clicks.
pub(super) const DEBOUNCE: f64 = 0.12;
/// Tab only cycles enemies within this many yards of the hero.
pub(super) const TAB_RANGE: f32 = 45.0;
/// Plate slots spawned at once when the pool runs short.
const POOL_GROW: usize = 16;
const DEBUFF_SLOTS: usize = 8;
/// Clear of the menu preview's 40, the world's 0, the UI pass's 1 and every portrait, glue,
/// minimap and UI-model layer (`2..=UI_MODEL_CAM_LAYER_BASE + UI_MODEL_CAM_LAYERS`).
pub(super) const PORTRAIT_LAYER: usize = 41;
const _: () = assert!(
    PORTRAIT_LAYER
        > crate::portrait::UI_MODEL_CAM_LAYER_BASE + crate::portrait::UI_MODEL_CAM_LAYERS
        && PORTRAIT_LAYER != 40
);
/// The portrait camera's vertical field of view.
const PORTRAIT_FOV: f32 = 0.55;
const PORTRAIT_PX: u32 = 128;

const PLATE_RED: Color = Color::srgb(0.86, 0.08, 0.06);
const PLATE_BACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.72);
const GOLD: Color = Color::srgb(1.0, 0.82, 0.0);
const EDGE: Color = Color::srgb(0.55, 0.45, 0.25);
const HP_GREEN: Color = Color::srgb(0.1, 0.78, 0.1);
const TIP_FILL: Color = Color::srgba(0.09, 0.09, 0.19, 0.95);

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SelectedTarget>()
        .init_resource::<ScreenCache>()
        .init_resource::<ClickTracker>()
        .add_systems(Startup, (setup_plates, setup_panel))
        .add_systems(
            Update,
            (
                project_enemies,
                select_input,
                validate_target,
                sync_plates,
                sync_panel,
                sync_portrait,
            )
                .chain()
                .after(WorldStage::Input)
                .after(super::abilities::apply_hits),
        )
        .add_systems(OnEnter(Phase::Menu), clear_target)
        .add_systems(
            PostUpdate,
            finish_portrait_appear
                .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
        );
}

// ── Resources and markers ───────────────────────────────────────────────────────────────────────

/// The enemy the player is looking at. Observation only: nothing in combat reads it.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct SelectedTarget {
    pub entity: Option<Entity>,
    /// Bumped on every change, so a consumer can rebuild once.
    pub generation: u32,
}

impl SelectedTarget {
    pub fn get(&self) -> Option<Entity> {
        self.entity
    }

    pub fn set(&mut self, entity: Option<Entity>) {
        if self.entity != entity {
            self.entity = entity;
            self.generation = self.generation.wrapping_add(1);
        }
    }

    pub fn clear(&mut self) {
        self.set(None);
    }
}

/// One live enemy as it falls on the screen this frame, window logical px.
#[derive(Clone, Copy)]
pub(super) struct OnScreen {
    pub entity: Entity,
    pub def: &'static EnemyDef,
    pub boss: bool,
    /// The projected body.
    pub body: Rect,
    /// The plate (and a boss's name over it).
    pub plate: Rect,
    /// Distance from the eye, for the front-most pick.
    pub depth: f32,
    /// Ground distance from the hero, for Tab.
    pub hero_dist: f32,
    pub hp: f32,
    pub max_hp: f32,
}

/// The live, on-screen enemies of the frame, shared by picking and the plates.
#[derive(Resource, Default)]
pub(super) struct ScreenCache {
    pub entries: Vec<OnScreen>,
    /// The UI scale the plate rects were built with.
    pub scale: f32,
}

#[derive(Resource, Default)]
pub(super) struct ClickTracker {
    press: Option<Vec2>,
    travel: f32,
    blocked: bool,
    last: Option<f64>,
}

/// Every node this module owns: its queries are filtered to it, so they never touch the HUD.
#[derive(Component)]
pub(super) struct TargetUiNode;
#[derive(Component)]
pub(super) struct PlateFrame;
#[derive(Component)]
pub(super) struct TargetPanel;
#[derive(Component)]
pub(super) struct DebuffSlot(pub usize);
/// The model clone filmed for the portrait.
#[derive(Component)]
pub(super) struct PortraitClone;

#[derive(Clone, Copy)]
struct PlateSlot {
    frame: Entity,
    fill: Entity,
    /// Boss plates only.
    name: Option<Entity>,
    hp: Option<Entity>,
}

#[derive(Resource)]
pub(super) struct PlatePool {
    root: Entity,
    normal: Vec<PlateSlot>,
    boss: Vec<PlateSlot>,
    normal_used: usize,
    boss_used: usize,
    ring: Entity,
    ring_pct: Entity,
    font: Handle<Font>,
}

#[derive(Resource)]
pub(super) struct PanelUi {
    root: Entity,
    name: Entity,
    level: Entity,
    hp_fill: Entity,
    hp_text: Entity,
    cast: Entity,
    cast_fill: Entity,
    cast_text: Entity,
    debuffs: Vec<(Entity, Entity, Entity)>,
    tooltip: Entity,
    tooltip_text: Entity,
}

#[derive(Resource)]
pub(super) struct PortraitRig {
    camera: Entity,
    clone: Option<Entity>,
    for_target: Option<Entity>,
}

// ── Pure helpers ────────────────────────────────────────────────────────────────────────────────

/// Health as a fraction, 0 for any degenerate maximum.
pub(super) fn hp_frac(hp: f32, max: f32) -> f32 {
    if !(max.is_finite() && max > 0.0 && hp.is_finite()) {
        return 0.0;
    }
    (hp / max).clamp(0.0, 1.0)
}

fn compact(v: f32) -> String {
    let v = if v.is_finite() { v.max(0.0) } else { 0.0 };
    if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 10_000.0 {
        format!("{:.1}k", v / 1000.0)
    } else {
        format!("{}", v.ceil() as u32)
    }
}

/// The yards from feet to crown a unit's body is taken to span: the draw scale is tuned against
/// the Stand height (about 2-3.5 yd for the horde, 4-5 for great beasts, 5-9 for a boss).
pub(super) fn body_height(def: &EnemyDef, boss: bool) -> f32 {
    if boss {
        (3.6 * def.scale).clamp(3.0, 10.0)
    } else {
        ((if def.big { 4.2 } else { 2.6 }) * def.scale).clamp(1.2, 8.0)
    }
}

/// The world camera's view as a world-to-window mapping.
#[derive(Clone, Copy, Debug)]
pub(super) struct ScreenView {
    pub clip_from_world: Mat4,
    pub size: Vec2,
}

impl ScreenView {
    pub fn new(projection: &Projection, eye: &Transform, size: Vec2) -> Self {
        Self {
            clip_from_world: projection.get_clip_from_view() * eye.to_matrix().inverse(),
            size,
        }
    }

    /// Window logical px and the depth, `None` behind the eye.
    pub fn project(&self, p: Vec3) -> Option<(Vec2, f32)> {
        let clip = self.clip_from_world * p.extend(1.0);
        if clip.w <= 1e-4 || !clip.is_finite() {
            return None;
        }
        let ndc = clip.truncate() / clip.w;
        Some((
            Vec2::new(
                (ndc.x + 1.0) * 0.5 * self.size.x,
                (1.0 - ndc.y) * 0.5 * self.size.y,
            ),
            clip.w,
        ))
    }
}

/// An enemy's body for [`collect`].
pub(super) struct Body<'a> {
    pub entity: Entity,
    pub pos: Vec3,
    pub enemy: &'a Enemy,
}

/// The live enemies whose body or plate falls in the window, in no particular order.
pub(super) fn collect<'a>(
    view: &ScreenView,
    scale: f32,
    hero: Option<Vec3>,
    bodies: impl Iterator<Item = Body<'a>>,
    out: &mut Vec<OnScreen>,
) {
    out.clear();
    let screen = Rect::from_corners(Vec2::ZERO, view.size);
    for b in bodies {
        let e = b.enemy;
        if e.dead || e.hp.is_nan() || e.hp <= 0.0 {
            continue;
        }
        let height = body_height(e.def, e.boss);
        let (Some((feet, depth)), Some((head, _))) =
            (view.project(b.pos), view.project(b.pos + Vec3::Y * height))
        else {
            continue;
        };
        let per_yard = feet.distance(head) / height;
        let half = (e.radius * per_yard).max(10.0);
        let cx = (feet.x + head.x) * 0.5;
        let body = Rect::new(cx - half, head.y.min(feet.y), cx + half, head.y.max(feet.y));
        let (w, h, above) = if e.boss {
            (BOSS_W, BOSS_H, BOSS_NAME_H)
        } else {
            (PLATE_W, PLATE_H, 0.0)
        };
        let bottom = head.y.min(feet.y) - PLATE_GAP * scale;
        let plate = Rect::new(
            head.x - w * scale * 0.5,
            bottom - (h + above) * scale,
            head.x + w * scale * 0.5,
            bottom,
        );
        if screen.intersect(body).is_empty() && screen.intersect(plate).is_empty() {
            continue;
        }
        out.push(OnScreen {
            entity: b.entity,
            def: e.def,
            boss: e.boss,
            body,
            plate,
            depth,
            hero_dist: hero.map_or(0.0, |h| super::units::flat(b.pos - h).length()),
            hp: e.hp,
            max_hp: e.max_hp,
        });
    }
}

/// The enemy under the cursor: a plate first (it draws on top), then the front-most body.
pub(super) fn pick(cursor: Vec2, entries: &[OnScreen]) -> Option<Entity> {
    entries
        .iter()
        .filter_map(|e| {
            let on_plate = e.plate.contains(cursor);
            (on_plate || e.body.contains(cursor)).then_some(((!on_plate) as u8, e.depth, e.entity))
        })
        .min_by(|a, b| {
            a.0.cmp(&b.0)
                .then(a.1.total_cmp(&b.1))
                .then(stable_key(a.2).cmp(&stable_key(b.2)))
        })
        .map(|(_, _, e)| e)
}

/// The next enemy for Tab: the on-screen ones in reach, nearest first, ties broken by entity so
/// the order holds still between presses.
/// Spawn order for tie-breaks. `Entity::to_bits` (and `Ord`) store the index bit-inverted
/// (`NonMaxU32`), so they sort newer entities first; this sorts by the real index.
fn stable_key(e: Entity) -> (u32, u32) {
    (e.index_u32(), e.generation().to_bits())
}

pub(super) fn cycle(
    entries: &[OnScreen],
    current: Option<Entity>,
    reverse: bool,
) -> Option<Entity> {
    let mut near: Vec<&OnScreen> = entries
        .iter()
        .filter(|e| e.hero_dist <= TAB_RANGE)
        .collect();
    if near.is_empty() {
        return None;
    }
    near.sort_by(|a, b| {
        a.hero_dist
            .total_cmp(&b.hero_dist)
            .then(stable_key(a.entity).cmp(&stable_key(b.entity)))
    });
    let n = near.len();
    let at = current.and_then(|c| near.iter().position(|e| e.entity == c));
    let next = match (at, reverse) {
        (Some(i), false) => (i + 1) % n,
        (Some(i), true) => (i + n - 1) % n,
        (None, false) => 0,
        (None, true) => n - 1,
    };
    Some(near[next].entity)
}

/// Whether a release ends a click that selects: not a camera drag, not on any UI, not a bounce.
pub(super) fn click_selects(travel: f32, ui_blocked: bool, since_last: f64) -> bool {
    !ui_blocked && travel <= DRAG_THRESHOLD && since_last >= DEBOUNCE
}

/// Any hovered or pressed UI node (buttons, cards, menus, tooltips' icons, the target frame)
/// swallows the click. The plates carry no `Interaction`, so they never block their own pick.
pub(super) fn ui_blocked<'a>(mut interactions: impl Iterator<Item = &'a Interaction>) -> bool {
    interactions.any(|i| *i != Interaction::None)
}

fn shown(phase: Option<&State<Phase>>) -> bool {
    matches!(
        phase.map(State::get),
        Some(Phase::Playing | Phase::LevelUp | Phase::Paused)
    )
}

/// A plain creature's name in the UI's language. Creature names live in the server's templates,
/// not in any client DBC, so the Chinese comes from this table and falls back to English.
pub(super) fn creature_name(english: &'static str) -> &'static str {
    if is_zh_cn() {
        zh_creature(english).unwrap_or(english)
    } else {
        english
    }
}

fn zh_creature(english: &str) -> Option<&'static str> {
    Some(match english {
        "Fleeting Plainstrider" => "迅捷平原陆行鸟",
        "Zhevra Runner" => "斑马奔行者",
        "Razormane Quilboar" => "钢鬃野猪人",
        "Razormane Hunter" => "钢鬃猎手",
        "Savannah Huntress" => "草原雌狮",
        "Sunscale Lashtail" => "阳鳞鞭尾迅猛龙",
        "Kolkar Wrangler" => "科卡尔驯马者",
        "Kolkar Stormer" => "科卡尔风暴使者",
        "Witchwing Harpy" => "巫翼鹰身人",
        "Hecklefang Hyena" => "嘲颚土狼",
        "Razormane Defender" => "钢鬃防御者",
        "Razormane Mystic" => "钢鬃秘法师",
        "Sunscale Screecher" => "阳鳞尖啸者",
        "Kolkar Bloodcharger" => "科卡尔血骑士",
        "Savannah Prowler" => "草原潜行者",
        "Thunderhawk Hatchling" => "幼雷鹰",
        "Witchwing Slayer" => "巫翼杀戮者",
        "Bristleback Geomancer" => "刺背地卜师",
        "Stormsnout" => "雷鼻",
        "Barrens Kodo" => "贫瘠之地科多兽",
        "Sunscale Scytheclaw" => "阳鳞镰爪迅猛龙",
        "Thunderhead" => "雷首",
        "Silithid Swarmer" => "异种虫群",
        "Greater Thunderhawk" => "大雷鹰",
        "Hecklefang Stalker" => "嘲颚潜伏者",
        "Kolkar Destroyer" => "科卡尔毁灭者",
        "Kolkar Mauler" => "科卡尔重殴者",
        "Greater Barrens Kodo" => "巨型贫瘠之地科多兽",
        "Elder Thunder Lizard" => "年长的雷霆蜥蜴",
        "Silithid Protector" => "异种虫保卫者",
        _ => return None,
    })
}

pub(super) fn enemy_name(enemy: &Enemy) -> &'static str {
    if enemy.boss {
        let name = bosses::name(enemy.def);
        if name == enemy.def.name {
            creature_name(name)
        } else {
            name
        }
    } else {
        creature_name(enemy.def.name)
    }
}

/// One debuff row entry: the spell whose icon it wears, its key, seconds left and stacks; the
/// class effects also carry their localized name and strength (a fraction, 0.15 is 15%).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct DebuffView {
    pub spell: u32,
    pub key: &'static str,
    pub left: f32,
    pub stacks: u32,
    pub name: Option<&'static str>,
    pub amount: Option<f32>,
}

impl DebuffView {
    fn timed(spell: u32, key: &'static str, left: f32) -> Self {
        Self {
            spell,
            key,
            left,
            stacks: 1,
            name: None,
            amount: None,
        }
    }
}

/// `Shadow Vulnerability`, the icon the warlock's vulnerability wears.
const VULNERABILITY_SPELL: u32 = 17800;

/// The crowd control each enemy carries as plain timers, with the spell icon it reads as.
const CONTROL: [(&str, u32); 5] = [
    ("stun", 853),
    ("fear", 5782),
    ("root", 122),
    ("slow", 116),
    ("weaken", 1160),
];

fn control_name(key: &str) -> &'static str {
    match key {
        "stun" => text("Stunned", "昏迷"),
        "fear" => text("Feared", "恐惧"),
        "root" => text("Rooted", "定身"),
        "slow" => text("Slowed", "减速"),
        _ => text("Weakened", "虚弱"),
    }
}

/// The enemy's crowd control, its damage over time merged by ability, then the run's class
/// effects on it (with no run, those are left out), at most [`DEBUFF_SLOTS`].
pub(super) fn debuffs(enemy: &Enemy, run: Option<&Run>) -> Vec<DebuffView> {
    let mut out = Vec::new();
    for (key, spell) in CONTROL {
        let left = match key {
            "stun" => enemy.stun_t,
            "fear" => enemy.fear_t,
            "root" => enemy.root_t,
            "slow" if enemy.slow < 1.0 => enemy.slow_t,
            "weaken" if enemy.weaken < 1.0 => enemy.weaken_t,
            _ => 0.0,
        };
        if left > 0.0 {
            out.push(DebuffView::timed(spell, key, left));
        }
    }
    for dot in &enemy.dots {
        if dot.left <= 0.0 {
            continue;
        }
        let Some(def) = dot
            .source
            .ability
            .and_then(|k| data::ABILITIES.iter().find(|a| a.key == k))
        else {
            continue;
        };
        let left = dot.left as f32;
        if let Some(have) = out.iter_mut().find(|d| d.key == def.key) {
            have.left = have.left.max(left);
            have.stacks += 1;
        } else {
            out.push(DebuffView::timed(def.spell, def.key, left));
        }
    }
    if let Some(run) = run {
        if let Some((amount, left)) = super::warlock::vulnerability(enemy, run.elapsed) {
            if left > 0.0 {
                out.push(DebuffView {
                    name: Some(text("Shadow Vulnerability", "暗影易伤")),
                    amount: Some(amount),
                    ..DebuffView::timed(VULNERABILITY_SPELL, "shadow_vulnerability", left)
                });
            }
        }
        for (name, spell, amount, left) in super::class_mechanics::target_buffs(run, enemy) {
            if left > 0.0 {
                out.push(DebuffView {
                    name: Some(name),
                    amount: Some(amount),
                    ..DebuffView::timed(spell, name, left)
                });
            }
        }
    }
    out.truncate(DEBUFF_SLOTS);
    out
}

fn debuff_name(d: &DebuffView, spells: Option<&crate::ui_action::Spells>) -> String {
    if let Some(name) = d.name {
        name.to_string()
    } else if CONTROL.iter().any(|(k, _)| *k == d.key) {
        control_name(d.key).to_string()
    } else {
        super::hud::spell_name(spells, d.spell, d.key)
    }
}

fn seconds(left: f32) -> String {
    if left >= 60.0 {
        format!("{}m", (left / 60.0).ceil() as u32)
    } else {
        format!("{}", left.ceil().max(0.0) as u32)
    }
}

// ── Node writes that only touch what changed ────────────────────────────────────────────────────

fn place(mut node: Mut<Node>, left: f32, top: f32, w: f32, h: f32) {
    let want = (
        Val::Px(left.round()),
        Val::Px(top.round()),
        Val::Px(w.round()),
        Val::Px(h.round()),
    );
    if (node.left, node.top, node.width, node.height) != want {
        let n = &mut *node;
        (n.left, n.top, n.width, n.height) = want;
    }
}

fn set_width_pct(mut node: Mut<Node>, frac: f32) {
    // Whole tenths of a percent: finer steps re-layout without a visible difference.
    let want = Val::Percent((frac * 1000.0).round() / 10.0);
    if node.width != want {
        node.width = want;
    }
}

fn show(mut vis: Mut<Visibility>, on: bool) {
    vis.set_if_neq(if on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

fn set_text(mut t: Mut<Text>, s: &str) {
    if t.0 != s {
        t.0 = s.to_string();
    }
}

fn label(font: &Handle<Font>, s: &str, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont {
            font: font.clone(),
            font_size: size,
            ..default()
        },
        TextColor(color),
        TextShadow {
            offset: Vec2::splat(1.0),
            color: Color::BLACK,
        },
    )
}

fn abs(left: f32, top: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(w),
        height: Val::Px(h),
        ..default()
    }
}

fn font_of(assets: Option<&AssetServer>) -> Handle<Font> {
    assets.map(crate::char_select::wow_font).unwrap_or_default()
}

// ── Setup ───────────────────────────────────────────────────────────────────────────────────────

pub(super) fn setup_plates(mut commands: Commands, assets: Option<Res<AssetServer>>) {
    let font = font_of(assets.as_deref());
    let root = commands
        .spawn((
            TargetUiNode,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(390),
            FocusPolicy::Pass,
            Visibility::Hidden,
        ))
        .id();
    let ring_pct = commands
        .spawn((
            TargetUiNode,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(100.0),
                top: Val::Px(-5.0),
                margin: UiRect::left(Val::Px(4.0)),
                ..default()
            },
            label(&font, "", 10.0, GOLD),
        ))
        .id();
    let ring = commands
        .spawn((
            TargetUiNode,
            Node {
                border: UiRect::all(Val::Px(2.0)),
                ..abs(0.0, 0.0, PLATE_W, PLATE_H)
            },
            BorderColor::all(GOLD),
            GlobalZIndex(395),
            FocusPolicy::Pass,
            Visibility::Hidden,
        ))
        .add_child(ring_pct)
        .id();
    commands.entity(root).add_child(ring);
    commands.insert_resource(PlatePool {
        root,
        normal: Vec::new(),
        boss: Vec::new(),
        normal_used: 0,
        boss_used: 0,
        ring,
        ring_pct,
        font,
    });
}

fn spawn_slot(commands: &mut Commands, pool: &PlatePool, boss: bool) -> PlateSlot {
    let (w, h) = if boss {
        (BOSS_W, BOSS_H)
    } else {
        (PLATE_W, PLATE_H)
    };
    let fill = commands
        .spawn((
            TargetUiNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(PLATE_RED),
        ))
        .id();
    let frame = commands
        .spawn((
            TargetUiNode,
            PlateFrame,
            Node {
                border: UiRect::all(Val::Px(1.0)),
                ..abs(0.0, 0.0, w, h)
            },
            BackgroundColor(PLATE_BACK),
            BorderColor::all(Color::BLACK),
            FocusPolicy::Pass,
            Visibility::Hidden,
        ))
        .add_child(fill)
        .id();
    let (mut name, mut hp) = (None, None);
    if boss {
        let hp_e = commands
            .spawn((
                TargetUiNode,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    top: Val::Px(-1.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                label(&pool.font, "", 9.0, Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ))
            .id();
        let name_e = commands
            .spawn((
                TargetUiNode,
                abs(-60.0, -BOSS_NAME_H - 1.0, BOSS_W + 120.0, BOSS_NAME_H),
                label(&pool.font, "", 12.0, GOLD),
                TextLayout::new_with_justify(Justify::Center),
            ))
            .id();
        commands.entity(frame).add_children(&[hp_e, name_e]);
        name = Some(name_e);
        hp = Some(hp_e);
    }
    commands.entity(pool.root).add_child(frame);
    PlateSlot {
        frame,
        fill,
        name,
        hp,
    }
}

pub(super) fn setup_panel(
    mut commands: Commands,
    assets: Option<Res<AssetServer>>,
    images: Option<ResMut<Assets<Image>>>,
) {
    let font = font_of(assets.as_deref());
    let f = &font;
    let mut image = Handle::<Image>::default();
    if let Some(mut images) = images {
        image = images.add(crate::portrait::new_target_image_sized(
            PORTRAIT_PX,
            PORTRAIT_PX,
        ));
        let camera = commands
            .spawn((
                crate::portrait::booth_view_shape(),
                Camera {
                    order: -59,
                    is_active: false,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::Image(image.clone().into()),
                benilla_world::ffx_glow::FfxGlow::BOOTH,
                Projection::Perspective(PerspectiveProjection {
                    fov: PORTRAIT_FOV,
                    near: 0.05,
                    far: 250.0,
                    ..default()
                }),
                RenderLayers::layer(PORTRAIT_LAYER),
                Transform::default(),
            ))
            .id();
        commands.insert_resource(PortraitRig {
            camera,
            clone: None,
            for_target: None,
        });
    }

    let portrait = commands
        .spawn((
            TargetUiNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ImageNode::new(image),
        ))
        .id();

    let portrait_frame = commands
        .spawn((
            TargetUiNode,
            Node {
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(34.0)),
                overflow: Overflow::clip(),
                ..abs(6.0, 6.0, 68.0, 68.0)
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            BorderColor::all(EDGE),
        ))
        .add_child(portrait)
        .id();
    let name = commands
        .spawn((
            TargetUiNode,
            abs(82.0, 3.0, 150.0, 17.0),
            label(f, "", 13.0, GOLD),
        ))
        .id();
    let level = commands
        .spawn((
            TargetUiNode,
            abs(214.0, 5.0, 80.0, 14.0),
            label(f, "", 10.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Right),
        ))
        .id();
    let hp_fill = commands
        .spawn((
            TargetUiNode,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(HP_GREEN),
        ))
        .id();
    let hp_text = commands
        .spawn((
            TargetUiNode,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(0.0),
                ..default()
            },
            label(f, "", 10.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
        ))
        .id();
    let hp_bar = commands
        .spawn((
            TargetUiNode,
            Node {
                border: UiRect::all(Val::Px(1.0)),
                ..abs(82.0, 21.0, 210.0, 15.0)
            },
            BackgroundColor(Color::srgba(0.15, 0.0, 0.0, 0.9)),
            BorderColor::all(Color::BLACK),
        ))
        .add_children(&[hp_fill, hp_text])
        .id();
    let cast_fill = commands
        .spawn((
            TargetUiNode,
            Node {
                width: Val::Percent(0.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(GOLD),
        ))
        .id();
    let cast_text = commands
        .spawn((
            TargetUiNode,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(-2.0),
                ..default()
            },
            label(f, "", 9.0, Color::WHITE),
            TextLayout::new_with_justify(Justify::Center),
        ))
        .id();
    let cast = commands
        .spawn((
            TargetUiNode,
            Node {
                border: UiRect::all(Val::Px(1.0)),
                ..abs(82.0, 39.0, 210.0, 11.0)
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            BorderColor::all(Color::BLACK),
            Visibility::Hidden,
        ))
        .add_children(&[cast_fill, cast_text])
        .id();
    let mut debuffs = Vec::with_capacity(DEBUFF_SLOTS);
    let mut debuff_nodes = Vec::with_capacity(DEBUFF_SLOTS);
    for i in 0..DEBUFF_SLOTS {
        let icon = commands
            .spawn((
                TargetUiNode,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ImageNode {
                    color: Color::srgba(0.0, 0.0, 0.0, 0.6),
                    ..default()
                },
            ))
            .id();
        let timer = commands
            .spawn((
                TargetUiNode,
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    bottom: Val::Px(-2.0),
                    ..default()
                },
                label(f, "", 9.0, Color::WHITE),
                TextLayout::new_with_justify(Justify::Center),
            ))
            .id();
        let slot = commands
            .spawn((
                TargetUiNode,
                DebuffSlot(i),
                Interaction::default(),
                Node {
                    border: UiRect::all(Val::Px(1.0)),
                    ..abs(82.0 + i as f32 * 21.0, 55.0, 19.0, 19.0)
                },
                BorderColor::all(Color::srgb(0.8, 0.0, 0.0)),
                Visibility::Hidden,
            ))
            .add_children(&[icon, timer])
            .id();
        debuffs.push((slot, icon, timer));
        debuff_nodes.push(slot);
    }
    let tooltip_text = commands
        .spawn((TargetUiNode, label(f, "", 11.0, Color::WHITE)))
        .id();
    let tooltip = commands
        .spawn((
            TargetUiNode,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(PANEL_H + 4.0),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(TIP_FILL),
            BorderColor::all(EDGE),
            FocusPolicy::Pass,
            Visibility::Hidden,
        ))
        .add_child(tooltip_text)
        .id();
    let root = commands
        .spawn((
            TargetUiNode,
            TargetPanel,
            Interaction::default(),
            Node {
                border: UiRect::all(Val::Px(1.0)),
                ..abs(PANEL_LEFT, PANEL_TOP, PANEL_W, PANEL_H)
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
            BorderColor::all(EDGE),
            GlobalZIndex(405),
            Visibility::Hidden,
        ))
        .add_children(&[portrait_frame, name, level, hp_bar, cast, tooltip])
        .add_children(&debuff_nodes)
        .id();
    commands.insert_resource(PanelUi {
        root,
        name,
        level,
        hp_fill,
        hp_text,
        cast,
        cast_fill,
        cast_text,
        debuffs,
        tooltip,
        tooltip_text,
    });
}

// ── Per frame ───────────────────────────────────────────────────────────────────────────────────

/// Where every live enemy falls on the window this frame.
#[allow(clippy::type_complexity)]
pub(super) fn project_enemies(
    windows: Query<&Window, With<PrimaryWindow>>,
    cams: Query<(&Projection, &Transform), With<WorldCamera>>,
    ui_scale: Option<Res<UiScale>>,
    hero: Query<&Transform, (With<Hero>, Without<WorldCamera>)>,
    enemies: Query<(Entity, &Transform, &Enemy), (Without<WorldCamera>, Without<Hero>)>,
    mut cache: ResMut<ScreenCache>,
) {
    let scale = ui_scale.map_or(1.0, |s| s.0).max(1e-3);
    cache.scale = scale;
    let (Ok(window), Ok((projection, eye))) = (windows.single(), cams.single()) else {
        cache.entries.clear();
        return;
    };
    let size = Vec2::new(window.width(), window.height());
    if size.x <= 0.0 || size.y <= 0.0 {
        cache.entries.clear();
        return;
    }
    let view = ScreenView::new(projection, eye, size);
    let hero = hero.single().ok().map(|t| t.translation);
    let bodies = enemies.iter().map(|(entity, tf, enemy)| Body {
        entity,
        pos: tf.translation,
        enemy,
    });
    collect(&view, scale, hero, bodies, &mut cache.entries);
}

/// Left click selects (a drag stays the camera's), Tab cycles, a click on nothing clears.
#[allow(clippy::too_many_arguments)]
pub(super) fn select_input(
    phase: Option<Res<State<Phase>>>,
    time: Option<Res<Time<Real>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    interactions: Query<&Interaction>,
    screen: Res<ScreenCache>,
    mut tracker: ResMut<ClickTracker>,
    mut selected: ResMut<SelectedTarget>,
) {
    if phase.as_deref().map(State::get) != Some(&Phase::Playing) {
        tracker.press = None;
        return;
    }
    let cursor = windows.single().ok().and_then(Window::cursor_position);
    if let Some(mouse) = mouse.as_deref() {
        if mouse.just_pressed(MouseButton::Left) {
            tracker.press = cursor;
            tracker.travel = 0.0;
            tracker.blocked = ui_blocked(interactions.iter());
        }
        if let (Some(press), Some(at)) = (tracker.press, cursor) {
            tracker.travel = tracker.travel.max(press.distance(at));
        }
        if mouse.just_released(MouseButton::Left) {
            if let (Some(_), Some(at)) = (tracker.press.take(), cursor) {
                tracker.blocked |= ui_blocked(interactions.iter());
                let now = time.as_deref().map_or(0.0, Time::elapsed_secs_f64);
                let since = tracker.last.map_or(f64::INFINITY, |l| now - l);
                if click_selects(tracker.travel, tracker.blocked, since) {
                    tracker.last = Some(now);
                    selected.set(pick(at, &screen.entries));
                }
            }
        }
    }
    if let Some(keys) = keys.as_deref() {
        if keys.just_pressed(KeyCode::Tab) {
            let reverse = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
            if let Some(next) = cycle(&screen.entries, selected.get(), reverse) {
                selected.set(Some(next));
            }
        }
    }
}

/// A dead or despawned target, or a new battleground, lets go of the selection.
pub(super) fn validate_target(
    mut selected: ResMut<SelectedTarget>,
    enemies: Query<&Enemy>,
    selection: Option<Res<Selection>>,
    mut stage: Local<Option<usize>>,
) {
    if let Some(selection) = selection {
        if stage.is_some_and(|s| s != selection.stage) {
            selected.clear();
        }
        *stage = Some(selection.stage);
    }
    if let Some(e) = selected.get() {
        if enemies.get(e).map_or(true, |e| e.dead) {
            selected.clear();
        }
    }
}

pub(super) fn clear_target(
    mut selected: ResMut<SelectedTarget>,
    mut tracker: ResMut<ClickTracker>,
) {
    selected.clear();
    tracker.press = None;
}

/// Seat a pooled plate over every live on-screen enemy; slots past the frame's count hide.
#[allow(clippy::too_many_arguments)]
pub(super) fn sync_plates(
    mut commands: Commands,
    phase: Option<Res<State<Phase>>>,
    screen: Res<ScreenCache>,
    selected: Res<SelectedTarget>,
    pool: Option<ResMut<PlatePool>>,
    mut nodes: Query<(&mut Node, &mut Visibility), With<TargetUiNode>>,
    mut texts: Query<&mut Text, With<TargetUiNode>>,
) {
    let Some(mut pool) = pool else { return };
    let on = shown(phase.as_deref());
    if let Ok((_, vis)) = nodes.get_mut(pool.root) {
        show(vis, on);
    }
    if !on {
        return;
    }
    let s = screen.scale.max(1e-3);
    let mut normal = 0;
    let mut boss = 0;
    let mut ring_at: Option<(Rect, f32)> = None;
    for e in &screen.entries {
        let slot = if e.boss {
            if boss == pool.boss.len() {
                let slot = spawn_slot(&mut commands, &pool, true);
                pool.boss.push(slot);
            }
            boss += 1;
            pool.boss[boss - 1]
        } else {
            if normal == pool.normal.len() {
                // Grow in batches so a wave arriving costs one spawn burst, not one per frame.
                // Every visible enemy gets a plate: the pool is bounded by the enemies alive.
                for _ in 0..POOL_GROW {
                    let slot = spawn_slot(&mut commands, &pool, false);
                    pool.normal.push(slot);
                }
            }
            normal += 1;
            pool.normal[normal - 1]
        };
        let frac = hp_frac(e.hp, e.max_hp);
        let (w, h) = if e.boss {
            (BOSS_W, BOSS_H)
        } else {
            (PLATE_W, PLATE_H)
        };
        let left = e.plate.min.x / s;
        let top = e.plate.max.y / s - h;
        if Some(e.entity) == selected.entity {
            ring_at = Some((Rect::new(left, top, left + w, top + h), frac));
        }
        if let Ok((node, vis)) = nodes.get_mut(slot.frame) {
            place(node, left, top, w, h);
            show(vis, true);
        }
        if let Ok((node, _)) = nodes.get_mut(slot.fill) {
            set_width_pct(node, frac);
        }
        if let Some(t) = slot.name.and_then(|n| texts.get_mut(n).ok()) {
            let name = if e.boss {
                let n = bosses::name(e.def);
                if n == e.def.name {
                    creature_name(n)
                } else {
                    n
                }
            } else {
                creature_name(e.def.name)
            };
            set_text(t, name);
        }
        if let Some(t) = slot.hp.and_then(|n| texts.get_mut(n).ok()) {
            set_text(t, &format!("{} / {}", compact(e.hp), compact(e.max_hp)));
        }
    }
    for i in normal..pool.normal_used.min(pool.normal.len()) {
        if let Ok((_, vis)) = nodes.get_mut(pool.normal[i].frame) {
            show(vis, false);
        }
    }
    for i in boss..pool.boss_used.min(pool.boss.len()) {
        if let Ok((_, vis)) = nodes.get_mut(pool.boss[i].frame) {
            show(vis, false);
        }
    }
    pool.normal_used = normal;
    pool.boss_used = boss;
    if let Ok((node, vis)) = nodes.get_mut(pool.ring) {
        show(vis, ring_at.is_some());
        if let Some((r, _)) = ring_at {
            place(
                node,
                r.min.x - 2.0,
                r.min.y - 2.0,
                r.width() + 4.0,
                r.height() + 4.0,
            );
        }
    }
    if let (Some((_, frac)), Ok(t)) = (ring_at, texts.get_mut(pool.ring_pct)) {
        set_text(t, &format!("{}%", (frac * 100.0).round() as u32));
    }
}

/// The target frame: name, level and rank, health, a boss's cast, the debuffs and their tooltip.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn sync_panel(
    phase: Option<Res<State<Phase>>>,
    selected: Res<SelectedTarget>,
    panel: Option<Res<PanelUi>>,
    enemies: Query<(&Enemy, Option<&Boss>)>,
    spells: Option<Res<crate::ui_action::Spells>>,
    mut assets: Option<ResMut<benilla_assets::WorldAssets>>,
    mut images: Option<ResMut<Assets<Image>>>,
    mut icons: Local<HashMap<u32, Handle<Image>>>,
    mut nodes: Query<(&mut Node, &mut Visibility), With<TargetUiNode>>,
    mut texts: Query<&mut Text, With<TargetUiNode>>,
    mut colors: Query<&mut TextColor, With<TargetUiNode>>,
    mut image_nodes: Query<&mut ImageNode, With<TargetUiNode>>,
    hovers: Query<(&Interaction, &DebuffSlot)>,
    run: Option<Res<Run>>,
) {
    let Some(panel) = panel else { return };
    let target = selected
        .entity
        .and_then(|e| enemies.get(e).ok())
        .filter(|(e, _)| !e.dead);
    let on = shown(phase.as_deref()) && target.is_some();
    if let Ok((_, vis)) = nodes.get_mut(panel.root) {
        show(vis, on);
    }
    let Some((enemy, boss)) = target.filter(|_| on) else {
        return;
    };
    if let Ok(t) = texts.get_mut(panel.name) {
        set_text(t, enemy_name(enemy));
    }
    // `EnemyDef::big` is a body size, not an elite rank: only a boss carries one.
    let rank = if enemy.boss {
        text("Boss", "首领")
    } else {
        ""
    };
    if let Ok(t) = texts.get_mut(panel.level) {
        let lv = text("Level", "等级");
        set_text(t, format!("{lv} {} {rank}", enemy.def.level).trim_end());
    }
    if let Ok(mut c) = colors.get_mut(panel.level) {
        let want = if enemy.boss { GOLD } else { Color::WHITE };
        if c.0 != want {
            c.0 = want;
        }
    }
    let frac = hp_frac(enemy.hp, enemy.max_hp);
    if let Ok((node, _)) = nodes.get_mut(panel.hp_fill) {
        set_width_pct(node, frac);
    }
    if let Ok(t) = texts.get_mut(panel.hp_text) {
        set_text(
            t,
            &format!(
                "{} / {} ({}%)",
                compact(enemy.hp),
                compact(enemy.max_hp),
                (frac * 100.0).round() as u32
            ),
        );
    }

    let cast = boss.and_then(bosses::cast_status);
    if let Ok((_, vis)) = nodes.get_mut(panel.cast) {
        show(vis, cast.is_some());
    }
    if let Some((name, remaining, total)) = cast {
        let progress = if total > 0.0 && total.is_finite() {
            (1.0 - remaining / total).clamp(0.0, 1.0)
        } else {
            1.0
        };
        if let Ok((node, _)) = nodes.get_mut(panel.cast_fill) {
            set_width_pct(node, progress);
        }
        if let Ok(t) = texts.get_mut(panel.cast_text) {
            set_text(t, &format!("{name} {:.1}", remaining.max(0.0)));
        }
    }

    let list = debuffs(enemy, run.as_deref());
    for (i, &(slot, icon, timer)) in panel.debuffs.iter().enumerate() {
        let d = list.get(i);
        if let Ok((_, vis)) = nodes.get_mut(slot) {
            show(vis, d.is_some());
        }
        let Some(d) = d else { continue };
        if let Ok(t) = texts.get_mut(timer) {
            set_text(t, &seconds(d.left));
        }
        let cached = icons.get(&d.spell).cloned();
        let handle = cached.or_else(|| {
            let spells = spells.as_deref()?;
            let path = spells
                .catalog
                .get(d.spell)
                .and_then(|s| s.icon.clone())
                .unwrap_or_else(|| "Interface\\Icons\\INV_Misc_QuestionMark".into());
            let h = assets
                .as_deref_mut()?
                .sprite_texture(&format!("{path}.blp"), images.as_deref_mut()?)?;
            icons.insert(d.spell, h.clone());
            Some(h)
        });
        if let (Some(h), Ok(mut img)) = (handle, image_nodes.get_mut(icon)) {
            if img.image != h {
                img.image = h;
                img.color = Color::WHITE;
            }
        }
    }

    let hovered = hovers
        .iter()
        .find(|(i, _)| **i != Interaction::None)
        .and_then(|(_, s)| list.get(s.0));
    if let Ok((_, vis)) = nodes.get_mut(panel.tooltip) {
        show(vis, hovered.is_some());
    }
    if let (Some(d), Ok(t)) = (hovered, texts.get_mut(panel.tooltip_text)) {
        let stacks = if d.stacks > 1 {
            format!(" ×{}", d.stacks)
        } else {
            String::new()
        };
        let left = text("remaining", "剩余");
        let amount = d
            .amount
            .filter(|a| a.is_finite())
            .map(|a| format!("\n{:+.0}%", a * 100.0))
            .unwrap_or_default();
        set_text(
            t,
            &format!(
                "{}{stacks}{amount}\n{left} {:.1}s",
                debuff_name(d, spells.as_deref()),
                d.left
            ),
        );
    }
}

/// The tallest point of a unit's drawn parts over its feet, from their bounds; `None` before the
/// model has loaded. Bounded so a stray effect or a broken bound cannot throw the framing off.
pub(super) fn measured_height(
    root: Entity,
    feet_y: f32,
    children: &Query<&Children>,
    bounds: &Query<(&Aabb, &GlobalTransform)>,
) -> Option<f32> {
    let mut top: Option<f32> = None;
    let mut stack = vec![(root, 0u8)];
    let mut visited = 0;
    while let Some((e, depth)) = stack.pop() {
        visited += 1;
        if visited > 512 {
            break;
        }
        if let Ok((aabb, gt)) = bounds.get(e) {
            let (c, h) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
            for i in 0..8 {
                let corner = c + h * Vec3::new(
                    if i & 1 == 0 { -1.0 } else { 1.0 },
                    if i & 2 == 0 { -1.0 } else { 1.0 },
                    if i & 4 == 0 { -1.0 } else { 1.0 },
                );
                let y = gt.transform_point(corner).y;
                if y.is_finite() {
                    top = Some(top.map_or(y, |t: f32| t.max(y)));
                }
            }
        }
        if depth < 12 {
            if let Ok(kids) = children.get(e) {
                stack.extend(kids.iter().map(|k| (k, depth + 1)));
            }
        }
    }
    top.map(|t| t - feet_y)
        .filter(|h| *h > 0.3)
        .map(|h| h.min(40.0))
}

/// The portrait camera for a body `height` yards tall standing at `feet` and facing `forward`:
/// the head and shoulders, from the front, whatever the size, a kobold or a kodo.
pub(super) fn portrait_view(feet: Vec3, forward: Vec3, height: f32) -> Transform {
    let h = if height.is_finite() {
        height.clamp(0.5, 40.0)
    } else {
        2.5
    };
    let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or(Vec3::NEG_Z);
    // The top 70% of the body fills the frame with a tenth to spare.
    let span = h * 0.7;
    let dist = span * 0.5 / (PORTRAIT_FOV * 0.5).tan() * 1.1;
    let focus = feet + Vec3::Y * (h * 0.65);
    let eye = focus + forward * dist + Vec3::Y * (h * 0.08);
    Transform::from_translation(eye).looking_at(focus, Vec3::Y)
}

/// The portrait: a clone of the selected unit on its own layer, standing where the unit stands
/// and facing as it faces, filmed from the front into the frame's image. A new target swaps the
/// clone; no target, or the frame hidden, despawns it and parks the camera.
#[allow(clippy::type_complexity)]
pub(super) fn sync_portrait(
    mut commands: Commands,
    phase: Option<Res<State<Phase>>>,
    selected: Res<SelectedTarget>,
    rig: Option<ResMut<PortraitRig>>,
    run: Option<ResMut<Run>>,
    enemies: Query<(&Transform, &Enemy), Without<PortraitClone>>,
    mut clones: Query<&mut Transform, (With<PortraitClone>, Without<Enemy>, Without<Camera>)>,
    mut cams: Query<(&mut Camera, &mut Transform), (Without<PortraitClone>, Without<Enemy>)>,
    children: Query<&Children>,
    bounds: Query<(&Aabb, &GlobalTransform)>,
    mut height: Local<Option<(Entity, f32)>>,
) {
    let Some(mut rig) = rig else { return };
    let target = selected
        .entity
        .filter(|_| shown(phase.as_deref()))
        .and_then(|e| enemies.get(e).ok().map(|t| (e, t)))
        .filter(|(_, (_, en))| !en.dead);
    // A clone reaped with the run's entities is forgotten, and made again if still wanted.
    if rig.clone.is_some_and(|c| clones.get(c).is_err()) {
        rig.clone = None;
        rig.for_target = None;
    }
    let want = target.map(|(e, _)| e);
    if want != rig.for_target {
        if let Some(c) = rig.clone.take() {
            commands.entity(c).try_despawn();
        }
        rig.for_target = None;
        if let (Some((e, (tf, enemy))), Some(mut run)) = (target, run) {
            let yaw = tf.rotation.to_euler(EulerRot::YXZ).0;
            let clone = units::spawn_unit(
                &mut commands,
                &UnitSpec {
                    serial: run.guid(),
                    display: enemy.def.display,
                    scale: enemy.def.scale,
                    level: enemy.def.level,
                    faction: units::FACTION_MONSTER,
                    bytes0: 1 << 8,
                    weapons: &[],
                    sheath: 0,
                },
                tf.translation,
                yaw,
            );
            let layer = RenderLayers::layer(PORTRAIT_LAYER);
            commands
                .entity(clone)
                .insert((PortraitClone, Propagate(layer.clone()), layer));
            rig.clone = Some(clone);
            rig.for_target = Some(e);
        }
    }
    let active = rig.clone.is_some() && target.is_some();
    if let Ok((mut cam, mut at)) = cams.get_mut(rig.camera) {
        if cam.is_active != active {
            cam.is_active = active;
        }
        if let (true, Some((e, (tf, enemy)))) = (active, target) {
            // Measured once the live body has bounds; the scale estimate until then.
            if height.is_none_or(|(of, _)| of != e) {
                *height = measured_height(e, tf.translation.y, &children, &bounds).map(|h| (e, h));
            }
            let h = height
                .filter(|(of, _)| *of == e)
                .map_or_else(|| body_height(enemy.def, enemy.boss), |(_, h)| h);
            let want = portrait_view(tf.translation, tf.forward().as_vec3(), h);
            if *at != want {
                *at = want;
            }
        }
    }
    if let (Some(c), Some((_, (tf, _)))) = (rig.clone, target) {
        if let Ok(mut ct) = clones.get_mut(c) {
            if ct.translation != tf.translation || ct.rotation != tf.rotation {
                ct.translation = tf.translation;
                ct.rotation = tf.rotation;
            }
        }
    }
}

/// A preview must not wait for the world's loading-cover gate or its eight-second timeout.
/// Complete the normal fade through its own writer, so its steady material and mesh alpha are
/// restored correctly even when virtual time is paused immediately after selecting.
fn finish_portrait_appear(
    mut commands: Commands,
    rig: Option<Res<PortraitRig>>,
    children: Query<&Children>,
    pending: Query<(), With<benilla_world::model_fade::PendingAppearFade>>,
) {
    use benilla_world::model_fade::{FadeCurve, PendingAppearFade, RenderFade, UnitAppearFade};
    let Some(root) = rig.and_then(|r| r.clone) else {
        return;
    };
    commands.entity(root).try_remove::<UnitAppearFade>();
    let mut stack = vec![root];
    while let Some(entity) = stack.pop() {
        if pending.contains(entity) {
            commands
                .entity(entity)
                .try_remove::<PendingAppearFade>()
                .try_insert(RenderFade {
                    started: 0.0,
                    duration: 0.0,
                    from: 1.0,
                    to: 1.0,
                    curve: FadeCurve::Cubic,
                });
        }
        if let Ok(kids) = children.get(entity) {
            stack.extend(kids.iter());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::window::WindowResolution;

    fn enemy(def: &'static EnemyDef, boss: bool) -> Enemy {
        Enemy {
            def,
            hp: 50.0,
            max_hp: 100.0,
            speed: 1.0,
            damage: 1.0,
            radius: 0.6,
            xp: 1,
            boss,
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

    const W: f32 = 1280.0;
    const H: f32 = 768.0;

    fn eye() -> Transform {
        Transform::from_xyz(0.0, 20.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y)
    }

    fn projection() -> Projection {
        Projection::Perspective(PerspectiveProjection {
            aspect_ratio: W / H,
            ..default()
        })
    }

    fn view() -> ScreenView {
        ScreenView::new(&projection(), &eye(), Vec2::new(W, H))
    }

    fn entry(n: u32, body: Rect, plate: Rect, depth: f32, dist: f32) -> OnScreen {
        OnScreen {
            entity: Entity::from_raw_u32(n).unwrap(),
            def: &data::ENEMIES[0],
            boss: false,
            body,
            plate,
            depth,
            hero_dist: dist,
            hp: 1.0,
            max_hp: 1.0,
        }
    }

    #[test]
    fn collection_keeps_only_live_enemies_in_the_window() {
        let def = &data::ENEMIES[0];
        let alive = enemy(def, false);
        let mut dead = enemy(def, false);
        dead.dead = true;
        let mut zero = enemy(def, false);
        zero.hp = 0.0;
        let behind = enemy(def, false);
        let aside = enemy(def, false);
        let ids: Vec<Entity> = (1..=5).map(|n| Entity::from_raw_u32(n).unwrap()).collect();
        let bodies = vec![
            Body {
                entity: ids[0],
                pos: Vec3::ZERO,
                enemy: &alive,
            },
            Body {
                entity: ids[1],
                pos: Vec3::ZERO,
                enemy: &dead,
            },
            Body {
                entity: ids[2],
                pos: Vec3::ZERO,
                enemy: &zero,
            },
            Body {
                entity: ids[3],
                pos: Vec3::new(0.0, 0.0, 200.0),
                enemy: &behind,
            },
            Body {
                entity: ids[4],
                pos: Vec3::new(900.0, 0.0, 0.0),
                enemy: &aside,
            },
        ];
        let mut out = Vec::new();
        collect(&view(), 1.0, Some(Vec3::ZERO), bodies.into_iter(), &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].entity, ids[0]);
        let centre = Vec2::new(W, H) / 2.0;
        assert!(out[0].body.contains(centre - Vec2::Y * 5.0));
        assert!(out[0].plate.max.y <= out[0].body.min.y);
        assert!((out[0].plate.width() - PLATE_W).abs() < 1e-3);
    }

    #[test]
    fn health_fraction_never_divides_by_zero() {
        assert_eq!(hp_frac(5.0, 0.0), 0.0);
        assert_eq!(hp_frac(5.0, f32::NAN), 0.0);
        assert_eq!(hp_frac(f32::NAN, 10.0), 0.0);
        assert_eq!(hp_frac(20.0, 10.0), 1.0);
        assert_eq!(hp_frac(5.0, 10.0), 0.5);
    }

    #[test]
    fn a_click_picks_the_plate_then_the_front_body_and_empty_clears() {
        let a = entry(
            1,
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Rect::new(0.0, -10.0, 64.0, -4.0),
            10.0,
            1.0,
        );
        let b = entry(
            2,
            Rect::new(50.0, 0.0, 150.0, 100.0),
            Rect::new(60.0, -10.0, 124.0, -4.0),
            5.0,
            2.0,
        );
        let entries = [a, b];
        assert_eq!(pick(Vec2::new(75.0, 50.0), &entries), Some(b.entity));
        assert_eq!(pick(Vec2::new(10.0, 50.0), &entries), Some(a.entity));
        assert_eq!(pick(Vec2::new(30.0, -7.0), &entries), Some(a.entity));
        assert_eq!(pick(Vec2::new(500.0, 500.0), &entries), None);
    }

    #[test]
    fn drags_ui_and_bounces_do_not_select() {
        assert!(click_selects(0.0, false, 1.0));
        assert!(click_selects(DRAG_THRESHOLD, false, 1.0));
        assert!(!click_selects(DRAG_THRESHOLD + 0.5, false, 1.0));
        assert!(!click_selects(0.0, true, 1.0));
        assert!(!click_selects(0.0, false, DEBOUNCE / 2.0));
        let hovered = [Interaction::None, Interaction::Hovered];
        assert!(ui_blocked(hovered.iter()));
        assert!(!ui_blocked([Interaction::None].iter()));
    }

    #[test]
    fn tab_cycles_nearby_in_a_stable_order() {
        let r = Rect::new(0.0, 0.0, 1.0, 1.0);
        let entries = [
            entry(3, r, r, 1.0, 9.0),
            entry(1, r, r, 1.0, 3.0),
            entry(2, r, r, 1.0, 3.0),
            entry(4, r, r, 1.0, TAB_RANGE + 1.0),
        ];
        let e = |n| Some(Entity::from_raw_u32(n).unwrap());
        assert_eq!(cycle(&entries, None, false), e(1));
        assert_eq!(cycle(&entries, e(1), false), e(2));
        assert_eq!(cycle(&entries, e(2), false), e(3));
        assert_eq!(cycle(&entries, e(3), false), e(1));
        assert_eq!(cycle(&entries, e(1), true), e(3));
        assert_eq!(cycle(&[], None, false), None);
    }

    #[test]
    fn control_and_dots_become_debuffs() {
        let mut e = enemy(&data::ENEMIES[0], false);
        e.stun_t = 1.5;
        e.slow_t = 3.0;
        assert_eq!(
            debuffs(&e, None).len(),
            1,
            "a slow at full speed is no slow"
        );
        e.slow = 0.5;
        let list = debuffs(&e, None);
        assert_eq!(
            list.iter().map(|d| d.key).collect::<Vec<_>>(),
            ["stun", "slow"]
        );
        let key = data::ABILITIES[0].key;
        for left in [2.0, 4.0] {
            e.dots.push(super::super::enemies::Dot {
                dps: 1.0,
                left,
                tick: 1.0,
                every: 1.0,
                extension_left: 0.0,
                source: super::super::skill_effects::Source {
                    ability: Some(key),
                    ..Default::default()
                },
            });
        }
        let list = debuffs(&e, None);
        let dot = list.iter().find(|d| d.key == key).unwrap();
        assert_eq!((dot.stacks, dot.left), (2, 4.0));
    }

    #[test]
    fn death_despawn_and_a_new_battleground_clear_the_selection() {
        let mut app = App::new();
        app.init_resource::<SelectedTarget>()
            .init_resource::<Selection>()
            .add_systems(Update, validate_target);
        let a = app.world_mut().spawn(enemy(&data::ENEMIES[0], false)).id();
        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(a));
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), Some(a));
        app.world_mut().get_mut::<Enemy>(a).unwrap().dead = true;
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), None);

        let b = app.world_mut().spawn(enemy(&data::ENEMIES[0], false)).id();
        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(b));
        app.world_mut().despawn(b);
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), None);

        let c = app.world_mut().spawn(enemy(&data::ENEMIES[0], false)).id();
        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(c));
        app.world_mut().resource_mut::<Selection>().stage += 1;
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), None);
    }

    fn visible_plates(app: &mut App) -> (usize, usize) {
        let world = app.world_mut();
        let mut q = world.query_filtered::<&Visibility, With<PlateFrame>>();
        let all: Vec<_> = q.iter(world).collect();
        (
            all.iter().filter(|v| ***v == Visibility::Inherited).count(),
            all.len(),
        )
    }

    #[test]
    fn plates_follow_live_on_screen_enemies_from_a_reused_pool() {
        let mut app = App::new();
        app.insert_resource(UiScale(1.0))
            .insert_resource(State::new(Phase::Playing))
            .init_resource::<SelectedTarget>()
            .init_resource::<ScreenCache>()
            .add_systems(Startup, setup_plates)
            .add_systems(Update, (project_enemies, sync_plates).chain());
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(W as u32, H as u32),
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut().spawn((WorldCamera, projection(), eye()));
        let def = &data::ENEMIES[0];
        let a = app
            .world_mut()
            .spawn((enemy(def, false), Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();
        let b = app
            .world_mut()
            .spawn((enemy(def, false), Transform::from_xyz(3.0, 0.0, 0.0)))
            .id();
        let mut dead = enemy(def, false);
        dead.dead = true;
        app.world_mut()
            .spawn((dead, Transform::from_xyz(-3.0, 0.0, 0.0)));
        app.world_mut()
            .spawn((enemy(def, false), Transform::from_xyz(0.0, 0.0, 200.0)));
        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(a));

        app.update();
        app.update();
        let (shown, pooled) = visible_plates(&mut app);
        assert_eq!(shown, 2);
        let ring = app.world().resource::<PlatePool>().ring;
        assert_eq!(
            app.world().get::<Visibility>(ring),
            Some(&Visibility::Inherited)
        );

        app.world_mut().get_mut::<Enemy>(b).unwrap().dead = true;
        app.update();
        let (shown_after, pooled_after) = visible_plates(&mut app);
        assert_eq!(shown_after, 1, "a death hides its plate at once");
        assert_eq!(pooled_after, pooled, "the pool is reused, not rebuilt");

        app.world_mut().get_mut::<Enemy>(a).unwrap().hp = 25.0;
        app.update();
        let slot = app.world().resource::<PlatePool>().normal[0];
        assert_eq!(
            app.world().get::<Node>(slot.fill).unwrap().width,
            Val::Percent(25.0)
        );

        app.insert_resource(State::new(Phase::Menu));
        app.update();
        let root = app.world().resource::<PlatePool>().root;
        assert_eq!(
            app.world().get::<Visibility>(root),
            Some(&Visibility::Hidden)
        );
    }

    #[test]
    fn the_portrait_frames_short_and_giant_bodies() {
        for h in [0.8, 2.5, 6.0, 12.0, 30.0] {
            let feet = Vec3::new(5.0, 3.0, -2.0);
            let cam = portrait_view(feet, Vec3::X, h);
            assert!(cam.translation.is_finite());
            let fwd = cam.forward().as_vec3();
            for y in [0.35 * h, h] {
                let to = (feet + Vec3::Y * y - cam.translation).normalize();
                assert!(
                    fwd.angle_between(to) <= PORTRAIT_FOV * 0.5 + 1e-3,
                    "height {h}: point {y} outside the frame"
                );
            }
            assert!(cam.translation.x > feet.x, "filmed from the front");
        }
        assert!(portrait_view(Vec3::ZERO, Vec3::ZERO, f32::NAN)
            .translation
            .is_finite());
    }

    fn press_release(app: &mut App, from: Vec2, to: Vec2) {
        let window = {
            let world = app.world_mut();
            let mut q = world.query_filtered::<Entity, With<PrimaryWindow>>();
            q.single(world).unwrap()
        };
        let at = |app: &mut App, p: Vec2| {
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(p));
        };
        at(app, from);
        app.world_mut()
            .resource_mut::<Time<Real>>()
            .advance_by(std::time::Duration::from_secs(1));
        {
            let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
            mouse.clear();
            mouse.press(MouseButton::Left);
        }
        app.update();
        at(app, to);
        {
            let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
            mouse.clear();
            mouse.release(MouseButton::Left);
        }
        app.update();
    }

    #[test]
    fn clicks_select_but_ui_drags_and_empty_ground_do_not() {
        let mut app = App::new();
        app.insert_resource(State::new(Phase::Playing))
            .init_resource::<Time<Real>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<SelectedTarget>()
            .init_resource::<ScreenCache>()
            .init_resource::<ClickTracker>()
            .add_systems(Update, select_input);
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(W as u32, H as u32),
                ..default()
            },
            PrimaryWindow,
        ));
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        let target = entry(7, r, Rect::new(0.0, -10.0, 64.0, -4.0), 1.0, 1.0);
        app.world_mut()
            .resource_mut::<ScreenCache>()
            .entries
            .push(target);
        let on = Vec2::new(50.0, 50.0);
        let selected = |app: &App| app.world().resource::<SelectedTarget>().get();

        let ui = app.world_mut().spawn(Interaction::Hovered).id();
        press_release(&mut app, on, on);
        assert_eq!(
            selected(&app),
            None,
            "a click on the UI never reaches the world"
        );

        *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::None;
        press_release(&mut app, on, on + Vec2::X * (DRAG_THRESHOLD * 4.0));
        assert_eq!(selected(&app), None, "a drag stays the camera's");

        press_release(&mut app, on, on);
        assert_eq!(selected(&app), Some(target.entity));

        *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::Pressed;
        press_release(&mut app, Vec2::new(900.0, 600.0), Vec2::new(900.0, 600.0));
        assert_eq!(
            selected(&app),
            Some(target.entity),
            "nor does a press on the UI"
        );

        *app.world_mut().get_mut::<Interaction>(ui).unwrap() = Interaction::None;
        press_release(&mut app, Vec2::new(900.0, 600.0), Vec2::new(900.0, 600.0));
        assert_eq!(selected(&app), None, "empty ground clears");
    }

    #[test]
    fn the_portrait_clone_is_isolated_and_follows_the_selection() {
        let mut app = App::new();
        app.insert_resource(State::new(Phase::Playing))
            .init_resource::<Run>()
            .init_resource::<SelectedTarget>()
            .add_systems(Update, sync_portrait);
        let camera = app
            .world_mut()
            .spawn((
                Camera {
                    is_active: false,
                    ..default()
                },
                Transform::default(),
            ))
            .id();
        app.insert_resource(PortraitRig {
            camera,
            clone: None,
            for_target: None,
        });
        let def = &data::ENEMIES[0];
        let a = app
            .world_mut()
            .spawn((enemy(def, false), Transform::from_xyz(1.0, 0.0, 0.0)))
            .id();
        let b = app
            .world_mut()
            .spawn((enemy(def, true), Transform::from_xyz(-4.0, 0.0, 0.0)))
            .id();
        let clones = |app: &mut App| {
            let world = app.world_mut();
            let mut q = world.query_filtered::<Entity, With<PortraitClone>>();
            q.iter(world).collect::<Vec<_>>()
        };
        let enemies = |app: &mut App| {
            let world = app.world_mut();
            world.query::<&Enemy>().iter(world).count()
        };

        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(a));
        app.update();
        let first = clones(&mut app);
        assert_eq!(first.len(), 1);
        let clone = app.world().entity(first[0]);
        assert!(!clone.contains::<Enemy>());
        assert!(!clone.contains::<Hero>());
        assert!(!clone.contains::<super::super::units::Marks>());
        assert!(clone.contains::<super::super::RunEntity>());
        assert_eq!(
            clone.get::<RenderLayers>(),
            Some(&RenderLayers::layer(PORTRAIT_LAYER))
        );
        assert_eq!(enemies(&mut app), 2, "the clone is never an enemy");
        assert!(app.world().get::<Camera>(camera).unwrap().is_active);

        app.world_mut()
            .resource_mut::<SelectedTarget>()
            .set(Some(b));
        app.update();
        let second = clones(&mut app);
        assert_eq!(second.len(), 1);
        assert_ne!(second[0], first[0], "a new target gets its own clone");

        app.world_mut().resource_mut::<SelectedTarget>().clear();
        app.update();
        assert!(clones(&mut app).is_empty());
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);
        assert_eq!(enemies(&mut app), 2);
    }

    #[test]
    fn a_paused_portrait_does_not_wait_for_the_world_appear_gate() {
        use benilla_world::model_fade::{PendingAppearFade, RenderFade, UnitAppearFade};
        let mut app = App::new();
        app.insert_resource(State::new(Phase::Paused))
            .add_systems(PostUpdate, finish_portrait_appear);
        let root = app
            .world_mut()
            .spawn((
                PortraitClone,
                UnitAppearFade::Pending {
                    since: 0.0,
                    arrived: 0.0,
                },
            ))
            .id();
        let part = app
            .world_mut()
            .spawn(PendingAppearFade {
                since: 0.0,
                arrived: 0.0,
            })
            .id();
        app.world_mut().entity_mut(root).add_child(part);
        let world_part = app
            .world_mut()
            .spawn(PendingAppearFade {
                since: 0.0,
                arrived: 0.0,
            })
            .id();
        app.insert_resource(PortraitRig {
            camera: root,
            clone: Some(root),
            for_target: None,
        });
        app.update();
        assert!(app.world().get::<UnitAppearFade>(root).is_none());
        assert!(app.world().get::<PendingAppearFade>(part).is_none());
        let fade = app.world().get::<RenderFade>(part).unwrap();
        assert_eq!(fade.duration, 0.0);
        assert_eq!(fade.alpha_at(0.0), 1.0);
        assert!(app.world().get::<PendingAppearFade>(world_part).is_some());
    }

    #[test]
    fn the_real_plugin_runs_its_whole_chain_without_query_conflicts() {
        let mut app = App::new();
        app.insert_resource(State::new(Phase::Playing))
            .insert_resource(UiScale(1.0))
            .init_resource::<Time<Real>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Run>()
            .init_resource::<Selection>()
            .add_plugins(plugin);
        app.world_mut().spawn((
            Window {
                resolution: WindowResolution::new(W as u32, H as u32),
                ..default()
            },
            PrimaryWindow,
        ));
        app.world_mut().spawn((WorldCamera, projection(), eye()));
        app.world_mut()
            .spawn((Hero { vy: 0.0, air: 0.0 }, Transform::default()));
        let def = &data::ENEMIES[0];
        let a = app
            .world_mut()
            .spawn((enemy(def, false), Transform::from_xyz(0.0, 0.0, 0.0)))
            .id();
        let boss = app
            .world_mut()
            .spawn((
                enemy(&data::BOSSES[0], true),
                Transform::from_xyz(4.0, 0.0, 0.0),
            ))
            .id();
        app.update();
        assert!(app.world().get_resource::<PortraitRig>().is_some());
        assert_eq!(app.world().resource::<ScreenCache>().entries.len(), 2);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), Some(a));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), Some(a));
        let panel = app.world().resource::<PanelUi>().root;
        assert_eq!(
            app.world().get::<Visibility>(panel),
            Some(&Visibility::Inherited)
        );
        {
            let world = app.world_mut();
            let n = world
                .query_filtered::<(), With<PortraitClone>>()
                .iter(world)
                .count();
            assert_eq!(n, 1);
        }
        let (shown, _) = visible_plates(&mut app);
        assert_eq!(shown, 2, "a plain plate and a boss plate");

        app.world_mut().get_mut::<Enemy>(a).unwrap().dead = true;
        app.update();
        app.update();
        assert_eq!(app.world().resource::<SelectedTarget>().get(), None);
        assert_eq!(
            app.world().get::<Visibility>(panel),
            Some(&Visibility::Hidden)
        );
        let (shown, _) = visible_plates(&mut app);
        assert_eq!(shown, 1);
        let _ = boss;
    }
}
