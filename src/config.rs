use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::color_schemes;

pub fn config_dir() -> PathBuf {
    let dir = ProjectDirs::from("moe", "msg", "Hello Work")
        .unwrap()
        .config_dir()
        .to_owned();
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create config directory");
    }
    dir
}

// be careful when changing field names
// we skip serializing on Option::None to avoid locking in default values
#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    // in minutes, as opposed to the seconds in the Pomo struct
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_scheme_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_end_audio: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_end_audio_volume: Option<f32>,
}

impl Config {
    pub fn read(file_path: &PathBuf) -> Self {
        let cfg = Self::parse_toml(file_path);

        cfg.unwrap_or_default()
    }
    fn parse_toml(file_path: &PathBuf) -> Option<Config> {
        match fs::read_to_string(file_path) {
            Ok(config_toml_str) => match toml::from_str(&config_toml_str) {
                Ok(config) => {
                    return Some(config);
                }
                Err(e) => {
                    dbg!("config.toml parse error");
                    dbg!(e);
                }
            },
            Err(_) => {
                // assume no config file exists yet, create it
                if let Err(e) = fs::File::create(file_path) {
                    dbg!("Cannot create config file");
                    dbg!(e);
                }
            }
        };
        None
    }
    pub fn write_config(&self, file_path: &PathBuf) {
        if let Ok(mut file) = fs::File::create(file_path) {
            let _ = file.write_all(toml::to_string(self).unwrap().as_bytes());
        }
    }
    pub fn get_color_scheme(&self) -> &color_schemes::ColorScheme {
        self.color_scheme_name
            .as_ref()
            .and_then(|color_scheme_name| {
                color_schemes::SCHEMES
                    .into_iter()
                    .find(|x| x.0 == color_scheme_name)
            })
            .map(|x| x.1)
            .unwrap_or(&color_schemes::CHAOS_THEORY)
    }
}
