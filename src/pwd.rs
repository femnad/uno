use std::env;

use crate::internal;

pub fn run(copy: bool) {
    let cwd = env::current_dir().unwrap().to_string_lossy().to_string();

    if copy {
        internal::copy_to_clipboard(cwd);
        return;
    }

    println!("{}", cwd);
}
