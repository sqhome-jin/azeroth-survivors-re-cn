//! Survivors adaptations, not reproductions of raid encounters. Every impact uses a captured
//! ground position: no tracking missiles, forced movement, permanent control or contact procs.
//! Sources: https://warcraft.wiki.gg/wiki/Kel%27Thuzad_(Classic) (fissure/frost blast),
//! https://warcraft.wiki.gg/wiki/Hogger (Classic head butt),
//! https://www.icy-veins.com/wow-classic/lord-kazzak-guide-strategy-abilities-loot
//! (mark/volley). AQ/ZG/Naxx spell names are encounter references; generic beasts, bandits and
//! quest NPCs use species/region-inspired equivalents, not claims about their original kits.
//! All warnings (1.6–2.6 s), radii, 1.1–1.9 × scaled enemy damage, and avoidance shapes are ours.

use benilla_world::world_point::WorldPoint;
use bevy::prelude::*;

use super::data::EnemyDef;
use super::enemies::Enemy;
use super::hero::{self, Hero, HERO_RADIUS};
use super::localization::text;
use super::units::{flat, Fx, COLOR_HURT};
use super::{Run, RunEntity};

#[derive(Component)]
pub(super) struct Boss {
    pub ordinal: u8,
    cooldown: f32,
    casts: u32,
    pending: Option<Cast>,
}

pub(super) fn spawn(ordinal: u8) -> Boss {
    Boss {
        ordinal: ordinal.clamp(1, 5),
        cooldown: 3.0,
        casts: 0,
        pending: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Pattern {
    Target,
    Triple,
    Nova,
    Ring,
    Line,
    Fork,
    Cross,
    Breath,
}

#[derive(Clone, Copy)]
struct Skill {
    english: &'static str,
    chinese: &'static str,
    pattern: Pattern,
    radius: f32,
    warning: f32,
    cooldown: f32,
    damage: f32,
    spell: u32,
}

macro_rules! skill {
    ($en:literal, $zh:literal, $pattern:ident, $radius:literal, $warning:literal,
     $cooldown:literal, $damage:literal, $spell:literal) => {
        Skill {
            english: $en,
            chinese: $zh,
            pattern: Pattern::$pattern,
            radius: $radius,
            warning: $warning,
            cooldown: $cooldown,
            damage: $damage,
            spell: $spell,
        }
    };
}

const POUNCE: Skill = skill!("Pounce", "扑击", Target, 3.2, 1.6, 6.8, 1.25, 6785);
const LEAP: Skill = skill!(
    "Leaping Strike",
    "跳跃打击",
    Triple,
    2.4,
    1.8,
    8.0,
    1.1,
    19434
);
const STOMP: Skill = skill!("War Stomp", "战争践踏", Nova, 7.0, 1.9, 8.5, 1.4, 20549);
const RAKE: Skill = skill!("Raking Claws", "横扫利爪", Fork, 2.5, 1.6, 7.0, 1.2, 1822);
const CLEAVE: Skill = skill!("Cleave", "顺劈斩", Breath, 2.8, 1.8, 7.5, 1.4, 845);
const SHADOW: Skill = skill!("Shadow Bolt", "暗影箭", Target, 3.4, 1.7, 6.5, 1.25, 686);
const VOLLEY: Skill = skill!(
    "Shadow Bolt Volley",
    "暗影箭雨",
    Triple,
    3.0,
    2.0,
    9.0,
    1.35,
    686
);
const HELLFIRE: Skill = skill!("Hellfire", "地狱烈焰", Ring, 9.0, 2.1, 9.5, 1.5, 1949);
const BREATH: Skill = skill!("Flame Breath", "烈焰吐息", Breath, 3.3, 2.0, 8.5, 1.5, 133);
const MARK: Skill = skill!(
    "Mark of Kazzak",
    "卡扎克的印记",
    Target,
    5.2,
    2.6,
    10.0,
    1.8,
    18789
);
const SAND: Skill = skill!("Sand Trap", "沙尘陷阱", Triple, 2.6, 1.9, 8.0, 1.15, 26102);
const THUNDER: Skill = skill!("Thundercrash", "雷霆震击", Ring, 10.0, 2.3, 9.0, 1.6, 6343);
const ERUPTION: Skill = skill!(
    "Arcane Eruption",
    "奥术爆发",
    Nova,
    9.0,
    2.5,
    10.5,
    1.8,
    1449
);
const PLAGUE: Skill = skill!(
    "Creeping Plague",
    "蔓延瘟疫",
    Triple,
    3.5,
    2.2,
    9.0,
    1.45,
    28169
);
const CYCLONE: Skill = skill!("Cyclone", "旋风", Cross, 2.4, 2.2, 8.5, 1.4, 1680);
const POISON: Skill = skill!("Venom Spit", "毒液喷吐", Target, 4.1, 1.9, 7.5, 1.25, 2818);
const FIRE: Skill = skill!("Flamestrike", "烈焰风暴", Target, 4.6, 2.0, 8.0, 1.45, 2120);
const BLAST: Skill = skill!("Blast Wave", "冲击波", Ring, 8.0, 2.0, 8.0, 1.4, 11113);
const ROAR: Skill = skill!(
    "Terrifying Roar",
    "恐惧咆哮",
    Nova,
    8.0,
    2.2,
    9.0,
    1.5,
    5782
);
const WHIRL: Skill = skill!("Whirlwind", "旋风斩", Nova, 7.5, 2.1, 8.5, 1.6, 1680);
const PUNCH: Skill = skill!("Force Punch", "强力一击", Line, 2.5, 1.8, 7.0, 1.5, 78);
const HEX: Skill = skill!("Shadow Shock", "暗影震击", Cross, 2.7, 2.0, 8.0, 1.4, 17364);
const BLOOD: Skill = skill!(
    "Corrupted Blood",
    "堕落之血",
    Triple,
    3.5,
    2.3,
    9.0,
    1.6,
    24328
);
const SIPHON: Skill = skill!(
    "Blood Siphon",
    "血液虹吸",
    Ring,
    11.0,
    2.5,
    10.0,
    1.75,
    18220
);
const STRIKE: Skill = skill!("Head Butt", "头槌", Line, 2.2, 1.6, 6.8, 1.3, 6730);
const SHOT: Skill = skill!("Aimed Shot", "瞄准射击", Line, 1.8, 1.7, 7.0, 1.35, 19434);
const FROST: Skill = skill!("Frost Nova", "冰霜新星", Ring, 7.0, 1.9, 7.8, 1.3, 122);
const HOWL: Skill = skill!("Howling Echo", "嚎叫回响", Cross, 2.3, 2.0, 8.0, 1.3, 5484);
const IMPALE: Skill = skill!("Impale", "穿刺", Line, 2.5, 2.1, 8.5, 1.55, 28783);
const RAIN: Skill = skill!("Rain of Fire", "火焰之雨", Triple, 3.6, 2.1, 9.0, 1.5, 5740);
const HATEFUL: Skill = skill!(
    "Hateful Strike",
    "仇恨打击",
    Breath,
    3.1,
    2.2,
    8.0,
    1.8,
    28308
);
const POLARITY: Skill = skill!(
    "Polarity Shift",
    "极性转换",
    Cross,
    2.7,
    2.5,
    10.0,
    1.7,
    28089
);
const FISSURE: Skill = skill!(
    "Shadow Fissure",
    "暗影裂隙",
    Target,
    4.5,
    2.5,
    9.0,
    1.9,
    27810
);
const FROST_BLAST: Skill = skill!(
    "Frost Blast",
    "冰霜冲击",
    Triple,
    3.4,
    2.4,
    9.5,
    1.65,
    27808
);
const INFECTION: Skill = skill!(
    "Volatile Infection",
    "快速传染",
    Triple,
    3.8,
    2.3,
    9.0,
    1.5,
    24928
);
const LIGHTNING: Skill = skill!("Lightning Wave", "闪电波", Fork, 2.7, 2.1, 8.5, 1.5, 24819);
const ARCANE: Skill = skill!("Arcane Blast", "奥术冲击", Nova, 8.5, 2.1, 8.5, 1.55, 24857);

struct Profile {
    english: &'static str,
    chinese: &'static str,
    primary: Skill,
    secondary: Option<Skill>,
}

macro_rules! profiles {
    ($(($en:literal, $zh:literal, $primary:ident $(, $secondary:ident)?)),* $(,)?) => {
        static PROFILES: &[Profile] = &[$(Profile {
            english: $en,
            chinese: $zh,
            primary: $primary,
            secondary: profiles!(@optional $($secondary)?),
        }),*];
    };
    (@optional $secondary:ident) => { Some($secondary) };
    (@optional) => { None };
}

profiles![
    ("Echeyakee", "埃其亚基", POUNCE),
    ("Takk the Leaper", "跳跃者塔克", LEAP),
    ("Lakota'mani", "拉克塔曼尼", STOMP),
    ("Humar the Pridelord", "狮王休玛", RAKE, POUNCE),
    ("Hezrul Bloodmark", "赫兹鲁尔·血印", CLEAVE, STOMP),
    ("Akubar the Seer", "预言者阿库巴尔", SHADOW),
    ("Lord Azrethoc", "阿兹雷索克领主", HELLFIRE),
    ("Lord Hel'nurath", "赫尔努拉斯领主", CLEAVE, SHADOW),
    ("Teremus the Devourer", "吞噬者特雷姆斯", BREATH, BLAST),
    ("Lord Kazzak", "卡扎克", MARK, VOLLEY),
    ("Kurinnaxx", "库林纳克斯", SAND),
    ("General Rajaxx", "拉贾克斯将军", THUNDER, CLEAVE),
    ("Moam", "莫阿姆", ERUPTION),
    ("Buru the Gorger", "吞咽者布鲁", PLAGUE, POUNCE),
    ("Ossirian the Unscarred", "无疤者奥斯里安", CYCLONE, THUNDER),
    ("Ravasaur Matriarch", "暴掠龙女王", POISON, LEAP),
    ("Blazerunner", "火焰使者", BLAST),
    ("Baron Charr", "火焰男爵查尔", FIRE, HELLFIRE),
    ("Tyrant Devilsaur", "暴龙之王", CLEAVE, STOMP),
    ("King Mosh", "莫什大王", ROAR, STOMP),
    ("King Bangalash", "虎王邦加拉什", POUNCE, RAKE),
    ("Bloodlord Mandokir", "血领主曼多基尔", WHIRL, SHOT),
    ("High Priest Thekal", "高阶祭司塞卡尔", PUNCH, POUNCE),
    ("Jin'do the Hexxer", "妖术师金度", HEX, SHADOW),
    ("Hakkar", "哈卡", BLOOD, SIPHON),
    ("Mother Fang", "母蜘蛛", POISON),
    ("Narg the Taskmaster", "工头纳尔格", PUNCH),
    ("Fedfennel", "费德菲尼尔", SHADOW),
    ("Gruff Swiftbite", "格拉夫·疾齿", RAKE),
    ("Hogger", "霍格", STRIKE, CLEAVE),
    ("Geolord Mottle", "地卜师玛尔特", ERUPTION),
    ("Razormane Battleguard", "钢鬃战士", CLEAVE),
    ("Burning Blade Neophyte", "火刃新兵", FIRE),
    ("Death Flayer", "死亡毒蝎", POISON),
    ("Burning Blade Cultist", "火刃祭司", HELLFIRE, SHADOW),
    ("Threggil", "瑟雷基尔", SHADOW),
    ("Uruson", "乌鲁森", CLEAVE),
    ("Duskstalker", "暮色巡游者", POUNCE),
    ("Gnarlpine Augur", "瘤背秘法师", HEX),
    ("Oakenscowl", "奥肯斯古尔", STOMP, ROAR),
    ("Edan the Howler", "嚎叫者埃丹", HOWL),
    ("Timber", "雪爪灰狼", RAKE),
    ("Great Father Arctikus", "霜鬃长老阿克提卡斯", FROST),
    ("Vagash", "瓦加什", STOMP),
    ("Bjarn", "冰爪熊比亚恩", RAKE, FROST),
    ("Prairie Stalker", "草原捕食者", POUNCE),
    ("Mazzranache", "马兹拉纳其", LEAP),
    ("Snagglespear", "断矛", SHOT),
    ("Swoop", "猛鹫", LEAP),
    ("Ghost Howl", "幽灵狼", HOWL, FROST),
    ("Tormented Spirit", "痛苦的灵魂", SHADOW),
    ("Bayne", "贝恩", RAKE),
    ("Ressan the Needler", "针刺者雷桑", FORK_BITE),
    ("Fellicent's Shade", "菲利森特的阴影", HEX),
    ("Gorefang", "血牙狼", RAKE, HOWL),
    ("Brainwashed Noble", "被洗脑的贵族", PUNCH),
    ("Marisa du'Paige", "玛里莎·杜派格", FIRE, FROST),
    ("Sergeant Brashclaw", "布拉克利中士", SHOT),
    ("Vultros", "秃鹫", LEAP),
    ("Klaven Mortwake", "克拉文·摩特维克", SHADOW, POISON),
    ("Lupos", "鲁伯斯", SHADOW),
    ("Naraxis", "纳拉克西斯", POISON),
    ("Nightbane Tainted One", "夜行邪齿狼人", HEX),
    ("Fenros", "芬罗斯", RAKE),
    ("Nefaru", "尼法鲁", HOWL, SHADOW),
    ("Blackrock Hunter", "黑石猎人", SHOT),
    ("Singe", "辛格", BREATH),
    ("Tharil'zun", "萨瑞祖恩", CLEAVE),
    ("Rohh the Silent", "沉默的罗恩", POISON, SHOT),
    ("Kazon", "卡松", WHIRL, CLEAVE),
    ("Prince Xavalis", "萨瓦里克王子", SHADOW, HELLFIRE),
    ("Taerar", "泰拉尔", ARCANE, BREATH),
    ("Lethon", "莱索恩", VOLLEY, BREATH),
    ("Emeriss", "艾莫莉丝", INFECTION, BREATH),
    ("Ysondre", "伊森德雷", LIGHTNING, BREATH),
    ("Anub'Rekhan", "阿努布雷坎", IMPALE, PLAGUE),
    ("Grand Widow Faerlina", "黑女巫法琳娜", RAIN, POISON),
    ("Patchwerk", "帕奇维克", HATEFUL, STOMP),
    ("Thaddius", "塔迪乌斯", POLARITY, LIGHTNING),
    ("Kel'Thuzad", "克尔苏加德", FISSURE, FROST_BLAST),
];

const FORK_BITE: Skill = skill!("Needle Dive", "针刺俯冲", Fork, 2.0, 1.7, 7.2, 1.2, 17253);

fn profile(english: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|p| p.english == english)
}

pub(super) fn name(def: &EnemyDef) -> &'static str {
    profile(def.name).map_or(def.name, |p| text(p.english, p.chinese))
}

#[derive(Clone, Copy)]
struct Zone {
    center: Vec3,
    outer: f32,
    inner: f32,
}

impl Zone {
    fn contains(self, point: Vec3) -> bool {
        let distance = flat(point - self.center).length();
        distance <= self.outer + HERO_RADIUS
            && (self.inner == 0.0 || distance + HERO_RADIUS >= self.inner)
    }
}

struct Cast {
    skill: Skill,
    remaining: f32,
    zones: Vec<Zone>,
    warnings: Vec<Entity>,
}

pub(super) fn cast_status(boss: &Boss) -> Option<(&'static str, f32, f32)> {
    boss.pending.as_ref().map(|cast| {
        (
            text(cast.skill.english, cast.skill.chinese),
            cast.remaining,
            cast.skill.warning,
        )
    })
}

#[derive(Component)]
pub(super) struct Warning {
    owner: Entity,
}

#[derive(Default)]
pub(super) struct WarningArt {
    ring: Option<Handle<Mesh>>,
    disc: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

fn zones(skill: Skill, caster: Vec3, target: Vec3) -> Vec<Zone> {
    let forward = flat(target - caster).try_normalize().unwrap_or(Vec3::X);
    let side = Vec3::new(-forward.z, 0.0, forward.x);
    let zone = |center, radius| Zone {
        center,
        outer: radius,
        inner: 0.0,
    };
    match skill.pattern {
        Pattern::Target => vec![zone(target, skill.radius)],
        Pattern::Triple => [-1.0, 0.0, 1.0]
            .into_iter()
            .map(|i| zone(target + side * i * (skill.radius * 2.0 + 1.5), skill.radius))
            .collect(),
        Pattern::Nova => vec![zone(caster, skill.radius)],
        Pattern::Ring => vec![Zone {
            center: caster,
            outer: skill.radius,
            inner: skill.radius * 0.55,
        }],
        Pattern::Line => (1..=6)
            .map(|i| zone(caster + forward * i as f32 * 3.0, skill.radius))
            .collect(),
        Pattern::Fork => [-0.5_f32, 0.0, 0.5]
            .into_iter()
            .flat_map(|angle| {
                let dir = forward * angle.cos() + side * angle.sin();
                (1..=4).map(move |i| zone(caster + dir * i as f32 * 3.0, skill.radius))
            })
            .collect(),
        Pattern::Cross => [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
            .into_iter()
            .flat_map(|dir| (0..=3).map(move |i| zone(target + dir * i as f32 * 3.5, skill.radius)))
            .collect(),
        Pattern::Breath => (1..=4)
            .map(|i| {
                zone(
                    caster + forward * i as f32 * 3.0,
                    skill.radius * i as f32 * 0.55,
                )
            })
            .collect(),
    }
}

fn cadence(skill: Skill, ordinal: u8, enraged: bool) -> f32 {
    (skill.cooldown * (1.0 - 0.045 * ordinal.saturating_sub(1) as f32)
        / if enraged { 1.15 } else { 1.0 })
    .max(4.5)
}

fn clear(commands: &mut Commands, boss: &mut Boss) {
    if let Some(cast) = boss.pending.take() {
        for warning in cast.warnings {
            commands.entity(warning).try_despawn();
        }
    }
}

/// One pending cast per boss, no catch-up casts after stalls, and interruptible warnings.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn cast_skills(
    mut commands: Commands,
    time: Res<Time>,
    mut run: ResMut<Run>,
    hero: Query<(Entity, &Transform), (With<Hero>, Without<Enemy>)>,
    mut bosses: Query<(Entity, &Transform, &Enemy, &mut Boss), Without<Hero>>,
    warnings: Query<(Entity, &Warning)>,
    world: WorldPoint,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut art: Local<WarningArt>,
    mut fx: Fx,
) {
    // Reap orphaned warnings even if a different damage system removed their caster.
    for (entity, warning) in &warnings {
        if bosses.get(warning.owner).is_err() {
            commands.entity(entity).try_despawn();
        }
    }
    let Ok((hero_e, hero_tf)) = hero.single() else {
        for (_, _, _, mut boss) in &mut bosses {
            clear(&mut commands, &mut boss);
        }
        return;
    };
    let dt = time.delta_secs();
    for (entity, transform, enemy, mut boss) in &mut bosses {
        if enemy.dead
            || enemy.hp <= 0.0
            || run.dead_t.is_some()
            || run.final_boss_defeated
            || run.hp <= 0.0
        {
            clear(&mut commands, &mut boss);
            continue;
        }
        if enemy.stun_t > 0.0 || enemy.root_t > 0.0 || enemy.fear_t > 0.0 {
            clear(&mut commands, &mut boss);
            boss.cooldown = boss.cooldown.max(1.0);
            continue;
        }
        if dt <= 0.0 {
            continue;
        }
        let enraged = enemy.hp < enemy.max_hp * 0.3;
        if let Some(cast) = &mut boss.pending {
            cast.remaining -= dt;
            if cast.remaining > 0.0 {
                continue;
            }
            let hit = cast.zones.iter().any(|z| z.contains(hero_tf.translation));
            for zone in &cast.zones {
                fx.cast(entity, cast.skill.spell, &[], Some(zone.center));
            }
            if hit {
                let weaken = if enemy.weaken_t > 0.0 {
                    enemy.weaken
                } else {
                    1.0
                };
                let damage = hero::hurt_by_spell(
                    &mut run,
                    enemy.damage * cast.skill.damage * weaken * if enraged { 1.15 } else { 1.0 },
                );
                if damage > 0.0 {
                    fx.impact(hero_e, cast.skill.spell);
                    fx.number(hero_e, damage, false, COLOR_HURT);
                }
            }
            clear(&mut commands, &mut boss);
            continue;
        }
        boss.cooldown = (boss.cooldown - dt).max(0.0);
        if boss.cooldown > 0.0 {
            continue;
        }
        let Some(profile) = profile(enemy.def.name) else {
            continue;
        };
        let skill = if boss.casts % 2 == 1 {
            profile.secondary.unwrap_or(profile.primary)
        } else {
            profile.primary
        };
        boss.casts = boss.casts.wrapping_add(1);
        boss.cooldown = cadence(skill, boss.ordinal, enraged);
        let mut zones = zones(skill, transform.translation, hero_tf.translation);
        let ring = art
            .ring
            .get_or_insert_with(|| meshes.add(Annulus::new(0.94, 1.0)))
            .clone();
        let disc = art
            .disc
            .get_or_insert_with(|| meshes.add(Circle::new(1.0)))
            .clone();
        let material = art
            .material
            .get_or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgba(1.0, 0.12, 0.02, 0.72),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })
            })
            .clone();
        let mut entities = Vec::new();
        for zone in &mut zones {
            zone.center.y = world
                .terrain_height_under(zone.center)
                .unwrap_or(zone.center.y)
                + 0.15;
            // Thin bright boundaries, plus an inset ring for annular safe centres. The warning
            // has no dependency on a spell's optional model/texture chain.
            for radius in [zone.outer, zone.inner].into_iter().filter(|r| *r > 0.0) {
                entities.push(
                    commands
                        .spawn((
                            Mesh3d(ring.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_translation(zone.center)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::splat(radius)),
                            Warning { owner: entity },
                            RunEntity,
                        ))
                        .id(),
                );
            }
            if zone.inner == 0.0 {
                entities.push(
                    commands
                        .spawn((
                            Mesh3d(disc.clone()),
                            MeshMaterial3d(material.clone()),
                            Transform::from_translation(zone.center - Vec3::Y * 0.02)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::splat(zone.outer * 0.16)),
                            Warning { owner: entity },
                            RunEntity,
                        ))
                        .id(),
                );
            }
        }
        let attack = text(skill.english, skill.chinese);
        run.banner = Some((
            if super::localization::is_zh_cn() {
                format!(
                    "{} · {}：{:.1}秒后命中红圈！",
                    name(enemy.def),
                    attack,
                    skill.warning
                )
            } else {
                format!(
                    "{} · {}: leave red marks ({:.1}s)!",
                    name(enemy.def),
                    attack,
                    skill.warning
                )
            },
            skill.warning,
        ));
        boss.pending = Some(Cast {
            skill,
            remaining: skill.warning,
            zones,
            warnings: entities,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_text::CombatTextSpawn;
    use crate::creature_anim::{CastEvent, PlaySeq, SpellGoTargets, SwingMessage};
    use std::time::Duration;

    fn encounter() -> (App, Entity, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Run>()
            .init_resource::<PlaySeq>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_message::<CastEvent>()
            .add_message::<SpellGoTargets>()
            .add_message::<SwingMessage>()
            .add_message::<CombatTextSpawn>()
            .add_systems(Update, cast_skills);
        benilla_world::world_point::init_world_point_resources(app.world_mut());
        let hero = app
            .world_mut()
            .spawn((
                Hero { vy: 0.0, air: 0.0 },
                Transform::from_xyz(12.0, 3.0, 0.0),
            ))
            .id();
        let mut boss = spawn(5);
        boss.cooldown = 0.0;
        let def = super::super::data::roster(15)
            .bosses
            .iter()
            .find(|def| def.name == "Kel'Thuzad")
            .unwrap();
        let enemy = Enemy {
            def,
            hp: 1000.0,
            max_hp: 1000.0,
            speed: 0.0,
            damage: 40.0,
            radius: 1.0,
            xp: 0,
            boss: true,
            attack_cd: 0.0,
            slow: 1.0,
            slow_t: 0.0,
            root_t: 0.0,
            stun_t: 0.0,
            fear_t: 0.0,
            weaken: 0.5,
            weaken_t: 10.0,
            dots: Vec::new(),
            warlock: default(),
            class_effects: default(),
            dead: false,
        };
        let boss = app
            .world_mut()
            .spawn((boss, enemy, Transform::from_xyz(0.0, 3.0, 0.0)))
            .id();
        (app, hero, boss)
    }

    fn step(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
        app.world_mut().run_schedule(Update);
    }

    fn warning_count(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query::<&Warning>().iter(world).count()
    }

    #[test]
    fn ecs_captured_cast_hits_once_or_is_dodged_without_tracking() {
        for dodge in [false, true] {
            let (mut app, hero, boss) = encounter();
            let mut expected = Run::default();
            let expected_damage = hero::hurt_by_spell(&mut expected, 40.0 * FISSURE.damage * 0.5);
            let before = app.world().resource::<Run>().hp;
            step(&mut app, 0.1);
            assert_eq!(warning_count(&mut app), 2);
            assert_eq!(app.world().resource::<Run>().hp, before);
            let captured = app
                .world()
                .get::<Boss>(boss)
                .unwrap()
                .pending
                .as_ref()
                .unwrap()
                .zones[0]
                .center;
            assert!((captured - Vec3::new(12.0, 3.15, 0.0)).length() < 0.001);
            if dodge {
                app.world_mut()
                    .get_mut::<Transform>(hero)
                    .unwrap()
                    .translation
                    .z = 15.0;
            }
            step(&mut app, FISSURE.warning - 0.1);
            assert_eq!(app.world().resource::<Run>().hp, before);
            step(&mut app, 0.2);
            let after = app.world().resource::<Run>().hp;
            assert!((after - (before - if dodge { 0.0 } else { expected_damage })).abs() < 0.001);
            assert_eq!(warning_count(&mut app), 0);
            assert!(app.world().get::<Boss>(boss).unwrap().pending.is_none());
            step(&mut app, 0.1);
            assert_eq!(app.world().resource::<Run>().hp, after);
        }
    }

    #[test]
    fn ecs_interrupts_terminal_states_and_removed_casters_cancel_hazards() {
        for scenario in 0..9 {
            let (mut app, _, boss) = encounter();
            step(&mut app, 0.1);
            assert_eq!(warning_count(&mut app), 2);
            match scenario {
                0 => app.world_mut().get_mut::<Enemy>(boss).unwrap().stun_t = 3.0,
                1 => app.world_mut().get_mut::<Enemy>(boss).unwrap().dead = true,
                2 => app.world_mut().resource_mut::<Run>().dead_t = Some(0.0),
                3 => app.world_mut().resource_mut::<Run>().hp = 0.0,
                4 => app.world_mut().resource_mut::<Run>().final_boss_defeated = true,
                5 => {
                    app.world_mut().despawn(boss);
                }
                6 => app.world_mut().get_mut::<Enemy>(boss).unwrap().root_t = 3.0,
                7 => app.world_mut().get_mut::<Enemy>(boss).unwrap().fear_t = 3.0,
                _ => app.world_mut().get_mut::<Enemy>(boss).unwrap().hp = 0.0,
            }
            let before = app.world().resource::<Run>().hp;
            step(&mut app, 3.0);
            assert_eq!(
                app.world().resource::<Run>().hp,
                before,
                "scenario {scenario}"
            );
            assert_eq!(warning_count(&mut app), 0, "scenario {scenario}");
            assert!(app.world().resource::<Messages<CastEvent>>().is_empty());
            assert!(app
                .world()
                .resource::<Messages<CombatTextSpawn>>()
                .is_empty());
            if let Some(boss) = app.world().get::<Boss>(boss) {
                assert!(boss.pending.is_none());
            }
        }
        for dead in [false, true] {
            let (mut app, _, boss) = encounter();
            if dead {
                app.world_mut().resource_mut::<Run>().dead_t = Some(0.0);
            } else {
                app.world_mut().resource_mut::<Run>().hp = 0.0;
            }
            step(&mut app, 10.0);
            assert_eq!(warning_count(&mut app), 0);
            assert!(app.world().get::<Boss>(boss).unwrap().pending.is_none());
            assert_eq!(app.world().get::<Boss>(boss).unwrap().casts, 0);
        }
    }

    #[test]
    fn ecs_warning_meshes_are_horizontal_double_sided_and_above_ground() {
        let (mut app, _, _) = encounter();
        step(&mut app, 0.1);
        let world = app.world_mut();
        let mut query = world.query_filtered::<
            (&Transform, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
            With<Warning>,
        >();
        for (transform, mesh, material) in query.iter(world) {
            assert!((transform.rotation * Vec3::Z - Vec3::Y).length() < 0.001);
            assert!(transform.translation.y >= 3.12);
            let mesh = world.resource::<Assets<Mesh>>().get(&mesh.0).unwrap();
            let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
            else {
                panic!("warning mesh lacks positions")
            };
            for position in positions {
                let point = transform.transform_point(Vec3::from_array(*position));
                assert!(point.y >= 3.12 && point.y <= 3.16);
            }
            let material = world
                .resource::<Assets<StandardMaterial>>()
                .get(&material.0)
                .unwrap();
            assert_eq!(material.alpha_mode, AlphaMode::Blend);
            assert!((material.base_color.alpha() - 0.72).abs() < 0.001);
            assert!(material.unlit && material.cull_mode.is_none());
        }
        assert_eq!(query.iter(world).count(), 2);
    }

    #[test]
    fn every_regional_boss_has_an_explicit_profile() {
        assert_eq!(PROFILES.len(), 80);
        for (i, profile) in PROFILES.iter().enumerate() {
            assert!(!profile.chinese.is_empty());
            assert!(!PROFILES[..i].iter().any(|p| p.english == profile.english));
            assert!(profile.primary.warning >= 1.6);
        }
        for map in 0..super::super::data::MAPS.len() {
            let roster = super::super::data::roster(map);
            assert_eq!(roster.bosses.len(), 5);
            for boss in &roster.bosses {
                assert!(
                    profile(boss.name).is_some(),
                    "unprofiled boss: {}",
                    boss.name
                );
            }
        }
    }

    #[test]
    fn captured_targets_do_not_follow_and_ring_centres_are_safe() {
        let caster = Vec3::ZERO;
        let target = Vec3::new(12.0, 0.0, 0.0);
        let fissure = zones(FISSURE, caster, target);
        assert!(fissure[0].contains(target));
        assert!(!fissure[0].contains(target + Vec3::Z * 10.0));
        let ring = zones(HELLFIRE, caster, target)[0];
        assert!(!ring.contains(caster));
        assert!(ring.contains(Vec3::X * 7.0));
        assert!(!ring.contains(Vec3::X * 12.0));
    }

    #[test]
    fn all_patterns_are_bounded_and_have_dodge_space() {
        let mut patterns = Vec::new();
        for p in PROFILES {
            for skill in std::iter::once(p.primary).chain(p.secondary) {
                if !patterns.contains(&skill.pattern) {
                    patterns.push(skill.pattern);
                }
                let zones = zones(skill, Vec3::ZERO, Vec3::X * 10.0);
                assert!(!zones.is_empty() && zones.len() <= 16);
                assert!(zones.iter().all(|z| !z.contains(Vec3::Z * 40.0)));
                assert!(cadence(skill, 5, true) >= 4.5);
            }
        }
        assert_eq!(patterns.len(), 8);
        assert_eq!(spawn(0).ordinal, 1);
        assert_eq!(spawn(9).ordinal, 5);
    }
}
