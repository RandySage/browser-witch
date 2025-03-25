use clap::Parser;
use directories::ProjectDirs;
use eframe::egui;
use serde::Deserialize;
use std::{fs, process};
use std::process::Command;

const COMMVENT_ORG_QUALIFIER: &str = "org.commvent";
const COMMVENT: &str = "CommVent";
const BROWSER_WITCH: &str = "Browser Witch";

#[derive(Clone, Debug, Deserialize)]
struct ConfigEntry {
    name: String,
    command: String,
    sort: i32,
}

#[derive(Debug, Deserialize)]
struct Config {
    entries: Vec<ConfigEntry>,
}

struct AppData {
    config_items: Vec<ConfigEntry>,
    url: String,
}

// Define your CLI structure
#[derive(Parser)]
#[command(name = "browser_witch")]
#[command(author = "Randy")]
#[command(version = "0.1.0")]
#[command(about = "Launch a URL with a selectable browser", long_about = None)]
struct Cli {
    // Define a positional argument
    #[arg(help = "URL to open")]
    url: String,

    // Define a flag
    #[arg(short, long, help = "Enable verbose mode")]
    verbose: bool,
}

fn get_config() -> Config {
    // Read the .toml file into a string
    if let Some(proj_dirs) = ProjectDirs::from(COMMVENT_ORG_QUALIFIER, COMMVENT,  BROWSER_WITCH) {
        let file_path = proj_dirs.config_dir().join("config.toml");
        println!("{} config path: {}", BROWSER_WITCH, file_path.as_path().display());
        let toml_content = fs::read_to_string(file_path).expect("Failed to read file");

        // Deserialize the string into the Config struct
        let config: Config = toml::from_str(&toml_content).expect("Failed to parse TOML");

        return config;
    } else {
        println!("Error: failed to construct config directory path; aborting");
        process::exit(1);
    }
}

impl AppData {
    fn from_config(config: Config, url: &str) -> Self {
        let mut sort_integers: Vec<i32> = Vec::new();
        for entry in config.entries.iter() {
            sort_integers.push(entry.sort);
        }
        sort_integers.sort();
        let mut dedup_integers: Vec<i32> = sort_integers.clone();
        dedup_integers.dedup();
        if sort_integers.len() != dedup_integers.len() {
            println!("Error: a sort index is repeated: {:#?}", sort_integers);
            println!("Aborting");
            process::exit(1);
        }
        let mut sort_config_items: Vec<ConfigEntry> = vec![ConfigEntry { name: "".to_string(), command: "".to_string(), sort: 0 }; sort_integers.len()];
        for entry in config.entries.iter() {
            for (index, sort) in sort_integers.iter().enumerate() {
                if *sort == entry.sort {
                    sort_config_items[index] = entry.clone();
                }
            }
        }
        Self {
            config_items: sort_config_items,
            url: url.to_string(),
        }
    }
}

fn open_url(cmd_string: &str, url: &str) -> Result<(), String> {
    // Split the string by whitespace
    let parts: Vec<&str> = cmd_string.split_whitespace().collect();

    if parts.is_empty() {
        return Err("Empty command string".to_string());
    }
    let mut replaced_parts: Vec<String> = Vec::with_capacity(parts.len());
    for part in parts {
        if part == "{url}" {
            replaced_parts.push(url.to_string());
        } else {
            replaced_parts.push(part.to_string());
        }
    }

    let command = &replaced_parts[0];
    let args = &replaced_parts[1..];

    Command::new(command)
        .args(args)
        .spawn()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    Ok(())
}


impl eframe::App for AppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for Ctrl+Q
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(BROWSER_WITCH);

            ui.add_space(20.0);

            for index in 0..self.config_items.len() {
                let text = egui::RichText::new(self.config_items[index].name.clone())
                    .size(36.0);
                if ui.button(text).clicked() {
                    println!("{} clicked", self.config_items[index].name);
                    let config_command = self.config_items[index].command.clone();
                    println!("{}, url={}", config_command, self.url);
                    match open_url(&config_command, &self.url) {
                        Ok(_) => {
                            println!("Command completed");
                            process::exit(0);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            process::exit(1);
                        }
                    }
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let cli = Cli::parse();

    let config = get_config();
    // Print the loaded configuration
    if cli.verbose {
        println!("{:#?}", config);
    }
    let config_app_data = AppData::from_config(config, &cli.url);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([320.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        BROWSER_WITCH,
        options,
        Box::new(|_context| {
            //egui_extras::install_image_loaders(&context.egui_ctx);
            Ok(Box::new(
                config_app_data
            ))
        }),
    )
}
