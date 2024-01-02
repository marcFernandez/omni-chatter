#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{
    fmt::format,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use eframe::egui;
use egui::Separator;

use crate::command::{BotCommand, CommandHandler};

#[derive(Clone)]
struct Toast {
    start: SystemTime,
    duration: Duration,
    msg: String,
}

impl Toast {
    fn new(duration: Duration, msg: String) -> Self {
        Self {
            start: SystemTime::now(),
            duration,
            msg,
        }
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
}

impl OmniChatter {
    fn new(command_handler: Arc<Mutex<CommandHandler>>) -> Self {
        Self {
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
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
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
        let command_names: Vec<String> = command_handler.get_command_names();

        // UI
        //
        //  _________________________________
        // |  Omnichatter                    |
        // |---------------------------------|
        // |             | Command*          |
        // |  Command1*  |                   |
        // |  Command2   | Command_contents  |
        // |  ...        |-------------------|
        // |             | Info area         |
        // |             |                   |
        // |             |                   |
        // |             |                   |
        //  ---------------------------------
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.heading("Omnichatter");
        });
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            for command_name in command_names {
                if ui.button(&command_name).clicked() {
                    self.current_command = command_handler.get_command(&command_name);
                    self.state = State::DisplayCommand;
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
            ui.scope(|ui| match self.state {
                State::DisplayCommand => {
                    ui.strong(&self.current_command.name);
                    ui.text_edit_multiline(&mut self.current_command.contents);
                    if ui.button("Update").clicked() {
                        command_handler
                            .update_command_content(&self.current_command.name, &self.current_command.contents)
                            .expect("Command to exist");
                        self.toasts.push(Toast::new(
                            Duration::from_secs_f32(1.25),
                            format!("Command {} updated successfully", self.current_command.name),
                        ))
                    }
                }
                State::CreateCommand => {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                        ui.label("name:");
                        ui.text_edit_singleline(&mut self.current_command.name);
                    });
                    ui.label("contents:");
                    ui.text_edit_multiline(&mut self.current_command.contents);
                    if ui.button("Create").clicked() {
                        match command_handler.create_command(&self.current_command) {
                            Ok(_) => self.toasts.push(Toast::new(
                                Duration::from_secs_f32(1.5),
                                format!("Command {} created successfully!", &self.current_command.name),
                            )),
                            Err(err) => self.toasts.push(Toast::new(Duration::from_secs_f32(2.5), format!("Error creating command: {:?}", err))),
                        }
                    }
                }
                State::Idle => {}
            });
            ui.add(Separator::default().horizontal());
            let now = SystemTime::now();
            // TODO: find a better approach without re-creating memory
            let mut toasts: Vec<Toast> = Vec::new();
            for toast in &self.toasts {
                if now.duration_since(toast.start).unwrap() < toast.duration {
                    ui.label(&toast.msg);
                    toasts.push(toast.clone());
                }
            }
            self.toasts = toasts;
            //ui.image(egui::include_image!("../../../crates/egui/assets/ferris.png"));
        });
    }
}
