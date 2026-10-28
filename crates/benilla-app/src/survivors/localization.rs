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
    }

    #[test]
    fn unknown_locale_and_terms_keep_the_english_fallback() {
        assert_eq!(choose("enUS", "Mage", "法师"), "Mage");
        assert_eq!(map_name_for("zhCN", "A custom map"), "A custom map");
        assert_eq!(zone_name_for("enUS", "Kalimdor"), "Kalimdor");
    }
}
