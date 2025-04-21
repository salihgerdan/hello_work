#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod db;
mod projects;

use std::time::{Duration, SystemTime};

use eframe::egui::{self, FontId, RichText, Ui};
use projects::Projects;
use rusqlite::Connection;

fn main() -> eframe::Result {
    //env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native(
        "Hello Work",
        options,
        Box::new(|cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<Pomo>::default())
        }),
    )
}

struct Pomo {
    session_length: u64,
    session_start: Option<SystemTime>,
    db: Connection,
    projects: Projects,
}

impl Pomo {
    fn is_running(&self) -> bool {
        self.session_start.is_some()
    }
    fn init_session(&mut self) {
        self.session_start = Some(SystemTime::now())
    }
    fn cancel_session(&mut self) {
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
    fn check_finished(&mut self) {
        self.time_elapsed().map(|elapsed| {
            if elapsed.as_secs() >= self.session_length {
                self.finish_session();
            }
        });
    }
    fn time_elapsed(&self) -> Option<Duration> {
        self.session_start.and_then(|s| s.elapsed().ok())
    }
    fn countdown_string(&self) -> String {
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
            session_length: 10,
            //session_length: 25 * 60,
            projects: Projects::new(&conn),
            db: conn,
        };
        pomo
    }
}

fn mini_ui(pomo: &mut Pomo, ui: &mut Ui) {
    ui.heading(format!(
        "{}",
        pomo.projects
            .get_active()
            .map(|x| x.name.as_str())
            .unwrap_or("Hello Work")
    ));
    ui.label(RichText::new(pomo.countdown_string()).font(FontId::proportional(40.0)));
}

fn main_ui(pomo: &mut Pomo, ui: &mut Ui) {
    ui.heading("Hello Work");
    let button = ui.button(if pomo.is_running() { "Cancel" } else { "Start" });
    if button.clicked() {
        if !pomo.is_running() {
            pomo.init_session()
        } else {
            pomo.cancel_session();
        }
    }
    ui.label(RichText::new(pomo.countdown_string()).font(FontId::proportional(45.0)));

    let selected_id = pomo.projects.get_active().map(|x| x.id);
    let mut clicked_proj_id = None;
    for proj in pomo.projects.get() {
        let proj_radio = ui.radio(
            selected_id.map(|id| proj.id == id).unwrap_or(false),
            &proj.name,
        );
        if proj_radio.clicked() {
            clicked_proj_id = Some(proj.id);
        }
    }
    if let Some(id) = clicked_proj_id {
        pomo.projects.set_active(id);
    }
}

impl eframe::App for Pomo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.available_width() > 200.0 && ui.available_height() > 200.0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::Normal,
                ));
                main_ui(self, ui);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
                mini_ui(self, ui);
            }
        });

        // repaint once the timer ticks to a whole second
        self.time_elapsed().map(|x| {
            ctx.request_repaint_after(Duration::from_millis(1000 - x.subsec_millis() as u64));
        });
        self.check_finished();
    }
}
