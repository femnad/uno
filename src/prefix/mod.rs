pub mod prefix;

use std::path::Path;
use std::process::Command;

pub fn get_prefix() -> Result<String, String> {
    let cmd = Command::new("git")
        .arg("rev-parse")
        .arg("--show-prefix")
        .output()
        .expect("failed to determine git repo prefix");

    if cmd.status.success() {
        let stdout = String::from_utf8_lossy(&cmd.stdout).trim().to_string();
        return Ok(stdout);
    }

    Err(String::from_utf8(cmd.stderr).unwrap().to_string())
}

pub fn get(reference: Option<String>) {
    let prefix = match get_prefix() {
        Ok(prefix) => prefix,
        Err(why) => panic!("{}", why),
    };

    if !reference.is_some() {
        if prefix.is_empty() {
            return;
        }
        println!("{}", prefix.as_str());
        return;
    }

    let joined = Path::join(Path::new(&prefix), Path::new(reference.unwrap().as_str()));
    println!("{}", joined.display());
}
