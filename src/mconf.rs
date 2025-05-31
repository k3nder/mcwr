use std::{collections::HashMap, fs::read};
use std::fs;
use std::path::Path;

use config::Config;
use serde::Deserialize;

/// devuelve todas la configuraciones
pub fn config() -> Config {
    Config::builder()
        .add_source(config::File::with_name("mcwr.conf").format(config::FileFormat::Toml))
        .add_source(config::Environment::with_prefix("MCW_CONFIG"))
        .build()
        .unwrap()
}
/// optiene una configuracion
pub fn get<'a, T: Deserialize<'a>>(key: &str) -> T {
    config().get(key).unwrap()
}
pub fn get_or<'a, T: Deserialize<'a>>(key: &str, default: T) -> T {
    let val = config().get(key);
    if val.is_err() {
        return default;
    } else {
        return val.unwrap();
    }
}
