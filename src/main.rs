use std::fs;
use std::path::Path;
use mclr::deserialize::json_version;
use mclr::deserialize::json_version::JsonVersion;
use mclr::utils::manifest::manifest;

mod config;
#[cfg(test)]
mod tests;
mod mvers;
mod mconf;
mod cline;

fn main() {
    init();
    cline::run();
}

fn init() {
    // establecemos los paths
    let versions_path = Path::new("versions");
    let assets_path = Path::new("assets");
    let workdir_path = Path::new("workdir");
    let user_conf_path = Path::new("user.conf");

    // valor por defecto de la configuracion
    let default_config = r#"username:imbecil
    xmx:4
    xms:2
    pwd:workdir
    assets:assets
    java:java
    versions:versions
    tmp:.client.json.tmp
    logger:logger.config.xml"#;

    // si no existen los creamos
    if !versions_path.exists() { fs::create_dir(versions_path).unwrap(); }
    if !assets_path.exists() { fs::create_dir(assets_path).unwrap(); }
    if !workdir_path.exists() { fs::create_dir(workdir_path).unwrap(); }

    if !user_conf_path.exists() {
        // escribimos el valor por defecto
        fs::write(user_conf_path, default_config).unwrap();
    }
}
fn manifest_get(version: &str) -> JsonVersion {
    let manifest = manifest();
    manifest.get(version).unwrap().save_and_load(mconf::get("tmp").as_str())
}