use std::env;
use std::path::Path;
use crate::internal;

pub fn run(path: Option<String>, print: bool) {
    let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
    let cwd = if path.is_some() {
        Path::new(&cwd).join(path.unwrap()).to_string_lossy().to_string()
    } else {
        cwd
    };

    internal::copy_to_clipboard(cwd.clone());

    if print {
        println!("{}", cwd);
    }
}
