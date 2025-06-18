#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod db;
mod pomo;
mod projects;

use iced::Size;
use iced::alignment::Horizontal::Right;
use iced::widget::right;
use iced::window::Level;
use iced::window::Mode;
use iced::window::Settings;
use projects::Projects;
use rusqlite::Connection;

use iced::Task;
use iced::keyboard;
use iced::time;
use iced::widget::MouseArea;
use iced::widget::{button, center, column, row, text};
use iced::window::{self, Id};
use iced::{Center, Element, Subscription, Theme};
use std::time::{Duration, Instant, SystemTime};

pub fn main() -> iced::Result {
    iced::application(Stopwatch::default, Stopwatch::update, Stopwatch::view)
        .subscription(Stopwatch::subscription)
        .theme(Stopwatch::theme)
        .run()
}

#[derive(Default)]
struct Stopwatch {
    // geometry: (iced::Size, iced::window::Position),
    mini_window: bool,
    pomo: pomo::Pomo,
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Tick,
    DragMove,
    MiniWindowToggle,
}

impl Stopwatch {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle => {
                if self.pomo.is_running() {
                    self.pomo.cancel_session();
                } else {
                    self.pomo.init_session();
                }
            }
            Message::Tick => {}
            Message::DragMove => {
                return window::get_latest()
                    .and_then(|window_id: window::Id| window::drag::<Message>(window_id));
            }
            Message::MiniWindowToggle => {
                self.mini_window = !self.mini_window;
                if self.mini_window {
                    return window::get_latest().and_then(|window_id| -> Task<Message> {
                        let mut settings = Settings::default();
                        settings.size = Size::new(100.0, 80.0);
                        settings.decorations = false;
                        settings.level = Level::AlwaysOnTop;
                        window::close(window_id).chain(window::open(settings).1.discard())
                    });

                    /*return window::get_latest().and_then(|window_id| -> Task<Message> {
                        window::set_level::<Message>(window_id, window::Level::AlwaysOnTop)
                            .chain(window::resize(window_id, Size::new(100.0, 80.0)))
                            .chain(window::toggle_decorations(window_id))
                    });*/
                } else {
                    return window::get_latest().and_then(|window_id| -> Task<Message> {
                        window::close(window_id)
                            .chain(window::open(Settings::default()).1.discard())
                    });

                    /*return window::get_latest().and_then(|window_id| -> Task<Message> {
                        window::set_level::<Message>(window_id, window::Level::Normal)
                            .chain(window::resize(window_id, Size::new(800.0, 600.0)))
                            .chain(window::toggle_decorations(window_id))
                    });*/
                }
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = if self.pomo.is_running() {
            time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        };

        fn handle_hotkey(key: keyboard::Key, _modifiers: keyboard::Modifiers) -> Option<Message> {
            use keyboard::key;

            match key.as_ref() {
                keyboard::Key::Named(key::Named::Space) => Some(Message::Toggle),
                //keyboard::Key::Character("r") => Some(Message::Reset),
                _ => None,
            }
        }

        Subscription::batch(vec![tick, keyboard::on_key_press(handle_hotkey)])
    }

    fn view(&self) -> Element<Message> {
        let duration = text(self.pomo.countdown_string()).size(40);

        //let button = |label| button(text(label).align_x(Center)).padding(10).width(80);

        let toggle_button = {
            let label = if self.pomo.is_running() {
                "Stop"
            } else {
                "Start"
            };

            button(text(label).align_x(Center))
                .padding(10)
                .width(80)
                .on_press(Message::Toggle)
        };

        let mini_window_button = button("M").on_press(Message::MiniWindowToggle);

        let content = if self.mini_window {
            column![right(mini_window_button), center(column![duration])]
        } else {
            column![
                right(mini_window_button),
                center(column![duration, toggle_button].align_x(Center).spacing(20))
            ]
        };

        let mouse_area = MouseArea::new(content).on_press(Message::DragMove);

        mouse_area.into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

/*fn main() -> eframe::Result {
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
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<Pomo>::default())
        }),
    )
}

fn kitty(pomo: &mut Pomo, ui: &mut Ui) {
    if ui.ui_contains_pointer() {
        ui.image(egui::include_image!("../img/kitty-dance2.gif"));
    } else {
        ui.image(egui::include_image!("../img/kitty-dance2-s.gif"));
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
    ui.horizontal(|ui| {
        ui.label(RichText::new(pomo.countdown_string()).font(FontId::proportional(45.0)));
        kitty(pomo, ui);
    });
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

    let null_proj_radio = ui.radio(selected_id.is_none(), "<No Project>");
    if null_proj_radio.clicked() {
        pomo.projects.set_active(None);
    }

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
        pomo.projects.set_active(Some(id));
    }
    kitty(pomo, ui);
}

impl eframe::App for Pomo {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let window_move_response = ui.interact(
                ui.max_rect(),
                Id::new("window_move"),
                Sense::click_and_drag(),
            );
            if window_move_response.drag_started_by(PointerButton::Primary) {
                ctx.send_viewport_cmd(ViewportCommand::StartDrag);
            }

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
*/
