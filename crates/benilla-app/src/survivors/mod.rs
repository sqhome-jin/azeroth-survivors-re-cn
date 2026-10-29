//! Warcraft Survivors: a top-down survival mode on the benilla world, started by
//! [`crate::run_survivors`]. There is no server: the hero, the enemies and the spells are local
//! entities dressed by the same attach, animation and spell-visual systems a streamed unit gets,
//! on a battleground picked at the menu ([`data::MAPS`]), with the player controller and FrameXML
//! off ([`crate::run_mode::CaptureMode`]).

mod abilities;
mod auto;
mod bosses;
mod buffs;
mod cards;
mod class_mechanics;
mod data;
mod enemies;
mod fill;
mod gear;
mod gear_ui;
mod grade;
mod growth;
mod hero;
mod hud;
mod localization;
mod menu;
mod menu_layout;
mod minimap;
mod mobility;
mod preview;
mod skill_descriptions;
mod skill_effects;
mod spell_expansion;
mod talent_catalog;
mod talents;
mod target_ui;
mod units;
mod warlock;

use benilla_world::schedule::WorldStage;
use bevy::prelude::*;

use data::{AbilityDef, ClassDef, PassiveDef, Stat};

/// Set by [`crate::run_survivors`] before the app builds. wasm32 has no process environment, so
/// the flag lives here; `WOW_SURVIVORS` (`?survivors=1` / `__wenilla_env.survivors` on the web)
/// still turns it on from outside.
pub(crate) static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn active() -> bool {
    ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
        || crate::webenv::var("WOW_SURVIVORS").is_some()
}

pub(crate) struct SurvivorsPlugin;

impl Plugin for SurvivorsPlugin {
    fn build(&self, app: &mut App) {
        // Present before `OnEnter(InWorld)`, where the UI lifecycle decides not to load FrameXML.
        app.insert_resource(crate::run_mode::CaptureMode)
            .add_plugins(grade::plugin)
            .add_plugins(talents::plugin)
            .add_plugins(gear::plugin)
            .add_plugins(gear_ui::plugin)
            .add_plugins(skill_effects::plugin)
            .add_plugins(target_ui::plugin)
            .init_state::<Phase>()
            .init_resource::<Rng>()
            .init_resource::<Selection>()
            .init_resource::<menu::MenuState>()
            .init_resource::<Run>()
            .init_resource::<hero::CamRig>()
            .init_resource::<hero::AimPoint>()
            .init_resource::<abilities::PendingHits>()
            .init_resource::<enemies::Spawner>()
            .init_resource::<hud::Offer>()
            .init_resource::<cards::PendingPick>()
            .add_message::<abilities::Hit>()
            .add_message::<enemies::Loot>()
            .add_plugins(bevy::ui_render::UiMaterialPlugin::<hud::SweepUiMaterial>::default())
            .add_plugins(bevy::ui_render::UiMaterialPlugin::<fill::FillMaterial>::default())
            .add_plugins(preview::plugin)
            .add_systems(Startup, (hud::setup_hud, preview::setup))
            .add_systems(Update, hero::daylight)
            .add_systems(
                Update,
                (
                    hero::drop_stray_buttons,
                    hero::aim_cursor,
                    mobility::cast_mobility,
                    hero::drive_hero,
                    hero::place_camera,
                    preview::assign_layer,
                    preview::frame,
                    preview::fill_view,
                )
                    .chain()
                    .in_set(WorldStage::Input),
            )
            .add_systems(
                Update,
                (
                    abilities::update_talent_environment,
                    enemies::spawn_enemies,
                    enemies::move_enemies,
                    bosses::cast_skills,
                    abilities::cast_abilities,
                    abilities::tick_areas,
                    abilities::apply_hits,
                    enemies::tick_marks,
                    enemies::drop_loot,
                    enemies::collect_gems,
                    hero::hero_vitals,
                )
                    .chain()
                    .after(WorldStage::Input)
                    .run_if(in_state(Phase::Playing))
                    // Entering a battleground raises its loading screen: the run waits behind it.
                    .run_if(|l: Res<crate::loading_screen::LoadingScreen>| !l.covering()),
            )
            .add_systems(
                Update,
                (enemies::reap_dead, enemies::still_gems).after(WorldStage::Input),
            )
            .add_systems(
                Update,
                (
                    hero::follow_map,
                    hud::menu_preview,
                    hud::buttons,
                    hud::keys,
                    hud::refresh_menu,
                    menu::restore_pick,
                    menu::build_menu,
                    menu::menu_clicks,
                    menu::scroll_zones,
                    menu::refresh_menu_ui.before(crate::glue::GlueVisuals),
                    cards::deal_cards,
                    cards::choose_card,
                    cards::animate_cards,
                    hud::refresh_hud,
                    hud::refresh_results,
                    hud::fill_icons,
                    hud::update_cooldown_sweeps,
                    hud::refresh_mobility,
                    hud::show_panels,
                )
                    .chain()
                    .after(WorldStage::Input),
            )
            .add_systems(
                Update,
                (
                    hud::apply_ui_scale,
                    hud::fill_art,
                    hud::xp_hover,
                    hud::icon_tooltip,
                    hud::hero_plate,
                    minimap::map_buttons,
                    minimap::zone_text,
                    minimap::compose,
                )
                    .chain()
                    .after(hud::show_panels),
            )
            .add_systems(OnEnter(Phase::Playing), unpause)
            .add_systems(OnEnter(Phase::LevelUp), (pause, hud::roll_offer))
            .add_systems(OnEnter(Phase::Paused), pause)
            .add_systems(OnEnter(Phase::GameOver), unpause)
            .add_systems(OnEnter(Phase::Menu), (unpause, reset_run));
        if let Some(auto) = auto::from_env() {
            app.insert_resource(auto)
                .add_systems(
                    Update,
                    auto::drive.after(hud::keys).before(hud::refresh_menu),
                )
                .add_systems(Update, auto::plan.before(WorldStage::Input));
        }
    }
}

/// The screen the mode is on.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) enum Phase {
    #[default]
    Menu,
    Playing,
    LevelUp,
    Paused,
    GameOver,
}

/// Everything spawned for a run, despawned when the next begins.
#[derive(Component)]
pub(super) struct RunEntity;

/// The class, look and battleground picked at the menu; the battleground holds for the run.
#[derive(Resource, Default)]
pub(super) struct Selection {
    pub class: Option<usize>,
    /// `(display, race, sex)`, one per race and sex the class's trainers come in.
    pub looks: Vec<(u32, u8, u8)>,
    pub look: Option<usize>,
    /// An index into [`data::MAPS`].
    pub map: usize,
    /// The battleground the world is standing in: the last one entered (or the saved pick at
    /// boot). Picking another at the menu changes only `map`, so the menu hero stays put and no
    /// loading screen rises until Enter World.
    pub stage: usize,
    /// Bumped on every change, so the menu rebuilds once.
    pub version: u32,
}

impl Selection {
    pub fn class_def(&self) -> Option<&'static ClassDef> {
        self.class.map(|i| &data::CLASSES[i])
    }

    pub fn map_def(&self) -> &'static data::MapDef {
        &data::MAPS[self.map.min(data::MAPS.len() - 1)]
    }

    pub fn stage_def(&self) -> &'static data::MapDef {
        &data::MAPS[self.stage.min(data::MAPS.len() - 1)]
    }
}

/// An ability the hero has, with its own cooldown.
pub(super) struct Owned {
    pub def: &'static AbilityDef,
    pub level: u32,
    pub growth: f32,
    pub cd: f32,
}

impl Owned {
    pub fn damage_mult(&self) -> f32 {
        growth::damage(self.growth)
    }
    pub fn cooldown(&self, stats: &Stats) -> f32 {
        let base = self.def.cooldown * growth::cooldown_multiplier(self.growth);
        (base * (1.0 - stats.haste).max(0.35)).max(0.2)
    }
    pub fn count(&self) -> u32 {
        self.def.count + growth::bonus_targets(self.growth)
    }
    pub fn radius(&self, stats: &Stats) -> f32 {
        self.def.radius * growth::radius_multiplier(self.growth) * stats.area
    }
    pub fn duration(&self) -> f32 {
        self.def.duration * growth::duration_multiplier(self.growth)
    }
}

/// The hero's derived numbers, from the base and the passives.
#[derive(Clone, Copy)]
pub(super) struct Stats {
    class: u8,
    talents: talents::Loadout,
    gear: gear::Bonuses,
    pub might: f32,
    pub max_hp: f32,
    pub speed: f32,
    pub regen: f32,
    pub haste: f32,
    pub area: f32,
    pub magnet: f32,
    pub growth: f32,
    pub armor: f32,
    pub crit: f32,
    pub luck: f32,
    pub luck_raw: f32,
    pub targets: u32,
    pub echo: f32,
    pub echo_damage: f32,
    pub duration: f32,
    pub range: f32,
    pub crit_damage: f32,
    pub leech: f32,
    pub shield_power: f32,
}

pub(super) const BASE_HP: f32 = 120.0;
pub(super) const BASE_SPEED: f32 = 7.0;
pub(super) const BASE_MAGNET: f32 = 4.5;

/// One run's state.
#[derive(Resource)]
pub(super) struct Run {
    pub class: &'static ClassDef,
    pub race: u8,
    pub elapsed: f32,
    pub kills: u32,
    pub boss_kills: u32,
    pub credited_kills: u32,
    pub credited_bosses: u32,
    pub talent_points_gained: u8,
    talents: talents::Loadout,
    pub talent_leech_budget: f32,
    gear: gear::Bonuses,
    effects: skill_effects::Runtime,
    warlock: warlock::Runtime,
    class_effects: class_mechanics::Runtime,
    expansion: spell_expansion::Runtime,
    pub talents_locked: bool,
    pub level: u32,
    pub xp: f32,
    pub hp: f32,
    pub abilities: Vec<Owned>,
    pub passives: Vec<(&'static PassiveDef, u32)>,
    pub passive_growth: std::collections::BTreeMap<&'static str, f32>,
    pub shield: f32,
    pub shield_t: f32,
    pub immune_t: f32,
    pub thorns: f32,
    pub thorns_t: f32,
    pub thorns_spell: u32,
    thorns_source: skill_effects::Source,
    pub heal_rate: f32,
    pub heal_t: f32,
    /// The class mobility's cooldown and running effect.
    mobile: mobility::MobilityState,
    pub pending_levels: u32,
    pub bosses: u32,
    pub rings: u32,
    pub won: bool,
    pub final_boss_defeated: bool,
    pub final_reward: Option<String>,
    pub reward_error: Option<String>,
    /// Seconds since the hero fell, while the death plays out.
    pub dead_t: Option<f32>,
    pub next_guid: u64,
    pub damage_done: f64,
    /// Bumped whenever abilities or passives change, so the HUD rebuilds the icon rows once.
    pub version: u32,
    /// A centre-screen announcement and its seconds left.
    pub banner: Option<(String, f32)>,
}

impl Default for Run {
    fn default() -> Self {
        Self {
            class: &data::CLASSES[0],
            race: 0,
            elapsed: 0.0,
            kills: 0,
            boss_kills: 0,
            credited_kills: 0,
            credited_bosses: 0,
            talent_points_gained: 0,
            talents: talents::Loadout::default(),
            talent_leech_budget: 0.0,
            gear: gear::Bonuses::default(),
            effects: skill_effects::Runtime::default(),
            warlock: warlock::Runtime::default(),
            class_effects: class_mechanics::Runtime::default(),
            expansion: spell_expansion::Runtime::default(),
            talents_locked: false,
            level: 1,
            xp: 0.0,
            hp: BASE_HP,
            abilities: Vec::new(),
            passives: Vec::new(),
            passive_growth: std::collections::BTreeMap::new(),
            shield: 0.0,
            shield_t: 0.0,
            immune_t: 0.0,
            thorns: 0.0,
            thorns_t: 0.0,
            thorns_spell: 0,
            thorns_source: skill_effects::Source::default(),
            heal_rate: 0.0,
            heal_t: 0.0,
            mobile: mobility::MobilityState::default(),
            pending_levels: 0,
            bosses: 0,
            rings: 0,
            won: false,
            final_boss_defeated: false,
            final_reward: None,
            reward_error: None,
            dead_t: None,
            next_guid: 1,
            damage_done: 0.0,
            version: 0,
            banner: None,
        }
    }
}

impl Stats {
    pub fn crit_multiplier(&self) -> f32 {
        (if self.class == 9 {
            self.talents.critical_multiplier()
        } else {
            2.0
        }) + self.gear.crit_damage()
            + self.crit_damage
    }
}

impl Run {
    pub fn passive_growth(&self, passive: &PassiveDef) -> f32 {
        self.passive_growth
            .get(passive.key)
            .copied()
            .unwrap_or_else(|| {
                self.passives
                    .iter()
                    .find(|(p, _)| p.key == passive.key)
                    .map_or(0.0, |(_, rank)| *rank as f32)
            })
    }

    pub fn incoming(&self) -> f32 {
        (if self.class.id == 9 {
            self.talents.incoming()
        } else {
            talent_catalog::incoming(self.class.id, self.talents)
        }) * class_mechanics::incoming(self)
            * spell_expansion::incoming(self)
            * self.gear.incoming()
            * (1.0 - self.effects.bonus(gear::EnhKind::BuffDefense, self.elapsed))
    }

    pub fn mobility_cooldown(&self) -> f32 {
        (if self.class.id == 9 {
            self.talents.mobility_cooldown()
        } else {
            talent_catalog::mobility_cooldown(self.class.id, self.talents)
        }) * self.gear.mobility_cooldown()
    }

    pub fn leech(&self) -> f32 {
        (if self.class.id == 9 {
            self.talents.leech()
        } else {
            0.0
        }) + self.gear.leech()
            + self.stats().leech
    }

    pub fn stats(&self) -> Stats {
        self.derive_stats(None)
    }

    pub fn stats_with_passive_growth(&self, passive: &PassiveDef, additional: f32) -> Stats {
        self.derive_stats(Some((passive, additional)))
    }

    fn derive_stats(&self, extra: Option<(&PassiveDef, f32)>) -> Stats {
        let mut s = Stats {
            class: self.class.id,
            talents: self.talents,
            gear: self.gear,
            might: 1.0,
            max_hp: BASE_HP,
            speed: BASE_SPEED,
            regen: 0.0,
            haste: 0.0,
            area: 1.0,
            magnet: BASE_MAGNET,
            growth: 1.0,
            armor: 0.0,
            crit: 0.05,
            luck: 0.0,
            luck_raw: 0.0,
            targets: 0,
            echo: 0.0,
            echo_damage: 0.0,
            duration: 0.0,
            range: 0.0,
            crit_damage: 0.0,
            leech: 0.0,
            shield_power: 1.0,
        };
        // Tanky classes start tougher, cloth starts quicker.
        match self.class.id {
            1 | 2 => s.max_hp += 40.0,
            7 | 11 | 3 | 4 => s.max_hp += 15.0,
            _ => s.speed += 0.5,
        }
        let class_hp = s.max_hp;
        let mut target_growth = 0.0;
        let new_passive = extra
            .filter(|(p, _)| !self.passives.iter().any(|(owned, _)| owned.key == p.key))
            .map(|(p, _)| p);
        for p in self.passives.iter().map(|(p, _)| *p).chain(new_passive) {
            let acquired = self.passive_growth(p)
                + extra
                    .filter(|(p2, _)| p2.key == p.key)
                    .map_or(0.0, |(_, amount)| amount);
            let v = p.per_level * acquired;
            match p.stat {
                Stat::Might => s.might += v,
                Stat::MaxHp => s.max_hp += class_hp * v,
                Stat::Speed => {
                    s.speed += BASE_SPEED * growth::bounded(v, 1.0);
                    s.regen += v;
                }
                Stat::Regen => s.regen += v,
                Stat::Haste => {
                    s.haste += growth::bounded(v, 0.65);
                    s.might += v * 0.2;
                }
                Stat::Area => {
                    s.area += growth::bounded(v, 1.5);
                    s.might += v * 0.2;
                }
                Stat::Magnet => {
                    s.magnet += BASE_MAGNET * growth::bounded(v, 4.0);
                    s.growth += v * 0.1;
                }
                Stat::Growth => s.growth += v,
                Stat::Armor => s.armor += v,
                Stat::Crit => {
                    s.crit += growth::bounded(v, 0.9);
                    s.crit_damage += v * 0.5;
                }
                Stat::Luck => {
                    s.luck += v;
                    s.might += v * 0.001;
                }
                Stat::Targets => {
                    target_growth += acquired * p.per_level / 3.0;
                    s.might += acquired * p.per_level * 0.03;
                }
                Stat::Echo => {
                    s.echo += growth::bounded(v, 0.5);
                    s.echo_damage += v;
                }
                Stat::Duration => {
                    s.duration += growth::bounded(v, 1.0);
                    s.might += v * 0.2;
                }
                Stat::Range => {
                    s.range += growth::bounded(v, 1.0);
                    s.might += v * 0.2;
                }
                Stat::CritDamage => s.crit_damage += v,
                Stat::Leech => {
                    s.leech += growth::bounded(v, 0.2);
                    s.regen += v * 10.0;
                }
                Stat::Shield => s.shield_power += v,
            }
        }
        s.targets = target_growth.floor().min(6.0) as u32;
        talents::modify_stats(&mut s);
        gear::modify_stats(&mut s, &self.gear);
        s.might *= 1.0 + self.effects.bonus(gear::EnhKind::BuffDamage, self.elapsed);
        s.haste +=
            self.effects.bonus(gear::EnhKind::BuffHaste, self.elapsed) + warlock::haste(self);
        s.crit = (s.crit + self.effects.bonus(gear::EnhKind::BuffCrit, self.elapsed)).min(1.0);
        s.speed *= 1.0 + self.effects.bonus(gear::EnhKind::BuffSpeed, self.elapsed);
        class_mechanics::modify_stats(self, &mut s);
        spell_expansion::modify_stats(self, &mut s);
        s.luck_raw = s.luck;
        s.luck = growth::effective_luck(s.luck);
        s
    }

    pub fn guid(&mut self) -> u64 {
        self.next_guid += 1;
        self.next_guid
    }
}

/// xorshift64*: the mode's dice, seeded from the clock.
#[derive(Resource)]
pub(super) struct Rng(u64);

impl Default for Rng {
    fn default() -> Self {
        let seed = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self(seed | 1)
    }
}

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    /// Uniform in `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.f32()
    }
    pub fn index(&mut self, len: usize) -> usize {
        (self.next_u64() % len.max(1) as u64) as usize
    }
}

fn pause(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn unpause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

/// Back at the menu: everything the last run spawned goes, and the run starts over.
fn reset_run(
    mut commands: Commands,
    spawned: Query<Entity, With<RunEntity>>,
    mut run: ResMut<Run>,
    mut pending: ResMut<abilities::PendingHits>,
    mut spawner: ResMut<enemies::Spawner>,
    mut selection: ResMut<Selection>,
) {
    for e in &spawned {
        commands.entity(e).despawn();
    }
    // Guids stay unique across runs: caches keyed by guid may outlive a despawn by a frame.
    let next_guid = run.next_guid;
    *run = Run {
        next_guid,
        ..Run::default()
    };
    pending.0.clear();
    *spawner = enemies::Spawner::default();
    // The preview hero is respawned for the look still picked.
    selection.version += 1;
}
