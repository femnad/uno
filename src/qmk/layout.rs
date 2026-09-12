use indexmap::IndexMap;
use regex::Regex;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::process::exit;
use std::sync::LazyLock;

use minijinja::{Environment, context};

const CHORDAL_LEFT: &str = "'L'";
const CHORDAL_RIGHT: &str = "'R'";
const CHORDAL_NEUTRAL: &str = "'*'";
const CHORDAL_CHAR_LENGTH: usize = 3;
const COL_LENGTH: usize = 9;
const HEADER: &str = r#"#include QMK_KEYBOARD_H
#include "version.h"
"#;
const KC_TRANSPARENT: &str = "_______";
const LAYOUT_START: &str = "const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {";
const LAYOUT_END: &str = "};";
const MODDED_KEY_PATTERN: &str = r"[r|l](ctl|alt|gui|sft)\(.*\)";
static MODDED_KEY_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(MODDED_KEY_PATTERN).unwrap());
const NON_KC_KEY_PATTERN: &str = "^(cw|ms|qk|rgb|rm)_.*";
static NON_KC_KEY_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(NON_KC_KEY_PATTERN).unwrap());
const ONE_SHOT_MOD_PATTERN: &str = r"osm\((.*)\)";
static ONE_SHOT_MOD_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(ONE_SHOT_MOD_PATTERN).unwrap());
const LAYER_PATTERN: &str = r"(osl|tg)\((.*)\)";
static LAYER_REGEX: LazyLock<Regex> = LazyLock::new(||
    Regex::new(LAYER_PATTERN).unwrap());
const TRANSPARENT_KEY: &str = "_";

const TAPPING_TERM_FUNCTION_DEF: &str = r#"uint16_t get_tapping_term(uint16_t keycode, keyrecord_t *record) {
  switch (keycode) {
{%- for mod_tap in mod_taps %}
    case {{ mod_tap }}:
{%- endfor %}
      return {{ tap_term }};
    default:
      return TAPPING_TERM;
  }
}
"#;

const HELPER_FNS: &str = r#"{% if is_rgb %}void maybe_reset_rgb_matrix(uint8_t mods) {
  if (mods == 0) {
    rgb_matrix_set_color_all(0, 0, 0);
  }
}

void oneshot_mods_changed_user(uint8_t mods) {
  maybe_reset_rgb_matrix(mods);
}

void oneshot_locked_mods_changed_user(uint8_t mods) {
  maybe_reset_rgb_matrix(mods);
}

void reset_color(int index) {
  rgb_matrix_set_color(index, 0, 0, 0);
}
{%- endif %}

void clear(void) {
  caps_word_off();
  clear_oneshot_mods();
  clear_oneshot_locked_mods();
  clear_keyboard();
  reset_oneshot_layer();
  layer_clear();
  layer_on(BASE);
{%- if is_rgb %}
  rgb_matrix_set_color_all(0, 0, 0);
{%- endif %}
}

bool process_record_user(uint16_t keycode, keyrecord_t *record) {
  if (record->event.pressed) {
    switch (keycode) {
      case CLEAR:
        clear();
        return false;
    }
  }
  return true;
}
{%- if is_rgb %}

bool rgb_matrix_indicators_user(void) {
  uint8_t mods = get_oneshot_mods();
  uint8_t locked_mods = get_oneshot_locked_mods();

#if defined(LEFT_SHIFT_INDEX) && defined(RIGHT_SHIFT_INDEX)
  if (mods & MOD_MASK_SHIFT) {
    rgb_matrix_set_color(LEFT_SHIFT_INDEX, 128, 0, 128);
    rgb_matrix_set_color(RIGHT_SHIFT_INDEX, 128, 0, 128);
  } else if (locked_mods & MOD_MASK_SHIFT) {
    rgb_matrix_set_color(LEFT_SHIFT_INDEX, 255, 0, 255);
    rgb_matrix_set_color(RIGHT_SHIFT_INDEX, 255, 0, 255);
  } else {
    reset_color(LEFT_SHIFT_INDEX);
    reset_color(RIGHT_SHIFT_INDEX);
  }
#endif

#if defined(INDX_OSL_LEFT_INDEX) && defined(INDX_OSL_RIGHT_INDEX)
#if defined(SYMB_OSL_LEFT_INDEX) && defined(SYMB_OSL_RIGHT_INDEX)
#if defined(MOVE_OSL_LEFT_INDEX) && defined(MOVE_OSL_RIGHT_INDEX)
  uint8_t osl_state = get_oneshot_layer_state();
  uint8_t osl_left_index = 0, osl_right_index = 0;
  uint8_t osl_r = 0, osl_g = 0, osl_b = 0;
  switch (get_oneshot_layer()) {
    case INDX:
      osl_left_index = INDX_OSL_LEFT_INDEX;
      osl_right_index = INDX_OSL_RIGHT_INDEX;
      osl_r = 255;
      break;
    case SYMB:
      osl_left_index = SYMB_OSL_LEFT_INDEX;
      osl_right_index = SYMB_OSL_RIGHT_INDEX;
      osl_b = 255;
      break;
    case MOVE:
      osl_left_index = MOVE_OSL_LEFT_INDEX;
      osl_right_index = MOVE_OSL_RIGHT_INDEX;
      osl_g = 255;
      break;
  }

  if (osl_state & ONESHOT_TOGGLED) {
    rgb_matrix_set_color(osl_left_index, osl_r, osl_g, osl_b);
    rgb_matrix_set_color(osl_right_index, osl_r, osl_g, osl_b);
  } else if (osl_state) {
    rgb_matrix_set_color(osl_left_index, osl_r / 2, osl_g / 2, osl_b / 2);
    rgb_matrix_set_color(osl_right_index, osl_r / 2, osl_g / 2, osl_b / 2);
  } else {
    reset_color(INDX_OSL_LEFT_INDEX);
    reset_color(INDX_OSL_RIGHT_INDEX);
    reset_color(SYMB_OSL_LEFT_INDEX);
    reset_color(SYMB_OSL_RIGHT_INDEX);
    reset_color(MOVE_OSL_LEFT_INDEX);
    reset_color(MOVE_OSL_RIGHT_INDEX);
  }
#endif
#endif
#endif

#if defined(CAPS_WORD_LEFT_INDEX) && defined(CAPS_WORD_RIGHT_INDEX)
  if (is_caps_word_on()) {
    rgb_matrix_set_color(CAPS_WORD_LEFT_INDEX, 255, 255, 0);
    rgb_matrix_set_color(CAPS_WORD_RIGHT_INDEX, 255, 255, 0);
  } else {
    reset_color(CAPS_WORD_LEFT_INDEX);
    reset_color(CAPS_WORD_RIGHT_INDEX);
  }
#endif

  return true;
}
{%- endif %}
"#;

#[derive(Debug, Deserialize)]
struct Config {
    custom_definitions: IndexMap<String, String>,
    custom_keys: Vec<String>,
    header_definitions: IndexMap<String, String>,
    layouts: IndexMap<String, Vec<String>>,
    mode_tap_term: usize,
    rules: IndexMap<String, String>,
}

enum Column {
    Empty(usize),
    Occupied(usize),
}

fn write_to_file(out: &mut File, s: &str) {
    out.write(s.as_bytes()).unwrap();
}

fn write_indented(out: &mut File, s: &str, indent: usize) {
    let mut indented = String::from(" ".repeat(indent));
    indented.push_str(s);
    write_new_lined(out, &indented);
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

fn parse_modded_key(value: &str) -> String {
    let mut outer = String::new();
    for (idx, c) in value.chars().enumerate() {
        if c == '(' {
            let sub = parse_modded_key(&value[idx + 1..]);
            return format!("{}({})", outer, sub);
        } else if c == ')' {
            return format!("kc_{}", outer.as_str());
        } else {
            outer.push(c);
        }
    }
    outer
}

fn get_qmk_key(value: &str, config: &Config) -> String {
    if config.custom_definitions.contains_key(value) {
        return value.to_string();
    }

    if config.custom_keys.contains(&value.to_string()) {
        return value.to_string();
    }

    if NON_KC_KEY_REGEX.is_match(value) {
        return value.to_string();
    }

    if let Some(caps) = ONE_SHOT_MOD_REGEX.captures(value) {
        let modded = &caps[1];
        return format!("osm(mod_{})", modded);
    }

    if LAYER_REGEX.is_match(value) {
        return value.to_string();
    }

    if MODDED_KEY_REGEX.is_match(value) {
        return parse_modded_key(value);
    }

    if value == TRANSPARENT_KEY {
        return KC_TRANSPARENT.to_string();
    }

    if value.starts_with("kc_") {
        return value.to_string();
    }

    format!("kc_{}", value)
}

fn get_key(value: &str, config: &Config) -> String {
    get_qmk_key(value, config).to_ascii_uppercase()
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
        "nyquist-lm" => Box::new(NyquistLM),
        "preonic" => Box::new(Preonic),
        _ => {
            println!("unknown keyboard {}", keyboard);
            exit(1);
        }
    };

    let mut out = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("keymap.c")
        .unwrap();

    write_new_lined(&mut out, HEADER);

    for (idx, layer) in config.layouts.keys().enumerate() {
        write_to_file(
            &mut out,
            &format!("#define {} {}\n", layer.to_ascii_uppercase(), idx),
        );
    }
    write_to_file(&mut out, "\n");

    let mod_layer_tap = Regex::new(r"^(mt\(|lt\().*").unwrap();
    let mut mod_taps = vec![];
    for (key, value) in &config.custom_definitions {
        if mod_layer_tap.is_match(&value) {
            mod_taps.push(key.clone().to_ascii_uppercase());
        }

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

    write_new_lined(&mut out, LAYOUT_START);

    let by_whitespace = Regex::new(r"\s+").unwrap();
    for (layer, rows) in &config.layouts {
        write_to_file(&mut out, "\n");
        let layer_upper = layer.to_ascii_uppercase();
        let mut layer_str = String::new();
        layer_str.push_str(format!("Layer: {}\n", layer_upper).as_str());

        let layer_map = keyboard.layer_map(rows);
        layer_str.push_str(format!("{}\n", layer_map).as_str());

        write_commented(&mut out, format!("{}\n", layer_str).as_str());

        write_new_lined(
            &mut out,
            format!("[{}] = LAYOUT_{}(", layer_upper, keyboard.layout_suffix()).as_str(),
        );

        let num_rows = rows.len();
        for (row_idx, row) in rows.iter().enumerate() {
            let cols = by_whitespace
                .split(&row)
                .map(|c| get_key(c, &config))
                .collect::<Vec<_>>();
            let col_out = cols.join(", ");
            let last_char = if row_idx < num_rows - 1 {
                ","
            } else {
                ""
            };
            write_indented(&mut out, format!("{}{}", &col_out, last_char).as_str(), 8)
        }
        write_new_lined(&mut out, "),");
    }

    write_new_lined(&mut out, LAYOUT_END);
    write_to_file(&mut out, "\n");

    write_new_lined(&mut out, "#ifdef CHORDAL_HOLD");
    write_new_lined(
        &mut out,
        format!(
            "const char chordal_hold_layout[MATRIX_ROWS][MATRIX_COLS] PROGMEM = LAYOUT_{}(",
            keyboard.layout_suffix()
        )
        .as_str(),
    );
    write_to_file(&mut out, keyboard.chordal_hold_layout().as_str());
    write_new_lined(&mut out, ");");

    let mut env = Environment::new();
    env.add_template("tap_term", TAPPING_TERM_FUNCTION_DEF).unwrap();
    let template = env.get_template("tap_term").unwrap();
    let fn_def = template.render(context! { mod_taps => mod_taps, tap_term => config.mode_tap_term }).unwrap();

    write_to_file(&mut out, "\n");
    write_new_lined(&mut out, fn_def.as_str());
    write_to_file(&mut out, "#endif\n");

    write_to_file(&mut out, format!("\n{}\n", keyboard.get_helper_fns()).as_str());

    let mut header = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("config.h")
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

    let mut rules = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rules.mk")
        .unwrap();
    for (key, value) in config.rules {
        let key = key.to_ascii_uppercase();
        let line = format!("{} = {}\n", key, value);
        rules.write(line.as_bytes()).unwrap();
    }
}

fn format_value(value: &str) -> String {
    let value = value.to_ascii_uppercase();
    let value = if value == TRANSPARENT_KEY {
        ""
    } else {
        value.as_str()
    };
    format!("{:^COL_LENGTH$.COL_LENGTH$}", value)
}

trait Keyboard {
    fn custom_keycode_prefix(&self) -> Option<String> {
        None
    }

    fn layout_suffix(&self) -> String;

    fn max_columns(&self) -> usize;

    fn row_map(&self) -> HashMap<usize, Vec<Column>>;

    fn rows(&self) -> usize;

    fn thumb_rows(&self) -> HashSet<usize>;

    fn chordal_hold_layout(&self) -> String {
        let row_map = self.row_map();
        let mut out = String::new();
        let length = self.max_columns();
        let half = length / 2;
        let thumb_rows = self.thumb_rows();

        let num_rows = self.rows();
        for (row_idx, row) in (0..num_rows).enumerate() {
            if !row_map.contains_key(&row) {
                let chordal = self.get_full_row_chordal_layout(row_idx);
                let maybe_comma = if row_idx < self.rows() - 1 {
                    ","
                } else {
                    ""
                };
                out.push_str(format!("    {}{}\n", chordal, maybe_comma).as_str());
                continue;
            }

            let row_occupancy = row_map.get(&row).unwrap();
            let mut col_idx = 0;
            for (occ_idx, column) in row_occupancy.iter().enumerate() {
                let col_series = row_occupancy.len();

                if col_idx == 0 {
                    out.push_str(" ".repeat(4).as_str());
                }

                match column {
                    Column::Empty(cols) => {
                        if row_idx == num_rows - 1 && occ_idx == col_series - 1 {
                            continue;
                        }

                        let empty_cols = vec![" ".repeat(CHORDAL_CHAR_LENGTH); *cols];
                        out.push_str(empty_cols.join("  ").as_str());
                        out.push_str(" ");

                        out.push_str(" ");
                        col_idx += cols;
                    }
                    Column::Occupied(cols) => {
                        let mut occupied_cols = vec![];
                        for i in col_idx..(col_idx + cols) {
                            let out_char = if thumb_rows.contains(&row) {
                                CHORDAL_NEUTRAL
                            } else if i < half {
                                CHORDAL_LEFT
                            } else {
                                CHORDAL_RIGHT
                            };
                            occupied_cols.push(out_char)
                        }
                        out.push_str(occupied_cols.join(", ").as_str());

                        // No comma before closing parentheses.
                        let next_col = row_occupancy.get(occ_idx + 1).or(None);
                        if row_idx < num_rows - 1
                            || col_idx < col_series - 1
                            || matches!(next_col, Some(Column::Occupied(_)))
                        {
                            out.push_str(",");
                        }

                        if col_idx < col_series - 1 {
                            out.push_str(" ");
                        }
                        col_idx += cols;
                    }
                }
            }
            out.push_str("\n");
        }
        out
    }

    fn get_full_row_chordal_layout(&self, row_idx: usize) -> String {
        if self.thumb_rows().contains(&row_idx) {
            return vec![CHORDAL_NEUTRAL; self.max_columns()].join(", ");
        }

        let half = self.max_columns() / 2;
        let left = vec![CHORDAL_LEFT; half];
        let right = vec![CHORDAL_RIGHT; half];
        let both = [left, right].concat();
        both.join(", ")
    }

    fn get_helper_fns(&self) -> String {
        let mut env = Environment::new();
        env.add_template("helper_fns", HELPER_FNS).unwrap();
        let template = env.get_template("helper_fns").unwrap();
        let helper_fns = template.render(context! { is_rgb => self.is_rgb() }).unwrap();
        helper_fns.to_string()
    }

    fn is_rgb(&self) -> bool {
        true
    }

    fn layer_map(&self, layer: &Vec<String>) -> String {
        let row_map = self.row_map();
        let col_boundry = "-".repeat(COL_LENGTH);

        let full_boundary = vec![col_boundry; self.max_columns()];
        let full_boundary = format!("+{}+\n", full_boundary.join("+"));

        let mut out = String::from(&full_boundary);
        let col_split = Regex::new(r"\s+").unwrap();
        for (row_idx, row) in layer.iter().enumerate() {
            let mut cols = col_split.split(row).collect::<Vec<_>>();

            let col_map = row_map.get(&row_idx);
            if col_map.is_none() {
                let cols = cols.iter().map(|col| format_value(col)).collect::<Vec<_>>();
                let col_out = cols.join("|");
                out.push_str(format!("|{}|\n", &col_out).as_str());
                out.push_str(full_boundary.as_str());
                continue;
            };

            let col_map =
                col_map.expect(format!("error finding column map for row {}", row_idx).as_str());
            let mut row_out = String::new();
            let mut boundary_out = String::new();

            if col_map
                .first()
                .is_some_and(|c| matches!(c, Column::Empty(_)))
            {
                boundary_out.push_str(" ");
                row_out.push_str(" ");
            }

            let num_cols = col_map.len();
            for (col_idx, col) in col_map.iter().enumerate() {
                match col {
                    Column::Empty(size) => {
                        if col_idx == num_cols - 1 {
                            continue;
                        }

                        let mut out_cols: Vec<String> = Vec::new();
                        for _ in 0..*size {
                            out_cols.push(" ".repeat(COL_LENGTH));
                        }
                        let col_out = out_cols.join(" ");
                        row_out.push_str(col_out.as_str());
                        boundary_out.push_str(col_out.as_str());
                    }
                    Column::Occupied(size) => {
                        let mut out_cols: Vec<String> = Vec::new();
                        let mut boundary_cols: Vec<String> = Vec::new();
                        for _ in 0..*size {
                            let col_out = cols.pop().expect("");
                            out_cols.push(format_value(col_out));
                            boundary_cols.push("-".repeat(COL_LENGTH));
                        }
                        let col_out = out_cols.join("|");
                        row_out.push_str(format!("|{}|", &col_out).as_str());
                        let boundary = boundary_cols.join("+");
                        boundary_out.push_str(format!("+{}+", &boundary).as_str());
                    }
                }
            }
            out.push_str(row_out.as_str());
            out.push_str("\n");
            out.push_str(boundary_out.as_str());
            out.push_str("\n");
        }
        out.push_str("\n");
        out
    }
}

struct ErgodoxEz;

impl Keyboard for ErgodoxEz {
    fn layout_suffix(&self) -> String {
        todo!()
    }

    fn max_columns(&self) -> usize {
        todo!()
    }

    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        todo!()
    }

    fn rows(&self) -> usize {
        todo!()
    }

    fn thumb_rows(&self) -> HashSet<usize> {
        todo!()
    }
}

struct Moonlander;

impl Keyboard for Moonlander {
    fn layout_suffix(&self) -> String {
        "moonlander".to_string()
    }

    fn max_columns(&self) -> usize {
        14
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
                    Column::Empty(2),
                    Column::Occupied(3),
                    Column::Empty(3),
                ],
            ),
        ])
    }

    fn rows(&self) -> usize {
        6
    }

    fn thumb_rows(&self) -> HashSet<usize> {
        HashSet::from([5])
    }
}

struct Preonic;

impl Keyboard for Preonic {
    fn custom_keycode_prefix(&self) -> Option<String> {
        Option::from("preonic".to_string())
    }

    fn layout_suffix(&self) -> String {
        "preonic_grid".to_string()
    }

    fn max_columns(&self) -> usize {
        12
    }

    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        HashMap::from([])
    }

    fn rows(&self) -> usize {
        5
    }

    fn thumb_rows(&self) -> HashSet<usize> {
        HashSet::from([4])
    }

    fn is_rgb(&self) -> bool {
        false
    }
}

struct NyquistLM;

impl Keyboard for NyquistLM {
    fn layout_suffix(&self) -> String {
        "ortho_5x12".to_string()
    }

    fn max_columns(&self) -> usize {
        12
    }

    fn row_map(&self) -> HashMap<usize, Vec<Column>> {
        HashMap::from([])
    }

    fn rows(&self) -> usize {
        5
    }

    fn thumb_rows(&self) -> HashSet<usize> {
        HashSet::from([4])
    }
}