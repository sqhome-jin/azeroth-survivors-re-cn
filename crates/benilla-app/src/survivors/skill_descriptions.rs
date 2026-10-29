//! Survivors names and source-level explanations, independent of the installed DBC locale.

use super::{
    class_mechanics,
    data::{AbilityDef, Kind, Stat},
    gear::{self, EnhKind, Gear},
    growth, localization, skill_effects, talents, warlock, Run,
};

pub(super) fn ability_name(key: &str) -> Option<&'static str> {
    let key = key
        .to_ascii_lowercase()
        .replace([':', '\''], "")
        .replace(' ', "_");
    if let Some(passive) = super::data::PASSIVES.iter().find(|p| {
        p.name
            .to_ascii_lowercase()
            .replace([':', '\''], "")
            .replace(' ', "_")
            == key
            && p.key != key
    }) {
        return ability_name(passive.key);
    }
    Some(match key.as_str() {
        "frostbolt" => "寒冰箭",
        "fireball" => "火球术",
        "arcane_missiles" => "奥术飞弹",
        "frost_nova" => "冰霜新星",
        "arcane_explosion" => "魔爆术",
        "blizzard" => "暴风雪",
        "flamestrike" => "烈焰风暴",
        "fire_blast" => "火焰冲击",
        "cone_of_cold" => "冰锥术",
        "blast_wave" => "冲击波",
        "pyroblast" => "炎爆术",
        "cleave" => "顺劈斩",
        "heroic_strike" => "英勇打击",
        "whirlwind" => "旋风斩",
        "thunder_clap" => "雷霆一击",
        "sweeping_strikes" => "横扫攻击",
        "demoralizing_shout" => "挫志怒吼",
        "execute" => "斩杀",
        "mortal_strike" => "致死打击",
        "retaliation" => "反击风暴",
        "sinister_strike" => "邪恶攻击",
        "eviscerate" => "剔骨",
        "blade_flurry" => "剑刃乱舞",
        "rupture" => "割裂",
        "gouge" => "凿击",
        "kidney_shot" => "肾击",
        "smite" => "惩击",
        "shadow_word_pain" => "暗言术：痛",
        "holy_fire" => "神圣之火",
        "mind_blast" => "心灵震爆",
        "holy_nova" => "神圣新星",
        "psychic_scream" => "心灵尖啸",
        "power_word_shield" => "真言术：盾",
        "shadow_bolt" => "暗影箭",
        "corruption" => "腐蚀术",
        "curse_of_agony" => "痛苦诅咒",
        "immolate" => "献祭",
        "rain_of_fire" => "火焰之雨",
        "hellfire" => "地狱烈焰",
        "death_coil" => "死亡缠绕",
        "searing_pain" => "灼热之痛",
        "shadowburn" => "暗影灼烧",
        "arcane_shot" => "奥术射击",
        "multi_shot" => "多重射击",
        "serpent_sting" => "毒蛇钉刺",
        "volley" => "乱射",
        "explosive_trap" => "爆炸陷阱",
        "concussive_shot" => "震荡射击",
        "aimed_shot" => "瞄准射击",
        "lightning_bolt" => "闪电箭",
        "chain_lightning" => "闪电链",
        "earth_shock" => "地震术",
        "flame_shock" => "烈焰震击",
        "frost_shock" => "冰霜震击",
        "searing_totem" => "灼热图腾",
        "magma_totem" => "熔岩图腾",
        "lightning_shield" => "闪电之盾",
        "hammer_of_wrath" => "愤怒之锤",
        "consecration" => "奉献",
        "holy_shock" => "神圣震击",
        "exorcism" => "驱邪术",
        "holy_wrath" => "神圣愤怒",
        "hammer_of_justice" => "制裁之锤",
        "divine_shield" => "圣盾术",
        "judgement" => "审判",
        "wrath" => "愤怒",
        "moonfire" => "月火术",
        "starfire" => "星火术",
        "hurricane" => "飓风",
        "insect_swarm" => "虫群",
        "entangling_roots" => "纠缠根须",
        "thorns" => "荆棘术",
        "rejuvenation" => "回春术",
        "charge" => "冲锋",
        "blessing_of_freedom" => "自由祝福",
        "aspect_of_the_cheetah" => "猎豹守护",
        "sprint" => "疾跑",
        "fade" => "渐隐术",
        "ghost_wolf" => "幽魂之狼",
        "blink" => "闪现术",
        "howl_of_terror" => "恐惧嚎叫",
        "dash" => "急奔",
        "might" | "blessing_of_might" => "力量祝福",
        "fortitude" | "power_word_fortitude" => "真言术：韧",
        "pack" | "aspect_of_the_pack" => "豹群守护",
        "renew" => "恢复",
        "intellect" | "arcane_intellect" => "奥术智慧",
        "amplify" | "amplify_magic" => "魔法增效",
        "detect" | "detect_lesser_invisibility" => "侦测次级隐形",
        "wisdom" | "blessing_of_wisdom" => "智慧祝福",
        "wild" | "mark_of_the_wild" => "野性印记",
        "kings" | "blessing_of_kings" => "王者祝福",
        "warrior_bleed" => "重伤",
        "ignite" => "点燃",
        _ => {
            return super::spell_expansion::name(&key).or_else(|| {
                super::data::PASSIVES
                    .iter()
                    .find(|p| {
                        p.name
                            .to_ascii_lowercase()
                            .replace([':', '\''], "")
                            .replace(' ', "_")
                            == key
                    })
                    .and_then(|p| super::spell_expansion::name(p.key))
            });
        }
    })
}

fn chinese(text: &str) -> bool {
    text.chars().any(|c| ('\u{3400}'..='\u{9fff}').contains(&c))
}

pub(super) fn ability_name_for(locale: &str, key: &str, dbc: Option<&str>) -> String {
    let dbc = dbc.filter(|name| !name.trim().is_empty());
    if crate::char_select::locale_is_zh_cn(locale) {
        if let Some(name) = dbc.filter(|name| chinese(name)) {
            return name.to_owned();
        }
        if let Some(name) = ability_name(key) {
            return name.to_owned();
        }
    }
    dbc.map(str::to_owned)
        .unwrap_or_else(|| key.replace('_', " "))
}

pub(super) fn mobility_description(class: u8) -> &'static str {
    let Some(m) = super::mobility::for_class(class) else {
        return "";
    };
    localization::text(
        match class {
            1 => "Charge the nearest enemy within 25 yards, stunning it for 1.5 sec; lunge forward if none is in range. Boss stun duration is 30%.",
            2 => "Run 60% faster for 5 sec. Does not additionally remove control effects in this mode.",
            7 => "Run 60% faster for 6 sec. Does not change the character model in this mode.",
            9 => "Fear ordinary enemies within 12 yards for 4 sec. Bosses are immune to fear.",
            _ => m.desc,
        },
        match class {
            1 => "冲向25码内最近的敌人，使其昏迷1.5秒；没有目标时向前冲刺。首领昏迷时间为30%。",
            2 => "移动速度提高60%，持续5秒。本模式不额外解除控制。",
            3 => "移动速度提高70%，持续5秒。",
            4 | 11 => "移动速度提高80%，持续4秒。",
            5 => "敌人停止追逐你，移动速度提高40%，持续4秒。",
            7 => "移动速度提高60%，持续6秒。本模式不切换模型形态。",
            8 => "向前传送18码。",
            9 => "使12码内的普通敌人恐惧逃跑4秒。首领免疫恐惧。",
            _ => "",
        },
    )
}

fn name(key: &str) -> String {
    ability_name_for(&crate::char_select::ui_locale(), key, None)
}

fn potency(def: &AbilityDef) -> bool {
    def.damage > 0.0 || def.dot > 0.0
}

fn damaging(def: &AbilityDef) -> bool {
    potency(def) && !matches!(def.kind, Kind::Shield | Kind::Heal | Kind::Immune)
}

fn can_crit(run: &Run, def: &AbilityDef) -> bool {
    damaging(def)
        && (def.kind != Kind::Dot
            || talents::periodic_crits(
                &run.stats(),
                skill_effects::Source {
                    ability: Some(def.key),
                    ..Default::default()
                },
            ))
}

fn relevant_stat(run: &Run, def: &AbilityDef, stat: Stat) -> bool {
    match stat {
        Stat::Might => potency(def),
        Stat::Haste => true,
        Stat::Area => def.radius > 0.0,
        Stat::Crit => can_crit(run, def),
        Stat::Luck | Stat::Range | Stat::Targets => potency(def),
        Stat::Echo | Stat::Leech => damaging(def),
        Stat::Duration => def.duration > 0.0,
        Stat::CritDamage => can_crit(run, def),
        Stat::Shield => def.kind == Kind::Shield,
        _ => false,
    }
}

fn relevant_affix(run: &Run, def: &AbilityDef, kind: u8) -> bool {
    match kind {
        0 => potency(def),
        4 => true,
        5 => def.radius > 0.0,
        9 | 10 => can_crit(run, def),
        11 => damaging(def),
        _ => false,
    }
}

/// Dynamic relationships include both the proc's source and its beneficiaries.
/// Static changes are detected from the actual cast pipeline below, not a second value table.
fn dynamic_talent(run: &Run, def: &AbilityDef, slot: usize) -> bool {
    let key = def.key;
    let melee = matches!(
        key,
        "cleave"
            | "heroic_strike"
            | "whirlwind"
            | "sweeping_strikes"
            | "execute"
            | "mortal_strike"
            | "bloodthirst"
            | "overpower"
            | "slam"
            | "shield_slam"
            | "shield_bash"
    );
    let shots = matches!(
        key,
        "arcane_shot" | "multi_shot" | "aimed_shot" | "concussive_shot"
    );
    let rogue = matches!(
        key,
        "sinister_strike"
            | "eviscerate"
            | "blade_flurry"
            | "gouge"
            | "kidney_shot"
            | "backstab"
            | "ambush"
    );
    let holy = matches!(
        key,
        "holy_shock"
            | "hammer_of_wrath"
            | "exorcism"
            | "holy_wrath"
            | "hammer_of_justice"
            | "judgement"
    );
    let priest_holy = matches!(key, "smite" | "holy_fire" | "holy_nova");
    let shadow = matches!(
        key,
        "mind_blast" | "shadow_word_pain" | "mind_flay" | "devouring_plague" | "shadowguard"
    );
    let shocks = matches!(key, "earth_shock" | "flame_shock" | "frost_shock");
    let lightning = matches!(key, "lightning_bolt" | "chain_lightning");
    let totem = matches!(key, "searing_totem" | "magma_totem");
    let arcane = matches!(key, "arcane_missiles" | "arcane_explosion");
    let fire = matches!(key, "fireball" | "fire_blast" | "blast_wave" | "pyroblast");
    let frost = matches!(
        key,
        "frostbolt" | "frost_nova" | "cone_of_cold" | "blizzard"
    );
    let affliction = warlock::affliction(Some(key));
    let warlock_direct = matches!(
        key,
        "shadow_bolt" | "immolate" | "death_coil" | "searing_pain" | "shadowburn"
    );
    match (run.class.id, slot) {
        (1, 5) => matches!(key, "heroic_strike" | "mortal_strike"),
        (1, 6 | 7 | 14) => melee,
        (1, 13) => true,
        (1, 15) => can_crit(run, def),
        (1, 21) => key == "thunder_clap",
        (2, 5) => key == "holy_shock",
        (2, 6) => matches!(key, "holy_shock" | "hammer_of_wrath"),
        (2, 7) => key == "holy_shock",
        (2, 13 | 14) => key == "consecration",
        (2, 21) => holy,
        (2, 22) => potency(def),
        (2, 23) => key == "hammer_of_wrath",
        (3, 5) => true,
        (3, 6 | 13 | 21) => shots,
        (3, 7) => potency(def),
        (3, 14) => matches!(key, "arcane_shot" | "aimed_shot"),
        (3, 15) => key == "multi_shot",
        (3, 22) => key == "explosive_trap",
        (3, 23) => key == "serpent_sting",
        (4, 5 | 21) => rogue,
        (4, 6) => matches!(key, "eviscerate" | "rupture"),
        (4, 7) => key == "rupture",
        (4, 13) => matches!(key, "sinister_strike" | "blade_flurry"),
        (4, 14 | 15) => true,
        (4, 23) => key == "sinister_strike",
        (5, 5 | 7) => true,
        (5, 6) => key == "power_word_shield",
        (5, 13) => matches!(key, "smite" | "holy_fire"),
        (5, 14 | 15) => priest_holy,
        (5, 21..=23) => shadow,
        (7, 5) => shocks || lightning,
        (7, 6 | 7) => lightning,
        (7, 13 | 15) => shocks || key == "lightning_shield",
        (7, 14) => true,
        (7, 21..=23) => totem,
        (8, 5) => arcane,
        (8, 6) => key == "arcane_missiles",
        (8, 7) => true,
        (8, 13..=15) => fire,
        (8, 21) => frost && key != "blizzard",
        (8, 22) => key == "frost_nova",
        (8, 23) => frost,
        (9, 2 | 6 | 7) => affliction,
        (9, 4) => damaging(def) && (def.dot > 0.0 || def.duration > 0.0),
        (9, 5) => affliction || warlock_direct,
        (9, 14 | 16) => damaging(def),
        (9, 17) => warlock_direct,
        (9, 22) => true,
        (11, 5) => matches!(key, "wrath" | "starfire"),
        (11, 6) => matches!(key, "moonfire" | "starfire"),
        (11, 7) => matches!(key, "moonfire" | "insect_swarm"),
        (11, 13 | 15) => key == "thorns",
        (11, 14) => matches!(
            key,
            "wrath" | "starfire" | "moonfire" | "insect_swarm" | "entangling_roots"
        ),
        (11, 21 | 22) => key == "rejuvenation",
        (11, 23) => matches!(key, "wrath" | "starfire" | "rejuvenation"),
        _ => false,
    }
}

fn cast_signature(c: &talents::Cast) -> [f32; 10] {
    [
        c.dmg,
        c.dot,
        c.cooldown,
        c.radius,
        c.duration,
        c.range,
        c.count as f32,
        c.modifiers.control,
        c.modifiers.frequency,
        c.modifiers.chain,
    ]
}

fn static_talent(run: &Run, def: &'static AbilityDef, slot: usize) -> bool {
    let mut probe = Run {
        class: run.class,
        ..Default::default()
    };
    let before = talents::cast_for_run(def, 1, &probe);
    let before_crit = probe.stats().crit_multiplier();
    probe.talents.0[slot] = run.talents.0[slot];
    let after = talents::cast_for_run(def, 1, &probe);
    cast_signature(&before) != cast_signature(&after)
        || (can_crit(run, def)
            && (probe.stats().crit != 0.05 || probe.stats().crit_multiplier() != before_crit))
}

fn active_talent_affects(run: &Run, def: &AbilityDef, slot: usize, label: &str) -> bool {
    match (run.class.id, slot) {
        (2, 6) => def.key == "hammer_of_wrath",
        (2, 7) => def.key == "holy_shock",
        (7, 5) => matches!(def.key, "lightning_bolt" | "chain_lightning"),
        (7, 13) => matches!(def.key, "earth_shock" | "flame_shock" | "frost_shock"),
        (8, 23) => matches!(def.key, "frostbolt" | "frost_nova" | "cone_of_cold"),
        (11, 5) => {
            (def.key == "starfire" && label.contains("Starfire"))
                || (def.key == "wrath" && label.contains("Wrath"))
        }
        _ => dynamic_talent(run, def, slot),
    }
}

fn effective(m: skill_effects::Modifiers, kind: EnhKind) -> Option<(f32, Option<f32>)> {
    Some(match kind {
        EnhKind::Damage => (m.damage, None),
        EnhKind::Cooldown => (m.cooldown, Some(0.6)),
        EnhKind::Count => (m.count as f32, Some(6.0)),
        EnhKind::Radius => (m.radius, Some(1.0)),
        EnhKind::Duration => (m.duration, Some(1.0)),
        EnhKind::Frequency => (m.frequency, Some(2.0)),
        EnhKind::Range => (m.range, Some(1.0)),
        EnhKind::ChainPower => (m.chain, Some(0.15)),
        EnhKind::Echo => (m.echo, Some(0.5)),
        EnhKind::Splash => (m.splash, Some(1.0)),
        EnhKind::Ember => (m.ember, Some(1.0)),
        EnhKind::Execute => (m.execute, Some(2.0)),
        EnhKind::Leech => (m.leech, Some(0.2)),
        EnhKind::Control => (m.control, Some(0.5)),
        _ => return None,
    })
}

fn exact(kind: EnhKind, value: f32) -> String {
    if kind == EnhKind::Count {
        format!("+{}", value.round() as u32)
    } else {
        format!(
            "{}{:.2}%",
            if kind == EnhKind::Cooldown { "-" } else { "+" },
            value * 100.0
        )
    }
}

pub(super) fn mobility_details(run: &Run, gear: Option<&Gear>) -> Vec<String> {
    let tr = localization::text;
    let mut lines = vec![tr(
        "Mobility enhancements (learned at level 5):",
        "机动技能强化（局内5级解锁）：",
    )
    .into()];
    if let Some(gear) = gear {
        for slot in 0..gear::SLOT_COUNT {
            if let Some(item) = gear.worn(run.class.id, slot) {
                for roll in &item.affixes {
                    if roll.kind == 13 {
                        lines.push(format!("{} — {}", item.name(), gear::describe(roll)));
                    }
                }
            }
        }
    }
    lines.push(format!(
        "{}: {:.2}% ({} 60%)",
        tr("Equipment cooldown reduction", "装备冷却缩减"),
        (1.0 - run.gear.mobility_cooldown()) * 100.0,
        tr("cap", "上限")
    ));
    if let Some(nodes) = talents::nodes(run.class.id) {
        for (slot, node) in nodes.iter().enumerate() {
            let rank = run.talents.0[slot];
            if rank == 0 {
                continue;
            }
            let mut probe = Run {
                class: run.class,
                ..Default::default()
            };
            probe.talents.0[slot] = rank;
            if probe.mobility_cooldown() != 1.0
                || node.desc.starts_with("F ")
                || node.desc.starts_with("F:")
            {
                lines.push(format!(
                    "{} {rank}/{} — {}",
                    tr(node.name, node.zh),
                    if slot % 8 == 7 { 1 } else { 5 },
                    tr(node.desc, node.desc_zh)
                ));
            }
        }
    }
    lines.push(tr("Equipment and talent cooldown multipliers combine multiplicatively; automatic-skill haste does not shorten F's cooldown.", "装备与天赋冷却倍率乘算；自动技能急速不会缩短F键冷却。").into());
    lines
}

/// Every source comes from the run's locked snapshot. `gear` only resolves item names and
/// ordinary affixes, which are live-equipped (including equipment changed while paused).
pub(super) fn details(run: &Run, def: &'static AbilityDef, gear: Option<&Gear>) -> Vec<String> {
    let tr = localization::text;
    let stats = run.stats();
    let mut lines = vec![tr(
        "Enhancement sources (current snapshot):",
        "强化来源（当前快照）：",
    )
    .into()];
    let owned = run.abilities.iter().find(|o| o.def.key == def.key);
    let level = owned.map_or(1, |o| o.level);
    let acquired = owned.map_or(1.0, |o| o.growth);
    let mut rank_effects = vec![format!(
        "{} ×{:.3}",
        tr("base cooldown", "基础冷却"),
        growth::cooldown_multiplier(acquired)
    )];
    if potency(def) {
        rank_effects.push(format!(
            "{} ×{:.2}",
            tr("potency", "威力"),
            growth::damage(acquired)
        ));
    }
    if def.radius > 0.0 {
        rank_effects.push(format!(
            "{} ×{:.2}",
            tr("radius", "半径"),
            growth::radius_multiplier(acquired)
        ));
    }
    if def.duration > 0.0 {
        rank_effects.push(format!(
            "{} ×{:.2}",
            tr("duration", "持续时间"),
            growth::duration_multiplier(acquired)
        ));
    }
    if matches!(
        def.kind,
        Kind::Bolt | Kind::Strike | Kind::Melee | Kind::Chain | Kind::Dot | Kind::Orbit
    ) {
        rank_effects.push(format!(
            "{} +{}",
            tr("growth targets (diminishing)", "成长目标数（收益渐减）"),
            growth::bonus_targets(acquired),
        ));
    }
    lines.push(format!(
        "{} {level} · {} {acquired:.2}: {}",
        tr("Skill rank", "技能等级"),
        tr("growth", "累计成长"),
        rank_effects.join("; ")
    ));
    lines.push(tr(
        "Each card adds one rank; quality contributes 1 / 1.35 / 1.8 / 2.4 / 3.2 additive growth. No maximum rank.",
        "每张卡增加1级；普通/优秀/精良/史诗/传说分别累计1 / 1.35 / 1.8 / 2.4 / 3.2成长，无等级上限。",
    ).into());
    if localization::is_zh_cn() {
        if let Some(description) = super::spell_expansion::description(def.key) {
            lines.push(description.into());
        }
    }
    for &(p, rank) in &run.passives {
        if relevant_stat(run, def, p.stat) {
            lines.push(format!(
                "{}: {} {rank} — {} {:.2}; {} +{:.2}",
                tr("Passive", "局内被动"),
                name(p.key),
                tr("growth", "成长"),
                run.passive_growth(p),
                tr("raw contribution", "原始贡献"),
                p.per_level * run.passive_growth(p),
            ));
        }
    }
    if let Some(gear) = gear {
        for slot in 0..gear::SLOT_COUNT {
            if let Some(item) = gear.worn(run.class.id, slot) {
                for roll in &item.affixes {
                    if relevant_affix(run, def, roll.kind) {
                        lines.push(format!(
                            "{}: {} — {}",
                            tr("Equipment", "装备"),
                            item.name(),
                            gear::describe(roll)
                        ));
                    }
                }
            }
        }
    }
    let rolls: Vec<_> = run
        .gear
        .enh
        .iter()
        .flatten()
        .copied()
        .filter(|r| r.matches(def))
        .collect();
    for roll in &rolls {
        let item = gear.and_then(|g| g.item(roll.item)).map_or_else(
            || format!("{} #{}", tr("Equipment", "装备"), roll.item),
            |item| format!("{} #{}", item.name(), item.id),
        );
        let enh = gear::EnhRoll {
            ability: roll.ability.unwrap_or_default().into(),
            kind: gear::ENH_KINDS
                .iter()
                .position(|k| *k == roll.kind)
                .unwrap() as u8,
            value: (roll.value * 10_000.0).round() as u32,
        };
        lines.push(format!(
            "{item}: {} [{}]",
            gear::describe_enhancement(&enh, &name(def.key)),
            exact(roll.kind, roll.value)
        ));
    }
    let modifiers = skill_effects::Modifiers::from_stats(&stats, def);
    for kind in gear::ENH_KINDS {
        if !rolls.iter().any(|r| r.kind == kind) {
            continue;
        }
        if let Some((value, mut cap)) = effective(modifiers, kind) {
            if kind == EnhKind::Count && def.kind == Kind::Chain {
                cap = Some(8.0);
            }
            let cap = cap.map_or_else(
                || tr("uncapped", "无叠加上限").to_owned(),
                |v| format!("{} {}", tr("cap", "上限"), kind.show(v)),
            );
            lines.push(format!(
                "{}: {} {} ({cap})",
                tr("Effective equipment total", "装备实际合计"),
                kind.label(),
                exact(kind, value)
            ));
        }
    }
    if rolls.iter().any(|r| r.kind == EnhKind::Echo) {
        lines.push(
            tr(
                "Echo: 50% damage; no recursive echoes or talent procs.",
                "回响：造成50%伤害；派生伤害不会再次回响或触发天赋。",
            )
            .into(),
        );
    }
    if rolls.iter().any(|r| r.kind == EnhKind::Ember) {
        lines.push(
            tr(
                "Ember: total extra damage over 3 sec, one tick per second.",
                "余烬：额外总伤害分3秒结算，每秒一跳。",
            )
            .into(),
        );
    }
    if run.gear.leech() > 0.0 && damaging(def) {
        lines.push(format!(
            "{}: {:.2}%",
            tr("Equipment life leech", "装备生命偷取"),
            run.gear.leech() * 100.0
        ));
    }
    if let Some(nodes) = talents::nodes(run.class.id) {
        for (slot, node) in nodes.iter().enumerate() {
            let rank = run.talents.0[slot];
            if rank == 0 || (!static_talent(run, def, slot) && !dynamic_talent(run, def, slot)) {
                continue;
            }
            lines.push(format!(
                "{}: {} {rank}/{} — {}",
                tr("Talent (conditions apply)", "天赋（依所述条件生效）"),
                tr(node.name, node.zh),
                if slot % 8 == 7 { 1 } else { 5 },
                tr(node.desc, node.desc_zh),
            ));
        }
    }
    for (slot, en, zh, left) in class_mechanics::buffs(run) {
        if active_talent_affects(run, def, slot, en) {
            lines.push(format!(
                "{}: {} ({left:.1}{})",
                tr("Active class buff", "当前职业增益"),
                tr(en, zh),
                tr("s remaining", "秒剩余")
            ));
        }
    }
    if run.class.id == 5 && run.talents.0[5] > 0 && run.shield > 0.0 && run.shield_t > 0.0 {
        lines.push(format!(
            "{}: {} +{:.1}%",
            tr("Active", "当前生效"),
            tr("Borrowed Time / haste", "争分夺秒／急速"),
            f32::from(run.talents.0[5]) * 2.0
        ));
    }
    if warlock::haste(run) > 0.0 {
        lines.push(format!(
            "{}: {} +{:.1}% ({:.1}{})",
            tr("Active", "当前生效"),
            tr(talents::NODES[22].name, talents::NODES[22].zh),
            warlock::haste(run) * 100.0,
            warlock::haste_remaining(run),
            tr("s remaining", "秒剩余")
        ));
    }
    for (item_id, kind, value, remaining) in run.effects.active_buffs(run.elapsed) {
        let relevant = match kind {
            EnhKind::BuffDamage => potency(def),
            EnhKind::BuffHaste => true,
            EnhKind::BuffCrit => can_crit(run, def),
            _ => false,
        };
        if !relevant {
            continue;
        }
        let item = gear.and_then(|g| g.item(item_id)).map_or_else(
            || format!("{} #{item_id}", tr("Equipment", "装备")),
            |item| format!("{} #{item_id}", item.name()),
        );
        lines.push(format!(
            "{}: {item} — {} {} ({remaining:.1}{})",
            tr("Active equipment source", "当前装备增益来源"),
            kind.label(),
            exact(kind, value),
            tr("s remaining", "秒剩余"),
        ));
    }
    for (kind, relevant, cap) in [
        (EnhKind::BuffDamage, potency(def), 1.0),
        (EnhKind::BuffHaste, true, 0.4),
        (EnhKind::BuffCrit, can_crit(run, def), 0.3),
    ] {
        let value = run.effects.bonus(kind, run.elapsed);
        if relevant && value > 0.0 {
            lines.push(format!(
                "{}: {} {} ({} {})",
                tr("Active equipment buff", "当前装备增益"),
                kind.label(),
                exact(kind, value),
                tr("cap", "上限"),
                kind.show(cap)
            ));
        }
    }
    let mut totals = vec![format!(
        "{} {:.2}% ({} 65%)",
        tr("haste cooldown reduction", "急速冷却缩减"),
        stats.haste.min(0.65) * 100.0,
        tr("cap", "上限")
    )];
    if potency(def) {
        totals.push(format!("{} ×{:.2}", tr("potency", "威力"), stats.might));
    }
    if def.radius > 0.0 {
        totals.push(format!("{} ×{:.2}", tr("area", "范围"), stats.area));
    }
    lines.push(format!(
        "{}: {}. {}",
        tr("Combined stats", "通用合计"),
        totals.join("; "),
        tr(
            "Final skill cooldown cannot fall below 0.2 sec.",
            "技能最终冷却不低于0.2秒。"
        )
    ));
    if damaging(def) && (run.leech() > 0.0 || modifiers.leech > 0.0 || run.talents.spent() > 0) {
        lines.push(tr("Leech: effective damage only; shared recovery budget is 4% maximum health per second. Target/proc conditions are not assumed active in the preview.", "吸血仅按有效伤害结算，共享每秒4%生命上限的恢复预算。预览不假定目标条件或概率触发已经满足。").into());
    }
    if def.kind == Kind::Immune {
        lines.push(
            tr(
                "Immunity duration cannot exceed 50% of its final cooldown.",
                "免疫持续时间不超过最终冷却的50%。",
            )
            .into(),
        );
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::super::data;
    use super::*;

    fn run(class: u8) -> Run {
        Run {
            class: data::CLASSES.iter().find(|c| c.id == class).unwrap(),
            ..Default::default()
        }
    }

    #[test]
    fn every_ability_passive_and_mobility_has_a_chinese_fallback() {
        for def in data::ABILITIES {
            let translated = ability_name(def.key).unwrap_or_else(|| panic!("{}", def.key));
            assert!(chinese(translated));
            assert_eq!(
                ability_name_for("zhCN", def.key, Some("English DBC")),
                translated
            );
            assert_eq!(ability_name_for("zhCN", def.key, None), translated);
        }

        for p in data::PASSIVES {
            assert!(chinese(ability_name(p.key).unwrap()));
            assert!(chinese(ability_name(p.name).unwrap()));
        }
        for c in data::CLASSES {
            let m = super::super::mobility::for_class(c.id).unwrap();
            assert!(chinese(ability_name(m.name).unwrap()));
        }
    }

    #[test]
    fn expanded_spells_only_list_their_actual_talent_families() {
        for key in [
            "bloodthirst",
            "overpower",
            "slam",
            "shield_slam",
            "shield_bash",
        ] {
            assert!(dynamic_talent(&run(1), data::ability(key), 6), "{key}");
            assert!(!dynamic_talent(&run(1), data::ability(key), 21), "{key}");
        }
        for key in ["backstab", "ambush"] {
            assert!(dynamic_talent(&run(4), data::ability(key), 5), "{key}");
            assert!(dynamic_talent(&run(4), data::ability(key), 21), "{key}");
            assert!(!dynamic_talent(&run(4), data::ability(key), 6), "{key}");
        }
        for key in ["mind_flay", "devouring_plague", "shadowguard"] {
            assert!(dynamic_talent(&run(5), data::ability(key), 21), "{key}");
            assert!(!dynamic_talent(&run(5), data::ability(key), 14), "{key}");
        }
        for key in ["siphon_life", "drain_life", "drain_soul", "curse_of_doom"] {
            for slot in [2, 5, 6, 7] {
                assert!(
                    dynamic_talent(&run(9), data::ability(key), slot),
                    "{key} slot {slot}"
                );
            }
            assert!(!dynamic_talent(&run(9), data::ability(key), 17), "{key}");
        }
    }

    #[test]
    fn localized_dbc_is_preserved_and_english_locale_is_unchanged() {
        assert_eq!(
            ability_name_for("zhCN", "frostbolt", Some("本地寒冰箭")),
            "本地寒冰箭"
        );
        assert_eq!(
            ability_name_for("enUS", "frostbolt", Some("Frostbolt")),
            "Frostbolt"
        );
        assert_eq!(
            ability_name_for("deDE", "frostbolt", Some("Frostblitz")),
            "Frostblitz"
        );
        assert_eq!(ability_name_for("enUS", "unknown_key", None), "unknown key");
        assert_eq!(ability_name_for("zhCN", "frostbolt", Some("")), "寒冰箭");
    }

    #[test]
    fn exact_equipment_caps_and_all_skill_compatibility_follow_runtime() {
        let mut run = run(8);
        for (i, kind) in [
            EnhKind::Cooldown,
            EnhKind::Cooldown,
            EnhKind::Count,
            EnhKind::Echo,
        ]
        .into_iter()
        .enumerate()
        {
            run.gear.enh[i] = Some(gear::SkillRoll {
                item: i as u64 + 1,
                index: 0,
                ability: None,
                kind,
                value: 10.0,
            });
        }
        let frostbolt = data::ability("frostbolt");
        let m = skill_effects::Modifiers::from_stats(&run.stats(), frostbolt);
        assert_eq!(effective(m, EnhKind::Cooldown), Some((0.6, Some(0.6))));
        assert_eq!(m.count, 6);
        assert_eq!(m.echo, 0.5);
        let text = details(&run, frostbolt, None).join("\n");
        assert!(text.contains("60%"));
        assert!(text.contains("50%"));
        assert!(text.contains("#1") && text.contains("#2"));
        assert!(!details(&run, data::ability("blizzard"), None)
            .iter()
            .any(|line| line.starts_with("Equipment #4:") || line.starts_with("装备 #4:")));
    }

    #[test]
    fn static_and_dynamic_talents_are_skill_and_class_specific() {
        let mut mage = run(8);
        mage.talents.0[8] = 3; // Fireball damage.
        mage.talents.0[13] = 2; // Ignite.
        assert!(static_talent(&mage, data::ability("fireball"), 8));
        assert!(!static_talent(&mage, data::ability("frostbolt"), 8));
        assert!(dynamic_talent(&mage, data::ability("fireball"), 13));
        assert!(!dynamic_talent(&mage, data::ability("flamestrike"), 13));
        let frost = details(&mage, data::ability("frostbolt"), None).join("\n");
        assert!(!frost.contains("Ignite") && !frost.contains("点燃"));
        assert!(!frost.contains("Deep Wounds") && !frost.contains("重伤"));
        let fire = details(&mage, data::ability("fireball"), None).join("\n");
        let nodes = talents::nodes(8).unwrap();
        assert!(fire.contains(localization::text(nodes[8].name, nodes[8].zh)));
        assert!(fire.contains("3/5"));
        assert!(fire.contains(localization::text(nodes[13].desc, nodes[13].desc_zh)));
        assert!(!fire.contains("Active class buff:") && !fire.contains("当前职业增益："));
    }

    #[test]
    fn generic_defense_does_not_leak_into_damage_skill_sources() {
        let mage = run(8);
        for kind in [1, 2, 3, 6, 7, 8, 12, 13] {
            assert!(!relevant_affix(&mage, data::ability("fireball"), kind));
        }
        assert!(!relevant_affix(&mage, data::ability("corruption"), 9));
        let mut warlock = run(9);
        warlock.talents.0[7] = 1;
        assert!(relevant_affix(&warlock, data::ability("corruption"), 9));
    }

    #[test]
    fn active_class_buffs_are_distinguished_from_trigger_conditions() {
        let mut warrior = run(1);
        warrior.talents.0[15] = 1;
        let def = data::ability("cleave");
        let inactive = details(&warrior, def, None).join("\n");
        assert!(!inactive.contains("Active class buff") && !inactive.contains("当前职业增益"));
        class_mechanics::on_mobility(&mut warrior);
        let active = details(&warrior, def, None).join("\n");
        assert!(active.contains(localization::text("Active class buff", "当前职业增益")));
        assert!(active.contains(localization::text("Recklessness", "鲁莽")));
        let druid = run(11);
        assert!(!active_talent_affects(
            &druid,
            data::ability("wrath"),
            5,
            "Nature's Grace: Starfire"
        ));
        assert!(active_talent_affects(
            &druid,
            data::ability("starfire"),
            5,
            "Nature's Grace: Starfire"
        ));
    }

    #[test]
    fn all_caps_are_read_from_effective_modifiers_not_uncapped_roll_sums() {
        let mut shaman = run(7);
        for (i, kind) in gear::ENH_KINDS
            .into_iter()
            .filter(|kind| !kind.is_buff())
            .enumerate()
        {
            shaman.gear.enh[i] = Some(gear::SkillRoll {
                item: i as u64 + 1,
                index: 0,
                ability: None,
                kind,
                value: 100.0,
            });
        }
        let def = data::ability("chain_lightning");
        let m = skill_effects::Modifiers::from_stats(&shaman.stats(), def);
        assert_eq!(m.damage, 100.0);
        assert_eq!(m.count, 8);
        assert_eq!(m.chain, 0.15);
        assert_eq!(m.echo, 0.5);
        assert_eq!(m.execute, 2.0);
        assert_eq!(m.leech, 0.2);
        let text = details(&shaman, def, None).join("\n");
        assert!(text.contains("+15.00%"));
        assert!(text.contains("+8"));
        assert!(!text.contains("10000.00% (cap"));
    }

    #[test]
    fn equipment_item_names_are_resolved_without_changing_the_locked_snapshot() {
        let mut run = run(8);
        let mut gear = Gear::default();
        let id = gear.add(gear::Item {
            id: 0,
            slot: 0,
            rarity: 2,
            base: 0,
            class: 8,
            affixes: vec![
                gear::Roll {
                    kind: 0,
                    value: 1000,
                },
                gear::Roll {
                    kind: 8,
                    value: 10000,
                },
            ],
            enhancement: None,
            enhancements: vec![gear::EnhRoll {
                ability: "fireball".into(),
                kind: 0,
                value: 4000,
            }],
        });
        assert!(gear.equip(8, id));
        run.gear = gear.bonuses(8);
        let text = details(&run, data::ability("fireball"), Some(&gear)).join("\n");
        assert!(text.contains(gear.item(id).unwrap().name()));
        assert!(text.contains("40%") && text.contains("10.0%"));
        assert!(!text.contains(gear::affix(8).en) && !text.contains(gear::affix(8).zh));
        assert_eq!(run.gear.enh[0].unwrap().value, 0.4);
    }
}
