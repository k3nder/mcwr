use crate::config::{self, Types};
use std::collections::HashMap;

#[test]
fn config_serialize() {
    let test = r#"
    key:'value'
    key2:'value2'
    "#
    .to_string();
    let serial = config::serialize(test);
    let mut result = HashMap::new();
    result.insert(String::from("key"), Types::String(String::from("value")));
    result.insert(String::from("key2"), Types::String(String::from("value2")));
    assert_eq!(result, serial);
}

#[test]
fn config_deserialize() {
    let mut test = HashMap::new();
    test.insert(String::from("key"), Types::String(String::from("value")));
    test.insert(String::from("key2"), Types::String(String::from("value2")));

    let result = "key2:'value2'\nkey:'value'".to_string();

    let test = config::deserialize(test);

    assert_eq!(result, test);
}

#[test]
fn config_string_parse() {
    let string = String::from("'hello'");
    assert_eq!(Types::is_string(&string), true);
    let typ = Types::into_string(string).unwrap();
    assert_eq!(typ, Types::String(String::from("hello")));
}

#[test]
fn config_vec_parse() {
    env_logger::init();
    let vec = String::from("[*1*, *1*, ['1', '2']]");
    assert_eq!(Types::is_vec(&vec), true);
    let typ = Types::into_vec(vec);
    println!("{:?}", typ);
    //assert_eq!(true, false);
}
