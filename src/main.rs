#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod audio;
mod color_schemes;
mod config;
mod db;
mod pomo;
mod projects;
mod stats;
mod util;

use chrono::{Datelike, Days, Utc};
use iced::{
    Center, Element, Padding, Size, Subscription, Task, Theme, keyboard,
    theme::{Custom, Palette},
    time,
    widget::{
        MouseArea, button, center, center_x, column, container, image, pick_list, right, row,
        scrollable, slider, text, text_input, tooltip,
    },
    window::{self, Level, Settings},
};
use pliced::Chart;
use plotters::{prelude::*, style::Color};
use std::iter;
use std::time::Duration;
use std::{env, sync::Arc};

const MAIN_W: f32 = 400.0;
const MAIN_H: f32 = 600.0;
const MINI_W: f32 = 110.0;
const MINI_H: f32 = 65.0;

const FONT_SANS: iced::Font = iced::Font::with_name("Lato");
static HELLO_WORK_ICON: &[u8] = include_bytes!("../img/hello_work_pixel.png");

static CONFIG_ICON: &[u8] = include_bytes!("../img/config.png");
static ADD_ICON: &[u8] = include_bytes!("../img/add.png");
static ARCHIVE_ICON: &[u8] = include_bytes!("../img/archive.png");
static OKAY_ICON: &[u8] = include_bytes!("../img/okay.png");

pub fn main() -> iced::Result {
    let icon = iced::window::icon::from_file_data(HELLO_WORK_ICON, None).ok();
    let app = iced::application(App::title, App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .window(Settings {
            icon: icon,
            ..Default::default()
        })
        .default_font(FONT_SANS)
        .window_size(Size::new(MAIN_W, MAIN_H));
    app.run_with(move || App::new())
}

#[derive(Default, Debug, Clone)]
enum Tab {
    #[default]
    Main,
    Projects,
    Stats,
    Settings,
}

struct App {
    // geometry: (iced::Size, iced::window::Position),
    mini_window: bool,
    current_tab: Tab,
    pomo: pomo::Pomo,
    theme: Theme,
}

impl Default for App {
    fn default() -> Self {
        let mut app = App {
            mini_window: false,
            current_tab: Tab::default(),
            pomo: pomo::Pomo::default(),
            theme: Theme::CatppuccinLatte,
        };
        //initialize theme here
        app.update_theme();
        app
    }
}

#[derive(Debug, Clone)]
enum Message {
    Ignore,
    Toggle,
    Tick,
    DragMove,
    MiniWindowToggle,
    ProjectSelected(usize),
    NewProject { parent: Option<usize> },
    EditProjectInitiate(usize),
    EditProjectFinish,
    EditProjectArchive,
    EditProjectNameInput(String),
    TabSelected(Tab),
    SessionLengthChanged(String),
    ThemeChanged(Option<String>),
    FilePickerWorkEndAudio,
    WorkEndAudioVolumeChanged(f32),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let commands = vec![
            iced::font::load(include_bytes!("../img/Lato-Regular.ttf")).map(|_| Message::Ignore),
        ];
        (Self::default(), Task::batch(commands))
    }
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Ignore => {}
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

                let is_wayland = match env::var("XDG_SESSION_TYPE") {
                    Ok(val) => val == "wayland",
                    Err(_) => false,
                };

                // If we're working under Wayland, create a new window with the
                // desired size, otherwise modify the current window.
                // This is because Wayland doesn't let us modify our current window
                // effectively, for some reason.

                if self.mini_window {
                    if is_wayland {
                        return window::get_latest().and_then(|window_id| -> Task<Message> {
                            let mut settings = Settings::default();
                            settings.size = Size::new(MINI_W, MINI_H);
                            settings.decorations = false;
                            settings.level = Level::AlwaysOnTop;
                            window::close(window_id).chain(window::open(settings).1.discard())
                        });
                    } else {
                        return window::get_latest().and_then(|window_id| -> Task<Message> {
                            window::set_level::<Message>(window_id, window::Level::AlwaysOnTop)
                                .chain(window::toggle_decorations(window_id))
                                .chain(window::resize(window_id, Size::new(MINI_W, MINI_H)))
                            // the order matters, first toggle decorations then resize
                            // to avoid ending up with a larger than intended window,
                            // as Windows compensates for the lost decoration space by growing the inner size
                        });
                    }
                } else {
                    if is_wayland {
                        self.current_tab = Tab::Main; // Always switch to main tab, stats tab crashes when coming from mini
                        return window::get_latest().and_then(|window_id| -> Task<Message> {
                            window::close(window_id).chain({
                                let mut settings = Settings::default();
                                settings.size = Size::new(MAIN_W, MAIN_H);
                                window::open(settings).1.discard()
                            })
                        });
                    } else {
                        return window::get_latest().and_then(|window_id| -> Task<Message> {
                            window::set_level::<Message>(window_id, window::Level::Normal)
                                .chain(window::resize(window_id, Size::new(MAIN_W, MAIN_H)))
                                .chain(window::toggle_decorations(window_id))
                        });
                    }
                }
            }
            Message::ProjectSelected(id) => {
                self.pomo.projects.set_active(Some(id));
            }
            Message::NewProject { parent } => {
                self.pomo.projects.add(parent, &self.pomo.db);
            }
            Message::EditProjectInitiate(id) => {
                self.pomo.projects.initiate_edit(Some(id));
            }
            Message::EditProjectFinish => {
                self.pomo.projects.finish_edit(&self.pomo.db);
            }
            Message::EditProjectNameInput(name) => {
                self.pomo.projects.set_edited_name(name);
            }
            Message::EditProjectArchive => {
                self.pomo.projects.archive_edited_item(&self.pomo.db);
            }
            Message::TabSelected(tab) => {
                self.current_tab = tab;
            }
            Message::SessionLengthChanged(session_length) => {
                // session length is input as minutes in the interface
                if let Ok(new_in_min) = session_length.parse::<f64>() {
                    self.pomo.change_session_length(new_in_min);
                }
            }
            Message::ThemeChanged(color_scheme_name) => {
                self.pomo.change_color_scheme(color_scheme_name);
                self.update_theme();
            }
            Message::FilePickerWorkEndAudio => {
                // clears if already set
                if self.pomo.config.work_end_audio.is_some() {
                    self.pomo.change_work_end_audio(None);
                } else {
                    let file = rfd::FileDialog::new()
                        .add_filter("mp3", &["mp3"])
                        .pick_file();
                    self.pomo.change_work_end_audio(file);
                }
            }
            Message::WorkEndAudioVolumeChanged(volume) => {
                // the slider uses 0.0..=100.0 while the real volume goes up to 1.0
                self.pomo.change_work_end_audio_volume(Some(volume / 100.0));
                self.update_theme();
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
            .get_active_project()
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
            self.pomo
                .projects
                .get_all_tree_style()
                .into_iter()
                .map(|(depth, p)| {
                    let mut p = p.clone();
                    p.name = util::truncate_with_ellipsis(
                        (0..depth)
                            .map(|_| "  ")
                            .chain(iter::once("› "))
                            .collect::<String>()
                            + &p.name,
                        40,
                    );
                    p
                })
                .collect::<Vec<_>>(),
            self.pomo.projects.get_active_project(),
            |p| Message::ProjectSelected(p.id),
        );

        center(
            column![duration, toggle_button, project_picker]
                .align_x(Center)
                .spacing(20),
        )
        .into()
    }

    fn projects_tab_view(&self) -> Element<Message> {
        let projects_list = column(self.pomo.projects.get_all_tree_style().into_iter().map(
            |(depth, p)| {
                if self
                    .pomo
                    .projects
                    .get_edited_id()
                    .map_or(false, |edited_id| edited_id == p.id)
                {
                    row![
                        text(
                            (0..depth)
                                .map(|_| "  ")
                                .chain(iter::once("› "))
                                .collect::<String>()
                        ),
                        text_input(
                            "Project Name",
                            &self
                                .pomo
                                .projects
                                .get_edited()
                                .map(|p| p.name.as_str())
                                .unwrap()
                        )
                        .on_input(Message::EditProjectNameInput),
                        right(row![
                            button(
                                image(image::Handle::from_bytes(OKAY_ICON))
                                    .height(16)
                                    .width(16)
                            )
                            .on_press(Message::EditProjectFinish),
                            button(
                                image(image::Handle::from_bytes(ARCHIVE_ICON))
                                    .height(16)
                                    .width(16)
                            )
                            .on_press(Message::EditProjectArchive),
                        ])
                    ]
                    .height(32)
                    .into()
                } else {
                    row![
                        text(util::truncate_with_ellipsis(
                            (0..depth)
                                .map(|_| "  ")
                                .chain(iter::once("› "))
                                .collect::<String>()
                                + &p.name,
                            40
                        )),
                        right(if self.pomo.projects.get_edited_id().is_none() {
                            row![
                                text!("{ :<4}", (p.total_hours * 10.0).round() / 10.0),
                                button(
                                    image(image::Handle::from_bytes(CONFIG_ICON))
                                        .height(16)
                                        .width(16)
                                )
                                .on_press(Message::EditProjectInitiate(p.id)),
                                button(
                                    image(image::Handle::from_bytes(ADD_ICON))
                                        .height(16)
                                        .width(16)
                                )
                                .on_press(Message::NewProject { parent: Some(p.id) })
                            ]
                        } else {
                            row![]
                        })
                    ]
                    .height(32)
                    .into()
                }
            },
        ))
        .spacing(3)
        .max_width(500);

        let new_button = button("+ New").on_press(Message::NewProject { parent: None });

        scrollable(column![center_x(projects_list), center_x(new_button)].padding(20)).into()
    }

    fn stats_tab_view(&self) -> Element<Message> {
        Chart::from_program(self).into()
    }

    fn settings_tab_view(&self) -> Element<Message> {
        let color_scheme_picker = pick_list(
            color_schemes::SCHEMES
                .iter()
                .map(|x| x.0)
                .collect::<Vec<_>>(),
            self.pomo
                .config
                .color_scheme_name
                .as_ref()
                .map(|x| x.as_str()),
            |p: &str| Message::ThemeChanged(Some(p.to_string())),
        );
        center(scrollable(
            column![
                row![
                    text("Session Length: "),
                    text_input("", &(self.pomo.session_length / 60).to_string())
                        .width(70)
                        .on_input(Message::SessionLengthChanged)
                ],
                row![
                    text("Audio: "),
                    text(
                        self.pomo
                            .config
                            .work_end_audio
                            .as_ref()
                            .map(|x| x.file_name().unwrap_or_default().to_string_lossy())
                            .unwrap_or_default()
                    ),
                    tooltip(
                        button(if self.pomo.config.work_end_audio.is_none() {
                            "Pick"
                        } else {
                            "Clear"
                        })
                        .on_press(Message::FilePickerWorkEndAudio),
                        container("Only supports mp3, the format adored by the world")
                            .padding(10)
                            .style(container::rounded_box),
                        tooltip::Position::Bottom,
                    )
                ],
                row![
                    text("Volume: "),
                    slider(
                        0.0..=100.0,
                        self.pomo.config.work_end_audio_volume.unwrap_or(1.0) * 100.0,
                        Message::WorkEndAudioVolumeChanged
                    )
                    .width(130)
                ],
                row![text("Colors: "), color_scheme_picker]
            ]
            .spacing(10)
            .max_width(500)
            .padding(20),
        ))
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
        self.theme.clone()
    }

    fn update_theme(&mut self) {
        fn into_iced_color(c: &color_schemes::Color) -> iced::Color {
            iced::Color::from_rgb8(c.r, c.g, c.b)
        }
        let color_scheme = self.pomo.config.get_color_scheme();
        let iced_theme = Theme::Custom(Arc::new(Custom::new(
            "name".to_string(),
            Palette {
                background: into_iced_color(&color_scheme.bg_color),
                text: into_iced_color(&color_scheme.text_color),
                primary: into_iced_color(&color_scheme.main_color),
                success: into_iced_color(&color_scheme.sub_color),
                warning: into_iced_color(&color_scheme.error_color),
                danger: into_iced_color(&color_scheme.colorful_error_color),
            },
        )));
        self.theme = iced_theme
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

        let sub_c = &self.pomo.config.get_color_scheme().sub_color;
        let color = plotters::style::RGBColor(sub_c.r, sub_c.g, sub_c.b);

        let text_c = &self.pomo.config.get_color_scheme().text_color;
        let text_color = plotters::style::RGBColor(text_c.r, text_c.g, text_c.b);

        let style = ShapeStyle {
            color: color.into(),
            filled: true,
            stroke_width: 2,
        };

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

        chart
            .configure_mesh()
            .label_style(TextStyle::from(("sans-serif", 15).into_font()).color(&text_color))
            // take out the year display from the dates
            .x_label_formatter(&|x| format!("{}-{}", x.month(), x.day()))
            .draw()
            .unwrap();

        chart
            .draw_series(
                AreaSeries::new(
                    data.iter().map(|x| *x), // The data iter
                    0.0,                     // Baseline
                    &color.mix(0.2),         // Make the series opac
                )
                .border_style(style), // Make a brighter border
            )
            .unwrap();
    }
}
