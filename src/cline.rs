use std::io;
use std::io::Write;
use std::process::exit;
use clap::{Parser, Subcommand};
use mclr::deserialize::json_version;
use mclr::deserialize::json_version::JsonVersion;
use mclr::utils::{CounterEvent, HandleEvent};
use mclr::utils::manifest::manifest;
use crate::{mconf, mvers};

#[derive(Parser, Debug)]
#[command(author = "kristian/k3nder", version = "1.0", about)]
struct Args {
    #[command(subcommand)]
    command: Commands
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
        no_assets: bool
    },
    Run {
        #[arg()]
        version: String,
        #[arg(short = 'S')]
        silent: bool
    },
    Ls {
        #[arg(short = 'S')]
        short: bool
    },
    Remove {
        #[arg()]
        version: String,
        #[arg(short = 'C')]
        confirm: bool
    },
    Config {
        key: Option<String>,
        value: Option<String>
    }
}

pub fn run() {
    let args = Args::parse();

    match args.command {
        Commands::Download { version, run, silent, no_assets } => {
            let version = if version.starts_with("./") {
                json_version::load(version.as_str())
            } else {
                let ma = manifest();
                ma.get(version.as_str()).unwrap().save_and_load(mconf::get("tmp").as_str())
            };
            mvers::download(version, !no_assets);
        },
        Commands::Run { version, silent } => {
            let vers = mvers::get(version).expect("Version not found in MVERS");

            let std: fn(String) = if silent { |e| {} } else { |e| { println!("{}", e); } };

            vers.run(std, std, mconf::get("pwd"));
        },
        Commands::Ls { short } => {
            let versions = mvers::list();
            for (k,v) in versions.iter() {
               let message = if short { format!("{}", k) } else
               { format!("{} - {} - JAVA: {} - ASSETS: {} - MAIN: {}", k, "VANILLA", v.java, v.assets, v.main) };
                println!("{}", message);
            }
        },
        Commands::Remove { version, confirm } => {
           if !confirm {
               if !confirmation(format!("¿Quieres eliminar la version {}?", version).as_str()) {
                   exit(0);
               }
           }
           mvers::remove(version);
        },
        Commands::Config { key, value } => {
            if key.is_none() && value.is_none() {
                let config = mconf::config();
                for (k, v) in config.iter() {
                    println!("{} = {}", k, v);
                }
            }
            if key.is_some() && value.is_some() {
                mconf::set(key.clone().unwrap().as_str(), value.clone().unwrap());
            }
            if key.is_some() && value.is_none() {
                println!("{}", mconf::get(key.clone().unwrap().as_str()));
            }
        }
    }
}

fn confirmation(message: &str) -> bool {
    print!("{} (s/n): ", message);
    io::stdout().flush().unwrap();  // Asegura que el mensaje se imprima antes de leer

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