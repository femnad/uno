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
