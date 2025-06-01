use std::path::Path;
use std::{env, fs};

use anyhow::Ok;
use anyhow::Result;
use log::trace;

mod cline;
#[cfg(feature = "interactive")]
mod interactive;
mod mconf;
mod errors;
mod mvers;
#[macro_use]

mod mtmp;

fn main() -> Result<()> {
    // initialize env_logger
    env_logger::init();
    init();
    #[cfg(feature = "interactive")]
    if cfg!(feature = "interactive") && env::args().len() == 1 {
        trace!("RUNNING INTERACTIVE");
        interactive::run()?;
        return Ok(());
    }
    trace!("NORMAL CLIENT RUN");
    cline::run()?;
    Ok(())
}

fn init() {
    trace!("CALL TO INIT CHECK");
    let user_conf_path = Path::new("mcwr.conf");
    let default_config = include_str!("mcwr.default.conf");
    if !user_conf_path.exists() {
        trace!("CONFING FILE NOT EXIST, WRITING DEFAULT");
        // escribimos el valor por defecto
        fs::write(user_conf_path, default_config).unwrap();
    }

    // establecemos los paths
    let versions_path = mconf::get::<String>("versions");
    let assets_path = mconf::get::<String>("resources");
    let versions_path = Path::new(&versions_path);
    let assets_path = Path::new(&assets_path);

    // si no existen los creamos
    if !versions_path.exists() {
        trace!("VERSIONS DIR NOT EXISTS, WRITING DEFAULT");
        fs::create_dir(versions_path).unwrap();
    }
    if !assets_path.exists() {
        trace!("ASSETS DIR NOT EXISTS, WRITING DEFAULT");
        fs::create_dir(assets_path).unwrap();
    }
}
