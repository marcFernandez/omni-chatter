#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use eframe::egui;
use egui::{Color32, Label, RichText, Separator, TextEdit};

use crate::command::{BotCommand, CommandHandler};

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
}

impl Toast {
    fn new(duration: Duration, msg: String, level: ToastLevel) -> Self {
        Self {
            start: SystemTime::now(),
            duration,
            msg,
            level,
        }
    }

    fn get_label(&self) -> Label {
        let text = RichText::new(&self.msg);
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
}

struct OmniChatter {
    command_handler: Arc<Mutex<CommandHandler>>,
    current_command: BotCommand,
    toasts: Vec<Toast>,
    state: State,
    command_search: String,
}

impl OmniChatter {
    fn new(command_handler: Arc<Mutex<CommandHandler>>) -> Self {
        Self {
            command_search: "".to_string(),
            command_handler,
            current_command: BotCommand {
                name: String::new(),
                contents: String::new(),
            },
            toasts: Vec::new(),
            state: State::Idle,
        }
    }
}

pub fn run(command_handler: Arc<Mutex<CommandHandler>>) -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Omnichatter")
            .with_min_inner_size([600.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            //egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<OmniChatter>::new(OmniChatter::new(command_handler))
        }),
    )
}

impl eframe::App for OmniChatter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: make it lock-safe aka not fail on lock error
        let mut command_handler = self.command_handler.lock().unwrap();

        // UI
        //
        //  _________________________________
        // |  Omnichatter                    |
        // |---------------------------------|
        // |  Command1*  | Command*          |
        // |  Command2   |                   |
        // |  ...        | Command_contents  |
        // |-------------|-------------------|
        // |  Action1    | Info area         |
        // |  Action2    |                   |
        // |  ...        |                   |
        // |             |                   |
        //  ---------------------------------
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 20.0;
            ui.label(RichText::new("Omnichatter").heading().strong())
        });

        egui::SidePanel::left("left_panel").resizable(false).show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = 5.0;
            ui.text_edit_singleline(&mut self.command_search);
            for command_name in command_handler.get_command_names() {
                if self.command_search.len() == 0 || command_name.starts_with(&self.command_search) {
                    if ui.button(&*command_name).clicked() {
                        self.command_search = String::new();
                        self.current_command = command_handler.get_command(&command_name);
                        self.state = State::DisplayCommand;
                    };
                };
            }
            ui.add(Separator::default().horizontal());
            if ui.button("Create command").clicked() {
                self.current_command = BotCommand {
                    name: String::new(),
                    contents: String::new(),
                };
                self.state = State::CreateCommand;
            };
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: parametrize that and use manual positioning on Widgets
            ui.spacing_mut().item_spacing.x = 10.0;
            ui.spacing_mut().item_spacing.y = 10.0;
            ui.scope(|ui| match self.state {
                State::DisplayCommand => {
                    ui.strong(&self.current_command.name);
                    let available_rect = ctx.available_rect();
                    let command_contents = TextEdit::multiline(&mut self.current_command.contents)
                        .min_size([(available_rect.right() / 10.) * 7., available_rect.bottom() / 10.].into());
                    ui.add(command_contents);
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        if ui.button("Update").clicked() {
                            command_handler
                                .update_command_content(&self.current_command.name, &self.current_command.contents)
                                .expect("Command to exist");
                            self.toasts.push(Toast::new(
                                Duration::from_secs_f32(1.25),
                                format!("Command {} updated successfully", self.current_command.name),
                                ToastLevel::Success,
                            ))
                        }
                        if ui
                            .add(egui::Button::new("Delete").fill(Color32::from_rgb(94, 25, 25)))
                            .clicked()
                        {
                            command_handler
                                .delete_command(&self.current_command.name)
                                .expect("Command to be deleted");
                            self.toasts.push(Toast::new(
                                Duration::from_secs_f32(1.25),
                                format!("Command {} deleted successfully", self.current_command.name),
                                ToastLevel::Success,
                            ));
                            self.state = State::Idle;
                        }
                    });
                }
                State::CreateCommand => {
                    ui.strong("Create command");
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.label("name:");
                        ui.text_edit_singleline(&mut self.current_command.name);
                    });
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.label("contents:");
                        ui.text_edit_multiline(&mut self.current_command.contents);
                    });
                    if ui.button("Create").clicked() {
                        match command_handler.create_command(&self.current_command) {
                            Ok(_) => {
                                self.toasts.push(Toast::new(
                                    Duration::from_secs_f32(2.5),
                                    format!("Command {} created successfully!", &self.current_command.name),
                                    ToastLevel::Success,
                                ));
                                self.state = State::DisplayCommand
                            }
                            Err(err) => self.toasts.push(Toast::new(
                                Duration::from_secs_f32(2.5),
                                format!("Error creating command: {:?}", err),
                                ToastLevel::Error,
                            )),
                        }
                    }
                }
                State::Idle => {
                    ui.label("Select any command or action");
                }
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
    }
}
