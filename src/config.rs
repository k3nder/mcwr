use std::collections::HashMap;

use log::debug;

#[derive(Debug, PartialEq, Clone)]
pub enum Types {
    String(String),
    Vec(Vec<Types>),
    Map(HashMap<String, Types>),
    Number(f32),
}
#[derive(Debug)]
enum Flag {
    String,
    Number,
    Vec,
    Map,
    Normal,
}
impl Flag {
    pub fn string_delimiters() -> (char, char) {
        ('\'', '\'')
    }
    pub fn number_delimiters() -> (char, char) {
        ('*', '*')
    }
    pub fn vec_delimiters() -> (char, char) {
        ('[', ']')
    }
    pub fn map_delimiters() -> (char, char) {
        ('{', '}')
    }
    pub fn delimiters(&self) -> (char, char) {
        match self {
            Flag::String => Self::string_delimiters(),
            Flag::Number => Self::number_delimiters(),
            Flag::Vec => Self::vec_delimiters(),
            Flag::Map => Self::map_delimiters(),
            Flag::Normal => ('\0', '\0'), // No delimiters for normal mode
        }
    }
    pub fn start(ch: char) -> Flag {
        match ch {
            '\'' => Flag::String,
            '[' => Flag::Vec,
            '{' => Flag::Map,
            '*' => Flag::Number,
            _ => Flag::Normal,
        }
    }
}

impl Types {
    pub fn display(&self) -> String {
        match self {
            Types::String(val) => format!("{:?}", val),
            Types::Vec(items) => format!("{:?}", items),
            Types::Map(hash_map) => format!("{:?}", hash_map),
            Types::Number(number) => format!("{:?}", number),
        }
    }
    pub fn from_value(string: String) -> Option<Types> {
        if Self::is_map(&string) {
            Types::into_map(string)
        } else if Self::is_vec(&string) {
            Types::into_vec(string)
        } else if string.parse::<f32>().is_ok() {
            Some(Types::Number(string.parse::<f32>().unwrap()))
        } else {
            Some(Types::String(string))
        }
    }
    pub fn from(text: String) -> Option<Types> {
        if Self::is_string(&text) {
            return Self::into_string(text);
        } else if Self::is_num(&text) {
            return Self::into_number(text);
        } else if Self::is_vec(&text) {
            return Self::into_vec(text);
        } else if Self::is_map(&text) {
            return Self::into_map(text);
        } else {
            return None;
        }
    }
    pub fn is_string(string: &String) -> bool {
        let string = string.trim();
        string.starts_with("'") && string.ends_with("'")
    }
    pub fn is_vec(string: &String) -> bool {
        let string = string.trim();
        string.starts_with("[") && string.ends_with("]")
    }
    pub fn is_map(string: &String) -> bool {
        let string = string.trim();
        string.starts_with("{") && string.ends_with("}")
    }
    pub fn is_num(string: &String) -> bool {
        let mut string = string.trim().to_owned();
        string.starts_with('*') && string.ends_with('*') && {
            Self::trim_remove(&mut string);
            string.parse::<f32>().is_ok()
        }
    }
    pub fn into_string(string: String) -> Option<Types> {
        let mut string = string.clone().trim().to_owned();
        Self::trim_remove(&mut string);
        Some(Types::String(string))
    }
    pub fn into_number(string: String) -> Option<Types> {
        let mut string = string.clone().trim().to_owned();
        Self::trim_remove(&mut string);
        Some(Types::Number(string.parse::<f32>().unwrap()))
    }
    pub fn into_map(string: String) -> Option<Types> {
        let mut string = string.replace("\n", "").trim().to_owned();
        debug!(
            "ATTEMPT TO PARSING TO MAP, STRING WITTHOUT LINE BREAKS: {}",
            string
        );
        Self::trim_remove(&mut string);
        debug!("WITHOUT {{}} : {}", string);

        let mut tokens: Vec<String> = vec![];
        let mut cache = String::new();

        let chars: Vec<char> = string.chars().collect();

        // let mut vec_mode = false;
        // let mut string_mode = false;
        // let mut map_mode = false;
        // let mut num_mode = false;

        let mut flag = Flag::Normal;

        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }

            // string mode
            debug!("CHAR {}", c);

            match flag {
                Flag::Normal => {
                    flag = Flag::start(c);
                    if c == ';' {
                        debug!("-> SAVING TOKEN <-");
                        debug!("-> NEW TOKEN <-");
                        tokens.push(cache);
                        cache = String::new();
                    } else {
                        let new_flag = Flag::start(c);
                        if !matches!(new_flag, Flag::Normal) {
                            flag = new_flag;
                            debug!("-> {:?} MODE <-", flag);
                        }
                        cache.push(c);
                    }
                    i += 1;
                }
                _ => {
                    let (_, end) = flag.delimiters();
                    if c == end {
                        debug!("-> NORMAL MODE <-");
                        flag = Flag::Normal;
                        cache.push(c);
                        i += 1;
                        if !(i < chars.len()) {
                            debug!("-> SAVING TOKEN <-");
                            debug!("-> NEW TOKEN <-");
                            tokens.push(cache);
                            cache = String::new();
                        }
                        continue;
                    }
                    i += 1;
                    cache.push(c);
                    continue;
                }
            }
        }

        debug!("{:?}", tokens);

        let tokens: HashMap<String, Types> = tokens
            .iter()
            .map(|t| serialize_line(t.clone()).unwrap())
            .collect();
        Some(Types::Map(tokens))
    }
    pub fn into_vec(string: String) -> Option<Types> {
        let mut string = string.replace("\n", "").trim().to_owned();
        debug!(
            "ATTEMPT TO PARSING TO VEC, STRING WITTHOUT LINE BREAKS: {}",
            string
        );
        Self::trim_remove(&mut string);
        debug!("WITHOUT [] : {}", string);

        let mut tokens: Vec<String> = vec![];
        let mut cache = String::new();

        let chars: Vec<char> = string.chars().collect();

        // let mut vec_mode = false;
        // let mut string_mode = false;
        // let mut map_mode = false;
        // let mut num_mode = false;

        let mut flag = Flag::Normal;

        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }

            // string mode
            debug!("CHAR {}", c);

            match flag {
                Flag::Normal => {
                    flag = Flag::start(c);
                    if c == ',' {
                        debug!("-> SAVING TOKEN <-");
                        debug!("-> NEW TOKEN <-");
                        tokens.push(cache);
                        cache = String::new();
                    } else {
                        let new_flag = Flag::start(c);
                        if !matches!(new_flag, Flag::Normal) {
                            flag = new_flag;
                            debug!("-> {:?} MODE <-", flag);
                        }
                        cache.push(c);
                    }
                    i += 1;
                }
                _ => {
                    let (_, end) = flag.delimiters();
                    if c == end {
                        debug!("-> NORMAL MODE <-");
                        flag = Flag::Normal;
                        cache.push(c);
                        i += 1;
                        if !(i < chars.len()) {
                            debug!("-> SAVING TOKEN <-");
                            debug!("-> NEW TOKEN <-");
                            tokens.push(cache);
                            cache = String::new();
                        }
                        continue;
                    }
                    i += 1;
                    cache.push(c);
                    continue;
                }
            }
        }

        let tokens: Vec<Types> = tokens
            .iter()
            .map(|t| Types::from(t.clone()).unwrap())
            .collect();
        Some(Types::Vec(tokens))
    }
    fn trim_remove(string: &mut String) {
        string.remove(0);
        string.remove(string.len() - 1);
    }

    pub fn format_string(typ: &Types) -> String {
        match typ {
            Types::String(string) => format!("'{}'", string),
            _ => panic!("Isn't a string"),
        }
    }
    pub fn format_number(typ: &Types) -> String {
        match typ {
            Types::Number(number) => format!("*{}*", number),
            _ => panic!("Isn't a number"),
        }
    }
    pub fn format_vec(typ: &Types) -> String {
        match typ {
            Types::Vec(vector) => {
                format!("[{}]", {
                    let mut string = String::new();
                    for item in vector {
                        string.push_str(item.format().as_str());
                    }
                    string
                })
            }
            _ => panic!("Isn't a vec"),
        }
    }
    pub fn format_map(typ: &Types) -> String {
        match typ {
            Types::Map(vector) => {
                format!("{{{}}}", {
                    let mut string = String::new();
                    for (key, value) in vector {
                        string.push_str(format!("{}:{};", key, value.format().as_str()).as_str());
                    }
                    string
                })
            }
            _ => panic!("Isn't a map"),
        }
    }
    pub fn format(&self) -> String {
        match self {
            Types::String(_) => Self::format_string(self),
            Types::Vec(_) => Self::format_vec(self),
            Types::Map(_) => Self::format_map(self),
            Types::Number(_) => Self::format_number(self),
        }
    }
    pub fn get_string(&self) -> String {
        match self {
            Types::String(string) => string.clone(),
            _ => panic!("Isn't a string"),
        }
    }
    pub fn get_number(&self) -> f32 {
        match self {
            Types::Number(number) => number.clone(),
            _ => panic!("Isn't a number"),
        }
    }
    pub fn get_vec(&self) -> Vec<Types> {
        match self {
            Types::Vec(vec) => vec.clone(),
            _ => panic!("Isn't a vec"),
        }
    }
    pub fn get_map(&self) -> HashMap<String, Types> {
        match self {
            Types::Map(map) => map.clone(),
            _ => panic!("Isn't a map"),
        }
    }
}

pub fn serialize(text: String) -> HashMap<String, Types> {
    // separa en lineas
    let lines: Vec<String> = text.split("\n").map(|x| x.to_string()).collect();
    let mut map: HashMap<String, Types> = HashMap::new();
    // itera cada linea y las serializa, si la serializacion es invalida la salta, si es valida la guarda en el map
    for line in lines {
        // serializa la linea
        let serial = serialize_line(line.trim().to_string());
        if serial.is_none() {
            continue;
        }
        let serial = serial.unwrap();
        map.insert(serial.0, serial.1);
    }
    map
}
fn serialize_line(line: String) -> Option<(String, Types)> {
    // separa la clave y el valor apartir del separador ':'
    let separator = ":";
    // busca el separador, si esta devuelve su index, sino la funcion devuelve None
    let separator_index = match line.find(separator) {
        Option::Some(index) => index,
        _ => return Option::None,
    };
    // guarda clave y valor en variables
    let key = &line[..separator_index];
    let value = &line[separator_index + 1..];
    //println!("{}", value);
    Some((key.to_string(), Types::from(value.to_string()).unwrap()))
}

pub fn deserialize(map: HashMap<String, Types>) -> String {
    map.iter()
        .map(|(key, val)| format!("{}:{}", key, val.format()).to_string())
        .collect::<Vec<String>>()
        .join("\n")
}
