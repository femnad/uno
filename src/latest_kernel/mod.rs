pub mod comparison;

use std::process::exit;
use crate::latest_kernel::comparison::kernel_info;

pub fn check(print: bool) {
    let result = match kernel_info() {
        Ok(result) => result,
        Err(why) => {
            println!("{}", why);
            exit(1);
        }
    };

    if print {
        println!("Current kernel version : {}", result.running);
        println!("Latest kernel version  : {}", result.latest);
        return
    }

    let is_running_latest = result.running.eq(result.latest.as_str());
    if is_running_latest {
        exit(0);
    }

    exit(1);
}