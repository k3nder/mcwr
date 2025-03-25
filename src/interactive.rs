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
    let version_id = select_version(&term);
    print_system_message(format!("The {}, a good version", version_id).as_str());
    let assets = open_select(
        "Do you want to download the assets?",
        vec!["Yes (For optimal performance)", "No (For faster download)"],
    );
    print_system_message("Starting download in 3 seconds");
    counter_back(3);
    let version = manifest()
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
