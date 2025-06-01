use dwldutil::Downloader;
use log::{info, trace, warn};
use mcd::api::client::Client;
use mcd::api::manifest::{Latest, Manifest};
use mcd::api::{ApiClientError, ApiClientUtil};
use mcd::command::{build_args, Command};
use mcd::errors::CommandError;
use mcd::file::fetch_client;
use mcd::java::JavaUtil;
use mcd::libs::LibsUtil;
use mcd::resource::ResourceUtil;
use serde::{Deserialize, Serialize};

use crate::errors::{self, DownloadError, ReadingError};
use crate::mconf;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::fs;

#[cfg(target_os = "linux")]
static CP_SEPARATOR: char = ':';
#[cfg(target_os = "windows")]
static CP_SEPARATOR: char = ';';

#[cfg(target_os = "linux")]
static JAVA_BIN: &str = "java";
#[cfg(target_os = "windows")]
static JAVA_BIN: &str = "java.exe";
static META_FILE: &str = ".info";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub pwd: String,
    pub version: String,
    pub assets: String,
    pub main: String,
    pub java: String,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub data: HashMap<String, String>,
    pub version_type: String,
    pub natives: String,
    pub libraries: String,
    pub classpath: String,
    pub java_version: usize,
}
impl Version {
    pub fn run(self, stdout_callback: fn(String), stderr_callback: fn(String)) -> Result<(), CommandError> {
        trace!("RUNNING VERSION {}", self.version);
        let mut data = data();
        trace!("WITH DEFAULT DATA {:?}", data);
        data.insert("natives_directory".to_owned(), self.natives);
        data.insert(
            "classpath".to_owned(),
            self.classpath,
        );
        data.insert("main_class".to_owned(), self.main);
        data.insert("version_name".to_owned(), self.version);
        data.insert("game_directory".to_owned(), self.pwd);
        data.insert("assets_index_name".to_owned(), self.assets.clone());
        data.insert("version_type".to_owned(), self.version_type);
        data.insert("library_directory".to_owned(), self.libraries);
        for (k,v) in self.data {
            data.insert(k, v);
        }
        trace!("FINAL DATA {:?}", data);
        trace!("BUILDING COMMAND WITH ARGS \n\tJVM ARGS: {:?}\n\tGAME ARGS: {:?}", self.jvm_args, self.game_args);
        let command = Command::from_args(self.game_args, self.jvm_args, data);
        trace!("COMMAND BUILDED... EXECUTING");
        let mut child = command.execute(self.java, vec![], self.java_version)?;

        let stdout = child.stdout.take().expect("NO STDOUT");
        let stderr = child.stderr.take().expect("NO STDERR");
        let stdout = BufReader::new(stdout);
        let stderr = BufReader::new(stderr);

        for line in stdout.lines() {
            let line = line.unwrap();
            stdout_callback(line);
        }
        for line in stderr.lines() {
            let line = line.unwrap();
            stderr_callback(line);
        }
        Ok(())
    }
    pub fn from_path(dir: &PathBuf) -> Result<Version, ReadingError> {
        // define variables
        trace!("READING VERSION FROM PATH {:?}", dir);
        let meta_file = format!("{}/{}", dir.to_str().expect("CANNOT CONVERT TO STRING"), META_FILE);
        trace!("METADATA FILE WILL LOCATED IN {}", meta_file);
        let meta_file = Path::new(meta_file.as_str());
        if !meta_file.exists() {
            warn!("PATH {:?} IS NOT A VERSION", dir);
        }
        let content = fs::read_to_string(meta_file)?;
        trace!("CONTENT OF THE META FILE {}",content);
        let version: Version = toml::from_str(&content)?;
        trace!("VERSION SUCCESSFUL LOADED");
        Ok(version)
    }
    /// Crea un archivo establecido en `dir` y escribe el contenido de `map`
    fn mkmeta(self, dir: &str) -> Result<(), errors::WritingError> {
        trace!("CALL TO MKMETA, CREATING META FILE ON {}", dir);
        trace!("DESERIALIZING VERSION");
        let deserialize = toml::to_string(&self)?;
        trace!("VERSION DESERIALIZED ON {}", deserialize);
        trace!("WRITING FILE");
        fs::write(dir, deserialize)?;
        trace!("FILE SUCCESSFUL WRITED");
        Ok(())
    }
}
pub fn download(client: &Client, assets: bool) -> Result<(), DownloadError> {
    // Crear utilitarios
    trace!("CALL TO DOWNLOAD, ASSETS: {}", assets);
    trace!("INITIALIZING UTILITIES");
    let javau = JavaUtil::new();
    let libsu = LibsUtil::new();
    let resu = ResourceUtil::new();

    // definir paths
    trace!("DEFINING PATHS");
    let home = format!("{}/{}", mconf::get::<String>("versions"), client.id);
    let jar_path = format!("{}/{}.jar", home, client.id);
    let libs_path = format!("{}/libraries", home);
    let resource_path = mconf::get::<String>("resources");
    let index_path = format!("{}/indexes/{}.json", resource_path, client.assets);
    let java_home = mconf::get::<String>("java");
    let natives_path = format!("{}/bin", home);
    let info_path = format!("{}/.info", home);
    trace!("HOME: {}\n\tJAR: {}\n\tLIBS: {}\n\tRESOURCES: {}\n\tINDEX: {}\n\tJAVA: {}\n\tNATIVES: {}\n\tINFO: {}",
        home, jar_path, libs_path, resource_path, index_path, java_home, natives_path, info_path);
    // crear cola de descarga
    let mut files = Vec::new();
    // anyadir el cliente
    trace!("FETCH ON CLIENT");
    match fetch_client(&client, &jar_path) {
        Ok(file) => files.push(file),
        Err(e) => warn!("WARN --- {}", e),
    }
    // anyadir la version de java
    trace!("FETCH ON JAVA");
    match javau.fetch(client.java(), &java_home) {
        Ok(file) => files.push(file),
        Err(e) => warn!("WARN --- {}", e),
    }
    // anyadir las librerias
    trace!("FETCH ON LIBS");
    let mut classpath = match libsu.fetch(&libs_path, &natives_path, &client) {
        Ok((mut file, classpath)) => { files.append(&mut file); classpath },
        Err(e) => { warn!("WARN --- {}", e); Vec::new() },
    };
    // anyadir el cliente al classpath
    classpath.push(jar_path);
    // anyadir librerias si se pide
    if assets {
        trace!("FETCH ON ASSETS");
        let index = resu.index_of(&client, &index_path)?;
        match resu.fetch(&index, &resource_path) {
            Ok(mut file) => files.append(&mut file),
            Err(e) => warn!("WARN --- {}", e),
        }
    }
    // descargar todo
    info!("DOWNLOADING...");
    Downloader::new().with_files(files).with_max_concurrent_downloads(mconf::get("max_current_downloads")).start();
    // obtener argumentos
    trace!("BUILDING ARGS");
    let (game, jvm) = build_args(&client, mconf::get("options"));
    // registrar la version
    let version = Version {
        pwd: mconf::get("pwd"),
        version: client.id.clone(),
        assets: client.assets.clone(),
        main: client.main_class.clone(),
        java: format!("{}/{}/bin/{}", mconf::get::<String>("java"), javau.id_of(client.java()).unwrap(), JAVA_BIN),
        jvm_args: jvm,
        game_args: game,
        data: HashMap::new(),
        version_type: client.version_type.clone(),
        classpath: classpath.join(CP_SEPARATOR.to_string().as_str()),
        natives: natives_path,
        libraries: libs_path,
        java_version: client.java()
    };
    // escribir el archivo de metadatos
    trace!("WRITING METADATA");
    version.mkmeta(&info_path)?;
    Ok(())
}

/// Lista todas las versions
pub fn list() -> Result<HashMap<String, Version>, errors::ReadingError> {
    trace!("CALL TO LIST");
    // define el dir e inicializa el map
    let dir: String = mconf::get("versions");
    trace!("REAIDNG VERSIONS DIR: {}", dir);
    let dir = Path::new(dir.as_str());
    let mut map: HashMap<String, Version> = HashMap::new();
    // por cada directorio existente en dir, si contiene un .info, lo anyade al map
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        trace!("FOUND ENTRY ON DIR: {:?}", entry);
        let path = entry.path();
        let meta = path.metadata()?;
        if meta.is_dir() {
            trace!("LOADING DIR: {:?}", entry);
            let version = Version::from_path(&path)?;
            map.insert(version.version.clone(), version);
        }
    }
    Ok(map)
}

/// Obtiene una version en concreto
pub fn get(version: String) -> Option<Version> {
    trace!("CALL GET {}", version);
    let list = list().unwrap_or(HashMap::new());
    if list.get(&version).is_none() {
        warn!("VERSION NOT FOUND");
        return None;
    } else {
        Some(list.get(&version).unwrap().clone())
    }
}
/// elimina una version
pub fn remove(version: String) {
    trace!("CALL TO REMOVE {}", version);
    let path = format!("{}/{}", mconf::get::<String>("versions"), version);
    trace!("ON PATH {}", path);
    let version_path = Path::new(path.as_str());
    fs::remove_dir_all(version_path).ok();
}

/// lista todas las versiones del manifest
pub fn list_manifest() -> Vec<String> {
    trace!("CALL TO LIST MANIFEST");
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
fn data() -> HashMap<String, String> {
    let mut data: HashMap<String, String> = mconf::get::<HashMap<String, String>>("data");
    data.insert("auth_uuid".to_owned(), mconf::get_or("uuid", String::from("000")));
    data.insert("auth_access_token".to_owned(), mconf::get_or("token", String::from("000")));
    data.insert("clientid".to_owned(), mconf::get_or("clientid", String::from("000")));
    data.insert("auth_xuid".to_owned(), mconf::get_or("xuid", String::from("000")));
    data.insert("user_type".to_owned(), mconf::get_or("usertype", String::from("msa")));
    data.insert("classpath_separator".to_owned(), CP_SEPARATOR.to_string());
    data.insert("assets_root".to_owned(), mconf::get("resources"));
    data.insert("game_assets".to_owned(), mconf::get("resources"));
    data
}
