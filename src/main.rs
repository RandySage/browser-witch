use clap::Parser;
use directories::ProjectDirs;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Label, Orientation};
use log::info;
use serde::Deserialize;
use std::process::Command;
use std::{fs, process};
use systemd_journal_logger::JournalLog;

const APP_ID: &str = "org.commvent.BrowserWitch";
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

#[derive(Parser)]
#[command(name = "browser_witch")]
#[command(author = "Randy")]
#[command(version = "0.1.0")]
#[command(about = "Launch a URL with a selectable browser", long_about = None)]
struct Cli {
    #[arg(help = "URL to open")]
    url: String,

    #[arg(short, long, help = "Enable verbose mode")]
    verbose: bool,
}

fn get_config() -> Config {
    if let Some(proj_dirs) = ProjectDirs::from(COMMVENT_ORG_QUALIFIER, COMMVENT, BROWSER_WITCH) {
        let file_path = proj_dirs.config_dir().join("config.toml");
        println!("{} config path: {}", BROWSER_WITCH, file_path.display());
        let toml_content = fs::read_to_string(file_path).expect("Failed to read file");
        toml::from_str(&toml_content).expect("Failed to parse TOML")
    } else {
        println!("Error: failed to construct config directory path; aborting");
        process::exit(1);
    }
}

fn sorted_entries(config: Config) -> Vec<ConfigEntry> {
    let mut entries = config.entries;
    entries.sort_by_key(|e| e.sort);

    let has_duplicates = entries.windows(2).any(|w| w[0].sort == w[1].sort);
    if has_duplicates {
        println!("Error: a sort index is repeated.");
        process::exit(1);
    }

    entries
}

fn open_url(cmd_string: &str, url: &str) -> Result<(), String> {
    let parts: Vec<&str> = cmd_string.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command string".to_string());
    }
    let replaced: Vec<String> = parts
        .iter()
        .map(|p| if *p == "{url}" { url.to_string() } else { p.to_string() })
        .collect();

    Command::new(&replaced[0])
        .args(&replaced[1..])
        .spawn()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    Ok(())
}

fn build_ui(app: &Application, entries: Vec<ConfigEntry>, url: String) {
    let css = CssProvider::new();
    css.load_from_data("button { font-size: 36px; }");
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to display"),
        &css,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title(BROWSER_WITCH)
        .default_width(320)
        .default_height(320)
        .build();

    let key_controller = gtk4::EventControllerKey::new();
    let window_ref = window.clone();
    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        if key == gdk::Key::q && modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
            window_ref.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    window.add_controller(key_controller);

    let vbox = GtkBox::new(Orientation::Vertical, 10);
    vbox.set_margin_top(10);
    vbox.set_margin_bottom(10);
    vbox.set_margin_start(10);
    vbox.set_margin_end(10);

    let heading = Label::new(Some(BROWSER_WITCH));
    heading.set_markup(&format!("<span weight='bold'>{}</span>", BROWSER_WITCH));
    vbox.append(&heading);

    for entry in entries {
        let button = Button::with_label(&entry.name);
        let url_clone = url.clone();
        button.connect_clicked(move |_| {
            info!(
                "browser-witch selected: {} command: {} url: {}",
                entry.name, entry.command, url_clone
            );
            match open_url(&entry.command, &url_clone) {
                Ok(_) => process::exit(0),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        });
        vbox.append(&button);
    }

    window.set_child(Some(&vbox));
    window.present();
}

fn main() {
    std::env::remove_var("XMODIFIERS");

    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let cli = Cli::parse();
    info!("browser-witch started with url: {}", cli.url);

    let config = get_config();
    if cli.verbose {
        println!("{:#?}", config);
    }

    let entries = sorted_entries(config);
    let url = cli.url;

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(move |app| {
        build_ui(app, entries.clone(), url.clone());
    });

    // Pass no args — clap already consumed them; GTK must not see them.
    process::exit(app.run_with_args::<String>(&[]).value());
}
