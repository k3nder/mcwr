use std::collections::HashMap;

pub fn serialize(text: String) -> HashMap<String, String> {
    // separa en lineas
    let lines: Vec<String> = text.split("\n").map(|x| x.to_string()).collect();
    let mut map: HashMap<String, String> = HashMap::new();
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
fn serialize_line(line: String) -> Option<(String, String)> {
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
    Some((key.to_string(), value.to_string()))
}

pub fn deserialize(map: HashMap<String, String>) -> String {
     map.iter().map(|(key, val)| format!("{}:{}", key, val).to_string() )
        .collect::<Vec<String>>().join("\n")
}