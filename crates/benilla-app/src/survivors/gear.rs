//! Survivors-only gear: random drops with rolled affixes, kept between runs.
//! Nothing here touches the original client's item pipeline.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::{Phase, Rng, Run, Stats, data, localization};

pub(super) const BAG_CAP: usize = 200;
const SCALE: f32 = 10_000.0;

pub(super) const RARITY_COUNT: usize = 6;
pub(super) const SLOT_COUNT: usize = 9;
const MAIN_SLOT: usize = 7;
const OFF_SLOT: usize = 8;

/// Affix counts per rarity; a primal also carries one skill enhancement.
pub(super) const AFFIX_COUNTS: [usize; RARITY_COUNT] = [2, 4, 6, 8, 10, 10];

const NORMAL_WEIGHTS: [f32; RARITY_COUNT] = [50.0, 30.0, 13.0, 5.0, 1.6, 0.4];
const BOSS_WEIGHTS: [f32; RARITY_COUNT] = [0.0, 20.0, 38.0, 28.0, 11.0, 3.0];
pub(super) const NORMAL_DROP_CHANCE: f32 = 0.008;
pub(super) const BOSS_EXTRA_CHANCE: f32 = 0.25;

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
    af("Damage", "伤害", 200, 600, true),
    af("Max Health", "最大生命", 300, 800, true),
    af("Move Speed", "移动速度", 200, 500, true),
    af("Health Regen /s", "每秒生命回复", 1000, 4000, false),
    af("Haste", "急速", 100, 400, true),
    af("Area", "范围", 200, 600, true),
    af("Pickup Range", "拾取范围", 400, 1200, true),
    af("Experience", "经验获取", 200, 600, true),
    af("Armor", "护甲", 3000, 10000, false),
    af("Crit Chance", "暴击几率", 50, 200, true),
    af("Crit Damage", "暴击伤害", 300, 1000, true),
    af("Life Leech", "生命偷取", 10, 40, true),
    af("Damage Reduction", "伤害减免", 50, 200, true),
    af("Mobility Cooldown", "位移冷却缩减", 100, 400, true),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum EnhKind {
    Damage,
    Cooldown,
    Count,
    Radius,
    Duration,
}

pub(super) const ENH_KINDS: [EnhKind; 5] = [
    EnhKind::Damage,
    EnhKind::Cooldown,
    EnhKind::Count,
    EnhKind::Radius,
    EnhKind::Duration,
];

impl EnhKind {
    /// Roll range in 1/10000 units; a count is whole numbers.
    fn range(self) -> (u32, u32) {
        match self {
            Self::Damage => (2500, 5000),
            Self::Cooldown => (1500, 3000),
            Self::Count => (10_000, 10_000),
            Self::Radius | Self::Duration => (2000, 4000),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Damage => localization::text("Damage", "伤害"),
            Self::Cooldown => localization::text("Cooldown", "冷却时间"),
            Self::Count => localization::text("Projectiles / Hits", "数量"),
            Self::Radius => localization::text("Area", "范围"),
            Self::Duration => localization::text("Duration", "持续时间"),
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
    format!(
        "{ability_name}: {} {}",
        kind.label(),
        kind.show(enh.value as f32 / SCALE)
    )
}

/// The derived effect of everything a class wears, folded into [`Stats`].
#[derive(Clone, Copy, Default)]
pub(super) struct Bonuses {
    sum: [f32; AFFIX_COUNT],
    enh: [Option<(&'static str, EnhKind, f32)>; SLOT_COUNT],
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
        self.enh
            .iter()
            .flatten()
            .filter(move |(key, _, _)| *key == ability)
            .map(|&(_, kind, value)| (kind, value))
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

pub(super) fn roll_rarity(rng: &mut Rng, boss: bool) -> u8 {
    let weights = if boss { BOSS_WEIGHTS } else { NORMAL_WEIGHTS };
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

fn roll_value(rng: &mut Rng, lo: u32, hi: u32, primal: bool) -> u32 {
    let one = |rng: &mut Rng| lo + (rng.f32() * (hi - lo + 1) as f32) as u32;
    let v = one(rng).min(hi);
    if primal { v.max(one(rng).min(hi)) } else { v }
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

    let enhancement = primal.then(|| {
        let ability = class.pool[rng.index(class.pool.len())];
        let kind = rng.index(ENH_KINDS.len());
        let (lo, hi) = ENH_KINDS[kind].range();
        EnhRoll {
            ability: ability.to_owned(),
            kind: kind as u8,
            value: roll_value(rng, lo, hi, true),
        }
    });

    Item {
        id: 0,
        slot: slot as u8,
        rarity,
        base: base as u8,
        class: class.id,
        affixes,
        enhancement,
    }
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
            version: 1,
            next_id: 1,
            bag: Vec::new(),
            worn: BTreeMap::new(),
        }
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
            let found = item.enhancement.as_ref().and_then(|e| {
                data::ABILITIES
                    .iter()
                    .find(|a| a.key == e.ability)
                    .map(|d| (e, d))
            });
            if let Some((e, def)) = found {
                let kind = ENH_KINDS[usize::from(e.kind).min(ENH_KINDS.len() - 1)];
                out.enh[enh] = Some((def.key, kind, e.value as f32 / SCALE));
                enh += 1;
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
            Ok(save) if save.version == 1 => gear.save = save,
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
    run.gear = bonuses;
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Gear>()
        .add_systems(Startup, load)
        .add_systems(OnEnter(Phase::Playing), sync_run)
        .add_systems(
            Update,
            (sync_run.run_if(resource_changed::<Gear>), persist).chain(),
        );
}

#[cfg(test)]
mod tests {
    use super::*;

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
                assert_eq!(item.enhancement.is_some(), usize::from(rarity) == 5);
            }
        }
    }

    #[test]
    fn enhancement_targets_the_droppers_ability() {
        let mut rng = Rng::default();
        for c in data::CLASSES {
            let item = roll_item(&mut rng, c, 5);
            let e = item.enhancement.expect("primal");
            assert!(c.pool.contains(&e.ability.as_str()));
            let (lo, hi) = ENH_KINDS[usize::from(e.kind)].range();
            assert!((lo..=hi).contains(&e.value));
        }
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
