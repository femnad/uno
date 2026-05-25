use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(content: String) {
    let cmd = Command::new("xclip").stdin(Stdio::piped()).spawn();
    if let Some(mut stdin) = cmd.unwrap().stdin.take() {
        stdin.write_all(content.as_bytes()).unwrap();
    }
}
