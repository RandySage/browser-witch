use eframe::egui;
use serde::Deserialize;
use std::{env, fs, process};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
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
    buttons: Vec<String>,
    clicks: Vec<i32>,
}

fn get_config() -> Config {
    // Read the .toml file into a string
    let home: PathBuf = env::home_dir().unwrap();
    //assert_eq!(home.is_some(), true);

    let file_path = Path::new(home.as_path()).join(".local/share/browser-witch/config.toml");
    println!("Path: {}", file_path.as_path().display());
    let toml_content = fs::read_to_string(file_path).expect("Failed to read file");

    // Deserialize the string into the Config struct
    let config: Config = toml::from_str(&toml_content).expect("Failed to parse TOML");

    return config;
}

fn get_stdin() -> std::string::String {

    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {  // Expecting exactly one input
        // Process the first argument after the program name
        let input = &args[1];
        println!("Received link, {}", input);
        return input.to_string();
    } else {
        println!("Error:2~No arguments provided.\nUsage:\n\tbrowser_witch <url>");
        process::exit(1);
    }
    return String::from("");
}


impl AppData {
    fn from_config(config: Config, command_input: &str) -> Self {
        println!("TODO: implement command_input handling for '{}'", command_input);
        let mut sort_integers: Vec<i32> = Vec::new();
        for entry in config.entries.iter() {
            sort_integers.push(entry.sort);
        }
        sort_integers.sort();
        let pre_dedup_length = sort_integers.len();
        let mut dedup_integers: Vec<i32> = sort_integers.clone();
        dedup_integers.dedup();
        if sort_integers.len() != dedup_integers.len() {
            println!("Error: a sort index is repeated: {:#?}", sort_integers);
            println!("Aborting");
            process::exit(1);
        }
        let mut buttons_from_config: Vec<String> = vec!["".to_string(); sort_integers.len()];
        let clicks_from_config: Vec<i32> = vec![0; sort_integers.len()];
        for entry in config.entries.iter() {
            for (index, sort) in sort_integers.iter().enumerate() {
                if *sort == entry.sort {
                    buttons_from_config[index] = entry.name.clone();
                }
            }
        }
        Self {
            buttons: buttons_from_config,
            clicks: clicks_from_config,
            command: command_input,
        }
    }
}

impl eframe::App for AppData {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for Ctrl+Q
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Q)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Browser Witch");

            ui.add_space(20.0);

            for index in 0..self.clicks.len() {
                if ui.button(self.buttons[index].clone()).clicked() {
                    self.clicks[index] += 1;
                    println!("{} was clicked! Total clicks: {}", self.buttons[index], self.clicks[index]);
                }
                // TODO: print comment
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let command_str: String = get_stdin();

    let config = get_config();
    // Print the loaded configuration
    println!("{:#?}", config);
    let config_app_data = AppData::from_config(config, &command_str);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([320.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Browser Witch",
        options,
        Box::new(|_context| {
            //egui_extras::install_image_loaders(&context.egui_ctx);
            Ok(Box::new(
                config_app_data
            ))
        }),
    )
}
