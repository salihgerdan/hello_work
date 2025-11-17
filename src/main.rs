#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod config;
mod db;
mod pomo;
mod projects;
mod stats;

use chrono::Days;
use chrono::NaiveDate;
use chrono::Utc;
use iced::Length;
use iced::Padding;
use iced::Size;
use iced::application::Title;
use iced::widget::center_x;
use iced::widget::container;
use iced::widget::pick_list;
use iced::widget::right;
use iced::widget::scrollable;
use iced::widget::text_input;
use iced::window::Level;
use iced::window::Settings;

use iced::Task;
use iced::keyboard;
use iced::time;
use iced::widget::MouseArea;
use iced::widget::{button, center, column, row, text};
use iced::window;
use iced::{Center, Element, Subscription, Theme};
use pliced::{Chart, line_series, point_series};
use plotters::prelude::*;
use std::time::Duration;

use crate::db::Project;

const MAIN_W: f32 = 400.0;
const MAIN_H: f32 = 600.0;
const MINI_W: f32 = 110.0;
const MINI_H: f32 = 65.0;

pub fn main() -> iced::Result {
    iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size(Size::new(MAIN_W, MAIN_H))
        .run()
}

#[derive(Default, Debug, Clone)]
enum Tab {
    #[default]
    Main,
    Projects,
    Stats,
    Settings,
}

#[derive(Default)]
struct App {
    // geometry: (iced::Size, iced::window::Position),
    mini_window: bool,
    current_tab: Tab,
    pomo: pomo::Pomo,
}

#[derive(Debug, Clone)]
enum Message {
    Toggle,
    Tick,
    DragMove,
    MiniWindowToggle,
    ProjectSelected(Project),
    TabSelected(Tab),
    SessionLengthChanged(String),
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Toggle => {
                if self.pomo.is_running() {
                    self.pomo.cancel_session();
                } else {
                    self.pomo.init_session();
                }
            }
            Message::Tick => {
                self.pomo.check_finished();
            }
            Message::DragMove => {
                return window::get_latest()
                    .and_then(|window_id: window::Id| window::drag::<Message>(window_id));
            }
            Message::MiniWindowToggle => {
                self.mini_window = !self.mini_window;
                if self.mini_window {
                    return window::get_latest().and_then(|window_id| -> Task<Message> {
                        let mut settings = Settings::default();
                        settings.size = Size::new(MINI_W, MINI_H);
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
                    self.current_tab = Tab::Main; // Always switch to main tab, stats tab crashes when coming from mini
                    return window::get_latest().and_then(|window_id| -> Task<Message> {
                        window::close(window_id).chain({
                            let mut settings = Settings::default();
                            settings.size = Size::new(MAIN_W, MAIN_H);
                            window::open(settings).1.discard()
                        })
                    });

                    /*return window::get_latest().and_then(|window_id| -> Task<Message> {
                        window::set_level::<Message>(window_id, window::Level::Normal)
                            .chain(window::resize(window_id, Size::new(400.0, 200.0)))
                            .chain(window::toggle_decorations(window_id))
                    });*/
                }
            }
            Message::ProjectSelected(project) => {
                self.pomo.projects.set_active(Some(project.id));
            }
            Message::TabSelected(tab) => {
                self.current_tab = tab;
            }
            Message::SessionLengthChanged(session_length) => {
                // session length is input as minutes in the interface
                if let Ok(len) = session_length.parse::<u64>() {
                    self.pomo.session_length = len * 60;
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
                _ => None,
            }
        }

        Subscription::batch(vec![tick, keyboard::on_key_press(handle_hotkey)])
    }

    fn title(&self) -> String {
        self.pomo
            .projects
            .get_active()
            .map(|p| (p.name.clone() + " - Hello Work"))
            .unwrap_or("Hello Work".to_string())
    }

    fn mini_window_view(&self) -> Element<Message> {
        let duration = text(self.pomo.countdown_string()).size(40);
        column![center(duration)].into()
    }

    fn main_tab_view(&self) -> Element<Message> {
        let duration = text(self.pomo.countdown_string()).size(40);

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

        let project_picker = pick_list(
            self.pomo.projects.get(),
            self.pomo.projects.get_active(),
            |p| Message::ProjectSelected(p),
        );

        center(
            column![duration, toggle_button, project_picker]
                .align_x(Center)
                .spacing(20),
        )
        .into()
    }

    fn projects_tab_view(&self) -> Element<Message> {
        let projects_list = center(scrollable(column(
            self.pomo.projects.get().into_iter().map(|p| {
                row![button(text!("{}", p.name)), right(button("..."))]
                    .padding([4, 30])
                    .into()
            }),
        )));

        projects_list.into()
    }

    fn stats_tab_view(&self) -> Element<Message> {
        Chart::from_program(self).into()
    }

    fn settings_tab_view(&self) -> Element<Message> {
        center(scrollable(column![row![
            text_input("", &(self.pomo.session_length / 60).to_string())
                .on_input(Message::SessionLengthChanged)
        ]]))
        .into()
    }

    fn view(&self) -> Element<Message> {
        //let button = |label| button(text(label).align_x(Center)).padding(10).width(80);

        let tabs = row![
            button("Main").on_press(Message::TabSelected(Tab::Main)),
            button("Projects").on_press(Message::TabSelected(Tab::Projects)),
            button("Stats").on_press(Message::TabSelected(Tab::Stats)),
            button("Settings").on_press(Message::TabSelected(Tab::Settings))
        ]
        .spacing(10);

        let mini_window_button = button("m")
            .width(31)
            .height(31)
            .on_press(Message::MiniWindowToggle);

        let top_bar = row![
            center_x(tabs).padding(Padding::ZERO.left(31)), // padding equal to the mini_window button size
            mini_window_button
        ];

        let content = if self.mini_window {
            self.mini_window_view()
        } else {
            column![
                top_bar,
                match self.current_tab {
                    Tab::Main => self.main_tab_view(),
                    Tab::Projects => self.projects_tab_view(),
                    Tab::Stats => self.stats_tab_view(),
                    Tab::Settings => self.settings_tab_view(),
                }
            ]
            .into()
        };

        let mouse_area = MouseArea::new(content)
            .on_press(Message::DragMove)
            .on_double_click(Message::MiniWindowToggle);

        mouse_area.into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}

impl pliced::Program<Message> for App {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        chart: &mut plotters::prelude::ChartBuilder<pliced::IcedChartBackend<iced::Renderer>>,
        _theme: &iced::Theme,
        _bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) {
        let data = stats::last_week_chart(&self.pomo.db);

        let y_max = data
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .1;

        let mut chart = chart
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(
                Utc::now()
                    .date_naive()
                    .checked_sub_days(Days::new(6))
                    .unwrap()..Utc::now().date_naive(),
                0.0_f32..y_max,
            )
            .unwrap();

        chart.configure_mesh().draw().unwrap();

        chart
            .draw_series(
                AreaSeries::new(
                    data.iter().map(|x| *x), // The data iter
                    0.0,                     // Baseline
                    &RED.mix(0.2),           // Make the series opac
                )
                .border_style(&RED), // Make a brighter border
            )
            .unwrap();
    }
}
