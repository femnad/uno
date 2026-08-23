use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn get_clipboard_command() -> Result<String, String> {
    let session_type = env::var("XDG_SESSION_TYPE").expect("Unable to determine session type");
    let session_type = session_type.as_str();
    match session_type {
        "wayland" => Ok("wl-copy".to_string()),
        "x11" => Ok("xclip".to_string()),
        _ => Err("Unrecognized session type".to_string()),
    }
}

fn copy_to_clipboard(content: &String) {
    let cmd = get_clipboard_command().expect("Unable to determine clipboard command");
    let cmd = Command::new(cmd).stdin(Stdio::piped()).spawn();
    if let Some(mut stdin) = cmd.expect("Unable to run clipboard command").stdin.take() {
        stdin.write_all(content.as_bytes()).unwrap();
    }
}

fn copy_tmux_buffer(content: &String) {
    if env::var("TMUX").unwrap_or_default().is_empty() {
        return;
    }

    Command::new("tmux")
        .arg("set-buffer")
        .arg(format!("{}", content))
        .spawn()
        .unwrap();
}

pub fn copy(content: &String) {
    copy_to_clipboard(content);
    copy_tmux_buffer(content);
}
