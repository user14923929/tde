use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub keybinds: Keybinds,
    #[serde(default)]
    pub statusbar: StatusbarConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub accent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    pub quit: String,
    pub focus_next: String,
    pub focus_prev: String,
    pub open_launcher: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub close_pane: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusbarConfig {
    pub show_clock: bool,
    pub show_cpu: bool,
    pub show_mem: bool,
    pub position: StatusbarPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatusbarPosition {
    Top,
    Bottom,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".into(),
            accent: "cyan".into(),
        }
    }
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            quit: "q".into(),
            focus_next: "Tab".into(),
            focus_prev: "BackTab".into(),
            open_launcher: "space".into(),
            split_horizontal: "h".into(),
            split_vertical: "v".into(),
            close_pane: "x".into(),
        }
    }
}

impl Default for StatusbarConfig {
    fn default() -> Self {
        Self {
            show_clock: true,
            show_cpu: true,
            show_mem: true,
            position: StatusbarPosition::Bottom,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let text = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config: {}", path.display()))?;
            let cfg: Config =
                toml::from_str(&text).with_context(|| "Failed to parse config TOML")?;
            Ok(cfg)
        } else {
            let default = Config::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, toml::to_string_pretty(&default)?)?;
            Ok(default)
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("tde")
        .join("config.toml")
}