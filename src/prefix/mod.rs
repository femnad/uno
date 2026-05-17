pub mod prefix;

use std::path::Path;

use crate::prefix::prefix::get_prefix;

pub fn get(reference: Option<String>) {
    let prefix = match get_prefix() {
        Ok(prefix) => prefix,
        Err(why) => panic!("{}", why),
    };

    if !reference.is_some() {
        if prefix.is_empty() {
            return;
        }
        println!("{}", prefix.as_str());
        return;
    }

    let joined = Path::join(Path::new(&prefix), Path::new(reference.unwrap().as_str()));
    println!("{}", joined.display());
}
