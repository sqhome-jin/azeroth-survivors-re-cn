//! Shared helpers for the typed DBC loaders. A DBC carries no column types, so each loader supplies
//! a [`Schema`] whose field count must match the file header, and reads fields by index.

use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{Context, Result, anyhow};
use benilla_dbc::{DbcParser, FieldType, Record, RecordSet, Schema, SchemaField, StringRef, Value};

use crate::Chain;

/// Parse a DBC with `schema`; `what` names the file in errors.
pub(crate) fn parse(bytes: &[u8], schema: Schema, what: &str) -> Result<RecordSet> {
    let parser = DbcParser::parse(&mut Cursor::new(bytes))
        .map_err(|e| anyhow!("parsing {what} header: {e}"))?;
    let parser = parser
        .with_schema(schema)
        .map_err(|e| anyhow!("applying {what} schema (field-count mismatch?): {e}"))?;
    parser
        .parse_records()
        .map_err(|e| anyhow!("parsing {what} records: {e}"))
}

/// An unsigned field, from either int variant: the schema tag only picks the decoding.
pub(crate) fn u32_at(r: &Record, i: usize) -> Option<u32> {
    match r.get_value(i)? {
        Value::UInt32(v) => Some(*v),
        Value::Int32(v) => Some(*v as u32),
        _ => None,
    }
}

pub(crate) fn f32_at(r: &Record, i: usize) -> Option<f32> {
    match r.get_value(i)? {
        Value::Float32(v) => Some(*v),
        _ => None,
    }
}

/// A signed field; either int variant reads, bit for bit.
pub(crate) fn i32_at(r: &Record, i: usize) -> Option<i32> {
    match r.get_value(i)? {
        Value::Int32(v) => Some(*v),
        Value::UInt32(v) => Some(*v as i32),
        _ => None,
    }
}

/// A string field from the string block; an empty string reads as `None`.
pub(crate) fn str_at(rs: &RecordSet, r: &Record, i: usize) -> Option<String> {
    match r.get_value(i)? {
        Value::StringRef(StringRef(off)) => {
            let s = rs.get_string(StringRef(*off)).ok()?;
            (!s.is_empty()).then(|| s.to_string())
        }
        _ => None,
    }
}

const LOCALE_COUNT: usize = 8;

pub(crate) fn localized_str_at(
    rs: &RecordSet,
    r: &Record,
    base: usize,
    locale: usize,
) -> Option<String> {
    if locale >= LOCALE_COUNT {
        return None;
    }
    str_at(rs, r, base + locale).or_else(|| (locale != 0).then(|| str_at(rs, r, base)).flatten())
}

fn preferred_locale_from_counts(counts: [usize; LOCALE_COUNT]) -> usize {
    (1..LOCALE_COUNT).fold(0, |best, locale| {
        if counts[locale] > counts[best] {
            locale
        } else {
            best
        }
    })
}

pub(crate) fn preferred_locale(rs: &RecordSet, name_field: usize) -> usize {
    let mut counts = [0; LOCALE_COUNT];
    for record in rs.records() {
        for (locale, count) in counts.iter_mut().enumerate() {
            if str_at(rs, record, name_field + locale).is_some() {
                *count += 1;
            }
        }
    }
    preferred_locale_from_counts(counts)
}

#[cfg(test)]
mod locale_tests {
    use super::*;

    #[test]
    fn preferred_locale_defaults_to_enus_on_ties() {
        assert_eq!(preferred_locale_from_counts([12, 0, 0, 0, 12, 0, 0, 0]), 0);
        assert_eq!(preferred_locale_from_counts([3, 0, 0, 0, 8, 0, 0, 0]), 4);
    }
}
/// `SpellIcon.dbc`: id → extensionless texture path (`Interface\Icons\…`).
pub(crate) fn load_spell_icon_map(chain: &mut Chain) -> Result<HashMap<u32, String>> {
    let bytes = chain
        .read_file("DBFilesClient\\SpellIcon.dbc")
        .context("reading SpellIcon.dbc")?;
    let mut schema = Schema::new("SpellIcon");
    schema.add_field(SchemaField::new("ID", FieldType::UInt32));
    schema.add_field(SchemaField::new("TextureFilename", FieldType::String));
    let set = parse(&bytes, schema, "SpellIcon.dbc")?;
    let mut icons = HashMap::new();
    for r in set.records() {
        if let (Some(id), Some(path)) = (u32_at(r, 0), str_at(&set, r, 1)) {
            icons.insert(id, path);
        }
    }
    Ok(icons)
}

/// An id → name schema of `cols` four-byte columns; the name is an eight-slot localized string.
pub(crate) fn id_name_schema(what: &str, name_col: usize, cols: usize) -> Schema {
    let mut s = Schema::new(what);
    for i in 0..cols {
        if (name_col..name_col + LOCALE_COUNT).contains(&i) {
            s.add_field(SchemaField::new(format!("Name{i}"), FieldType::String));
        } else {
            s.add_field(SchemaField::new(format!("c{i}"), FieldType::UInt32));
        }
    }
    s
}

/// Read an id → localized name DBC into a map, skipping rows with no name in any locale.
pub(crate) fn load_id_name_table(
    chain: &mut Chain,
    file: &str,
    name_col: usize,
    cols: usize,
    what: &str,
) -> Result<HashMap<u32, String>> {
    let bytes = chain
        .read_file(file)
        .with_context(|| format!("reading {file}"))?;
    let rs = parse(&bytes, id_name_schema(what, name_col, cols), what)?;
    let locale = preferred_locale(&rs, name_col);
    let mut out = HashMap::with_capacity(rs.records().len());
    for r in rs.records() {
        if let (Some(id), Some(name)) = (u32_at(r, 0), localized_str_at(&rs, r, name_col, locale)) {
            out.insert(id, name);
        }
    }
    Ok(out)
}
