pub mod prefix;

use crate::prefix::prefix::get_prefix;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn get(reference: Option<String>, copy: bool) {
    let prefix = match get_prefix() {
        Ok(prefix) => prefix,
        Err(why) => panic!("{}", why),
    };

    let resolved = if reference.is_some() {
        String::from(
            Path::join(Path::new(&prefix), Path::new(reference.unwrap().as_str()))
                .to_str()
                .unwrap(),
        )
    } else {
        prefix
    };

    if resolved.is_empty() {
        return;
    }

    if copy {
        let cmd = Command::new("xclip").stdin(Stdio::piped()).spawn();
        if let Some(mut stdin) = cmd.unwrap().stdin.take() {
            stdin.write_all(resolved.as_bytes()).unwrap();
        }
        return;
    }

    println!("{}", resolved);
}
