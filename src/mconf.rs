use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::config;
/// devuelve todas la configuraciones
pub fn config() -> HashMap<String, String> {
    let file = Path::new("user.conf");
    let config = fs::read_to_string(file).unwrap();
    config::serialize(config)
}
/// optiene una configuracion
pub fn get(key: &str) -> String {
    match config().get(key).map(|s| s.clone()) {
        Some(s) => s,
        None => panic!("Unknown configuration: {}", key),
    }
}
/// guarda una configuracion
pub fn save(map: HashMap<String, String>) {
    let file = Path::new("user.conf");
    let config = config::deserialize(map);
    fs::write(file, config).unwrap();
}
/// settea una configuracion
pub fn set(key: &str, val: String) {
    let mut map = config();
    map.insert(key.to_string(), val);
    save(map);
}