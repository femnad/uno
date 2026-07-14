use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

const PREONIC_CHORDAL_LAYOUT: &str = r#"#ifdef CHORDAL_HOLD
const char chordal_hold_layout[MATRIX_ROWS][MATRIX_COLS] PROGMEM = LAYOUT_preonic_grid(
    'L', 'L', 'L', 'L', 'L', 'L',  'R', 'R', 'R', 'R', 'R', 'R',
    'L', 'L', 'L', 'L', 'L', 'L',  'R', 'R', 'R', 'R', 'R', 'R',
    'L', 'L', 'L', 'L', 'L', 'L',  'R', 'R', 'R', 'R', 'R', 'R',
    'L', 'L', 'L', 'L', 'L', 'L',  'R', 'R', 'R', 'R', 'R', 'R',
    '*', '*', '*', '*', '*', '*',  '*', '*', '*', '*', '*', '*'
);
#endif
"#;

#[derive(Debug, Deserialize)]
struct Config {
    custom_keys: Vec<String>,
    header_definitions: IndexMap<String, String>,
    layouts: IndexMap<String, Vec<String>>,
}

fn get_key(key: String) -> String {
    format!("KC_{}", key.to_ascii_uppercase())
}

pub fn write_layout(keyboard: String, config: String) {
    let mut file = File::open(config.clone()).expect(format!("cannot open {}", config).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect(format!("cannot read {}", config).as_str());

    let config: Config =
        yaml_serde::from_str(&contents).expect(format!("could not parse {}", contents).as_str());

    let keyboard: Box<dyn Keyboard> = match keyboard.as_str() {
        "ergodox-ez" => Box::new(ErgodoxEz),
        "moonlander" => Box::new(Moonlander),
        "preonic" => Box::new(Preonic),
        _ => {
            panic!("unknown keyboard {}", keyboard);
        }
    };
    let chordal_hold = keyboard.chordal_hold_layout();

    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("keymap-new.c")
        .unwrap();

    let prefix = keyboard
        .custom_keycode_prefix()
        .unwrap_or("custom".to_string());
    out.write(format!("enum {}_keycodes {{\n", prefix).as_bytes())
        .unwrap();
    for (idx, code) in config.custom_keys.iter().enumerate() {
        let code = code.to_ascii_uppercase();
        let suffix = if idx == 0 { " = SAFE_RANGE,\n" } else { ",\n" };
        let line = format!("  {}{}", code, suffix);
        out.write(line.as_bytes()).unwrap();
    }
    out.write("};\n\n".as_bytes()).unwrap();

    out.write(chordal_hold.as_bytes()).unwrap();

    let mut header = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("config-new.h")
        .unwrap();

    for (key, value) in config.header_definitions {
        let key = key.to_ascii_uppercase();
        let value = value.to_ascii_uppercase();
        let suffix = if value.is_empty() {
            "".to_string()
        } else {
            format!(" {}", value)
        };
        let line = format!("#define {}{}\n", key, suffix);
        header.write(line.as_bytes()).unwrap();
    }

    let whitespace = Regex::new(r"\s+").unwrap();
    for (layer, rows) in config.layouts {
        out.write(format!("[{}] LAYOUT(\n", layer).as_bytes())
            .unwrap();
        for row in rows {
            let cols = whitespace.split(row.as_str());
            for col in cols {
                let col = get_key(col.to_string());
                out.write(col.as_bytes()).unwrap();
            }
        }
        out.write(")\n".as_bytes()).unwrap();
    }
}

trait Keyboard {
    fn chordal_hold_layout(&self) -> String;
    fn custom_keycode_prefix(&self) -> Option<String> {
        None
    }
}

struct ErgodoxEz;

impl Keyboard for ErgodoxEz {
    fn chordal_hold_layout(&self) -> String {
        "".to_string()
    }
}

struct Moonlander;

impl Keyboard for Moonlander {
    fn chordal_hold_layout(&self) -> String {
        "".to_string()
    }
}

struct Preonic;

impl Keyboard for Preonic {
    fn chordal_hold_layout(&self) -> String {
        PREONIC_CHORDAL_LAYOUT.to_string()
    }
    fn custom_keycode_prefix(&self) -> Option<String> {
        Option::from("preonic".to_string())
    }
}
