use crate::{
    config::{self, Config},
    db,
    projects::Projects,
};
use rusqlite::Connection;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

pub struct Pomo {
    pub session_length: u64,
    pub session_start: Option<SystemTime>,
    pub db: Connection,
    pub config_file_path: PathBuf,
    pub config: Config,
    pub projects: Projects,
}

impl Pomo {
    pub fn is_running(&self) -> bool {
        self.session_start.is_some()
    }
    pub fn init_session(&mut self) {
        self.session_start = Some(SystemTime::now())
    }
    pub fn cancel_session(&mut self) {
        self.session_start = None
    }
    fn finish_session(&mut self) {
        db::add_work_session(
            &self.db,
            &db::WorkSession {
                time_start: self
                    .session_start
                    .unwrap()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                duration: self.session_length,
                project_id: self.projects.get_active(),
            },
        )
        .expect("Recording work session into DB failed");
        self.projects.fetch(&self.db); // refresh total work durations per project
        self.session_start = None
    }
    pub fn check_finished(&mut self) {
        self.time_elapsed().map(|elapsed| {
            if elapsed.as_secs() >= self.session_length {
                self.finish_session();
            }
        });
    }
    pub fn time_elapsed(&self) -> Option<Duration> {
        self.session_start.and_then(|s| s.elapsed().ok())
    }
    pub fn countdown_string(&self) -> String {
        match self.time_elapsed() {
            Some(t) => {
                let secs = t.as_secs();
                let rem = self.session_length - secs;
                format!("{:02}:{:02}", rem / 60, rem % 60)
            }
            None => "--:--".to_owned(),
        }
    }
    pub fn change_session_length(&mut self, new_in_min: f64) {
        self.session_length = (new_in_min * 60.0) as u64;
        self.config.session_length = Some(new_in_min);
        self.config.write_config(&self.config_file_path);
    }
}

impl Default for Pomo {
    fn default() -> Self {
        let config_file_path = config::config_dir().join("config.toml");
        let config = config::Config::read(&config_file_path);
        let conn = db::init_db(&config::config_dir().join("hellowork.db"));
        let pomo = Self {
            session_start: None,
            session_length: (config.session_length.unwrap_or(25.0) * 60.0) as u64,
            projects: Projects::new(&conn),
            config_file_path,
            config,
            db: conn,
        };
        pomo
    }
}
