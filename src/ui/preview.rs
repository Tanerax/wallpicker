use iced::event::Event;
use iced::{keyboard, window};
use iced::widget::{container, text, Image};
use iced::widget::image::Handle as IcedImageHandle;
use iced::{application, Element, Length, Size, Task, Theme, ContentFit};

use std::path::PathBuf;

use image::GenericImageView;

#[derive(Debug)]
struct PreviewApp {
    title: String,
    handle: Option<IcedImageHandle>,
    path: PathBuf,
}

#[derive(Debug, Clone)]
enum Message {
    EventOccurred(Event),
    LoadedPreview(Option<IcedImageHandle>),
}

const MAX_PREVIEW_W: u32 = 1920;

pub fn run(path: PathBuf) -> iced::Result {
    let title = String::from("Wallpicker - Preview");

    application(|s: &PreviewApp| s.title.clone(), update, view)
        .theme(|_| Theme::Dark)
        .antialiasing(true)
        .window(window::Settings {
            size: Size::new(1920.0, 1080.0),
            resizable: true,
            decorations: true,
            platform_specific: window::settings::PlatformSpecific {
                application_id: String::from("wallpicker-preview"),
                override_redirect: false
            },
            ..Default::default()
        })
        .subscription(|_state| iced::event::listen().map(Message::EventOccurred))
        .run_with(move || {
            let p = path.clone();
            (
                PreviewApp {
                    title: title.clone(),
                    handle: None,
                    path: path.clone(),
                },
                Task::perform(load_preview_handle(p), Message::LoadedPreview),
            )
        })
}

async fn load_preview_handle(path: PathBuf) -> Option<IcedImageHandle> {
    tokio::task::spawn_blocking(move || {
        let img = image::open(&path).ok()?;

        let (w, h) = img.dimensions();
        let scaled = if w > MAX_PREVIEW_W {
            let scale = MAX_PREVIEW_W as f32 / w as f32;
            let target_h = ((h as f32) * scale).round().max(1.0) as u32;

            img.resize(
                MAX_PREVIEW_W,
                target_h,
                image::imageops::FilterType::CatmullRom,
            )
        } else {
            img
        };

        let rgba = scaled.to_rgba8();
        let (w2, h2) = rgba.dimensions();
        Some(IcedImageHandle::from_rgba(w2, h2, rgba.into_raw()))
    })
    .await
    .unwrap_or(None)
}

fn update(state: &mut PreviewApp, message: Message) -> Task<Message> {
    match message {
        Message::LoadedPreview(handle) => {
            state.handle = handle;
        }
        Message::EventOccurred(event) => match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                if state.handle.is_some() {
                    match key {
                        keyboard::Key::Character(c) if c.eq_ignore_ascii_case("d") => {
                            let _ = std::fs::remove_file(&state.path);
                            std::process::exit(10);
                        }
                        keyboard::Key::Named(iced::keyboard::key::Named::Delete) => {
                            let _ = std::fs::remove_file(&state.path);
                            std::process::exit(10);
                        }
                        _ => {
                            std::process::exit(0);
                        }
                    }
                } else {
                    std::process::exit(0);
                }
            }
            Event::Mouse(iced::mouse::Event::ButtonPressed(_)) => {
                std::process::exit(0);
            }
            _ => {}
        },
    }
    Task::none()
}

fn view(state: &'_ PreviewApp) -> Element<'_, Message> {
    let content: Element<Message> = if let Some(handle) = &state.handle {
        Image::new(handle.clone())
            .content_fit(ContentFit::Contain)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        container(text("Loading preview…"))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
        .into()
}
