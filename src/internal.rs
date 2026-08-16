use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

fn copy_to_clipboard(content: &String) {
    let cmd = Command::new("xclip").stdin(Stdio::piped()).spawn();
    if let Some(mut stdin) = cmd.unwrap().stdin.take() {
        stdin.write_all(content.as_bytes()).unwrap();
    }
}

fn copy_tmux_buffer(content: &String) {
    if env::var("TMUX").unwrap_or_default().is_empty() {
        return;
    }

    Command::new("tmux").arg("set-buffer").arg(format!("{}", content)).spawn().unwrap();
}

pub fn copy(content: &String) {
    copy_to_clipboard(content);
    copy_tmux_buffer(content);
}