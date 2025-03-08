use eframe::egui;
use serde::Deserialize;
use std::{env, fs};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ConfigEntry {
    name: String,
    command: String,
    comment: String,
    integer: i32,
}

#[derive(Debug, Deserialize)]
struct Config {
    entries: Vec<ConfigEntry>,
}

struct MyApp {
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

impl Default for MyApp {
    fn default() -> Self {
        Self {
            buttons: vec![
                String::from("Button 1"),
                String::from("Button 2"),
            ],
            clicks: vec![0; 2],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Four Button Example");

            ui.add_space(20.0);

            for index in 0..self.clicks.len() {
                if ui.button(self.buttons[index].clone()).clicked() {
                    self.clicks[index] += 1;
                    println!("{} was clicked! Total clicks: {}", self.buttons[index], self.clicks[index]);
                }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let config = get_config();
    // Print the loaded configuration
    println!("{:#?}", config);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_always_on_top()
            .with_inner_size([320.0, 320.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Four Button App",
        options,
        Box::new(|_context| {
            //egui_extras::install_image_loaders(&context.egui_ctx);
            Ok(Box::new(
                MyApp::default()
            ))
        }),
    )
}
