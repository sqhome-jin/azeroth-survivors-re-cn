//! Survivors-only gear: random drops with rolled affixes, kept between runs.
//! Nothing here touches the original client's item pipeline.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{data, localization, Phase, Rng, Run, Stats};

pub(super) const BAG_CAP: usize = 200;
const SCALE: f32 = 10_000.0;

pub(super) const RARITY_COUNT: usize = 6;
pub(super) const SLOT_COUNT: usize = 9;
const MAIN_SLOT: usize = 7;
const OFF_SLOT: usize = 8;

/// Ordinary and skill-affix counts per rarity.
pub(super) const AFFIX_COUNTS: [usize; RARITY_COUNT] = [1, 2, 3, 4, 5, 5];
pub(super) const ENH_COUNTS: [usize; RARITY_COUNT] = [0, 0, 1, 2, 3, 4];
pub(super) const MAX_ENH: usize = SLOT_COUNT * 4;

const NORMAL_WEIGHTS: [f32; RARITY_COUNT] = [50.0, 30.0, 13.0, 5.0, 1.6, 0.4];
const BOSS_WEIGHTS: [f32; RARITY_COUNT] = [0.0, 20.0, 38.0, 28.0, 11.0, 3.0];
const NORMAL_DROP_CHANCE: f32 = 0.002;

pub(super) const RARITY_NAMES: [(&str, &str); RARITY_COUNT] = [
    ("Common", "普通"),
    ("Uncommon", "优秀"),
    ("Rare", "精良"),
    ("Epic", "史诗"),
    ("Legendary", "传奇"),
    ("Primal", "太古"),
];

pub(super) const RARITY_COLORS: [Color; RARITY_COUNT] = [
    Color::srgb(0.78, 0.78, 0.78),
    Color::srgb(0.12, 1.0, 0.0),
    Color::srgb(0.0, 0.44, 0.87),
    Color::srgb(0.64, 0.21, 0.93),
    Color::srgb(1.0, 0.5, 0.0),
    Color::srgb(1.0, 0.2, 0.25),
];

pub(super) const SLOT_NAMES: [(&str, &str); SLOT_COUNT] = [
    ("Head", "头部"),
    ("Shoulder", "肩部"),
    ("Chest", "胸部"),
    ("Hands", "手部"),
    ("Waist", "腰部"),
    ("Legs", "腿部"),
    ("Feet", "脚部"),
    ("Main Hand", "主手"),
    ("Off Hand", "副手"),
];

/// `UI-PaperDoll-Slot-*` empty-slot art, in slot order.
pub(super) const SLOT_ART: [&str; SLOT_COUNT] = [
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Head",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Shoulder",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Chest",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Hands",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Waist",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Legs",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-Feet",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-MainHand",
    "Interface\\PaperDoll\\UI-PaperDoll-Slot-SecondaryHand",
];

/// (English, Chinese, icon). The icon is chosen with the name so they always agree.
type Base = (&'static str, &'static str, &'static str);

const BASES: [&[Base]; SLOT_COUNT] = [
    &[
        ("Iron Helm", "铁盔", "Interface\\Icons\\INV_Helmet_01"),
        (
            "Warden Helm",
            "守望者头盔",
            "Interface\\Icons\\INV_Helmet_02",
        ),
        (
            "Crested Helm",
            "羽饰头盔",
            "Interface\\Icons\\INV_Helmet_03",
        ),
        ("Horned Helm", "牛角盔", "Interface\\Icons\\INV_Helmet_08"),
        ("Dread Helm", "恐惧头盔", "Interface\\Icons\\INV_Helmet_17"),
        (
            "Sage Circlet",
            "贤者头环",
            "Interface\\Icons\\INV_Helmet_22",
        ),
        (
            "Bone Mask",
            "骸骨面具",
            "Interface\\Icons\\INV_Misc_Bone_HumanSkull_01",
        ),
    ],
    &[
        (
            "Iron Spaulders",
            "铁肩甲",
            "Interface\\Icons\\INV_Shoulder_01",
        ),
        (
            "Warden Mantle",
            "守望者肩甲",
            "Interface\\Icons\\INV_Shoulder_02",
        ),
        (
            "Spiked Pauldrons",
            "尖刺肩铠",
            "Interface\\Icons\\INV_Shoulder_05",
        ),
        (
            "Dread Shoulders",
            "恐惧肩饰",
            "Interface\\Icons\\INV_Shoulder_09",
        ),
        (
            "Sage Mantle",
            "贤者披肩",
            "Interface\\Icons\\INV_Shoulder_14",
        ),
        (
            "Gilded Epaulets",
            "鎏金肩章",
            "Interface\\Icons\\INV_Shoulder_23",
        ),
    ],
    &[
        (
            "Apprentice Robe",
            "学徒长袍",
            "Interface\\Icons\\INV_Chest_Cloth_01",
        ),
        (
            "Silken Robe",
            "丝绸长袍",
            "Interface\\Icons\\INV_Chest_Cloth_05",
        ),
        (
            "Archmage Vestment",
            "大法师法衣",
            "Interface\\Icons\\INV_Chest_Cloth_17",
        ),
        (
            "Hunter Jerkin",
            "猎手皮甲",
            "Interface\\Icons\\INV_Chest_Leather_01",
        ),
        (
            "Iron Cuirass",
            "铁胸甲",
            "Interface\\Icons\\INV_Chest_Plate01",
        ),
        (
            "Dread Plate",
            "恐惧板甲",
            "Interface\\Icons\\INV_Chest_Plate03",
        ),
    ],
    &[
        (
            "Iron Gauntlets",
            "铁护手",
            "Interface\\Icons\\INV_Gauntlets_01",
        ),
        (
            "Warden Grips",
            "守望者护手",
            "Interface\\Icons\\INV_Gauntlets_04",
        ),
        (
            "Spiked Gauntlets",
            "尖刺护手",
            "Interface\\Icons\\INV_Gauntlets_10",
        ),
        (
            "Sage Gloves",
            "贤者手套",
            "Interface\\Icons\\INV_Gauntlets_17",
        ),
        (
            "Dread Fists",
            "恐惧之拳",
            "Interface\\Icons\\INV_Gauntlets_25",
        ),
    ],
    &[
        ("Leather Belt", "皮腰带", "Interface\\Icons\\INV_Belt_01"),
        ("Studded Belt", "铆钉腰带", "Interface\\Icons\\INV_Belt_03"),
        (
            "Warden Girdle",
            "守望者束带",
            "Interface\\Icons\\INV_Belt_08",
        ),
        ("Sage Sash", "贤者腰带", "Interface\\Icons\\INV_Belt_13"),
        ("Dread Cinch", "恐惧腰带", "Interface\\Icons\\INV_Belt_18"),
    ],
    &[
        ("Linen Pants", "亚麻长裤", "Interface\\Icons\\INV_Pants_01"),
        (
            "Leather Leggings",
            "皮护腿",
            "Interface\\Icons\\INV_Pants_02",
        ),
        ("Iron Greaves", "铁护腿", "Interface\\Icons\\INV_Pants_04"),
        (
            "Dread Legplates",
            "恐惧腿甲",
            "Interface\\Icons\\INV_Pants_08",
        ),
        (
            "Sage Leggings",
            "贤者护腿",
            "Interface\\Icons\\INV_Pants_Cloth_05",
        ),
    ],
    &[
        ("Worn Boots", "旧靴", "Interface\\Icons\\INV_Boots_01"),
        ("Leather Boots", "皮靴", "Interface\\Icons\\INV_Boots_02"),
        ("Iron Sabatons", "铁战靴", "Interface\\Icons\\INV_Boots_05"),
        ("Dread Treads", "恐惧之靴", "Interface\\Icons\\INV_Boots_08"),
        (
            "Sage Slippers",
            "贤者软鞋",
            "Interface\\Icons\\INV_Boots_Cloth_03",
        ),
    ],
    &[
        (
            "Gnarled Staff",
            "扭曲法杖",
            "Interface\\Icons\\INV_Staff_01",
        ),
        ("Arcane Staff", "奥术法杖", "Interface\\Icons\\INV_Staff_07"),
        ("Dread Staff", "恐惧法杖", "Interface\\Icons\\INV_Staff_13"),
        ("Iron Sword", "铁剑", "Interface\\Icons\\INV_Sword_01"),
        (
            "Warden Blade",
            "守望者之刃",
            "Interface\\Icons\\INV_Sword_27",
        ),
        ("Dread Blade", "恐惧之刃", "Interface\\Icons\\INV_Sword_39"),
        ("Iron Mace", "铁锤", "Interface\\Icons\\INV_Mace_01"),
        ("Spiked Mace", "尖刺锤", "Interface\\Icons\\INV_Mace_02"),
        ("Hand Axe", "手斧", "Interface\\Icons\\INV_Axe_01"),
        ("Dread Axe", "恐惧战斧", "Interface\\Icons\\INV_Axe_09"),
        (
            "Apprentice Wand",
            "学徒魔杖",
            "Interface\\Icons\\INV_Wand_01",
        ),
        ("Sage Wand", "贤者魔杖", "Interface\\Icons\\INV_Wand_02"),
        (
            "Stiletto",
            "短剑",
            "Interface\\Icons\\INV_Weapon_ShortBlade_05",
        ),
    ],
    &[
        ("Wooden Shield", "木盾", "Interface\\Icons\\INV_Shield_01"),
        ("Iron Shield", "铁盾", "Interface\\Icons\\INV_Shield_02"),
        (
            "Warden Shield",
            "守望者之盾",
            "Interface\\Icons\\INV_Shield_04",
        ),
        (
            "Dread Shield",
            "恐惧之盾",
            "Interface\\Icons\\INV_Shield_06",
        ),
        ("Tower Shield", "塔盾", "Interface\\Icons\\INV_Shield_10"),
        (
            "Tattered Tome",
            "残破典籍",
            "Interface\\Icons\\INV_Misc_Book_01",
        ),
        (
            "Arcane Tome",
            "奥术典籍",
            "Interface\\Icons\\INV_Misc_Book_07",
        ),
        (
            "Dread Grimoire",
            "恐惧魔典",
            "Interface\\Icons\\INV_Misc_Book_09",
        ),
        ("Crystal Orb", "水晶球", "Interface\\Icons\\INV_Misc_Orb_01"),
        (
            "Shadow Orb",
            "暗影宝珠",
            "Interface\\Icons\\INV_Misc_Orb_02",
        ),
        ("Frost Orb", "冰霜宝珠", "Interface\\Icons\\INV_Misc_Orb_03"),
        ("Fire Orb", "火焰宝珠", "Interface\\Icons\\INV_Misc_Orb_04"),
        (
            "Nature Orb",
            "自然宝珠",
            "Interface\\Icons\\INV_Misc_Orb_05",
        ),
        (
            "Scholar's Relic",
            "学者遗物",
            "Interface\\Icons\\INV_Offhand_Stratholme_A_02",
        ),
    ],
];

/// What a main-hand or off-hand base looks like on the hero, by `ItemDisplayInfo` row:
/// `(display, subclass, material, inventory type, sheath)`. `None` keeps the class's own.
type Look = Option<(u32, u8, u8, u8, u8)>;

const MAIN_LOOKS: [Look; 13] = [
    Some((472, 10, 2, 17, 2)),
    Some((1190, 10, 2, 17, 2)),
    Some((1643, 10, 2, 17, 2)),
    Some((1550, 7, 1, 13, 3)),
    Some((862, 7, 1, 13, 3)),
    Some((7482, 7, 1, 13, 3)),
    Some((1160, 4, 1, 13, 3)),
    Some((861, 4, 1, 13, 3)),
    Some((1386, 0, 1, 13, 3)),
    Some((1390, 0, 1, 13, 3)),
    Some((5720, 19, 2, 13, 3)),
    Some((5712, 19, 2, 13, 3)),
    Some((2707, 15, 1, 13, 3)),
];

/// Shields wear as armour; orbs are held like a one-handed weapon, as the rogue's off-hand
/// dagger is. Tomes and relics have no model of their own here.
const OFF_LOOKS: [Look; 14] = [
    Some((2052, 6, 1, 14, 4)),
    Some((2329, 6, 1, 14, 4)),
    Some((1705, 6, 1, 14, 4)),
    Some((2080, 6, 1, 14, 4)),
    Some((2181, 6, 1, 14, 4)),
    None,
    None,
    None,
    Some((5566, 15, 1, 13, 3)),
    Some((9222, 15, 1, 13, 3)),
    Some((5926, 15, 1, 13, 3)),
    Some((5927, 15, 1, 13, 3)),
    Some((8043, 15, 1, 13, 3)),
    None,
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct AffixDef {
    pub en: &'static str,
    pub zh: &'static str,
    /// Roll range in 1/10000 units.
    pub lo: u32,
    pub hi: u32,
    /// Shown as a percentage rather than a flat number.
    pub percent: bool,
}

const fn af(en: &'static str, zh: &'static str, lo: u32, hi: u32, percent: bool) -> AffixDef {
    AffixDef {
        en,
        zh,
        lo,
        hi,
        percent,
    }
}

pub(super) const AFFIX_COUNT: usize = 14;
pub(super) const AFFIXES: [AffixDef; AFFIX_COUNT] = [
    af("Damage", "伤害", 400, 1200, true),
    af("Max Health", "最大生命", 600, 1600, true),
    af("Move Speed", "移动速度", 400, 1000, true),
    af("Health Regen /s", "每秒生命回复", 2000, 8000, false),
    af("Haste", "急速", 200, 800, true),
    af("Area", "范围", 400, 1200, true),
    af("Pickup Range", "拾取范围", 800, 2400, true),
    af("Experience", "经验获取", 400, 1200, true),
    af("Armor", "护甲", 6000, 20000, false),
    af("Crit Chance", "暴击几率", 100, 400, true),
    af("Crit Damage", "暴击伤害", 600, 2000, true),
    af("Life Leech", "生命偷取", 20, 80, true),
    af("Damage Reduction", "伤害减免", 100, 400, true),
    af("Mobility Cooldown", "位移冷却缩减", 200, 800, true),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EnhKind {
    Damage,
    Cooldown,
    Count,
    Radius,
    Duration,
    Frequency,
    Range,
    ChainPower,
    Echo,
    Splash,
    Ember,
    Execute,
    Leech,
    Control,
    BuffDamage,
    BuffHaste,
    BuffCrit,
    BuffSpeed,
    BuffDefense,
}

pub(super) const ENH_KINDS: [EnhKind; 19] = [
    EnhKind::Damage,
    EnhKind::Cooldown,
    EnhKind::Count,
    EnhKind::Radius,
    EnhKind::Duration,
    EnhKind::Frequency,
    EnhKind::Range,
    EnhKind::ChainPower,
    EnhKind::Echo,
    EnhKind::Splash,
    EnhKind::Ember,
    EnhKind::Execute,
    EnhKind::Leech,
    EnhKind::Control,
    EnhKind::BuffDamage,
    EnhKind::BuffHaste,
    EnhKind::BuffCrit,
    EnhKind::BuffSpeed,
    EnhKind::BuffDefense,
];

impl EnhKind {
    pub fn is_buff(self) -> bool {
        matches!(
            self,
            Self::BuffDamage
                | Self::BuffHaste
                | Self::BuffCrit
                | Self::BuffSpeed
                | Self::BuffDefense
        )
    }

    pub fn supports(self, def: &data::AbilityDef) -> bool {
        if let Some(supported) = super::spell_expansion::supports(def, self) {
            return supported;
        }
        use data::Kind;
        match self {
            Self::Damage => def.damage > 0.0 || def.dot > 0.0,
            Self::Cooldown => true,
            Self::Count => matches!(
                def.kind,
                Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Chain | Kind::Dot | Kind::Orbit
            ),
            Self::Radius => {
                def.radius > 0.0
                    && (matches!(
                        def.kind,
                        Kind::Bolt
                            | Kind::Strike
                            | Kind::Nova
                            | Kind::Cone
                            | Kind::Ground
                            | Kind::GroundSelf
                            | Kind::Chain
                            | Kind::Aura
                            | Kind::Orbit
                    ) || (def.kind == Kind::Totem && def.key == "magma_totem"))
            }
            Self::Duration => {
                def.duration > 0.0
                    && (def.dot > 0.0
                        || matches!(
                            def.kind,
                            Kind::Ground
                                | Kind::GroundSelf
                                | Kind::Shield
                                | Kind::Immune
                                | Kind::Thorns
                                | Kind::Totem
                        ))
            }
            Self::Frequency => {
                def.dot > 0.0
                    || matches!(def.kind, Kind::Ground | Kind::GroundSelf | Kind::Totem)
                        && def.duration > 0.0
            }
            Self::Range => {
                matches!(
                    def.kind,
                    Kind::Bolt | Kind::Strike | Kind::Dot | Kind::Ground | Kind::Totem
                ) && def.range > 0.0
            }
            Self::ChainPower => def.kind == Kind::Chain,
            Self::Echo => matches!(
                def.kind,
                Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Nova | Kind::Cone | Kind::Chain
            ),
            Self::Splash => matches!(def.kind, Kind::Bolt | Kind::Strike) && def.radius == 0.0,
            Self::Ember => {
                def.dot == 0.0
                    && matches!(
                        def.kind,
                        Kind::Bolt
                            | Kind::Strike
                            | Kind::Melee
                            | Kind::Nova
                            | Kind::Cone
                            | Kind::Chain
                    )
            }
            Self::Control => matches!(
                def.effect,
                data::Effect::Slow(..)
                    | data::Effect::Root(_)
                    | data::Effect::Stun(_)
                    | data::Effect::Fear(_)
                    | data::Effect::Weaken(..)
            ),
            Self::Execute
            | Self::Leech
            | Self::BuffDamage
            | Self::BuffHaste
            | Self::BuffCrit
            | Self::BuffSpeed
            | Self::BuffDefense => {
                (def.damage > 0.0 || def.dot > 0.0)
                    && !matches!(def.kind, Kind::Shield | Kind::Heal)
            }
        }
    }

    /// Roll range in 1/10000 units; a count is whole numbers.
    fn range(self) -> (u32, u32) {
        match self {
            Self::Damage => (2500, 5000),
            Self::Cooldown => (1500, 3000),
            Self::Count => (10_000, 20_000),
            Self::Radius | Self::Duration => (2000, 4000),
            Self::Frequency => (3000, 10_000),
            Self::Range => (2000, 4000),
            Self::ChainPower => (500, 1000),
            Self::Echo => (1500, 3000),
            Self::Splash => (2000, 3500),
            Self::Ember => (2000, 4000),
            Self::Execute => (3000, 6000),
            Self::Leech => (300, 600),
            Self::Control => (1500, 3000),
            Self::BuffDamage => (3000, 3000),
            Self::BuffHaste => (2000, 2000),
            Self::BuffCrit => (1000, 1000),
            Self::BuffSpeed => (2500, 2500),
            Self::BuffDefense => (2000, 2000),
        }
    }

    fn roll_range(self, all: bool) -> (u32, u32) {
        if !all {
            return if self == Self::Cooldown {
                (1000, 2500)
            } else {
                self.range()
            };
        }
        match self {
            Self::Count => (10_000, 10_000),
            Self::Damage => (800, 1500),
            Self::Cooldown => (400, 1000),
            Self::Radius | Self::Duration | Self::Range => (800, 1500),
            Self::Frequency => (1000, 3000),
            Self::Leech => (100, 200),
            Self::BuffDamage => (1500, 1500),
            Self::BuffHaste | Self::BuffDefense => (1000, 1000),
            Self::BuffCrit => (500, 500),
            Self::BuffSpeed => (1200, 1200),
            _ => {
                let (lo, hi) = self.range();
                (lo / 2, hi / 2)
            }
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Damage => localization::text("Damage", "伤害"),
            Self::Cooldown => localization::text("Cooldown", "冷却时间"),
            Self::Count => localization::text("Projectiles / Hits", "数量"),
            Self::Radius => localization::text("Area", "范围"),
            Self::Duration => localization::text("Duration", "持续时间"),
            Self::Frequency => localization::text("Tick frequency", "触发频率"),
            Self::Range => localization::text("Cast range", "施法距离"),
            Self::ChainPower => localization::text("Chain retention", "弹跳伤害保留"),
            Self::Echo => localization::text("Echo chance (50% damage)", "回响概率（50%伤害）"),
            Self::Splash => localization::text("Splash damage (3 yd)", "溅射伤害（3码）"),
            Self::Ember => localization::text("Extra damage over 3 sec", "额外3秒余烬伤害"),
            Self::Execute => {
                localization::text("Damage below 30% health", "对生命低于30%的敌人增伤")
            }
            Self::Leech => localization::text("Damage converted to healing", "伤害转化为治疗"),
            Self::Control => localization::text("Control duration", "控制持续时间"),
            Self::BuffDamage => localization::text("Damage buff", "毁灭之力·伤害"),
            Self::BuffHaste => localization::text("Haste buff", "施法狂热·急速"),
            Self::BuffCrit => localization::text("Crit buff", "致命洞察·暴击率"),
            Self::BuffSpeed => localization::text("Speed buff", "灵动疾行·移速"),
            Self::BuffDefense => localization::text("Defense buff", "坚韧护体·减伤"),
        }
    }

    pub fn show(self, value: f32) -> String {
        match self {
            Self::Count => format!("+{}", value.round() as u32),
            Self::Cooldown => format!("-{:.0}%", value * 100.0),
            _ => format!("+{:.0}%", value * 100.0),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct Roll {
    pub kind: u8,
    pub value: u32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct EnhRoll {
    /// Empty for enhancements affecting every compatible skill.
    pub ability: String,
    pub kind: u8,
    pub value: u32,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub(super) struct Item {
    pub id: u64,
    pub slot: u8,
    pub rarity: u8,
    pub base: u8,
    /// The class that dropped it, which the enhancement is drawn from.
    pub class: u8,
    pub affixes: Vec<Roll>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enhancement: Option<EnhRoll>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enhancements: Vec<EnhRoll>,
}

impl Item {
    fn base(&self) -> &'static Base {
        let list = BASES[usize::from(self.slot).min(SLOT_COUNT - 1)];
        &list[usize::from(self.base).min(list.len() - 1)]
    }

    pub fn name(&self) -> &'static str {
        let b = self.base();
        localization::text(b.0, b.1)
    }

    pub fn icon(&self) -> &'static str {
        self.base().2
    }

    pub fn color(&self) -> Color {
        RARITY_COLORS[usize::from(self.rarity).min(RARITY_COUNT - 1)]
    }

    pub fn rarity_name(&self) -> &'static str {
        let r = RARITY_NAMES[usize::from(self.rarity).min(RARITY_COUNT - 1)];
        localization::text(r.0, r.1)
    }

    pub fn slot_name(&self) -> &'static str {
        let s = SLOT_NAMES[usize::from(self.slot).min(SLOT_COUNT - 1)];
        localization::text(s.0, s.1)
    }
}

pub(super) fn affix(kind: u8) -> &'static AffixDef {
    &AFFIXES[usize::from(kind).min(AFFIX_COUNT - 1)]
}

/// One affix line as the tooltip shows it, with the range it rolled in.
pub(super) fn describe(roll: &Roll) -> String {
    let def = affix(roll.kind);
    let name = localization::text(def.en, def.zh);
    let show = |v: u32| {
        if def.percent {
            format!("{:.1}%", v as f32 / SCALE * 100.0)
        } else {
            format!("{:.1}", v as f32 / SCALE)
        }
    };
    format!(
        "+{} {}  [{} - {}]",
        show(roll.value),
        name,
        show(def.lo),
        show(def.hi)
    )
}

pub(super) fn describe_enhancement(enh: &EnhRoll, ability_name: &str) -> String {
    let kind = ENH_KINDS[usize::from(enh.kind).min(ENH_KINDS.len() - 1)];
    let target = if enh.ability.is_empty() {
        localization::text("All compatible skills", "所有相关技能")
    } else {
        ability_name
    };
    if kind.is_buff() {
        return format!(
            "{target}: {}% {} → {} {} ({})",
            if enh.ability.is_empty() { 15 } else { 30 },
            localization::text("on damage", "伤害触发"),
            kind.label(),
            kind.show(enh.value as f32 / SCALE),
            localization::text("30s / ICD 30s", "持续30秒 / 内置冷却30秒")
        );
    }
    format!(
        "{target}: {} {}",
        kind.label(),
        kind.show(enh.value as f32 / SCALE)
    )
}

/// The derived effect of everything a class wears, folded into [`Stats`].
#[derive(Clone, Copy)]
pub(super) struct Bonuses {
    sum: [f32; AFFIX_COUNT],
    pub enh: [Option<SkillRoll>; MAX_ENH],
}

impl Default for Bonuses {
    fn default() -> Self {
        Self {
            sum: [0.0; AFFIX_COUNT],
            enh: [None; MAX_ENH],
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct SkillRoll {
    pub item: u64,
    pub index: u8,
    pub ability: Option<&'static str>,
    pub kind: EnhKind,
    pub value: f32,
}

impl SkillRoll {
    pub fn matches(self, def: &data::AbilityDef) -> bool {
        self.ability.is_none_or(|key| key == def.key) && self.kind.supports(def)
    }
}

impl Bonuses {
    fn get(&self, kind: usize) -> f32 {
        self.sum[kind]
    }

    pub fn crit_damage(&self) -> f32 {
        self.get(10)
    }

    pub fn leech(&self) -> f32 {
        self.get(11)
    }

    /// Multiplier on damage taken.
    pub fn incoming(&self) -> f32 {
        1.0 - self.get(12).min(0.6)
    }

    /// Multiplier on the mobility cooldown.
    pub fn mobility_cooldown(&self) -> f32 {
        1.0 - self.get(13).min(0.6)
    }

    pub fn enhancements<'a>(
        &'a self,
        ability: &'a str,
    ) -> impl Iterator<Item = (EnhKind, f32)> + 'a {
        let def = data::ability(ability);
        self.enh
            .iter()
            .flatten()
            .filter(move |roll| roll.matches(def))
            .map(|roll| (roll.kind, roll.value))
    }
}

pub(super) fn modify_stats(stats: &mut Stats, b: &Bonuses) {
    stats.might += b.get(0);
    stats.max_hp *= 1.0 + b.get(1);
    stats.speed *= 1.0 + b.get(2);
    stats.regen += b.get(3);
    stats.haste += b.get(4);
    stats.area += b.get(5);
    stats.magnet *= 1.0 + b.get(6);
    stats.growth += b.get(7);
    stats.armor += b.get(8);
    stats.crit = (stats.crit + b.get(9)).min(1.0);
    stats.gear = *b;
}

#[cfg(test)]
fn roll_rarity(rng: &mut Rng, boss: bool) -> u8 {
    roll_rarity_with_luck(rng, boss, 0.0)
}

fn rarity_weights(boss: bool, luck: f32) -> [f32; RARITY_COUNT] {
    let weights = if boss { BOSS_WEIGHTS } else { NORMAL_WEIGHTS };
    let luck = luck.clamp(0.0, 200.0) / 100.0;
    std::array::from_fn(|index| weights[index] * (1.0 + luck * index as f32 / 5.0))
}

pub(super) fn roll_rarity_with_luck(rng: &mut Rng, boss: bool, luck: f32) -> u8 {
    let weights = rarity_weights(boss, luck);
    let total: f32 = weights.iter().sum();
    let mut pick = rng.f32() * total;
    for (i, w) in weights.iter().enumerate() {
        if pick < *w {
            return i as u8;
        }
        pick -= w;
    }
    weights.iter().rposition(|w| *w > 0.0).unwrap_or(0) as u8
}

#[cfg(test)]
fn roll_drop_rarity(rng: &mut Rng, boss: bool) -> Option<u8> {
    roll_drop_with_luck(rng, boss, 0.0)
}

pub(super) fn drop_chance(luck: f32) -> f32 {
    NORMAL_DROP_CHANCE * (1.0 + luck.clamp(0.0, 200.0) / 100.0)
}

pub(super) fn roll_drop_with_luck(rng: &mut Rng, boss: bool, luck: f32) -> Option<u8> {
    if boss || rng.f32() < drop_chance(luck) {
        Some(roll_rarity_with_luck(rng, boss, luck))
    } else {
        None
    }
}

fn roll_value(rng: &mut Rng, lo: u32, hi: u32, primal: bool) -> u32 {
    let one = |rng: &mut Rng| lo + (rng.f32() * (hi - lo + 1) as f32) as u32;
    let v = one(rng).min(hi);
    if primal {
        v.max(one(rng).min(hi))
    } else {
        v
    }
}

/// A fresh item. `id` is assigned by the store.
pub(super) fn roll_item(rng: &mut Rng, class: &data::ClassDef, rarity: u8) -> Item {
    let rarity = rarity.min(RARITY_COUNT as u8 - 1);
    let primal = usize::from(rarity) == RARITY_COUNT - 1;
    let slot = rng.index(SLOT_COUNT);
    let base = rng.index(BASES[slot].len());

    // A partial shuffle picks distinct affix kinds.
    let mut kinds: Vec<u8> = (0..AFFIX_COUNT as u8).collect();
    let count = AFFIX_COUNTS[usize::from(rarity)];
    let mut affixes = Vec::with_capacity(count);
    for i in 0..count {
        let j = i + rng.index(kinds.len() - i);
        kinds.swap(i, j);
        let def = affix(kinds[i]);
        affixes.push(Roll {
            kind: kinds[i],
            value: roll_value(rng, def.lo, def.hi, primal),
        });
    }

    let enhancements = roll_enhancements(
        rng,
        class,
        ENH_COUNTS[usize::from(rarity)],
        primal,
        Vec::new(),
    );

    Item {
        id: 0,
        slot: slot as u8,
        rarity,
        base: base as u8,
        class: class.id,
        affixes,
        enhancement: None,
        enhancements,
    }
}

fn roll_enhancements(
    rng: &mut Rng,
    class: &data::ClassDef,
    count: usize,
    primal: bool,
    mut out: Vec<EnhRoll>,
) -> Vec<EnhRoll> {
    let mut candidates = Vec::new();
    for (kind, effect) in ENH_KINDS.into_iter().enumerate() {
        if class
            .pool
            .iter()
            .any(|key| effect.supports(data::ability(key)))
        {
            candidates.push((String::new(), kind));
        }
        for &key in class.pool {
            if effect.supports(data::ability(key)) {
                candidates.push((key.to_owned(), kind));
            }
        }
    }
    while out.len() < count {
        let targeted: BTreeMap<_, _> = out
            .iter()
            .filter(|e| !e.ability.is_empty())
            .map(|e| (e.ability.as_str(), ()))
            .collect();
        let has_buff = out.iter().any(|e| ENH_KINDS[usize::from(e.kind)].is_buff());
        let eligible: Vec<_> = candidates
            .iter()
            .filter(|(key, kind)| {
                !out.iter()
                    .any(|e| e.ability == *key && usize::from(e.kind) == *kind)
                    && (!has_buff || !ENH_KINDS[*kind].is_buff())
                    && (key.is_empty() || targeted.len() < 2 || targeted.contains_key(key.as_str()))
            })
            .collect();
        // Choose the scope first so numerous specific skills don't crowd out global rolls.
        let all = rng.f32() < 0.35;
        let scoped: Vec<_> = eligible
            .iter()
            .copied()
            .filter(|(key, _)| key.is_empty() == all)
            .collect();
        let pool = if scoped.is_empty() {
            &eligible
        } else {
            &scoped
        };
        let (key, kind) = pool[rng.index(pool.len())];
        let (lo, hi) = ENH_KINDS[*kind].roll_range(key.is_empty());
        let value = if ENH_KINDS[*kind] == EnhKind::Count {
            roll_value(rng, lo / 10_000, hi / 10_000, primal) * 10_000
        } else {
            roll_value(rng, lo, hi, primal)
        };
        out.push(EnhRoll {
            ability: key.clone(),
            kind: *kind as u8,
            value,
        });
    }
    out
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Save {
    version: u32,
    next_id: u64,
    bag: Vec<Item>,
    /// class id → slot → item id.
    worn: BTreeMap<String, BTreeMap<String, u64>>,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            version: 3,
            next_id: 1,
            bag: Vec::new(),
            worn: BTreeMap::new(),
        }
    }
}

impl Save {
    fn validate_skills(&self) -> Result<(), &'static str> {
        for item in &self.bag {
            if usize::from(item.rarity) >= RARITY_COUNT
                || !data::CLASSES.iter().any(|c| c.id == item.class)
                || item.enhancements.len() + usize::from(item.enhancement.is_some()) > 4
            {
                return Err("invalid equipment skill data");
            }
            for enh in item.enhancements.iter().chain(item.enhancement.iter()) {
                if usize::from(enh.kind) >= ENH_KINDS.len()
                    || (!enh.ability.is_empty()
                        && !data::ABILITIES.iter().any(|a| a.key == enh.ability))
                    || enh.value > 100_000
                {
                    return Err("invalid skill enhancement");
                }
            }
        }
        Ok(())
    }

    fn upgrade_affixes(&mut self) -> Result<(), &'static str> {
        if self.version == 1 {
            for item in &mut self.bag {
                let count = AFFIX_COUNTS
                    .get(usize::from(item.rarity))
                    .ok_or("invalid item rarity")?;
                item.affixes.truncate(*count);
                for roll in &mut item.affixes {
                    roll.value = roll.value.checked_mul(2).ok_or("affix value overflow")?;
                }
            }
            self.version = 2;
        }
        Ok(())
    }

    /// Old rolls could target an unused parameter. Replace only those with a valid cooldown roll.
    fn repair_enhancements(&mut self) -> usize {
        let mut repaired = 0;
        for item in &mut self.bag {
            for enh in item
                .enhancement
                .iter_mut()
                .chain(item.enhancements.iter_mut())
            {
                let Some(def) = data::ABILITIES.iter().find(|a| a.key == enh.ability) else {
                    continue;
                };
                if ENH_KINDS
                    .get(usize::from(enh.kind))
                    .is_some_and(|kind| !kind.supports(def))
                {
                    let (lo, hi) = EnhKind::Cooldown.range();
                    enh.kind = ENH_KINDS
                        .iter()
                        .position(|k| *k == EnhKind::Cooldown)
                        .expect("cooldown enhancement") as u8;
                    enh.value = enh.value.clamp(lo, hi);
                    repaired += 1;
                }
            }
        }
        repaired
    }

    fn upgrade_skills(&mut self) {
        if self.version != 2 {
            return;
        }
        for item in &mut self.bag {
            if let Some(old) = item.enhancement.take() {
                item.enhancements.push(old);
            }
            let class = data::CLASSES.iter().find(|c| c.id == item.class);
            if let Some(class) = class {
                // Stable migration: the same old item gets the same additional rolls on retry.
                let mut rng = Rng(item.id.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
                item.enhancements = roll_enhancements(
                    &mut rng,
                    class,
                    ENH_COUNTS[usize::from(item.rarity)],
                    item.rarity == 5,
                    std::mem::take(&mut item.enhancements),
                );
            }
        }
        self.version = 3;
    }
}

/// The shared backpack and each class's worn set.
#[derive(Resource, Default)]
pub(super) struct Gear {
    save: Save,
    dirty: bool,
    read_only: bool,
    save_elapsed: f32,
    pub error: Option<String>,
    /// Bumped on every change so panels know to redraw.
    pub version: u32,
}

impl Gear {
    pub fn bag(&self) -> &[Item] {
        &self.save.bag
    }

    pub fn item(&self, id: u64) -> Option<&Item> {
        self.save.bag.iter().find(|i| i.id == id)
    }

    pub fn worn(&self, class: u8, slot: usize) -> Option<&Item> {
        let id = self
            .save
            .worn
            .get(&class.to_string())?
            .get(&slot.to_string())?;
        self.item(*id)
    }

    /// The class wearing this item, if any.
    pub fn wearer(&self, id: u64) -> Option<u8> {
        self.save.worn.iter().find_map(|(class, slots)| {
            slots
                .values()
                .any(|&worn| worn == id)
                .then(|| class.parse().ok())
                .flatten()
        })
    }

    fn touch(&mut self) {
        self.dirty = true;
        self.version = self.version.wrapping_add(1);
    }

    /// Adds a dropped item, making room by discarding the lowest unworn one when full.
    pub fn add(&mut self, mut item: Item) -> u64 {
        if self.save.bag.len() >= BAG_CAP {
            let victim = self
                .save
                .bag
                .iter()
                .enumerate()
                .filter(|(_, i)| self.wearer(i.id).is_none())
                .min_by_key(|(_, i)| i.rarity)
                .map(|(at, _)| at);
            match victim {
                Some(at) => {
                    self.save.bag.remove(at);
                }
                None => return 0,
            }
        }
        item.id = self.save.next_id;
        self.save.next_id += 1;
        let id = item.id;
        self.save.bag.push(item);
        self.touch();
        id
    }

    pub fn award_primal(
        &mut self,
        rng: &mut Rng,
        class: &'static data::ClassDef,
    ) -> Result<String, String> {
        if self.read_only {
            return Err(localization::text(
                "Primal reward blocked: gear save is unreadable. Back up and repair the save.",
                "太古奖励发放失败：装备存档无法读取，请备份并修复存档。",
            )
            .into());
        }
        let item = roll_item(rng, class, (RARITY_COUNT - 1) as u8);
        let label = format!("[{}] {}", item.rarity_name(), item.name());
        if self.add(item) == 0 {
            return Err(localization::text(
                "Primal reward blocked: no unworn backpack slot is available.",
                "太古奖励发放失败：背包已满且没有可替换的未穿戴装备。",
            )
            .into());
        }
        // Force the existing persistence path this frame, before entering the results screen.
        self.save_elapsed = 5.0;
        Ok(label)
    }

    /// An item is worn by one class at a time; wearing it elsewhere takes it off.
    pub fn equip(&mut self, class: u8, id: u64) -> bool {
        let Some(slot) = self.item(id).map(|i| i.slot) else {
            return false;
        };
        if self.read_only {
            return false;
        }
        for slots in self.save.worn.values_mut() {
            slots.retain(|_, worn| *worn != id);
        }
        self.save
            .worn
            .entry(class.to_string())
            .or_default()
            .insert(slot.to_string(), id);
        self.touch();
        true
    }

    pub fn unequip(&mut self, class: u8, slot: usize) {
        if self.read_only {
            return;
        }
        let removed = self
            .save
            .worn
            .get_mut(&class.to_string())
            .is_some_and(|slots| slots.remove(&slot.to_string()).is_some());
        if removed {
            self.touch();
        }
    }

    pub fn discard(&mut self, id: u64) {
        if self.read_only {
            return;
        }
        for slots in self.save.worn.values_mut() {
            slots.retain(|_, worn| *worn != id);
        }
        self.save.bag.retain(|i| i.id != id);
        self.touch();
    }

    /// Discards every unworn item of at most this rarity; returns how many.
    pub fn discard_up_to(&mut self, rarity: u8) -> usize {
        if self.read_only {
            return 0;
        }
        let before = self.save.bag.len();
        let keep: Vec<u64> = self
            .save
            .bag
            .iter()
            .filter(|i| i.rarity > rarity || self.wearer(i.id).is_some())
            .map(|i| i.id)
            .collect();
        self.save.bag.retain(|i| keep.contains(&i.id));
        let gone = before - self.save.bag.len();
        if gone > 0 {
            self.touch();
        }
        gone
    }

    /// The class's virtual weapons with its worn main hand and off hand swapped in.
    pub fn weapons(&self, class: &data::ClassDef) -> Vec<data::Weapon> {
        let mut out = class.weapons.to_vec();
        for (virtual_slot, slot, looks) in [
            (0u16, MAIN_SLOT, &MAIN_LOOKS[..]),
            (1, OFF_SLOT, &OFF_LOOKS[..]),
        ] {
            let look = self
                .worn(class.id, slot)
                .and_then(|i| looks.get(usize::from(i.base)).copied().flatten());
            let Some((display, sub, material, inv, sheath)) = look else {
                continue;
            };
            let item_class = if inv == 14 { 4 } else { 2 };
            let weapon = (
                virtual_slot,
                display,
                item_class,
                sub,
                material,
                inv,
                sheath,
            );
            match out.iter_mut().find(|w| w.0 == virtual_slot) {
                Some(w) => *w = weapon,
                None => out.push(weapon),
            }
        }
        out
    }

    pub fn bonuses(&self, class: u8) -> Bonuses {
        let mut out = Bonuses::default();
        let mut enh = 0;
        for slot in 0..SLOT_COUNT {
            let Some(item) = self.worn(class, slot) else {
                continue;
            };
            for roll in &item.affixes {
                out.sum[usize::from(roll.kind).min(AFFIX_COUNT - 1)] += roll.value as f32 / SCALE;
            }
            for (index, e) in item
                .enhancements
                .iter()
                .chain(item.enhancement.iter())
                .enumerate()
            {
                let found = if e.ability.is_empty() {
                    Some(None)
                } else {
                    data::ABILITIES
                        .iter()
                        .find(|a| a.key == e.ability)
                        .map(|d| Some(d.key))
                };
                if let Some(ability) = found {
                    let kind = ENH_KINDS[usize::from(e.kind).min(ENH_KINDS.len() - 1)];
                    if enh < MAX_ENH {
                        out.enh[enh] = Some(SkillRoll {
                            item: item.id,
                            index: index as u8,
                            ability,
                            kind,
                            value: e.value as f32 / SCALE,
                        });
                        enh += 1;
                    }
                }
            }
        }
        out
    }
}

fn save_path() -> Option<std::path::PathBuf> {
    crate::local_state::home().map(|p| p.join("survivors-gear.toml"))
}

fn load(mut gear: ResMut<Gear>) {
    let Some(path) = save_path() else { return };
    match crate::local_state::read_to_string(&path) {
        Ok(text) => match toml::from_str::<Save>(&text) {
            Ok(mut save) if matches!(save.version, 1..=3) => {
                if let Err(error) = save.validate_skills() {
                    fail_load(&mut gear, error);
                    return;
                }
                let skill_upgrade = save.version < 3;
                let upgraded = save.version == 1;
                if let Err(error) = save.upgrade_affixes() {
                    fail_load(&mut gear, error);
                    return;
                }
                if upgraded {
                    info!("survivors gear: upgraded affixes to save version 2");
                    gear.dirty = true;
                }
                let repaired = save.repair_enhancements();
                if repaired > 0 {
                    warn!(
                        "survivors gear: repaired {repaired} ineffective skill enhancements to cooldown reduction"
                    );
                    gear.dirty = true;
                }
                if skill_upgrade {
                    save.upgrade_skills();
                    gear.dirty = true;
                    info!("survivors gear: upgraded skill enhancements to save version 3");
                }
                gear.save = save;
            }
            Ok(_) => fail_load(&mut gear, "unsupported save version"),
            Err(error) => fail_load(&mut gear, &error.to_string()),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => fail_load(&mut gear, &error.to_string()),
    }
}

fn fail_load(gear: &mut Gear, reason: &str) {
    warn!("survivors gear: cannot load: {reason}");
    gear.read_only = true;
    gear.error = Some(
        localization::text(
            "Cannot read gear. Saving disabled to protect the existing save.",
            "无法读取装备存档，已停止写入以保护原存档。请备份并检查存档。",
        )
        .into(),
    );
}

fn persist(
    time: Res<Time<Real>>,
    phase: Res<State<Phase>>,
    mut gear: ResMut<Gear>,
    exit: MessageReader<AppExit>,
) {
    gear.save_elapsed += time.delta_secs();
    let leaving =
        matches!(phase.get(), Phase::Menu | Phase::GameOver | Phase::Paused) || !exit.is_empty();
    if !gear.dirty || gear.read_only || (!leaving && gear.save_elapsed < 5.0) {
        return;
    }
    if gear.error.is_some() && gear.save_elapsed < 5.0 {
        return;
    }
    gear.save_elapsed = 0.0;
    let result = save_path()
        .ok_or_else(|| "local persistence unavailable".to_owned())
        .and_then(|path| {
            toml::to_string(&gear.save)
                .map_err(|e| e.to_string())
                .and_then(|text| {
                    crate::local_state::write_atomic(&path, &text).map_err(|e| e.to_string())
                })
        });
    match result {
        Ok(()) => {
            gear.dirty = false;
            gear.error = None;
        }
        Err(error) => {
            warn!("survivors gear: save failed: {error}");
            gear.error = Some(
                localization::text("Gear not saved; retrying.", "装备尚未保存，正在重试。").into(),
            );
        }
    }
}

/// What the run wears, kept current whenever the worn set changes.
fn sync_run(gear: Res<Gear>, mut run: ResMut<Run>) {
    let bonuses = gear.bonuses(run.class.id);
    apply_to_run(&mut run, bonuses);
}

pub(super) fn apply_to_run(run: &mut Run, bonuses: Bonuses) {
    run.effects.reconcile(&bonuses, run.elapsed);
    run.gear = bonuses;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Gear>()
        .add_systems(Startup, load)
        .add_systems(
            OnEnter(Phase::Playing),
            sync_run.before(super::talents::lock_loadout),
        )
        .add_systems(
            Update,
            (
                sync_run.run_if(resource_changed::<Gear>),
                persist
                    .after(super::enemies::drop_loot)
                    .before(super::hero::hero_vitals),
            )
                .chain(),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn luck_zero_preserves_the_original_drop_sequence() {
        let mut baseline = Rng(12345);
        let mut lucky = Rng(12345);
        for boss in [false, true] {
            for _ in 0..10_000 {
                let expected = if boss || baseline.f32() < NORMAL_DROP_CHANCE {
                    Some(roll_rarity(&mut baseline, boss))
                } else {
                    None
                };
                assert_eq!(expected, roll_drop_with_luck(&mut lucky, boss, 0.0),);
            }
        }
        assert_eq!(drop_chance(0.0), 0.002);
        assert_eq!(drop_chance(100.0), 0.004);
        assert_eq!(drop_chance(200.0), 0.006);
        assert_eq!(drop_chance(20_000.0), 0.006);
    }

    #[test]
    fn luck_improves_both_drop_frequency_and_quality_without_opening_forbidden_tiers() {
        let mean = |weights: [f32; RARITY_COUNT]| {
            weights
                .iter()
                .enumerate()
                .map(|(i, weight)| i as f32 * weight)
                .sum::<f32>()
                / weights.iter().sum::<f32>()
        };
        for boss in [false, true] {
            assert!(mean(rarity_weights(boss, 100.0)) > mean(rarity_weights(boss, 0.0)));
            assert!(mean(rarity_weights(boss, 200.0)) > mean(rarity_weights(boss, 100.0)));
        }
        assert_eq!(rarity_weights(true, 200.0)[0], 0.0);
        let mut normal_rng = Rng(12345);
        let mut lucky_rng = Rng(12345);
        let mut normal = 0;
        let mut lucky = 0;
        for _ in 0..200_000 {
            normal += usize::from(roll_drop_with_luck(&mut normal_rng, false, 0.0).is_some());
            lucky += usize::from(roll_drop_with_luck(&mut lucky_rng, false, 100.0).is_some());
        }
        assert!((300..500).contains(&normal), "{normal}");
        assert!((650..950).contains(&lucky), "{lucky}");
        assert!(lucky > normal);
    }

    #[test]
    fn final_reward_is_class_compatible_primal_even_when_backpack_is_full() {
        let mut gear = Gear::default();
        let mut rng = Rng(12345);
        for _ in 0..BAG_CAP {
            gear.add(roll_item(&mut rng, class(0), 0));
        }
        for class in data::CLASSES {
            let reward = gear.award_primal(&mut rng, class).unwrap();
            assert!(!reward.is_empty());
            assert_eq!(gear.bag().len(), BAG_CAP);
            let item = gear.bag().last().unwrap();
            assert_eq!(item.rarity, 5);
            assert_eq!(item.enhancements.len(), ENH_COUNTS[5]);
            assert!(gear.dirty);
            assert!(gear.save_elapsed >= 5.0);
            let serialized = toml::to_string(&gear.save).unwrap();
            let restored: Save = toml::from_str(&serialized).unwrap();
            assert_eq!(restored.bag.last().unwrap().id, item.id);
            assert_eq!(restored.bag.last().unwrap().rarity, 5);
        }
    }

    #[test]
    fn unreadable_gear_never_silently_claims_final_reward_success() {
        let mut gear = Gear {
            read_only: true,
            ..default()
        };
        assert!(gear.award_primal(&mut Rng(1), class(0)).is_err());
        assert!(gear.bag().is_empty());
        assert!(!gear.dirty);
    }

    fn class(i: usize) -> &'static data::ClassDef {
        &data::CLASSES[i]
    }

    #[test]
    fn affix_counts_follow_rarity() {
        let mut rng = Rng::default();
        for rarity in 0..RARITY_COUNT as u8 {
            for _ in 0..50 {
                let item = roll_item(&mut rng, class(0), rarity);
                assert_eq!(item.affixes.len(), AFFIX_COUNTS[usize::from(rarity)]);
                let mut kinds: Vec<u8> = item.affixes.iter().map(|r| r.kind).collect();
                kinds.sort_unstable();
                kinds.dedup();
                assert_eq!(kinds.len(), item.affixes.len(), "affixes must be distinct");
                for roll in &item.affixes {
                    let def = affix(roll.kind);
                    assert!((def.lo..=def.hi).contains(&roll.value));
                }
                assert_eq!(item.enhancements.len(), ENH_COUNTS[usize::from(rarity)]);
                assert!(item.enhancement.is_none());
            }
        }
    }

    #[test]
    fn enhancement_targets_the_droppers_ability() {
        let mut rng = Rng(12345);
        for c in data::CLASSES {
            for _ in 0..1000 {
                let item = roll_item(&mut rng, c, 5);
                for e in item.enhancements {
                    assert!(e.ability.is_empty() || c.pool.contains(&e.ability.as_str()));
                    let kind = ENH_KINDS[usize::from(e.kind)];
                    if !e.ability.is_empty() {
                        assert!(kind.supports(data::ability(&e.ability)));
                    }
                    let (lo, hi) = kind.roll_range(e.ability.is_empty());
                    assert!((lo..=hi).contains(&e.value));
                }
            }
        }
    }

    #[test]
    fn normal_drop_rate_is_two_per_thousand_and_bosses_drop_once() {
        let mut rng = Rng(12345);
        assert_eq!(NORMAL_DROP_CHANCE, 0.002);
        let drops = (0..100_000)
            .filter_map(|_| roll_drop_rarity(&mut rng, false))
            .count();
        assert!((150..=250).contains(&drops), "got {drops} drops");
        for _ in 0..1000 {
            assert!(matches!(roll_drop_rarity(&mut rng, true), Some(1..=5)));
        }
    }

    fn enhanced_item(ability: &str, kind: EnhKind, value: u32) -> Item {
        Item {
            id: 0,
            slot: 0,
            rarity: 5,
            base: 0,
            class: 1,
            affixes: Vec::new(),
            enhancement: Some(EnhRoll {
                ability: ability.to_owned(),
                kind: ENH_KINDS.iter().position(|k| *k == kind).unwrap() as u8,
                value,
            }),
            enhancements: Vec::new(),
        }
    }

    #[test]
    fn every_supported_enhancement_changes_the_corresponding_cast_parameter() {
        for def in data::ABILITIES {
            for kind in ENH_KINDS.into_iter().take(5) {
                if !kind.supports(def) {
                    continue;
                }
                let value = if kind == EnhKind::Count { 10_000 } else { 2500 };
                let mut gear = Gear::default();
                let id = gear.add(enhanced_item(def.key, kind, value));
                assert!(gear.equip(1, id));
                let mut run = Run::default();
                let plain = super::super::talents::cast(def, 1, &run.stats());
                run.gear = gear.bonuses(1);
                let enhanced = super::super::talents::cast(def, 1, &run.stats());
                let close = |a: f32, b: f32| assert!((a - b).abs() < 0.0001, "{}", def.key);
                close(
                    enhanced.dmg,
                    plain.dmg * if kind == EnhKind::Damage { 1.25 } else { 1.0 },
                );
                close(
                    enhanced.dot,
                    plain.dot * if kind == EnhKind::Damage { 1.25 } else { 1.0 },
                );
                close(
                    enhanced.cooldown,
                    plain.cooldown * if kind == EnhKind::Cooldown { 0.75 } else { 1.0 },
                );
                assert_eq!(
                    enhanced.count,
                    plain.count + u32::from(kind == EnhKind::Count)
                );
                close(
                    enhanced.radius,
                    plain.radius * if kind == EnhKind::Radius { 1.25 } else { 1.0 },
                );
                close(
                    enhanced.duration,
                    plain.duration * if kind == EnhKind::Duration { 1.25 } else { 1.0 },
                );
                close(enhanced.range, plain.range);
            }
        }
    }

    #[test]
    fn gear_sync_stacks_enhancements_only_for_the_worn_class_and_target_skill() {
        let mut gear = Gear::default();
        let first = gear.add(enhanced_item("frostbolt", EnhKind::Damage, 5000));
        let mut second = enhanced_item("frostbolt", EnhKind::Damage, 2500);
        second.slot = 1;
        let second = gear.add(second);
        gear.equip(1, first);
        gear.equip(1, second);
        let mut app = App::new();
        app.insert_resource(gear)
            .init_resource::<Run>()
            .add_systems(Update, sync_run);
        let frost = data::ability("frostbolt");
        let fire = data::ability("fireball");
        let damage = |app: &App, def| {
            super::super::talents::cast(def, 1, &app.world().resource::<Run>().stats()).dmg
        };
        let base = damage(&app, frost);
        let other = damage(&app, fire);
        app.update();
        assert_eq!(damage(&app, frost), base * 1.75);
        assert_eq!(damage(&app, fire), other);
        app.world_mut().resource_mut::<Gear>().unequip(1, 0);
        app.update();
        assert_eq!(damage(&app, frost), base * 1.25);
        app.world_mut().resource_mut::<Gear>().equip(9, second);
        app.update();
        assert_eq!(damage(&app, frost), base);
    }

    #[test]
    fn incompatible_legacy_enhancements_are_repaired_once_without_losing_items() {
        let cases = [
            ("frostbolt", EnhKind::Duration, 4000, 3000),
            ("frost_nova", EnhKind::Count, 10_000, 3000),
            ("divine_shield", EnhKind::Damage, 5000, 3000),
            ("fire_blast", EnhKind::Radius, 2000, 2000),
            ("searing_totem", EnhKind::Count, 10_000, 3000),
        ];
        let mut gear = Gear::default();
        for (ability, kind, value, _) in cases {
            gear.add(enhanced_item(ability, kind, value));
        }
        let valid = gear.add(enhanced_item("fireball", EnhKind::Damage, 5000));
        gear.equip(1, valid);
        assert_eq!(gear.save.repair_enhancements(), cases.len());
        assert_eq!(gear.save.repair_enhancements(), 0);
        assert_eq!(gear.bag().len(), cases.len() + 1);
        for (item, (ability, _, _, expected)) in gear.bag().iter().zip(cases) {
            let e = item.enhancement.as_ref().unwrap();
            assert_eq!(e.ability, ability);
            assert!(ENH_KINDS[usize::from(e.kind)] == EnhKind::Cooldown);
            assert_eq!(e.value, expected);
            let mut run = Run::default();
            let def = data::ability(ability);
            let plain = super::super::talents::cast(def, 1, &run.stats());
            run.gear.enh[0] = Some(SkillRoll {
                item: item.id,
                index: 0,
                ability: Some(def.key),
                kind: EnhKind::Cooldown,
                value: e.value as f32 / SCALE,
            });
            assert!(super::super::talents::cast(def, 1, &run.stats()).cooldown < plain.cooldown);
        }
        assert_eq!(gear.worn(1, 0).unwrap().id, valid);
        assert_eq!(
            gear.item(valid)
                .unwrap()
                .enhancement
                .as_ref()
                .unwrap()
                .value,
            5000
        );
    }

    #[test]
    fn affix_ranges_are_doubled_and_counts_are_halved() {
        assert_eq!(AFFIX_COUNTS, [1, 2, 3, 4, 5, 5]);
        let original = [
            (200, 600),
            (300, 800),
            (200, 500),
            (1000, 4000),
            (100, 400),
            (200, 600),
            (400, 1200),
            (200, 600),
            (3000, 10000),
            (50, 200),
            (300, 1000),
            (10, 40),
            (50, 200),
            (100, 400),
        ];
        for (def, (lo, hi)) in AFFIXES.iter().zip(original) {
            assert_eq!((def.lo, def.hi), (lo * 2, hi * 2));
        }
        assert_eq!(EnhKind::Damage.range(), (2500, 5000));
        assert_eq!(EnhKind::Cooldown.range(), (1500, 3000));
        assert_eq!(EnhKind::Count.range(), (10_000, 20_000));
        assert_eq!(EnhKind::Radius.range(), (2000, 4000));
        assert_eq!(EnhKind::Duration.range(), (2000, 4000));
    }

    #[test]
    fn legacy_affixes_upgrade_in_order_with_double_values_only_once() {
        let mut gear = Gear::default();
        for (rarity, count) in [2, 4, 6, 8, 10, 10].into_iter().enumerate() {
            let mut item = enhanced_item("fireball", EnhKind::Damage, 5000);
            item.rarity = rarity as u8;
            item.slot = rarity as u8;
            item.affixes = (0..count)
                .map(|kind| Roll {
                    kind,
                    value: AFFIXES[usize::from(kind)].lo / 2,
                })
                .collect();
            if rarity != 5 {
                item.enhancement = None;
            }
            let id = gear.add(item);
            gear.equip(1, id);
        }
        let worn = gear.save.worn.clone();
        let old = gear.save.bag.clone();
        gear.save.version = 1;
        gear.save.upgrade_affixes().unwrap();
        assert_eq!(gear.save.version, 2);
        assert_eq!(gear.save.worn, worn);
        for (before, after) in old.iter().zip(&gear.save.bag) {
            assert_eq!(before.id, after.id);
            assert_eq!(before.enhancement, after.enhancement);
            assert_eq!(after.affixes.len(), AFFIX_COUNTS[usize::from(after.rarity)]);
            for (a, b) in before.affixes.iter().zip(&after.affixes) {
                assert_eq!(a.kind, b.kind);
                assert_eq!(a.value * 2, b.value);
            }
        }
        let encoded = toml::to_string(&gear.save).unwrap();
        let mut reloaded: Save = toml::from_str(&encoded).unwrap();
        reloaded.upgrade_affixes().unwrap();
        assert_eq!(toml::to_string(&reloaded).unwrap(), encoded);
    }

    #[test]
    fn skill_rolls_follow_quality_scopes_and_uniqueness_rules() {
        let mut rng = Rng(12345);
        let mut globals = 0;
        let mut targeted = 0;
        for class in data::CLASSES {
            for rarity in 0..6 {
                for _ in 0..100 {
                    let item = roll_item(&mut rng, class, rarity);
                    assert_eq!(item.enhancements.len(), ENH_COUNTS[usize::from(rarity)]);
                    let mut unique = BTreeMap::new();
                    let mut skills = BTreeMap::new();
                    let mut buffs = 0;
                    for e in item.enhancements {
                        assert!(unique.insert((e.ability.clone(), e.kind), ()).is_none());
                        let kind = ENH_KINDS[usize::from(e.kind)];
                        buffs += usize::from(kind.is_buff());
                        if e.ability.is_empty() {
                            globals += 1;
                            assert!(class
                                .pool
                                .iter()
                                .any(|key| kind.supports(data::ability(key))));
                        } else {
                            targeted += 1;
                            skills.insert(e.ability.clone(), ());
                        }
                        let (lo, hi) = kind.roll_range(e.ability.is_empty());
                        assert!((lo..=hi).contains(&e.value));
                        if kind == EnhKind::Count {
                            assert_eq!(e.value % 10_000, 0);
                        }
                    }
                    assert!(skills.len() <= 2);
                    assert!(buffs <= 1);
                }
            }
        }
        assert!(globals > 0 && targeted > globals);
    }

    #[test]
    fn legacy_skill_migration_adds_quality_rolls_preserves_old_effect_and_is_idempotent() {
        let class = data::CLASSES.iter().find(|c| c.id == 8).unwrap();
        let mut rng = Rng(12345);
        let mut save = Save {
            version: 2,
            ..default()
        };
        for rarity in 0..6 {
            let mut item = roll_item(&mut rng, class, rarity);
            item.id = u64::from(rarity) + 1;
            item.enhancements.clear();
            if rarity == 5 {
                item.enhancement = Some(EnhRoll {
                    ability: "fireball".into(),
                    kind: 0,
                    value: 5000,
                });
            }
            save.bag.push(item);
        }
        let before = save.bag.clone();
        save.upgrade_skills();
        assert_eq!(save.version, 3);
        for (old, new) in before.iter().zip(&save.bag) {
            assert_eq!(new.id, old.id);
            assert_eq!(new.affixes, old.affixes);
            assert_eq!(new.enhancements.len(), ENH_COUNTS[usize::from(new.rarity)]);
            assert!(new.enhancement.is_none());
            if let Some(old) = &old.enhancement {
                assert_eq!(&new.enhancements[0], old);
            }
        }
        let encoded = toml::to_string(&save).unwrap();
        save.upgrade_skills();
        assert_eq!(toml::to_string(&save).unwrap(), encoded);
        let mut retried = Save {
            version: 2,
            bag: before,
            ..default()
        };
        retried.upgrade_skills();
        assert_eq!(retried.bag, save.bag);
    }

    #[test]
    fn every_base_has_an_icon_and_names() {
        for list in BASES {
            assert!(!list.is_empty());
            for (en, zh, icon) in list {
                assert!(!en.is_empty() && !zh.is_empty());
                assert!(icon.starts_with("Interface\\Icons\\INV_"));
            }
        }
    }

    #[test]
    fn rarity_weights_respect_the_tables() {
        let mut rng = Rng::default();
        for _ in 0..2000 {
            assert!(roll_rarity(&mut rng, true) >= 1, "bosses never drop common");
        }
        let primal = (0..20000)
            .filter(|_| roll_rarity(&mut rng, false) == 5)
            .count();
        assert!(primal < 400);
    }

    #[test]
    fn every_base_has_a_look_entry() {
        assert_eq!(MAIN_LOOKS.len(), BASES[MAIN_SLOT].len());
        assert_eq!(OFF_LOOKS.len(), BASES[OFF_SLOT].len());
    }

    #[test]
    fn worn_weapons_replace_the_class_default() {
        let mut gear = Gear::default();
        let warlock = data::CLASSES.iter().find(|c| c.id == 9).unwrap();
        let id = gear.add(Item {
            id: 0,
            slot: MAIN_SLOT as u8,
            rarity: 0,
            base: 3,
            class: warlock.id,
            affixes: Vec::new(),
            enhancement: None,
            enhancements: Vec::new(),
        });
        gear.equip(warlock.id, id);
        let weapons = gear.weapons(warlock);
        assert_eq!(weapons.len(), 1);
        assert_eq!(weapons[0].1, 1550);
        assert_eq!(Gear::default().weapons(warlock), warlock.weapons.to_vec());
    }

    #[test]
    fn save_round_trips() {
        let mut rng = Rng::default();
        let mut gear = Gear::default();
        for r in 0..RARITY_COUNT as u8 {
            let id = gear.add(roll_item(&mut rng, class(0), r));
            assert!(gear.equip(9, id));
        }
        let text = toml::to_string(&gear.save).unwrap();
        let back: Save = toml::from_str(&text).unwrap();
        assert_eq!(back.bag, gear.save.bag);
        assert_eq!(back.worn, gear.save.worn);
    }

    #[test]
    fn an_item_is_worn_by_one_class_at_a_time() {
        let mut rng = Rng::default();
        let mut gear = Gear::default();
        let id = gear.add(roll_item(&mut rng, class(0), 2));
        let slot = usize::from(gear.item(id).unwrap().slot);
        assert!(gear.equip(1, id));
        assert!(gear.equip(9, id));
        assert!(gear.worn(1, slot).is_none());
        assert_eq!(gear.worn(9, slot).map(|i| i.id), Some(id));
        assert_eq!(gear.wearer(id), Some(9));
        gear.discard(id);
        assert!(gear.worn(9, slot).is_none() && gear.item(id).is_none());
    }

    #[test]
    fn a_full_bag_discards_the_lowest_unworn_item() {
        let mut rng = Rng::default();
        let mut gear = Gear::default();
        let kept = gear.add(roll_item(&mut rng, class(0), 0));
        gear.equip(9, kept);
        for _ in 0..BAG_CAP - 1 {
            gear.add(roll_item(&mut rng, class(0), 1));
        }
        assert_eq!(gear.bag().len(), BAG_CAP);
        let fresh = gear.add(roll_item(&mut rng, class(0), 3));
        assert_eq!(gear.bag().len(), BAG_CAP);
        assert!(gear.item(kept).is_some(), "worn items survive");
        assert!(gear.item(fresh).is_some());
    }

    #[test]
    fn bonuses_sum_the_worn_rolls() {
        let mut rng = Rng::default();
        let mut gear = Gear::default();
        let id = gear.add(roll_item(&mut rng, class(0), 4));
        gear.equip(9, id);
        let b = gear.bonuses(9);
        let item = gear.item(id).unwrap();
        for roll in &item.affixes {
            assert!(b.sum[usize::from(roll.kind)] >= roll.value as f32 / SCALE - 1e-4);
        }
        assert!(gear.bonuses(1).sum.iter().all(|v| *v == 0.0));
    }
}
