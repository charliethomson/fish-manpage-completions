#[macro_export]
macro_rules! regex {
    ($pattern: expr) => {{
        lazy_static::lazy_static! {
            static ref REGEX: regex::Regex = regex::Regex::new($pattern).unwrap();
        }
        &REGEX
    }};
}

use crate::char_len;
use encoding_rs_io::DecodeReaderBytes;
use std::io::Read;
use std::{collections::HashMap, iter::FromIterator};

pub struct TranslationTable {
    table: HashMap<char, char>,
}
impl TranslationTable {
    pub fn new(f: &str, t: &str) -> Result<Self, &'static str> {
        if char_len(f) != char_len(t) {
            Err("Arguments passed to `TranslationTable::new` must have equal character length")
        } else {
            Ok(TranslationTable {
                table: HashMap::from_iter(f.chars().zip(t.chars())),
            })
        }
    }

    pub fn translate(&self, s: &str) -> String {
        s.chars()
            .map(|c| *self.table.get(&c).unwrap_or(&c))
            .collect()
    }
}

#[test]
fn test_translationtable_new() {
    assert!(TranslationTable::new("aaa", "Incorrect Length").is_err());
    assert!(TranslationTable::new("aaa", "bbb").is_ok());
    let tr = TranslationTable::new("ab!", "cd.").unwrap();

    let mut expected = HashMap::new();
    expected.insert('a', 'c');
    expected.insert('b', 'd');
    expected.insert('!', '.');

    assert_eq!(tr.table, expected);

    // Unicode tests
    let tr = TranslationTable::new("🗻🚀🚁", "mrh").unwrap();
    let mut expected = HashMap::new();
    expected.insert('🗻', 'm');
    expected.insert('🚀', 'r');
    expected.insert('🚁', 'h');

    assert_eq!(tr.table, expected);
}

#[test]
fn test_translationtable_translate() {
    let tr = TranslationTable::new("ab!", "cd.").unwrap();

    assert_eq!(tr.translate("aabb!!"), "ccdd..".to_owned());

    assert_eq!(tr.translate("Hello World!"), "Hello World.".to_owned(),);

    assert_eq!(tr.translate("applebees!"), "cppledees.".to_owned(),);

    // Unicode tests
    let tr = TranslationTable::new("🗻🚀🚁", "mrh").unwrap();

    assert_eq!(
        tr.translate("This 🗻 is a mountain!"),
        "This m is a mountain!".to_owned(),
    );

    assert_eq!(
        tr.translate("This 🚀 is a rocket! (rocket.rs :))"),
        "This r is a rocket! (rocket.rs :))".to_owned(),
    );

    assert_eq!(
        tr.translate("This 🚁 is a helicopter!"),
        "This h is a helicopter!".to_owned(),
    )
}

pub fn decode_bytes(bytes: &[u8]) -> std::io::Result<String> {
    let mut decoder = DecodeReaderBytes::new(bytes);
    let mut dest = String::new();
    decoder.read_to_string(&mut dest)?;
    Ok(dest)
}

#[test]
fn test_decode_bytes() {
    // 안녕하세요 세계
    let mut bytes = [
        0xFF, 0xB7, 0xFF, 0xC2, 0xFF, 0xA4, 0xFF, 0xA4, 0xFF, 0xCA, 0xFF, 0xB7, 0xFF, 0xBE, 0xFF,
        0xC2, 0xFF, 0xB5, 0xFF, 0xC7, 0xFF, 0xB7, 0xFF, 0xD2, 0xFF, 0xB5, 0xFF, 0xC7, 0xFF, 0xFF,
        0xA1, 0xFF, 0xCB,
    ];

    let result = decode_bytes(&bytes).unwrap();
    assert_eq!(result, "안녕하세요 세계");
}
