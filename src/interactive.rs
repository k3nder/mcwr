use std::{io::Write, thread, time::Duration};

use console::{style, Style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use mclr::utils::manifest;

use crate::{mconf, mvers};

pub fn run() {
    let term = Term::stdout();
    term.clear_screen().unwrap();

    println!("Running interactive mode...");
    welcome();
    loop {
        match prompt_user_action() {
            Action::ShowDownloadedVersions => {
                show_downloaded_versions(&term);
            }
            Action::DownloadVersion => {
                download_version(&term);
            }
            Action::Configurations => {
                configurations(&term);
            }
            Action::DeleteVersion => {
                delete_version(&term);
            }
            Action::RunGame => {
                run_game(&term);
            }
            Action::Exit => {
                println!("Exiting...");
                break;
            }
        }
    }
}
enum ConfigurationAction {
    GetConfigurations,
    SetConfigurations,
}
impl ConfigurationAction {
    fn prompt() -> ConfigurationAction {
        let select = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(system_message("Choose an action"))
            .items(&["Get Configurations", "Set Configurations (WARN)"])
            .interact()
            .unwrap();
        match select {
            0 => ConfigurationAction::GetConfigurations,
            1 => ConfigurationAction::SetConfigurations,
            _ => unreachable!(),
        }
    }
}
fn configurations(term: &Term) {
    print_system_message("Remember, the settings section is very useful, but don't touch anything you don't need to touch, have fun!");
    let action = ConfigurationAction::prompt();
    match action {
        ConfigurationAction::GetConfigurations => {
            print_system_message("All right, what confuguration to consult?");
            let config = mconf::config();
            let rendereable_config = config.iter().map(|(key, _)| format!("{}", key));
            let select = FuzzySelect::with_theme(&ColorfulTheme::default())
                .items(&rendereable_config.collect::<Vec<String>>())
                .interact()
                .unwrap();
            let key = config.keys().nth(select).unwrap();
            let value = config.get(key).unwrap();
            print_system_message(&format!(
                "{} is {}",
                style(key).green().bold().bright(),
                style(value).red().bold().bright()
            ));
        }
        ConfigurationAction::SetConfigurations => {
            print_system_message("Remember again, don't touch the wrong things.");
            let config = mconf::config();
            let rendereable_config = config.iter().map(|(key, _)| format!("{}", key));
            let select = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt(system_message("Select Configuration"))
                .items(&rendereable_config.collect::<Vec<String>>())
                .interact()
                .unwrap();
            let key = config.keys().nth(select).unwrap();
            let value: String = Input::new()
                .with_prompt(system_message("Enter new value"))
                .interact()
                .unwrap();
            mconf::set(key, value);
        }
    }
}
fn show_downloaded_versions(term: &Term) {
    print_system_message("Ok... Here are all the downloaded versions");
    let versions = mvers::list();
    if versions.is_empty() {
        term.move_cursor_up(1).unwrap();
        term.clear_line().unwrap();
        print_system_message("Oh... you don't have downloaded versions");
    } else {
        versions.iter().for_each(|(name, _)| {
            println!("{}", name);
        });
    }
}
fn download_version(term: &Term) {
    print_system_message("you want to start a new adventure, let's start");
    let version_id = select_version(&term);
    print_system_message(format!("The {}, a good version", version_id).as_str());
    let assets = open_select(
        "Do you want to download the assets?",
        vec!["Yes (For optimal performance)", "No (For faster download)"],
    );
    print_system_message("Starting download in 3 seconds");
    counter_back(3);
    let version = mclr::utils::manifest::manifest()
        .get(&version_id)
        .unwrap()
        .save_and_load(mconf::get("tmp").as_str());
    mvers::download(version.clone(), assets == 0);

    print_system_message("Everything is ready");
    let launch = open_select(
        "Finally, do you want to launch the game?",
        vec!["Yes", "No"],
    );
    if launch == 0 {
        print_system_message("Launching game... Good Luck!");
        let version = mvers::get(version_id).unwrap();
        version.run(
            |l| println!("{}", l),
            |e| println!("{}", e),
            mconf::get("pwd"),
        );
        print_system_message("BYE!!!");
        std::process::exit(0);
    }
}
fn select_version(term: &Term) -> String {
    let versions = mvers::list_manifest();
    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(system_message("Select a version"))
        .default(0)
        .items(versions.as_slice())
        .interact()
        .unwrap();
    versions[selection].clone()
}
fn select_downloaded_version(term: &Term) -> String {
    let versions = mvers::list()
        .iter()
        .map(|v| v.0.clone())
        .collect::<Vec<String>>();

    if versions.is_empty() {
        print_system_message("No versions found");
        let confirm = Confirm::new()
            .with_prompt(system_message("Do you want to download a new version?"))
            .interact()
            .unwrap();
        if !confirm {
            std::process::exit(0);
        } else {
            download_version(term);
            return select_downloaded_version(term);
        }
    }

    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(system_message("Select a version"))
        .default(0)
        .items(versions.as_slice())
        .interact()
        .unwrap();
    versions[selection].clone()
}
fn delete_version(term: &Term) {
    print_system_message("don't delete anything you don't want");
    let version_id = select_downloaded_version(term);

    let confirmation = Confirm::new()
        .with_prompt(system_message(
            "Are you sure you want to delete this version?",
        ))
        .interact()
        .unwrap();

    if confirmation {
        mvers::remove(version_id);
    } else {
        print_system_message("Aborting...");
    }

    print_system_message("Version deleted");
}
fn run_game(term: &Term) {
    print_system_message("Ready to play?");
    let version_id = select_downloaded_version(term);
    print_system_message("Loading game...");
    let version = mvers::get(version_id).unwrap();
    version.run(
        |l| println!("{}", l),
        |e| println!("{}", e),
        mconf::get("pwd"),
    );
    print_system_message("Game finished, BYE!");
    std::process::exit(0);
}

enum Action {
    ShowDownloadedVersions,
    DownloadVersion,
    Configurations,
    DeleteVersion,
    RunGame,
    Exit,
}
fn prompt_user_action() -> Action {
    let selection = open_select(
        system_message("What do you want to do?").as_str(),
        vec![
            "Show downloaded versions",
            "Download a version",
            "Configurations",
            "Delete a version",
            "Run game",
            style("Exit").bold().bright().red().to_string().as_str(),
        ],
    );
    match selection {
        0 => Action::ShowDownloadedVersions,
        1 => Action::DownloadVersion,
        2 => Action::Configurations,
        3 => Action::DeleteVersion,
        4 => Action::RunGame,
        _ => Action::Exit,
    }
}
fn welcome() {
    println!(
        "{} {} {}",
        style("Welcome to the").bold(),
        style("interactive").bold().green(),
        style("mode").bold(),
    );
}
fn open_select(prompt: &str, options: Vec<&str>) -> usize {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&options)
        .interact()
        .unwrap();
    selection
}
fn system_message(message: &str) -> String {
    format!("{}", style(message).bold().bright().color256(208))
}
fn print_system_message(message: &str) {
    println!("{}", system_message(message));
}
fn counter_back(seconds: u32) {
    print!("{}", seconds);
    std::io::stdout().flush().unwrap();
    for i in (0..seconds).rev() {
        for _ in (0..4).rev() {
            print!(".");
            std::io::stdout().flush().unwrap();
            thread::sleep(Duration::from_millis(250));
        }
        print!("{}", i);
        std::io::stdout().flush().unwrap();
    }
}
