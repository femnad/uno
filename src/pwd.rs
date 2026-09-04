use crate::internal;
use std::env;
use std::path::Path;

pub fn run(path: Option<String>, print: bool) {
    let cwd = env::current_dir().unwrap().to_string_lossy().to_string();
    let cwd = if path.is_some() {
        Path::new(&cwd)
            .join(path.unwrap())
            .to_string_lossy()
            .to_string()
    } else {
        cwd
    };

    let cwd = internal::normalize(&cwd);
    internal::copy(&cwd);

    if print {
        println!("{}", cwd);
    }
}
