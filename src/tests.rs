use std::collections::HashMap;
use crate::config;

#[test]
fn config_serialize() {
    let test = r#"
    key:value
    key2:value2
    "#.to_string();
    let serial = config::serialize(test);
    let mut result = HashMap::new();
    result.insert(String::from("key"), String::from("value"));
    result.insert(String::from("key2"), String::from("value2"));
    assert_eq!(result, serial);
}

#[test]
fn config_deserialize() {
    let mut test = HashMap::new();
    test.insert(String::from("key"), String::from("value"));
    test.insert(String::from("key2"), String::from("value2"));

    let result = "key:value\nkey2:value2".to_string();

    let test = config::deserialize(test);

    assert_eq!(result, test);
}