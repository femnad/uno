pub mod comparison;

use std::process::exit;
use crate::latest_kernel::comparison::running_latest;

pub fn check() {
    let result = match running_latest() {
        Ok(result) => result,
        Err(why) => {
            println!("{}", why);
            exit(1);
        }
    };

    if result {
        exit(0);
    }

    exit(1);
}