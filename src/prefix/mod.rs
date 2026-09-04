pub mod prefix;

use crate::prefix::prefix::get_prefix;
use std::path::Path;
use crate::internal;

pub fn get(reference: Option<String>, print: bool) {
    let prefix = match get_prefix() {
        Ok(prefix) => prefix,
        Err(why) => panic!("{}", why),
    };

    let resolved = if reference.is_some() {
        String::from(
            Path::join(Path::new(&prefix), Path::new(reference.unwrap().as_str()))
                .to_str()
                .unwrap(),
        )
    } else {
        prefix
    };

    if resolved.is_empty() {
        return;
    }

    let resolved = internal::normalize(&resolved);
    internal::copy(&resolved);

    if print {
        println!("{}", resolved);
    }
}
