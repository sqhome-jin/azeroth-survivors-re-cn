//! Class-local talent text and unconditional modifiers. Stateful procs live in the runtime.

use super::{
    talents::{Cast, Loadout, NodeDef},
    Stats,
};

pub(super) struct ClassTalents {
    pub trees: [(&'static str, &'static str); 3],
    pub backgrounds: [[&'static str; 4]; 3],
    pub nodes: [NodeDef; 24],
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

macro_rules! bg {
    ($base:literal) => {
        [
            concat!("Interface\\TalentFrame\\", $base, "-TopLeft"),
            concat!("Interface\\TalentFrame\\", $base, "-TopRight"),
            concat!("Interface\\TalentFrame\\", $base, "-BottomLeft"),
            concat!("Interface\\TalentFrame\\", $base, "-BottomRight"),
        ]
    };
}

static WARRIOR: ClassTalents = ClassTalents {
    trees: [("Arms", "武器"), ("Fury", "狂怒"), ("Protection", "防护")],
    backgrounds: [
        bg!("WarriorArms"),
        bg!("WarriorFury"),
        bg!("WarriorProtection"),
    ],
    nodes: [
        node!(
            "improved_heroic_strike",
            "Improved Heroic Strike",
            "强化英勇打击",
            "Ability_Rogue_Ambush",
            "Heroic Strike damage +6% per rank.",
            "每级使英勇打击伤害提高6%。"
        ),
        node!(
            "improved_mortal_strike",
            "Improved Mortal Strike",
            "强化致死打击",
            "Ability_Warrior_SavageBlow",
            "Mortal Strike damage +6% per rank.",
            "每级使致死打击伤害提高6%。"
        ),
        node!(
            "weapon_mastery",
            "Weapon Mastery",
            "武器掌握",
            "Ability_Warrior_SavageBlow",
            "Heroic Strike and Mortal Strike cooldown -4% per rank.",
            "每级使英勇打击和致死打击冷却缩短4%。"
        ),
        node!(
            "sweeping_reach",
            "Sweeping Reach",
            "横扫延伸",
            "Ability_Warrior_Cleave",
            "Cleave range +0.4 yards per rank.",
            "每级使顺劈斩攻击距离增加0.4码。"
        ),
        node!(
            "improved_execute",
            "Improved Execute",
            "强化斩杀",
            "INV_Sword_48",
            "Execute damage +6% per rank.",
            "每级使斩杀伤害提高6%。"
        ),
        node!(
            "deep_wounds",
            "Deep Wounds",
            "重伤",
            "Ability_BackStab",
            "Heroic Strike and Mortal Strike primary direct critical hits cause a 4-second bleed totaling 8% of the triggering damage per rank.",
            "英勇打击和致死打击的主要直接伤害暴击附加4秒流血，总伤害为触发伤害的每级8%。"
        ),
        node!(
            "blood_taste",
            "Blood Taste",
            "血之气息",
            "Spell_Shadow_BloodBoil",
            "Cleave, Heroic Strike, Mortal Strike and Execute primary direct damage +4% per rank against bleeding targets.",
            "近战攻击（含嗜血、压制、猛击、盾牌猛击和盾击）对流血目标的主要直接伤害每级提高4%。"
        ),
        node!(
            "sudden_death",
            "Sudden Death",
            "猝死",
            "INV_Sword_48",
            "Cleave, Heroic Strike, Mortal Strike and Execute primary direct critical hits have a 20% chance to reset learned Execute. Shared 6-second internal cooldown; no extra cast.",
            "顺劈斩、英勇打击、致死打击和斩杀的主要直接伤害暴击有20%概率刷新已学斩杀；共用6秒内置冷却，不额外施法。"
        ),
        node!(
            "improved_cleave",
            "Improved Cleave",
            "强化顺劈斩",
            "Ability_Warrior_Cleave",
            "Cleave damage +6% per rank.",
            "每级使顺劈斩伤害提高6%。"
        ),
        node!(
            "improved_whirlwind",
            "Improved Whirlwind",
            "强化旋风斩",
            "Ability_Whirlwind",
            "Whirlwind damage +6% per rank.",
            "每级使旋风斩伤害提高6%。"
        ),
        node!(
            "whirlwind_mastery",
            "Whirlwind Mastery",
            "旋风掌握",
            "Ability_Whirlwind",
            "Whirlwind cooldown -4% per rank.",
            "每级使旋风斩冷却缩短4%。"
        ),
        node!(
            "sweeping_power",
            "Sweeping Power",
            "横扫之力",
            "Ability_Rogue_SliceDice",
            "Sweeping Strikes damage +6% per rank.",
            "每级使横扫攻击伤害提高6%。"
        ),
        node!(
            "bounding_fury",
            "Bounding Fury",
            "狂怒突进",
            "Ability_Warrior_Charge",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "flurry",
            "Flurry",
            "乱舞",
            "Ability_GhoulFrenzy",
            "Cleave, Heroic Strike, Mortal Strike and Execute primary direct critical hits grant 3% haste per rank for 4 seconds. Shared 6-second internal cooldown.",
            "顺劈斩、英勇打击、致死打击和斩杀的主要直接伤害暴击使急速每级提高3%，持续4秒；共用6秒内置冷却。"
        ),
        node!(
            "bloodthirst",
            "Bloodthirst",
            "嗜血",
            "Spell_Nature_BloodLust",
            "Cleave, Heroic Strike, Mortal Strike and Execute primary direct damage heals for 0.4% of effective damage per rank. All talent leech shares a cap of 4% maximum health per second.",
            "顺劈斩、英勇打击、致死打击和斩杀的主要直接有效伤害每级吸血0.4%；所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "recklessness",
            "Recklessness",
            "鲁莽",
            "Ability_CriticalStrike",
            "F grants +20 percentage points of critical chance for 5 seconds.",
            "施放F后暴击率提高20个百分点，持续5秒。"
        ),
        node!(
            "defensive_stance",
            "Defensive Stance",
            "防御姿态",
            "Ability_Warrior_DefensiveStance",
            "Incoming damage -1% per rank.",
            "每级使所受伤害降低1%。"
        ),
        node!(
            "iron_skin",
            "Iron Skin",
            "钢铁皮肤",
            "Spell_Nature_MirrorImage",
            "Armor +0.4 per rank.",
            "每级增加0.4护甲。"
        ),
        node!(
            "vitality",
            "Vitality",
            "活力",
            "Spell_Nature_AncestralGuardian",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "improved_thunder_clap",
            "Improved Thunder Clap",
            "强化雷霆一击",
            "Spell_Nature_ThunderClap",
            "Thunder Clap radius +4% per rank.",
            "每级使雷霆一击半径扩大4%。"
        ),
        node!(
            "booming_voice",
            "Booming Voice",
            "震耳嗓音",
            "Ability_Warrior_WarCry",
            "Demoralizing Shout weakness duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使挫志怒吼虚弱持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "concussive_retaliation",
            "Concussive Retaliation",
            "震荡反击",
            "Ability_Warrior_Revenge",
            "Taking contact damage reduces learned Thunder Clap's remaining cooldown by 0.2 seconds per rank. Shared 1-second internal cooldown.",
            "受到接触伤害使已学雷霆一击剩余冷却每级缩短0.2秒；共用1秒内置冷却。"
        ),
        node!(
            "toughness",
            "Toughness",
            "坚韧",
            "Spell_Holy_Devotion",
            "While below 35% health, incoming damage -2% per rank.",
            "生命低于35%时，每级使所受伤害降低2%。"
        ),
        node!(
            "shield_wall",
            "Shield Wall",
            "盾墙",
            "Ability_Warrior_ShieldWall",
            "Crossing below 30% health grants 40% damage reduction for 4 seconds. 30-second internal cooldown; does not rescue a lethal hit.",
            "生命降至30%以下时获得40%减伤，持续4秒；内置冷却30秒，不抵消致命伤害。"
        ),
    ],
};

static PALADIN: ClassTalents = ClassTalents {
    trees: [
        ("Holy", "神圣"),
        ("Protection", "防护"),
        ("Retribution", "惩戒"),
    ],
    backgrounds: [
        bg!("PaladinHoly"),
        bg!("PaladinProtection"),
        bg!("PaladinCombat"),
    ],
    nodes: [
        node!(
            "improved_holy_shock",
            "Improved Holy Shock",
            "强化神圣震击",
            "Spell_Holy_SearingLight",
            "Holy Shock damage +6% per rank.",
            "每级使神圣震击伤害提高6%。"
        ),
        node!(
            "holy_focus",
            "Holy Focus",
            "神圣专注",
            "Spell_Holy_SearingLightPriest",
            "Holy Shock cooldown -4% per rank.",
            "每级使神圣震击冷却缩短4%。"
        ),
        node!(
            "improved_exorcism",
            "Improved Exorcism",
            "强化驱邪术",
            "Spell_Holy_Excorcism_02",
            "Exorcism damage +6% per rank.",
            "每级使驱邪术伤害提高6%。"
        ),
        node!(
            "improved_holy_wrath",
            "Improved Holy Wrath",
            "强化神圣愤怒",
            "Spell_Holy_Excorcism",
            "Holy Wrath damage +6% per rank.",
            "每级使神圣愤怒伤害提高6%。"
        ),
        node!(
            "holy_reach",
            "Holy Reach",
            "神圣延伸",
            "Spell_Holy_Purify",
            "Holy Shock and Exorcism range +1 yard per rank.",
            "每级使神圣震击和驱邪术射程增加1码。"
        ),
        node!(
            "holy_mending",
            "Holy Mending",
            "神圣修复",
            "Spell_Holy_HolyBolt",
            "Holy Shock primary direct damage heals for 1% of effective damage per rank. All talent leech shares a cap of 4% maximum health per second.",
            "神圣震击的主要直接有效伤害每级吸血1%；所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "infusion_of_light",
            "Infusion of Light",
            "圣光灌注",
            "Spell_Holy_Heal",
            "A Holy Shock primary direct critical hit empowers the next Hammer of Wrath cast within 6 seconds: damage +5% per rank. At most one proc per Holy Shock cast.",
            "神圣震击主要直接伤害暴击使6秒内下一次愤怒之锤伤害每级提高5%；每次神圣震击最多触发一次。"
        ),
        node!(
            "divine_favor",
            "Divine Favor",
            "神恩术",
            "Spell_Holy_Heal",
            "F makes the next Holy Shock cast within 6 seconds a guaranteed critical hit.",
            "施放F后，6秒内下一次神圣震击必定暴击。"
        ),
        node!(
            "sacred_vitality",
            "Sacred Vitality",
            "神圣活力",
            "Spell_Holy_BlessingOfStamina",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "sanctuary",
            "Sanctuary",
            "庇护",
            "Spell_Holy_BlessingOfProtection",
            "Incoming damage -1% per rank.",
            "每级使所受伤害降低1%。"
        ),
        node!(
            "devotion",
            "Devotion",
            "虔诚",
            "Spell_Holy_DevotionAura",
            "Armor +0.4 per rank.",
            "每级增加0.4护甲。"
        ),
        node!(
            "consecrated_ground",
            "Consecrated Ground",
            "神圣之地",
            "Spell_Holy_InnerFire",
            "Consecration radius +4% per rank.",
            "每级使奉献半径扩大4%。"
        ),
        node!(
            "righteous_stride",
            "Righteous Stride",
            "正义步伐",
            "Spell_Holy_SealOfFury",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "hallowed_ground",
            "Hallowed Ground",
            "祝圣之地",
            "Spell_Holy_InnerFire",
            "While inside your own Consecration area, incoming damage -2% per rank.",
            "位于自己的奉献区域内时，每级使所受伤害降低2%。"
        ),
        node!(
            "sacred_shield",
            "Sacred Shield",
            "圣洁护盾",
            "Spell_Holy_PowerWordShield",
            "Consecration effective damage adds a shield equal to 0.5% of that damage per rank, lasting 8 seconds. Total shield is capped at 10% maximum health; refreshes cannot exceed that cap.",
            "奉献有效伤害的每级0.5%转化为8秒护盾；护盾总量最多为生命上限的10%，刷新或叠加也不能超过上限。"
        ),
        node!(
            "ardent_defender",
            "Ardent Defender",
            "炽热防御者",
            "Spell_Holy_BlessingOfProtection",
            "While below 35% health, incoming damage -20%.",
            "生命低于35%时，所受伤害降低20%。"
        ),
        node!(
            "improved_hammer_of_wrath",
            "Improved Hammer of Wrath",
            "强化愤怒之锤",
            "Ability_ThunderClap",
            "Hammer of Wrath damage +6% per rank.",
            "每级使愤怒之锤伤害提高6%。"
        ),
        node!(
            "improved_judgement",
            "Improved Judgement",
            "强化审判",
            "Spell_Holy_RighteousFury",
            "Judgement damage +6% per rank.",
            "每级使审判伤害提高6%。"
        ),
        node!(
            "swift_retribution",
            "Swift Retribution",
            "迅捷惩戒",
            "Spell_Holy_AuraOfLight",
            "Hammer of Wrath and Judgement cooldown -4% per rank.",
            "每级使愤怒之锤和审判冷却缩短4%。"
        ),
        node!(
            "wrathful_prayer",
            "Wrathful Prayer",
            "愤怒祷言",
            "Spell_Holy_Excorcism",
            "Holy Wrath cooldown -4% per rank.",
            "每级使神圣愤怒冷却缩短4%。"
        ),
        node!(
            "improved_hammer_of_justice",
            "Improved Hammer of Justice",
            "强化制裁之锤",
            "Spell_Holy_SealOfMight",
            "Hammer of Justice stun duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使制裁之锤昏迷持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "judgement_of_the_crusader",
            "Judgement of the Crusader",
            "十字军审判",
            "Spell_Holy_HolySmite",
            "Judgement marks its target for 6 seconds: your Hammer of Wrath, Holy Shock, Exorcism, Holy Wrath, Hammer of Justice and Judgement primary direct damage +3% per rank. Applies after the triggering hit; refreshes, never stacks.",
            "审判标记目标6秒，使你的愤怒之锤、神圣震击、驱邪术、神圣愤怒、制裁之锤和审判主要直接伤害每级提高3%；触发命中不受益，仅刷新、不叠加。"
        ),
        node!(
            "vengeance",
            "Vengeance",
            "复仇",
            "Ability_Racial_Avatar",
            "Hammer of Wrath, Holy Shock, Exorcism, Holy Wrath, Hammer of Justice and Judgement primary direct critical hits grant +3% Holy damage per rank for 4 seconds. Shared 6-second internal cooldown.",
            "愤怒之锤、神圣震击、驱邪术、神圣愤怒、制裁之锤和审判主要直接伤害暴击使神圣伤害每级提高3%，持续4秒；共用6秒内置冷却。"
        ),
        node!(
            "sanctified_wrath",
            "Sanctified Wrath",
            "圣洁怒火",
            "Spell_Holy_HolySmite",
            "Hammer of Wrath damage +40% against targets below 30% health, including bosses.",
            "愤怒之锤对生命低于30%的目标伤害提高40%，对首领同样有效。"
        ),
    ],
};

static HUNTER: ClassTalents = ClassTalents {
    trees: [
        ("Beast Mastery", "野兽控制"),
        ("Marksmanship", "射击"),
        ("Survival", "生存"),
    ],
    backgrounds: [
        bg!("HunterBeastMastery"),
        bg!("HunterMarksmanship"),
        bg!("HunterSurvival"),
    ],
    nodes: [
        node!(
            "improved_arcane_shot",
            "Improved Arcane Shot",
            "强化奥术射击",
            "Ability_ImpalingBolt",
            "Arcane Shot damage +6% per rank.",
            "每级使奥术射击伤害提高6%。"
        ),
        node!(
            "improved_serpent_sting",
            "Improved Serpent Sting",
            "强化毒蛇钉刺",
            "Ability_Hunter_Quickshot",
            "Serpent Sting direct and periodic damage +6% per rank.",
            "每级使毒蛇钉刺直接和持续伤害提高6%。"
        ),
        node!(
            "endurance_training",
            "Endurance Training",
            "耐久训练",
            "Spell_Nature_Reincarnation",
            "Maximum health +3% per rank; no pet required.",
            "每级使自身生命上限提高3%，不需要宠物。"
        ),
        node!(
            "pathfinding",
            "Pathfinding",
            "寻路",
            "Ability_Mount_JungleTiger",
            "Movement speed +2% per rank.",
            "每级使移动速度提高2%。"
        ),
        node!(
            "swift_escape",
            "Swift Escape",
            "迅捷脱离",
            "Ability_Rogue_Feint",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "rapid_instincts",
            "Rapid Instincts",
            "迅猛本能",
            "Ability_Hunter_BeastialDiscipline",
            "F grants 3% haste per rank for 5 seconds; no pet required.",
            "施放F后急速每级提高3%，持续5秒，不需要宠物。"
        ),
        node!(
            "predatory_mending",
            "Predatory Mending",
            "掠食疗愈",
            "Ability_Hunter_MendPet",
            "Arcane Shot, Multi-Shot, Aimed Shot and Concussive Shot primary direct damage heals for 0.4% of effective damage per rank. All talent leech shares a cap of 4% maximum health per second.",
            "奥术射击、多重射击、瞄准射击和震荡射击的主要直接有效伤害每级吸血0.4%；所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "bestial_wrath",
            "Bestial Wrath",
            "狂野怒火",
            "Ability_Druid_FerociousBite",
            "F grants +20% damage and 15% damage reduction for 5 seconds; no pet required.",
            "施放F后伤害提高20%，所受伤害降低15%，持续5秒，不需要宠物。"
        ),
        node!(
            "improved_aimed_shot",
            "Improved Aimed Shot",
            "强化瞄准射击",
            "INV_Spear_07",
            "Aimed Shot damage +6% per rank.",
            "每级使瞄准射击伤害提高6%。"
        ),
        node!(
            "barrage",
            "Barrage",
            "弹幕",
            "Ability_UpgradeMoonGlaive",
            "Multi-Shot damage +6% per rank.",
            "每级使多重射击伤害提高6%。"
        ),
        node!(
            "efficient_shots",
            "Efficient Shots",
            "高效射击",
            "Ability_Hunter_Quickshot",
            "Arcane Shot and Aimed Shot cooldown -4% per rank.",
            "每级使奥术射击和瞄准射击冷却缩短4%。"
        ),
        node!(
            "hawk_eye",
            "Hawk Eye",
            "鹰眼",
            "Ability_TownWatch",
            "Arcane Shot, Multi-Shot, Aimed Shot and Concussive Shot range +1 yard per rank.",
            "每级使奥术射击、多重射击、瞄准射击和震荡射击射程增加1码。"
        ),
        node!(
            "improved_volley",
            "Improved Volley",
            "强化乱射",
            "Ability_Marksmanship",
            "Volley radius +4% per rank.",
            "每级使乱射半径扩大4%。"
        ),
        node!(
            "hunters_mark",
            "Hunter's Mark",
            "猎人印记",
            "Ability_Hunter_SniperShot",
            "Aimed Shot marks its target for 6 seconds: your Arcane Shot, Multi-Shot, Aimed Shot and Concussive Shot primary direct damage +3% per rank. Triggering hit does not benefit; refreshes, never stacks.",
            "瞄准射击标记目标6秒，使你的奥术射击、多重射击、瞄准射击和震荡射击主要直接伤害每级提高3%；触发命中不受益，仅刷新、不叠加。"
        ),
        node!(
            "focused_aim",
            "Focused Aim",
            "专注瞄准",
            "Ability_Hunter_CriticalShot",
            "Arcane Shot primary direct critical hits reduce learned Aimed Shot's remaining cooldown by 0.2 seconds per rank. Shared 1-second internal cooldown.",
            "奥术射击主要直接伤害暴击使已学瞄准射击剩余冷却每级缩短0.2秒；共用1秒内置冷却。"
        ),
        node!(
            "wild_quiver",
            "Wild Quiver",
            "狂野箭袋",
            "Ability_Whirlwind",
            "Multi-Shot has one 20% chance per cast to add a hit for 50% damage. The extra hit cannot trigger another extra hit.",
            "多重射击每次施法有一次20%概率追加一次50%伤害的命中；追加命中不会递归触发。"
        ),
        node!(
            "trap_mastery",
            "Trap Mastery",
            "陷阱掌握",
            "Spell_Fire_SelfDestruct",
            "Explosive Trap damage +6% per rank.",
            "每级使爆炸陷阱伤害提高6%。"
        ),
        node!(
            "lasting_sting",
            "Lasting Sting",
            "持久钉刺",
            "Ability_Hunter_Quickshot",
            "Serpent Sting duration +6% per rank.",
            "每级使毒蛇钉刺持续时间延长6%。"
        ),
        node!(
            "survival_instincts",
            "Survival Instincts",
            "生存本能",
            "Spell_Nature_SpiritArmor",
            "Incoming damage -1% per rank.",
            "每级使所受伤害降低1%。"
        ),
        node!(
            "improved_concussive_shot",
            "Improved Concussive Shot",
            "强化震荡射击",
            "Spell_Frost_Stun",
            "Concussive Shot slow duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使震荡射击减速持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "resourcefulness",
            "Resourcefulness",
            "足智多谋",
            "Spell_Fire_SelfDestruct",
            "Explosive Trap cooldown -4% per rank.",
            "每级使爆炸陷阱冷却缩短4%。"
        ),
        node!(
            "noxious_stings",
            "Noxious Stings",
            "毒性钉刺",
            "Ability_Hunter_Quickshot",
            "Arcane Shot, Multi-Shot, Aimed Shot and Concussive Shot primary direct damage +4% per rank against targets with your Serpent Sting.",
            "奥术射击、多重射击、瞄准射击和震荡射击对受自己毒蛇钉刺影响的目标主要直接伤害每级提高4%。"
        ),
        node!(
            "entrapment",
            "Entrapment",
            "诱捕",
            "Spell_Nature_StrangleVines",
            "Explosive Trap's initial hit roots for 0.2 seconds per rank. Bosses receive 30% of that duration.",
            "爆炸陷阱首次命中使目标定身每级0.2秒；首领持续时间为该值的30%。"
        ),
        node!(
            "toxic_precision",
            "Toxic Precision",
            "剧毒精准",
            "Ability_Hunter_Quickshot",
            "Serpent Sting ticks can critically hit and deal +20% periodic damage against slowed targets.",
            "毒蛇钉刺每跳可以暴击，对减速目标的持续伤害提高20%。"
        ),
    ],
};

static ROGUE: ClassTalents = ClassTalents {
    trees: [
        ("Assassination", "刺杀"),
        ("Combat", "战斗"),
        ("Subtlety", "敏锐"),
    ],
    backgrounds: [
        bg!("RogueAssassination"),
        bg!("RogueCombat"),
        bg!("RogueSubtlety"),
    ],
    nodes: [
        node!(
            "improved_eviscerate",
            "Improved Eviscerate",
            "强化剔骨",
            "Ability_Rogue_Eviscerate",
            "Eviscerate damage +6% per rank.",
            "每级使剔骨伤害提高6%。"
        ),
        node!(
            "serrated_blades",
            "Serrated Blades",
            "锯齿利刃",
            "INV_Sword_17",
            "Rupture periodic damage +6% per rank.",
            "每级使割裂持续伤害提高6%。"
        ),
        node!(
            "lasting_rupture",
            "Lasting Rupture",
            "持久割裂",
            "Ability_Rogue_Rupture",
            "Rupture duration +6% per rank.",
            "每级使割裂持续时间延长6%。"
        ),
        node!(
            "relentless_strikes",
            "Relentless Strikes",
            "无情打击",
            "Ability_Warrior_DecisiveStrike",
            "Eviscerate cooldown -4% per rank; no energy cost.",
            "每级使剔骨冷却缩短4%，不涉及能量。"
        ),
        node!(
            "long_blades",
            "Long Blades",
            "长刃",
            "INV_ThrowingKnife_04",
            "Rupture range +1 yard per rank.",
            "每级使割裂攻击距离增加1码。"
        ),
        node!(
            "sanguinary_vein",
            "Sanguinary Vein",
            "血脉",
            "Spell_Shadow_BloodBoil",
            "Primary direct melee damage, including Backstab and Ambush, +4% per rank against targets with your Rupture.",
            "近战攻击（含背刺、伏击）对受自己割裂影响的目标主要直接伤害每级提高4%。"
        ),
        node!(
            "cut_to_the_chase",
            "Cut to the Chase",
            "穷追猛砍",
            "Ability_Rogue_Eviscerate",
            "Eviscerate primary direct critical hits extend your Rupture on that target by 0.2 seconds per rank. Total lifetime extension is capped at 50% of its initial duration.",
            "剔骨主要直接伤害暴击使目标身上自己的割裂每级延长0.2秒；每个割裂累计延长最多为初始持续时间的50%。"
        ),
        node!(
            "venomous_wounds",
            "Venomous Wounds",
            "剧毒创伤",
            "Ability_Rogue_Rupture",
            "Rupture ticks can critically hit. Periodic leech +2%; all talent leech shares a cap of 4% maximum health per second.",
            "割裂每跳可以暴击；持续伤害额外吸血2%，所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "improved_sinister_strike",
            "Improved Sinister Strike",
            "强化影袭",
            "Spell_Shadow_RitualOfSacrifice",
            "Sinister Strike damage +6% per rank.",
            "每级使影袭伤害提高6%。"
        ),
        node!(
            "improved_blade_flurry",
            "Improved Blade Flurry",
            "强化剑刃乱舞",
            "Ability_Warrior_PunishingBlow",
            "Blade Flurry damage +6% per rank.",
            "每级使剑刃乱舞伤害提高6%。"
        ),
        node!(
            "blade_reach",
            "Blade Reach",
            "剑刃延伸",
            "Ability_Warrior_PunishingBlow",
            "Blade Flurry radius +4% per rank.",
            "每级使剑刃乱舞半径扩大4%。"
        ),
        node!(
            "lightning_reflexes",
            "Lightning Reflexes",
            "闪电反射",
            "Spell_Nature_Invisibilty",
            "Sinister Strike cooldown -4% per rank.",
            "每级使影袭冷却缩短4%。"
        ),
        node!(
            "endurance",
            "Endurance",
            "耐久",
            "Spell_Shadow_ShadowWard",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "combat_potency",
            "Combat Potency",
            "战斗潜能",
            "INV_Weapon_ShortBlade_38",
            "Sinister Strike primary direct hits have a 20% chance to reduce learned Blade Flurry's remaining cooldown by 0.2 seconds per rank. Shared 1-second internal cooldown; no energy required.",
            "影袭主要直接伤害命中有20%概率使已学剑刃乱舞剩余冷却每级缩短0.2秒；共用1秒内置冷却，不涉及能量。"
        ),
        node!(
            "slice_and_dice",
            "Slice and Dice",
            "切割",
            "Ability_Rogue_SliceDice",
            "Eviscerate primary direct hits grant 3% haste per rank for 4 seconds. Shared 6-second internal cooldown.",
            "剔骨主要直接伤害命中使急速每级提高3%，持续4秒；共用6秒内置冷却。"
        ),
        node!(
            "adrenaline_rush",
            "Adrenaline Rush",
            "冲动",
            "Spell_Shadow_ShadowWordDominate",
            "F grants 25% haste for 4 seconds; no energy required.",
            "施放F后急速提高25%，持续4秒，不涉及能量。"
        ),
        node!(
            "dirty_tricks",
            "Dirty Tricks",
            "卑鄙伎俩",
            "Ability_Gouge",
            "Gouge and Kidney Shot stun duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使凿击和肾击昏迷持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "fleet_footed",
            "Fleet Footed",
            "健步如飞",
            "Ability_Rogue_Sprint",
            "Movement speed +2% per rank.",
            "每级使移动速度提高2%。"
        ),
        node!(
            "elusiveness",
            "Elusiveness",
            "飘忽不定",
            "Spell_Magic_LesserInvisibilty",
            "Incoming damage -1% per rank.",
            "每级使所受伤害降低1%。"
        ),
        node!(
            "setup",
            "Setup",
            "伺机而动",
            "Spell_Nature_MirrorImage",
            "Gouge and Kidney Shot cooldown -4% per rank.",
            "每级使凿击和肾击冷却缩短4%。"
        ),
        node!(
            "improved_sprint",
            "Improved Sprint",
            "强化疾跑",
            "Ability_Rogue_Sprint",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "prey_on_the_weak",
            "Prey on the Weak",
            "欺凌弱小",
            "Ability_CriticalStrike",
            "Primary direct melee damage, including Backstab and Ambush, +4% per rank against rooted or stunned targets.",
            "近战攻击（含背刺、伏击）对定身或昏迷目标的主要直接伤害每级提高4%。"
        ),
        node!(
            "shadow_guard",
            "Shadow Guard",
            "暗影守卫",
            "Ability_Rogue_Feint",
            "F grants 3% damage reduction per rank for 4 seconds.",
            "施放F后所受伤害每级降低3%，持续4秒。"
        ),
        node!(
            "shadow_dance",
            "Shadow Dance",
            "暗影之舞",
            "Ability_Stealth",
            "F empowers the next two Sinister Strike casts within 6 seconds: damage +40%.",
            "施放F后，6秒内接下来两次影袭施法伤害提高40%。"
        ),
    ],
};

static PRIEST: ClassTalents = ClassTalents {
    trees: [("Discipline", "戒律"), ("Holy", "神圣"), ("Shadow", "暗影")],
    backgrounds: [
        bg!("PriestDiscipline"),
        bg!("PriestHoly"),
        bg!("PriestShadow"),
    ],
    nodes: [
        node!(
            "improved_power_word_shield",
            "Improved Power Word: Shield",
            "强化真言术：盾",
            "Spell_Holy_PowerWordShield",
            "Power Word: Shield absorb +6% per rank.",
            "每级使真言术：盾吸收量提高6%。"
        ),
        node!(
            "soul_warding",
            "Soul Warding",
            "灵魂守护",
            "Spell_Holy_PowerWordShield",
            "Power Word: Shield cooldown -4% per rank.",
            "每级使真言术：盾冷却缩短4%。"
        ),
        node!(
            "lasting_shield",
            "Lasting Shield",
            "持久护盾",
            "Spell_Holy_InnerFire",
            "Power Word: Shield duration +6% per rank.",
            "每级使真言术：盾持续时间延长6%。"
        ),
        node!(
            "improved_fortitude",
            "Improved Fortitude",
            "强化坚韧",
            "Spell_Holy_WordFortitude",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "mental_agility",
            "Mental Agility",
            "精神敏锐",
            "Ability_Hibernation",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "borrowed_time",
            "Borrowed Time",
            "争分夺秒",
            "Spell_Holy_PowerWordShield",
            "While Power Word: Shield is active, haste +2% per rank.",
            "真言术：盾生效时，每级使急速提高2%。"
        ),
        node!(
            "reflective_shield",
            "Reflective Shield",
            "反射护盾",
            "Spell_Holy_PowerWordShield",
            "Reflect 5% per rank of contact damage actually absorbed by your shield. Reflected damage cannot recursively reflect or trigger primary-hit procs.",
            "护盾实际吸收的接触伤害每级反射5%；反射伤害不会递归反射或触发主要命中特效。"
        ),
        node!(
            "power_infusion",
            "Power Infusion",
            "能量灌注",
            "Spell_Holy_PowerInfusion",
            "F grants 20% haste for 6 seconds.",
            "施放F后急速提高20%，持续6秒。"
        ),
        node!(
            "improved_smite",
            "Improved Smite",
            "强化惩击",
            "Spell_Holy_HolySmite",
            "Smite damage +6% per rank.",
            "每级使惩击伤害提高6%。"
        ),
        node!(
            "searing_light",
            "Searing Light",
            "灼热之光",
            "Spell_Holy_SearingLightPriest",
            "Holy Fire direct and periodic damage +6% per rank.",
            "每级使神圣之火直接和持续伤害提高6%。"
        ),
        node!(
            "holy_reach",
            "Holy Reach",
            "神圣延伸",
            "Spell_Holy_HolyNova",
            "Holy Nova radius +4% per rank.",
            "每级使神圣新星半径扩大4%。"
        ),
        node!(
            "divine_fury",
            "Divine Fury",
            "神圣之怒",
            "Spell_Holy_SealOfWrath",
            "Smite and Holy Fire cooldown -4% per rank.",
            "每级使惩击和神圣之火冷却缩短4%。"
        ),
        node!(
            "lingering_flame",
            "Lingering Flame",
            "余焰",
            "Spell_Holy_SearingLight",
            "Holy Fire periodic-effect duration +6% per rank.",
            "每级使神圣之火持续伤害时长延长6%。"
        ),
        node!(
            "holy_weaving",
            "Holy Weaving",
            "神圣交织",
            "Spell_Holy_HolySmite",
            "Smite primary direct damage +4% per rank against targets with your Holy Fire periodic effect.",
            "惩击对受自己神圣之火持续效果影响的目标主要直接伤害每级提高4%。"
        ),
        node!(
            "surge_of_light",
            "Surge of Light",
            "圣光涌动",
            "Spell_Holy_SearingLightPriest",
            "Smite, Holy Fire and Holy Nova primary direct critical hits reduce learned Holy Nova's remaining cooldown by 0.3 seconds per rank. Shared 2-second internal cooldown.",
            "惩击、神圣之火和神圣新星主要直接伤害暴击使已学神圣新星剩余冷却每级缩短0.3秒；共用2秒内置冷却。"
        ),
        node!(
            "holy_burst",
            "Holy Burst",
            "神圣爆发",
            "Spell_Holy_HolyNova",
            "Holy Nova primary direct critical hits grant +15% Holy damage for 4 seconds, with a shared 6-second internal cooldown. Holy Nova healing +50% (10% to 15% of effective damage), capped at 4% maximum health per cast.",
            "神圣新星主要直接伤害暴击使神圣伤害提高15%，持续4秒，共用6秒内置冷却；神圣新星治疗提高50%（有效伤害的10%变为15%），每次施法最多恢复生命上限的4%。"
        ),
        node!(
            "improved_shadow_word_pain",
            "Improved Shadow Word: Pain",
            "强化暗言术：痛",
            "Spell_Shadow_ShadowWordPain",
            "Shadow Word: Pain periodic damage +6% per rank.",
            "每级使暗言术：痛持续伤害提高6%。"
        ),
        node!(
            "improved_mind_blast",
            "Improved Mind Blast",
            "强化心灵震爆",
            "Spell_Shadow_UnholyFrenzy",
            "Mind Blast damage +6% per rank.",
            "每级使心灵震爆伤害提高6%。"
        ),
        node!(
            "swift_pain",
            "Swift Pain",
            "迅疾痛楚",
            "Spell_Shadow_ShadowWordPain",
            "Shadow Word: Pain cooldown -4% per rank.",
            "每级使暗言术：痛冷却缩短4%。"
        ),
        node!(
            "focused_mind",
            "Focused Mind",
            "心灵专注",
            "Spell_Shadow_UnholyFrenzy",
            "Mind Blast cooldown -4% per rank.",
            "每级使心灵震爆冷却缩短4%。"
        ),
        node!(
            "shadow_reach",
            "Shadow Reach",
            "暗影延伸",
            "Spell_Shadow_ChillTouch",
            "Mind Blast and Shadow Word: Pain range +1 yard per rank.",
            "每级使心灵震爆和暗言术：痛射程增加1码。"
        ),
        node!(
            "shadow_weaving",
            "Shadow Weaving",
            "暗影交织",
            "Spell_Shadow_BlackPlague",
            "Shadow primary direct hits and Shadow Word: Pain ticks add target Shadow vulnerability: +1% per rank per stack, up to 3 stacks for 6 seconds. Applies after the triggering hit.",
            "暗影主要直接命中和暗言术：痛每跳叠加目标暗影易伤，每层每级1%，最多3层，持续6秒；触发该效果的一击不受益。"
        ),
        node!(
            "vampiric_embrace",
            "Vampiric Embrace",
            "吸血鬼的拥抱",
            "Spell_Shadow_UnsummonBuilding",
            "Heal for 0.4% of effective Shadow damage per rank. All talent leech shares a cap of 4% maximum health per second.",
            "暗影有效伤害每级吸血0.4%；所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "shadowform",
            "Shadowform",
            "暗影形态",
            "Spell_Shadow_Shadowform",
            "Shadow damage +20% and incoming damage -10%. Does not disable Holy spells.",
            "暗影伤害提高20%，所受伤害降低10%，不禁用神圣法术。"
        ),
    ],
};

static SHAMAN: ClassTalents = ClassTalents {
    trees: [
        ("Elemental", "元素"),
        ("Enhancement", "增强"),
        ("Restoration", "恢复"),
    ],
    backgrounds: [
        bg!("ShamanElementalCombat"),
        bg!("ShamanEnhancement"),
        bg!("ShamanRestoration"),
    ],
    nodes: [
        node!(
            "concussion",
            "Concussion",
            "震荡",
            "Spell_Nature_Lightning",
            "Lightning Bolt and Chain Lightning damage +6% per rank.",
            "每级使闪电箭和闪电链伤害提高6%。"
        ),
        node!(
            "shock_mastery",
            "Shock Mastery",
            "震击掌握",
            "Spell_Nature_EarthShock",
            "Earth Shock, Flame Shock and Frost Shock damage +6% per rank, including Flame Shock ticks.",
            "每级使地震术、烈焰震击和冰霜震击伤害提高6%，包括烈焰震击持续伤害。"
        ),
        node!(
            "lightning_mastery",
            "Lightning Mastery",
            "闪电掌握",
            "Spell_Lightning_LightningBolt01",
            "Lightning Bolt and Chain Lightning cooldown -4% per rank.",
            "每级使闪电箭和闪电链冷却缩短4%。"
        ),
        node!(
            "elemental_reach",
            "Elemental Reach",
            "元素延伸",
            "Spell_Nature_Lightning",
            "Lightning Bolt and Chain Lightning range +1 yard per rank.",
            "每级使闪电箭和闪电链射程增加1码。"
        ),
        node!(
            "improved_chain_lightning",
            "Improved Chain Lightning",
            "强化闪电链",
            "Spell_Nature_ChainLightning",
            "Chain Lightning jump-power modifier +1 percentage point per rank, subject to its 15-point cap.",
            "每级使闪电链跳跃威力修正增加1个百分点，该修正上限15个百分点。"
        ),
        node!(
            "elemental_focus",
            "Elemental Focus",
            "元素集中",
            "Spell_Shadow_ManaBurn",
            "Earth Shock, Flame Shock and Frost Shock primary direct critical hits empower the next Lightning Bolt or Chain Lightning cast within 6 seconds: damage +5% per rank.",
            "地震术、烈焰震击和冰霜震击主要直接伤害暴击使6秒内下一次闪电箭或闪电链伤害每级提高5%。"
        ),
        node!(
            "storm_and_flame",
            "Storm and Flame",
            "风暴烈焰",
            "Spell_Fire_Immolation",
            "Lightning Bolt and Chain Lightning primary direct damage +4% per rank against targets with your Flame Shock periodic effect.",
            "闪电箭和闪电链对受自己烈焰震击持续效果影响的目标主要直接伤害每级提高4%。"
        ),
        node!(
            "lightning_overload",
            "Lightning Overload",
            "闪电过载",
            "Spell_Nature_ChainLightning",
            "Lightning Bolt and Chain Lightning each have one 20% chance per cast to add a hit for 50% damage. The extra hit cannot trigger another extra hit.",
            "闪电箭和闪电链每次施法有一次20%概率追加一次50%伤害的命中；追加命中不会递归触发。"
        ),
        node!(
            "improved_lightning_shield",
            "Improved Lightning Shield",
            "强化闪电之盾",
            "Spell_Nature_LightningShield",
            "Lightning Shield damage +6% per rank.",
            "每级使闪电之盾伤害提高6%。"
        ),
        node!(
            "static_charge",
            "Static Charge",
            "静电充能",
            "Spell_Nature_LightningShield",
            "Lightning Shield pulse-frequency modifier +4% per rank, subject to the 200% frequency-bonus cap.",
            "每级使闪电之盾触发频率修正提高4%，频率加成上限200%。"
        ),
        node!(
            "reverberation",
            "Reverberation",
            "回响",
            "Spell_Frost_FrostShock",
            "Earth Shock, Flame Shock and Frost Shock cooldown -4% per rank.",
            "每级使地震术、烈焰震击和冰霜震击冷却缩短4%。"
        ),
        node!(
            "storm_reach",
            "Storm Reach",
            "风暴延伸",
            "Spell_Nature_LightningShield",
            "Lightning Shield radius +4% per rank.",
            "每级使闪电之盾半径扩大4%。"
        ),
        node!(
            "ancestral_endurance",
            "Ancestral Endurance",
            "先祖耐久",
            "Spell_Nature_AncestralGuardian",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "static_focus",
            "Static Focus",
            "静电专注",
            "Spell_Nature_LightningShield",
            "Lightning Shield hits empower the next Earth Shock, Flame Shock or Frost Shock cast within 6 seconds: damage +5% per rank. Shared 2-second internal cooldown.",
            "闪电之盾命中使6秒内下一次地震术、烈焰震击或冰霜震击伤害每级提高5%；共用2秒内置冷却。"
        ),
        node!(
            "elemental_flurry",
            "Elemental Flurry",
            "元素乱舞",
            "Spell_Nature_BloodLust",
            "Earth Shock, Flame Shock and Frost Shock primary direct critical hits grant 3% haste per rank for 4 seconds. Shared 6-second internal cooldown.",
            "地震术、烈焰震击和冰霜震击主要直接伤害暴击使急速每级提高3%，持续4秒；共用6秒内置冷却。"
        ),
        node!(
            "shamanistic_rage",
            "Shamanistic Rage",
            "萨满之怒",
            "Spell_Nature_BloodLust",
            "F grants Lightning Shield +50% pulse frequency and Earth Shock, Flame Shock and Frost Shock +20% damage for 6 seconds.",
            "施放F后闪电之盾触发频率提高50%，地震术、烈焰震击和冰霜震击伤害提高20%，持续6秒。"
        ),
        node!(
            "totemic_power",
            "Totemic Power",
            "图腾之力",
            "Spell_Fire_SearingTotem",
            "Searing Totem and Magma Totem damage +6% per rank.",
            "每级使灼热图腾和熔岩图腾伤害提高6%。"
        ),
        node!(
            "ancestral_protection",
            "Ancestral Protection",
            "先祖庇护",
            "Spell_Nature_StoneSkinTotem",
            "Incoming damage -1% per rank.",
            "每级使所受伤害降低1%。"
        ),
        node!(
            "totemic_longevity",
            "Totemic Longevity",
            "持久图腾",
            "Spell_Nature_EarthBindTotem",
            "Searing Totem and Magma Totem duration +6% per rank.",
            "每级使灼热图腾和熔岩图腾持续时间延长6%。"
        ),
        node!(
            "totemic_focus",
            "Totemic Focus",
            "图腾集中",
            "Spell_Nature_StoneClawTotem",
            "Searing Totem and Magma Totem cooldown -4% per rank.",
            "每级使灼热图腾和熔岩图腾冷却缩短4%。"
        ),
        node!(
            "ancestral_swiftness",
            "Ancestral Swiftness",
            "先祖迅捷",
            "Spell_Nature_RavenForm",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "totemic_shelter",
            "Totemic Shelter",
            "图腾庇护",
            "Spell_Nature_StoneSkinTotem",
            "Within 10 yards of your own active totem, incoming damage -2% per rank.",
            "距离自己的存续图腾10码内时，每级使所受伤害降低2%。"
        ),
        node!(
            "healing_stream",
            "Healing Stream",
            "治疗之泉",
            "INV_Spear_04",
            "Your totem damage heals for 0.4% of effective damage per rank. All talent leech shares a cap of 4% maximum health per second.",
            "自己图腾的有效伤害每级吸血0.4%；所有天赋吸血共用每秒生命上限4%的恢复上限。"
        ),
        node!(
            "ancestral_guardian",
            "Ancestral Guardian",
            "先祖守护",
            "Spell_Nature_AncestralGuardian",
            "While below 35% health and within 10 yards of your own active totem, gain a shield for 20% maximum health lasting 8 seconds. 30-second internal cooldown.",
            "生命低于35%且位于自己的存续图腾10码内时，获得生命上限20%的护盾，持续8秒；内置冷却30秒。"
        ),
    ],
};

static MAGE: ClassTalents = ClassTalents {
    trees: [("Arcane", "奥术"), ("Fire", "火焰"), ("Frost", "冰霜")],
    backgrounds: [bg!("MageArcane"), bg!("MageFire"), bg!("MageFrost")],
    nodes: [
        node!(
            "improved_arcane_missiles",
            "Improved Arcane Missiles",
            "强化奥术飞弹",
            "Spell_Nature_StarFall",
            "Arcane Missiles damage +6% per rank.",
            "每级使奥术飞弹伤害提高6%。"
        ),
        node!(
            "improved_arcane_explosion",
            "Improved Arcane Explosion",
            "强化魔爆术",
            "Spell_Nature_WispSplode",
            "Arcane Explosion damage +6% per rank.",
            "每级使魔爆术伤害提高6%。"
        ),
        node!(
            "arcane_focus",
            "Arcane Focus",
            "奥术集中",
            "Spell_Holy_Devotion",
            "Arcane Missiles and Arcane Explosion cooldown -4% per rank.",
            "每级使奥术飞弹和魔爆术冷却缩短4%。"
        ),
        node!(
            "arcane_fortitude",
            "Arcane Fortitude",
            "奥术坚韧",
            "Spell_Holy_Devotion",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "improved_blink",
            "Improved Blink",
            "强化闪现术",
            "Spell_Arcane_Blink",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "arcane_blast",
            "Arcane Blast",
            "奥术冲击",
            "Spell_Nature_WispSplode",
            "Arcane Missiles and Arcane Explosion casts grant +1% Arcane damage per rank per stack, up to 3 stacks for 6 seconds. Stacks apply after the triggering cast.",
            "奥术飞弹和魔爆术施法后叠加奥术伤害加成，每层每级1%，最多3层，持续6秒；触发的施法不受益。"
        ),
        node!(
            "missile_barrage",
            "Missile Barrage",
            "飞弹速射",
            "Spell_Nature_StarFall",
            "Arcane Missiles has one 4% chance per rank per cast to add a hit for 50% damage. The extra hit cannot trigger another extra hit.",
            "奥术飞弹每次施法有一次每级4%概率追加一次50%伤害的命中；追加命中不会递归触发。"
        ),
        node!(
            "arcane_power",
            "Arcane Power",
            "奥术强化",
            "Spell_Nature_Lightning",
            "F grants +20% damage and 15% haste for 5 seconds.",
            "施放F后伤害提高20%，急速提高15%，持续5秒。"
        ),
        node!(
            "improved_fireball",
            "Improved Fireball",
            "强化火球术",
            "Spell_Fire_FlameBolt",
            "Fireball damage +6% per rank.",
            "每级使火球术伤害提高6%。"
        ),
        node!(
            "empowered_pyroblast",
            "Empowered Pyroblast",
            "强化炎爆术",
            "Spell_Fire_Fireball02",
            "Pyroblast damage +6% per rank.",
            "每级使炎爆术伤害提高6%。"
        ),
        node!(
            "improved_fire_blast",
            "Improved Fire Blast",
            "强化火焰冲击",
            "Spell_Fire_Fireball",
            "Fire Blast cooldown -4% per rank.",
            "每级使火焰冲击冷却缩短4%。"
        ),
        node!(
            "flame_spreading",
            "Flame Spreading",
            "烈焰扩散",
            "Spell_Fire_SelfDestruct",
            "Flamestrike and Blast Wave radius +4% per rank.",
            "每级使烈焰风暴和冲击波半径扩大4%。"
        ),
        node!(
            "flame_throwing",
            "Flame Throwing",
            "烈焰投掷",
            "Spell_Fire_Flare",
            "Fireball and Pyroblast range +1 yard per rank.",
            "每级使火球术和炎爆术射程增加1码。"
        ),
        node!(
            "ignite",
            "Ignite",
            "点燃",
            "Spell_Fire_Incinerate",
            "Fireball, Fire Blast, Pyroblast and Blast Wave primary direct critical hits cause a 4-second Ignite totaling 8% of the triggering damage per rank. Flamestrike pulses and Ignite cannot trigger it.",
            "火球术、火焰冲击、炎爆术和冲击波主要直接伤害暴击附加4秒点燃，总伤害为触发伤害的每级8%；烈焰风暴跳伤和点燃不能触发。"
        ),
        node!(
            "hot_streak",
            "Hot Streak",
            "热力迸发",
            "Spell_Fire_Fire",
            "Two consecutive Fireball, Fire Blast, Pyroblast or Blast Wave primary direct critical hits have a 20% chance per rank (100% at rank 5) to reset learned Pyroblast. A noncritical eligible hit breaks the sequence. Shared 8-second internal cooldown; Flamestrike pulses and Ignite are excluded.",
            "火球术、火焰冲击、炎爆术或冲击波主要直接伤害连续暴击两次，有每级20%概率（5级100%）刷新已学炎爆术；符合条件但未暴击的命中中断连击，共用8秒内置冷却；烈焰风暴跳伤和点燃不计入。"
        ),
        node!(
            "combustion",
            "Combustion",
            "燃烧",
            "Spell_Fire_SealOfFire",
            "Fireball, Fire Blast, Pyroblast and Blast Wave primary direct critical hits grant +5 percentage points of Fire critical chance per stack, up to 3 stacks for 6 seconds. Flamestrike pulses and Ignite cannot trigger it.",
            "火球术、火焰冲击、炎爆术和冲击波主要直接伤害暴击使火焰暴击率每层提高5个百分点，最多3层，持续6秒；烈焰风暴跳伤和点燃不能触发。"
        ),
        node!(
            "improved_frostbolt",
            "Improved Frostbolt",
            "强化寒冰箭",
            "Spell_Frost_FrostBolt02",
            "Frostbolt damage +6% per rank.",
            "每级使寒冰箭伤害提高6%。"
        ),
        node!(
            "improved_blizzard",
            "Improved Blizzard",
            "强化暴风雪",
            "Spell_Frost_IceStorm",
            "Blizzard damage +6% per rank.",
            "每级使暴风雪伤害提高6%。"
        ),
        node!(
            "piercing_cold",
            "Piercing Cold",
            "刺骨寒冷",
            "Spell_Frost_Glacier",
            "Cone of Cold damage +6% per rank.",
            "每级使冰锥术伤害提高6%。"
        ),
        node!(
            "improved_frost_nova",
            "Improved Frost Nova",
            "强化冰霜新星",
            "Spell_Frost_FrostNova",
            "Frost Nova cooldown -4% per rank.",
            "每级使冰霜新星冷却缩短4%。"
        ),
        node!(
            "permafrost",
            "Permafrost",
            "永冻",
            "Spell_Frost_Wisp",
            "Frostbolt, Blizzard and Cone of Cold slow duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使寒冰箭、暴风雪和冰锥术减速持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "shatter",
            "Shatter",
            "碎冰",
            "Spell_Frost_FrostShock",
            "Frostbolt, Frost Nova and Cone of Cold primary direct hits against slowed or rooted targets gain +4 percentage points of critical chance per rank. Blizzard pulses are excluded.",
            "寒冰箭、冰霜新星和冰锥术主要直接伤害对减速或定身目标的暴击率每级提高4个百分点；不包括暴风雪跳伤。"
        ),
        node!(
            "ice_barrier",
            "Ice Barrier",
            "寒冰护体",
            "Spell_Ice_Lament",
            "Frost Nova hits grant a shield for 2% maximum health per rank, lasting 6 seconds. Shared 6-second internal cooldown.",
            "冰霜新星命中获得每级生命上限2%的护盾，持续6秒；共用6秒内置冷却。"
        ),
        node!(
            "fingers_of_frost",
            "Fingers of Frost",
            "寒冰指",
            "Spell_Frost_FrostArmor",
            "Frostbolt, Frost Nova and Cone of Cold hits and Blizzard pulses have a 15% chance to grant Frozen Opportunity (one attempt per Blizzard pulse), up to 2 charges for 6 seconds. The next Frostbolt, Frost Nova or Cone of Cold primary direct casts consume a charge and use Shatter's target condition; no extra cast.",
            "寒冰箭、冰霜新星、冰锥术命中和暴风雪脉冲有15%概率获得冰冻契机（每次暴风雪脉冲只判定一次），最多2层，持续6秒；接下来的寒冰箭、冰霜新星或冰锥术主要直接施法消耗一层并视为符合碎冰条件，不额外施法。"
        ),
    ],
};

static DRUID: ClassTalents = ClassTalents {
    trees: [
        ("Balance", "平衡"),
        ("Feral Combat", "野性战斗"),
        ("Restoration", "恢复"),
    ],
    backgrounds: [
        bg!("DruidBalance"),
        bg!("DruidFeralCombat"),
        bg!("DruidRestoration"),
    ],
    nodes: [
        node!(
            "improved_wrath",
            "Improved Wrath",
            "强化愤怒",
            "Spell_Nature_AbolishMagic",
            "Wrath damage +6% per rank.",
            "每级使愤怒伤害提高6%。"
        ),
        node!(
            "improved_starfire",
            "Improved Starfire",
            "强化星火术",
            "Spell_Arcane_StarFire",
            "Starfire damage +6% per rank.",
            "每级使星火术伤害提高6%。"
        ),
        node!(
            "natures_splendor",
            "Nature's Splendor",
            "自然之美",
            "Spell_Nature_StarFall",
            "Moonfire direct and periodic damage and Insect Swarm periodic damage +6% per rank.",
            "每级使月火术直接及持续伤害和虫群持续伤害提高6%。"
        ),
        node!(
            "gale_force",
            "Gale Force",
            "强风",
            "Spell_Nature_Cyclone",
            "Hurricane radius +4% per rank.",
            "每级使飓风半径扩大4%。"
        ),
        node!(
            "starlight_wrath",
            "Starlight Wrath",
            "星光之怒",
            "Spell_Nature_Sentinal",
            "Wrath and Starfire cooldown -4% per rank.",
            "每级使愤怒和星火术冷却缩短4%。"
        ),
        node!(
            "eclipse",
            "Eclipse",
            "日月之蚀",
            "Spell_Nature_Sentinal",
            "Wrath primary direct critical hits empower the next Starfire cast within 6 seconds: damage +5% per rank. Starfire primary direct critical hits likewise empower the next Wrath cast.",
            "愤怒主要直接伤害暴击使6秒内下一次星火术伤害每级提高5%；星火术主要直接伤害暴击同样强化下一次愤怒。"
        ),
        node!(
            "shooting_stars",
            "Shooting Stars",
            "坠星",
            "Spell_Arcane_StarFire",
            "Moonfire ticks have a 20% chance to reduce learned Starfire's remaining cooldown by 0.2 seconds per rank. Shared 1-second internal cooldown.",
            "月火术每跳有20%概率使已学星火术剩余冷却每级缩短0.2秒；共用1秒内置冷却。"
        ),
        node!(
            "celestial_focus",
            "Celestial Focus",
            "星空专注",
            "Spell_Nature_StarFall",
            "Moonfire and Insect Swarm periodic damage can critically hit.",
            "月火术和虫群持续伤害可以暴击。"
        ),
        node!(
            "brambles",
            "Brambles",
            "荆棘",
            "Spell_Nature_Thorns",
            "Thorns damage +6% per rank; no form required.",
            "每级使荆棘术伤害提高6%，不需要变形。"
        ),
        node!(
            "grasping_roots",
            "Grasping Roots",
            "缠绕之根",
            "Spell_Nature_StrangleVines",
            "Entangling Roots root duration +6% per rank, subject to the 50% control-duration bonus cap.",
            "每级使纠缠根须定身持续时间延长6%，控制时长加成上限50%。"
        ),
        node!(
            "improved_entangling_roots",
            "Improved Entangling Roots",
            "强化纠缠根须",
            "Spell_Nature_StrangleVines",
            "Entangling Roots cooldown -4% per rank.",
            "每级使纠缠根须冷却缩短4%。"
        ),
        node!(
            "feral_swiftness",
            "Feral Swiftness",
            "野性迅捷",
            "Ability_Druid_Dash",
            "Movement speed +2% per rank; no form required.",
            "每级使移动速度提高2%，不需要变形。"
        ),
        node!(
            "wild_stride",
            "Wild Stride",
            "野性步伐",
            "Ability_Druid_Dash",
            "F mobility cooldown -4% per rank.",
            "每级使F键位移技能冷却缩短4%。"
        ),
        node!(
            "inhibiting_thorns",
            "Inhibiting Thorns",
            "迟滞荆棘",
            "Spell_Nature_Thorns",
            "Thorns hits slow by 4% per rank for 2 seconds. Half strength on bosses; refreshes, never stacks.",
            "荆棘术命中使目标减速每级4%，持续2秒；首领减速强度减半，仅刷新、不叠加。"
        ),
        node!(
            "predatory_strikes",
            "Predatory Strikes",
            "猛兽攻击",
            "Ability_Hunter_Pet_Cat",
            "Wrath, Starfire and Moonfire primary direct damage +4% per rank against rooted targets; no form required.",
            "愤怒、星火术和月火术对定身目标的主要直接伤害每级提高4%，不需要变形。"
        ),
        node!(
            "barkskin",
            "Barkskin",
            "树皮术",
            "Spell_Nature_StoneClawTotem",
            "F grants 20% damage reduction and +30% Thorns damage for 4 seconds; no form required.",
            "施放F后所受伤害降低20%，荆棘术伤害提高30%，持续4秒，不需要变形。"
        ),
        node!(
            "improved_rejuvenation",
            "Improved Rejuvenation",
            "强化回春术",
            "Spell_Nature_Rejuvenation",
            "Rejuvenation healing +6% per rank.",
            "每级使回春术治疗量提高6%。"
        ),
        node!(
            "natural_recovery",
            "Natural Recovery",
            "自然恢复",
            "Spell_Nature_HealingTouch",
            "Regenerate 0.2 health per second per rank.",
            "每级每秒额外恢复0.2生命值。"
        ),
        node!(
            "gift_of_the_wild",
            "Gift of the Wild",
            "野性赐福",
            "Spell_Nature_Regeneration",
            "Maximum health +3% per rank.",
            "每级使生命上限提高3%。"
        ),
        node!(
            "swift_rejuvenation",
            "Swift Rejuvenation",
            "迅捷回春",
            "Spell_Nature_Rejuvenation",
            "Rejuvenation cooldown -4% per rank.",
            "每级使回春术冷却缩短4%。"
        ),
        node!(
            "lasting_rejuvenation",
            "Lasting Rejuvenation",
            "持久回春",
            "Spell_Nature_Rejuvenation",
            "Rejuvenation duration +6% per rank.",
            "每级使回春术持续时间延长6%。"
        ),
        node!(
            "living_bark",
            "Living Bark",
            "生命树皮",
            "Spell_Nature_ProtectionformNature",
            "While Rejuvenation is active, incoming damage -2% per rank.",
            "回春术生效时，每级使所受伤害降低2%。"
        ),
        node!(
            "living_seed",
            "Living Seed",
            "生命之种",
            "Spell_Nature_HealingTouch",
            "When Rejuvenation ends naturally, gain an 8-second shield for 4% per rank of its total effective healing, capped at 10% maximum health. Recasting does not trigger it.",
            "回春术自然结束时，获得其累计有效治疗量每级4%的护盾，持续8秒，最多为生命上限的10%；重新施放不触发。"
        ),
        node!(
            "natures_vigil",
            "Nature's Vigil",
            "自然守护",
            "Spell_Nature_HealingTouch",
            "Effective Wrath and Starfire damage extends active Rejuvenation by 1 second. Shared 1-second internal cooldown; total lifetime extension is capped at 50% of its initial duration.",
            "愤怒和星火术造成有效伤害使当前回春术延长1秒；共用1秒内置冷却，每个回春术累计延长最多为初始持续时间的50%。"
        ),
    ],
};

pub(super) fn for_class(class: u8) -> Option<&'static ClassTalents> {
    match class {
        1 => Some(&WARRIOR),
        2 => Some(&PALADIN),
        3 => Some(&HUNTER),
        4 => Some(&ROGUE),
        5 => Some(&PRIEST),
        7 => Some(&SHAMAN),
        8 => Some(&MAGE),
        11 => Some(&DRUID),
        _ => None,
    }
}

fn rank(t: Loadout, index: usize) -> f32 {
    f32::from(t.0[index].min(if index % 8 == 7 { 1 } else { 5 }))
}

pub(super) fn modify_stats(class: u8, stats: &mut Stats) {
    let r = |i| rank(stats.talents, i);
    let (health, armor, speed, regen) = match class {
        1 => (0.03 * r(18), 0.4 * r(17), 0.0, 0.0),
        2 => (0.03 * r(8), 0.4 * r(10), 0.0, 0.0),
        3 => (0.03 * r(2), 0.0, 0.02 * r(3), 0.0),
        4 => (0.03 * r(12), 0.0, 0.02 * r(17), 0.0),
        5 => (0.03 * r(3), 0.0, 0.0, 0.0),
        7 => (0.03 * r(12), 0.0, 0.0, 0.0),
        8 => (0.03 * r(3), 0.0, 0.0, 0.0),
        11 => (0.03 * r(18), 0.0, 0.02 * r(11), 0.2 * r(17)),
        _ => return,
    };
    stats.max_hp *= 1.0 + health;
    stats.armor += armor;
    stats.speed *= 1.0 + speed;
    stats.regen += regen;
}

pub(super) fn incoming(class: u8, t: Loadout) -> f32 {
    let reduction = match class {
        1 => 0.01 * rank(t, 16),
        2 => 0.01 * rank(t, 9),
        3 | 4 => 0.01 * rank(t, 18),
        7 => 0.01 * rank(t, 17),
        _ => 0.0,
    };
    1.0 - reduction
}

pub(super) fn mobility_cooldown(class: u8, t: Loadout) -> f32 {
    let index = match class {
        1 | 2 | 11 => 12,
        3 | 5 | 8 => 4,
        4 | 7 => 20,
        _ => return 1.0,
    };
    1.0 - 0.04 * rank(t, index)
}

pub(super) fn modify_cast(class: u8, key: &str, stats: &Stats, cast: &mut Cast) {
    let r = |i| rank(stats.talents, i);
    let mut damage = 0.0;
    let mut cooldown = 0.0;
    let mut radius = 0.0;
    let mut duration = 0.0;
    let mut range = 0.0;
    let mut control = 0.0;
    let mut frequency = 0.0;
    let mut chain = 0.0;
    match class {
        1 => match key {
            "heroic_strike" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(2);
            }
            "mortal_strike" => {
                damage = 0.06 * r(1);
                cooldown = 0.04 * r(2);
            }
            "cleave" => {
                damage = 0.06 * r(8);
                range = 0.4 * r(3);
            }
            "execute" => damage = 0.06 * r(4),
            "whirlwind" => {
                damage = 0.06 * r(9);
                cooldown = 0.04 * r(10);
            }
            "sweeping_strikes" => damage = 0.06 * r(11),
            "thunder_clap" => radius = 0.04 * r(19),
            "demoralizing_shout" => control = 0.06 * r(20),
            _ => {}
        },
        2 => match key {
            "holy_shock" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(1);
                range = r(4);
            }
            "exorcism" => {
                damage = 0.06 * r(2);
                range = r(4);
            }
            "holy_wrath" => {
                damage = 0.06 * r(3);
                cooldown = 0.04 * r(19);
            }
            "consecration" => radius = 0.04 * r(11),
            "hammer_of_wrath" => {
                damage = 0.06 * r(16);
                cooldown = 0.04 * r(18);
            }
            "judgement" => {
                damage = 0.06 * r(17);
                cooldown = 0.04 * r(18);
            }
            "hammer_of_justice" => control = 0.06 * r(20),
            _ => {}
        },
        3 => match key {
            "arcane_shot" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(10);
                range = r(11);
            }
            "serpent_sting" => {
                damage = 0.06 * r(1);
                duration = 0.06 * r(17);
            }
            "aimed_shot" => {
                damage = 0.06 * r(8);
                cooldown = 0.04 * r(10);
                range = r(11);
            }
            "multi_shot" => {
                damage = 0.06 * r(9);
                range = r(11);
            }
            "concussive_shot" => {
                range = r(11);
                control = 0.06 * r(19);
            }
            "volley" => radius = 0.04 * r(12),
            "explosive_trap" => {
                damage = 0.06 * r(16);
                cooldown = 0.04 * r(20);
            }
            _ => {}
        },
        4 => match key {
            "eviscerate" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(3);
            }
            "rupture" => {
                damage = 0.06 * r(1);
                duration = 0.06 * r(2);
                range = r(4);
            }
            "sinister_strike" => {
                damage = 0.06 * r(8);
                cooldown = 0.04 * r(11);
            }
            "blade_flurry" => {
                damage = 0.06 * r(9);
                radius = 0.04 * r(10);
            }
            "gouge" | "kidney_shot" => {
                control = 0.06 * r(16);
                cooldown = 0.04 * r(19);
            }
            _ => {}
        },
        5 => match key {
            "power_word_shield" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(1);
                duration = 0.06 * r(2);
            }
            "smite" => {
                damage = 0.06 * r(8);
                cooldown = 0.04 * r(11);
            }
            "holy_fire" => {
                damage = 0.06 * r(9);
                cooldown = 0.04 * r(11);
                duration = 0.06 * r(12);
            }
            "holy_nova" => radius = 0.04 * r(10),
            "shadow_word_pain" => {
                damage = 0.06 * r(16);
                cooldown = 0.04 * r(18);
                range = r(20);
            }
            "mind_blast" => {
                damage = 0.06 * r(17);
                cooldown = 0.04 * r(19);
                range = r(20);
            }
            _ => {}
        },
        7 => match key {
            "lightning_bolt" | "chain_lightning" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(2);
                range = r(3);
                if key == "chain_lightning" {
                    chain = 0.01 * r(4);
                }
            }
            "earth_shock" | "flame_shock" | "frost_shock" => {
                damage = 0.06 * r(1);
                cooldown = 0.04 * r(10);
            }
            "lightning_shield" => {
                damage = 0.06 * r(8);
                frequency = 0.04 * r(9);
                radius = 0.04 * r(11);
            }
            "searing_totem" | "magma_totem" => {
                damage = 0.06 * r(16);
                duration = 0.06 * r(18);
                cooldown = 0.04 * r(19);
            }
            _ => {}
        },
        8 => match key {
            "arcane_missiles" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(2);
            }
            "arcane_explosion" => {
                damage = 0.06 * r(1);
                cooldown = 0.04 * r(2);
            }
            "fireball" => {
                damage = 0.06 * r(8);
                range = r(12);
            }
            "pyroblast" => {
                damage = 0.06 * r(9);
                range = r(12);
            }
            "fire_blast" => cooldown = 0.04 * r(10),
            "flamestrike" | "blast_wave" => radius = 0.04 * r(11),
            "frostbolt" => {
                damage = 0.06 * r(16);
                control = 0.06 * r(20);
            }
            "blizzard" => {
                damage = 0.06 * r(17);
                control = 0.06 * r(20);
            }
            "cone_of_cold" => {
                damage = 0.06 * r(18);
                control = 0.06 * r(20);
            }
            "frost_nova" => cooldown = 0.04 * r(19),
            _ => {}
        },
        11 => match key {
            "wrath" => {
                damage = 0.06 * r(0);
                cooldown = 0.04 * r(4);
            }
            "starfire" => {
                damage = 0.06 * r(1);
                cooldown = 0.04 * r(4);
            }
            "moonfire" | "insect_swarm" => damage = 0.06 * r(2),
            "hurricane" => radius = 0.04 * r(3),
            "thorns" => damage = 0.06 * r(8),
            "entangling_roots" => {
                control = 0.06 * r(9);
                cooldown = 0.04 * r(10);
            }
            "rejuvenation" => {
                damage = 0.06 * r(16);
                cooldown = 0.04 * r(19);
                duration = 0.06 * r(20);
            }
            _ => {}
        },
        _ => return,
    }
    cast.dmg *= 1.0 + damage;
    cast.dot *= 1.0 + damage;
    cast.radius *= 1.0 + radius;
    cast.duration *= 1.0 + duration;
    cast.range += range;
    if cooldown > 0.0 {
        cast.cooldown = (cast.cooldown * (1.0 - cooldown)).max(0.2);
    }
    if control > 0.0 {
        cast.modifiers.control = (cast.modifiers.control + control).min(0.5);
    }
    if frequency > 0.0 {
        let previous = cast.modifiers.frequency;
        cast.modifiers.frequency = (previous + frequency).min(2.0);
        // Orbit pulses use the cast clock, unlike ground and totem tick intervals.
        cast.cooldown =
            (cast.cooldown * (1.0 + previous) / (1.0 + cast.modifiers.frequency)).max(0.2);
    }
    if chain > 0.0 {
        cast.modifiers.chain = (cast.modifiers.chain + chain).min(0.15);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn eight_class_catalogs_have_stable_local_ids_and_complete_art() {
        let parts = ["TopLeft", "TopRight", "BottomLeft", "BottomRight"];
        for class in [1, 2, 3, 4, 5, 7, 8, 11] {
            let catalog = for_class(class).unwrap();
            let mut ids = BTreeSet::new();
            for node in &catalog.nodes {
                assert!(ids.insert(node.id), "class {class}: duplicate {}", node.id);
                assert!(node.id.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'));
                assert!(!node.name.is_empty() && !node.desc.is_empty());
                assert!(!node.zh.is_ascii() && !node.desc_zh.is_ascii());
                assert!(node.icon.starts_with("Interface\\Icons\\"));
            }
            assert_eq!(ids.len(), 24);
            for (tree, backgrounds) in catalog.trees.iter().zip(catalog.backgrounds) {
                assert!(!tree.0.is_empty() && !tree.1.is_ascii());
                for (path, suffix) in backgrounds.iter().zip(parts) {
                    assert!(path.starts_with("Interface\\TalentFrame\\"));
                    assert!(path.ends_with(&format!("-{suffix}")));
                }
            }
        }
        for class in [0, 6, 9, 10, 255] {
            assert!(for_class(class).is_none());
        }
    }

    #[test]
    fn ranks_are_not_legacy_doubled_and_are_capped_by_node_type() {
        let mut t = Loadout::default();
        for index in 0..24 {
            t.0[index] = 1;
            assert_eq!(rank(t, index), 1.0);
            t.0[index] = u8::MAX;
            assert_eq!(rank(t, index), if index % 8 == 7 { 1.0 } else { 5.0 });
        }
    }

    #[test]
    fn defensive_and_mobility_basics_are_class_local() {
        for class in [1, 2, 3, 4, 5, 7, 8, 9, 11] {
            assert_eq!(incoming(class, Loadout::default()), 1.0);
            assert_eq!(mobility_cooldown(class, Loadout::default()), 1.0);
        }
        for (class, index) in [(1, 16), (2, 9), (3, 18), (4, 18), (7, 17)] {
            let mut t = Loadout::default();
            t.0[index] = 5;
            assert!((incoming(class, t) - 0.95).abs() < 1e-6);
        }
        for (class, index) in [
            (1, 12),
            (2, 12),
            (3, 4),
            (4, 20),
            (5, 4),
            (7, 20),
            (8, 4),
            (11, 12),
        ] {
            let mut t = Loadout::default();
            t.0[index] = 5;
            assert!((mobility_cooldown(class, t) - 0.8).abs() < 1e-6);
            assert_eq!(mobility_cooldown(9, t), 1.0);
        }
    }
}
