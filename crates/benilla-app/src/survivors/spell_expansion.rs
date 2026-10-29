//! Vanilla identities with explicitly mode-specific automatic utility and weapon procs.
use bevy::prelude::*;

use super::{
    Rng, Run, Stats,
    abilities::Hit,
    data::{self, Kind},
    enemies::Enemy,
    skill_effects::Source,
    talents::Cast,
};

#[derive(Default)]
pub(super) struct Runtime {
    active: Vec<Active>,
    debuffs: Vec<Debuff>,
    observed_hp: Option<f32>,
    reactive_until: f32,
    proc_ready: f32,
    frost_ready: f32,
    swing_ready: f32,
}

struct Active {
    key: &'static str,
    until: f32,
    power: f32,
    growth: f32,
}

struct Debuff {
    target: Entity,
    key: &'static str,
    until: f32,
}

pub(super) struct BuffEntry {
    pub key: &'static str,
    pub spell: u32,
    pub name: &'static str,
    pub description: String,
    pub remaining: f32,
    pub stacks: u32,
}

const NAMES: &[(&str, &str)] = &[
    ("bloodthirst", "嗜血"),
    ("overpower", "压制"),
    ("rend", "撕裂"),
    ("slam", "猛击"),
    ("shield_slam", "盾牌猛击"),
    ("shield_bash", "盾击"),
    ("disarm", "缴械"),
    ("shield_wall", "盾墙"),
    ("last_stand", "破釜沉舟"),
    ("seal_of_righteousness", "正义圣印"),
    ("seal_of_command", "命令圣印"),
    ("seal_of_wisdom", "智慧圣印"),
    ("seal_of_light", "光明圣印"),
    ("flash_of_light", "圣光闪现"),
    ("holy_light", "圣光术"),
    ("devotion_aura", "虔诚光环"),
    ("retribution_aura", "惩罚光环"),
    ("raptor_strike", "猛禽一击"),
    ("mongoose_bite", "猫鼬撕咬"),
    ("wing_clip", "摔绊"),
    ("immolation_trap", "献祭陷阱"),
    ("frost_trap", "冰霜陷阱"),
    ("wyvern_sting", "翼龙钉刺"),
    ("rapid_fire", "急速射击"),
    ("backstab", "背刺"),
    ("ambush", "伏击"),
    ("garrote", "锁喉"),
    ("deadly_poison", "致命药膏"),
    ("expose_armor", "破甲"),
    ("evasion", "闪避"),
    ("mind_flay", "精神鞭笞"),
    ("vampiric_embrace", "吸血鬼的拥抱"),
    ("devouring_plague", "噬灵瘟疫"),
    ("flash_heal", "快速治疗"),
    ("prayer_of_healing", "治疗祷言"),
    ("inner_fire", "心灵之火"),
    ("shadowguard", "暗影守卫"),
    ("windfury_weapon", "风怒武器"),
    ("rockbiter_weapon", "石化武器"),
    ("frostbrand_weapon", "冰封武器"),
    ("healing_wave", "治疗波"),
    ("lesser_healing_wave", "次级治疗波"),
    ("healing_stream_totem", "治疗之泉图腾"),
    ("earthbind_totem", "地缚图腾"),
    ("stoneclaw_totem", "石爪图腾"),
    ("polymorph", "变形术"),
    ("ice_barrier", "寒冰护体"),
    ("ice_block", "寒冰屏障"),
    ("frost_armor", "霜甲术"),
    ("mage_armor", "法师护甲"),
    ("mana_shield", "法力护盾"),
    ("dampen_magic", "魔法抑制"),
    ("arcane_power", "奥术强化"),
    ("presence_of_mind", "气定神闲"),
    ("combustion", "燃烧"),
    ("cold_snap", "急速冷却"),
    ("siphon_life", "生命虹吸"),
    ("drain_life", "吸取生命"),
    ("drain_soul", "吸取灵魂"),
    ("soul_fire", "灵魂之火"),
    ("curse_of_elements", "元素诅咒"),
    ("curse_of_weakness", "虚弱诅咒"),
    ("curse_of_tongues", "语言诅咒"),
    ("curse_of_doom", "厄运诅咒"),
    ("fear", "恐惧术"),
    ("claw", "爪击"),
    ("rake", "斜掠"),
    ("rip", "割裂"),
    ("swipe", "挥击"),
    ("ferocious_bite", "凶猛撕咬"),
    ("regrowth", "愈合"),
    ("healing_touch", "治疗之触"),
    ("barkskin", "树皮术"),
    ("fortune", "幸运祝福（自创）"),
    ("multistrike", "横扫延伸（自创）"),
    ("echo", "奥术回响（自创）"),
    ("persistence", "绵延魔法（自创）"),
    ("far_sight", "远见（自创）"),
    ("ruin", "毁灭之力（自创）"),
    ("soul_leech", "灵魂汲取（自创）"),
    ("warding", "护盾专精（自创）"),
];

pub(super) fn name(key: &str) -> Option<&'static str> {
    NAMES.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

pub(super) fn description(key: &str) -> Option<&'static str> {
    Some(match key {
        "overpower" | "mongoose_bite" => "受到伤害后3秒内自动近战反击；未触发时保留冷却。",
        "shield_bash" => "只攻击施法型敌人，造成近战伤害并打断（短暂昏迷）。",
        "execute" | "drain_soul" => {
            "只选择生命不高于30%的敌人；吸取灵魂持续伤害，在低生命时处决非首领。"
        }
        "shield_wall" => "生命低于65%时自动施放，短时间内受到的伤害降低60%。",
        "last_stand" => "生命低于65%时获得临时吸收护盾；不永久增加生命上限。",
        "seal_of_righteousness" => "持续期间，武器命中额外造成神圣伤害；每次攻击最多触发一次。",
        "seal_of_command" => "持续期间，武器命中有35%几率追加一次强力神圣打击。",
        "seal_of_wisdom" => "持续期间提高技能冷却恢复效率；本模式无独立法力系统。",
        "seal_of_light" => "持续期间武器命中恢复生命；触发有独立冷却，不会递归。",
        "deadly_poison" => "持续期间武器命中附加持续毒伤；不在涂毒时直接攻击敌人。",
        "windfury_weapon" => "持续期间武器命中有20%几率追加两次风怒伤害，不会递归。",
        "rockbiter_weapon" => "持续期间武器近战伤害提高25%。",
        "frostbrand_weapon" => "持续期间武器命中追加冰霜伤害并减速。",
        "rapid_fire" => "短时间提高远程技能施放频率。",
        "devotion_aura" | "inner_fire" => "持续期间提高护甲；只对自身施放，不向敌人发送增益法术。",
        "frost_armor" => "持续期间提高护甲，并自动减速身边的近战敌人。",
        "mage_armor" => "持续期间恢复生命并降低受到的伤害；本模式无独立法力系统。",
        "dampen_magic" => "持续期间受到的伤害降低20%，不提供直接伤害。",
        "arcane_power" => "短时间法术伤害提高30%，不影响武器攻击。",
        "presence_of_mind" => "下一次实际命中的直接法术获得50%伤害加成并回响一次；随后消耗。",
        "combustion" => "持续期间火焰法术的暴击率提高30%。",
        "cold_snap" => "有冰霜技能正在冷却时自动重置其冷却，不重置自身。",
        "barkskin" | "evasion" => "低生命时自动开启防御；树皮减伤30%，闪避减伤50%。",
        "ice_block" => "生命低于65%且近处有敌人时自动进入短暂无敌。",
        "ice_barrier" | "mana_shield" => "生命低于85%且护盾耗尽时自动吸收伤害，不向敌人发送护盾。",
        "flash_of_light"
        | "holy_light"
        | "flash_heal"
        | "prayer_of_healing"
        | "healing_wave"
        | "lesser_healing_wave"
        | "healing_touch" => "生命低于85%时自动对自身施放即时治疗；没有队友时不会选择敌人为目标。",
        "regrowth" => "生命低于85%时先立即恢复部分生命，再持续恢复。",
        "healing_stream_totem" => "放置友方图腾，在范围内每秒治疗自己；不造成伤害或召唤爆炸。",
        "earthbind_totem" => "放置友方图腾，周期性减速附近敌人；不造成召唤爆炸。",
        "stoneclaw_totem" => "放置友方图腾，周期性削弱附近敌人伤害；本模式以减伤模拟牵制。",
        "vampiric_embrace" => "标记敌人，之后对该敌人造成的法术伤害回复15%生命。",
        "curse_of_elements" => "标记敌人，使其受到的法术伤害提高20%。",
        "curse_of_weakness" => "削弱目标10秒，使其造成的伤害降低35%。",
        "curse_of_tongues" => "仅诅咒施法型敌人，延缓其下一次攻击并降低伤害。",
        "curse_of_doom" => "诅咒8秒后造成一次强力伤害；施放时不立即造成伤害。",
        "expose_armor" => "近战破甲，使目标随后受到的武器伤害提高20%。",
        "drain_life" | "siphon_life" | "devouring_plague" => {
            "持续伤害的每次实际伤害均按技能比例恢复生命；不会从过量伤害或尸体吸血。"
        }
        "mind_flay" => "持续3秒鞭笞目标并减速，每秒造成伤害。",
        "polymorph" => "控制非首领敌人；首领只受到缩短的控制，不造成直接伤害。",
        "claw" | "rake" | "rip" | "swipe" | "ferocious_bite" => {
            "人形英雄使用野性主题近战技巧（不会变身，也不需要连击点）；撕裂类造成持续伤害。"
        }
        "backstab" => "近战打击；受控制的目标受到额外50%伤害，本模式无需背后站位。",
        "ambush" => "优先攻击尚未受伤的敌人，造成强力近战伤害；不需要潜行系统。",
        _ => return None,
    })
}

fn active(run: &Run, key: &str) -> bool {
    run.expansion
        .active
        .iter()
        .any(|b| b.key == key && b.until > run.elapsed)
}

pub(super) fn buffs(run: &Run) -> Vec<BuffEntry> {
    run.expansion
        .active
        .iter()
        .filter(|b| b.until > run.elapsed)
        .map(|b| BuffEntry {
            key: b.key,
            spell: data::ability(b.key).spell,
            name: name(b.key).unwrap_or(b.key),
            description: buff_description(run, b),
            remaining: b.until - run.elapsed,
            stacks: 1,
        })
        .collect()
}

fn buff_description(run: &Run, buff: &Active) -> String {
    let power = buff.power;
    let mut text = match buff.key {
        "frost_armor" => "护甲 +3。每1秒减速4码内敌人：移动速度降低40%，持续2秒；首领减速最多25%。".into(),
        "devotion_aura" => "自身护甲 +5；不向敌人施放增益。".into(),
        "inner_fire" => "自身护甲 +4；不向敌人施放增益。".into(),
        "mage_armor" => "生命恢复 +1/秒，受到的伤害降低10%；本模式不使用法力。".into(),
        "shield_wall" => "受到的伤害降低60%。生命低于65%、未处于无敌且30码内有敌人时自动开启。".into(),
        "barkskin" => "受到的伤害降低30%。生命低于65%、未处于无敌且30码内有敌人时自动开启。".into(),
        "evasion" => "受到的伤害降低50%（以减伤模拟闪避）。生命低于65%、未处于无敌且30码内有敌人时自动开启。".into(),
        "dampen_magic" => "受到的伤害降低20%，不造成直接伤害。".into(),
        "arcane_power" => "非近战技能的直接伤害和持续伤害提高30%，不增强普通武器挥击。".into(),
        "presence_of_mind" => "下一次成功施放的直接法术伤害提高50%，并追加一次50%伤害的回响（回响伤害加成另计）；随后消耗，不触发递归回响。".into(),
        "combustion" => "火球术、火焰冲击、炎爆术、烈焰风暴和冲击波暴击率 +30个百分点，总暴击率最多100%。".into(),
        "rapid_fire" => "奥术射击、多重射击、瞄准射击、毒蛇钉刺和乱射的施放频率提高40%（冷却除以1.4，最低0.2秒）。".into(),
        "rockbiter_weapon" => "武器近战技能伤害提高25%；普通附魔挥击伤害提高25%。".into(),
        "seal_of_wisdom" => "急速 +15个百分点；技能冷却的急速缩减最多65%。本模式不使用法力。".into(),
        "seal_of_righteousness" => format!("武器命中追加 {power:.1} 点神圣伤害。"),
        "seal_of_command" => format!("武器命中有35%几率追加 {power:.1} 点神圣伤害。"),
        "seal_of_light" => format!("武器命中恢复最多 {power:.1} 点生命；与吸血共享每秒最大生命4%的预算，不超过生命上限。"),
        "windfury_weapon" => format!("武器命中有20%几率追加两击合计 {:.1} 点风怒伤害（作为一次次级命中结算）。", power * 2.0),
        "frostbrand_weapon" => format!("武器命中追加 {power:.1} 点冰霜伤害，并降低移动速度50%，持续3秒；首领减速最多25%。"),
        "deadly_poison" => format!("武器命中附加每秒 {power:.1} 点毒伤，持续6秒；每1秒跳伤。"),
        _ => description(buff.key).unwrap_or("").into(),
    };
    if buff.key.starts_with("seal_of_")
        || buff.key.ends_with("_weapon")
        || buff.key == "deadly_poison"
    {
        text.push_str(" 5码内自动武器挥击每1.5秒一次，基础伤害8×当前伤害倍率；圣印同组、武器附魔同组不能同时生效。");
        if matches!(
            buff.key,
            "seal_of_righteousness"
                | "seal_of_command"
                | "seal_of_light"
                | "windfury_weapon"
                | "frostbrand_weapon"
                | "deadly_poison"
        ) {
            text.push_str(" 命中触发共享0.5秒内置冷却，只触发一次，不从持续或次级伤害递归。");
        }
    }
    let might = (buff.growth - 1.0).max(0.0) * 0.01;
    if might > 0.0 {
        text.push_str(&format!(
            " 本次增益的品质成长另提供伤害倍率 +{:.2}个百分点。",
            might * 100.0
        ));
    }
    if run.immune_t > 0.0 {
        text.push_str(&format!(
            " 当前另有无敌 {:.1}秒、吸收护盾 {:.1}；两者不由本增益提供。",
            run.immune_t, run.shield
        ));
    }
    text
}

pub(super) fn observe(run: &mut Run) {
    let hp = run.hp;
    if run.expansion.observed_hp.is_some_and(|old| hp < old) {
        run.expansion.reactive_until = run.elapsed + 3.0;
    }
    run.expansion.observed_hp = Some(hp);
    let now = run.elapsed;
    run.expansion.active.retain(|b| b.until > now);
    run.expansion.debuffs.retain(|b| b.until > now);
}

pub(super) fn reactive(run: &Run, key: &str) -> bool {
    !matches!(key, "overpower" | "mongoose_bite") || run.expansion.reactive_until > run.elapsed
}

pub(super) fn frost_pulse(run: &mut Run) -> bool {
    if !active(run, "frost_armor") || run.elapsed < run.expansion.frost_ready {
        return false;
    }
    run.expansion.frost_ready = run.elapsed + 1.0;
    true
}

/// Imbues and seals supply a modest automatic weapon swing in this otherwise spell-only mode.
pub(super) fn weapon_swing(run: &mut Run) -> bool {
    if run.expansion.swing_ready > run.elapsed
        || !run.expansion.active.iter().any(|b| {
            b.until > run.elapsed
                && (b.key.starts_with("seal_of_")
                    || b.key.ends_with("_weapon")
                    || b.key == "deadly_poison")
        })
    {
        return false;
    }
    run.expansion.swing_ready = run.elapsed + 1.5;
    true
}

fn weapon(source: Source) -> bool {
    source
        .ability
        .is_none_or(|k| k == "judgement" || data::ability(k).kind == Kind::Melee)
}

pub(super) fn supports(def: &data::AbilityDef, kind: super::gear::EnhKind) -> Option<bool> {
    use super::gear::EnhKind as E;
    if matches!(def.key, "cold_snap") {
        return Some(kind == E::Cooldown);
    }
    if matches!(
        def.key,
        "shield_wall"
            | "barkskin"
            | "evasion"
            | "devotion_aura"
            | "inner_fire"
            | "frost_armor"
            | "mage_armor"
            | "dampen_magic"
            | "arcane_power"
            | "presence_of_mind"
            | "combustion"
            | "rapid_fire"
            | "seal_of_righteousness"
            | "seal_of_command"
            | "seal_of_light"
            | "seal_of_wisdom"
            | "windfury_weapon"
            | "rockbiter_weapon"
            | "frostbrand_weapon"
            | "deadly_poison"
    ) {
        return Some(
            matches!(kind, E::Cooldown | E::Duration) || (kind == E::Damage && def.damage > 0.0),
        );
    }
    if def.key == "healing_stream_totem" {
        return Some(matches!(
            kind,
            E::Damage | E::Cooldown | E::Radius | E::Duration | E::Frequency
        ));
    }
    if matches!(def.key, "earthbind_totem" | "stoneclaw_totem") {
        return Some(matches!(
            kind,
            E::Cooldown | E::Radius | E::Duration | E::Frequency | E::Control
        ));
    }
    if matches!(
        def.key,
        "curse_of_elements" | "vampiric_embrace" | "expose_armor"
    ) {
        return Some(
            matches!(kind, E::Cooldown | E::Duration | E::Range | E::Count)
                || (kind == E::Damage && def.damage > 0.0),
        );
    }
    None
}

pub(super) fn caster(enemy: &Enemy) -> bool {
    let name = enemy.def.name.to_ascii_lowercase();
    [
        "mage", "shaman", "caster", "warlock", "witch", "priest", "seer", "sorcer", "necroman",
        "acolyte", "mystic", "healer", "conjur", "evoker", "druid",
    ]
    .iter()
    .any(|s| name.contains(s))
}

pub(super) fn incoming(run: &Run) -> f32 {
    let mut m = 1.0;
    for (key, factor) in [
        ("shield_wall", 0.4),
        ("barkskin", 0.7),
        ("evasion", 0.5),
        ("dampen_magic", 0.8),
        ("mage_armor", 0.9),
    ] {
        if active(run, key) {
            m *= factor;
        }
    }
    m
}

pub(super) fn modify_stats(run: &Run, stats: &mut Stats) {
    // Utility growth remains useful even once its duration or cooldown reaches a safety limit.
    for buff in run
        .expansion
        .active
        .iter()
        .filter(|b| b.until > run.elapsed)
    {
        stats.might += (buff.growth - 1.0).max(0.0) * 0.01;
    }
    if active(run, "devotion_aura") {
        stats.armor += 5.0;
    }
    if active(run, "inner_fire") {
        stats.armor += 4.0;
    }
    if active(run, "frost_armor") {
        stats.armor += 3.0;
    }
    if active(run, "mage_armor") {
        stats.regen += 1.0;
    }
    if active(run, "seal_of_wisdom") {
        stats.haste += 0.15;
    }
}

fn fire(key: &str) -> bool {
    matches!(
        key,
        "fireball" | "fire_blast" | "pyroblast" | "flamestrike" | "blast_wave"
    )
}

pub(super) fn frost(key: &str) -> bool {
    matches!(
        key,
        "frostbolt" | "frost_nova" | "blizzard" | "cone_of_cold" | "ice_barrier" | "ice_block"
    )
}

pub(super) fn modify_cast(run: &Run, key: &str, cast: &mut Cast, stats: &mut Stats) {
    let def = data::ability(key);
    if def.kind == Kind::Shield
        && !matches!(
            key,
            "power_word_shield" | "ice_barrier" | "mana_shield" | "last_stand"
        )
    {
        // Shield is the self-targeted carrier kind; weapon imbues do not absorb damage.
        cast.dmg /= stats.shield_power.max(0.001);
    }
    let melee = def.kind == Kind::Melee;
    if active(run, "arcane_power") && !melee {
        cast.dmg *= 1.3;
        cast.dot *= 1.3;
    }
    if active(run, "rockbiter_weapon") && melee {
        cast.dmg *= 1.25;
        cast.dot *= 1.25;
    }
    if active(run, "rapid_fire")
        && matches!(
            key,
            "arcane_shot" | "multi_shot" | "aimed_shot" | "serpent_sting" | "volley"
        )
    {
        cast.cooldown = (cast.cooldown / 1.4).max(0.2);
    }
    if active(run, "combustion") && fire(key) {
        stats.crit = (stats.crit + 0.3).min(1.0);
    }
    if active(run, "presence_of_mind") && !melee && cast.dmg > 0.0 && def.kind != Kind::Shield {
        cast.dmg *= 1.5;
        cast.modifiers.echo = 1.0;
    }
}

/// Returns None for the ordinary damage pipeline, Some(false) when a utility must wait.
pub(super) fn utility(run: &mut Run, key: &'static str, cast: &Cast, nearby: bool) -> Option<bool> {
    if key == "cold_snap" {
        let ready = run.abilities.iter().any(|o| frost(o.def.key) && o.cd > 0.2);
        if ready {
            for o in &mut run.abilities {
                if frost(o.def.key) {
                    o.cd = 0.0;
                }
            }
            let growth = run
                .abilities
                .iter()
                .find(|o| o.def.key == key)
                .map_or(1.0, |o| o.growth);
            run.shield += (growth - 1.0).max(0.0);
            if run.shield > 0.0 {
                run.shield_t = run.shield_t.max(3.0);
            }
        }
        return Some(ready);
    }
    if !matches!(
        key,
        "shield_wall"
            | "barkskin"
            | "evasion"
            | "devotion_aura"
            | "inner_fire"
            | "frost_armor"
            | "mage_armor"
            | "dampen_magic"
            | "arcane_power"
            | "presence_of_mind"
            | "combustion"
            | "rapid_fire"
            | "seal_of_righteousness"
            | "seal_of_command"
            | "seal_of_light"
            | "seal_of_wisdom"
            | "windfury_weapon"
            | "rockbiter_weapon"
            | "frostbrand_weapon"
            | "deadly_poison"
    ) {
        return None;
    }
    if active(run, key)
        || !nearby
        || (matches!(key, "shield_wall" | "barkskin" | "evasion")
            && (run.hp >= run.stats().max_hp * 0.65 || run.immune_t > 0.0))
    {
        return Some(false);
    }
    // Seals and weapon imbues are choices, not an unlimited stack of simultaneous procs.
    let family = |k: &str| {
        if k.starts_with("seal_of_") {
            1
        } else if k.ends_with("_weapon") {
            2
        } else {
            0
        }
    };
    let group = family(key);
    if group != 0
        && run
            .expansion
            .active
            .iter()
            .any(|b| family(b.key) == group && b.until > run.elapsed)
    {
        return Some(false);
    }
    run.expansion.active.retain(|b| b.key != key);
    let growth = run
        .abilities
        .iter()
        .find(|o| o.def.key == key)
        .map_or(1.0, |o| o.growth);
    run.expansion.active.push(Active {
        key,
        until: run.elapsed + cast.duration.max(0.2),
        power: cast.dmg,
        growth,
    });
    Some(true)
}

pub(super) fn on_cast(run: &mut Run, key: &str, direct_spell: bool) {
    if key != "presence_of_mind" && direct_spell {
        run.expansion.active.retain(|b| b.key != "presence_of_mind");
    }
    if matches!(key, "overpower" | "mongoose_bite") {
        run.expansion.reactive_until = 0.0;
    }
}

pub(super) fn damage_multiplier(run: &Run, source: Source, target: Entity) -> f32 {
    let melee = weapon(source);
    let mut m = if source.ability.is_none() && active(run, "rockbiter_weapon") {
        1.25
    } else {
        1.0
    };
    for d in &run.expansion.debuffs {
        if d.target == target
            && d.until > run.elapsed
            && ((d.key == "curse_of_elements" && !melee) || (d.key == "expose_armor" && melee))
        {
            m *= 1.2;
        }
    }
    m
}

fn heal(run: &mut Run, value: f32) {
    if run.dead_t.is_none() && run.hp > 0.0 {
        let value = value.max(0.0).min(run.talent_leech_budget.max(0.0));
        run.talent_leech_budget -= value;
        run.hp = (run.hp + value).min(run.stats().max_hp);
    }
}

pub(super) fn on_hit(
    run: &mut Run,
    source: Source,
    target: Entity,
    enemy: &mut Enemy,
    effective: f32,
    duration: f32,
    rng: &mut Rng,
) -> Option<Hit> {
    let key = source.ability.unwrap_or("");
    if duration > 0.0
        && !source.secondary
        && !source.periodic
        && matches!(
            key,
            "curse_of_elements" | "expose_armor" | "vampiric_embrace"
        )
    {
        run.expansion
            .debuffs
            .retain(|d| !(d.target == target && d.key == key));
        run.expansion.debuffs.push(Debuff {
            target,
            key,
            until: run.elapsed + duration,
        });
    }
    if duration > 0.0 && key == "curse_of_tongues" && !source.secondary {
        enemy.attack_cd += 2.0;
    }
    if effective <= 0.0 || source.secondary {
        return None;
    }
    if source.periodic {
        let frac = match source.ability.map(|key| data::ability(key).effect) {
            Some(data::Effect::Drain(f)) => f,
            _ => 0.0,
        };
        heal(run, effective * frac);
    }
    if !weapon(source)
        && run
            .expansion
            .debuffs
            .iter()
            .any(|d| d.target == target && d.key == "vampiric_embrace" && d.until > run.elapsed)
    {
        heal(run, effective * 0.15);
    }
    if source.periodic
        || !weapon(source)
        || enemy.hp <= 0.0
        || run.expansion.proc_ready > run.elapsed
    {
        return None;
    }
    let buff = run.expansion.active.iter().find(|b| {
        b.until > run.elapsed
            && matches!(
                b.key,
                "seal_of_righteousness"
                    | "seal_of_command"
                    | "seal_of_light"
                    | "windfury_weapon"
                    | "frostbrand_weapon"
                    | "deadly_poison"
            )
    })?;
    let (key, power) = (buff.key, buff.power);
    run.expansion.proc_ready = run.elapsed + 0.5;
    if key == "seal_of_light" {
        heal(run, power);
        return None;
    }
    if key == "seal_of_command" && rng.f32() >= 0.35 {
        return None;
    }
    if key == "windfury_weapon" && rng.f32() >= 0.2 {
        return None;
    }
    let mut hit = Hit::direct(
        target,
        if key == "windfury_weapon" {
            power * 2.0
        } else {
            power
        },
        false,
    );
    hit.color = super::units::COLOR_SPELL;
    hit.source = Source {
        ability: Some(key),
        secondary: true,
        ..source
    };
    if key == "frostbrand_weapon" {
        hit.effect = data::Effect::Slow(0.5, 3.0);
    }
    if key == "deadly_poison" {
        hit.damage = 0.0;
        hit.dot = power;
        hit.dot_dur = 6.0;
    }
    Some(hit)
}

#[cfg(test)]
mod tests {
    use super::super::Owned;
    use super::*;
    use std::collections::BTreeSet;

    fn own(run: &mut Run, key: &'static str, growth: f32, cd: f32) {
        run.abilities.push(Owned {
            def: data::ability(key),
            level: 1,
            growth,
            cd,
        });
    }

    fn cast(run: &Run, key: &'static str) -> Cast {
        let growth = run
            .abilities
            .iter()
            .find(|o| o.def.key == key)
            .map_or(1.0, |o| o.growth);
        super::super::talents::cast_with_growth(data::ability(key), 1, growth, &run.stats())
    }

    #[test]
    fn every_vanilla_class_pool_is_exactly_doubled_without_alternate_ranks() {
        let mut all = BTreeSet::new();
        let mut spells = BTreeSet::new();
        for (class, original) in data::CLASSES.iter().zip([9, 8, 7, 6, 7, 8, 11, 9, 8]) {
            assert_eq!(class.pool.len(), original * 2, "{}", class.name);
            assert!(class.pool.contains(&class.start));
            for key in class.pool {
                assert!(all.insert(*key), "duplicate ability {key}");
                let def = data::ability(key);
                assert!(
                    spells.insert(def.spell),
                    "alternate rank or duplicate spell {key}"
                );
                assert!(def.spell > 0 && def.spell < 31000, "{key}");
            }
        }
        assert_eq!(all.len(), 146);
        assert_eq!(data::ABILITIES.len(), 146);
        assert_eq!(data::PASSIVES.len(), 18);
        let extra: Vec<_> = data::ABILITIES
            .iter()
            .filter(|d| name(d.key).is_some())
            .collect();
        assert_eq!(extra.len(), 73);
        let chinese: BTreeSet<_> = extra.iter().map(|d| name(d.key).unwrap()).collect();
        assert_eq!(chinese.len(), 73);
        for def in extra {
            if def.kind == Kind::Shield && def.damage == 0.0 {
                assert!(
                    description(def.key).is_some(),
                    "unexplained utility {}",
                    def.key
                );
            }
        }
    }

    #[test]
    fn utility_is_timed_and_defensive_automation_waits_for_danger() {
        let mut run = Run::default();
        own(&mut run, "shield_wall", 1.0, 0.0);
        let c = cast(&run, "shield_wall");
        assert_eq!(utility(&mut run, "shield_wall", &c, true), Some(false));
        run.hp = 30.0;
        assert_eq!(utility(&mut run, "shield_wall", &c, false), Some(false));
        assert_eq!(utility(&mut run, "shield_wall", &c, true), Some(true));
        assert!((incoming(&run) - 0.4).abs() < 0.0001);
        assert_eq!(buffs(&run)[0].spell, 871);
        run.elapsed += c.duration + 0.01;
        assert_eq!(incoming(&run), 1.0);
        assert!(buffs(&run).is_empty());
    }

    #[test]
    fn buff_hover_reports_actual_numeric_strength_growth_and_proc_limits() {
        let mut run = Run::default();
        own(&mut run, "frost_armor", 3.0, 0.0);
        let c = cast(&run, "frost_armor");
        assert_eq!(utility(&mut run, "frost_armor", &c, true), Some(true));
        let text = &buffs(&run)[0].description;
        for expected in [
            "护甲 +3",
            "每1秒",
            "4码",
            "40%",
            "2秒",
            "25%",
            "+2.00个百分点",
        ] {
            assert!(text.contains(expected), "{expected}: {text}");
        }
        let mut stats = run.stats();
        let original = stats.might;
        modify_stats(&run, &mut stats);
        assert!((stats.might - original - 0.02).abs() < 0.0001);
        run.expansion.active.clear();
        own(&mut run, "seal_of_righteousness", 2.0, 0.0);
        let c = cast(&run, "seal_of_righteousness");
        assert_eq!(
            utility(&mut run, "seal_of_righteousness", &c, true),
            Some(true)
        );
        let text = &buffs(&run)[0].description;
        assert!(text.contains(&format!("{:.1}", c.dmg)));
        assert!(
            text.contains("0.5秒")
                && text.contains("1.5秒")
                && text.contains("不从持续或次级伤害递归")
        );
    }

    #[test]
    fn presence_of_mind_is_consumed_only_by_an_actual_direct_spell() {
        let mut run = Run::default();
        let c = cast(&run, "presence_of_mind");
        assert_eq!(utility(&mut run, "presence_of_mind", &c, true), Some(true));
        on_cast(&mut run, "frost_armor", false);
        let mut bolt = cast(&run, "frostbolt");
        let base = bolt.dmg;
        let mut stats = run.stats();
        modify_cast(&run, "frostbolt", &mut bolt, &mut stats);
        assert!((bolt.dmg - base * 1.5).abs() < 0.001);
        assert_eq!(bolt.modifiers.echo, 1.0);
        on_cast(&mut run, "frostbolt", true);
        assert!(!active(&run, "presence_of_mind"));
    }

    #[test]
    fn cold_snap_resets_only_frost_and_utility_growth_never_becomes_neutral() {
        let mut run = Run::default();
        own(&mut run, "cold_snap", 20.0, 0.0);
        own(&mut run, "fireball", 1.0, 10.0);
        own(&mut run, "frostbolt", 1.0, 10.0);
        let c = cast(&run, "cold_snap");
        assert_eq!(utility(&mut run, "cold_snap", &c, true), Some(true));
        assert_eq!(run.abilities[1].cd, 10.0);
        assert_eq!(run.abilities[2].cd, 0.0);
        assert_eq!(run.shield, 19.0);
        assert_eq!(utility(&mut run, "cold_snap", &c, true), Some(false));
    }

    #[test]
    fn reactive_skills_wait_for_damage_and_do_not_retrigger_after_consumption() {
        let mut run = Run::default();
        observe(&mut run);
        assert!(!reactive(&run, "overpower"));
        run.hp -= 1.0;
        observe(&mut run);
        assert!(reactive(&run, "overpower"));
        on_cast(&mut run, "overpower", false);
        assert!(!reactive(&run, "overpower"));
        assert!(reactive(&run, "heroic_strike"));
    }
}
