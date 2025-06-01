use crate::{mconf, mvers, temp};
use clap::{Parser, Subcommand};
use flate2::read::GzEncoder;
use flate2::Compression;
use mcd::api::ApiClientUtil;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::path::Path;
use std::process::exit;
use tar::Builder;
use anyhow::{Ok, Result};

#[derive(Parser, Debug)]
#[command(author = "kristian/k3nder", version = "0.3.0", about)]
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

pub fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Download {
            version,
            run,
            silent,
            no_assets,
        } => {
            let apic = ApiClientUtil::new(&mconf::get::<String>("manifest"))?;

            let client = if version.starts_with("./") {
                apic.load(version.as_str(), temp!("mcwr-client.tmp"))?
            } else {
                apic.fetch(&version, temp!("mcwr-client.tmp"))?
            };
            mvers::download(&client, !no_assets)?;
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
            vers.run(std, std)?;
        }
        Commands::Ls { short } => {
            let versions = mvers::list()?;
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
        Commands::Find { version } => {
            if version.eq("release") {
                let release = mvers::manifest_latest_release();
                println!("{}", release);
                return Ok(());
            } else if version.eq("snapshot") {
                let snapshot = mvers::manifest_latest_snapshot();
                println!("{}", snapshot);
                return Ok(());
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
                mconf::get_or("export_path", String::from("exports"));
            if !Path::new(&path).exists() {
                fs::create_dir(&path).expect("Cannot create export path");
            }
            let file = format!("{}/{}.tar.gz", path, version);
            let versions_path = format!("{}/{}", mconf::get::<&str>("versions"), version);
            let file = File::create(file)?;
            let enc = GzEncoder::new(file, Compression::best());
            let mut tar = Builder::new(enc);
            tar.append_dir_all(&version, &versions_path)?;
            tar.finish()?
        }
    }

    Ok(())
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
