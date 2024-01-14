#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use eframe::egui;
use egui::{
    include_image, scroll_area::ScrollBarVisibility, Button, Color32, FontFamily, FontId, Image, Label, RichText, ScrollArea, Separator,
    TextEdit, ViewportBuilder,
};

use tokio::sync::mpsc::UnboundedReceiver;

use crate::{
    command::{BotCommand, CommandHandler},
    messages::{Platform, PlatformMessage},
};

pub fn run(command_handler: Arc<Mutex<CommandHandler>>, receiver: UnboundedReceiver<PlatformMessage>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Omnichatter")
            .with_min_inner_size([300.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OmniChatter",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::<OmniChatter>::new(OmniChatter::new(command_handler, receiver))
        }),
    )
}

#[derive(Clone)]
#[allow(dead_code)]
enum ToastLevel {
    Info,
    Warn,
    Error,
    Success,
}

#[derive(Clone)]
struct Toast {
    start: SystemTime,
    duration: Duration,
    msg: String,
    level: ToastLevel,
    font_id: FontId,
}

impl Toast {
    fn new(duration: Duration, msg: String, level: ToastLevel, font_id: FontId) -> Self {
        Self {
            start: SystemTime::now(),
            duration,
            msg,
            level,
            font_id,
        }
    }

    fn get_label(&self) -> Label {
        let text = RichText::new(&self.msg).font(self.font_id.clone());
        let text = match self.level {
            ToastLevel::Info => text,
            ToastLevel::Warn => text.color(Color32::LIGHT_YELLOW),
            ToastLevel::Error => text.color(Color32::LIGHT_RED).strong(),
            ToastLevel::Success => text.color(Color32::LIGHT_GREEN),
        };
        Label::new(text)
    }
}

enum State {
    DisplayCommand,
    CreateCommand,
    Idle,
    ChatFullScreen,
    Config,
}

struct OmniChatter {
    command_handler: Arc<Mutex<CommandHandler>>,
    current_command: BotCommand,
    toasts: Vec<Toast>,
    state: State,
    command_search: String,
    receiver: UnboundedReceiver<PlatformMessage>,
    platform_messages: Vec<PlatformMessage>,
    scrolling_chat: bool,
    text_size: f32,
}

impl OmniChatter {
    fn new(command_handler: Arc<Mutex<CommandHandler>>, receiver: UnboundedReceiver<PlatformMessage>) -> Self {
        Self {
            command_search: "".to_string(),
            command_handler,
            current_command: BotCommand {
                name: String::new(),
                contents: String::new(),
            },
            toasts: Vec::new(),
            state: State::Idle,
            receiver,
            platform_messages: Vec::new(),
            scrolling_chat: false,
            text_size: 12.,
        }
    }
}

impl eframe::App for OmniChatter {
    // Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: make it lock-safe aka not fail on lock error
        let mut command_handler = self.command_handler.lock().unwrap();

        // TODO?: Cloning this over and over doesn't seem right
        let font_id = FontId {
            size: self.text_size,
            family: FontFamily::Proportional,
        };

        match self.receiver.try_recv() {
            Ok(msg) => {
                self.platform_messages.push(msg);
            }
            Err(err) => match err {
                tokio::sync::mpsc::error::TryRecvError::Empty => {}
                tokio::sync::mpsc::error::TryRecvError::Disconnected => {
                    eprintln!("ERROR - should not happen");
                }
            },
        };

        if let State::ChatFullScreen = self.state {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                if ui.button(RichText::new("toggle fullscreen").font(font_id.clone())).clicked() {
                    self.state = match self.state {
                        // TODO: keep track of last state
                        State::ChatFullScreen => State::Idle,
                        _ => State::ChatFullScreen,
                    };
                }
                ScrollArea::vertical()
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        for PlatformMessage { msg, platform, sender } in &self.platform_messages {
                            ui.horizontal(|ui| {
                                let color = match platform {
                                    Platform::Youtube => Color32::RED,
                                    Platform::Twitch => Color32::from_rgb(191, 148, 255),
                                };
                                ui.label(RichText::new(" ").font(font_id.clone()).background_color(color));
                                let tmp =
                                    ui.add(Label::new(RichText::new(format!("[{}] {}", sender, msg)).font(font_id.clone())).wrap(true));
                                if !self.scrolling_chat {
                                    tmp.scroll_to_me(None);
                                }
                            });
                        }
                    });
                if self.scrolling_chat {
                    if ui.button(RichText::new("resume scrolling").font(font_id.clone())).clicked() {
                        self.scrolling_chat = true
                    }
                }
            });
        } else {
            // UI
            //
            //  ___________________________________________
            // |  Omnichatter                              |
            // |-------------------------------------------|
            // |  Command1*  | Command*          |chatMsg   |
            // |  Command2   |                   |chatMsg   |
            // |  ...        | Command_contents  |...       |
            // |-------------|-------------------|          |
            // |  Action1    | Info area         |          |
            // |  Action2    |                   |          |
            // |  ...        |                   |          |
            // |             |                   |          |
            //  -------------------------------------------
            egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 20.0;
                ui.label(RichText::new("Omnichatter").heading().font(font_id.clone()).strong())
            });

            egui::SidePanel::left("left_panel").resizable(false).show(ctx, |ui| {
                ui.spacing_mut().item_spacing.y = 5.0;
                ui.add(TextEdit::multiline(&mut self.command_search).font(font_id.clone()));
                for command_name in command_handler.get_command_names() {
                    if self.command_search.len() == 0 || command_name.starts_with(&self.command_search) {
                        if ui.button(RichText::new(&*command_name).font(font_id.clone())).clicked() {
                            self.command_search = String::new();
                            self.current_command = command_handler.get_command(&command_name);
                            self.state = State::DisplayCommand;
                        };
                    };
                }
                ui.add(Separator::default().horizontal());
                if ui.button(RichText::new("Create command").font(font_id.clone())).clicked() {
                    self.current_command = BotCommand {
                        name: String::new(),
                        contents: String::new(),
                    };
                    self.state = State::CreateCommand;
                };
                let config_button = Button::image(
                    Image::new(include_image!("../images/gear.png"))
                        .rounding(5.)
                        .max_width(32.)
                        .show_loading_spinner(true),
                );
                if ui.add(config_button).clicked() {
                    self.state = State::Config;
                }
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                // TODO: parametrize that and use manual positioning on Widgets
                ui.spacing_mut().item_spacing.x = 10.0;
                ui.spacing_mut().item_spacing.y = 10.0;
                ui.scope(|ui| match self.state {
                    State::Config => {
                        ui.add(egui::Slider::new(&mut self.text_size, (10.)..=40.).text(RichText::new("Font size").font(font_id.clone())));
                    }
                    State::DisplayCommand => {
                        ui.strong(RichText::new(&self.current_command.name).font(font_id.clone()));
                        let available_rect = ctx.available_rect();
                        let command_contents = TextEdit::multiline(&mut self.current_command.contents)
                            .font(font_id.clone())
                            .min_size([(available_rect.right() / 10.) * 7., available_rect.bottom() / 10.].into());
                        ui.add(command_contents);
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            if ui.button(RichText::new("Update").font(font_id.clone())).clicked() {
                                command_handler
                                    .update_command_content(&self.current_command.name, &self.current_command.contents)
                                    .expect("Command to exist");
                                self.toasts.push(Toast::new(
                                    Duration::from_secs_f32(1.25),
                                    format!("Command {} updated successfully", self.current_command.name),
                                    ToastLevel::Success,
                                    font_id.clone(),
                                ))
                            }
                            if ui
                                .add(Button::new(RichText::new("Delete").font(font_id.clone())).fill(Color32::from_rgb(94, 25, 25)))
                                .clicked()
                            {
                                command_handler
                                    .delete_command(&self.current_command.name)
                                    .expect("Command to be deleted");
                                self.toasts.push(Toast::new(
                                    Duration::from_secs_f32(1.25),
                                    format!("Command {} deleted successfully", self.current_command.name),
                                    ToastLevel::Success,
                                    font_id.clone(),
                                ));
                                self.state = State::Idle;
                            }
                        });
                    }
                    State::CreateCommand => {
                        ui.strong(RichText::new("Create command").font(font_id.clone()));
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            ui.label(RichText::new("name:").font(font_id.clone()));
                            ui.add(TextEdit::multiline(&mut self.current_command.name).font(font_id.clone()));
                        });
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                            ui.label(RichText::new("contents:").font(font_id.clone()));
                            ui.add(TextEdit::multiline(&mut self.current_command.contents).font(font_id.clone()));
                        });
                        if ui.button(RichText::new("Create").font(font_id.clone())).clicked() {
                            match command_handler.create_command(&self.current_command) {
                                Ok(_) => {
                                    self.toasts.push(Toast::new(
                                        Duration::from_secs_f32(2.5),
                                        format!("Command {} created successfully!", &self.current_command.name),
                                        ToastLevel::Success,
                                        font_id.clone(),
                                    ));
                                    self.state = State::DisplayCommand
                                }
                                Err(err) => self.toasts.push(Toast::new(
                                    Duration::from_secs_f32(2.5),
                                    format!("Error creating command: {:?}", err),
                                    ToastLevel::Error,
                                    font_id.clone(),
                                )),
                            }
                        }
                    }
                    State::Idle => {
                        ui.label(RichText::new("Select any command or action").font(font_id.clone()));
                    }
                    State::ChatFullScreen => {}
                });
                let now = SystemTime::now();
                // TODO: find a better approach without re-creating memory
                let mut toasts: Vec<Toast> = Vec::new();
                for toast in &self.toasts {
                    if now.duration_since(toast.start).unwrap() < toast.duration {
                        ui.add_sized([400., 400.], toast.get_label());
                        toasts.push(toast.clone());
                    }
                }
                self.toasts = toasts;
                //ui.image(egui::include_image!("../../../crates/egui/assets/ferris.png"));
            });
            egui::SidePanel::right("right_panel")
                .default_width(300.)
                .min_width(300.)
                .show(ctx, |ui| {
                    ui.spacing_mut().item_spacing.y = 5.0;
                    if ui.button(RichText::new("toggle fullscreen").font(font_id.clone())).clicked() {
                        self.state = State::ChatFullScreen
                    }
                    // TODO: set self.scrolling_chat to true when scroll detected
                    ScrollArea::vertical()
                        .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                        .show(ui, |ui| {
                            for PlatformMessage { msg, platform, sender } in &self.platform_messages {
                                ui.horizontal(|ui| {
                                    let color = match platform {
                                        Platform::Youtube => Color32::RED,
                                        Platform::Twitch => Color32::from_rgb(191, 148, 255),
                                    };
                                    ui.label(RichText::new(" ").background_color(color).font(font_id.clone()));
                                    let tmp =
                                        ui.add(Label::new(RichText::new(format!("[{}] {}", sender, msg)).font(font_id.clone())).wrap(true));
                                    if !self.scrolling_chat {
                                        tmp.scroll_to_me(None);
                                    }
                                });
                            }
                        });
                    if self.scrolling_chat {
                        if ui.button(RichText::new("resume scrolling").font(font_id.clone())).clicked() {
                            self.scrolling_chat = true
                        }
                    }
                });

            // TODO: Add a button to toggle fullscreen chat
        }
        // check if this needs to be limited to avoid any issues
        ctx.request_repaint()
    }
}

impl Into<String> for &PlatformMessage {
    fn into(self) -> String {
        match self.platform {
            Platform::Twitch => {
                RichText::new(" ")
                    .background_color(Color32::from_rgb(191, 148, 255))
                    .text()
                    .to_owned()
                    + &format!("[{}]", self.sender)
                    + &self.msg
            }
            Platform::Youtube => {
                RichText::new(" ").background_color(Color32::RED).text().to_owned() + &format!("[{}]", self.sender) + &self.msg
            }
        }
    }
}
