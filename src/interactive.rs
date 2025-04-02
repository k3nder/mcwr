use std::{io::Write, thread, time::Duration};

use console::{style, Term};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Select};
use translateutil::translate;

use crate::{config::Types, mconf, mvers};

pub fn run() {
    let term = Term::stdout();
    term.clear_screen().unwrap();

    println!(translate!("info.initial"));
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
            Action::ViewMetadata => {
                view_metadata(&term);
            }
            Action::Exit => {
                println!(translate!("info.exit"));
                break;
            }
        }
    }
}

fn view_metadata(term: &Term) {
    print_system_message(translate!("meta.view.initial"));
    let version = select_downloaded_version(term);
    print_system_message(translate!("meta.view.message"));
    let version = mvers::get(version).unwrap();
    print_meta("java", version.java);
    print_meta("main", version.main);
    print_meta("version", version.version);
    print_meta_array("args", version.args);
    print_meta_array("jvm", version.jvm);
}
fn print_meta_array(name: &str, values: Vec<String>) {
    print_system_message(&format!(
        "{} {}",
        style(name).green().bold().bright(),
        translate!("words.is")
    ));
    for val in values {
        print_system_message(&format!("  - {}", style(val).red().bold().bright()));
    }
}
fn print_meta(name: &str, value: String) {
    print_system_message(&format!(
        "{} {} {}",
        style(name).green().bold().bright(),
        translate!("words.is"),
        style(value).red().bold().bright()
    ));
}
enum ConfigurationAction {
    GetConfigurations,
    SetConfigurations,
}
impl ConfigurationAction {
    fn prompt() -> ConfigurationAction {
        let select = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(system_message(translate!("config.choose.action.prompt")))
            .items(&[
                translate!("config.choose.action.get"),
                translate!("config.choose.action.set"),
            ])
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
    print_system_message(translate!("config.initial"));
    let action = ConfigurationAction::prompt();
    match action {
        ConfigurationAction::GetConfigurations => {
            print_system_message(translate!("config.get.initial"));
            let config = mconf::config();
            let rendereable_config = config.iter().map(|(key, _)| format!("{}", key));
            let select = FuzzySelect::with_theme(&ColorfulTheme::default())
                .items(&rendereable_config.collect::<Vec<String>>())
                .interact()
                .unwrap();
            let key = config.keys().nth(select).unwrap();
            let value = config.get(key).unwrap().get_string();
            print_system_message(&format!(
                "{} {} {}",
                style(key).green().bold().bright(),
                translate!("words.is"),
                style(value).red().bold().bright()
            ));
        }
        ConfigurationAction::SetConfigurations => {
            print_system_message(translate!("config.set.initial"));
            let config = mconf::config();
            let rendereable_config = config.iter().map(|(key, _)| format!("{}", key));
            let select = FuzzySelect::with_theme(&ColorfulTheme::default())
                .with_prompt(system_message(translate!("config.set.prompt")))
                .items(&rendereable_config.collect::<Vec<String>>())
                .interact()
                .unwrap();
            let key = config.keys().nth(select).unwrap();
            let value: String = Input::new()
                .with_prompt(system_message(translate!("config.set.ask.new")))
                .interact()
                .unwrap();
            mconf::set(key, Types::from_value(value).unwrap());
        }
    }
}
fn show_downloaded_versions(term: &Term) {
    print_system_message(translate!("ls.initial"));
    let versions = mvers::list();
    if versions.is_empty() {
        term.move_cursor_up(1).unwrap();
        term.clear_line().unwrap();
        print_system_message(translate!("ls.empty"));
    } else {
        versions.iter().for_each(|(name, _)| {
            println!("{}", name);
        });
    }
}
fn download_version(term: &Term) {
    print_system_message(translate!("dwld.initial"));
    let version_id = select_version(&term);
    print_system_message(
        format!(
            "{} {} {}",
            translate!("dwld.confirm.0"),
            version_id,
            translate!("dwld.confirm.1")
        )
        .as_str(),
    );
    let assets = open_select(
        translate!("dwld.prompt.assets.title"),
        vec![
            translate!("dwld.prompt.assets.yes"),
            translate!("dwld.prompt.assets.no"),
        ],
    );
    print_system_message(translate!("dwld.cooldown.message"));
    counter_back(3);
    let version = mclr::utils::manifest::manifest()
        .get(&version_id)
        .unwrap()
        .save_and_load(mconf::get("tmp").get_string().as_str());
    mvers::download(version.clone(), assets == 0);

    print_system_message(translate!("dwld.done"));
    let launch = open_select(
        translate!("dwld.prompt.launch.title"),
        vec![
            translate!("dwld.prompt.launch.yes"),
            translate!("dwld.prompt.launch.no"),
        ],
    );
    if launch == 0 {
        print_system_message(translate!("dwld.launch.initial"));
        let version = mvers::get(version_id).unwrap();
        version.run(
            |l| println!("{}", l),
            |e| println!("{}", e),
            mconf::get("pwd").get_string(),
        );
        print_system_message(translate!("info.finish"));
        std::process::exit(0);
    }
}
fn select_version(term: &Term) -> String {
    let versions = mvers::list_manifest();
    let selection = dialoguer::FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(system_message(translate!("select.version.prompt")))
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
        print_system_message(translate!("select.version.empty"));
        let confirm = Confirm::new()
            .with_prompt(system_message(translate!("select.version.ask.dwld")))
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
        .with_prompt(system_message(translate!("select.version.prompt")))
        .default(0)
        .items(versions.as_slice())
        .interact()
        .unwrap();
    versions[selection].clone()
}
fn delete_version(term: &Term) {
    print_system_message(translate!("delete.initial"));
    let version_id = select_downloaded_version(term);

    let confirmation = Confirm::new()
        .with_prompt(system_message(translate!("delete.confirm")))
        .interact()
        .unwrap();

    if confirmation {
        mvers::remove(version_id);
    } else {
        print_system_message(translate!("delete.abort"));
    }

    print_system_message(translate!("delete.success"));
}
fn run_game(term: &Term) {
    print_system_message(translate!("run.initial"));
    let version_id = select_downloaded_version(term);
    print_system_message(translate!("run.loading"));
    let version = mvers::get(version_id).unwrap();
    version.run(
        |l| println!("{}", l),
        |e| println!("{}", e),
        mconf::get("pwd").get_string(),
    );
    print_system_message(translate!("run.finish"));
    print_system_message(translate!("info.finish"));
    std::process::exit(0);
}

enum Action {
    ShowDownloadedVersions,
    DownloadVersion,
    Configurations,
    DeleteVersion,
    RunGame,
    ViewMetadata,
    Exit,
}
fn prompt_user_action() -> Action {
    let selection = open_select(
        system_message(translate!("options.title")).as_str(),
        vec![
            translate!("options.ls"),
            translate!("options.dwld"),
            translate!("options.config"),
            translate!("options.delete"),
            translate!("options.run"),
            translate!("options.view.meta"),
            style(translate!("options.exit"))
                .bold()
                .bright()
                .red()
                .to_string()
                .as_str(),
        ],
    );
    match selection {
        0 => Action::ShowDownloadedVersions,
        1 => Action::DownloadVersion,
        2 => Action::Configurations,
        3 => Action::DeleteVersion,
        4 => Action::RunGame,
        5 => Action::ViewMetadata,
        _ => Action::Exit,
    }
}
fn welcome() {
    println!(
        "{} {} {}",
        style(translate!("welcome.0")).bold(),
        style(translate!("welcome.1")).bold().green(),
        style(translate!("welcome.2")).bold(),
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
