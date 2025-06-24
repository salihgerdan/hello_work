use crate::{config, db, projects::Projects};
use rusqlite::Connection;
use std::time::{Duration, Instant, SystemTime};

pub struct Pomo {
    pub session_length: u64,
    pub session_start: Option<SystemTime>,
    db: Connection,
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
                project_id: self.projects.get_active().map(|x| x.id),
            },
        )
        .expect("Recording work session into DB failed");
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
}

impl Default for Pomo {
    fn default() -> Self {
        let conn = db::init_db(&config::config_dir().join("hellowork.db"));
        let pomo = Self {
            session_start: None,
            //session_length: 10,
            session_length: 25 * 60,
            projects: Projects::new(&conn),
            db: conn,
        };
        pomo
    }
}
