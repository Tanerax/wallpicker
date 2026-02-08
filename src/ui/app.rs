use iced::event::Event;
use iced::keyboard::{self, Key, key::Named};
use iced::widget::image::Handle as IcedImageHandle;
use iced::widget::mouse_area;
use iced::widget::svg::{self, Svg};
use iced::widget::{Image, button, column, container, row, scrollable, text};
use iced::{Color, Element, Length, Size, Task, Theme, application, window};

use iced::widget::scrollable::{Direction, Scrollbar};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::config::Config;
use crate::image::{THUMB_SIZE, load_thumb};
use crate::scanner::scan_directories;
use crate::ui::icons::{DICE_SVG, WALLHAVEN_SVG};
use super::platform_specific_settings;

pub fn run(window_width: u32, window_height: u32) -> iced::Result {
    let w = window_width as f32;
    let h = window_height as f32;

    application("Wallpicker", update, view)
        .theme(|_| Theme::Dark)
        .antialiasing(true)
        .window(window::Settings {
            size: Size::new(w, h),
            resizable: true,
            decorations: false,
            platform_specific: platform_specific_settings("wallpicker-main"),
            ..Default::default()
        })
        .subscription(|_state| iced::event::listen().map(Message::EventOccurred))
        .run_with(move || {
            let mut s = WallPicker::default();
            let cfg = crate::config::load_or_create_config();
            s.config = cfg;
            s.window_width = window_width;
            s.window_height = window_height;
            (s, Task::perform(async {}, |_| Message::ScanDirectory))
        })
}

#[derive(Debug, Default)]
struct WallPicker {
    paths: Vec<PathBuf>,
    thumbs: HashMap<PathBuf, IcedImageHandle>,
    loading: HashSet<PathBuf>,
    selected: Option<PathBuf>,
    window_width: u32,
    window_height: u32,
    config: Config,
}

#[derive(Debug, Clone)]
enum Message {
    ScanDirectory,
    Scanned(Vec<PathBuf>),
    LoadedThumb(PathBuf, Option<IcedImageHandle>),
    SelectWallpaper(PathBuf),
    SetRandomWallpaper,
    SetWallhavenWallpaper,
    EventOccurred(Event),
    OpenPreview(PathBuf),
    PreviewClosed(PathBuf, bool),
    Close
}

fn update(state: &mut WallPicker, message: Message) -> Task<Message> {
    match message {
        Message::ScanDirectory => {
            let folders = state.config.folders.clone();

            return Task::perform(scan_directories(folders), Message::Scanned);
        }
        Message::Scanned(paths) => {
            state.paths = paths;

            let tasks: Vec<Task<Message>> = state
                .paths
                .iter()
                .filter(|p| !state.thumbs.contains_key(*p))
                .map(|p| {
                    let p = p.clone();
                    state.loading.insert(p.clone());
                    Task::perform(load_thumb(p.clone()), move |h| {
                        Message::LoadedThumb(p.clone(), h)
                    })
                })
                .collect();

            return Task::batch(tasks);
        }
        Message::LoadedThumb(path, handle_opt) => {
            if let Some(handle) = handle_opt {
                state.thumbs.insert(path.clone(), handle);
            }

            state.loading.remove(&path);
        }
        Message::SelectWallpaper(path) => {
            state.selected = Some(path.clone());

            let copy_to_tmp = state.config.copy_to_tmp;
            return Task::perform(crate::commands::set_wallpaper(path, copy_to_tmp), |_| Message::Close);
        }
        Message::OpenPreview(p) => {
            let p_for_process = p.clone();
            let p_for_cb = p.clone();
            return Task::perform(
                crate::commands::open_preview(p_for_process),
                move |deleted| Message::PreviewClosed(p_for_cb.clone(), deleted),
            );
        }
        Message::PreviewClosed(p, deleted) => {
            if deleted {
                state.paths.retain(|x| x != &p);
                state.thumbs.remove(&p);
                state.loading.remove(&p);

                if matches!(state.selected.as_ref(), Some(sel) if sel == &p) {
                    state.selected = None;
                }
            }
        }
        Message::SetWallhavenWallpaper => {
            let cfg = state.config.clone();
            return Task::perform(
                crate::commands::set_random_wallpaper_via_wallhaven(cfg),
                |_| Message::Close,
            );
        }
        Message::SetRandomWallpaper => {
            let cfg = state.config.clone();
            return Task::perform(
                crate::commands::set_random_wallpaper(cfg),
                |_| Message::Close,
            );
        }

        Message::Close => {
            std::process::exit(0);
        }

        Message::EventOccurred(event) => match event {
            Event::Window(window::Event::Resized(size)) => {
                state.window_width = size.width as u32;
                state.window_height = size.height as u32;
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if key == Key::Named(Named::Escape) {
                    std::process::exit(0);
                }
            }
            _ => {}
        }
    }
    Task::none()
}

fn view(state: &WallPicker) -> Element<'_, Message> {
    let cols = state.suggested_columns();
    let mut tiles: Vec<Element<Message>> = Vec::new();

    tiles.push(state.random_widget());
    tiles.push(state.wallhaven_widget());

    for p in state.paths.iter() {
        tiles.push(state.thumbnail_widget(p));
    }

    let mut rows_ui: Vec<Element<Message>> = Vec::new();
    let mut it = tiles.into_iter();
    let cols = cols.max(1);

    loop {
        let Some(first) = it.next() else { break };
        let mut r = row![];

        r = r.push(first);
        for _ in 1..cols {
            if let Some(elem) = it.next() {
                r = r.push(elem);
            } else {
                break;
            }
        }
        rows_ui.push(r.into());
    }

    let grid = column(rows_ui).spacing(1);
    let scroll = scrollable(container(grid).width(Length::Fill))
        .direction(Direction::Vertical(Scrollbar::default().width(0).scroller_width(0).margin(0)))
        .height(Length::Fill);

    container(scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding([10,0])
        .into()
}

impl WallPicker {
    fn suggested_columns(&self) -> usize {
        let total = self.window_width.max(1) as i32;
        let cols = (total / THUMB_SIZE as i32).max(1) as usize;
        cols
    }

    fn thumbnail_widget(&self, path: &PathBuf) -> Element<'_, Message> {
        let base: Element<Message> = if let Some(handle) = self.thumbs.get(path) {
            Image::new(handle.clone())
                .width(Length::Fixed(THUMB_SIZE as f32))
                .height(Length::Fixed(THUMB_SIZE as f32))
                .into()
        } else {
            container(text(" "))
                .width(Length::Fixed(THUMB_SIZE as f32))
                .height(Length::Fixed(THUMB_SIZE as f32))
                .into()
        };

        let tile = container(base)
            .padding(0)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed(THUMB_SIZE as f32));

        let btn = button(tile)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed((THUMB_SIZE / 2) as f32))
            .style(|_theme, _status| iced::widget::button::Style {
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                ..Default::default()
            })
            .on_press(Message::SelectWallpaper(path.clone()));

        let p = path.clone();

        mouse_area(btn)
            .on_right_press(Message::OpenPreview(p))
            .into()
    }

    fn random_widget(&self) -> Element<'_, Message> {
        let handle = svg::Handle::from_memory(DICE_SVG.as_bytes());
        let icon = Svg::new(handle)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed((THUMB_SIZE as f32) * 0.65));

        let base: Element<Message> = container(column![icon, text("Random")].spacing(6))
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed(THUMB_SIZE as f32))
            .into();

        let tile = container(base)
            .padding(0)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed(THUMB_SIZE as f32));

        let btn = button(tile)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed((THUMB_SIZE / 2) as f32))
            .style(|_theme, _status| iced::widget::button::Style {
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                ..Default::default()
            })
            .on_press(Message::SetRandomWallpaper);

        mouse_area(btn).into()
    }

    fn wallhaven_widget(&self) -> Element<'_, Message> {
        let handle = svg::Handle::from_memory(WALLHAVEN_SVG.as_bytes());
        let icon = Svg::new(handle)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed((THUMB_SIZE as f32) * 0.65));

        let base: Element<Message> = container(column![icon, text("Wallhaven"),].spacing(6))
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed(THUMB_SIZE as f32))
            .into();

        let tile = container(base)
            .padding(0)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed(THUMB_SIZE as f32));

        let btn = button(tile)
            .width(Length::Fixed(THUMB_SIZE as f32))
            .height(Length::Fixed((THUMB_SIZE / 2) as f32))
            .style(|_theme, _status| iced::widget::button::Style {
                text_color: Color::WHITE,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
                ..Default::default()
            })
            .on_press(Message::SetWallhavenWallpaper);

        mouse_area(btn).into()
    }
}