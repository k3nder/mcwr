use crate::config::Types;
use crate::{mconf, mvers};
use clap::{Parser, Subcommand};
use flate2::read::GzEncoder;
use flate2::Compression;
use mclr::deserialize::json_version;
use mclr::utils::manifest::manifest;
use std::fmt::format;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tar::Builder;

#[derive(Parser, Debug)]
#[command(author = "kristian/k3nder", version = "0.2.3", about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Debug, Subcommand)]
enum Commands {
    Download {
        #[arg()]
        version: String,
        #[arg(short = 'R')]
        run: bool,
        #[arg(short = 'S')]
        silent: bool,
        #[arg(short = 'A')]
        no_assets: bool,
    },
    Run {
        #[arg()]
        version: String,
        #[arg(short = 'S')]
        silent: bool,
    },
    Ls {
        #[arg(short = 'S')]
        short: bool,
    },
    Remove {
        #[arg()]
        version: String,
        #[arg(short = 'C')]
        confirm: bool,
    },
    Config {
        key: Option<String>,
        value: Option<String>,
    },
    Find {
        #[arg()]
        version: String,
    },
    #[cfg(feature = "export")]
    Export {
        #[arg()]
        version: String,
    },
}

pub fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Download {
            version,
            run,
            silent,
            no_assets,
        } => {
            let version = if version.starts_with("./") {
                json_version::load(version.as_str())
            } else {
                let ma = manifest();
                ma.get(version.as_str())
                    .unwrap()
                    .save_and_load(mconf::get("tmp").get_string().as_str())
            };
            mvers::download(version, !no_assets);
        }
        Commands::Run { version, silent } => {
            let vers = mvers::get(version).expect("Version not found in MVERS");

            let std: fn(String) = if silent {
                |_| {}
            } else {
                |e| {
                    println!("{}", e);
                }
            };

            vers.run(std, std, mconf::get("pwd").get_string());
        }
        Commands::Ls { short } => {
            let versions = mvers::list();
            for (k, v) in versions.iter() {
                let message = if short {
                    format!("{}", k)
                } else {
                    format!(
                        "{} - {} - JAVA: {} - ASSETS: {} - MAIN: {}",
                        k, "VANILLA", v.java, v.assets, v.main
                    )
                };
                println!("{}", message);
            }
        }
        Commands::Remove { version, confirm } => {
            if !confirm {
                if !confirmation(format!("¿Quieres eliminar la version {}?", version).as_str()) {
                    exit(0);
                }
            }
            mvers::remove(version);
        }
        Commands::Config { key, value } => {
            if key.is_none() && value.is_none() {
                let config = mconf::config();
                for (k, v) in config.iter() {
                    println!("{} = {}", k, v.display());
                }
            }
            if key.is_some() && value.is_some() {
                let value = Types::from_value(value.clone().unwrap());
                mconf::set(key.clone().unwrap().as_str(), value.clone().unwrap());
            }
            if key.is_some() && value.is_none() {
                println!("{}", mconf::get(key.clone().unwrap().as_str()).get_string());
            }
        }
        Commands::Find { version } => {
            if version.eq("release") {
                let release = mvers::manifest_latest_release();
                println!("{}", release);
                return;
            } else if version.eq("snapshot") {
                let snapshot = mvers::manifest_latest_snapshot();
                println!("{}", snapshot);
                return;
            }
            let versions = mvers::list_manifest();
            versions
                .iter()
                .filter(|v| v.contains(&version))
                .for_each(|v| println!("{}", v));
        }
        #[cfg(feature = "export")]
        Commands::Export { version } => {
            let path =
                mconf::get_or("export_path", Types::String(String::from("exports"))).get_string();
            if !Path::new(&path).exists() {
                fs::create_dir(&path).expect("Cannot create export path");
            }
            let file = format!("{}/{}.tar.gz", path, version);
            let versions_path = format!("{}/{}", mconf::get("versions").get_string(), version);
            let file = File::create(file).expect("Cannot create output file");
            let enc = GzEncoder::new(file, Compression::best());
            let mut tar = Builder::new(enc);

            tar.append_dir_all(&version, &versions_path)
                .expect("Cannot find version");

            tar.finish().expect("Cannot create export");
        }
    }
}

fn confirmation(message: &str) -> bool {
    print!("{} (s/n): ", message);
    io::stdout().flush().unwrap(); // Asegura que el mensaje se imprima antes de leer

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Convertimos la entrada a minúsculas y removemos espacios y saltos de línea
    let input = input.trim().to_lowercase();

    if input == "s" || input == "si" {
        true
    } else {
        false
    }
}
