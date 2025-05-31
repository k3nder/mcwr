use dwldutil::Downloader;
use log::warn;
use mcd::api::client::Client;
use mcd::api::manifest::{Latest, Manifest};
use mcd::api::{ApiClientError, ApiClientUtil};
use mcd::command::{self, Command};
use mcd::errors::FetchError;
use mcd::file::fetch_client;
use mcd::java::JavaUtil;
use mcd::libs::LibsUtil;
use mcd::resource::ResourceUtil;
use serde::{Deserialize, Serialize};

use crate::mconf;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{fs, i32};

static META_FILE: &str = ".info";
static CLIENT_FILE: &str = "client.jar";
static LIBS_DIR: &str = "libs";
static NATIVES_DIR: &str = "natives";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub pwd: String,
    pub version: String,
    pub assets: String,
    pub main: String,
    pub java: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub options: HashMap<String, usize>,
    pub data: HashMap<String, String>,
    pub version_type: String,
    pub classpath: String,
}
impl Version {
    pub fn run(self, stdout: fn(String), stderr: fn(String), workdir: String) {
        // declarar variables
        let dir = Path::new(&self.pwd);
        let assets: String = mconf::get("assets");
        let username = mconf::get("username");

        let natives_dir = format!("{}/{}", dir.to_str().unwrap(), NATIVES_DIR);
        let libs_path = format!("{}/{}", dir.to_str().unwrap(), LIBS_DIR);
        let jar_file = format!("{}/{}", dir.to_str().unwrap(), CLIENT_FILE);

        let mut data: HashMap<String, String> = mconf::get::<HashMap<String, String>>("data");

           data.insert("natives_directory".to_owned(), natives_dir);
           data.insert(
               "classpath".to_owned(),
               self.classpath,
           );
           data.insert("main_class".to_owned(), self.main);
           data.insert("auth_player_name".to_owned(), username);
           data.insert("version_name".to_owned(), self.version);
           data.insert("game_directory".to_owned(), self.pwd);
           data.insert("assets_root".to_owned(), mconf::get("assets"));
           data.insert(
               "game_assets".to_owned(),
               format!("{}/virtual/legacy/", assets),
           );
           data.insert("assets_index_name".to_owned(), self.assets);
           data.insert("auth_uuid".to_owned(), mconf::get("uuid"));
           data.insert("auth_access_token".to_owned(), mconf::get("token"));
           data.insert("clientid".to_owned(), mconf::get("clientid"));
           data.insert("auth_xuid".to_owned(), mconf::get("xuid"));
           data.insert("user_type".to_owned(), mconf::get("usertype"));
           data.insert("version_type".to_owned(), self.version_type);
           data.insert("library_directory".to_owned(), libs_path);
           data.insert("classpath_separator".to_owned(), ":".to_owned());

           let mut child = Command::from_args(self.game_args, self.jvm_args, data)
            .execute(
               self.java,
               vec![],
           ).unwrap();

           if let Some(stdout) = child.stdout.take() {
                   let reader = BufReader::new(stdout);

                   // Leemos línea a línea y las imprimimos
                   for line in reader.lines() {
                    let line = line.unwrap();
                       println!("{}", line);
                   }
               }
               let status = child.wait().unwrap();


        // ejecutar commando
        //Command {
        //    resources: CommandResourcesConfig {
        //        libraries: format!("{}/{}", dir.to_str().unwrap(), LIBS_DIR),
        //        jar_file: format!("{}/{}", dir.to_str().unwrap(), CLIENT_FILE),
        //        bin: format!("{}/{}", dir.to_str().unwrap(), NATIVES_DIR),
        //        logger: mconf::get("logger").get_string(),
        //    },
        //    java_home: self.java.to_string(),
        //    game_dir: workdir,
        //    assets: CommandAssetsConfig {
        //        assets_dir: assets,
        //        assets_index: (&self.assets).clone(),
        //    },
        //    user: CommandUserConfig {
        //        user_type: mconf::get_or("usertype", Types::String("user".to_owned())).get_string(),
        //        client_id: mconf::get_or("clientid", Types::String("client".to_owned()))
        //            .get_string(),
        //        uuid: mconf::get_or("uuid", Types::String("0".to_owned())).get_string(),
        //        xuid: mconf::get_or("xuid", Types::String("0".to_owned())).get_string(),
        //        access_token: mconf::get_or("token", Types::String("0".to_owned())).get_string(),
        //        user_name: username,
        //    },
        //    version: CommandVersionConfig {
        //        version_id: self.version.clone(),
        //        version_type: "Vanilla".to_owned(),
        //        main_class: self.main.clone(),
        //    },
        //    ram: CommandRamConfig {
        //        xmx: mconf::get("xmx").get_number() as i32,
        //        xms: mconf::get("xms").get_number() as i32,
        //    },
        //    event: stdout,
        //    err_event: stderr,
        //    args: self.args.clone(),
        //    jvm: self.jvm.clone(),
        //}
        //.run(RunType::NORMAL);
    }
}

pub fn download(client: &Client, assets: bool) {
    // definir variables y paths
    let dir = format!(
        "{}/{}",
        mconf::get::<String>("versions"),
        client.id
    );
    let dir = Path::new(&dir);
    if !dir.exists() {
        fs::create_dir_all(dir).unwrap();
    }
    let meta_file = format!("{}/{}", dir.display(), META_FILE);
    let meta_file = Path::new(&meta_file);
    let libs_path = format!("{}/{}", dir.display(), LIBS_DIR);
    let natives_path = format!("{}/{}", dir.display(), NATIVES_DIR);
    let jar_path = format!("{}/{}", dir.display(), CLIENT_FILE);
    let libs_path = Path::new(&libs_path);
    let natives_path = Path::new(&natives_path);
    let jar_path = Path::new(&jar_path);
    // iniciar utilitarios
    let javau = JavaUtil::new();
    let libsu = LibsUtil::new();
    let resu = ResourceUtil::new();

    // crear una lista que contandra los archivos para descargar
    let mut files = Vec::new();

    // descargar java
    println!("Downloading... Java");
    match javau.fetch(client.java(), &mconf::get::<String>("java")) {
        Ok(f) => files.push(f),
        Err(e) => warn!("{}", e),
    }
    // crear archivo .info
    println!("Creating... Meta");
    let (game, jvm) = command::build_args(&client, mconf::get::<HashMap<String, bool>>("options"));

    // Download libs
    println!("Downloading... Libs");
    let mut classpath = match libsu.fetch(libs_path.to_str().unwrap(), natives_path.to_str().unwrap(), &client) {
            Ok((mut fis, classpath)) => { files.append(&mut fis); classpath },
            Err(e) => { warn!("{}", e); Vec::new() },
        };

    classpath.push(String::from(jar_path.to_str().unwrap()));

    let version = Version {
        pwd: mconf::get("pwd"),
        version: client.id.clone(),
        assets: client.assets.clone(),
        main: client.main_class.clone(),
        java: format!("{}/{}/bin/java", mconf::get::<String>("java"), javau.id_of(client.java()).unwrap()),
        jvm_args: jvm,
        game_args: game,
        options: HashMap::new(),
        data: HashMap::new(),
        version_type: client.version_type.clone(),
        classpath: classpath.join(":"),
    };
   create_meta(&meta_file, version);
    // descargar el jar del juego
    println!("Downloading... Game");
    match fetch_client(&client, jar_path.to_str().unwrap()) {
        Ok(f) => files.push(f),
        Err(e) => warn!("{}", e),
    }
    //if !Path::new(mconf::get("logger").get_string().as_str()).exists() {
    //    if let Some(logg) = &version.clone().logging {
    //        mc::get_config_logger(logg, mconf::get("logger").get_string().as_str());
    //    }
    //}
    // si se piden, descargar assets
    if assets {
        println!("Downloading... Assets");
        let indexes_loc = format!("{}/indexes", assets);
        if !Path::new(&indexes_loc).exists() {
            fs::create_dir(&indexes_loc);
        }
        let index = resu.index_of(&client, &format!("{}/{}.json", &indexes_loc, &client.assets)).unwrap();
        match resu.fetch(&index, mconf::get("assets")) {
            Ok(mut fis) => files.append(&mut fis),
            Err(e) => warn!("{}", e),
        }

    }

    Downloader::new().with_max_concurrent_downloads(mconf::get("max_current_downloads")).with_files(files).start();
}

/// Crea un archivo establecido en `dir` y escribe el contenido de `map`
fn create_meta(dir: &Path, version: Version) {
    let deserialize = toml::to_string(&version).unwrap();
    fs::write(dir, deserialize).unwrap();
}
/// Lista todas las versions
pub fn list() -> HashMap<String, Version> {
    // define el dir e inicializa el map
    let dir: String = mconf::get("versions");
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
    let meta_file = format!("{}/{}", dir.to_str().unwrap(), META_FILE);
    let meta_file = Path::new(meta_file.as_str());
    let content = fs::read_to_string(meta_file).unwrap();
    let version: Version = toml::from_str(&content).unwrap();
    (version.version.clone(), version)
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
    let path = format!("{}/{}", mconf::get::<String>("versions"), version);
    let version_path = Path::new(path.as_str());
    fs::remove_dir_all(version_path).ok();
}

/// lista todas las versiones del manifest
pub fn list_manifest() -> Vec<String> {
    let manifest = manifest().unwrap();
    manifest
        .versions
        .iter()
        .map(|version| version.id.clone())
        .collect()
}
/// get manifest latest
pub fn manifest_latest() -> Latest {
    let manifest = manifest().unwrap();
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
pub fn manifest() -> Result<Manifest, ApiClientError> {
    Ok(ApiClientUtil::new(&mconf::get::<String>("manifest"))?.manifest)
}
