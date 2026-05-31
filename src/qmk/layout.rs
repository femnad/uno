use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use indexmap::IndexMap;

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
    custom_keycodes: Vec<String>,
    custom_keys: IndexMap<String, String>,
    header_definitions: IndexMap<String, String>,
    layers: Vec<String>,
    layouts: IndexMap<String, Vec<String>>,
}

pub fn write_layout(keyboard: String, config: String) {
    let mut file = File::open(config.clone()).expect(format!("cannot open {}", config).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect(format!("cannot read {}", config).as_str());

    let config: Config =
        yaml_serde::from_str(&contents).expect(format!("could not parse {}", contents).as_str());

    let keyboard: Box<dyn Keyboard> = match keyboard.as_str() {
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

    for (key, value) in config.custom_keys {
        let key = key.to_ascii_uppercase();
        let value = value.to_ascii_uppercase();
        out.write(format!("#define {} {}\n", key, value).as_bytes())
            .unwrap();
    }
    out.write("\n".as_bytes()).unwrap();

    let prefix = keyboard.custom_keycode_prefix().unwrap_or("custom".to_string());
    out.write(format!("enum {}_keycodes {{\n", prefix).as_bytes()).unwrap();
    for (idx, code) in config.custom_keycodes.iter().enumerate() {
        let code = code.to_ascii_uppercase();
        let suffix = if idx == 0 {
            " = SAFE_RANGE,\n"
        } else {
            ",\n"
        };
        let line = format!("  {}{}", code, suffix);
        out.write(line.as_bytes()).unwrap();
    }
    out.write("};\n\n".as_bytes()).unwrap();

    if let Some(layer_name) = keyboard.layer_name() {
        out.write(format!("enum {}_layers {{\n", layer_name).as_bytes())
            .unwrap();
        for layer in config.layers {
            out.write(format!("  {},\n", layer).as_bytes()).unwrap();
        }
        out.write("}}\n\n".as_bytes()).unwrap();
    }

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
}

trait Keyboard {
    fn chordal_hold_layout(&self) -> String;
    fn custom_keycode_prefix(&self) -> Option<String> {
        None
    }
    fn layer_name(&self) -> Option<String> {
        None
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
    fn layer_name(&self) -> Option<String> {
        Option::from("preonic".to_string())
    }
}

struct Moonlander;

impl Keyboard for Moonlander {
    fn chordal_hold_layout(&self) -> String {
        "".to_string()
    }
}
