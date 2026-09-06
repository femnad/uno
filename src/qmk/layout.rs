use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::exit;

const COL_LENGTH: usize = 8;
const HEADER: &str = r#"#include QMK_KEYBOARD_H
#include "version.h"
"#;
const LAYOUT_START: &str = "const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {";
const LAYOUT_END: &str = "};";
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
    custom_definitions: IndexMap<String, String>,
    custom_keys: Vec<String>,
    header_definitions: IndexMap<String, String>,
    layouts: IndexMap<String, Vec<String>>,
}

enum Column {
    Empty(usize),
    Occupied(usize),
}

fn get_key(key: String) -> String {
    format!("KC_{}", key.to_ascii_uppercase())
}

fn write_to_file(out: &mut File, s: &str) {
    out.write(s.as_bytes()).unwrap();
}

fn write_new_lined(out: &mut File, s: &str) {
    write_to_file(out, format!("{}\n", s).as_str());
}

fn write_commented(out: &mut File, s: &str) {
    let lines = s.trim().lines().collect::<Vec<_>>();
    let num_lines = lines.len();
    if num_lines == 0 {
        return;
    }

    if num_lines == 1 {
        write_new_lined(out, format!("// {}", &lines[0]).as_str());
    }

    for (idx, line) in lines.iter().enumerate() {
        if idx == 0 {
            write_new_lined(out, format!("/* {}", line).as_str());
        } else if idx == num_lines - 1 {
            write_new_lined(out, format!(" * {}", line).as_str());
            write_new_lined(out, "*/");
        } else {
            write_new_lined(out, format!(" * {}", line).as_str());
        }
    }
}

pub fn write_layout(keyboard: String, config: String) {
    let file = File::open(config.clone());
    if file.is_err() {
        println!("Failed to open config file: {}", config.clone());
        exit(1);
    }

    let mut contents = String::new();
    file.unwrap()
        .read_to_string(&mut contents)
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

    write_new_lined(&mut out, HEADER);

    for (idx, layer) in config.layouts.keys().enumerate() {
        write_to_file(
            &mut out,
            &format!("#define {} {}\n", layer.to_ascii_uppercase(), idx),
        );
    }
    write_to_file(&mut out, "\n");

    for (key, value) in config.custom_definitions {
        write_to_file(
            &mut out,
            &format!(
                "#define {} {}\n",
                key.to_ascii_uppercase(),
                value.to_ascii_uppercase()
            ),
        );
    }
    write_to_file(&mut out, "\n");

    write_new_lined(&mut out, LAYOUT_START);

    for (layer, rows) in &config.layouts {
        let mut layer_str = String::new();
        layer_str.push_str(format!("Layer: {}\n", layer).as_str());

        let layer_map = keyboard.layer_map(rows);
        layer_str.push_str(format!("{}\n", layer_map).as_str());

        write_commented(&mut out, format!("{}\n", layer_str).as_str());
    }

    write_new_lined(&mut out, LAYOUT_END);
    write_to_file(&mut out, "\n");

    write_to_file(&mut out, PREONIC_CHORDAL_LAYOUT);

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

    fn row_map(&self) -> HashMap<usize, Vec<Column>>;

    fn layer_map(&self, layer: &Vec<String>) -> String {
        let row_map = self.row_map();

        let mut out = String::new();
        let col_split = Regex::new(r"\s+").unwrap();
        for (idx, row) in layer.iter().enumerate() {
            let mut cols = col_split.split(row).collect::<Vec<_>>();

            let col_map = row_map.get(&idx);
            if col_map.is_none() {
                let col_out = cols.join("|");
                out.push_str(format!("{}\n", &col_out).as_str());
                continue;
            };

            for col in col_map.expect(format!("error finding column map for row {}", idx).as_str())
            {
                match col {
                    Column::Empty(size) => {
                        for _ in 0..*size {
                            out.push_str(" ".repeat(COL_LENGTH).as_str());
                        }
                    }
                    Column::Occupied(size) => {
                        for _ in 0..*size {
                            let col_out = cols.pop().expect("");
                            out.push_str(format!("{}|", col_out).as_str());
                        }
                    }
                }
            }
            out.push_str("\n");
        }
        out
    }
}

struct ErgodoxEz;

impl Keyboard for ErgodoxEz {
    fn chordal_hold_layout(&self) -> String {
        "".to_string()
    }

    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        todo!()
    }
}

struct Moonlander;

impl Keyboard for Moonlander {
    fn chordal_hold_layout(&self) -> String {
        "".to_string()
    }

    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        HashMap::from([
            (
                3,
                vec![Column::Occupied(6), Column::Empty(2), Column::Occupied(6)],
            ),
            (
                4,
                vec![Column::Occupied(6), Column::Empty(2), Column::Occupied(6)],
            ),
            (
                5,
                vec![
                    Column::Empty(3),
                    Column::Occupied(3),
                    Column::Empty(3),
                    Column::Occupied(3),
                    Column::Empty(3),
                ],
            ),
        ])
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
    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        todo!()
    }
}
