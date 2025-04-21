use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    let dir = ProjectDirs::from("moe", "msg", "Hello Work")
        .unwrap()
        .config_dir()
        .to_owned();
    if !dir.exists() {
        fs::create_dir(&dir).expect("Failed to create config directory");
    }
    dir
}
