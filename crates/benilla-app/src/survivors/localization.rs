use crate::char_select::{locale_is_zh_cn, ui_locale};

fn choose<'a>(locale: &str, english: &'a str, chinese: &'a str) -> &'a str {
    if locale_is_zh_cn(locale) {
        chinese
    } else {
        english
    }
}

pub(super) fn text(english: &'static str, chinese: &'static str) -> &'static str {
    choose(&ui_locale(), english, chinese)
}

pub(super) fn is_zh_cn() -> bool {
    locale_is_zh_cn(&ui_locale())
}

pub(super) fn victory_line(english: &'static str) -> &'static str {
    victory_line_for(&ui_locale(), english)
}

fn victory_line_for(locale: &str, english: &'static str) -> &'static str {
    let chinese = match english {
        "The Barrens are yours." => "贫瘠之地已在你的掌握之中。",
        "The Dark Portal stands silent." => "黑暗之门归于沉寂。",
        "The Qiraji retreat behind the Scarab Wall." => "其拉虫人退回了甲虫之墙后。",
        "Even King Mosh has fallen." => "连莫什大王也已倒下。",
        "The arena roars your name." => "竞技场为你的名字欢呼。",
        "Elwynn Forest is safe once more." => "艾尔文森林再次恢复安宁。",
        "Durotar is quiet again." => "杜隆塔尔重归寂静。",
        "Teldrassil blooms again." => "泰达希尔再次焕发生机。",
        _ => english,
    };
    choose(locale, english, chinese)
}

pub(super) fn race_name(race: u8, english: &'static str) -> &'static str {
    race_name_for(&ui_locale(), race, english)
}

fn race_name_for(locale: &str, race: u8, english: &'static str) -> &'static str {
    let chinese = match race {
        1 => "人类",
        2 => "兽人",
        3 => "矮人",
        4 => "暗夜精灵",
        5 => "亡灵",
        6 => "牛头人",
        7 => "侏儒",
        8 => "巨魔",
        _ => english,
    };
    choose(locale, english, chinese)
}

pub(super) fn class_name(class: u8, english: &'static str) -> &'static str {
    class_name_for(&ui_locale(), class, english)
}

fn class_name_for(locale: &str, class: u8, english: &'static str) -> &'static str {
    let chinese = match class {
        1 => "战士",
        2 => "圣骑士",
        3 => "猎人",
        4 => "潜行者",
        5 => "牧师",
        7 => "萨满祭司",
        8 => "法师",
        9 => "术士",
        11 => "德鲁伊",
        _ => english,
    };
    choose(locale, english, chinese)
}

pub(super) fn map_name(english: &'static str) -> &'static str {
    map_name_for(&ui_locale(), english)
}

fn map_name_for(locale: &str, english: &'static str) -> &'static str {
    let chinese = match english {
        "The Barrens" => "贫瘠之地",
        "The Dark Portal" => "黑暗之门",
        "Gates of Ahn'Qiraj" => "安其拉之门",
        "Fire Plume Ridge" => "火羽山",
        "Gurubashi Arena" => "古拉巴什竞技场",
        "Elwynn Forest" => "艾尔文森林",
        "Durotar" => "杜隆塔尔",
        "Teldrassil" => "泰达希尔",
        "Dun Morogh" => "丹莫罗",
        "Mulgore" => "莫高雷",
        "Tirisfal Glades" => "提瑞斯法林地",
        "Westfall" => "西部荒野",
        "Duskwood" => "暮色森林",
        "Redridge Mountains" => "赤脊山",
        "Mount Hyjal" => "海加尔山",
        "Naxxramas" => "纳克萨玛斯",
        _ => english,
    };
    choose(locale, english, chinese)
}

pub(super) fn map_blurb(english: &'static str) -> &'static str {
    map_blurb_for(&ui_locale(), english)
}

fn map_blurb_for(locale: &str, english: &'static str) -> &'static str {
    let (en, zh) = match english {
        "The Barrens" => (
            "Endless plains where raptors and quillboar roam. Face Echeyakee, Humar and finally Hezrul Bloodmark.",
            "迅猛龙与野猪人游荡的平原，迎战埃其亚基、狮王休玛，最终击败赫兹鲁尔·血印。",
        ),
        "The Dark Portal" => (
            "Fel beasts and demons pour through the Portal. Wave bosses end with Lord Kazzak.",
            "邪能野兽与恶魔不断涌出黑暗之门，最终首领为末日领主卡扎克。",
        ),
        "Gates of Ahn'Qiraj" => (
            "Silithid swarms before the sealed temple. Fight on to Ossirian the Unscarred.",
            "封印神殿前虫群汹涌，一路杀至无疤者奥斯里安。",
        ),
        "Fire Plume Ridge" => (
            "Devilsaurs and fire elementals in Un'Goro's volcano. King Mosh waits at the end.",
            "安戈洛火山中恐暴龙与火焰元素横行，终点是莫什王。",
        ),
        "Gurubashi Arena" => (
            "Troll bloodlords and beasts of Zul'Gurub. Survive until Hakkar, the Soulflayer.",
            "祖尔格拉布的巨魔与野兽齐聚，坚持到魔灵哈卡降临。",
        ),
        "Elwynn Forest" => (
            "Gnolls, bandits and wolves near Goldshire. Hogger leads the bosses.",
            "闪金镇周边的豺狼人、盗贼与野狼，首领包括霍格。",
        ),
        "Durotar" => (
            "Scorching badlands of the Razormane and Burning Blade. A fast, aggressive early zone.",
            "钢鬃野猪人与火刃氏族盘踞的灼热荒地，前期节奏偏快。",
        ),
        "Teldrassil" => (
            "The world tree's forest, haunted by Gnarlpine furbolgs and corrupted wildlife.",
            "世界之树的森林，被瘤背熊怪与受腐蚀的野兽侵扰。",
        ),
        "Dun Morogh" => (
            "Frozen peaks with wolves, bears and troggs. Beware Great Father Arctikus.",
            "冰封山岭中狼、熊与穴居怪出没，小心霜鬃巨魔阿尔蒂库斯。",
        ),
        "Mulgore" => (
            "Rolling plains of the Tauren. Beasts roam in packs and bosses arrive one by one.",
            "牛头人的连绵草原，鸟类与野兽成群，首领将随波次逐一登场。",
        ),
        "Tirisfal Glades" => (
            "Undead-haunted farmland and graveyards. Ghosts and ghouls wait for the unwary.",
            "亡灵出没的农田与坟地，幽魂与食尸鬼潜伏在暗处。",
        ),
        "Westfall" => (
            "Defias thieves and wild beasts across the farmlands. Defeat Klaven Mortwake in the finale.",
            "迪菲亚盗贼与野兽遍布农田，最终迎战克拉文·摩特维克。",
        ),
        "Duskwood" => (
            "Gloomy woods of worgen, spiders and the undead. Face Nefaru at the climax.",
            "幽暗的密林里有狼人、蜘蛛与亡灵，最终面对尼法鲁。",
        ),
        "Redridge Mountains" => (
            "Blackrock orcs and gnolls hold the red cliffs. Waves of bosses test your burst damage.",
            "黑石兽人与豺狼人占据赤红山崖，一波接一波的首领考验你的爆发。",
        ),
        "Mount Hyjal" => (
            "Corrupted dragons and fel demons. The four Emerald Dragons arrive as bosses.",
            "腐化的龙族与邪能恶魔横行，四位翡翠巨龙将作为首领登场。",
        ),
        "Naxxramas" => (
            "The floating necropolis. Arachnids, abominations and Kel'Thuzad, the hardest finale.",
            "漂浮的亡灵要塞，蛛魔与憎恶遍布，终点是最强的克尔苏加德。",
        ),
        _ => ("", ""),
    };
    choose(locale, en, zh)
}

pub(super) fn zone_name(english: &'static str) -> &'static str {
    zone_name_for(&ui_locale(), english)
}

fn zone_name_for(locale: &str, english: &'static str) -> &'static str {
    let chinese = match english {
        "Kalimdor" => "卡利姆多",
        "Blasted Lands" => "诅咒之地",
        "Silithus" => "希利苏斯",
        "Un'Goro Crater" => "安戈洛环形山",
        "Stranglethorn Vale" => "荆棘谷",
        "Eastern Kingdoms" => "东部王国",
        "Eastern Plaguelands" => "东瘟疫之地",
        _ => english,
    };
    choose(locale, english, chinese)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zh_cn_names_cover_menu_entities_and_maps() {
        assert_eq!(race_name_for("zhCN", 4, "Night Elf"), "暗夜精灵");
        assert_eq!(class_name_for("zhCN", 8, "Mage"), "法师");
        assert_eq!(map_name_for("zhCN", "The Barrens"), "贫瘠之地");
        assert_eq!(zone_name_for("zhCN", "Eastern Kingdoms"), "东部王国");
        assert_eq!(zone_name_for("zhCN", "Eastern Plaguelands"), "东瘟疫之地");
        assert_eq!(
            victory_line_for("zhCN", "The Barrens are yours."),
            "贫瘠之地已在你的掌握之中。"
        );
    }

    #[test]
    fn unknown_locale_and_terms_keep_the_english_fallback() {
        assert_eq!(choose("enUS", "Mage", "法师"), "Mage");
        assert_eq!(map_name_for("zhCN", "A custom map"), "A custom map");
        assert_eq!(zone_name_for("enUS", "Kalimdor"), "Kalimdor");
    }
}
