use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Select};

use crate::mvers;

pub fn run() {
    println!("Running interactive mode...");
    welcome();
    prompt_user_action();
    todo!("Implement the rest of the interactive mode");
}
fn prompt_user_action() {
    open_select(
        style("What do you want to do?")
            .bold()
            .bright()
            .color256(208)
            .to_string()
            .as_str(),
        vec![
            "Show versions",
            "Show downloaded versions",
            "Download a version",
            "Delete a version",
            "Run game",
            style("Exit").bold().bright().red().to_string().as_str(),
        ],
    );
}
fn welcome() {
    println!(
        "{} {} {}",
        style("Welcome to the").bold(),
        style("interactive").bold().green(),
        style("mode").bold(),
    );
}
fn open_select(prompt: &str, options: Vec<&str>) -> String {
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&options)
        .interact()
        .unwrap();
    options[selection].to_string().clone()
}
