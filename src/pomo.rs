use crate::{
    config::{self, Config},
    db,
    projects::Projects,
    todo_tasks::TodoTasks,
};
use rusqlite::Connection;
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

pub struct Pomo {
    pub session_length: u64,
    pub session_start: Option<SystemTime>,
    pub partial_start: Option<SystemTime>,
    pub db: Connection,
    pub config_file_path: PathBuf,
    pub config: Config,
    pub projects: Projects,
    pub tasks: TodoTasks,
}

impl Pomo {
    pub fn is_running(&self) -> bool {
        self.session_start.is_some()
    }
    pub fn init_session(&mut self) {
        self.session_start = Some(SystemTime::now());
        self.partial_start = Some(SystemTime::now());
    }
    pub fn cancel_session(&mut self) {
        self.save_partial_session_if_enabled();
        self.session_start = None;
        self.partial_start = None;
    }
    fn finish_session(&mut self) {
        let already_recorded = (self
            .partial_start
            .unwrap()
            .duration_since(self.session_start.unwrap()))
        .unwrap()
        .as_secs();

        let duration_secs = self.session_length - already_recorded;

        let start_unix = self
            .partial_start
            .unwrap()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        db::add_work_session(
            &self.db,
            &db::WorkSession {
                time_start: start_unix,
                duration: duration_secs,
                project_id: self.projects.get_active(),
            },
        )
        .expect("Recording work session into DB failed");
        println!("Session: {start_unix}, {duration_secs}");
        self.projects.fetch(&self.db); // refresh total work durations per project

        self.session_start = None;
        self.partial_start = None;
        crate::audio::play_audio(
            self.config.work_end_audio.clone(),
            self.config.work_end_audio_volume.unwrap_or(1.0),
        );
    }
    pub fn save_partial_session_if_enabled(&mut self) {
        if !self.config.get_save_partial_sessions() {
            return;
        }

        let duration_secs = self.partial_elapsed().unwrap().as_secs();

        // Only save if some meaningful progress was made
        if duration_secs > 30 {
            let partial_start_unix = self
                .partial_start
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            db::add_work_session(
                &self.db,
                &db::WorkSession {
                    time_start: partial_start_unix,
                    duration: duration_secs,
                    project_id: self.projects.get_active(),
                },
            )
            .expect("Recording partial work session into DB failed");
            println!("Partial session: {partial_start_unix}, {duration_secs}");
            self.partial_start = Some(SystemTime::now());

            self.projects.fetch(&self.db); // Refresh total project hours
        }
    }
    pub fn check_finished(&mut self) {
        self.session_elapsed().map(|elapsed| {
            if elapsed.as_secs() >= self.session_length {
                self.finish_session();
            }
        });
    }
    pub fn session_elapsed(&self) -> Option<Duration> {
        self.session_start.and_then(|s| s.elapsed().ok())
    }
    pub fn partial_elapsed(&self) -> Option<Duration> {
        self.partial_start.and_then(|s| s.elapsed().ok())
    }
    pub fn countdown_string(&self) -> String {
        match self.session_elapsed() {
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
    pub fn change_color_scheme(&mut self, color_scheme_name: Option<String>) {
        self.config.color_scheme_name = color_scheme_name;
        self.config.write_config(&self.config_file_path);
    }
    pub fn change_work_end_audio(&mut self, work_end_audio: Option<PathBuf>) {
        self.config.work_end_audio = work_end_audio;
        self.config.write_config(&self.config_file_path);
    }
    pub fn change_work_end_audio_volume(&mut self, work_end_audio_volume: Option<f32>) {
        self.config.work_end_audio_volume = work_end_audio_volume;
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
            partial_start: None,
            session_length: (config.session_length.unwrap_or(25.0) * 60.0) as u64,
            projects: Projects::new(&conn, config.get_last_active_project()),
            tasks: TodoTasks::new(&conn, None),
            config_file_path,
            config,
            db: conn,
        };
        pomo
    }
}
