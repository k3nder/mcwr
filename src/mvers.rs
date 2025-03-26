use crate::{config, mconf};
use mclr::deserialize::json_manifest::Latest;
use mclr::deserialize::json_version::JsonVersion;
use mclr::mc;
use mclr::mc::get_compatible_java;
use mclr::mc::utils::command_builder::{
    CommandAssetsConfig, CommandRamConfig, CommandResourcesConfig, CommandUserConfig,
    CommandVersionConfig, RunType,
};
use mclr::utils::HandleEvent;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
#[derive(Debug, Clone)]
pub struct Version {
    pub pwd: String,
    pub version: String,
    pub assets: String,
    pub main: String,
    pub java: String,
    pub args: Vec<String>,
}
impl Version {
    pub fn run(&self, stdout: fn(String), stderr: fn(String), workdir: String) {
        // declarar variables
        let dir = Path::new(&self.pwd);
        let assets = mconf::get("assets");
        let username = mconf::get("username");

        // ejecutar commando
        mc::utils::command_builder::Command {
            resources: CommandResourcesConfig {
                libraries: format!("{}/libs", dir.to_str().unwrap()),
                jar_file: format!("{}/game.jar", dir.to_str().unwrap()),
                bin: format!("{}/natives", dir.to_str().unwrap()),
                logger: mconf::get("logger"),
            },
            java_home: self.java.to_string(),
            game_dir: workdir,
            assets: CommandAssetsConfig {
                assets_dir: assets,
                assets_index: (&self.assets).clone(),
            },
            user: CommandUserConfig {
                user_type: mconf::get_or("usertype", "user"),
                client_id: mconf::get_or("clientid", "client"),
                uuid: mconf::get_or("uuid", "0"),
                xuid: mconf::get_or("xuid", "0"),
                access_token: mconf::get_or("token", "0"),
                user_name: username,
            },
            version: CommandVersionConfig {
                version_id: self.version.clone(),
                version_type: "Vanilla".to_owned(),
                main_class: self.main.clone(),
            },
            ram: CommandRamConfig {
                xmx: mconf::get("xmx")
                    .parse()
                    .expect("XMX configuration isn't number"),
                xms: mconf::get("xms")
                    .parse()
                    .expect("XMS configuration isn't number"),
            },
            event: stdout,
            err_event: stderr,
            args: self.args.clone(),
        }
        .run(RunType::NORMAL);
    }
}

pub fn download(version: JsonVersion, assets: bool) {
    // definir variables y paths
    let dir = format!("{}/{}", mconf::get("versions").as_str(), version.id);
    let dir = Path::new(&dir);
    if !dir.exists() {
        fs::create_dir_all(dir).unwrap();
    }
    let meta_file = format!("{}/.info", dir.display());
    let meta_file = Path::new(&meta_file);
    let libs_path = format!("{}/libs", dir.display());
    let natives_path = format!("{}/natives", dir.display());
    let game_path = format!("{}/game.jar", dir.display());
    let libs_path = Path::new(&libs_path);
    let natives_path = Path::new(&natives_path);
    let game_path = Path::new(&game_path);
    if !libs_path.exists() {
        fs::create_dir_all(libs_path).unwrap();
    }
    if !natives_path.exists() {
        fs::create_dir_all(natives_path).unwrap();
    }
    // descargar java
    println!("Downloading... Java");
    let java_home = get_compatible_java(mconf::get("java").as_str(), &version.java_version.clone());
    // crear archivo .info
    println!("Creating... Meta");
    let mut meta = HashMap::new();
    meta.insert("version".to_string(), version.id.clone());
    meta.insert("assets".to_string(), version.assets.clone());
    meta.insert("main".to_string(), version.main_class.clone());
    meta.insert("java".to_string(), java_home);
    meta.insert("args".to_string(), "".to_string());
    create_meta(&meta_file, meta);
    // descargar librerias
    println!("Downloading... Libs");
    let libs = &version.libraries.clone();
    mc::utils::libs_utils::find(
        libs_path.to_str().unwrap(),
        natives_path.to_str().unwrap(),
        libs,
        HandleEvent::new(move |e| {
            println!("LIBS[{}]", e.percent());
        }),
    )
    .expect("Error downloading libs")
    .start();
    // descargar el jar del juego
    println!("Downloading... Game");
    mc::download(game_path.to_str().unwrap(), &version);
    if !Path::new(mconf::get("logger").as_str()).exists() {
        if let Some(logg) = &version.clone().logging {
            mc::get_config_logger(logg, mconf::get("logger").as_str());
        }
    }
    // si se piden, descargar assets
    if assets {
        println!("Downloading... Assets");
        mc::utils::assets_utils::find(
            mconf::get("assets").as_str(),
            &version,
            HandleEvent::new(move |_| {}),
        )
        .start();
    }
}

/// Crea un archivo establecido en `dir` y escribe el contenido de `map`
fn create_meta(dir: &Path, map: HashMap<String, String>) {
    let deserialize = config::deserialize(map);
    fs::write(dir, deserialize).unwrap();
}
/// Lista todas las versions
pub fn list() -> HashMap<String, Version> {
    // define el dir e inicializa el map
    let dir = mconf::get("versions");
    let dir = Path::new(dir.as_str());
    let mut map: HashMap<String, Version> = HashMap::new();
    // por cada directorio existente en dir, si contiene un .info, lo anyade al map
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let meta = path.metadata().unwrap();
        if meta.is_dir() {
            let version = read_dir_to_version(&path);
            map.insert(version.0, version.1);
        }
    }
    map
}

/// lee el directorio a version
fn read_dir_to_version(dir: &PathBuf) -> (String, Version) {
    // define variables
    let meta_file = format!("{}/.info", dir.to_str().unwrap());
    let meta_file = Path::new(meta_file.as_str());
    let content = fs::read_to_string(meta_file).unwrap();
    let meta = config::serialize(content);
    let version_name = meta.get("version").unwrap().to_string();
    let args_str = meta.get("args").unwrap().to_string();
    let mut args = vec![];
    if !args_str.is_empty() {
        args = args_str.split(",,").map(|s| s.to_string()).collect();
    }

    // crea la version y la devuelve
    let version = Version {
        pwd: dir.to_str().unwrap().to_string(),
        version: version_name.clone(),
        assets: meta.get("assets").unwrap().to_string(),
        main: meta.get("main").unwrap().to_string(),
        java: meta.get("java").unwrap().to_string(),
        args,
    };
    (version_name, version)
}
/// Obtiene una version en concreto
pub fn get(version: String) -> Option<Version> {
    if list().get(&version).is_none() {
        return None;
    } else {
        Some(list().get(&version).unwrap().clone())
    }
}
/// elimina una version
pub fn remove(version: String) {
    let path = format!("versions/{}", version);
    let version_path = Path::new(path.as_str());
    fs::remove_dir_all(version_path).ok();
}

/// lista todas las versiones del manifest
pub fn list_manifest() -> Vec<String> {
    let manifest = mclr::utils::manifest::manifest();
    manifest
        .versions
        .iter()
        .map(|version| version.id.clone())
        .collect()
}
/// get manifest latest
pub fn manifest_latest() -> Latest {
    let manifest = mclr::utils::manifest::manifest();
    manifest.latest
}
/// get manifest latest release
pub fn manifest_latest_release() -> String {
    manifest_latest().release.clone()
}
/// get manifest latest snapshot
pub fn manifest_latest_snapshot() -> String {
    manifest_latest().snapshot.clone()
}
