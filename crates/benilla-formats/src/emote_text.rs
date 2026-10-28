//! `EmotesText.dbc` joined with `EmotesTextData.dbc`: the sentence an `SMSG_TEXT_EMOTE` becomes
//! ("Bob waves at you."). [`crate::emotes`] owns the same table's command, animation and voice
//! columns; this module owns its 16 `EmoteText[]` columns and the strings they point into.
//!
//! The composer is the reference's `0x49b200`: a 4-bit column selector, a four-rung fallback when
//! the column has no string for the locale, then one of four `SStrPrintf` sites, performer first.
//! All four rungs empty means no chat line at all; the emote is then only an animation and a voice.

use std::collections::HashMap;

use anyhow::{Context, Result};
use benilla_dbc::{FieldType, Schema, SchemaField};

use crate::Chain;
use crate::dbc::{parse, str_at, u32_at};

const EMOTES_TEXT: &str = "DBFilesClient\\EmotesText.dbc";
const EMOTES_TEXT_DATA: &str = "DBFilesClient\\EmotesTextData.dbc";

/// `EmoteText[]`'s width, which the 4-bit selector indexes.
const FORMS: usize = 16;

/// Locale columns per `EmotesTextData.dbc` row: id, 8 strings and flags make the 10 fields the
/// loader checks (`0x544579`); the client reads the slot named in `[0xc0e080]`.
const LOCALES: usize = 8;

/// The joined text-emote sentence tables.
#[derive(Debug, Default, Clone)]
pub struct EmoteTextCatalog {
    /// `EmotesText.dbc` id to its 16 `EmoteText[]` columns, each an `EmotesTextData` id or 0.
    forms: HashMap<u32, [u32; FORMS]>,
    data: HashMap<u32, [Option<String>; LOCALES]>,
}

/// The facts `0x49b200` branches on. A self-emote never names you as its target: the sending
/// client clears a self-target before building the packet (`DoEmote`, `0x5ef611`), so the wire
/// carries it as untargeted.
#[derive(Debug, Clone, Copy)]
pub struct EmoteLine<'a> {
    /// The performer's resolved name, filled from its `NameCache` record.
    pub performer: &'a str,
    /// The performer guid is the active player's (`0x49b2c8`).
    pub performer_is_you: bool,
    /// The performer's gender flag selects the `+8` text columns (`0x49b31c`).
    pub performer_female: bool,
    /// The target name as the wire carries it; empty is untargeted (`0x49b311`).
    pub target: &'a str,
    /// Your own name, compared case-insensitively with `target` (`0x49b2f3`).
    pub your_name: &'a str,
}

impl EmoteTextCatalog {
    /// The chat line for a text emote; `None` when the locale fallback ladder runs dry
    /// (`0x49b4bd`). `locale` is the client's DBC string column, 4 for zhCN.
    pub fn compose(&self, text_id: u32, line: &EmoteLine, locale: usize) -> Option<String> {
        let forms = self.forms.get(&text_id)?;

        // The selector order matters: a matching performer skips the target compare entirely.
        let (mut performer_is_other, mut target_is_other) = (true, true);
        let mut ctx = 0usize;
        if line.performer_is_you {
            ctx = 2;
            performer_is_other = false;
        } else if !line.target.is_empty() && line.target.eq_ignore_ascii_case(line.your_name) {
            ctx = 1;
            target_is_other = false;
        }
        if line.target.is_empty() {
            ctx |= 4;
        }
        let gender = usize::from(line.performer_female) * 8;

        // The reference tries gendered then ungendered forms, and rewrites ctx before rung 3.
        let template = self
            .template(forms, gender | ctx, locale)
            .or_else(|| self.template(forms, ctx, locale))
            .or_else(|| {
                ctx = (ctx & !1) | 4;
                self.template(forms, gender | ctx, locale)
            })
            .or_else(|| self.template(forms, ctx, locale))?;

        // The four `SStrPrintf` sites (`0x49b3fa`-`0x49b479`) fill the performer first.
        Some(match (performer_is_other, target_is_other) {
            (true, true) => fill(template, &[line.performer, line.target]),
            (true, false) => fill(template, &[line.performer]),
            (false, true) => fill(template, &[line.target]),
            (false, false) => fill(template, &[]),
        })
    }

    /// One `EmoteText[index]` column's non-empty string for this locale, the rung test.
    fn template<'a>(
        &'a self,
        forms: &[u32; FORMS],
        index: usize,
        locale: usize,
    ) -> Option<&'a str> {
        let data_id = *forms.get(index)?;
        self.data.get(&data_id)?.get(locale)?.as_deref()
    }

    /// `EmotesText.dbc` rows, 169 in build 5875.
    pub fn len(&self) -> usize {
        self.forms.len()
    }

    pub fn is_empty(&self) -> bool {
        self.forms.is_empty()
    }
}

/// `SStrPrintf` (`0x64a7f0`) reduced to the `%s` these templates use. Deviation: a surplus `%s`
/// stays verbatim, where the reference would print stack garbage; no shipped row has one.
fn fill(template: &str, args: &[&str]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    let mut args = args.iter();
    while let Some(at) = rest.find("%s") {
        let Some(arg) = args.next() else { break };
        out.push_str(&rest[..at]);
        out.push_str(arg);
        rest = &rest[at + 2..];
    }
    out.push_str(rest);
    out
}

/// `EmotesText.dbc`: 19 fields, the 76-byte record the reference loader checks for.
fn emotes_text_schema() -> Schema {
    let mut s = Schema::new("EmotesText");
    s.add_field(SchemaField::new("ID", FieldType::UInt32));
    s.add_field(SchemaField::new("Name", FieldType::String));
    s.add_field(SchemaField::new("EmoteID", FieldType::UInt32));
    for i in 0..FORMS {
        s.add_field(SchemaField::new(format!("EmoteText{i}"), FieldType::UInt32));
    }
    s
}

/// `EmotesTextData.dbc`: a 1.12 `LocalizedString` block, the 40-byte record the reader checks
/// (`cmp eax,0x28`).
fn emotes_text_data_schema() -> Schema {
    let mut s = Schema::new("EmotesTextData");
    s.add_field(SchemaField::new("ID", FieldType::UInt32));
    for i in 0..LOCALES {
        s.add_field(SchemaField::new(format!("Text{i}"), FieldType::String));
    }
    s.add_field(SchemaField::new("Flags", FieldType::UInt32));
    s
}

/// Read and join both tables off the patch chain.
pub fn load_emote_text_catalog(chain: &mut Chain) -> Result<EmoteTextCatalog> {
    let bytes = chain
        .read_file(EMOTES_TEXT)
        .with_context(|| format!("reading {EMOTES_TEXT}"))?;
    let rs = parse(&bytes, emotes_text_schema(), "EmotesText.dbc")?;
    let mut forms = HashMap::with_capacity(rs.records().len());
    for r in rs.records() {
        let Some(id) = u32_at(r, 0) else { continue };
        let mut cols = [0u32; FORMS];
        for (i, slot) in cols.iter_mut().enumerate() {
            *slot = u32_at(r, 3 + i).unwrap_or(0);
        }
        forms.insert(id, cols);
    }

    let bytes = chain
        .read_file(EMOTES_TEXT_DATA)
        .with_context(|| format!("reading {EMOTES_TEXT_DATA}"))?;
    let rs = parse(&bytes, emotes_text_data_schema(), "EmotesTextData.dbc")?;
    let mut data = HashMap::with_capacity(rs.records().len());
    for r in rs.records() {
        let Some(id) = u32_at(r, 0) else { continue };
        let mut texts: [Option<String>; LOCALES] = Default::default();
        for (locale, slot) in texts.iter_mut().enumerate() {
            // `str_at` answers `None` for an empty string: the reference's blank-means-next-rung
            // test (`cmpb $0,(%edx)`).
            *slot = str_at(&rs, r, 1 + locale);
        }
        data.insert(id, texts);
    }

    Ok(EmoteTextCatalog { forms, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WAVE: a column or locale slip shows as the wrong sentence, never a silent empty.
    const WAVE: u32 = 101;

    fn line<'a>(performer: &'a str, target: &'a str, you: &'a str) -> EmoteLine<'a> {
        EmoteLine {
            performer,
            performer_is_you: false,
            performer_female: false,
            target,
            your_name: you,
        }
    }

    fn locale(chain: &mut Chain) -> usize {
        let bytes = chain
            .read_file(EMOTES_TEXT_DATA)
            .expect("EmotesTextData.dbc");
        let records = parse(&bytes, emotes_text_data_schema(), "EmotesTextData").unwrap();
        crate::dbc::preferred_locale(&records, 1)
    }

    #[test]
    fn fill_substitutes_left_to_right_and_keeps_a_surplus_verbatim() {
        assert_eq!(
            fill("%s waves at %s.", &["Bob", "Jane"]),
            "Bob waves at Jane."
        );
        assert_eq!(fill("%s waves at you.", &["Bob"]), "Bob waves at you.");
        assert_eq!(fill("You wave.", &["Bob"]), "You wave.");
        assert_eq!(fill("%s waves at %s.", &["Bob"]), "Bob waves at %s.");
        assert_eq!(fill("", &["Bob"]), "");
    }

    #[test]
    fn the_shipped_tables_compose_waves_five_forms() {
        let data = crate::wow_data_or_skip!();
        let mut chain = Chain::open(&data).expect("open patch chain");
        let cat = load_emote_text_catalog(&mut chain).expect("load");
        let locale = locale(&mut chain);
        assert_eq!(cat.len(), 169, "one form table per EmotesText row");

        // ctx 0: someone else waves at someone else.
        assert!(
            cat.compose(WAVE, &line("Bob", "Jane", "Me"), locale)
                .is_some_and(|text| text.contains("Bob") && text.contains("Jane"))
        );
        // ctx 1: someone else waves at you (the target name matches your own).
        assert!(
            cat.compose(WAVE, &line("Bob", "Me", "Me"), locale)
                .is_some_and(|text| text.contains("Bob") && !text.contains("Me"))
        );
        // ctx 4: someone else waves at nobody.
        assert!(
            cat.compose(WAVE, &line("Bob", "", "Me"), locale)
                .is_some_and(|text| text.contains("Bob"))
        );
        // ctx 2: you wave at someone.
        let mine = EmoteLine {
            performer_is_you: true,
            ..line("Me", "Jane", "Me")
        };
        assert!(
            cat.compose(WAVE, &mine, locale)
                .is_some_and(|text| text.contains("Jane"))
        );
        // ctx 6: you wave at nobody.
        let mine = EmoteLine {
            performer_is_you: true,
            ..line("Me", "", "Me")
        };
        assert!(cat.compose(WAVE, &mine, locale).is_some());
    }

    /// The branch order would render your own name for a self-target, but the send side clears a
    /// self-target (`DoEmote`, `0x5ef611`), so the server echoes an empty target.
    #[test]
    fn the_self_named_target_form_is_unreachable_from_the_wire() {
        let data = crate::wow_data_or_skip!();
        let mut chain = Chain::open(&data).expect("open patch chain");
        let cat = load_emote_text_catalog(&mut chain).expect("load");
        let locale = locale(&mut chain);
        let me = EmoteLine {
            performer_is_you: true,
            ..line("Me", "Me", "Me")
        };
        assert!(
            cat.compose(WAVE, &me, locale)
                .is_some_and(|text| text.contains("Me"))
        );
        // What the wire delivers for that action: an empty target.
        let me = EmoteLine {
            performer_is_you: true,
            ..line("Me", "", "Me")
        };
        assert!(
            cat.compose(WAVE, &me, locale)
                .is_some_and(|text| !text.contains("Me"))
        );
    }

    /// WAVE has no female column, so `8|ctx` misses and rung 2, `ctx` alone, answers.
    #[test]
    fn a_missing_female_column_falls_back_to_the_ungendered_one() {
        let data = crate::wow_data_or_skip!();
        let mut chain = Chain::open(&data).expect("open patch chain");
        let cat = load_emote_text_catalog(&mut chain).expect("load");
        let locale = locale(&mut chain);
        let her = EmoteLine {
            performer_female: true,
            ..line("Ann", "Jane", "Me")
        };
        assert!(
            cat.compose(WAVE, &her, locale)
                .is_some_and(|text| text.contains("Ann") && text.contains("Jane"))
        );
    }

    /// An unshipped locale is empty everywhere, so the ladder runs dry: no line (`0x49b4bd`).
    #[test]
    fn an_unshipped_locale_composes_nothing() {
        let data = crate::wow_data_or_skip!();
        let mut chain = Chain::open(&data).expect("open patch chain");
        let cat = load_emote_text_catalog(&mut chain).expect("load");
        assert_eq!(cat.compose(WAVE, &line("Bob", "Jane", "Me"), 5), None);
        assert_eq!(cat.compose(9999, &line("Bob", "Jane", "Me"), 4), None);
    }

    /// Three rows are silent as shipped: `SIT` (86) points columns 4 and 6 at `EmotesTextData`
    /// rows 446 and 447, blank in every locale, and `STAND` (141) and `TRAIN` (264) have all
    /// sixteen columns zero. Every other row composes a `%s`-free sentence in all five contexts.
    #[test]
    fn every_shipped_emote_composes_except_the_three_silent_rows() {
        let data = crate::wow_data_or_skip!();
        let mut chain = Chain::open(&data).expect("open patch chain");
        let cat = load_emote_text_catalog(&mut chain).expect("load");
        let locale = locale(&mut chain);
        let mut ids: Vec<u32> = cat.forms.keys().copied().collect();
        ids.sort_unstable();
        let mut silent = Vec::new();
        for id in ids {
            for (label, l) in [
                ("other→other", line("Bob", "Jane", "Me")),
                ("other→you", line("Bob", "Me", "Me")),
                ("other→none", line("Bob", "", "Me")),
                (
                    "you→other",
                    EmoteLine {
                        performer_is_you: true,
                        ..line("Me", "Jane", "Me")
                    },
                ),
                (
                    "you→none",
                    EmoteLine {
                        performer_is_you: true,
                        ..line("Me", "", "Me")
                    },
                ),
            ] {
                match cat.compose(id, &l, locale) {
                    None => silent.push(id),
                    Some(s) => {
                        assert!(!s.is_empty(), "EmotesText {id} composed empty for {label}");
                        assert!(
                            !s.contains("%s"),
                            "EmotesText {id} left a %s unfilled for {label}: {s:?}"
                        );
                    }
                }
            }
        }
        silent.dedup();
        assert_eq!(silent, [86, 141, 264], "the silent set is SIT/STAND/TRAIN");
    }
}
