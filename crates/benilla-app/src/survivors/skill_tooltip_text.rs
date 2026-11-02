pub(super) fn special_description(key: &str, zh_cn: bool) -> Option<&'static str> {
    if key == "polymorph" {
        return Some(if zh_cn {
            "目标可减速移动但不能攻击；受击不会解除变羊，并且受到暴击率提高20个百分点。"
        } else {
            "Targets can move slowly but cannot attack. Damage does not break Polymorph; they take 20 percentage points more critical-hit chance."
        });
    }
    if key == "ice_barrier" {
        return None;
    }
    if key == "mana_shield" {
        return Some(if zh_cn {
            "本模式不消耗法力，并将原版仅吸收物理伤害改为吸收所有伤害；与其他护盾共用护盾池。"
        } else {
            "In this mode, costs no mana and absorbs all damage rather than only physical damage; shares the absorption pool with other shields."
        });
    }
    if key == "arcane_power" {
        return Some(if zh_cn {
            "法术的直接伤害和持续伤害提高30%；不增强治疗、护盾或武器攻击。本模式不消耗法力。"
        } else {
            "Increases spell direct and periodic damage by 30%, not healing, absorption or weapon attacks. This mode has no mana cost."
        });
    }
    if key == "combustion" {
        return Some(if zh_cn {
            "每次直接火焰法术命中后，后续火焰法术暴击率提高10个百分点，最高100%；累计3次直接火焰暴击或持续时间结束时消失。"
        } else {
            "Each direct Fire spell hit adds 10 percentage points to subsequent Fire spell critical-hit chance, up to 100%. Ends after 3 direct Fire critical hits or when its duration expires."
        });
    }
    if zh_cn {
        return super::spell_expansion::description(key);
    }
    Some(match key {
        "sweeping_strikes" => "Your next 5 effective weapon hits also strike one nearby enemy. Does not independently pulse area damage; secondary hits cannot trigger it again.",
        "blade_flurry" => "Increases melee weapon skill frequency by 20%; effective weapon hits also strike one nearby enemy. Secondary hits cannot trigger it again.",
        "lightning_shield" => "In this mode, retaliates against melee contact attacks while active without consuming lightning-ball charges; it does not proactively zap nearby enemies.",
        "overpower" | "mongoose_bite" => {
            "Automatically counterattacks in melee for 3 sec after taking damage; its cooldown is retained until it triggers."
        }
        "shield_bash" => {
            "Only targets caster enemies; deals melee damage and interrupts with a brief stun."
        }
        "execute" | "drain_soul" => {
            "Only selects enemies at or below 30% health. Drain Soul deals periodic damage and executes non-bosses at low health."
        }
        "hammer_of_wrath" => "Only targets enemies at or below 20% health.",
        "shield_wall" => {
            "Reduces damage taken by 60% briefly."
        }
        "last_stand" => {
            "Its temporary absorb shield does not permanently increase maximum health."
        }
        "seal_of_righteousness" => {
            "Weapon hits deal additional Holy damage while active, at most once per attack."
        }
        "seal_of_command" => {
            "Weapon hits have a 35% chance to deal an additional heavy Holy strike while active."
        }
        "seal_of_wisdom" => {
            "Improves skill cooldown recovery while active; this mode has no separate mana system."
        }
        "seal_of_light" => {
            "Weapon hits restore health while active; the effect has its own cooldown and cannot recurse."
        }
        "deadly_poison" => {
            "Weapon hits apply periodic poison damage while active; the coating itself does not directly attack."
        }
        "windfury_weapon" => {
            "Weapon hits have a 20% chance to add two Windfury damage hits while active; they cannot recurse."
        }
        "rockbiter_weapon" => "Increases weapon melee damage by 25% while active.",
        "frostbrand_weapon" => {
            "Weapon hits deal additional Frost damage and slow while active."
        }
        "rapid_fire" => "Briefly increases ranged skill frequency.",
        "devotion_aura" => "Increases your armor while active; it affects only you.",
        "inner_fire" => "Increases your armor while active; it affects only you.",
        "frost_armor" => {
            "Increases armor and automatically slows nearby melee enemies while active. Mutually exclusive with Mage Armor when learning skills; their buffs cannot stack."
        }
        "mage_armor" => {
            "Restores health and reduces damage taken while active; this mode has no separate mana system. Mutually exclusive with Frost Armor when learning skills; their buffs cannot stack."
        }
        "dampen_magic" => "Reduces damage taken by 20% while active; it deals no direct damage.",
        "presence_of_mind" => {
            "While active, damaging Mage skills recover cooldowns at twice normal speed, including cooldowns already running. Multiplies with other cooldown modifiers. Casting neither consumes nor extends it. Does not change initial cooldowns, damage or periodic tick frequency; excludes pure control, defenses, utility buffs, itself and F Blink."
        }
        "cold_snap" => "Resets only learned Frostbolt, Frost Nova, Blizzard, Cone of Cold, Ice Barrier and Ice Block cooldowns, not itself. Does not add or extend shields; an existing Ice Barrier is not overwritten.",
        "barkskin" => "Reduces damage taken by 30% while active.",
        "evasion" => "Reduces damage taken by 50% while active.",
        "ice_block" => {
            "Automatically grants immunity below 65% health with an enemy within 5 yd. Cannot move, actively cast or use F Blink; existing missiles, areas and periodic damage continue, and cooldowns keep recovering."
        }
        "flash_of_light" | "holy_light" | "flash_heal" | "prayer_of_healing"
        | "healing_wave" | "lesser_healing_wave" | "healing_touch" => {
            "When below 85% health, automatically casts an instant heal on you; with no party, it never targets an enemy."
        }
        "regrowth" => {
            "When below 85% health, restores part of its healing immediately and the rest over time."
        }
        "healing_stream_totem" => {
            "Places a friendly totem that heals you each second while you are in range; it deals no damage and does not explode."
        }
        "earthbind_totem" => {
            "Places a friendly totem that periodically slows nearby enemies; it deals no damage and does not explode."
        }
        "stoneclaw_totem" => {
            "Places a friendly totem that periodically reduces nearby enemies' damage; this mode uses it to simulate distraction."
        }
        "vampiric_embrace" => {
            "Marks an enemy; subsequent Shadow damage dealt to it restores 15% of that damage as health."
        }
        "curse_of_elements" => "Marks an enemy, increasing Fire and Frost damage it takes by 20%.",
        "curse_of_weakness" => {
            "Weakens the target for 10 sec, reducing its damage by 35%."
        }
        "curse_of_tongues" => {
            "Only targets caster enemies; delays their next attack and reduces its damage."
        }
        "curse_of_doom" => {
            "Curses the target for 8 sec, then deals heavy damage; it deals no damage on cast."
        }
        "expose_armor" => {
            "A melee armor break that increases subsequent weapon damage taken by 20%."
        }
        "drain_life" | "siphon_life" | "devouring_plague" => {
            "Each effective periodic-damage tick restores health according to the skill's ratio; overkill and corpses do not heal you."
        }
        "mind_flay" => "Channels a 3-sec damage effect that slows the target and deals damage each second.",
        "claw" | "rake" | "rip" | "swipe" | "ferocious_bite" => {
            "Uses a feral-themed melee technique without shifting form or requiring combo points; Rake and Rip deal periodic damage."
        }
        "backstab" => {
            "A melee strike that deals 50% more damage to controlled targets; no rear positioning is required in this mode."
        }
        "ambush" => {
            "Prioritizes nearly uninjured enemies for a heavy melee strike; stealth is not required."
        }
        _ => return None,
    })
}

pub(super) fn activation_condition(key: &str, zh_cn: bool) -> Option<&'static str> {
    Some(match key {
        "shield_wall" | "barkskin" | "evasion" => {
            if zh_cn {
                "生命低于65%、30码内有敌人且未处于无敌时自动触发；"
            } else {
                "Automatically triggers below 65% health when an enemy is within 30 yd and you are not immune; "
            }
        }
        "cold_snap" => {
            if zh_cn {
                "有冰霜技能正在冷却时自动触发；"
            } else {
                "Automatically triggers when a Frost skill is on cooldown; "
            }
        }
        "seal_of_righteousness" | "seal_of_command" | "seal_of_wisdom" | "seal_of_light" => {
            if zh_cn {
                "30码内有敌人且没有其他圣印生效时自动触发；"
            } else {
                "Automatically activates with an enemy within 30 yd when no other seal is active; "
            }
        }
        "windfury_weapon" | "rockbiter_weapon" | "frostbrand_weapon" => {
            if zh_cn {
                "30码内有敌人且没有其他武器附魔生效时自动触发；"
            } else {
                "Automatically activates with an enemy within 30 yd when no other weapon imbue is active; "
            }
        }
        "devotion_aura" | "inner_fire" | "frost_armor" | "mage_armor" | "dampen_magic"
        | "arcane_power" | "presence_of_mind" | "combustion" | "rapid_fire" | "deadly_poison"
        | "sweeping_strikes" | "blade_flurry" => {
            if zh_cn {
                "30码内有敌人且效果未生效时自动触发；"
            } else {
                "Automatically activates when an enemy is within 30 yd and the effect is inactive; "
            }
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::survivors::data::{self, Kind};

    #[test]
    fn shield_tooltips_cover_each_utility_skill_and_its_trigger() {
        for def in data::ABILITIES.iter().filter(|def| def.kind == Kind::Buff) {
            assert!(
                special_description(def.key, false).is_some(),
                "{} has no English effect description",
                def.key
            );
            assert!(
                activation_condition(def.key, false).is_some(),
                "{} has no English trigger description",
                def.key
            );
        }
    }

    #[test]
    fn arcane_power_and_combustion_describe_their_actual_effects_in_both_languages() {
        for (key, term) in [
            ("arcane_power", "30%"),
            ("combustion", "10 percentage points"),
        ] {
            let english = special_description(key, false).unwrap();
            assert!(english.contains(term), "{key}: {english}");
            assert!(!english.contains("absorb"), "{key}: {english}");
            assert!(special_description(key, true).is_some());
        }
    }

    #[test]
    fn polymorph_description_leaves_target_count_and_duration_dynamic() {
        let english = special_description("polymorph", false).unwrap();
        let chinese = special_description("polymorph", true).unwrap();
        assert!(english.contains("20 percentage points"));
        assert!(chinese.contains("20个百分点"));
        assert!(!english.contains("5 sec"));
        assert!(!english.contains("3 nearest"));
        assert!(!chinese.contains("5秒"));
        assert!(!chinese.contains("最多3"));
    }
}
