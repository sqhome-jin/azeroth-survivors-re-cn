//! Survivors-only, class-local progression. The original client's network talents are unrelated.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::hud::{text_bundle, HudFont, UiArt};
use super::{data, localization, Owned, Phase, Run, Selection, Stats};

pub(super) const MAX_POINTS: u8 = 51;
const WARLOCK: u8 = 9;
const NODE_COUNT: usize = 24;
const GOLD: Color = Color::srgb(0.94, 0.76, 0.34);
const EDGE: Color = Color::srgb(0.42, 0.32, 0.12);
const EFFECT_MULTIPLIER: f32 = 2.0;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Loadout(pub [u8; NODE_COUNT]);

#[derive(Clone, Copy)]
pub(super) struct NodeDef {
    pub id: &'static str,
    pub name: &'static str,
    pub zh: &'static str,
    pub icon: &'static str,
    pub desc: &'static str,
    pub desc_zh: &'static str,
}

macro_rules! node {
    ($id:literal, $name:literal, $zh:literal, $icon:literal, $desc:literal, $desc_zh:literal) => {
        NodeDef {
            id: $id,
            name: $name,
            zh: $zh,
            icon: concat!("Interface\\Icons\\", $icon),
            desc: $desc,
            desc_zh: $desc_zh,
        }
    };
}

pub(super) const NODES: [NodeDef; NODE_COUNT] = [
    node!("improved_corruption", "Improved Corruption", "强化腐蚀术", "Spell_Shadow_AbominationExplosion", "Corruption cooldown -6% per rank.", "每级使腐蚀术冷却缩短6%。"),
    node!("improved_agony", "Improved Curse of Agony", "强化痛苦诅咒", "Spell_Shadow_CurseOfSargeras", "Curse of Agony damage +8% per rank.", "每级使痛苦诅咒伤害提高8%。"),
    node!("fel_concentration", "Fel Concentration", "恶魔专注", "Spell_Shadow_FingerOfDeath", "Damage-over-time duration +8% per rank.", "每级使腐蚀术、痛苦诅咒和献祭的持续伤害时间延长8%。"),
    node!("grim_reach", "Grim Reach", "无情延伸", "Spell_Shadow_CallofBone", "Corruption and Curse of Agony range +2 yards per rank.", "每级使腐蚀术和痛苦诅咒射程增加2码。"),
    node!("siphon_life", "Siphon Life", "生命虹吸", "Spell_Shadow_Requiem", "Heal for 0.4% of effective periodic damage per rank; all talent leech capped at 4% maximum health per second.", "每级将持续伤害有效值的0.4%转化为生命；所有天赋吸血合计每秒最多恢复生命上限的4%。"),
    node!("shadow_mastery", "Shadow Mastery", "暗影掌握", "Spell_Shadow_ShadeTrueSight", "Shadow spell damage +6% per rank.", "每级使暗影箭、腐蚀术、痛苦诅咒、死亡缠绕和暗影灼烧伤害提高6%。"),
    node!("contagion", "Contagion", "蔓延痛苦", "Spell_Shadow_PlagueCloud", "Corruption, Curse of Agony and Immolate periodic damage +6% per rank.", "每级使腐蚀术、痛苦诅咒和献祭的持续伤害提高6%。"),
    node!("dark_pact", "Dark Pact", "黑暗契约", "Spell_Shadow_DarkRitual", "Periodic damage +30%; periodic damage heals for an additional 2%, sharing the leech cap.", "持续伤害提高30%，额外将持续伤害有效值的2%转化为生命，与生命虹吸共享恢复上限。"),
    node!("demonic_embrace", "Demonic Embrace", "恶魔之拥", "Spell_Shadow_Metamorphosis", "Maximum health +6% per rank.", "每级使最大生命值提高6%。"),
    node!("fel_armor", "Fel Armor", "恶魔护甲", "Spell_Shadow_RagingScream", "Armor +0.8 per rank.", "每级增加0.8护甲，按现有规则减少每次接触伤害。"),
    node!("fel_vitality", "Fel Vitality", "恶魔活力", "Spell_Nature_Reincarnation", "Regenerate 0.30 health per second per rank.", "每级每秒额外恢复0.30生命值。"),
    node!("fel_stride", "Fel Stride", "恶魔步伐", "Ability_Rogue_Sprint", "Movement speed +4% per rank.", "每级使移动速度提高4%。"),
    node!("improved_death_coil", "Improved Death Coil", "强化死亡缠绕", "Spell_Shadow_DeathCoil", "Death Coil cooldown -8% per rank.", "每级使死亡缠绕冷却缩短8%。"),
    node!("improved_howl", "Improved Howl of Terror", "强化恐惧嚎叫", "Spell_Shadow_DeathScream", "F: Howl of Terror cooldown -8% per rank. Still learned at level 5.", "每级使F键恐惧嚎叫冷却缩短8%；仍需局内5级解锁。"),
    node!("master_demonologist", "Master Demonologist", "恶魔学识大师", "Spell_Shadow_AntiShadow", "Incoming damage -4% per rank after armor, before shields.", "每级使护甲结算后、护盾吸收前的所受伤害降低4%。"),
    node!("soul_link", "Soul Link", "灵魂链接", "Spell_Shadow_GatherShadows", "Incoming damage -20%; no pet required. Multiplicative with Master Demonologist.", "所受伤害降低20%，不需要宠物；与恶魔学识大师乘算。"),
    node!("improved_shadow_bolt", "Improved Shadow Bolt", "强化暗影箭", "Spell_Shadow_ShadowBolt", "Shadow Bolt damage +6% per rank.", "每级使暗影箭伤害提高6%。"),
    node!("improved_immolate", "Improved Immolate", "强化献祭", "Spell_Fire_Immolation", "Immolate damage +6% per rank.", "每级使献祭的直接和持续伤害提高6%。"),
    node!("devastation", "Devastation", "毁灭", "Spell_Fire_FlameShock", "Direct damage critical strike chance +2 percentage points per rank.", "每级使直接伤害暴击率提高2个百分点；持续伤害不暴击。"),
    node!("bane", "Bane", "灾祸", "Spell_Shadow_DeathPact", "Shadow Bolt, Immolate and Searing Pain cooldown -4% per rank.", "每级使暗影箭、献祭和灼热之痛冷却缩短4%。"),
    node!("destructive_reach", "Destructive Reach", "毁灭延伸", "Spell_Shadow_CorpseExplode", "Rain of Fire and Hellfire radius +6% per rank.", "每级使火焰之雨和地狱烈焰半径提高6%。"),
    node!("improved_shadowburn", "Improved Shadowburn", "强化暗影灼烧", "Spell_Shadow_ScourgeBuild", "Shadowburn damage +8% per rank.", "每级使暗影灼烧伤害提高8%。"),
    node!("emberstorm", "Emberstorm", "琥珀风暴", "Spell_Fire_Volcano", "Fire spell damage +6% per rank.", "每级使献祭、火焰之雨、地狱烈焰和灼热之痛伤害提高6%。"),
    node!("ruin", "Ruin", "毁灭之力", "Spell_Shadow_ShadowWordPain", "Direct damage critical multiplier increases from 2.0 to 3.0. Periodic damage cannot crit.", "直接伤害暴击倍率从2.0提高至3.0，持续伤害仍不暴击。"),
];

const TIERS: [u8; 8] = [0, 0, 1, 2, 3, 4, 5, 6];
const TREES: [(&str, &str); 3] = [
    ("Affliction", "痛苦"),
    ("Demonology", "恶魔学识"),
    ("Destruction", "毁灭"),
];

fn max_rank(index: usize) -> u8 {
    if index % 8 == 7 {
        1
    } else {
        5
    }
}

impl Loadout {
    pub fn spent(self) -> u8 {
        self.0
            .iter()
            .fold(0u8, |sum, &rank| sum.saturating_add(rank))
    }

    fn tree_spent(self, tree: usize) -> u8 {
        self.0[tree * 8..tree * 8 + 8]
            .iter()
            .fold(0u8, |sum, &rank| sum.saturating_add(rank))
    }

    fn valid(self, points: u8) -> bool {
        self.spent() <= points
            && self.0.iter().enumerate().all(|(i, &rank)| {
                let lower: u16 = self.0[i / 8 * 8..i / 8 * 8 + 8]
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| TIERS[*j] < TIERS[i % 8])
                    .map(|(_, r)| u16::from(*r))
                    .sum();
                rank <= max_rank(i) && (rank == 0 || lower >= u16::from(TIERS[i % 8]) * 5)
            })
    }

    fn learn(&mut self, index: usize, points: u8) -> bool {
        if index >= NODE_COUNT || self.0[index] >= max_rank(index) {
            return false;
        }
        let mut next = *self;
        next.0[index] += 1;
        let lower: u16 = next.0[index / 8 * 8..index / 8 * 8 + 8]
            .iter()
            .enumerate()
            .filter(|(j, _)| TIERS[*j] < TIERS[index % 8])
            .map(|(_, r)| u16::from(*r))
            .sum();
        if next.spent() > points || lower < u16::from(TIERS[index % 8]) * 5 {
            return false;
        }
        *self = next;
        true
    }

    fn rank(self, index: usize) -> f32 {
        f32::from(self.0[index]) * EFFECT_MULTIPLIER
    }

    pub fn incoming(self) -> f32 {
        (1.0 - 0.02 * self.rank(14)) * (1.0 - 0.10 * self.rank(15))
    }

    pub fn mobility_cooldown(self) -> f32 {
        1.0 - 0.04 * self.rank(13)
    }

    pub fn leech(self) -> f32 {
        0.002 * self.rank(4) + 0.01 * self.rank(7)
    }

    pub fn critical_multiplier(self) -> f32 {
        2.0 + 0.5 * self.rank(23)
    }
}

fn threshold(point: u8) -> u64 {
    match point {
        0 => 0,
        1..=5 => u64::from(point) * u64::from(point + 1) * 50,
        6..=15 => 1_500 + u64::from(point - 5) * 750,
        16..=25 => 9_000 + u64::from(point - 15) * 1_500,
        _ => 24_000 + u64::from(point - 25) * 2_500,
    }
}

fn points_for(score: u64) -> u8 {
    (1..=MAX_POINTS)
        .take_while(|&p| score >= threshold(p))
        .count() as u8
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ClassProgress {
    normal_kills: u64,
    boss_kills: u64,
    talents: BTreeMap<String, u8>,
}

impl ClassProgress {
    fn score(&self) -> u64 {
        self.normal_kills
            .saturating_add(self.boss_kills.saturating_mul(10))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    version: u32,
    classes: BTreeMap<String, ClassProgress>,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            version: 1,
            classes: BTreeMap::new(),
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct Progression {
    save: Save,
    dirty: bool,
    read_only: bool,
    save_elapsed: f32,
    pub error: Option<String>,
}

impl Progression {
    pub fn points(&self, class: u8) -> u8 {
        points_for(self.score(class))
    }

    pub fn score(&self, class: u8) -> u64 {
        self.save
            .classes
            .get(&class.to_string())
            .map_or(0, ClassProgress::score)
    }

    pub fn spent(&self, class: u8) -> u8 {
        self.loadout(class).spent()
    }

    pub fn loadout(&self, class: u8) -> Loadout {
        let mut result = Loadout::default();
        if class == WARLOCK {
            if let Some(saved) = self.save.classes.get(&class.to_string()) {
                for (i, def) in NODES.iter().enumerate() {
                    result.0[i] = saved.talents.get(def.id).copied().unwrap_or(0);
                }
            }
        }
        result
    }

    fn apply(&mut self, class: u8, loadout: Loadout) -> bool {
        if self.read_only || class != WARLOCK || !loadout.valid(self.points(class)) {
            return false;
        }
        let entry = self.save.classes.entry(class.to_string()).or_default();
        entry.talents = NODES
            .iter()
            .zip(loadout.0)
            .filter(|(_, rank)| *rank > 0)
            .map(|(def, rank)| (def.id.to_owned(), rank))
            .collect();
        self.dirty = true;
        true
    }
}

#[derive(Resource, Default)]
pub(super) struct TalentUi {
    pub open: bool,
    draft: Loadout,
    selected: usize,
    message: String,
}

#[derive(Component)]
struct TalentRoot;

#[derive(Component, Clone, Copy)]
enum Action {
    Select(usize),
    Learn,
    Unlearn,
    Reset,
    Apply,
    Cancel,
}

#[derive(Component)]
enum Label {
    Summary,
    Detail,
    Rank(usize),
    Tree(usize),
    Available,
    NodeName,
    Requirement,
}

#[derive(Component)]
struct TalentIcon(usize);

#[derive(Component)]
struct DetailIcon(usize);

#[derive(Component)]
struct ProgressFill;

#[derive(Component)]
struct TierLine {
    tree: usize,
    tier: u8,
}

const POSITIONS: [(f32, f32); 8] = [
    (37.5, 6.0),
    (62.5, 6.0),
    (12.5, 20.0),
    (62.5, 34.0),
    (37.5, 48.0),
    (62.5, 62.0),
    (37.5, 76.0),
    (37.5, 90.0),
];

const BACKGROUNDS: [[&str; 4]; 3] = [
    [
        "Interface\\TalentFrame\\WarlockCurses-TopLeft",
        "Interface\\TalentFrame\\WarlockCurses-TopRight",
        "Interface\\TalentFrame\\WarlockCurses-BottomLeft",
        "Interface\\TalentFrame\\WarlockCurses-BottomRight",
    ],
    [
        "Interface\\TalentFrame\\WarlockSummoning-TopLeft",
        "Interface\\TalentFrame\\WarlockSummoning-TopRight",
        "Interface\\TalentFrame\\WarlockSummoning-BottomLeft",
        "Interface\\TalentFrame\\WarlockSummoning-BottomRight",
    ],
    [
        "Interface\\TalentFrame\\WarlockDestruction-TopLeft",
        "Interface\\TalentFrame\\WarlockDestruction-TopRight",
        "Interface\\TalentFrame\\WarlockDestruction-BottomLeft",
        "Interface\\TalentFrame\\WarlockDestruction-BottomRight",
    ],
];

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Progression>()
        .init_resource::<TalentUi>()
        .add_systems(Startup, load)
        .add_systems(OnEnter(Phase::Playing), lock_loadout)
        .add_systems(
            Update,
            credit
                .after(super::abilities::apply_hits)
                .before(super::hero::hero_vitals),
        )
        .add_systems(Update, persist.after(credit))
        .add_systems(
            Update,
            (build_panel, panel_input, refresh_panel)
                .chain()
                .after(super::menu::menu_clicks),
        );
}

fn save_path() -> Option<std::path::PathBuf> {
    crate::local_state::home().map(|p| p.join("survivors-progress.toml"))
}

fn load(mut progression: ResMut<Progression>) {
    let Some(path) = save_path() else { return };
    let result = crate::local_state::read_to_string(&path);
    match result {
        Ok(text) => match toml::from_str::<Save>(&text) {
            Ok(save) if save.version == 1 => {
                progression.save = save;
                let loadout = progression.loadout(WARLOCK);
                let unknown = progression
                    .save
                    .classes
                    .get(&WARLOCK.to_string())
                    .is_some_and(|p| p.talents.keys().any(|id| !NODES.iter().any(|n| n.id == id)));
                if unknown || !loadout.valid(progression.points(WARLOCK)) {
                    progression.apply(WARLOCK, Loadout::default());
                    progression.error = Some(
                        localization::text(
                            "Talents changed; points refunded.",
                            "天赋数据已变更，配点已返还，击杀进度保留。",
                        )
                        .into(),
                    );
                    warn!("survivors talents: incompatible allocation refunded");
                }
            }
            Ok(_) => fail_load(&mut progression, "unsupported save version"),
            Err(error) => fail_load(&mut progression, &error.to_string()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => fail_load(&mut progression, &error.to_string()),
    }
}

fn fail_load(progression: &mut Progression, reason: &str) {
    warn!("survivors progression: cannot load: {reason}");
    progression.read_only = true;
    progression.error = Some(
        localization::text(
            "Cannot read progress. Saving disabled to protect the existing save.",
            "无法读取进度，已停止写入以保护原存档。请备份并检查存档。",
        )
        .into(),
    );
}

fn lock_loadout(mut run: ResMut<Run>, progression: Res<Progression>) {
    if !run.talents_locked {
        run.talents = progression.loadout(run.class.id);
        run.talents_locked = true;
        run.hp = run.stats().max_hp;
        run.version += 1;
    }
}

fn credit(mut run: ResMut<Run>, mut progression: ResMut<Progression>) {
    let kills = run.kills.saturating_sub(run.credited_kills);
    let bosses = run.boss_kills.saturating_sub(run.credited_bosses);
    if kills == 0 {
        return;
    }
    run.credited_kills = run.kills;
    run.credited_bosses = run.boss_kills;
    if progression.read_only {
        return;
    }
    let class = run.class.id;
    let before = progression.points(class);
    let entry = progression
        .save
        .classes
        .entry(class.to_string())
        .or_default();
    entry.normal_kills = entry
        .normal_kills
        .saturating_add(u64::from(kills.saturating_sub(bosses)));
    entry.boss_kills = entry.boss_kills.saturating_add(u64::from(bosses));
    progression.dirty = true;
    let gained = progression.points(class) - before;
    if gained > 0 {
        run.talent_points_gained += gained;
        run.banner = Some((
            format!(
                "{} +{gained} / 51",
                localization::text("Talent points", "天赋点")
            ),
            4.0,
        ));
        progression.save_elapsed = 5.0;
    }
}

fn persist(
    time: Res<Time<Real>>,
    phase: Res<State<Phase>>,
    mut progression: ResMut<Progression>,
    exit: MessageReader<AppExit>,
) {
    progression.save_elapsed += time.delta_secs();
    let leaving =
        matches!(phase.get(), Phase::Menu | Phase::GameOver | Phase::Paused) || !exit.is_empty();
    if !progression.dirty || progression.read_only || (!leaving && progression.save_elapsed < 5.0) {
        return;
    }
    // Retry at most every five seconds after a failure, even on a menu.
    if progression.error.is_some() && progression.save_elapsed < 5.0 {
        return;
    }
    progression.save_elapsed = 0.0;
    let result = save_path()
        .ok_or_else(|| "local persistence unavailable".to_owned())
        .and_then(|path| {
            toml::to_string(&progression.save)
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    crate::local_state::write_atomic(&path, &text).map_err(|e| e.to_string())
                })
        });
    match result {
        Ok(()) => {
            progression.dirty = false;
            progression.error = None;
        }
        Err(error) => {
            warn!("survivors progression: save failed: {error}");
            progression.error = Some(
                localization::text("Progress not saved; retrying.", "进度尚未保存，正在重试。")
                    .into(),
            );
        }
    }
}

/// Shared by the actual cast and every HUD preview.
pub(super) struct Cast {
    pub dmg: f32,
    pub dot: f32,
    pub count: u32,
    pub radius: f32,
    pub duration: f32,
    pub cooldown: f32,
    pub range: f32,
}

pub(super) fn cast(def: &'static data::AbilityDef, level: u8, stats: &Stats) -> Cast {
    let owned = Owned {
        def,
        level,
        cd: 0.0,
    };
    let t = stats.talents;
    let r = |i| t.rank(i);
    let shadow = matches!(
        def.key,
        "shadow_bolt" | "corruption" | "curse_of_agony" | "death_coil" | "shadowburn"
    );
    let fire = matches!(
        def.key,
        "immolate" | "rain_of_fire" | "hellfire" | "searing_pain"
    );
    let damage = 1.0
        + if shadow { 0.03 * r(5) } else { 0.0 }
        + if fire { 0.03 * r(22) } else { 0.0 }
        + match def.key {
            "curse_of_agony" => 0.04 * r(1),
            "shadow_bolt" => 0.03 * r(16),
            "immolate" => 0.03 * r(17),
            "shadowburn" => 0.04 * r(21),
            _ => 0.0,
        };
    let periodic = matches!(def.key, "corruption" | "curse_of_agony" | "immolate");
    let cooldown = match def.key {
        "corruption" => 0.03 * r(0),
        "death_coil" => 0.04 * r(12),
        "shadow_bolt" | "immolate" | "searing_pain" => 0.02 * r(19),
        _ => 0.0,
    };
    let mult = owned.damage_mult() * stats.might * damage;
    Cast {
        dmg: def.damage * mult,
        dot: def.dot
            * mult
            * if periodic {
                1.0 + 0.03 * r(6) + 0.15 * r(7)
            } else {
                1.0
            },
        count: owned.count(),
        radius: owned.radius(stats)
            * if matches!(def.key, "rain_of_fire" | "hellfire") {
                1.0 + 0.03 * r(20)
            } else {
                1.0
            },
        duration: owned.duration() * if periodic { 1.0 + 0.04 * r(2) } else { 1.0 },
        cooldown: (owned.cooldown(stats) * (1.0 - cooldown)).max(0.2),
        range: def.range
            + if matches!(def.key, "corruption" | "curse_of_agony") {
                r(3)
            } else {
                0.0
            },
    }
}

pub(super) fn modify_stats(stats: &mut Stats) {
    let t = stats.talents;
    stats.max_hp *= 1.0 + 0.03 * t.rank(8);
    stats.armor += 0.4 * t.rank(9);
    stats.regen += 0.15 * t.rank(10);
    stats.speed *= 1.0 + 0.02 * t.rank(11);
    stats.crit = (stats.crit + 0.01 * t.rank(18)).min(1.0);
}

pub(super) fn periodic_heal(run: &mut Run, effective_damage: f32, dt: f32) {
    if run.dead_t.is_some() {
        return;
    }
    let max_hp = run.stats().max_hp;
    run.talent_leech_budget = (run.talent_leech_budget + max_hp * 0.04 * dt).min(max_hp * 0.04);
    let heal = (effective_damage * run.talents.leech()).min(run.talent_leech_budget);
    run.talent_leech_budget -= heal;
    run.hp = (run.hp + heal).min(max_hp);
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

fn build_panel(
    mut commands: Commands,
    font: Option<Res<HudFont>>,
    roots: Query<(), With<TalentRoot>>,
) {
    if !roots.is_empty() {
        return;
    }
    let Some(font) = font else { return };
    let f = &font.0;
    commands.spawn((
        TalentRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0), height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)), row_gap: Val::Px(10.0),
            display: Display::None, ..default()
        },
        GlobalZIndex(460),
        BackgroundColor(Color::srgba(0.015, 0.012, 0.008, 0.98)),
    )).with_children(|root| {
        root.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, ..default() })
            .with_children(|header| {
                header.spawn(Node { flex_direction: FlexDirection::Column, ..default() }).with_children(|title| {
                    title.spawn(text_bundle(f, localization::text("TALENTS", "天赋"), 32.0, GOLD));
                    title.spawn(text_bundle(f, localization::text("WARLOCK · Free respec · Applies next run", "术士 · 免费重置 · 下一局生效"), 12.0, GOLD));
                });
                header.spawn(Node { column_gap: Val::Px(10.0), ..default() }).with_children(|buttons| {
                    button(buttons, f, Action::Cancel, localization::text("Back", "返回"));
                    button(buttons, f, Action::Apply, localization::text("Apply / Done", "应用 / 完成"));
                });
            });
        root.spawn(Node { flex_grow: 1.0, min_height: Val::Px(0.0), column_gap: Val::Px(14.0), ..default() })
            .with_children(|row| {
                for tree in 0..3 {
                    row.spawn((
                        Node {
                            flex_grow: 1.0, flex_basis: Val::Px(0.0), min_width: Val::Px(0.0),
                            padding: UiRect::all(Val::Px(9.0)), row_gap: Val::Px(8.0),
                            flex_direction: FlexDirection::Column, border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(7.0)), ..default()
                        },
                        BackgroundColor(Color::srgb(0.035, 0.027, 0.017)), BorderColor::all(EDGE),
                    )).with_children(|column| {
                        column.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(8.0), ..default() }).with_children(|head| {
                            head.spawn((UiArt::crop(NODES[tree * 8].icon, [0.0, 1.0, 0.0, 1.0]), ImageNode::default(),
                                Node { width: Val::Px(26.0), height: Val::Px(26.0), ..default() }));
                            head.spawn((Label::Tree(tree), text_bundle(f, "", 18.0, GOLD)));
                        });
                        column.spawn((
                            Node { width: Val::Percent(100.0), height: Val::Px(460.0), flex_shrink: 0.0,
                                border: UiRect::all(Val::Px(1.0)), overflow: Overflow::clip(), ..default() },
                            BorderColor::all(EDGE),
                        )).with_children(|field| {
                            background(field, tree);
                            // Lines indicate tier progression, not extra node prerequisites.
                            for tier in 1..=6 {
                                field.spawn((
                                    TierLine { tree, tier },
                                    Node { position_type: PositionType::Absolute,
                                        left: Val::Percent(50.0), top: Val::Percent(6.0 + f32::from(tier - 1) * 14.0),
                                        width: Val::Px(2.0), height: Val::Percent(14.0), ..default() },
                                    BackgroundColor(EDGE),
                                ));
                                let (x, y) = POSITIONS[usize::from(tier) + 1];
                                field.spawn((
                                    TierLine { tree, tier },
                                    Node { position_type: PositionType::Absolute,
                                        left: Val::Percent(x.min(50.0)), top: Val::Percent(y),
                                        width: Val::Percent((x - 50.0).abs()), height: Val::Px(2.0), ..default() },
                                    BackgroundColor(EDGE),
                                ));
                            }
                            field.spawn((Node { position_type: PositionType::Absolute,
                                left: Val::Percent(37.5), top: Val::Percent(6.0),
                                width: Val::Percent(25.0), height: Val::Px(2.0), ..default() }, BackgroundColor(EDGE)));
                            for (j, &(x, y)) in POSITIONS.iter().enumerate() {
                                let i = tree * 8 + j;
                                field.spawn((
                                    Button, Action::Select(i),
                                    Node { position_type: PositionType::Absolute,
                                        left: Val::Percent(x), top: Val::Percent(y),
                                        margin: UiRect::new(Val::Px(-19.0), Val::ZERO, Val::Px(-19.0), Val::ZERO),
                                        width: Val::Px(38.0), height: Val::Px(38.0),
                                        border: UiRect::all(Val::Px(2.0)), border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default() },
                                    BackgroundColor(Color::BLACK), BorderColor::all(EDGE),
                                )).with_children(|b| {
                                    b.spawn((TalentIcon(i), UiArt::crop(NODES[i].icon, [0.0, 1.0, 0.0, 1.0]), ImageNode::default(),
                                        Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }));
                                    b.spawn((Node { position_type: PositionType::Absolute,
                                        right: Val::Px(-5.0), bottom: Val::Px(-8.0),
                                        padding: UiRect::horizontal(Val::Px(3.0)),
                                        border_radius: BorderRadius::all(Val::Px(2.0)), ..default() },
                                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
                                    )).with_children(|badge| { badge.spawn((Label::Rank(i), text_bundle(f, "", 11.0, GOLD))); });
                                });
                            }
                        });
                        for (tier, j) in [(10, 3), (20, 5), (30, 7)] {
                            let def = &NODES[tree * 8 + j];
                            column.spawn(Node { align_items: AlignItems::Center, column_gap: Val::Px(6.0), ..default() }).with_children(|legend| {
                                legend.spawn((UiArt::crop(def.icon, [0.0, 1.0, 0.0, 1.0]), ImageNode::default(),
                                    Node { width: Val::Px(20.0), height: Val::Px(20.0), ..default() }));
                                legend.spawn(text_bundle(f, format!("{tier}{} · {}", localization::text(" pts", "点"), localization::text(def.name, def.zh)), 11.0, GOLD));
                            });
                        }
                    });
                }
                row.spawn((
                    Node { width: Val::Percent(23.0), min_width: Val::Px(210.0),
                        padding: UiRect::all(Val::Px(14.0)), row_gap: Val::Px(12.0),
                        flex_direction: FlexDirection::Column, border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(7.0)), ..default() },
                    BackgroundColor(Color::srgb(0.035, 0.027, 0.017)), BorderColor::all(EDGE),
                )).with_children(|side| {
                    side.spawn(text_bundle(f, localization::text("TALENT POINTS", "天赋点数"), 20.0, GOLD));
                    side.spawn((Label::Available, text_bundle(f, "", 24.0, GOLD)));
                    side.spawn((Label::Summary, text_bundle(f, "", 12.0, GOLD)));
                    side.spawn((Node { height: Val::Px(6.0), width: Val::Percent(100.0), ..default() }, BackgroundColor(Color::srgb(0.12, 0.10, 0.06))))
                        .with_children(|bar| { bar.spawn((ProgressFill, Node { height: Val::Percent(100.0), width: Val::Percent(0.0), ..default() }, BackgroundColor(Color::srgb(0.35, 0.75, 0.85)))); });
                    side.spawn(text_bundle(f, localization::text("Minion 1 / Boss 10. Persistent, class-local progress.\nSelect an icon to inspect. Gold lines show tier unlocks.", "小兵1 / 首领10，跨局累计，各职业独立。\n点击图标查看天赋，连线表示层级解锁。"), 11.0, GOLD));
                    button(side, f, Action::Reset, localization::text("Reset", "重置"));
                    side.spawn((Node { height: Val::Px(1.0), margin: UiRect::vertical(Val::Px(4.0)), ..default() }, BackgroundColor(EDGE)));
                    side.spawn(Node { min_height: Val::Px(42.0), column_gap: Val::Px(10.0), align_items: AlignItems::Center, ..default() }).with_children(|head| {
                        head.spawn(Node { width: Val::Px(40.0), height: Val::Px(40.0), flex_shrink: 0.0, ..default() }).with_children(|icons| {
                            for (i, def) in NODES.iter().enumerate() {
                                icons.spawn((DetailIcon(i), UiArt::crop(def.icon, [0.0, 1.0, 0.0, 1.0]), ImageNode::default(),
                                    Node { width: Val::Px(40.0), height: Val::Px(40.0), display: Display::None, ..default() }));
                            }
                        });
                        head.spawn((Label::NodeName, text_bundle(f, "", 13.0, GOLD)));
                    });
                    side.spawn((Label::Requirement, text_bundle(f, "", 11.0, GOLD)));
                    side.spawn((Label::Detail, text_bundle(f, "", 13.0, GOLD)));
                    side.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: Val::Px(8.0), row_gap: Val::Px(6.0), ..default() }).with_children(|buttons| {
                        button(buttons, f, Action::Learn, localization::text("Learn", "学习"));
                        button(buttons, f, Action::Unlearn, localization::text("Unlearn", "遗忘"));
                    });
                });
            });
    });
}

fn background(parent: &mut ChildSpawnerCommands, tree: usize) {
    // The original 300x331 canvas is tiled into padded power-of-two textures.
    for (part, (left, top, width, height, crop)) in [
        (0.0, 0.0, 256.0 / 300.0, 256.0 / 331.0, [0.0, 1.0, 0.0, 1.0]),
        (
            256.0 / 300.0,
            0.0,
            44.0 / 300.0,
            256.0 / 331.0,
            [0.0, 44.0 / 64.0, 0.0, 1.0],
        ),
        (
            0.0,
            256.0 / 331.0,
            256.0 / 300.0,
            75.0 / 331.0,
            [0.0, 1.0, 0.0, 75.0 / 128.0],
        ),
        (
            256.0 / 300.0,
            256.0 / 331.0,
            44.0 / 300.0,
            75.0 / 331.0,
            [0.0, 44.0 / 64.0, 0.0, 75.0 / 128.0],
        ),
    ]
    .into_iter()
    .enumerate()
    {
        parent.spawn((
            UiArt::crop(BACKGROUNDS[tree][part], crop),
            ImageNode::default(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(left * 100.0),
                top: Val::Percent(top * 100.0),
                width: Val::Percent(width * 100.0),
                height: Val::Percent(height * 100.0),
                ..default()
            },
        ));
    }
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.32)),
    ));
}

fn panel_input(
    mut ui: ResMut<TalentUi>,
    mut progression: ResMut<Progression>,
    phase: Res<State<Phase>>,
    selection: Res<Selection>,
    keys: Res<ButtonInput<KeyCode>>,
    presses: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut was_open: Local<bool>,
) {
    if *phase.get() != Phase::Menu || selection.class_def().is_none_or(|c| c.id != WARLOCK) {
        ui.open = false;
    }
    if ui.open && !*was_open {
        ui.draft = progression.loadout(WARLOCK);
        ui.message.clear();
    }
    *was_open = ui.open;
    if !ui.open {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        ui.open = false;
        *was_open = false;
        return;
    }
    for (interaction, action) in &presses {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            Action::Select(i) => {
                ui.selected = i;
                ui.message.clear();
            }
            Action::Learn => {
                let i = ui.selected;
                if !ui.draft.learn(i, progression.points(WARLOCK)) {
                    ui.message = localization::text(
                        "Cannot learn: check points, rank and lower tiers.",
                        "无法学习：请检查可用点数、等级上限和低层投入。",
                    )
                    .into();
                } else {
                    ui.message.clear();
                }
            }
            Action::Unlearn => {
                let i = ui.selected;
                ui.draft.0[i] = ui.draft.0[i].saturating_sub(1);
                ui.message.clear();
            }
            Action::Reset => {
                ui.draft = Loadout::default();
                ui.message.clear();
            }
            Action::Apply => {
                if progression.apply(WARLOCK, ui.draft) {
                    progression.save_elapsed = 5.0;
                    ui.open = false;
                } else {
                    ui.message = localization::text(
                        "Cannot apply: invalid build or save is read-only.",
                        "无法应用：配点不满足层级要求，或存档处于只读保护状态。",
                    )
                    .into();
                }
            }
            Action::Cancel => ui.open = false,
        }
    }
    *was_open = ui.open;
}

fn refresh_panel(
    ui: Res<TalentUi>,
    progression: Res<Progression>,
    mut roots: Query<&mut Node, With<TalentRoot>>,
    mut labels: Query<(&Label, &mut Text, &mut TextColor)>,
    mut buttons: Query<(&Action, &mut BorderColor, &mut BackgroundColor)>,
    mut icons: Query<(&TalentIcon, &mut ImageNode)>,
    mut detail_icons: Query<(&DetailIcon, &mut Node), Without<TalentRoot>>,
    mut progress_fill: Query<
        &mut Node,
        (With<ProgressFill>, Without<TalentRoot>, Without<DetailIcon>),
    >,
    mut lines: Query<(&TierLine, &mut BackgroundColor), Without<Action>>,
) {
    for mut root in &mut roots {
        root.display = if ui.open {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !ui.open {
        return;
    }
    let points = progression.points(WARLOCK);
    let score = progression.score(WARLOCK);
    let next = if points < MAX_POINTS {
        threshold(points + 1).saturating_sub(score).to_string()
    } else {
        localization::text("MAX", "已满").into()
    };
    for (label, mut text, mut color) in &mut labels {
        let value = match *label {
            Label::Summary => format!(
                "{} {}/51 · {} {}\n{}/{}/{}\n{} {score}\n{} {next}\n{}",
                localization::text("Earned", "已获得"),
                points,
                localization::text("Spent", "已投入"),
                ui.draft.spent(),
                ui.draft.tree_spent(0),
                ui.draft.tree_spent(1),
                ui.draft.tree_spent(2),
                localization::text("Kill credit", "累计击杀进度"),
                localization::text("Until next", "距离下一点"),
                progression.error.as_deref().unwrap_or(&ui.message),
            ),
            Label::NodeName => {
                let i = ui.selected;
                let def = &NODES[i];
                format!(
                    "{}\n{} {}/{}",
                    localization::text(def.name, def.zh),
                    localization::text("Rank", "等级"),
                    ui.draft.0[i],
                    max_rank(i)
                )
            }
            Label::Detail => {
                localization::text(NODES[ui.selected].desc, NODES[ui.selected].desc_zh).into()
            }
            Label::Requirement => {
                let i = ui.selected;
                let need = u16::from(TIERS[i % 8]) * 5;
                let lower: u16 = ui.draft.0[i / 8 * 8..i / 8 * 8 + 8]
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| TIERS[*j] < TIERS[i % 8])
                    .map(|(_, r)| u16::from(*r))
                    .sum();
                color.0 = if lower < need {
                    Color::srgb(1.0, 0.2, 0.12)
                } else {
                    Color::srgb(0.35, 0.8, 0.3)
                };
                format!(
                    "{} {need} ({lower}/{need})",
                    localization::text("Lower-tier points required:", "需要本系低层投入：")
                )
            }
            Label::Rank(i) => {
                color.0 = if ui.draft.0[i] == max_rank(i) {
                    GOLD
                } else if ui.draft.0[i] > 0 {
                    Color::srgb(0.25, 1.0, 0.3)
                } else {
                    Color::srgb(0.65, 0.62, 0.52)
                };
                format!("{}/{}", ui.draft.0[i], max_rank(i))
            }
            Label::Tree(i) => format!(
                "{} · {}",
                localization::text(TREES[i].0, TREES[i].1),
                ui.draft.tree_spent(i)
            ),
            Label::Available => format!(
                "{} {}",
                localization::text("Available", "可分配"),
                points.saturating_sub(ui.draft.spent())
            ),
        };
        if **text != value {
            **text = value;
        }
    }
    for (action, mut border, mut bg) in &mut buttons {
        if let Action::Select(i) = *action {
            let mut probe = ui.draft;
            let learnable = probe.learn(i, points);
            *border = BorderColor::all(if i == ui.selected || ui.draft.0[i] == max_rank(i) {
                GOLD
            } else if ui.draft.0[i] > 0 || learnable {
                Color::srgb(0.2, 0.8, 0.25)
            } else {
                EDGE
            });
            bg.0 = Color::BLACK;
        }
    }
    for (icon, mut image) in &mut icons {
        let mut probe = ui.draft;
        image.color =
            if ui.draft.0[icon.0] > 0 || probe.learn(icon.0, points) || icon.0 == ui.selected {
                Color::WHITE
            } else {
                Color::srgb(0.35, 0.35, 0.35)
            };
    }
    for (icon, mut node) in &mut detail_icons {
        node.display = if icon.0 == ui.selected {
            Display::Flex
        } else {
            Display::None
        };
    }
    for mut node in &mut progress_fill {
        let previous = threshold(points);
        let fraction = if points == MAX_POINTS {
            1.0
        } else {
            score.saturating_sub(previous) as f32 / (threshold(points + 1) - previous) as f32
        };
        node.width = Val::Percent((fraction * 100.0).clamp(0.0, 100.0));
    }
    for (line, mut color) in &mut lines {
        let lower: u16 = ui.draft.0[line.tree * 8..line.tree * 8 + 8]
            .iter()
            .enumerate()
            .filter(|(j, _)| TIERS[*j] < line.tier)
            .map(|(_, r)| u16::from(*r))
            .sum();
        color.0 = if lower >= u16::from(line.tier) * 5 {
            GOLD
        } else {
            Color::srgba(0.35, 0.31, 0.22, 0.6)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn talent_icons_are_unique_asset_paths() {
        let mut icons = std::collections::BTreeSet::new();
        for node in &NODES {
            assert!(node.icon.starts_with("Interface\\Icons\\"));
            assert!(
                icons.insert(node.icon.to_ascii_lowercase()),
                "duplicate icon for {}",
                node.id
            );
        }
        assert_eq!(icons.len(), NODE_COUNT);
    }

    #[test]
    fn thresholds_and_cap() {
        for point in 1..=MAX_POINTS {
            assert_eq!(points_for(threshold(point) - 1), point - 1);
            assert_eq!(points_for(threshold(point)), point);
        }
        assert_eq!(threshold(51), 89_000);
        assert_eq!(points_for(u64::MAX), 51);
    }

    #[test]
    fn boss_is_ten_and_classes_are_independent() {
        let mut p = Progression::default();
        p.save.classes.insert(
            "9".into(),
            ClassProgress {
                normal_kills: 90,
                boss_kills: 1,
                ..default()
            },
        );
        assert_eq!(p.score(9), 100);
        assert_eq!(p.points(9), 1);
        assert_eq!(p.points(1), 0);
    }

    #[test]
    fn tree_gates_budget_and_refund() {
        let mut t = Loadout::default();
        assert!(!t.learn(7, 51));
        for i in [0, 2, 3, 4, 5, 6] {
            for _ in 0..5 {
                assert!(t.learn(i, 51));
            }
        }
        assert!(t.learn(7, 51));
        assert_eq!(t.spent(), 31);
        assert!(!t.learn(7, 51));
        t.0[0] -= 1;
        assert!(!t.valid(51));
        assert!(Loadout::default().valid(51));
        assert!(!t.valid(29));
    }

    #[test]
    fn save_roundtrip_and_reset_preserve_credit() {
        let mut p = Progression::default();
        p.save.classes.insert(
            "9".into(),
            ClassProgress {
                normal_kills: 89_000,
                ..default()
            },
        );
        let mut t = Loadout::default();
        t.0[0] = 5;
        assert!(p.apply(9, t));
        let text = toml::to_string(&p.save).unwrap();
        let save: Save = toml::from_str(&text).unwrap();
        let mut restored = Progression { save, ..default() };
        assert_eq!(restored.loadout(9).0[0], 5);
        assert!(restored.apply(9, Loadout::default()));
        assert_eq!(restored.points(9), 51);
        assert_eq!(restored.spent(9), 0);
    }

    #[test]
    fn casts_keep_zero_talent_behavior_and_modify_only_target_skills() {
        let run = Run::default();
        let mut stats = run.stats();
        let def = data::ability("corruption");
        let plain = cast(def, 3, &stats);
        let o = Owned {
            def,
            level: 3,
            cd: 0.0,
        };
        assert_eq!(plain.dot, def.dot * o.damage_mult() * stats.might);
        assert_eq!(plain.cooldown, o.cooldown(&stats));
        stats.talents.0[0] = 5;
        assert!((cast(def, 3, &stats).cooldown / plain.cooldown - 0.70).abs() < 0.0001);
        assert_eq!(cast(data::ability("fireball"), 1, &stats).dmg, 20.0);
    }

    #[test]
    fn credit_is_exactly_once_and_catches_multiple_thresholds() {
        let mut app = App::new();
        app.init_resource::<Run>()
            .init_resource::<Progression>()
            .add_systems(Update, credit);
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.class = data::CLASSES.iter().find(|c| c.id == WARLOCK).unwrap();
            run.kills = 992;
            run.boss_kills = 2;
        }
        app.update();
        let p = app.world().resource::<Progression>();
        assert_eq!(p.score(WARLOCK), 1_010);
        assert_eq!(p.points(WARLOCK), 4);
        assert_eq!(app.world().resource::<Run>().talent_points_gained, 4);
        app.update();
        assert_eq!(app.world().resource::<Progression>().score(WARLOCK), 1_010);
        *app.world_mut().resource_mut::<Run>() = Run::default();
        app.world_mut().resource_mut::<Run>().kills = 1;
        app.update();
        assert_eq!(app.world().resource::<Progression>().score(1), 1);
        assert_eq!(app.world().resource::<Progression>().score(WARLOCK), 1_010);
    }

    #[test]
    fn loadout_is_locked_for_the_run_and_resume_does_not_heal() {
        let mut app = App::new();
        app.init_resource::<Run>()
            .init_resource::<Progression>()
            .add_systems(Update, lock_loadout);
        {
            let mut p = app.world_mut().resource_mut::<Progression>();
            p.save.classes.insert(
                "9".into(),
                ClassProgress {
                    normal_kills: 89_000,
                    ..default()
                },
            );
            let mut t = Loadout::default();
            t.0[8] = 5;
            assert!(p.apply(WARLOCK, t));
        }
        app.world_mut().resource_mut::<Run>().class =
            data::CLASSES.iter().find(|c| c.id == WARLOCK).unwrap();
        app.update();
        let hp = app.world().resource::<Run>().hp;
        assert!(hp > super::super::BASE_HP);
        app.world_mut().resource_mut::<Run>().hp = 20.0;
        assert!(app
            .world_mut()
            .resource_mut::<Progression>()
            .apply(WARLOCK, Loadout::default()));
        app.update();
        assert_eq!(app.world().resource::<Run>().hp, 20.0);
        assert_eq!(app.world().resource::<Run>().talents.0[8], 5);
    }

    #[test]
    fn invalid_allocations_are_rejected_without_overflow() {
        assert!(!Loadout([255; NODE_COUNT]).valid(51));
        let mut t = Loadout::default();
        t.0[0] = 5;
        assert!(!t.learn(1, 5));
        assert!(t.learn(2, 51));
        t.0[0] = 0;
        assert!(!t.valid(51));
        for _ in 0..5 {
            assert!(t.learn(0, 51));
        }
        assert!(t.valid(51));
    }

    #[test]
    fn every_node_has_a_measurable_effect() {
        let run = Run::default();
        let base = run.stats();
        for i in 0..NODE_COUNT {
            let mut stats = base;
            stats.talents.0[i] = 1;
            modify_stats(&mut stats);
            let changed = match i {
                0 => {
                    cast(data::ability("corruption"), 1, &stats).cooldown
                        < cast(data::ability("corruption"), 1, &base).cooldown
                }
                1 => {
                    cast(data::ability("curse_of_agony"), 1, &stats).dot
                        > cast(data::ability("curse_of_agony"), 1, &base).dot
                }
                2 => {
                    cast(data::ability("corruption"), 1, &stats).duration
                        > cast(data::ability("corruption"), 1, &base).duration
                }
                3 => {
                    cast(data::ability("corruption"), 1, &stats).range
                        > cast(data::ability("corruption"), 1, &base).range
                }
                4 => stats.talents.leech() > 0.0,
                5 | 16 => {
                    cast(data::ability("shadow_bolt"), 1, &stats).dmg
                        > cast(data::ability("shadow_bolt"), 1, &base).dmg
                }
                6 | 7 => {
                    cast(data::ability("corruption"), 1, &stats).dot
                        > cast(data::ability("corruption"), 1, &base).dot
                }
                8 => stats.max_hp > base.max_hp,
                9 => stats.armor > base.armor,
                10 => stats.regen > base.regen,
                11 => stats.speed > base.speed,
                12 => {
                    cast(data::ability("death_coil"), 1, &stats).cooldown
                        < cast(data::ability("death_coil"), 1, &base).cooldown
                }
                13 => stats.talents.mobility_cooldown() < 1.0,
                14 | 15 => stats.talents.incoming() < 1.0,
                17 => {
                    cast(data::ability("immolate"), 1, &stats).dmg
                        > cast(data::ability("immolate"), 1, &base).dmg
                }
                18 => stats.crit > base.crit,
                19 => {
                    cast(data::ability("shadow_bolt"), 1, &stats).cooldown
                        < cast(data::ability("shadow_bolt"), 1, &base).cooldown
                }
                20 => {
                    cast(data::ability("rain_of_fire"), 1, &stats).radius
                        > cast(data::ability("rain_of_fire"), 1, &base).radius
                }
                21 => {
                    cast(data::ability("shadowburn"), 1, &stats).dmg
                        > cast(data::ability("shadowburn"), 1, &base).dmg
                }
                22 => {
                    cast(data::ability("rain_of_fire"), 1, &stats).dmg
                        > cast(data::ability("rain_of_fire"), 1, &base).dmg
                }
                23 => stats.talents.critical_multiplier() == 3.0,
                _ => unreachable!(),
            };
            assert!(
                changed,
                "talent {} must affect its advertised mechanic",
                NODES[i].id
            );
        }
    }

    #[test]
    fn leech_has_a_one_second_budget_and_does_not_revive() {
        let mut run = Run {
            hp: 1.0,
            ..default()
        };
        run.talents.0[4] = 5;
        run.talents.0[7] = 1;
        let max = run.stats().max_hp;
        periodic_heal(&mut run, 0.0, 10.0);
        periodic_heal(&mut run, 10_000.0, 0.0);
        assert!((run.hp - (1.0 + max * 0.04)).abs() < 0.001);
        let before = run.hp;
        for _ in 0..60 {
            periodic_heal(&mut run, 10_000.0, 1.0 / 60.0);
        }
        assert!((run.hp - (before + max * 0.04)).abs() < 0.001);
        run.dead_t = Some(0.0);
        run.hp = 0.0;
        periodic_heal(&mut run, 10_000.0, 1.0);
        assert_eq!(run.hp, 0.0);
    }

    #[test]
    fn all_twenty_four_bonuses_are_doubled_exactly() {
        let base = Run::default().stats();
        let expected = [
            0.94, 1.08, 1.08, 2.0, 0.004, 1.06, 1.06, 1.30, 1.06, 0.8, 0.30, 1.04, 0.92, 0.92,
            0.96, 0.80, 1.06, 1.06, 0.02, 0.96, 1.06, 1.08, 1.06, 3.0,
        ];
        for (i, &want) in expected.iter().enumerate() {
            let mut stats = base;
            stats.talents.0[i] = 1;
            modify_stats(&mut stats);
            let ratio = |key| {
                let def = data::ability(key);
                let a = cast(def, 1, &stats);
                let b = cast(def, 1, &base);
                (a, b)
            };
            let actual = match i {
                0 => {
                    let (a, b) = ratio("corruption");
                    a.cooldown / b.cooldown
                }
                1 => {
                    let (a, b) = ratio("curse_of_agony");
                    a.dot / b.dot
                }
                2 => {
                    let (a, b) = ratio("corruption");
                    a.duration / b.duration
                }
                3 => {
                    let (a, b) = ratio("corruption");
                    a.range - b.range
                }
                4 => stats.talents.leech(),
                5 | 16 => {
                    let (a, b) = ratio("shadow_bolt");
                    a.dmg / b.dmg
                }
                6 | 7 => {
                    let (a, b) = ratio("corruption");
                    a.dot / b.dot
                }
                8 => stats.max_hp / base.max_hp,
                9 => stats.armor - base.armor,
                10 => stats.regen - base.regen,
                11 => stats.speed / base.speed,
                12 => {
                    let (a, b) = ratio("death_coil");
                    a.cooldown / b.cooldown
                }
                13 => stats.talents.mobility_cooldown(),
                14 | 15 => stats.talents.incoming(),
                17 => {
                    let (a, b) = ratio("immolate");
                    a.dmg / b.dmg
                }
                18 => stats.crit - base.crit,
                19 => {
                    let (a, b) = ratio("shadow_bolt");
                    a.cooldown / b.cooldown
                }
                20 => {
                    let (a, b) = ratio("rain_of_fire");
                    a.radius / b.radius
                }
                21 => {
                    let (a, b) = ratio("shadowburn");
                    a.dmg / b.dmg
                }
                22 => {
                    let (a, b) = ratio("rain_of_fire");
                    a.dmg / b.dmg
                }
                23 => stats.talents.critical_multiplier(),
                _ => unreachable!(),
            };
            assert!(
                (actual - want).abs() < 0.0001,
                "{}: {actual} != {want}",
                NODES[i].id
            );
        }
    }
}
