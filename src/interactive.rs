use console::{style, Style, Term};
use dialoguer::{theme::ColorfulTheme, Select};

use crate::mvers;

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
                println!("Exiting interactive mode...");
                break;
            }
        }
    }
}

fn configurations(term: &Term) -> _ {
    todo!()
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
}
fn select_version(term: &Term) {
    let versions = mvers::list();
    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(system_message("Select a version"))
        .default(0)
        .interact()
        .unwrap();
}
fn delete_version(term: &Term) {
    println!("Deleting version...");
}
fn run_game(term: &Term) {
    println!("Running game...");
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
