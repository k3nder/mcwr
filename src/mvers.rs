use crate::config::Types;
use crate::{config, mconf};
use mclr::deserialize::json_manifest::Latest;
use mclr::deserialize::json_version::JsonVersion;
use mclr::mc;
use mclr::mc::get_compatible_java;
use mclr::utils::io_utils::system::OperatingSystem;
use mclr::utils::HandleEvent;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::{fs, i32};

static META_FILE: &str = ".info";
static CLIENT_FILE: &str = "client.jar";
static LIBS_DIR: &str = "libs";
static NATIVES_DIR: &str = "natives";

#[derive(Debug, Clone)]
pub struct Version {
    pub pwd: String,
    pub version: String,
    pub assets: String,
    pub main: String,
    pub java: String,
    pub jvm: Vec<String>,
    pub args: Vec<String>,
}
impl Version {
    pub fn run(&self, stdout: fn(String), stderr: fn(String), workdir: String) {
        // declarar variables
        let dir = Path::new(&self.pwd);
        let assets = mconf::get("assets").get_string();
        let username = mconf::get("username").get_string();

        // ejecutar commando
        Command {
            resources: CommandResourcesConfig {
                libraries: format!("{}/{}", dir.to_str().unwrap(), LIBS_DIR),
                jar_file: format!("{}/{}", dir.to_str().unwrap(), CLIENT_FILE),
                bin: format!("{}/{}", dir.to_str().unwrap(), NATIVES_DIR),
                logger: mconf::get("logger").get_string(),
            },
            java_home: self.java.to_string(),
            game_dir: workdir,
            assets: CommandAssetsConfig {
                assets_dir: assets,
                assets_index: (&self.assets).clone(),
            },
            user: CommandUserConfig {
                user_type: mconf::get_or("usertype", Types::String("user".to_owned())).get_string(),
                client_id: mconf::get_or("clientid", Types::String("client".to_owned()))
                    .get_string(),
                uuid: mconf::get_or("uuid", Types::String("0".to_owned())).get_string(),
                xuid: mconf::get_or("xuid", Types::String("0".to_owned())).get_string(),
                access_token: mconf::get_or("token", Types::String("0".to_owned())).get_string(),
                user_name: username,
            },
            version: CommandVersionConfig {
                version_id: self.version.clone(),
                version_type: "Vanilla".to_owned(),
                main_class: self.main.clone(),
            },
            ram: CommandRamConfig {
                xmx: mconf::get("xmx").get_number() as i32,
                xms: mconf::get("xms").get_number() as i32,
            },
            event: stdout,
            err_event: stderr,
            args: self.args.clone(),
            jvm: self.jvm.clone(),
        }
        .run(RunType::NORMAL);
    }
}

pub fn download(version: JsonVersion, assets: bool) {
    // definir variables y paths
    let dir = format!(
        "{}/{}",
        mconf::get("versions").get_string().as_str(),
        version.id
    );
    let dir = Path::new(&dir);
    if !dir.exists() {
        fs::create_dir_all(dir).unwrap();
    }
    let meta_file = format!("{}/{}", dir.display(), META_FILE);
    let meta_file = Path::new(&meta_file);
    let libs_path = format!("{}/{}", dir.display(), LIBS_DIR);
    let natives_path = format!("{}/{}", dir.display(), NATIVES_DIR);
    let game_path = format!("{}/{}", dir.display(), CLIENT_FILE);
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
    let java_home = get_compatible_java(
        mconf::get("java").get_string().as_str(),
        &version.java_version.clone(),
    );
    // crear archivo .info
    println!("Creating... Meta");
    let mut meta = HashMap::new();
    meta.insert("assets".to_string(), Types::String(version.assets.clone()));
    meta.insert("version".to_string(), Types::String(version.id.clone()));
    meta.insert(
        "main".to_string(),
        Types::String(version.main_class.clone()),
    );
    meta.insert("java".to_string(), Types::String(java_home));
    meta.insert("args".to_string(), Types::Vec(vec![]));
    meta.insert("jvm".to_string(), Types::Vec(vec![]));
    create_meta(&meta_file, meta);
    // descargar librerias
    println!("Downloading... Libs");
    let libs = &version.libraries.clone();
    mc::utils::libs_utils::filter_libs(
        libs_path.to_str().unwrap(),
        natives_path.to_str().unwrap(),
        libs,
        HandleEvent::new(move |_| {
            //println!("LIBS[{}]", e.percent());
        }),
    )
    .expect("Error downloading libs")
    .start();
    // descargar el jar del juego
    println!("Downloading... Game");
    mc::download(game_path.to_str().unwrap(), &version);
    if !Path::new(mconf::get("logger").get_string().as_str()).exists() {
        if let Some(logg) = &version.clone().logging {
            mc::get_config_logger(logg, mconf::get("logger").get_string().as_str());
        }
    }
    // si se piden, descargar assets
    if assets {
        println!("Downloading... Assets");
        mc::utils::assets_utils::download_all(
            &mconf::get("assets").get_string().as_str(),
            &version,
            HandleEvent::new(move |_| {}),
        );
    }
}

/// Crea un archivo establecido en `dir` y escribe el contenido de `map`
fn create_meta(dir: &Path, map: HashMap<String, Types>) {
    let deserialize = config::deserialize(map);
    fs::write(dir, deserialize).unwrap();
}
/// Lista todas las versions
pub fn list() -> HashMap<String, Version> {
    // define el dir e inicializa el map
    let dir = mconf::get("versions").get_string();
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
    let meta = config::serialize(content);
    let version_name = meta.get("version").unwrap().get_string();
    let args = meta.get("args").unwrap().get_vec();
    let args = args.iter().map(|v| v.get_string()).collect();

    let jvm = meta.get("jvm").unwrap().get_vec();
    let jvm = jvm.iter().map(|v| v.get_string()).collect();

    // crea la version y la devuelve
    let version = Version {
        pwd: dir.to_str().unwrap().to_string(),
        version: version_name.clone(),
        assets: meta.get("assets").unwrap().get_string(),
        main: meta.get("main").unwrap().get_string(),
        java: meta.get("java").unwrap().get_string(),
        jvm,
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
    let path = format!("{}/{}", mconf::get("versions").get_string(), version);
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

pub enum RunType {
    WORLD(String),
    SERVER(String),
    NORMAL,
}

pub struct Command {
    pub resources: CommandResourcesConfig,
    pub java_home: String,
    pub game_dir: String,
    pub assets: CommandAssetsConfig,
    pub user: CommandUserConfig,
    pub version: CommandVersionConfig,
    pub ram: CommandRamConfig,
    pub event: fn(String),
    pub err_event: fn(String),
    pub args: Vec<String>,
    pub jvm: Vec<String>,
}
pub struct CommandResourcesConfig {
    pub libraries: String,
    pub jar_file: String,
    pub bin: String,
    pub logger: String,
}
pub struct CommandRamConfig {
    pub xmx: i32,
    pub xms: i32,
}
pub struct CommandAssetsConfig {
    pub assets_dir: String,
    pub assets_index: String,
}
pub struct CommandVersionConfig {
    pub version_id: String,
    pub version_type: String,
    pub main_class: String,
}
pub struct CommandUserConfig {
    pub user_type: String,
    pub client_id: String,
    pub uuid: String,
    pub xuid: String,
    pub access_token: String,
    pub user_name: String,
}
impl Command {
    pub fn run(&self, run_type: RunType) {
        //println!("{}", self.java_home.clone());

        match OperatingSystem::detect() {
            OperatingSystem::Linux => {
                let chmod = std::process::Command::new("/bin/chmod")
                    .arg("+x")
                    .arg(self.java_home.clone().as_str())
                    .spawn();

                chmod.unwrap().wait().unwrap();
            }
            _ => {}
        }

        let mut binding = std::process::Command::new(self.java_home.as_str());
        let mut java = binding;
        //.arg(format!("-Djna.tmpdir={}", self.resources.bin))
        //.arg(format!("-Dio.netty.native.workdir={}", self.resources.bin))
        //.arg(format!("-Djava.library.path={}", self.resources.bin))
        //.arg(format!(
        //    "-Dlog4j.configurationFile={}",
        //    self.resources.logger
        //));

        for jvm in &self.jvm {
            java.arg(jvm);
        }

        let child = java
            .arg("-cp")
            .arg(format!(
                "{}{}{}/*",
                self.resources.jar_file,
                (match OperatingSystem::detect() {
                    OperatingSystem::Linux => ":",
                    _ => ";",
                }),
                self.resources.libraries
            ))
            .arg(self.version.main_class.as_str())
            .arg("--version")
            .arg(self.version.version_id.as_str())
            .arg("--versionType")
            .arg(self.version.version_type.as_str())
            .arg("--accessToken")
            .arg(self.user.access_token.as_str())
            .arg("--uuid")
            .arg(self.user.uuid.as_str())
            .arg("--xuid")
            .arg(self.user.xuid.as_str())
            .arg("--clientId")
            .arg(self.user.client_id.as_str())
            .arg("--username")
            .arg(self.user.user_name.as_str())
            .arg("--userType")
            .arg(self.user.user_type.as_str())
            .arg("--assetIndex")
            .arg(self.assets.assets_index.as_str())
            .arg("--assetsDir")
            .arg(self.assets.assets_dir.as_str())
            .arg("--gameDir")
            .arg(self.game_dir.as_str())
            .arg(match &run_type {
                RunType::WORLD(_) => "--quickPlaySingleplayer",
                RunType::SERVER(_) => "--quickPlayMultiplayer",
                RunType::NORMAL => "",
            })
            .arg(match run_type {
                RunType::WORLD(name) => name,
                RunType::SERVER(ip) => ip,
                RunType::NORMAL => { "" }.parse().unwrap(),
            });

        for arg in &self.args {
            child.arg(arg);
        }

        let mut child = child
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        //println!("run");
        // Obtener el stdout del proceso hijo
        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let stderr = child.stderr.take().expect("Failed to capture stderr");

        // Leer la salida del proceso hijo de manera asíncrona
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            println!("{}", line.unwrap());
            std::io::stdout().flush().unwrap();
        }

        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            println!("{}", line.unwrap());
            std::io::stdout().flush().unwrap();
        }

        // Esperar a que el proceso hijo termine
        child.wait().unwrap();
    }
}
